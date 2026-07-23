use crate::*;

/// Source of a dynamic array element index for `a[...] = v` writes: either a
/// stringified literal/identifier (`text`) or a structured computed-index node.
enum ArrayWriteIndex {
    Text(String),
    Node(LirNodeId),
}

impl<'a> FunctionEmitter<'a> {
    pub(crate) fn emit_aggregate_literal(
        &mut self,
        function: &mut Function,
        node: &LirNode,
        _want_value: bool,
    ) -> EmittedValue {
        if self.is_object_literal(node) {
            for child in &node.children {
                let property = self.node(*child).clone();
                if property.children.len() != 2 {
                    continue;
                }
                let value = property.children[1];
                let produced = self.emit_node(function, value, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
            }
        } else {
            for child in &node.children {
                let produced = self.emit_node(function, *child, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
            }
        }

        function.instruction(&Instruction::I64Const(0));
        EmittedValue {
            produced: true,
            shape: ValueShape::Unknown,
        }
    }

    pub(crate) fn resolve_literal_aggregate(&self, mut id: LirNodeId) -> Option<LirNodeId> {
        let mut seen = HashSet::new();
        loop {
            if !seen.insert(id.0) {
                return None;
            }

            let node = self.node(id);
            if node.kind == LirNodeKind::Value
                && node.text.as_deref().is_some_and(|text| text.is_empty())
                && !node.children.is_empty()
            {
                id = *node.children.last().expect("sequence wrapper has a child");
                continue;
            }

            if node.kind == LirNodeKind::Value
                && node.children.is_empty()
                && node.text.as_deref().is_some()
            {
                let name = node.text.as_deref()?;
                if let Some(bound) = self.bindings.get(name).copied() {
                    id = bound;
                    continue;
                }
            }

            if self.is_object_freeze_call(node) {
                let argument = node.children.get(1).copied()?;
                id = argument;
                continue;
            }

            if self.is_frozen_array_from_call(node) {
                let argument = node.children.get(1).copied()?;
                id = argument;
                continue;
            }

            if self.is_array_from_call(node) {
                let argument = node.children.get(1).copied()?;
                id = argument;
                continue;
            }

            if let Some(argument) = self.resolve_identity_array_callback_source(node) {
                id = argument;
                continue;
            }

            if node.kind == LirNodeKind::Value && node.children.len() == 2 {
                match node.text.as_deref() {
                    Some("??") => {
                        let left = self.resolve_static_object_identity_value(node.children[0])?;
                        id = if left.is_nullish() {
                            node.children[1]
                        } else {
                            node.children[0]
                        };
                        continue;
                    }
                    Some("&&") => {
                        let left = self.resolve_static_object_identity_value(node.children[0])?;
                        match left.truthiness() {
                            Some(true) => {
                                id = node.children[1];
                                continue;
                            }
                            Some(false) => {
                                id = node.children[0];
                                continue;
                            }
                            None => {
                                let left_aggregate =
                                    self.resolve_literal_aggregate(node.children[0]);
                                let right_aggregate =
                                    self.resolve_literal_aggregate(node.children[1]);
                                if left_aggregate.is_some() && left_aggregate == right_aggregate {
                                    id = left_aggregate?;
                                    continue;
                                }
                            }
                        }
                    }
                    Some("||") => {
                        let left = self.resolve_static_object_identity_value(node.children[0])?;
                        match left.truthiness() {
                            Some(true) => {
                                id = node.children[0];
                                continue;
                            }
                            Some(false) => {
                                id = node.children[1];
                                continue;
                            }
                            None => {
                                let left_aggregate =
                                    self.resolve_literal_aggregate(node.children[0]);
                                let right_aggregate =
                                    self.resolve_literal_aggregate(node.children[1]);
                                if left_aggregate.is_some() && left_aggregate == right_aggregate {
                                    id = left_aggregate?;
                                    continue;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            // A static index member `arr[k]` whose base resolves to an array
            // literal resolves to the indexed ELEMENT node. This lets a nested
            // index (`arr[i][j]`) fold against the inner literal — the shape a
            // folded `Object.entries(obj)` produces (an array of `[key, value]`
            // 2-tuple literals), where `es[i][0]` / `es[i][1]` must read the
            // real key/value, not a runtime-array placeholder. Only `Node`
            // results are aggregates and continue; `String`/`Undefined`
            // elements are terminal and fall through to the return below.
            if let Some(StaticIndexMemberResult::Node(inner)) =
                self.resolve_static_index_member(node)
            {
                id = inner;
                continue;
            }

            return Some(id);
        }
    }

    /// True when `id` resolves (through transparent wrappers) to a nullish
    /// expression: the `null`/`undefined` LITERAL forms, or the bare
    /// identifier `undefined` (which parses as an Identifier → Value node —
    /// the form the old literal-only recognizer missed, storing ordinal 0
    /// instead of the -1 sentinel: wrong truthiness). Single recognizer for
    /// BOTH the types-side admit and the codegen stores, so the twins cannot
    /// disagree on nullish-ness again (the `??= undefined` reject existed
    /// only because of that disagreement). Used by the Spec 4a Task 4
    /// null-sentinel store: a nullish init/reassignment of a for-in-key alias
    /// stores `-1`, not `0`.
    pub(crate) fn is_null_or_undefined_expr(&self, id: LirNodeId) -> bool {
        let node = self.node(self.unwrap_transparent(id));
        if node.kind == LirNodeKind::Literal {
            return matches!(node.text.as_deref(), Some("null") | Some("undefined"));
        }
        self.bare_identifier_name(id).as_deref() == Some("undefined")
    }

    pub(crate) fn assignment_target_name(&self, _node: &LirNode, id: LirNodeId) -> Option<String> {
        let mut current = id;
        loop {
            let current_node = self.node(current);
            if current_node.kind == LirNodeKind::Value
                && current_node.children.len() == 1
                && match current_node.text.as_deref() {
                    None => true,
                    Some(text) => text.is_empty(),
                }
            {
                current = current_node.children[0];
                continue;
            }
            return (current_node.children.is_empty())
                .then(|| current_node.text.clone())
                .flatten();
        }
    }

    pub(crate) fn emit_assignment(
        &mut self,
        function: &mut Function,
        id: LirNodeId,
        node: &LirNode,
        op: &str,
        left: LirNodeId,
        right: LirNodeId,
    ) -> bool {
        if !matches!(
            op,
            "=" | "??=" | "&&=" | "||=" | "+=" | "-=" | "*=" | "/=" | "%=" | "**="
        ) {
            return false;
        }

        // Stage P5 T-new-A (review finding I-3): storing a
        // `crypto.getRandomValues(...)` result into an AGGREGATE slot
        // (`o.buf = fb`, `holder[0] = fb`) launders the handle out of the
        // name-keyed deny domain — the later read's receiver has no binding
        // name, so every gate in this lane misses it and the read diverges
        // silently (measured `0` / `2` where node reads `4`). A store to a bare
        // identifier (childless target node) is NOT affected: that target keeps
        // its name, and `record_crypto_random_result_binding` below re-derives
        // its provenance. Placed first so no other arm can claim the shape.
        if !self.node(left).children.is_empty() && self.is_crypto_random_result_value(right) {
            self.deny_e5506(function, Self::CRYPTO_RANDOM_RESULT_STORE_DENY);
            return true;
        }

        // Stage P5 T-new-C review M-1: the WRITE position of an event marker
        // member (`e.type = 'z'`, `e.type += 'z'`, `e.bubbles = true`). The
        // `.type` lane is read-only — its value is a compile-time literal
        // materialized on each read — so a store has nowhere to land; before
        // this arm it fell out of the lane entirely and was silently dropped
        // (kali printed the ORIGINAL `tick`, which happens to match node's
        // sloppy-mode no-op but diverges under ESM/strict, where node throws
        // `TypeError: Cannot assign to read only property 'type'`). Deny; never
        // silently discard a store. The cross-scope twins are covered too: an
        // enclosing-scope marker has no side-table entry here, so it is denied
        // on the same evidence the read arm uses.
        {
            let left_node = self.node(left).clone();
            if let Some(base_name) = self.bare_member_receiver_name(&left_node) {
                if self.is_event_marker(&base_name)
                    || self.is_module_scope_event_marker(&base_name)
                    || self.is_captured_event_marker(&base_name)
                {
                    self.deny_e5506(
                        function,
                        "assigning to a property of an Event/CustomEvent is not supported in \
                         the current phase (the event's `type` is a read-only compile-time \
                         value; a store would be silently dropped; fail-closed)",
                    );
                    return true;
                }
            }
        }

        // Stage P5 T-new-E: a bare-identifier reassignment `s = String(1n)`
        // makes `s` hold a String() coercion result whose repr stays `I64`
        // (F-newB-1). The provenance is now computed STRUCTURALLY in
        // `repr_infer`'s whole-program taint (`string_result_bindings`), which
        // sees the reassignment through its `visit_assignment` hook, so no
        // codegen-side recording is needed here — the render/arithmetic sinks
        // query the repr_infer taint directly.

        if op == "=" {
            if let Some(key_text) = process_env_property_key(&self.program.nodes, left) {
                let right_node = self.node(right);
                if right_node.kind == LirNodeKind::Value
                    && right_node.text.is_none()
                    && right_node.children.len() != 1
                {
                    self.diagnostics.push(Diagnostic::warning(
                        e8::UNIMPLEMENTED as u32,
                        "process.env property mutation is unavailable unless the assigned value is a statically-known literal in the current phase".to_string(),
                    ));
                    let produced = self.emit_node(function, right, true);
                    if produced.produced {
                        function.instruction(&Instruction::Drop);
                    }
                    function.instruction(&Instruction::I64Const(0));
                    return true;
                }

                let Some(value_text) = self.render_static_value(right) else {
                    self.diagnostics.push(Diagnostic::warning(
                        e8::UNIMPLEMENTED as u32,
                        "process.env property mutation is unavailable unless the assigned value is a statically-known literal in the current phase".to_string(),
                    ));
                    let produced = self.emit_node(function, right, true);
                    if produced.produced {
                        function.instruction(&Instruction::Drop);
                    }
                    function.instruction(&Instruction::I64Const(0));
                    return true;
                };

                let Some(import_index) = self.env_set_import_index else {
                    return false;
                };

                let (key_offset, key_len) = self.strings.intern(&key_text);
                let (value_offset, value_len) = self.strings.intern(&value_text);
                function.instruction(&Instruction::I32Const(key_offset as i32));
                function.instruction(&Instruction::I32Const(key_len as i32));
                function.instruction(&Instruction::I32Const(value_offset as i32));
                function.instruction(&Instruction::I32Const(value_len as i32));
                function.instruction(&Instruction::Call(import_index));
                function.instruction(&Instruction::Drop);
                function.instruction(&Instruction::I64Const(0));
                return true;
            }
        }

        // Fixed-shape object field store: `<base>.field = v` (including
        // through an array element: `bodies[0].vx = v`). Must precede the
        // array-write path: both lower as a 1-child member node, but here the
        // BASE (not the whole target) carries the object shape.
        if op == "=" {
            let left_node = self.node(left).clone();
            if left_node.kind == LirNodeKind::Value && left_node.children.len() == 1 {
                if let Some(field) = left_node.text.clone().filter(|text| !text.is_empty()) {
                    let base_id = left_node.children[0];
                    if let Some(shape) = self.object_shape_of_node(base_id) {
                        let Some((index, repr)) = self.repr_table.shape_field(shape, &field) else {
                            self.diagnostics.push(Diagnostic::error(
                                e5::FEATURE_UNAVAILABLE as u32,
                                format!(
                                    "unknown field '{field}' on a fixed-shape object; only declared fields can be assigned"
                                ),
                            ));
                            function.instruction(&Instruction::I64Const(0));
                            return true;
                        };
                        // Stage P2 review C-1a (silent-corruption close):
                        // reassigning a `GrowableArrayI64` field (`o.values =
                        // [4,5]`) has no sound lowering this phase — the generic
                        // `_ =>` store arm below would `I64Store` a non-handle
                        // over the valid tagged handle (then `o.values.join`
                        // prints empty). Deny is the sound minimal close (no
                        // re-seeding through `emit_growable_field_value` this
                        // wave). Reject BEFORE emitting base/RHS so the value
                        // stack stays balanced (single `I64Const(0)` result).
                        if matches!(repr, kali_common::Repr::GrowableArrayI64) {
                            self.diagnostics.push(Diagnostic::error(
                                e5::FEATURE_UNAVAILABLE as u32,
                                format!(
                                    "reassigning growable-array field '{field}' is unavailable in the current phase"
                                ),
                            ));
                            function.instruction(&Instruction::I64Const(0));
                            return true;
                        }
                        let scratch = self.locals.len() as u32;
                        let produced = self.emit_node(function, base_id, true);
                        if !produced.produced {
                            function.instruction(&Instruction::I64Const(0));
                        }
                        function.instruction(&Instruction::LocalTee(scratch));
                        function.instruction(&Instruction::I32WrapI64);
                        let mem = MemArg {
                            offset: (index * 8) as u64,
                            align: 3,
                            memory_index: 0,
                        };
                        let rhs = self.emit_node(function, right, true);
                        match repr {
                            kali_common::Repr::F64 => {
                                if !rhs.produced {
                                    function.instruction(&Instruction::F64Const(0.0.into()));
                                } else if !self.is_float_valued(right) {
                                    function.instruction(&Instruction::F64ConvertI64S);
                                }
                                function.instruction(&Instruction::F64Store(mem));
                                // Assignment expression result: reload the field.
                                function.instruction(&Instruction::LocalGet(scratch));
                                function.instruction(&Instruction::I32WrapI64);
                                function.instruction(&Instruction::F64Load(mem));
                            }
                            _ => {
                                if !rhs.produced {
                                    function.instruction(&Instruction::I64Const(0));
                                }
                                function.instruction(&Instruction::I64Store(mem));
                                function.instruction(&Instruction::LocalGet(scratch));
                                function.instruction(&Instruction::I32WrapI64);
                                function.instruction(&Instruction::I64Load(mem));
                            }
                        }
                        return true;
                    }
                }
            }
        }

        // Stage P3 Task 4 (alongside the C-1 field-store gate): a WRITE whose
        // TARGET is a member of a proven abort handle has no sound lowering —
        // node ignores `.aborted = x`/`.signal = x` silently (or throws in strict
        // mode), and the generic store below would `I64Store` over the shared
        // cell handle. Fail closed for `c.aborted = v`, `c.signal = v`, and
        // `c.signal.aborted = v` (base is `<ident>` or `<ident>.signal` over an
        // abort handle). Reject BEFORE emitting base/RHS so the value stack stays
        // balanced (single `I64Const(0)` result).
        if op == "=" {
            let left_node = self.node(left).clone();
            if left_node.kind == LirNodeKind::Value
                && left_node.children.len() == 1
                && left_node
                    .text
                    .as_deref()
                    .is_some_and(|text| !text.is_empty())
            {
                let base_id = left_node.children[0];
                let base = self.node(base_id);
                let base_is_handle_ident = base.children.is_empty()
                    && base
                        .text
                        .as_deref()
                        .is_some_and(|name| self.is_abort_handle(name));
                let base_is_signal_of_handle = base.kind == LirNodeKind::Value
                    && base.children.len() == 1
                    && base.text.as_deref() == Some("signal")
                    && {
                        let inner = self.node(base.children[0]);
                        inner.children.is_empty()
                            && inner
                                .text
                                .as_deref()
                                .is_some_and(|name| self.is_abort_handle(name))
                    };
                if base_is_handle_ident || base_is_signal_of_handle {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        "writes to AbortController/AbortSignal members are not supported \
                         (node ignores them silently; kali fails closed)"
                            .to_string(),
                    ));
                    function.instruction(&Instruction::I64Const(0));
                    return true;
                }
            }
        }

        // Stage P4 Task 6 (enumeration-wave close): the URL/USP twin of the
        // abort member-write gate above. A WRITE whose TARGET is a member of a
        // proven URL/URLSearchParams handle (`u.pathname = "/x"`, `q.size = 1`,
        // `u.searchParams.x = v`) has no sound lowering — the parsed arena
        // struct is compile-time-immutable in this phase, and every recognizer
        // below misses the shape, so the store previously fell through to a
        // SILENTLY DROPPED write (compiled green, did nothing — the Task-6 wave
        // leak). node would re-parse/reflect the mutation; kali fails closed.
        // Keyed on positive URL/USP base provenance (per-emitter local sets +
        // the module-scope and captured cross-function twins) — an allowlist-
        // style proof at the single member-write choke, not a per-sink denylist.
        // Reject BEFORE emitting base/RHS so the value stack stays balanced.
        if op == "=" {
            let left_node = self.node(left).clone();
            if left_node.kind == LirNodeKind::Value
                && left_node.children.len() == 1
                && left_node
                    .text
                    .as_deref()
                    .is_some_and(|text| !text.is_empty())
            {
                let base_id = left_node.children[0];
                let base = self.node(base_id);
                let base_is_url_handle_ident = base.children.is_empty()
                    && base.text.as_deref().is_some_and(|name| {
                        self.is_url(name)
                            || self.is_url_search_params(name)
                            || self.is_module_scope_url_handle(name)
                            || self.is_captured_url_handle(name)
                    });
                // `u.<component>.x = v` (e.g. `u.searchParams.sorted = 1`):
                // the base is itself a recognized URL member read.
                let base_is_url_member = self.url_member_read_parts(base_id).is_some();
                if base_is_url_handle_ident || base_is_url_member {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        "writes to URL/URLSearchParams members are not supported in the \
                         current phase (the parsed URL is compile-time-immutable; kali \
                         fails closed)"
                            .to_string(),
                    ));
                    function.instruction(&Instruction::I64Const(0));
                    return true;
                }
            }
        }

        // Stage P2 review C-1b (+ R1 rider): a WRITE through a `GrowableArrayI64`
        // object field has no sound lowering this phase — the growable field lane
        // is read-only except for `.push`. Two write shapes fall through every
        // recognizer below and are silently DROPPED, while their NAMED twins
        // already fail closed:
        //   * element write `o.values[0] = 9` (1-child literal/identifier index,
        //     or 2-child computed index) — named twin `a[0] = 9` E5506s;
        //   * `.length` write `o.values.length = 1` (1-child `length` member) —
        //     named twin `a.length = 1` E5506s; node TRUNCATES, so a dropped
        //     store is a silent miscompile (R1).
        // Both are keyed on the positive `object_field_is_growable_array` base
        // proof (allowlist) → fail closed E5506, never a dropped store.
        if op == "=" {
            let left_node = self.node(left).clone();
            let target_shape = match left_node.children.len() {
                // 1-child dot/subscript with an index or `length` in `text`.
                1 => left_node
                    .text
                    .as_deref()
                    .filter(|text| !text.is_empty())
                    .map(|text| {
                        if text == "length" {
                            "length"
                        } else {
                            "element"
                        }
                    }),
                // 2-child computed index (`o.values[i] = v`); binary operators
                // are not a member write.
                2 if !is_binary_operator_text(left_node.text.as_deref().unwrap_or_default()) => {
                    Some("element")
                }
                _ => None,
            };
            if let Some(kind) = target_shape {
                if self.object_field_is_growable_array(left_node.children[0]) {
                    let message = if kind == "length" {
                        "assigning to `.length` of a growable-array field is unavailable in the current phase"
                    } else {
                        "assigning to an element of a growable-array field is unavailable in the current phase"
                    };
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        message.to_string(),
                    ));
                    function.instruction(&Instruction::I64Const(0));
                    return true;
                }
            }
        }

        // Computed for-in-key object write `obj[c] = v` over a uniform-repr
        // fixed shape (Spec 4a Task 3): a dynamic headerless field slot store
        // at `base + c*8`, offset 0. Must precede the array-write path below —
        // both lower as a 2-child computed member, but here the base is an
        // object (never an array binding) and the index is the loop ordinal.
        // The static-field store arm above only matches the 1-child dot form,
        // so the computed bracket form falls through to here.
        if op == "=" {
            let left_node = self.node(left).clone();
            if let Some((base, index, elem)) = self.computed_forin_object_access(&left_node) {
                self.emit_object_field_write_dynamic(function, base, index, right, elem);
                return true;
            }
        }

        // Dynamic array element write: `a[i] = v` where `a` is a linear-memory
        // array. Literal/identifier indices lower to a 1-child member node with
        // the index in `text`; computed indices (`a[r - 1] = v`) lower to a
        // 2-child member node with the index expression in `children[1]`.
        if op == "=" {
            let left_node = self.node(left).clone();
            if left_node.kind == LirNodeKind::Value {
                let target = match left_node.children.len() {
                    1 => left_node
                        .text
                        .clone()
                        .filter(|index_text| !index_text.is_empty())
                        .map(|index_text| {
                            (left_node.children[0], ArrayWriteIndex::Text(index_text))
                        }),
                    2 if !is_binary_operator_text(
                        left_node.text.as_deref().unwrap_or_default(),
                    ) =>
                    {
                        Some((
                            left_node.children[0],
                            ArrayWriteIndex::Node(left_node.children[1]),
                        ))
                    }
                    _ => None,
                };

                if let Some((base_id, index)) = target {
                    if let Some(base_name) = self.assignment_target_name(node, base_id) {
                        if self.array_bindings.contains(&base_name) {
                            let scratch = self.locals.len() as u32;
                            match index {
                                ArrayWriteIndex::Text(index_text) => {
                                    self.emit_array_element_address(function, base_id, &index_text)
                                }
                                ArrayWriteIndex::Node(index_id) => self
                                    .emit_array_element_address_node(function, base_id, index_id),
                            }
                            match self.array_elem_repr(&base_name) {
                                kali_common::Repr::F64 => {
                                    // Stack: [address:i32]. `scratch` is always i64-typed
                                    // (see `lower.rs`'s two trailing i64 scratch locals), so
                                    // it cannot tee the i32 address or an f64 RHS directly.
                                    // Extend the address to i64 to tee it, then wrap back to
                                    // i32 to use it as a memory address; after the store,
                                    // reload from the same address to recover the stored
                                    // value as the assignment expression's result.
                                    function.instruction(&Instruction::I64ExtendI32U);
                                    function.instruction(&Instruction::LocalTee(scratch));
                                    function.instruction(&Instruction::I32WrapI64);
                                    let rhs = self.emit_node(function, right, true);
                                    if !rhs.produced {
                                        function.instruction(&Instruction::I64Const(0));
                                    }
                                    if !rhs.produced || !self.is_float_valued(right) {
                                        function.instruction(&Instruction::F64ConvertI64S);
                                    }
                                    function.instruction(&Instruction::F64Store(MemArg {
                                        offset: 8,
                                        align: 3,
                                        memory_index: 0,
                                    }));
                                    function.instruction(&Instruction::LocalGet(scratch));
                                    function.instruction(&Instruction::I32WrapI64);
                                    function.instruction(&Instruction::F64Load(MemArg {
                                        offset: 8,
                                        align: 3,
                                        memory_index: 0,
                                    }));
                                }
                                // Spec 3 activates the `String` case here: a
                                // proven string-element store lowers through the
                                // same i64-slot path (the value is a tagged string
                                // handle), no store-side change needed. A
                                // growable-array element (repr inference does not
                                // yet produce this for an array element, but the
                                // match must stay exhaustive) is likewise a
                                // tagged i64 handle into its header, so it stores
                                // through the same slot unchanged.
                                kali_common::Repr::I64
                                | kali_common::Repr::Object(_)
                                | kali_common::Repr::String
                                | kali_common::Repr::GrowableArrayI64
                                // AbortHandle: i64 handle slot; never reaches
                                // this position (inference gates it).
                                | kali_common::Repr::AbortHandle
                                // Url/UrlSearchParams: i64 handle slots (Stage
                                // P4); never reach this position yet (nothing
                                // seeds them into array elements) — grouped
                                // with the other i64 handles for exhaustiveness.
                                | kali_common::Repr::Url
                                | kali_common::Repr::UrlSearchParams
                                // Bytes: TextEncoder byte-buffer handle (Stage
                                // P5); never reaches this position yet (nothing
                                // seeds it into array elements) — grouped with
                                // the other i64 handles for exhaustiveness.
                                | kali_common::Repr::Bytes
                                // Event: a compile-time marker (Stage P5);
                                // never reaches this position (a store of an
                                // event marker is denied at the identifier
                                // choke) — grouped for exhaustiveness.
                                | kali_common::Repr::Event => {
                                    let rhs = self.emit_node(function, right, true);
                                    if !rhs.produced {
                                        function.instruction(&Instruction::I64Const(0));
                                    }
                                    function.instruction(&Instruction::LocalTee(scratch));
                                    function.instruction(&Instruction::I64Store(MemArg {
                                        offset: 8,
                                        align: 3,
                                        memory_index: 0,
                                    }));
                                    // Assignment expression result is the stored value.
                                    function.instruction(&Instruction::LocalGet(scratch));
                                }
                            }
                            return true;
                        }
                    }
                }
            }
        }

        // Compound-assign to a computed for-in-key object target `obj[c] op= v`
        // (Spec 4a Task 4): decompose to `obj[c] = (obj[c] op v)`, routing BOTH
        // the read of `obj[c]` and the write through Task 3's dynamic slot lane.
        // The types gate admits exactly the same accept condition
        // (`forin_key_member_target_is_uniform`) as `obj[c] = v`. Must precede
        // the `assignment_target_name` fallthrough below, which would otherwise
        // reject this member target fail-closed (E5506).
        if matches!(op, "+=" | "-=" | "*=" | "/=" | "%=" | "**=") {
            let left_node = self.node(left).clone();
            if let Some((base, index, elem)) = self.computed_forin_object_access(&left_node) {
                self.emit_object_field_compound_assign_dynamic(
                    function, base, index, right, elem, op,
                );
                return true;
            }
        }

        let Some(name) = self.assignment_target_name(node, left) else {
            if op == "=" {
                return false;
            }

            let message = if op == "??=" {
                "nullish assignment lowering is unavailable unless the target is a mutable local binding; use a mutable variable or the later compatibility path".to_string()
            } else {
                "compound assignment lowering is unavailable unless the target is a mutable local binding; use a mutable variable or the later compatibility path".to_string()
            };
            self.diagnostics
                .push(Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message));
            function.instruction(&Instruction::I64Const(0));
            return true;
        };
        // Stage-review I-9: assignment INTO a URL/USP binding (`u = 5`,
        // `q += x`, `u ??= y`) fails closed — the write-position twin of the
        // member-write gate above. The binding's local holds a raw struct
        // pointer / tagged store handle; overwriting it makes every admitted
        // read (`u.pathname`) a wild load off the new value (observed: prints
        // 0 at address 5+16; node throws on const reassignment). Keyed on all
        // four provenance classifiers so the module-scope and captured twins
        // are covered at the same choke.
        if self.is_url(&name)
            || self.is_url_search_params(&name)
            || self.is_module_scope_url_handle(&name)
            || self.is_captured_url_handle(&name)
        {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "assigning into URL/URLSearchParams binding '{name}' is not supported in \
                     the current phase (the binding holds an internal handle; overwriting it \
                     would make later member reads load from a wild address; fail-closed)"
                ),
            ));
            function.instruction(&Instruction::I64Const(0));
            return true;
        }
        // Stage P5 T-new-A: an assignment REPLACES what the local holds, so the
        // `crypto.getRandomValues(...)` result provenance is re-derived at this
        // choke exactly as at the declarator. `record_...` first revokes any
        // previous ADMISSION (an admitted `.length` loads a length header off
        // the local — a stale grant over `fb = 5` would load off address 5),
        // then re-admits only if the new RHS is itself a proven result. A
        // COMPOUND op only ever REVOKES — its result is a derived value, never
        // the buffer handle, even when its RHS happens to be a result call
        // (`fb += crypto.getRandomValues(rb)`) — and deny-domain membership is
        // never revoked, so the reassigned name's `.length` fails closed
        // instead of silently zeroing.
        if op == "=" {
            self.record_crypto_random_result_binding(&name, right);
        } else {
            self.crypto_random_result_array_bindings.remove(&name);
        }

        // Module-scope mutable scalar promoted to a persistent global: route the
        // write through `GlobalSet` (from a function OR module scope). Gated on
        // the target NOT being a local/param FIRST — a same-named local `var`/
        // `let` or param shadows the module global and must be written to its own
        // slot (JS lexical scoping), so this yields to the local lookup below.
        // (In `_start` a promoted name is never a local, so this still fires.)
        if !self.locals.contains_key(&name) {
            if let Some(&(global_index, repr)) = self.module_global_slots.get(&name) {
                return self.emit_module_global_assignment(function, op, global_index, repr, right);
            }
            // Stage C: a captured scalar promoted to an env cell (own cell or a
            // single-level synchronous outer capture) — route the write through
            // its env cell (read-modify-write for compound ops). `Some` iff
            // `name` is in this function's env plan (handled or E5506-rejected);
            // only genuinely unresolvable names fall through to the E5506 below.
            if let Some(handled) = self.try_emit_captured_assign(function, op, &name, right) {
                return handled;
            }
        }
        let Some(index) = self.locals.get(&name).copied() else {
            if op == "=" {
                return false;
            }

            let message = if op == "??=" {
                format!(
                    "nullish assignment lowering is unavailable for binding '{}' unless it is a mutable local binding; use a mutable variable or the later compatibility path",
                    name
                )
            } else {
                format!(
                    "compound assignment lowering is unavailable for binding '{}' unless it is a mutable local binding; use a mutable variable or the later compatibility path",
                    name
                )
            };
            self.diagnostics
                .push(Diagnostic::error(e5::FEATURE_UNAVAILABLE as u32, message));
            function.instruction(&Instruction::I64Const(0));
            return true;
        };

        match op {
            "=" => {
                // Spec 4a Task 4 null-sentinel: reassigning a for-in-key alias
                // to null/undefined stores `-1`, matching the declarator
                // null-init, so a later `if (alias)` (lowered to `>= 0`) reads
                // false. Recognized structurally via `for_in_key_aliases`.
                if self.for_in_key_aliases.contains(&name) && self.is_null_or_undefined_expr(right)
                {
                    function.instruction(&Instruction::I64Const(-1));
                    function.instruction(&Instruction::LocalTee(index));
                    return true;
                }
                // Spec 4a Task 5: an alias assignment `last = c` (target and RHS
                // both for-in-key provenance) copies the raw ORDINAL, never the
                // materialized string handle — the alias's local must stay an
                // ordinal so `table[last]` indexes correctly, even when the RHS
                // key is ALSO used as a string elsewhere (so its scalar repr is
                // `String` and the generic identifier emit would materialize a
                // handle). Reads the RHS ordinal local directly, bypassing the
                // string-materialization arm in `emit_value`.
                if self.for_in_key_aliases.contains(&name) {
                    if let Some(rhs_name) = self.bare_identifier_name(right) {
                        if self.for_in_key_shapes.contains_key(&rhs_name) {
                            if let Some(&ord_local) = self.locals.get(&rhs_name) {
                                function.instruction(&Instruction::LocalGet(ord_local));
                                function.instruction(&Instruction::LocalTee(index));
                                return true;
                            }
                        }
                    }
                }
                // `a = new Array(n)`: same routing as the declarator path
                // (control_flow.rs:596-610) — the allocation needs a stable
                // handle in the local, and the binding (re)registers as an
                // array so element/length lanes stay routed. Uses `LocalTee`
                // (not the declarator's `LocalSet`) because this arm is an
                // assignment EXPRESSION: it must leave the assigned value on
                // the stack, mirroring the generic path below.
                if let Some(size_arg) = self.resolve_array_alloc_call(right) {
                    let allocated = self.emit_array_allocation(function, size_arg);
                    if !allocated.produced {
                        function.instruction(&Instruction::I64Const(0));
                    }
                    function.instruction(&Instruction::LocalTee(index));
                    self.array_bindings.insert(name.clone());
                    return true;
                }
                // `a = b` where `b` is an array binding: the local already
                // holds an i64 handle either way, but `a` must (re)register
                // as an array binding so its element/length lanes stay
                // routed after the reassignment.
                if let Some(rhs_name) = self.bare_identifier_name(right) {
                    if self.array_bindings.contains(&rhs_name) {
                        let rhs = self.emit_node(function, right, true);
                        if !rhs.produced {
                            function.instruction(&Instruction::I64Const(0));
                        }
                        function.instruction(&Instruction::LocalTee(index));
                        self.array_bindings.insert(name.clone());
                        return true;
                    }
                }
                let rhs = self.emit_node(function, right, true);
                // Promote an integer-valued rhs when the target local holds an f64.
                let f64_target = self.scalar_repr(&name) == kali_common::Repr::F64;
                if !rhs.produced {
                    if f64_target {
                        function.instruction(&Instruction::F64Const(0.0.into()));
                    } else {
                        function.instruction(&Instruction::I64Const(0));
                    }
                } else if f64_target && !self.is_float_valued(right) {
                    function.instruction(&Instruction::F64ConvertI64S);
                }
                function.instruction(&Instruction::LocalTee(index));
                true
            }
            "??=" => {
                let temp_local = self.locals.len() as u32;
                function.instruction(&Instruction::LocalGet(index));
                function.instruction(&Instruction::LocalSet(temp_local));
                function.instruction(&Instruction::LocalGet(temp_local));
                // Resolve guarantees the only `??=` target reaching codegen is a
                // for-in-key ALIAS, whose null sentinel is `-1` (key ordinals are
                // 0-based). The nullish test must be a `-1` sentinel compare, NOT
                // a falsy `I64Eqz` (which would fire on the valid ordinal `0`,
                // overwriting the first key). Sibling `||=`/`&&=` below stay
                // `I64Eqz` — falsy semantics is correct for THEM.
                function.instruction(&Instruction::I64Const(-1));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                // Fired branch, nullish RHS (`null` literal, `undefined`
                // literal, or the bare identifier `undefined`): store the
                // `-1` null sentinel, NOT the generic null lowering (`0` — a
                // VALID key ordinal, which would flip the alias's truthiness
                // from false to true). Mirrors the `=` arm's null-store
                // special case above. Resolve now admits exactly this same
                // nullish set on the `??=` RHS (`is_null_or_undefined_expr`
                // is the single recognizer both sides share), so the
                // generic-emit fallback below is defensive only.
                if self.for_in_key_aliases.contains(&name) && self.is_null_or_undefined_expr(right)
                {
                    function.instruction(&Instruction::I64Const(-1));
                } else {
                    let rhs = self.emit_node(function, right, true);
                    if !rhs.produced {
                        function.instruction(&Instruction::I64Const(0));
                    }
                }
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(temp_local));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalSet(index));
                function.instruction(&Instruction::LocalGet(index));
                true
            }
            "&&=" | "||=" => {
                let temp_local = self.locals.len() as u32;
                function.instruction(&Instruction::LocalGet(index));
                function.instruction(&Instruction::LocalSet(temp_local));
                function.instruction(&Instruction::LocalGet(temp_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                if op == "&&=" {
                    function.instruction(&Instruction::LocalGet(temp_local));
                } else {
                    let rhs = self.emit_node(function, right, true);
                    if !rhs.produced {
                        function.instruction(&Instruction::I64Const(0));
                    }
                }
                function.instruction(&Instruction::Else);
                if op == "&&=" {
                    let rhs = self.emit_node(function, right, true);
                    if !rhs.produced {
                        function.instruction(&Instruction::I64Const(0));
                    }
                } else {
                    function.instruction(&Instruction::LocalGet(temp_local));
                }
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::LocalSet(index));
                function.instruction(&Instruction::LocalGet(index));
                true
            }
            "+=" | "-=" | "*=" | "/=" | "%=" | "**=" => {
                if self.scalar_repr(&name) == kali_common::Repr::String {
                    // String compound-assign. Only `+=` has a meaning
                    // (concatenation); `-=`/`*=`/… on a string are nonsensical
                    // and have no lowering — reject fail-closed.
                    if op != "+=" {
                        self.diagnostics.push(Diagnostic::error(
                            e5::FEATURE_UNAVAILABLE as u32,
                            format!(
                                "compound assignment '{op}' on string binding '{name}' is unavailable in the current phase"
                            ),
                        ));
                        function.instruction(&Instruction::I64Const(0));
                        return true;
                    }
                    // `a += e` ≡ `a = a + e`: concatenate the current handle
                    // with the (stringified) rhs and store the fresh handle.
                    // Per-site arena routing (fasta Spec 7 Task 4d): the `+=`
                    // node (`id`) has text `"+="`, which `is_string_site` never
                    // records in the string-site stream, so
                    // `string_concat_import_index` ALWAYS misses here and fails
                    // closed to the global `string_concat` — the accumulator
                    // outlives the iteration (bound to a name), so its result
                    // must NOT be reclaimed. Routed through the shared selector
                    // anyway so the two concat sites stay a single oracle.
                    function.instruction(&Instruction::LocalGet(index));
                    self.emit_as_string(function, right);
                    function.instruction(&Instruction::Call(self.string_concat_import_index(id)));
                    function.instruction(&Instruction::LocalTee(index));
                    return true;
                }
                if self.scalar_repr(&name) == kali_common::Repr::F64 {
                    // f64 compound-assign: the accumulator is an f64, so the read, the
                    // rhs, and the arithmetic all use the f64 opcodes. `%=`/`**=` on
                    // floats are out of scope for this slice.
                    if matches!(op, "%=" | "**=") {
                        self.diagnostics.push(Diagnostic::error(
                            e5::FEATURE_UNAVAILABLE as u32,
                            format!(
                                "compound assignment '{}' on floating-point binding '{}' is unavailable in the current phase",
                                op, name
                            ),
                        ));
                        function.instruction(&Instruction::F64Const(0.0.into()));
                        return true;
                    }
                    function.instruction(&Instruction::LocalGet(index));
                    let rhs = self.emit_node(function, right, true);
                    if !rhs.produced {
                        function.instruction(&Instruction::F64Const(0.0.into()));
                    } else if !self.is_float_valued(right) {
                        function.instruction(&Instruction::F64ConvertI64S);
                    }
                    match op {
                        "+=" => function.instruction(&Instruction::F64Add),
                        "-=" => function.instruction(&Instruction::F64Sub),
                        "*=" => function.instruction(&Instruction::F64Mul),
                        "/=" => function.instruction(&Instruction::F64Div),
                        _ => unreachable!(),
                    };
                    function.instruction(&Instruction::LocalSet(index));
                    function.instruction(&Instruction::LocalGet(index));
                    return true;
                }
                function.instruction(&Instruction::LocalGet(index));
                let rhs = self.emit_node(function, right, true);
                if !rhs.produced {
                    function.instruction(&Instruction::I64Const(0));
                }
                match op {
                    "+=" => function.instruction(&Instruction::I64Add),
                    "-=" => function.instruction(&Instruction::I64Sub),
                    "*=" => function.instruction(&Instruction::I64Mul),
                    "/=" => function.instruction(&Instruction::I64DivS),
                    "%=" => function.instruction(&Instruction::I64RemS),
                    "**=" => function.instruction(&Instruction::Call(MATH_POW_IMPORT_INDEX)),
                    _ => unreachable!(),
                };
                function.instruction(&Instruction::LocalSet(index));
                function.instruction(&Instruction::LocalGet(index));
                true
            }
            _ => false,
        }
    }

    /// Assignment (`=` or an arithmetic compound `+= -= *= /= %= **=`) to a
    /// module-scope mutable scalar promoted to a persistent WASM global. Plain
    /// `=` emits `<rhs> ; GlobalSet`; a compound decomposes to
    /// `GlobalGet ; <rhs> ; op ; GlobalSet`. Both leave the stored value on the
    /// stack (`GlobalGet` after the set), so the assignment EXPRESSION result is
    /// available — matching the `LocalTee` contract of the local path.
    ///
    /// The nullish/logical compounds (`??= &&= ||=`) and `%= **=` on an f64
    /// global are out of scope for this slice — rejected fail-closed (E5506)
    /// rather than mis-lowered.
    pub(crate) fn emit_module_global_assignment(
        &mut self,
        function: &mut Function,
        op: &str,
        global_index: u32,
        repr: kali_common::Repr,
        right: LirNodeId,
    ) -> bool {
        let is_f64 = repr == kali_common::Repr::F64;
        match op {
            "=" => {
                let rhs = self.emit_node(function, right, true);
                if !rhs.produced {
                    if is_f64 {
                        function.instruction(&Instruction::F64Const(0.0.into()));
                    } else {
                        function.instruction(&Instruction::I64Const(0));
                    }
                } else if is_f64 && !self.is_float_valued(right) {
                    function.instruction(&Instruction::F64ConvertI64S);
                }
                function.instruction(&Instruction::GlobalSet(global_index));
                function.instruction(&Instruction::GlobalGet(global_index));
                true
            }
            "+=" | "-=" | "*=" | "/=" | "%=" | "**=" => {
                if is_f64 && matches!(op, "%=" | "**=") {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "compound assignment '{op}' on a floating-point module global is unavailable in the current phase"
                        ),
                    ));
                    function.instruction(&Instruction::F64Const(0.0.into()));
                    return true;
                }
                function.instruction(&Instruction::GlobalGet(global_index));
                let rhs = self.emit_node(function, right, true);
                if is_f64 {
                    if !rhs.produced {
                        function.instruction(&Instruction::F64Const(0.0.into()));
                    } else if !self.is_float_valued(right) {
                        function.instruction(&Instruction::F64ConvertI64S);
                    }
                    match op {
                        "+=" => function.instruction(&Instruction::F64Add),
                        "-=" => function.instruction(&Instruction::F64Sub),
                        "*=" => function.instruction(&Instruction::F64Mul),
                        "/=" => function.instruction(&Instruction::F64Div),
                        _ => unreachable!(),
                    };
                } else {
                    if !rhs.produced {
                        function.instruction(&Instruction::I64Const(0));
                    }
                    match op {
                        "+=" => function.instruction(&Instruction::I64Add),
                        "-=" => function.instruction(&Instruction::I64Sub),
                        "*=" => function.instruction(&Instruction::I64Mul),
                        "/=" => function.instruction(&Instruction::I64DivS),
                        "%=" => function.instruction(&Instruction::I64RemS),
                        "**=" => function.instruction(&Instruction::Call(MATH_POW_IMPORT_INDEX)),
                        _ => unreachable!(),
                    };
                }
                function.instruction(&Instruction::GlobalSet(global_index));
                function.instruction(&Instruction::GlobalGet(global_index));
                true
            }
            // `??= &&= ||=` on a module global: out of scope, fail-closed.
            _ => {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "assignment operator '{op}' on a module global is unavailable in the current phase"
                    ),
                ));
                if is_f64 {
                    function.instruction(&Instruction::F64Const(0.0.into()));
                } else {
                    function.instruction(&Instruction::I64Const(0));
                }
                true
            }
        }
    }
}

#[cfg(test)]
#[path = "literal_tests.rs"]
mod literal_tests;
