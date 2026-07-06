use crate::*;

impl<'a> FunctionEmitter<'a> {
    pub(crate) fn emit_update_expression(
        &mut self,
        function: &mut Function,
        node: &LirNode,
        op: &str,
        arg: LirNodeId,
    ) -> EmittedValue {
        let Some(name) = self.assignment_target_name(node, arg) else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "update expression lowering is unavailable unless the target is a mutable local binding; use a mutable variable or the later compatibility path",
            ));
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        };
        let Some(index) = self.locals.get(&name).copied() else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "update expression lowering is unavailable for binding '{}' unless it is a mutable local binding; use a mutable variable or the later compatibility path",
                    name
                ),
            ));
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        };

        let is_increment = matches!(op, "prefix++" | "postfix++");
        let is_prefix = matches!(op, "prefix++" | "prefix--");
        let temp_local = self.locals.len() as u32;

        if !is_prefix {
            function.instruction(&Instruction::LocalGet(index));
            function.instruction(&Instruction::LocalSet(temp_local));
        }

        function.instruction(&Instruction::LocalGet(index));
        function.instruction(&Instruction::I64Const(1));
        if is_increment {
            function.instruction(&Instruction::I64Add);
        } else {
            function.instruction(&Instruction::I64Sub);
        }
        function.instruction(&Instruction::LocalSet(index));

        if is_prefix {
            function.instruction(&Instruction::LocalGet(index));
        } else {
            function.instruction(&Instruction::LocalGet(temp_local));
        }

        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
    }

    pub(crate) fn emit_unary(&mut self, function: &mut Function, node: &LirNode) -> EmittedValue {
        let op = node.text.as_deref().unwrap_or_default();
        let arg = node.children[0];
        // A string operand under a numeric/logical unary op has no correct
        // lowering: `-`/`+`/`~` would arithmetic on a raw handle; `!` truthiness
        // is wrong for a fresh concat handle (empty-string handle is non-zero).
        // Reject fail-closed. `-`/`+`/`~` reject any string; `!` rejects only a
        // tainted (runtime-concat) string (an interned literal keeps today's
        // behavior, matching the base compiler).
        if (matches!(op, "-" | "+" | "~") && self.is_string_valued(arg))
            || (op == "!" && self.is_runtime_concat_string(arg))
        {
            self.diagnostics.push(Diagnostic::error(
                e3::TYPE_MISMATCH as u32,
                format!(
                    "unary operator '{op}' on a runtime string value is unavailable in the current direct-runtime path"
                ),
            ));
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        match op {
            "prefix++" | "postfix++" | "prefix--" | "postfix--" => {
                self.emit_update_expression(function, node, op, arg)
            }
            "-" => {
                if self.is_float_valued(arg) {
                    let _ = self.emit_node(function, arg, true);
                    function.instruction(&Instruction::F64Neg);
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Float,
                    };
                }
                function.instruction(&Instruction::I64Const(0));
                let _ = self.emit_node(function, arg, true);
                function.instruction(&Instruction::I64Sub);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                }
            }
            "+" => self.emit_node(function, arg, true),
            "~" => {
                function.instruction(&Instruction::I64Const(0));
                let _ = self.emit_node(function, arg, true);
                function.instruction(&Instruction::I64Sub);
                function.instruction(&Instruction::I64Const(1));
                function.instruction(&Instruction::I64Sub);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                }
            }
            "!" => {
                let _ = self.emit_node(function, arg, true);
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I64ExtendI32U);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                }
            }
            "void" => {
                let produced = self.emit_node(function, arg, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
                function.instruction(&Instruction::I64Const(0));
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                }
            }
            "delete" => {
                if let Some(key_text) = process_env_property_key(&self.program.nodes, arg) {
                    let Some(import_index) = self.env_delete_import_index else {
                        self.diagnostics.push(Diagnostic::warning(
                            e8::UNIMPLEMENTED as u32,
                            "process.env property deletion is unavailable until the later mutable env path is enabled".to_string(),
                        ));
                        let produced = self.emit_node(function, arg, true);
                        if produced.produced {
                            function.instruction(&Instruction::Drop);
                        }
                        function.instruction(&Instruction::I64Const(0));
                        return EmittedValue {
                            produced: true,
                            shape: ValueShape::Unknown,
                        };
                    };
                    let (key_offset, key_len) = self.strings.intern(&key_text);
                    function.instruction(&Instruction::I32Const(key_offset as i32));
                    function.instruction(&Instruction::I32Const(key_len as i32));
                    function.instruction(&Instruction::I32Const(0));
                    function.instruction(&Instruction::I32Const(0));
                    function.instruction(&Instruction::Call(import_index));
                    function.instruction(&Instruction::Drop);
                    function.instruction(&Instruction::I64Const(0));
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Unknown,
                    };
                }

                self.diagnostics.push(Diagnostic::warning(
                    e8::UNIMPLEMENTED as u32,
                    format!("unsupported unary operator '{}'", op),
                ));
                let produced = self.emit_node(function, arg, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
                function.instruction(&Instruction::I64Const(0));
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                }
            }
            "pid" => {
                if self.is_deno_pid(arg) || self.is_process_pid(arg) {
                    function.instruction(&Instruction::Call(PROCESS_PID_IMPORT_INDEX));
                    function.instruction(&Instruction::I64ExtendI32U);
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    };
                }

                self.push_placeholder_fallback_diagnostic("identifier", "pid");
                function.instruction(&Instruction::I64Const(0));
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                }
            }
            "length" => {
                if self.is_process_argv(arg) || self.is_deno_args(arg) {
                    function.instruction(&Instruction::Call(ARGS_LEN_IMPORT_INDEX));
                    function.instruction(&Instruction::I64ExtendI32U);
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    };
                }

                if let Some(slice_start) = self.process_argv_slice_start(arg) {
                    function.instruction(&Instruction::Call(ARGS_LEN_IMPORT_INDEX));
                    function.instruction(&Instruction::I64ExtendI32U);
                    function.instruction(&Instruction::I64Const(slice_start));
                    function.instruction(&Instruction::I64Sub);
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    };
                }

                if let Some(aggregate_id) = self.resolve_literal_aggregate(arg) {
                    let aggregate = self.node(aggregate_id).clone();
                    if self.is_array_literal(&aggregate) {
                        function
                            .instruction(&Instruction::I64Const(aggregate.children.len() as i64));
                        return EmittedValue {
                            produced: true,
                            shape: ValueShape::Scalar,
                        };
                    }
                }

                if let Some(StaticObjectIdentityValue::String(value)) =
                    self.resolve_static_object_identity_value(arg)
                {
                    function
                        .instruction(&Instruction::I64Const(value.encode_utf16().count() as i64));
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    };
                }

                let produced = self.emit_node(function, arg, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
                function.instruction(&Instruction::I64Const(0));
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                }
            }
            op if op.parse::<usize>().is_ok() || op.parse::<isize>().is_ok() => {
                if let Ok(index) = op.parse::<usize>() {
                    if let Some(element) = self.resolve_static_array_slice_element(arg, index) {
                        return self.emit_node(function, element, true);
                    }
                }

                if let Some(aggregate_id) = self.resolve_literal_aggregate(arg) {
                    let aggregate = self.node(aggregate_id).clone();
                    if self.is_array_literal(&aggregate)
                        && op.parse::<isize>().ok().is_some_and(|index| index >= 0)
                    {
                        if let Ok(index) = op.parse::<usize>() {
                            if let Some(element) = aggregate.children.get(index).copied() {
                                return self.emit_node(function, element, true);
                            }
                        }
                    }

                    let field = op
                        .parse::<isize>()
                        .ok()
                        .map(|value| {
                            if value == 0 {
                                "0".to_string()
                            } else {
                                value.to_string()
                            }
                        })
                        .unwrap_or_else(|| op.to_string());
                    if let Some(field_value) = self.object_literal_field(&aggregate, &field) {
                        return self.emit_node(function, field_value, true);
                    }
                }

                let produced = self.emit_node(function, arg, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
                function.instruction(&Instruction::I64Const(0));
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                }
            }
            "version" => {
                if let Some(rendered) = self.render_package_json_version_access(arg) {
                    let (offset, len) = self.strings.intern(&rendered);
                    function.instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    };
                }

                if self.has_semver_import() {
                    if let Some(rendered) = self.render_static_value(arg) {
                        let (offset, len) = self.strings.intern(&rendered);
                        function
                            .instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
                        return EmittedValue {
                            produced: true,
                            shape: ValueShape::Scalar,
                        };
                    }
                }

                let produced = self.emit_node(function, arg, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
                function.instruction(&Instruction::I64Const(0));
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                }
            }
            "yield" | "yield*" | "delegate" => {
                let is_delegate = matches!(op, "yield*" | "delegate");
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    generator_function_yield_lowering_unavailable_message(
                        matches!(
                            self.current_function_flavor,
                            Some(FunctionFlavor::AsyncGenerator)
                        ),
                        is_delegate,
                    ),
                ));
                function.instruction(&Instruction::Unreachable);
                EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                }
            }
            _ => {
                if let Some(shape) = self.object_shape_of_node(arg) {
                    return self.emit_object_field_read(function, arg, shape, op);
                }

                if let Some(aggregate_id) = self.resolve_literal_aggregate(arg) {
                    let aggregate = self.node(aggregate_id).clone();
                    if let Some(field_value) = self.object_literal_field(&aggregate, op) {
                        // The fold lane found a structural field. Unknown
                        // fields on a KNOWN object literal stay fold-first
                        // (that `None` case falls all the way through to the
                        // pre-existing warning+0 fallback below, unchanged —
                        // see `unknown_field_read_is_fold_first_until_materialized`).
                        // But if the field's OWN value resolves (through the
                        // same const-fold alias chain) to ANOTHER unlabeled
                        // object-literal aggregate with no proven runtime
                        // shape, emitting it directly falls into
                        // `emit_aggregate_literal`'s drop-only fallback, which
                        // always pushes a placeholder `0` — silently wrong for
                        // a value read (only correct in a discarded-statement
                        // position). REJECT-DON'T-MISCOMPILE: reject that
                        // specific dead end at compile time instead.
                        if self.fold_lane_field_value_is_dead_end(field_value) {
                            self.diagnostics.push(Diagnostic::error(
                                e5::FEATURE_UNAVAILABLE as u32,
                                format!(
                                    "reading property '{op}' of a value with no statically inferred object shape is unavailable in the current phase"
                                ),
                            ));
                            function.instruction(&Instruction::Unreachable);
                            return EmittedValue {
                                produced: false,
                                shape: ValueShape::Unknown,
                            };
                        }
                        return self.emit_node(function, field_value, true);
                    }
                }

                // Chained member read off a fold-lane member base
                // (`t.left.v` where `t.left` folds to an object reference):
                // the inner read may produce a perfectly valid object pointer
                // (e.g. `arr[0]` with materialized object elements), but this
                // OUTER read has no way to classify a member-chain base
                // (`object_shape_of_node` recognizes identifier and
                // array-subscript bases only), so it would fall into the
                // warning+0 placeholder below — a silent wrong answer.
                // REJECT-DON'T-MISCOMPILE: reject when the inner fold-lane
                // substitution is object-shaped (either a dead end or a node
                // with a proven object shape).
                if let Some(substituted) = self.fold_lane_member_chain_field_value(arg) {
                    if self.fold_lane_field_value_is_dead_end(substituted)
                        || self.object_shape_of_node(substituted).is_some()
                    {
                        self.diagnostics.push(Diagnostic::error(
                            e5::FEATURE_UNAVAILABLE as u32,
                            format!(
                                "reading property '{op}' of a value with no statically inferred object shape is unavailable in the current phase"
                            ),
                        ));
                        function.instruction(&Instruction::Unreachable);
                        return EmittedValue {
                            produced: false,
                            shape: ValueShape::Unknown,
                        };
                    }
                }

                self.diagnostics.push(Diagnostic::warning(
                    e8::UNIMPLEMENTED as u32,
                    format!("unsupported unary operator '{}'", op),
                ));
                let produced = self.emit_node(function, arg, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
                function.instruction(&Instruction::I64Const(0));
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                }
            }
        }
    }

    /// True when `field_value` — a field found via the compile-time fold lane
    /// (`object_literal_field`) — is an object reference that emitting as a
    /// VALUE would silently mislower to `emit_aggregate_literal`'s drop-only
    /// fallback (an unconditional placeholder `I64Const(0)`, correct only in
    /// a discarded-statement position). REJECT-DON'T-MISCOMPILE: callers must
    /// turn this into a compile-time E5506 instead of emitting it.
    ///
    /// Positively matched dead-end shapes (each pinned by a rejection test in
    /// `kali_cli/tests/object_call_result_args_runtime.rs`):
    /// - an inline nested object literal (`{ left: { v: 5 } }` — `t.left`
    ///   would emit the placeholder, so `t.left.v` reads 0 and
    ///   `t.left === null` prints true);
    /// - a bare identifier whose const-fold binding (`self.bindings`)
    ///   resolves through `resolve_literal_aggregate` to an object literal
    ///   (`const t = { left: leafA }`);
    /// - a static index into a const-bound array literal (`arr[0]`) whose
    ///   element is, recursively, one of these dead-end shapes.
    ///
    /// Everything else stays on today's green surface, by construction:
    /// scalar/string/array-literal field values are not `is_object_literal`;
    /// host/global member chains (`Deno.env`, `globalThis.Math`, `Object`,
    /// `Set`, `Map`, …) never resolve through `resolve_literal_aggregate` to
    /// a user object literal; and a MISSING field never reaches this
    /// predicate at all (`object_literal_field` returns `None` first), so
    /// fold-first unknown-field reads keep printing the JS-`undefined`
    /// placeholder (`unknown_field_read_is_fold_first_until_materialized`).
    /// When `id` is a one-child member-read node (`<base>.field`, property
    /// name in `text`) whose base resolves through the const-fold alias chain
    /// to an object literal declaring that field, returns the field's value
    /// node — the node the fold lane would substitute when emitting `id`.
    /// Used by `emit_unary`'s default arm to see through ONE level of member
    /// chaining (`t.left.v`) and gate the outer read when the substitution is
    /// object-shaped. `None` for every other shape (identifier, subscript,
    /// call, host chain, unknown field), which keeps all of those on their
    /// existing paths.
    fn fold_lane_member_chain_field_value(&self, id: LirNodeId) -> Option<LirNodeId> {
        let id = self.unwrap_transparent(id);
        let node = self.node(id).clone();
        if node.kind != LirNodeKind::Value || node.children.len() != 1 {
            return None;
        }
        let field = node.text.as_deref().filter(|text| !text.is_empty())?;
        let aggregate_id = self.resolve_literal_aggregate(node.children[0])?;
        let aggregate = self.node(aggregate_id).clone();
        self.object_literal_field(&aggregate, field)
    }

    pub(crate) fn fold_lane_field_value_is_dead_end(&self, field_value: LirNodeId) -> bool {
        self.fold_lane_field_value_is_dead_end_at_depth(field_value, 0)
    }

    fn fold_lane_field_value_is_dead_end_at_depth(
        &self,
        field_value: LirNodeId,
        depth: usize,
    ) -> bool {
        // Self-referential const initializers (`const arr = [arr[0]]`) make
        // the static-index resolution below return the same node id forever;
        // bail out conservatively (no rejection) past a small alias depth.
        if depth > 8 {
            return false;
        }
        let node = self.node(field_value).clone();
        // Inline nested object literal, or an identifier alias resolving to
        // one: `resolve_literal_aggregate` returns the literal itself for the
        // inline shape and follows `self.bindings` for the alias shape.
        if let Some(resolved) = self.resolve_literal_aggregate(field_value) {
            if self.is_object_literal(&self.node(resolved).clone()) {
                return true;
            }
        }
        // Static array-literal element (`arr[0]`): recurse into the element
        // the fold lane would substitute.
        if let Some(StaticIndexMemberResult::Node(element)) =
            self.resolve_static_index_member(&node)
        {
            return self.fold_lane_field_value_is_dead_end_at_depth(element, depth + 1);
        }
        false
    }

    /// Unwraps transparent single-child `Value` wrappers (no operator text) so a
    /// string-classification query inspects the underlying literal/expression.
    pub(crate) fn unwrap_transparent(&self, mut id: LirNodeId) -> LirNodeId {
        let mut guard = 0;
        loop {
            let node = self.node(id);
            if node.kind == LirNodeKind::Value
                && node.children.len() == 1
                && node.text.as_deref().is_none_or(|text| text.is_empty())
            {
                id = node.children[0];
                guard += 1;
                if guard > 64 {
                    return id;
                }
                continue;
            }
            return id;
        }
    }

    /// Returns true when `id` statically evaluates to a string value: a string (or
    /// template) literal, a `+` expression whose either operand is string-valued,
    /// a bare identifier whose binding's repr is proven `Repr::String` (mirroring
    /// `is_float_valued`'s local-vs-module-const resolution: a name not declared
    /// locally in a non-`_start` function reads the module `_start` binding), or
    /// a call to a function whose return repr is proven `Repr::String`. The
    /// identifier/call arms are the codegen half of the runtime string-value
    /// flow — `kali_types`'s `E3200` gate (`operand_repr_is_string`) is narrowed
    /// to allow exactly the operands these arms also recognize, so the two never
    /// disagree.
    pub(crate) fn is_string_valued(&self, id: LirNodeId) -> bool {
        let id = self.unwrap_transparent(id);
        // Resolve a local `const` fold-alias (`self.bindings`) BEFORE the bare-
        // identifier repr lookup below, mirroring `is_float_valued` (which has
        // the same hazard, documented there): a fold-lane `const` is NOT in
        // `self.locals`, so the identifier arm would misroute it to the module
        // `_start` table — classifying a function-local `const s = "a"` as
        // non-string (int-coercing a string handle) even though `emit_node`'s
        // identifier fallback resolves the read through `self.bindings` to the
        // string literal. Resolving first classifies the node `emit_node` will
        // actually emit.
        let id = self.resolve_bound_node(id);
        let node = self.node(id);
        match node.kind {
            LirNodeKind::Literal => node.text.as_deref().is_some_and(|text| {
                let trimmed = text.trim();
                let mut chars = trimmed.chars();
                matches!(
                    (chars.next(), trimmed.chars().last()),
                    (Some('"'), Some('"')) | (Some('\''), Some('\'')) | (Some('`'), Some('`'))
                )
            }),
            LirNodeKind::Value if node.children.len() == 2 && node.text.as_deref() == Some("+") => {
                self.is_string_valued(node.children[0]) || self.is_string_valued(node.children[1])
            }
            // Ternary `test ? a : b` (marker text "?"): string-valued iff
            // either arm is — mirrors `emit_conditional`'s `string_result` (and
            // the symmetric `is_float_valued` ternary arm) so a string-armed
            // ternary used as a `+` operand takes the string-concat path
            // instead of leaking its tagged handle through `int_to_string`.
            LirNodeKind::Value if node.children.len() == 3 && node.text.as_deref() == Some("?") => {
                self.is_string_valued(node.children[1]) || self.is_string_valued(node.children[2])
            }
            // Bare identifier read: string iff its binding's repr is String.
            LirNodeKind::Value if node.children.is_empty() => {
                node.text.as_deref().is_some_and(|name| {
                    if !self.locals.contains_key(name) && self.function_name != "_start" {
                        self.repr_table.scalar("_start", name) == kali_common::Repr::String
                    } else {
                        self.scalar_repr(name) == kali_common::Repr::String
                    }
                })
            }
            // Runtime substring: a slice of a string is a string.
            LirNodeKind::Call if self.runtime_substring_call_parts(node).is_some() => true,
            // Call to a string-returning function.
            LirNodeKind::Call => {
                let Some(callee) = node.children.first().copied() else {
                    return false;
                };
                let callee = self.unwrap_transparent(callee);
                let callee_node = self.node(callee);
                callee_node.text.as_deref().is_some_and(|name| {
                    self.repr_table.return_repr(name) == kali_common::Repr::String
                })
            }
            _ => false,
        }
    }

    /// True when `id` is a string value backed by a FRESH runtime
    /// `string_concat` handle (a `+` expression, an interpolated template, a
    /// `Repr::String`-tainted identifier/call), as opposed to an interned
    /// literal constant. Such a handle must never be identity-compared
    /// (`==`/`!=`/`===`/`!==`) or truthiness-tested: two equal-valued concat
    /// results have DIFFERENT handles, and every non-empty AND empty handle is
    /// non-zero. Interned literals (and identifiers proven string but NOT
    /// tainted) are safe to compare/test by identity and return `false` here.
    /// Mirrors `is_string_valued`'s local-vs-module identifier resolution so the
    /// taint query keys the same table entry.
    pub(crate) fn is_runtime_concat_string(&self, id: LirNodeId) -> bool {
        let id = self.unwrap_transparent(id);
        let id = self.resolve_bound_node(id);
        let node = self.node(id);
        match node.kind {
            // Interned literal constant: identity == value, never tainted.
            LirNodeKind::Literal => false,
            // Inline `+`: a string `+` lowers to a fresh runtime concat handle.
            LirNodeKind::Value if node.children.len() == 2 && node.text.as_deref() == Some("+") => {
                self.is_string_valued(id)
            }
            // Bare identifier read: tainted iff its binding is marked tainted.
            LirNodeKind::Value if node.children.is_empty() => {
                node.text.as_deref().is_some_and(|name| {
                    if !self.locals.contains_key(name) && self.function_name != "_start" {
                        self.repr_table.is_string_concat_tainted("_start", name)
                    } else {
                        self.repr_table
                            .is_string_concat_tainted(&self.function_name, name)
                    }
                })
            }
            // A runtime substring result is a non-interned runtime string.
            LirNodeKind::Call
                if self.runtime_substring_call_parts(node).is_some()
                    && self.resolve_static_string_substring_call(node).is_none() =>
            {
                true
            }
            // Call to a string-returning function: tainted iff the return is.
            LirNodeKind::Call => {
                let Some(callee) = node.children.first().copied() else {
                    return false;
                };
                let callee = self.unwrap_transparent(callee);
                let callee_node = self.node(callee);
                callee_node
                    .text
                    .as_deref()
                    .is_some_and(|name| self.repr_table.is_string_concat_tainted_return(name))
            }
            // Any other string-valued node (e.g. an interpolated template that
            // reached here without folding): treat as runtime (fail-closed).
            _ => self.is_string_valued(id),
        }
    }

    /// Push a fail-closed diagnostic when `cond` is a fresh runtime concat
    /// string used in a boolean/truthiness position (if/while/for/do-while/
    /// ternary test): a concat handle is always non-zero, so JS empty-string
    /// falsiness is lost. Interned literals (and non-tainted string bindings)
    /// keep today's behavior — matching the base compiler, no regression.
    /// Emission continues; the error aborts the compile and the bytes are
    /// discarded.
    pub(crate) fn reject_string_condition(&mut self, cond: LirNodeId) {
        if self.is_runtime_concat_string(cond) {
            self.diagnostics.push(Diagnostic::error(
                e3::TYPE_MISMATCH as u32,
                "a runtime string value is unavailable as a condition in the current direct-runtime path; its truthiness (empty vs non-empty) is not evaluated".to_string(),
            ));
        }
    }

    /// Returns the identifier name of an array base `a` in a member read `a[i]`,
    /// after unwrapping transparent wrappers. `None` when the base is not a bare
    /// identifier.
    fn array_read_base_name(&self, base: LirNodeId) -> Option<String> {
        let base = self.unwrap_transparent(base);
        let node = self.node(base);
        if node.kind == LirNodeKind::Value && node.children.is_empty() {
            node.text.clone()
        } else {
            None
        }
    }

    /// True when a numeric literal text denotes a float (has a fractional part or
    /// exponent), i.e. it does not round-trip through the integer parser but does
    /// parse as an `f64`.
    fn is_float_literal_text(text: &str) -> bool {
        parse_number_literal(text).is_none() && parse_numeric_literal_value(text).is_some()
    }

    /// True when `id` (after unwrapping transparent wrappers and resolving
    /// const bindings) is a BigInt literal such as `3n`, or a unary `-` applied
    /// to one (`-3n`) — `emit_unary`'s `-` arm lowers a BigInt-literal operand
    /// via `i64.const 0` / `i64.sub`, which stays on the i64 lane, so the
    /// negated form is just as div_s-eligible as the plain literal. BigInt
    /// arithmetic stays on the i64 lane; in particular JS BigInt `/` truncates
    /// toward zero — `i64.div_s` — never `f64.div`. Scope is deliberately
    /// literal / const-bound-literal operands (optionally negated): the repr
    /// machinery has no BigInt axis yet, so BigInt-typed mutable locals keep
    /// the (wrong) float path, and mixed `3n / 2` (a JS TypeError) still
    /// floats too — both recorded follow-ups.
    fn is_bigint_literal_valued(&self, id: LirNodeId) -> bool {
        let id = self.unwrap_transparent(id);
        let id = self.resolve_bound_node(id);
        let node = self.node(id);
        if node.kind == LirNodeKind::Value
            && node.children.len() == 1
            && node.text.as_deref() == Some("-")
        {
            return self.is_bigint_literal_valued(node.children[0]);
        }
        node.kind == LirNodeKind::Literal
            && node
                .text
                .as_deref()
                .and_then(|text| text.strip_suffix('n'))
                .is_some_and(|digits| digits.parse::<i64>().is_ok())
    }

    /// Structural oracle: true when the value produced by `id` is represented as an
    /// `f64`. Mirrors `is_string_valued`; consulted per-operand by `emit_binary`
    /// to decide instruction selection and int->float promotion.
    ///
    /// - identifier -> its scalar repr is `F64` (or it is a float literal),
    /// - `/` -> always float (JS division yields a double in this model),
    /// - `+ - *` -> float if either operand is float,
    /// - array read `a[i]` -> the array's element repr is `F64`,
    /// - call -> the callee's return repr is `F64`,
    /// - float literal -> true.
    pub(crate) fn is_float_valued(&self, id: LirNodeId) -> bool {
        let id = self.unwrap_transparent(id);
        // Resolve a local `const` fold-alias (`self.bindings`, e.g. a for-of/for-in
        // bound name or an unpromoted scalar `const`) BEFORE treating a bare
        // identifier as a candidate for module-const inlining below. Without this,
        // a local binding whose name collides with a float module `const` (a
        // for-of/for-in loop variable, a catch parameter, or any other unpromoted
        // local `const`) is misclassified by the module-const fallback as sharing
        // the module binding's float-ness, even though `emit_node`'s own identifier
        // fallback (see `control_flow.rs`) already correctly resolves the read
        // through `self.bindings` to the LOCAL (int) value — a type/value mismatch
        // that produces an invalid WASM module. Mirrors `is_bigint_literal_valued`,
        // which already calls `resolve_bound_node` for the same reason.
        let id = self.resolve_bound_node(id);
        let node = self.node(id);
        // Fixed-shape object field read: the repr comes from the shape table.
        if node.kind == LirNodeKind::Value && node.children.len() == 1 {
            if let (Some(field), Some(shape)) = (
                node.text.as_deref().filter(|text| !text.is_empty()),
                self.object_shape_of_node(node.children[0]),
            ) {
                return matches!(
                    self.repr_table.shape_field(shape, field),
                    Some((_, kali_common::Repr::F64))
                );
            }
        }
        match node.kind {
            LirNodeKind::Literal => node
                .text
                .as_deref()
                .is_some_and(Self::is_float_literal_text),
            LirNodeKind::Call => {
                let Some(callee) = node.children.first().copied() else {
                    return false;
                };
                let callee = self.unwrap_transparent(callee);
                let callee_node = self.node(callee);
                if callee_node.text.as_deref() == Some("sqrt") && self.is_math_object(callee_node) {
                    // `Math.sqrt(x)` is f64 at runtime (`F64Sqrt`) UNLESS `x` is a
                    // statically-known perfect square, in which case codegen still
                    // constant-folds to a plain `i64` scalar (see
                    // `math_sqrt_constant_root` in `emit/call.rs`). Mirror that
                    // decision here so consumers (e.g. `<`) pick the matching
                    // instruction shape.
                    return node
                        .children
                        .get(1)
                        .copied()
                        .is_some_and(|arg| self.math_sqrt_constant_root(arg).is_none());
                }
                callee_node
                    .text
                    .as_deref()
                    .is_some_and(|name| self.repr_table.return_repr(name) == kali_common::Repr::F64)
            }
            LirNodeKind::Value => match node.children.len() {
                0 => node.text.as_deref().is_some_and(|name| {
                    if Self::is_float_literal_text(name) {
                        return true;
                    }
                    // Module const inlined at this site: classify by its initializer.
                    if !self.locals.contains_key(name) && self.function_name != "_start" {
                        if let Some(&init) = self.module_const_inits.get(name) {
                            return self.is_float_valued(init);
                        }
                    }
                    self.scalar_repr(name) == kali_common::Repr::F64
                }),
                1 => {
                    let text = node.text.as_deref().unwrap_or_default();
                    if text.is_empty() || text == "-" || text == "+" {
                        // Transparent wrapper or unary sign: float-ness follows the
                        // operand.
                        self.is_float_valued(node.children[0])
                    } else if text == "length" {
                        // `a.length` shares the one-child member shape (base child +
                        // property `text`) with an array element read `a[idx]`, but
                        // the length header is always emitted as an i64 (see the
                        // `.length` load in control_flow.rs). Never treat it as a
                        // float element read, or a relational/arithmetic op would
                        // wrongly select the float path and leave an i64 where an
                        // f64 is expected.
                        false
                    } else {
                        // Array element read `a[<literal/identifier index>]`.
                        self.array_read_base_name(node.children[0])
                            .is_some_and(|name| {
                                self.array_elem_repr(&name) == kali_common::Repr::F64
                            })
                    }
                }
                2 => {
                    let text = node.text.as_deref().unwrap_or_default();
                    if is_binary_operator_text(text) {
                        match text {
                            "/" => {
                                !(self.is_bigint_literal_valued(node.children[0])
                                    && self.is_bigint_literal_valued(node.children[1]))
                            }
                            "+" | "-" | "*" => {
                                self.is_float_valued(node.children[0])
                                    || self.is_float_valued(node.children[1])
                            }
                            _ => false,
                        }
                    } else {
                        // Computed array element read `a[<expr>]`.
                        self.array_read_base_name(node.children[0])
                            .is_some_and(|name| {
                                self.array_elem_repr(&name) == kali_common::Repr::F64
                            })
                    }
                }
                3 => {
                    // Ternary `test ? a : b` (marker text "?"): float-valued
                    // iff either arm is — mirrors `emit_conditional`'s
                    // `float_result` so a store into an f64 local promotes once
                    // (inside the arms) and this site never double-converts.
                    node.text.as_deref() == Some("?")
                        && (self.is_float_valued(node.children[1])
                            || self.is_float_valued(node.children[2]))
                }
                _ => false,
            },
            _ => false,
        }
    }

    /// Emits operand `id`, inserting an `f64.convert_i64_s` promotion when the
    /// surrounding operation is float-typed (`float_op`) but this operand is itself
    /// integer-valued. Per-side so mixed `int <op> float` operands both land as
    /// `f64` on the stack before the float instruction.
    pub(crate) fn emit_float_operand(
        &mut self,
        function: &mut Function,
        id: LirNodeId,
        float_op: bool,
    ) {
        let operand_is_float = self.is_float_valued(id);
        let _ = self.emit_node(function, id, true);
        if float_op && !operand_is_float {
            function.instruction(&Instruction::F64ConvertI64S);
        }
    }

    /// Lowers a bitwise-integer operator with JS 32-bit semantics: both operands
    /// are `ToInt32`-coerced (`i32.wrap_i64`), the op runs on `i32` (wasm masks
    /// shift counts mod 32, matching JS `& 31`), and the result extends back to
    /// `i64` — sign-extended for every op except `>>>`, which zero-extends
    /// (uint32). Float operands are rejected before this point (see `emit_binary`).
    fn emit_bitwise(
        &mut self,
        function: &mut Function,
        op: &str,
        left: LirNodeId,
        right: LirNodeId,
    ) -> EmittedValue {
        self.emit_float_operand(function, left, false);
        function.instruction(&Instruction::I32WrapI64);
        self.emit_float_operand(function, right, false);
        function.instruction(&Instruction::I32WrapI64);
        match op {
            "&" => function.instruction(&Instruction::I32And),
            "|" => function.instruction(&Instruction::I32Or),
            "^" => function.instruction(&Instruction::I32Xor),
            "<<" => function.instruction(&Instruction::I32Shl),
            ">>" => function.instruction(&Instruction::I32ShrS),
            ">>>" => function.instruction(&Instruction::I32ShrU),
            _ => unreachable!("emit_bitwise called with non-bitwise op"),
        };
        if op == ">>>" {
            function.instruction(&Instruction::I64ExtendI32U);
        } else {
            function.instruction(&Instruction::I64ExtendI32S);
        }
        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
    }

    /// Emits `id` as a string handle: if it is already string-valued the emitted
    /// value is a handle; float-shaped values are stringified via
    /// `float_to_string` (JS `String(number)` semantics); otherwise the produced
    /// i64 is coerced to a decimal-string handle via `int_to_string`.
    pub(crate) fn emit_as_string(&mut self, function: &mut Function, id: LirNodeId) {
        let is_string = self.is_string_valued(id);
        let emitted = self.emit_node(function, id, true);
        if !emitted.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        if is_string {
            return;
        }
        if emitted.produced
            && (matches!(emitted.shape, ValueShape::Float) || self.is_float_valued(id))
        {
            function.instruction(&Instruction::Call(FLOAT_TO_STRING_IMPORT_INDEX));
        } else {
            function.instruction(&Instruction::Call(INT_TO_STRING_IMPORT_INDEX));
        }
    }

    pub(crate) fn emit_binary(&mut self, function: &mut Function, node: &LirNode) -> EmittedValue {
        let op = node.text.as_deref().unwrap_or_default();
        let left = node.children[0];
        let right = node.children[1];

        if self.emit_assignment(function, node, op, left, right) {
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        }

        // Object misuse gate: a genuine arithmetic/comparison operator applied
        // to an object reference (e.g. `p + 1`) would silently operate on the
        // raw pointer. `=` and the compound-assignment operators are handled
        // above by `emit_assignment` (which returns `true` and short-circuits
        // before this point) and legitimately support object-reference
        // reassignment (`q = p`, aliasing), so they never reach here.
        if self.object_shape_of_node(left).is_some() || self.object_shape_of_node(right).is_some() {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "operator '{op}' on an object reference is unavailable in the current phase; operate on its fields instead"
                ),
            ));
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if matches!(op, "===" | "!==") {
            if let (Some(left_value), Some(right_value)) = (
                self.static_bigint_literal_value(left),
                self.static_bigint_literal_value(right),
            ) {
                function.instruction(&Instruction::I64Const(
                    if (left_value == right_value) == (op == "===") {
                        1
                    } else {
                        0
                    },
                ));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                };
            }
        }

        if matches!(op, "<" | "<=" | ">" | ">=") {
            if let Some(result) = self.static_ascii_string_relational_result(left, right, op) {
                function.instruction(&Instruction::I64Const(if result { 1 } else { 0 }));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                };
            }
        }

        // String-typed `+`: if either operand is a string value, this is a string
        // concatenation, not integer addition. Both operands are coerced to string
        // handles (integers via `int_to_string`) and joined with `string_concat`.
        // Static constant folds happen earlier in LIR, so only unfolded operands
        // reach this path.
        if op == "+" && (self.is_string_valued(left) || self.is_string_valued(right)) {
            self.emit_as_string(function, left);
            self.emit_as_string(function, right);
            function.instruction(&Instruction::Call(STRING_CONCAT_IMPORT_INDEX));
            return EmittedValue {
                produced: true,
                shape: ValueShape::String,
            };
        }

        // A string operand in a NON-`+` position has no correct lowering here.
        // Static string folds have already returned above (relational literal
        // folds, `===`/`!==` bigint folds), so a string operand reaching this
        // point is a genuine runtime value. Reject fail-closed (a wrong runtime
        // result is worse than a compile error):
        //   - Relational / arithmetic / bitwise / logical: compare or combine
        //     RAW handles — always wrong. Reject ANY string-valued operand.
        //   - Equality (`== != === !==`): identity-comparing handles is correct
        //     ONLY for interned literal constants; a fresh runtime concat handle
        //     is not the interned handle of the same text. Reject only a tainted
        //     (runtime-concat-derived) operand, preserving `s == "hi"` etc.
        // NOTE: `&&`/`||`/`??` are deliberately EXCLUDED — they are
        // value-SELECTING (return one operand unchanged, a valid string result)
        // and are statically folded in string-fold positions (e.g. dynamic
        // import specifiers). The string+int selection hazard is caught by the
        // repr inference's `merge_nodes` guard, not here.
        let is_equality = matches!(op, "==" | "!=" | "===" | "!==");
        let is_order_or_arith = matches!(
            op,
            "-" | "*"
                | "/"
                | "%"
                | "**"
                | "&"
                | "|"
                | "^"
                | "<<"
                | ">>"
                | ">>>"
                | "<"
                | "<="
                | ">"
                | ">="
        );
        if is_equality || is_order_or_arith {
            let reject = if is_equality {
                self.is_runtime_concat_string(left) || self.is_runtime_concat_string(right)
            } else {
                self.is_string_valued(left) || self.is_string_valued(right)
            };
            if reject {
                self.diagnostics.push(Diagnostic::error(
                    e3::TYPE_MISMATCH as u32,
                    format!(
                        "operator '{op}' on a runtime string value is unavailable in the current direct-runtime path; only string concatenation with '+' is lowered, and a runtime string cannot be compared, ordered, arithmetic-combined, or truthiness-tested here"
                    ),
                ));
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }
        }

        // Repr-directed float selection. Arithmetic `+ - *` is float when either
        // operand is float; `/` is always float (JS division yields a double in
        // this model); relational ops compare as doubles when either operand is
        // float. `%`, logical, `??`, and `**` stay on the integer path. For an
        // all-integer program every operand is integer-valued and `/` never
        // reaches here (no float seeds), so `float_op` is always false and the
        // emitted code is byte-identical to the pre-repr path.
        let operand_float = self.is_float_valued(left) || self.is_float_valued(right);
        let is_bitwise = matches!(op, "&" | "|" | "^" | "<<" | ">>" | ">>>");
        if is_bitwise && operand_float {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "bitwise operator '{op}' on a floating-point operand is unavailable in the current phase; use integer operands or the later compatibility path"
                ),
            ));
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }
        let float_op = match op {
            // `/` is float (JS division yields a double in this model) UNLESS
            // both operands are BigInt literals: BigInt division truncates
            // toward zero and must stay on the i64 lane (`i64.div_s`).
            "/" => !(self.is_bigint_literal_valued(left) && self.is_bigint_literal_valued(right)),
            "+" | "-" | "*" => operand_float,
            "<" | "<=" | ">" | ">=" | "==" | "===" | "!=" | "!==" => operand_float,
            _ => false,
        };

        if op != "??" && op != "**" && !is_bitwise {
            self.emit_float_operand(function, left, float_op);
            self.emit_float_operand(function, right, float_op);
        }

        match op {
            "+" => {
                if float_op {
                    function.instruction(&Instruction::F64Add);
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Float,
                    }
                } else {
                    function.instruction(&Instruction::I64Add);
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    }
                }
            }
            "-" => {
                if float_op {
                    function.instruction(&Instruction::F64Sub);
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Float,
                    }
                } else {
                    function.instruction(&Instruction::I64Sub);
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    }
                }
            }
            "*" => {
                if float_op {
                    function.instruction(&Instruction::F64Mul);
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Float,
                    }
                } else {
                    function.instruction(&Instruction::I64Mul);
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    }
                }
            }
            "/" => {
                if float_op {
                    function.instruction(&Instruction::F64Div);
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Float,
                    }
                } else {
                    // BigInt `/`: truncation toward zero is exactly `i64.div_s`.
                    function.instruction(&Instruction::I64DivS);
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    }
                }
            }
            "%" => {
                function.instruction(&Instruction::I64RemS);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                }
            }
            "==" | "===" => {
                if float_op {
                    function.instruction(&Instruction::F64Eq);
                } else {
                    function.instruction(&Instruction::I64Eq);
                }
                function.instruction(&Instruction::I64ExtendI32U);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                }
            }
            "!=" | "!==" => {
                if float_op {
                    function.instruction(&Instruction::F64Ne);
                } else {
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::I32Eqz);
                }
                function.instruction(&Instruction::I64ExtendI32U);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                }
            }
            "<" => {
                if float_op {
                    function.instruction(&Instruction::F64Lt);
                } else {
                    function.instruction(&Instruction::I64LtS);
                }
                function.instruction(&Instruction::I64ExtendI32U);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                }
            }
            "<=" => {
                if float_op {
                    function.instruction(&Instruction::F64Le);
                } else {
                    function.instruction(&Instruction::I64LeS);
                }
                function.instruction(&Instruction::I64ExtendI32U);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                }
            }
            ">" => {
                if float_op {
                    function.instruction(&Instruction::F64Gt);
                } else {
                    function.instruction(&Instruction::I64GtS);
                }
                function.instruction(&Instruction::I64ExtendI32U);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                }
            }
            ">=" => {
                if float_op {
                    function.instruction(&Instruction::F64Ge);
                } else {
                    function.instruction(&Instruction::I64GeS);
                }
                function.instruction(&Instruction::I64ExtendI32U);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                }
            }
            "&&" => {
                function.instruction(&Instruction::I64And);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                }
            }
            "||" => {
                function.instruction(&Instruction::I64Or);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                }
            }
            "**" => self.emit_exponentiation_expression(
                function,
                &[left, right],
                "Exponentiation operator '**'",
            ),
            "??" => {
                let left = node.children[0];
                let right = node.children[1];
                let left_result = self.emit_node(function, left, true);
                if !left_result.produced {
                    function.instruction(&Instruction::I64Const(0));
                }
                let temp_local = self.locals.len() as u32;
                function.instruction(&Instruction::LocalSet(temp_local));
                function.instruction(&Instruction::LocalGet(temp_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                let right_result = self.emit_node(function, right, true);
                if !right_result.produced {
                    function.instruction(&Instruction::I64Const(0));
                }
                function.instruction(&Instruction::Else);
                function.instruction(&Instruction::LocalGet(temp_local));
                function.instruction(&Instruction::End);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                }
            }
            "&" | "|" | "^" | "<<" | ">>" | ">>>" => self.emit_bitwise(function, op, left, right),
            _ => {
                self.diagnostics.push(Diagnostic::warning(
                    e8::UNIMPLEMENTED as u32,
                    format!("unsupported binary operator '{}'", op),
                ));
                function.instruction(&Instruction::I64Add);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                }
            }
        }
    }

    pub(crate) fn emit_exponentiation_expression(
        &mut self,
        function: &mut Function,
        operands: &[LirNodeId],
        label: &str,
    ) -> EmittedValue {
        let Some(base) = operands.first() else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "{label} requires at least two operands in the current phase; use explicit operands or the later compatibility path"
                ),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        };
        let Some(exponent) = operands.get(1) else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "{label} requires at least two operands in the current phase; use explicit operands or the later compatibility path"
                ),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        };

        let base_unit = self
            .render_static_value(*base)
            .and_then(|rendered| parse_numeric_literal_value(&rendered))
            .is_some_and(|value| value == 1.0 || value == -1.0);
        if self.contains_negative_numeric_literal(*exponent) && !base_unit {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "{label} is unavailable for negative numeric literals unless the base is a statically-known ±1 in the current phase; use a non-negative exponent or the later compatibility path"
                ),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        let base_numeric_value = self
            .render_static_value(*base)
            .and_then(|rendered| parse_numeric_literal_value(&rendered));
        let exponent_numeric_value = self
            .render_static_value(*exponent)
            .and_then(|rendered| parse_numeric_literal_value(&rendered));
        let base_zero = base_numeric_value.is_some_and(|value| value == 0.0);
        let exponent_positive_integer =
            exponent_numeric_value.is_some_and(|value| value > 0.0 && value.fract() == 0.0);
        let exponent_integer = exponent_numeric_value.is_some_and(|value| value.fract() == 0.0);
        if base_zero && exponent_positive_integer {
            let _ = self.emit_node(function, *base, true);
            function.instruction(&Instruction::Drop);
            let _ = self.emit_node(function, *exponent, true);
            function.instruction(&Instruction::Drop);
            for arg in operands.iter().skip(2) {
                let produced = self.emit_node(function, *arg, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
            }
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        let base_identity =
            base_numeric_value.filter(|value| *value == 0.0 || *value == 1.0 || *value == -1.0);
        let exponent_identity =
            exponent_numeric_value.filter(|value| *value == 0.0 || *value == 1.0);

        if let Some(base_identity) = base_identity {
            if base_identity == 1.0 {
                let _ = self.emit_node(function, *base, true);
                function.instruction(&Instruction::Drop);
                let produced = self.emit_node(function, *exponent, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
                for arg in operands.iter().skip(2) {
                    let produced = self.emit_node(function, *arg, true);
                    if produced.produced {
                        function.instruction(&Instruction::Drop);
                    }
                }
                function.instruction(&Instruction::I64Const(1));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if base_identity == -1.0 && exponent_integer {
                let _ = self.emit_node(function, *base, true);
                function.instruction(&Instruction::Drop);
                let produced = self.emit_node(function, *exponent, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
                for arg in operands.iter().skip(2) {
                    let produced = self.emit_node(function, *arg, true);
                    if produced.produced {
                        function.instruction(&Instruction::Drop);
                    }
                }
                let folded = if exponent_numeric_value
                    .is_some_and(|value| value.abs().rem_euclid(2.0) == 0.0)
                {
                    1
                } else {
                    -1
                };
                function.instruction(&Instruction::I64Const(folded));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if let Some(exponent) = exponent_identity {
                if exponent == 0.0 {
                    let _ = self.emit_node(function, *base, true);
                    function.instruction(&Instruction::Drop);
                    for arg in operands.iter().skip(2) {
                        let produced = self.emit_node(function, *arg, true);
                        if produced.produced {
                            function.instruction(&Instruction::Drop);
                        }
                    }
                    function.instruction(&Instruction::I64Const(1));
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    };
                }
            }
        }

        if let Some(exponent_identity) = exponent_identity {
            match exponent_identity {
                0.0 => {
                    let base_result = self.emit_node(function, *base, true);
                    if base_result.produced {
                        function.instruction(&Instruction::Drop);
                    }
                    for arg in operands.iter().skip(2) {
                        let produced = self.emit_node(function, *arg, true);
                        if produced.produced {
                            function.instruction(&Instruction::Drop);
                        }
                    }
                    function.instruction(&Instruction::I64Const(1));
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    };
                }
                1.0 => {
                    if !self.emit_integer_math_arg(function, *base, "pow") {
                        return EmittedValue {
                            produced: false,
                            shape: ValueShape::Unknown,
                        };
                    }
                    for arg in operands.iter().skip(2) {
                        if !self.emit_integer_math_arg(function, *arg, "pow") {
                            return EmittedValue {
                                produced: false,
                                shape: ValueShape::Unknown,
                            };
                        }
                        function.instruction(&Instruction::Drop);
                    }
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    };
                }
                _ => unreachable!(),
            }
        }

        if !self.emit_integer_math_arg(function, *base, "pow") {
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }
        if !self.emit_integer_math_arg(function, *exponent, "pow") {
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }
        function.instruction(&Instruction::Call(MATH_POW_IMPORT_INDEX));
        for arg in operands.iter().skip(2) {
            if !self.emit_integer_math_arg(function, *arg, "pow") {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
            function.instruction(&Instruction::Drop);
        }
        EmittedValue {
            produced: true,
            shape: ValueShape::Scalar,
        }
    }

    pub(crate) fn perfect_square_root_i128(&self, value: i128) -> Option<i64> {
        if value < 0 {
            return None;
        }

        let mut low = 0_i128;
        let mut high = i128::from(i64::MAX).min(value);
        while low <= high {
            let mid = low + (high - low) / 2;
            let square = mid.checked_mul(mid)?;
            if square == value {
                return Some(mid as i64);
            }
            if square < value {
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        }

        None
    }
}

#[cfg(test)]
#[path = "operators_tests.rs"]
mod operators_tests;
