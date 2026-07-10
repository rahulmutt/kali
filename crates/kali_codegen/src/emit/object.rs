//! Runtime fixed-shape heap objects: bump-allocated headerless structs in
//! linear memory (field `i` at `base + i*8`), lowered type-directed off the
//! `Repr::Object(ShapeId)` entries the shape inference recorded. Object
//! literals with no table entry keep the compile-time fold lane in
//! `intrinsics/object.rs` untouched.
use crate::*;

impl<'a> FunctionEmitter<'a> {
    /// Static shape of the object reference produced by `id`, when known:
    /// a bare identifier whose binding repr is `Object(_)`, or a subscript
    /// `a[i]` of a registered array binding whose element repr is `Object(_)`.
    /// A field read of a scalar field returns `None` (nested objects are
    /// gated by the inference).
    pub(crate) fn object_shape_of_node(&self, id: LirNodeId) -> Option<kali_common::ShapeId> {
        let id = self.unwrap_transparent(id);
        let node = self.node(id);
        if node.kind != LirNodeKind::Value {
            return None;
        }
        if node.children.is_empty() {
            let name = node.text.as_deref()?;
            if let kali_common::Repr::Object(shape) = self.scalar_repr(name) {
                return Some(shape);
            }
            return None;
        }
        // Subscript `a[index]`: 1-child with the index in `text`, or 2-child
        // computed (non-operator text). Field reads have the same 1-child
        // shape but their base is not an array binding.
        if node.children.len() == 2
            && is_binary_operator_text(node.text.as_deref().unwrap_or_default())
        {
            return None;
        }
        if node.children.len() > 2 || node.text.as_deref().is_none_or(str::is_empty) {
            return None;
        }
        // `a.length` shares the same one-child `Value` shape (`text` = property
        // name) as a literal-index array read `a["length"]`-shaped node — see
        // the identical ambiguity note at `control_flow.rs`'s runtime-array-read
        // arm. `.length` is always a scalar (the array's element count), never
        // an object reference, so it must be excluded here ahead of the
        // array-subscript interpretation below.
        if node.children.len() == 1 && node.text.as_deref() == Some("length") {
            return None;
        }
        let base = node.children[0];
        let base_name = self.assignment_target_name(node, base)?;
        if !self.array_bindings.contains(&base_name) {
            return None;
        }
        if let kali_common::Repr::Object(shape) = self.array_elem_repr(&base_name) {
            return Some(shape);
        }
        None
    }

    /// Bump-allocate a fixed-layout object for `literal` (an object-literal
    /// LIR node) with layout `shape`, leaving the i64 base pointer on the
    /// stack. Field values are emitted in shape order via the literal's own
    /// field lookup, promoted to the field's repr.
    pub(crate) fn emit_object_allocation(
        &mut self,
        function: &mut Function,
        literal: &LirNode,
        shape: kali_common::ShapeId,
    ) -> EmittedValue {
        let scratch = self.locals.len() as u32;
        let fields = self.repr_table.shape_fields(shape).to_vec();

        // base = __alloc(nfields * 8), kept i64-extended in `scratch` exactly
        // as the old inline `__heap` bump did — only the pointer source
        // changed (the shared allocator instead of an inline global bump).
        function.instruction(&Instruction::I32Const((fields.len() * 8) as i32));
        function.instruction(&Instruction::Call(self.alloc_callee_index()));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(scratch));

        for (index, (name, repr)) in fields.iter().enumerate() {
            let Some(value_id) = self.object_literal_field(literal, name) else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "object literal is missing field '{name}' required by its inferred shape"
                    ),
                ));
                continue;
            };
            function.instruction(&Instruction::LocalGet(scratch));
            function.instruction(&Instruction::I32WrapI64);
            let produced = self.emit_node(function, value_id, true);
            let mem = MemArg {
                offset: (index * 8) as u64,
                align: 3,
                memory_index: 0,
            };
            match repr {
                kali_common::Repr::F64 => {
                    if !produced.produced {
                        function.instruction(&Instruction::F64Const(0.0.into()));
                    } else if !self.is_float_valued(value_id) {
                        function.instruction(&Instruction::F64ConvertI64S);
                    }
                    function.instruction(&Instruction::F64Store(mem));
                }
                _ => {
                    if !produced.produced {
                        function.instruction(&Instruction::I64Const(0));
                    }
                    function.instruction(&Instruction::I64Store(mem));
                }
            }
        }

        function.instruction(&Instruction::LocalGet(scratch));
        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
    }

    /// Spec 4a Task 5: emit a for-in-key identifier's ORDINAL (the raw `i64`
    /// counter in its reserved local), bypassing the STRING-VALUE materialization
    /// in `emit_value`'s identifier arm. Used at the INDEX sites (`table[c]`),
    /// where the key must stay the raw ordinal even when it is also materialized
    /// as a string elsewhere in the same loop (the dual-role crux). Falls back to
    /// `emit_node` if the index is not a bound identifier (defensive; the
    /// recognizer only ever passes a bound for-in-key identifier here).
    fn emit_forin_key_ordinal(&mut self, function: &mut Function, index: LirNodeId) {
        let index_id = self.unwrap_transparent(index);
        if let Some(name) = self.node(index_id).text.as_deref() {
            if let Some(&ord_local) = self.locals.get(name) {
                function.instruction(&Instruction::LocalGet(ord_local));
                return;
            }
        }
        let _ = self.emit_node(function, index, true);
    }

    /// Recognize a computed for-in-key object access `obj[c]` over a
    /// uniform-repr fixed shape: the base carries a known object shape, every
    /// field of that shape shares one repr, and the bracket index is a bare
    /// identifier bound to a `for..in` key over *that same* shape (recorded by
    /// `emit_for_in` in `for_in_key_shapes`). Returns
    /// `(base, index, element_repr)` for the dynamic slot lane.
    ///
    /// This is the structural codegen twin of the `kali_types`
    /// gate (`resolve/expression.rs`, `computed_forin_object_key_access`):
    /// both admit exactly `obj[c]` where `object_shape_of_*(obj) == Some(s)`,
    /// `shape_is_uniform_repr(s) == Some(_)`, and `for_in_key_shape(c) ==
    /// Some(s)`. Only the COMPUTED bracket form (2-child member node) matches;
    /// dot access `obj.c` (1-child) stays a static field read, and a string
    /// LITERAL key `obj["c"]` (index child is a `Literal`, not an identifier)
    /// stays static too — both are correct JS semantics.
    pub(crate) fn computed_forin_object_access(
        &self,
        node: &LirNode,
    ) -> Option<(LirNodeId, LirNodeId, kali_common::Repr)> {
        if node.kind != LirNodeKind::Value || node.children.len() != 2 {
            return None;
        }
        // A binary expression also lowers to a 2-child `Value` node; its
        // operator `text` cleanly separates it from a computed member access.
        if is_binary_operator_text(node.text.as_deref().unwrap_or_default()) {
            return None;
        }
        let base_id = node.children[0];
        let shape = self.object_shape_of_node(base_id)?;
        let elem = self.repr_table.shape_is_uniform_repr(shape)?;
        // The index must be a bare identifier (not a string/number literal key)
        // that this function registered as a `for..in` key over `shape`.
        let index_id = self.unwrap_transparent(node.children[1]);
        let index_node = self.node(index_id);
        if index_node.kind != LirNodeKind::Value || !index_node.children.is_empty() {
            return None;
        }
        let name = index_node.text.as_deref().filter(|t| !t.is_empty())?;
        if self.for_in_key_shapes.get(name) != Some(&shape) {
            return None;
        }
        Some((base_id, node.children[1], elem))
    }

    /// Dynamic computed read `obj[c]` over a uniform-repr headerless object:
    /// address `base + index*8`, typed load at `offset: 0` (objects carry NO
    /// length header, unlike arrays whose elements sit at `offset: 8`).
    pub(crate) fn emit_object_field_read_dynamic(
        &mut self,
        function: &mut Function,
        base: LirNodeId,
        index: LirNodeId,
        elem_repr: kali_common::Repr,
    ) -> EmittedValue {
        // address: base (i32) + index*8
        let produced = self.emit_node(function, base, true);
        if !produced.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::I32WrapI64);
        self.emit_forin_key_ordinal(function, index);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(8));
        function.instruction(&Instruction::I32Mul);
        function.instruction(&Instruction::I32Add);
        let memarg = MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        };
        match elem_repr {
            kali_common::Repr::F64 => function.instruction(&Instruction::F64Load(memarg)),
            _ => function.instruction(&Instruction::I64Load(memarg)),
        };
        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
    }

    /// Dynamic computed store `obj[c] = v` over a uniform-repr headerless
    /// object: address `base + index*8`, then the value (promoted to the
    /// element repr the same way the static field store does), then a typed
    /// store at `offset: 0`. Leaves the stored value on the stack as the
    /// assignment expression's result (reloaded from the same address).
    pub(crate) fn emit_object_field_write_dynamic(
        &mut self,
        function: &mut Function,
        base: LirNodeId,
        index: LirNodeId,
        value: LirNodeId,
        elem_repr: kali_common::Repr,
    ) -> EmittedValue {
        let scratch = self.locals.len() as u32;
        // address: base (i32) + index*8, saved (i64-extended) so it can be
        // reused for the result reload — `scratch` is an i64 local.
        let produced = self.emit_node(function, base, true);
        if !produced.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::I32WrapI64);
        self.emit_forin_key_ordinal(function, index);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(8));
        function.instruction(&Instruction::I32Mul);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalTee(scratch));
        function.instruction(&Instruction::I32WrapI64);
        let memarg = MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        };
        match elem_repr {
            kali_common::Repr::F64 => {
                let rhs = self.emit_node(function, value, true);
                if !rhs.produced {
                    function.instruction(&Instruction::F64Const(0.0.into()));
                } else if !self.is_float_valued(value) {
                    function.instruction(&Instruction::F64ConvertI64S);
                }
                function.instruction(&Instruction::F64Store(memarg));
                function.instruction(&Instruction::LocalGet(scratch));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::F64Load(memarg));
            }
            _ => {
                let rhs = self.emit_node(function, value, true);
                if !rhs.produced {
                    function.instruction(&Instruction::I64Const(0));
                }
                function.instruction(&Instruction::I64Store(memarg));
                function.instruction(&Instruction::LocalGet(scratch));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I64Load(memarg));
            }
        }
        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
    }

    /// Compound-assign `obj[c] op= rhs` over a uniform-repr headerless object
    /// (Spec 4a Task 4): the decompose of `obj[c] = (obj[c] op rhs)` where BOTH
    /// the read of `obj[c]` and the write route through the same dynamic slot
    /// (`base + index*8`, offset 0) so the address is computed once and reused.
    /// The op applies at the element repr (F64 for a uniform-float shape).
    /// Leaves the stored value on the stack (reloaded) as the expression result.
    /// `%=`/`**=` on a float shape have no lowering — reject fail-closed.
    pub(crate) fn emit_object_field_compound_assign_dynamic(
        &mut self,
        function: &mut Function,
        base: LirNodeId,
        index: LirNodeId,
        rhs: LirNodeId,
        elem_repr: kali_common::Repr,
        op: &str,
    ) -> EmittedValue {
        let scratch = self.locals.len() as u32;
        // address: base (i32) + index*8, saved (i64-extended) in `scratch` so
        // the same slot backs the current-value load, the store, and the result
        // reload — `scratch` is an i64 local.
        let produced = self.emit_node(function, base, true);
        if !produced.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::I32WrapI64);
        self.emit_forin_key_ordinal(function, index);
        function.instruction(&Instruction::I32WrapI64);
        function.instruction(&Instruction::I32Const(8));
        function.instruction(&Instruction::I32Mul);
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalTee(scratch));
        function.instruction(&Instruction::I32WrapI64); // store address on stack
        let memarg = MemArg {
            offset: 0,
            align: 3,
            memory_index: 0,
        };
        match elem_repr {
            kali_common::Repr::F64 => {
                if matches!(op, "%=" | "**=") {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "compound assignment '{op}' on a floating-point for-in-key object field is unavailable in the current phase"
                        ),
                    ));
                    function.instruction(&Instruction::Drop);
                    function.instruction(&Instruction::F64Const(0.0.into()));
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    };
                }
                // current value: obj[c]
                function.instruction(&Instruction::LocalGet(scratch));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::F64Load(memarg));
                // rhs, promoted to f64
                let r = self.emit_node(function, rhs, true);
                if !r.produced {
                    function.instruction(&Instruction::F64Const(0.0.into()));
                } else if !self.is_float_valued(rhs) {
                    function.instruction(&Instruction::F64ConvertI64S);
                }
                match op {
                    "+=" => function.instruction(&Instruction::F64Add),
                    "-=" => function.instruction(&Instruction::F64Sub),
                    "*=" => function.instruction(&Instruction::F64Mul),
                    "/=" => function.instruction(&Instruction::F64Div),
                    _ => unreachable!(),
                };
                function.instruction(&Instruction::F64Store(memarg));
                function.instruction(&Instruction::LocalGet(scratch));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::F64Load(memarg));
            }
            _ => {
                // `**=` on an integer field would lower via Math.pow (f64) and
                // then `I64Store` — a stack-type mismatch (invalid module), not a
                // meaningful lowering. Reject fail-closed, symmetric to the F64
                // branch's `%=`/`**=` rejection. (`%=` via `I64RemS` is fine.)
                if op == "**=" {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        "compound assignment '**=' on an integer for-in-key object field is unavailable in the current phase".to_string(),
                    ));
                    function.instruction(&Instruction::Drop);
                    function.instruction(&Instruction::I64Const(0));
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    };
                }
                // current value: obj[c]
                function.instruction(&Instruction::LocalGet(scratch));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I64Load(memarg));
                let r = self.emit_node(function, rhs, true);
                if !r.produced {
                    function.instruction(&Instruction::I64Const(0));
                }
                match op {
                    "+=" => function.instruction(&Instruction::I64Add),
                    "-=" => function.instruction(&Instruction::I64Sub),
                    "*=" => function.instruction(&Instruction::I64Mul),
                    "/=" => function.instruction(&Instruction::I64DivS),
                    "%=" => function.instruction(&Instruction::I64RemS),
                    _ => unreachable!(),
                };
                function.instruction(&Instruction::I64Store(memarg));
                function.instruction(&Instruction::LocalGet(scratch));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::I64Load(memarg));
            }
        }
        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
    }

    /// `<base>.field` read on a shaped base: typed load at the field's static
    /// offset. Unknown fields are gated, never miscompiled.
    pub(crate) fn emit_object_field_read(
        &mut self,
        function: &mut Function,
        base: LirNodeId,
        shape: kali_common::ShapeId,
        field: &str,
    ) -> EmittedValue {
        let Some((index, repr)) = self.repr_table.shape_field(shape, field) else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "unknown field '{field}' on a fixed-shape object; only declared fields are available"
                ),
            ));
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        };
        let produced = self.emit_node(function, base, true);
        if !produced.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::I32WrapI64);
        let mem = MemArg {
            offset: (index * 8) as u64,
            align: 3,
            memory_index: 0,
        };
        match repr {
            kali_common::Repr::F64 => function.instruction(&Instruction::F64Load(mem)),
            _ => function.instruction(&Instruction::I64Load(mem)),
        };
        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
    }
}

#[cfg(test)]
#[path = "object_tests.rs"]
mod object_tests;
