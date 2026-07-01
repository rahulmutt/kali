use crate::*;

impl<'a> FunctionEmitter<'a> {
    pub(crate) fn emit_break_or_continue(
        &mut self,
        function: &mut Function,
        is_continue: bool,
        node: &LirNode,
    ) -> EmittedValue {
        let Some(loop_frame) = self.loop_frames.last().copied() else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "break and continue are unavailable outside the supported static loop lowering path; use a supported loop form or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        };

        let control_text = node.text.as_deref().unwrap_or_default();
        let (keyword, label) = if let Some(label) = control_text.strip_prefix("break:") {
            ("break", label)
        } else if let Some(label) = control_text.strip_prefix("continue:") {
            ("continue", label)
        } else if control_text == "break" {
            ("break", "")
        } else if control_text == "continue" {
            ("continue", "")
        } else {
            (control_text, "")
        };

        if !label.is_empty() {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "{keyword} labels are unavailable in the current phase; use an unlabeled {keyword} inside the supported static loop lowering path or the later compatibility path"
                ),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        let target_index = if is_continue {
            loop_frame.continue_index
        } else {
            loop_frame.break_index
        };
        let depth = self.control_frame_depth(target_index);
        function.instruction(&Instruction::Br(depth));
        EmittedValue {
            produced: false,
            shape: ValueShape::Unknown,
        }
    }

    pub(crate) fn emit_return(&mut self, function: &mut Function, node: &LirNode) -> EmittedValue {
        if let Some(arg) = node.children.first().copied() {
            let produced = self.emit_node(function, arg, true);
            if !produced.produced {
                function.instruction(&Instruction::I64Const(0));
            }
        } else {
            function.instruction(&Instruction::I64Const(0));
        }
        function.instruction(&Instruction::Return);
        EmittedValue {
            produced: false,
            shape: ValueShape::Unknown,
        }
    }

    /// Lowers `while` / `do-while` / `for` to a real wasm `block { loop { ... } }`
    /// with a back-edge, so the body can run more than once and `break`/`continue`
    /// resolve via the loop-frame stack.
    ///
    /// Known limitation: in a `for` loop, `update` is emitted at the tail of the
    /// body (since wasm's `loop` has no native init/update clause), so a
    /// `continue` re-enters the loop *without* running `update`. `while` /
    /// `do-while` re-test correctly on `continue` since they have no `update`.
    pub(crate) fn emit_loop(&mut self, function: &mut Function, node: &LirNode) -> EmittedValue {
        let kind = node.text.as_deref().unwrap_or_default();

        // Resolve clauses by loop kind.
        let (init, test, update, body) = match kind {
            "while" => (
                None,
                node.children.first().copied(),
                None,
                node.children.get(1).copied(),
            ),
            "do-while" => (
                None,
                node.children.get(1).copied(),
                None,
                node.children.first().copied(),
            ),
            _ /* "for" */ => {
                // [init?, test?, update?, body] — body is always last; classify by count.
                let n = node.children.len();
                let body = node.children.last().copied();
                let (init, test, update) = match n {
                    1 => (None, None, None),
                    2 => (None, node.children.first().copied(), None),
                    3 => (
                        node.children.first().copied(),
                        node.children.get(1).copied(),
                        None,
                    ),
                    _ => (
                        node.children.first().copied(),
                        node.children.get(1).copied(),
                        node.children.get(2).copied(),
                    ),
                };
                (init, test, update, body)
            }
        };

        // for-init runs once, before the loop.
        if let Some(init) = init {
            let produced = self.emit_node(function, init, false);
            if produced.produced {
                function.instruction(&Instruction::Drop);
            }
        }

        let break_index = self.push_control_frame(ControlFlowLabelKind::LoopBreak);
        function.instruction(&Instruction::Block(BlockType::Empty));
        let continue_index = self.push_control_frame(ControlFlowLabelKind::LoopContinue);
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.loop_frames.push(LoopFrame {
            break_index,
            continue_index,
        });

        let emit_body_and_update = |emitter: &mut Self, function: &mut Function| {
            if let Some(body) = body {
                let produced = emitter.emit_node(function, body, false);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
            }
            if let Some(update) = update {
                let produced = emitter.emit_node(function, update, false);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
            }
        };

        if kind == "do-while" {
            // body first, then test at the bottom.
            emit_body_and_update(self, function);
            if let Some(test) = test {
                let cond = self.emit_node(function, test, true);
                if !cond.produced {
                    function.instruction(&Instruction::I64Const(0));
                }
            } else {
                function.instruction(&Instruction::I64Const(1));
            }
            function.instruction(&Instruction::I64Eqz); // 1 if falsy
            function.instruction(&Instruction::I32Eqz); // invert: 1 if truthy
            function.instruction(&Instruction::BrIf(0)); // continue if truthy
        } else {
            // test at the top; exit (break) when falsy.
            if let Some(test) = test {
                let cond = self.emit_node(function, test, true);
                if !cond.produced {
                    function.instruction(&Instruction::I64Const(0));
                }
                function.instruction(&Instruction::I64Eqz); // 1 if falsy
                function.instruction(&Instruction::BrIf(1)); // break out of `block` when falsy
            }
            emit_body_and_update(self, function);
            function.instruction(&Instruction::Br(0)); // back to loop top
        }

        function.instruction(&Instruction::End); // end loop
        self.loop_frames.pop();
        self.pop_control_frame(ControlFlowLabelKind::LoopContinue);
        function.instruction(&Instruction::End); // end block
        self.pop_control_frame(ControlFlowLabelKind::LoopBreak);

        EmittedValue {
            produced: false,
            shape: ValueShape::Unknown,
        }
    }

    pub(crate) fn emit_function_body(
        &mut self,
        function: &mut Function,
        body: LirNodeId,
        returns_value: bool,
        coverage_id: Option<u32>,
    ) {
        self.emit_coverage_hit(function, coverage_id);
        let produced = self.emit_node(function, body, returns_value);
        if returns_value && !produced.produced {
            // Fallthrough value must match the function's declared result type: an
            // f64-returning function needs an f64 zero here (this trailing value
            // still has to type-check even when the body always returns early).
            if self.repr_table.return_repr(&self.function_name) == kali_common::Repr::F64 {
                function.instruction(&Instruction::F64Const(0.0.into()));
            } else {
                function.instruction(&Instruction::I64Const(0));
            }
        } else if !returns_value && produced.produced {
            function.instruction(&Instruction::Drop);
        }
    }

    pub(crate) fn emit_sequence(
        &mut self,
        function: &mut Function,
        children: &[LirNodeId],
        want_value: bool,
    ) -> EmittedValue {
        let mut final_value = EmittedValue {
            produced: false,
            shape: ValueShape::Unknown,
        };
        for (idx, child) in children.iter().enumerate() {
            let child_want_value = want_value && idx + 1 == children.len();
            let child_result = self.emit_node(function, *child, child_want_value);
            if child_result.produced && !child_want_value {
                function.instruction(&Instruction::Drop);
            }
            if child_want_value {
                final_value = child_result;
            }
        }

        if want_value {
            final_value
        } else {
            EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            }
        }
    }

    pub(crate) fn emit_node(
        &mut self,
        function: &mut Function,
        id: LirNodeId,
        want_value: bool,
    ) -> EmittedValue {
        let node = self.node(id).clone();
        match node.kind {
            LirNodeKind::Program | LirNodeKind::Block => {
                self.emit_sequence(function, &node.children, want_value)
            }
            LirNodeKind::Instruction => {
                if matches!(node.text.as_deref(), Some("const" | "let" | "var")) {
                    let is_const = node.text.as_deref() == Some("const");
                    for declarator_id in &node.children {
                        let declarator = self.node(*declarator_id).clone();
                        if declarator.children.len() < 2 {
                            continue;
                        }
                        let init = declarator.children[1];

                        // `new Array(n)` allocations need a stable handle held in a
                        // local slot regardless of `const`/`let`, so the binding can be
                        // read and written through linear memory.
                        if let Some(size_arg) = self.resolve_array_alloc_call(init) {
                            let allocated = self.emit_array_allocation(function, size_arg);
                            if !allocated.produced {
                                function.instruction(&Instruction::I64Const(0));
                            }
                            if let Some(name) = declarator.text.clone() {
                                if let Some(index) = self.locals.get(&name).copied() {
                                    function.instruction(&Instruction::LocalSet(index));
                                    self.array_bindings.insert(name);
                                    continue;
                                }
                            }
                            function.instruction(&Instruction::Drop);
                            continue;
                        }

                        let init_result = self.emit_node(function, init, true);
                        // A named scalar local whose chosen repr is F64 must receive an
                        // f64 on the stack; promote an integer-valued init before the store.
                        let f64_local = match declarator.text.as_deref() {
                            Some(name) => {
                                self.locals.contains_key(name)
                                    && self.scalar_repr(name) == kali_common::Repr::F64
                            }
                            None => false,
                        };
                        if !init_result.produced {
                            if f64_local {
                                function.instruction(&Instruction::F64Const(0.0.into()));
                            } else {
                                function.instruction(&Instruction::I64Const(0));
                            }
                        } else if f64_local && !self.is_float_valued(init) {
                            function.instruction(&Instruction::F64ConvertI64S);
                        }
                        if let Some(name) = declarator.text.clone() {
                            if let Some(index) = self.locals.get(&name).copied() {
                                // `let`/`var`, or a `const` promoted to a local slot
                                // (array allocation / array read) — store eagerly.
                                function.instruction(&Instruction::LocalSet(index));
                            } else if is_const {
                                self.bindings.insert(name, declarator.children[1]);
                                function.instruction(&Instruction::Drop);
                            } else {
                                function.instruction(&Instruction::Drop);
                            }
                        } else {
                            function.instruction(&Instruction::Drop);
                        }
                    }
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                }
                if is_function_like(&self.program.nodes, id) {
                    // Function declarations are emitted separately from the body scan.
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                }
                self.emit_sequence(function, &node.children, false)
            }
            LirNodeKind::Literal => emit_literal(function, node.text.as_deref(), self.strings),
            LirNodeKind::Value => self.emit_value(function, &node, want_value),
            LirNodeKind::Call => self.emit_call(function, &node),
            LirNodeKind::Branch => match node.text.as_deref() {
                Some(text) if text.starts_with("break") => {
                    self.emit_break_or_continue(function, false, &node)
                }
                Some(text) if text.starts_with("continue") => {
                    self.emit_break_or_continue(function, true, &node)
                }
                Some("for-of") | Some("for-await-of") => {
                    self.emit_for_of_array_iteration(function, &node)
                }
                Some("return") => self.emit_return(function, &node),
                Some("while") | Some("do-while") | Some("for") => self.emit_loop(function, &node),
                _ => self.emit_branch(function, &node, want_value),
            },
            LirNodeKind::Unknown => {
                function.instruction(&Instruction::Unreachable);
                EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                }
            }
        }
    }

    pub(crate) fn emit_value(
        &mut self,
        function: &mut Function,
        node: &LirNode,
        want_value: bool,
    ) -> EmittedValue {
        if node.text.is_none() {
            return self.emit_aggregate_literal(function, node, want_value);
        }

        if self.is_supported_callable_reference(node) {
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        }

        match node.children.len() {
            0 => {
                if let Some(text) = node.text.as_deref() {
                    if let Some(index) = self.locals.get(text).copied() {
                        function.instruction(&Instruction::LocalGet(index));
                        return EmittedValue {
                            produced: true,
                            shape: ValueShape::Unknown,
                        };
                    }

                    if let Some(bound) = self.bindings.get(text).copied() {
                        return self.emit_node(function, bound, want_value);
                    }

                    if let Some(constant) = parse_number_literal(text) {
                        function.instruction(&Instruction::I64Const(constant));
                        return EmittedValue {
                            produced: true,
                            shape: ValueShape::Scalar,
                        };
                    }

                    match text {
                        "true" => {
                            function.instruction(&Instruction::I64Const(1));
                            return EmittedValue {
                                produced: true,
                                shape: ValueShape::Boolean,
                            };
                        }
                        "false" | "null" | "undefined" => {
                            function.instruction(&Instruction::I64Const(0));
                            return EmittedValue {
                                produced: true,
                                shape: ValueShape::Boolean,
                            };
                        }
                        "Set" | "Map" => {
                            function.instruction(&Instruction::I64Const(0));
                            return EmittedValue {
                                produced: true,
                                shape: ValueShape::Unknown,
                            };
                        }
                        _ => {}
                    }

                    self.push_placeholder_fallback_diagnostic("identifier", text);
                    function.instruction(&Instruction::I64Const(0));
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Unknown,
                    }
                } else {
                    function.instruction(&Instruction::I64Const(0));
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Unknown,
                    }
                }
            }
            1 => {
                if let Some(result) = self.resolve_static_index_member(node) {
                    return self.emit_static_index_member_result(function, result);
                }

                // Dynamic array element read: `a[i]` where `a` is a linear-memory array.
                if let Some(index_text) = node.text.as_deref() {
                    if !index_text.is_empty() {
                        let base_id = node.children[0];
                        if let Some(base_name) = self.assignment_target_name(node, base_id) {
                            if self.array_bindings.contains(&base_name) {
                                return self.emit_dynamic_array_read(
                                    function, base_id, index_text, &base_name,
                                );
                            }
                        }
                    }
                }

                if node.text.as_deref().unwrap_or_default().is_empty() {
                    self.emit_node(function, node.children[0], want_value)
                } else {
                    self.emit_unary(function, node)
                }
            }
            2 => {
                // A computed member access `a[<expr>]` also lowers to a two-child
                // `Value` node (`[object, index]`). A binary expression always
                // carries an operator `text`, while a computed index never
                // stringifies to a bare operator, so `text` cleanly separates the
                // two shapes.
                if is_binary_operator_text(node.text.as_deref().unwrap_or_default()) {
                    return self.emit_binary(function, node);
                }

                if let Some(result) = self.resolve_static_index_member(node) {
                    return self.emit_static_index_member_result(function, result);
                }

                // Dynamic linear-memory read `a[<expr>]` when the base is an array
                // binding; otherwise fall back to member handling (e.g. host
                // member chains such as `globalThis["process"]["pid"]`), matching
                // the single-child member path.
                let base_id = node.children[0];
                let index_id = node.children[1];
                if let Some(base_name) = self.assignment_target_name(node, base_id) {
                    if self.array_bindings.contains(&base_name) {
                        return self
                            .emit_dynamic_array_read_node(function, base_id, index_id, &base_name);
                    }
                }

                self.emit_unary(function, node)
            }
            _ => {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "array/object aggregate lowering is unavailable in the current phase; use a supported literal shape or the later compatibility path".to_string(),
                ));
                function.instruction(&Instruction::Unreachable);
                EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                }
            }
        }
    }

    pub(crate) fn emit_static_index_member_result(
        &mut self,
        function: &mut Function,
        result: StaticIndexMemberResult,
    ) -> EmittedValue {
        match result {
            StaticIndexMemberResult::Node(value) => self.emit_node(function, value, true),
            StaticIndexMemberResult::String(value) => {
                let (offset, len) = self.strings.intern(&value);
                function.instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                }
            }
            StaticIndexMemberResult::Undefined => {
                function.instruction(&Instruction::I64Const(0));
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                }
            }
        }
    }

    pub(crate) fn for_of_binding_name(&self, node: &LirNode) -> Option<String> {
        let left = node.children.first().copied()?;
        self.for_of_binding_name_from_node(left)
    }

    pub(crate) fn for_of_binding_name_from_node(&self, id: LirNodeId) -> Option<String> {
        let node = self.node(id);
        if node.children.is_empty() {
            return node.text.clone();
        }

        if matches!(node.text.as_deref(), Some("const" | "let" | "var")) {
            let declarator = node.children.first().copied()?;
            return self.node(declarator).text.clone();
        }

        if node.text.as_deref().is_some_and(|text| text.is_empty()) && !node.children.is_empty() {
            return self
                .for_of_binding_name_from_node(*node.children.last().expect("wrapper child"));
        }

        if node.text.is_none() && node.children.len() == 1 {
            return self.for_of_binding_name_from_node(node.children[0]);
        }

        None
    }

    pub(crate) fn emit_branch(
        &mut self,
        function: &mut Function,
        node: &LirNode,
        want_value: bool,
    ) -> EmittedValue {
        let Some(cond) = node.children.first().copied() else {
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        };

        let then_branch = node.children.get(1).copied();
        let else_branch = node.children.get(2).copied();

        let condition = self.emit_node(function, cond, true);
        if !condition.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        match condition.shape {
            ValueShape::Boolean => {
                function.instruction(&Instruction::I32WrapI64);
            }
            ValueShape::Scalar | ValueShape::Unknown | ValueShape::String => {
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
            }
            ValueShape::Float => {
                // f64 truthiness: nonzero is truthy. Leaves an i32 for `If`.
                function.instruction(&Instruction::F64Const(0.0.into()));
                function.instruction(&Instruction::F64Ne);
            }
        }
        let if_index = self.push_control_frame(ControlFlowLabelKind::If);
        function.instruction(&Instruction::If(if want_value {
            BlockType::Result(ValType::I64)
        } else {
            BlockType::Empty
        }));

        if let Some(then_branch) = then_branch {
            let produced = self.emit_node(function, then_branch, want_value);
            if want_value && !produced.produced {
                function.instruction(&Instruction::I64Const(0));
            } else if !want_value && produced.produced {
                function.instruction(&Instruction::Drop);
            }
        } else if want_value {
            function.instruction(&Instruction::I64Const(0));
        }

        if let Some(else_branch) = else_branch {
            function.instruction(&Instruction::Else);
            let produced = self.emit_node(function, else_branch, want_value);
            if want_value && !produced.produced {
                function.instruction(&Instruction::I64Const(0));
            } else if !want_value && produced.produced {
                function.instruction(&Instruction::Drop);
            }
        } else if want_value {
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(0));
        }

        function.instruction(&Instruction::End);
        self.pop_control_frame(ControlFlowLabelKind::If);
        debug_assert!(self.control_frames.get(if_index).is_none());
        EmittedValue {
            produced: want_value,
            shape: ValueShape::Unknown,
        }
    }
}

#[cfg(test)]
#[path = "control_flow_tests.rs"]
mod control_flow_tests;
