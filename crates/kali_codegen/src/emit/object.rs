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

        // base = __heap; __heap += nfields * 8.
        function.instruction(&Instruction::GlobalGet(0));
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(scratch));
        function.instruction(&Instruction::GlobalGet(0));
        function.instruction(&Instruction::I32Const((fields.len() * 8) as i32));
        function.instruction(&Instruction::I32Add);
        function.instruction(&Instruction::GlobalSet(0));

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
