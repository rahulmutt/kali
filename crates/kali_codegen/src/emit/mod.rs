//! Core emission methods for FunctionEmitter.

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
            function.instruction(&Instruction::I64Const(0));
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
                        let init_result = self.emit_node(function, init, true);
                        if !init_result.produced {
                            function.instruction(&Instruction::I64Const(0));
                        }
                        if let Some(name) = declarator.text.clone() {
                            if is_const {
                                self.bindings.insert(name, declarator.children[1]);
                                function.instruction(&Instruction::Drop);
                            } else if let Some(index) = self.locals.get(&name).copied() {
                                function.instruction(&Instruction::LocalSet(index));
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
                    return match result {
                        StaticIndexMemberResult::Node(value) => {
                            self.emit_node(function, value, true)
                        }
                        StaticIndexMemberResult::String(value) => {
                            let (offset, len) = self.strings.intern(&value);
                            function.instruction(&Instruction::I64Const(encode_string_handle(
                                offset, len,
                            )));
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
                    };
                }

                if node.text.as_deref().unwrap_or_default().is_empty() {
                    self.emit_node(function, node.children[0], want_value)
                } else {
                    self.emit_unary(function, node)
                }
            }
            2 => self.emit_binary(function, node),
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
        match op {
            "prefix++" | "postfix++" | "prefix--" | "postfix--" => {
                self.emit_update_expression(function, node, op, arg)
            }
            "-" => {
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
                if let Some(aggregate_id) = self.resolve_literal_aggregate(arg) {
                    let aggregate = self.node(aggregate_id).clone();
                    if let Some(field_value) = self.object_literal_field(&aggregate, op) {
                        return self.emit_node(function, field_value, true);
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

            return Some(id);
        }
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
                let rhs = self.emit_node(function, right, true);
                if !rhs.produced {
                    function.instruction(&Instruction::I64Const(0));
                }
                function.instruction(&Instruction::LocalSet(index));
                function.instruction(&Instruction::LocalGet(index));
                true
            }
            "??=" => {
                let temp_local = self.locals.len() as u32;
                function.instruction(&Instruction::LocalGet(index));
                function.instruction(&Instruction::LocalSet(temp_local));
                function.instruction(&Instruction::LocalGet(temp_local));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
                let rhs = self.emit_node(function, right, true);
                if !rhs.produced {
                    function.instruction(&Instruction::I64Const(0));
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

        if op != "??" && op != "**" {
            let _ = self.emit_node(function, left, true);
            let _ = self.emit_node(function, right, true);
        }

        match op {
            "+" => {
                function.instruction(&Instruction::I64Add);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                }
            }
            "-" => {
                function.instruction(&Instruction::I64Sub);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                }
            }
            "*" => {
                function.instruction(&Instruction::I64Mul);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                }
            }
            "/" => {
                function.instruction(&Instruction::I64DivS);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
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
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I64ExtendI32U);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                }
            }
            "!=" | "!==" => {
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I32Eqz);
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

    pub(crate) fn emit_call(&mut self, function: &mut Function, node: &LirNode) -> EmittedValue {
        let Some(callee) = node.children.first().copied() else {
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        };

        let callee_node = self
            .resolve_bound_member_callable_node(callee)
            .map(|bound| self.node(bound).clone())
            .unwrap_or_else(|| self.node(callee).clone());
        if self.is_kali_test_call(&callee_node) {
            if let Some(callback_index) = self.kali_test_callback_index(node) {
                function.instruction(&Instruction::I32Const(callback_index as i32));
                function.instruction(&Instruction::Call(TEST_REGISTER_IMPORT_INDEX));
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }

            self.diagnostics.push(Diagnostic::warning(
                e8::IR_UNREADABLE as u32,
                "`Kali.test(...)` requires a function callback lowered as an exported function",
            ));
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        let callee_name = callee_node.text.as_deref().unwrap_or_default();
        let resolved = self.functions.get(callee_name).copied();

        if self.is_console_assert(&callee_node) {
            let message_args: Vec<LirNodeId> = node.children.iter().skip(2).copied().collect();
            let Some(condition) = node.children.get(1).copied() else {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::Else);
                let (offset, len) = self.strings.intern("Assertion failed");
                function.instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
                function.instruction(&Instruction::Call(CONSOLE_ERROR_IMPORT_INDEX));
                function.instruction(&Instruction::End);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            let condition_result = self.emit_node(function, condition, true);
            if !condition_result.produced {
                function.instruction(&Instruction::I64Const(0));
            }
            match condition_result.shape {
                ValueShape::Boolean => {
                    function.instruction(&Instruction::I32WrapI64);
                }
                ValueShape::Scalar | ValueShape::Unknown => {
                    function.instruction(&Instruction::I64Eqz);
                    function.instruction(&Instruction::I32Eqz);
                }
            }
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            if !message_args.is_empty() {
                if let Some(rendered) = self.render_console_arguments(&message_args) {
                    let (offset, len) = self.strings.intern(&rendered);
                    let handle = encode_string_handle(offset, len);
                    function.instruction(&Instruction::I64Const(handle));
                    function.instruction(&Instruction::Call(CONSOLE_ERROR_IMPORT_INDEX));
                } else if let Some(first_arg) = message_args.first().copied() {
                    let _ = self.emit_node(function, first_arg, true);
                    function.instruction(&Instruction::Call(CONSOLE_ERROR_IMPORT_INDEX));
                    for arg in message_args.iter().skip(1) {
                        let _ = self.emit_node(function, *arg, true);
                        function.instruction(&Instruction::Drop);
                    }
                }
            } else {
                let (offset, len) = self.strings.intern("Assertion failed");
                let handle = encode_string_handle(offset, len);
                function.instruction(&Instruction::I64Const(handle));
                function.instruction(&Instruction::Call(CONSOLE_ERROR_IMPORT_INDEX));
            }
            function.instruction(&Instruction::End);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(import_index) = self.console_import_index(&callee_node) {
            if let Some(rendered) = self.render_console_call(node) {
                let (offset, len) = self.strings.intern(&rendered);
                let handle = encode_string_handle(offset, len);
                function.instruction(&Instruction::I64Const(handle));
                function.instruction(&Instruction::Call(import_index));
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }

            let mut args = node.children.iter().skip(1);
            if let Some(first_arg) = args.next() {
                let _ = self.emit_node(function, *first_arg, true);
            } else {
                function.instruction(&Instruction::I64Const(0));
            }
            function.instruction(&Instruction::Call(import_index));
            for arg in args {
                let _ = self.emit_node(function, *arg, true);
                function.instruction(&Instruction::Drop);
            }
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_object_freeze_call(node) {
            let mut args = node.children.iter().skip(1);
            let Some(value) = args.next() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Object.freeze requires at least one argument in the current phase; use an explicit value or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            let emitted = self.emit_node(function, *value, true);
            for arg in args {
                let produced = self.emit_node(function, *arg, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
            }
            return emitted;
        }

        if let Some(result) = self.resolve_static_global_number_predicate_call(node, &callee_node) {
            function.instruction(&Instruction::I64Const(if result { 1 } else { 0 }));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }

        if let Some(result) = self.resolve_static_parse_int_call(node, &callee_node) {
            function.instruction(&Instruction::I64Const(result));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(result) = self.resolve_static_parse_float_call(node, &callee_node) {
            function.instruction(&Instruction::I64Const(result));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(result) = self.resolve_static_string_from_char_code_call(node, &callee_node) {
            let (offset, len) = self.strings.intern(&result);
            function.instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(result) = self.resolve_static_array_some_every_call(node, "some") {
            function.instruction(&Instruction::I64Const(if result { 1 } else { 0 }));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }

        if let Some(result) = self.resolve_static_array_some_every_call(node, "every") {
            function.instruction(&Instruction::I64Const(if result { 1 } else { 0 }));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }

        if let Some(result) = self.resolve_static_array_find_call(node, "find") {
            return match result {
                StaticArraySearchResult::Value(value) => self.emit_node(function, value, true),
                StaticArraySearchResult::Index(index) => {
                    function.instruction(&Instruction::I64Const(index));
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    }
                }
            };
        }

        if let Some(result) = self.resolve_static_array_find_call(node, "findIndex") {
            return match result {
                StaticArraySearchResult::Value(value) => self.emit_node(function, value, true),
                StaticArraySearchResult::Index(index) => {
                    function.instruction(&Instruction::I64Const(index));
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    }
                }
            };
        }

        if let Some(result) = self.resolve_static_array_find_call(node, "findLast") {
            return match result {
                StaticArraySearchResult::Value(value) => self.emit_node(function, value, true),
                StaticArraySearchResult::Index(index) => {
                    function.instruction(&Instruction::I64Const(index));
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    }
                }
            };
        }

        if let Some(result) = self.resolve_static_array_find_call(node, "findLastIndex") {
            return match result {
                StaticArraySearchResult::Value(value) => self.emit_node(function, value, true),
                StaticArraySearchResult::Index(index) => {
                    function.instruction(&Instruction::I64Const(index));
                    EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    }
                }
            };
        }

        if let Some(result) = self.resolve_static_array_search_call(node, "includes") {
            function.instruction(&Instruction::I64Const(if result >= 0 { 1 } else { 0 }));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }

        if let Some(result) = self.resolve_static_array_search_call(node, "indexOf") {
            function.instruction(&Instruction::I64Const(result));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(result) = self.resolve_static_array_search_call(node, "lastIndexOf") {
            function.instruction(&Instruction::I64Const(result));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(result) = self.resolve_static_string_search_call(node, "includes") {
            function.instruction(&Instruction::I64Const(if result >= 0 { 1 } else { 0 }));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }

        if let Some(result) = self.resolve_static_string_search_call(node, "indexOf") {
            function.instruction(&Instruction::I64Const(result));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(result) = self.resolve_static_string_search_call(node, "lastIndexOf") {
            function.instruction(&Instruction::I64Const(result));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(result) = self.resolve_static_string_search_call(node, "startsWith") {
            function.instruction(&Instruction::I64Const(if result >= 0 { 1 } else { 0 }));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }

        if let Some(result) = self.resolve_static_string_search_call(node, "endsWith") {
            function.instruction(&Instruction::I64Const(if result >= 0 { 1 } else { 0 }));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }

        if let Some(result) = self.resolve_static_string_identity_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_slice_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_substring_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_repeat_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_concat_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_pad_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_at_call(node) {
            let literal = match result {
                StaticStringAtResult::Value(value) => self.alloc_scratch_node(
                    LirNodeKind::Literal,
                    Some(quote_string_literal(&value)),
                    vec![],
                ),
                StaticStringAtResult::OutOfRange => self.alloc_scratch_node(
                    LirNodeKind::Literal,
                    Some("undefined".to_string()),
                    vec![],
                ),
            };
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_char_at_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_char_code_at_call(node) {
            let literal = self.alloc_scratch_node(LirNodeKind::Literal, Some(result), vec![]);
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_code_point_at_call(node) {
            let literal = match result {
                StaticStringAtResult::Value(value) => {
                    self.alloc_scratch_node(LirNodeKind::Literal, Some(value), vec![])
                }
                StaticStringAtResult::OutOfRange => self.alloc_scratch_node(
                    LirNodeKind::Literal,
                    Some("undefined".to_string()),
                    vec![],
                ),
            };
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_trim_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_case_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_normalize_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_replace_call(node, "replace") {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_string_replace_call(node, "replaceAll") {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(parts) = self.resolve_static_string_split_call(node) {
            let children = parts
                .into_iter()
                .map(|part| {
                    self.alloc_scratch_node(
                        LirNodeKind::Literal,
                        Some(quote_string_literal(&part)),
                        vec![],
                    )
                })
                .collect();
            let literal = self.alloc_scratch_node(LirNodeKind::Value, None, children);
            return self.emit_node(function, literal, true);
        }

        if let Some(method) = self.string_identity_call_method_with_literal_receiver(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "String.prototype.{method} is unavailable with arguments in the current direct-runtime path; use a no-argument static string call or the later compatibility path"
                ),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_string_repeat_call_with_literal_receiver(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "String.prototype.repeat is unavailable unless the receiver is a statically-known ASCII string literal and the repeat count is a statically-known integer from 0 through 1024 in the current direct-runtime path; use explicit ASCII literals or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_string_concat_call_with_literal_receiver(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "String.prototype.concat is unavailable unless the receiver and all operands are statically-known ASCII string literals in the current direct-runtime path; use explicit ASCII literals or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(method) = self.string_pad_call_method(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "String.prototype.{method} is unavailable unless the receiver is a statically-known ASCII string literal, the target length is a statically-known integer from 0 through 1024, and the optional pad string is statically-known ASCII in the current direct-runtime path; use explicit ASCII literals or the later compatibility path"
                ),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_string_at_call_with_literal_receiver(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "String.prototype.at is unavailable unless the receiver is a statically-known ASCII string literal and the optional index is a statically-known integer in the current direct-runtime path; use explicit ASCII literals or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_string_char_at_call_with_literal_receiver(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "String.prototype.charAt is unavailable unless the receiver is a statically-known ASCII string literal and the optional index is a statically-known integer in the current direct-runtime path; use explicit ASCII literals or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_string_char_code_at_call_with_literal_receiver(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "String.prototype.charCodeAt is unavailable unless the receiver is a statically-known ASCII string literal and the optional index is a statically-known integer in the current direct-runtime path; use explicit ASCII literals or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_string_code_point_at_call_with_literal_receiver(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "String.prototype.codePointAt is unavailable unless the receiver is a statically-known ASCII string literal and the optional index is a statically-known integer in the current direct-runtime path; use explicit ASCII literals or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(method) = self.string_case_call_method(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "String.prototype.{method} is unavailable unless the receiver is a statically-known ASCII string literal and no arguments are supplied in the current direct-runtime path; use explicit ASCII literals or the later compatibility path"
                ),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_string_normalize_call_with_literal_receiver(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "String.prototype.normalize is unavailable unless the receiver is a statically-known ASCII string literal and the optional normalization form is one of the statically-known strings NFC, NFD, NFKC, or NFKD in the current direct-runtime path; use explicit ASCII literals or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(method) = self.string_replace_call_method_with_literal_receiver(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "String.prototype.{method} is unavailable unless the receiver, search value, and replacement are statically-known ASCII string literals and the replacement contains no substitution markers in the current direct-runtime path; use explicit ASCII literals or the later compatibility path"
                ),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_string_split_call_with_literal_receiver(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "String.prototype.split is unavailable unless the receiver is a statically-known ASCII string literal, the optional separator is a statically-known ASCII string literal, and the optional limit is a statically-known integer from 0 through 1024 in the current direct-runtime path; use explicit ASCII literals or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(result) = self.resolve_static_array_at_call(node) {
            match result {
                StaticArrayAtResult::Value(value) => return self.emit_node(function, value, true),
                StaticArrayAtResult::OutOfRange => {
                    let undefined = self.alloc_scratch_node(
                        LirNodeKind::Literal,
                        Some("undefined".to_string()),
                        vec![],
                    );
                    return self.emit_node(function, undefined, true);
                }
            }
        }

        if let Some(result) = self.resolve_static_array_join_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if let Some(result) = self.resolve_static_array_to_string_call(node) {
            let literal = self.alloc_scratch_node(
                LirNodeKind::Literal,
                Some(quote_string_literal(&result)),
                vec![],
            );
            return self.emit_node(function, literal, true);
        }

        if self.is_array_at_call_with_literal_receiver(node) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "Array.prototype.at is unavailable unless the receiver is a statically-known array literal and the index is a statically-known in-range integer in the current phase; use explicit constants or the later compatibility path".to_string(),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(result) = self.resolve_static_array_reduce_call(node, "reduce") {
            function.instruction(&Instruction::I64Const(result));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(result) = self.resolve_static_array_reduce_call(node, "reduceRight") {
            function.instruction(&Instruction::I64Const(result));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if self.is_array_is_array_call(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(argument) = args.next() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Array.isArray requires at least one statically-known argument in the current phase; use an explicit literal or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            if let Some(result) = self.static_array_is_array_result(*argument) {
                for arg in args {
                    let produced = self.emit_node(function, *arg, true);
                    if produced.produced {
                        function.instruction(&Instruction::Drop);
                    }
                }
                function.instruction(&Instruction::I64Const(if result { 1 } else { 0 }));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                };
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "Array.isArray is unavailable unless the argument is a statically-known array, object, or primitive literal in the current phase; use explicit literals or the later compatibility path",
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_object_has_own_call(node, &callee_node) {
            let Some(object_id) = node.children.get(1).copied() else {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };
            let Some(key_id) = node.children.get(2).copied() else {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };
            let Some(key) = self.render_static_value(key_id) else {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            let Some(has_own) = self.static_object_has_own(object_id, &key) else {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            function.instruction(&Instruction::I64Const(if has_own { 1 } else { 0 }));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }

        if self.is_object_identity_object(&callee_node) && callee_node.text.as_deref() == Some("is")
        {
            let mut args = node.children.iter().skip(1);
            let Some(left) = args.next() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Object.is requires at least two statically-known primitive literal arguments or the same statically-known reference in the current phase; use explicit constants or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };
            let Some(right) = args.next() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Object.is requires at least two statically-known primitive literal arguments or the same statically-known reference in the current phase; use explicit constants or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            let left_value = self.resolve_static_object_identity_value(*left);
            let right_value = self.resolve_static_object_identity_value(*right);
            if let (Some(left_value), Some(right_value)) = (left_value, right_value) {
                let same_value = left_value.same_value(&right_value);

                for arg in args {
                    let produced = self.emit_node(function, *arg, true);
                    if produced.produced {
                        function.instruction(&Instruction::Drop);
                    }
                }

                function.instruction(&Instruction::I64Const(if same_value { 1 } else { 0 }));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                };
            }

            if let (Some(left_reference), Some(right_reference)) = (
                self.resolve_static_reference_root_name(*left),
                self.resolve_static_reference_root_name(*right),
            ) {
                if left_reference == right_reference {
                    for arg in args {
                        let produced = self.emit_node(function, *arg, true);
                        if produced.produced {
                            function.instruction(&Instruction::Drop);
                        }
                    }

                    function.instruction(&Instruction::I64Const(1));
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Boolean,
                    };
                }
            }

            if let (Some(left_ref), Some(right_ref)) = (
                self.resolve_literal_aggregate(*left),
                self.resolve_literal_aggregate(*right),
            ) {
                for arg in args {
                    let produced = self.emit_node(function, *arg, true);
                    if produced.produced {
                        function.instruction(&Instruction::Drop);
                    }
                }

                function.instruction(&Instruction::I64Const(if left_ref == right_ref {
                    1
                } else {
                    0
                }));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                };
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "Object.is is unavailable unless both arguments are statically-known primitive literals or the same statically-known reference in the current phase; use explicit constants or the later compatibility path".to_string(),
            ));
            function.instruction(&Instruction::Unreachable);
            return EmittedValue {
                produced: false,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_number_object(&callee_node)
            && matches!(
                callee_node.text.as_deref(),
                Some("isFinite") | Some("isNaN") | Some("isInteger") | Some("isSafeInteger")
            )
        {
            let method = callee_node.text.as_deref().unwrap_or("isFinite");
            let mut args = node.children.iter().skip(1);
            let Some(value) = args.next() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Number.{method} requires at least one statically-known primitive value in the current phase; use an explicit constant or the later compatibility path"
                    ),
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            let Some(value) = self.resolve_static_object_identity_value(*value) else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Number.{method} is unavailable unless the argument is a statically-known primitive value in the current phase; use an explicit constant or the later compatibility path"
                    ),
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            let supported = match value {
                StaticObjectIdentityValue::Number(number) => match method {
                    "isFinite" => number.is_finite(),
                    "isNaN" => number.is_nan(),
                    "isInteger" => number.is_finite() && number.fract() == 0.0,
                    "isSafeInteger" => {
                        number.is_finite()
                            && number.fract() == 0.0
                            && number.abs() <= 9007199254740991.0
                    }
                    _ => false,
                },
                _ => false,
            };

            for arg in args {
                let produced = self.emit_node(function, *arg, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
            }

            function.instruction(&Instruction::I64Const(if supported { 1 } else { 0 }));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }

        if let Some(import_index) = self.math_max_import_index(&callee_node) {
            let args: Vec<_> = node.children.iter().skip(1).copied().collect();
            let Some(first_arg) = args.first() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.max requires at least one argument in the current phase; use an explicit argument or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            if let Some(folded) = self.math_extrema_static_literal_value("max", &args) {
                function.instruction(&Instruction::I64Const(folded));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if !self.emit_integer_math_arg(function, *first_arg, "max") {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
            for arg in args.iter().skip(1) {
                if !self.emit_integer_math_arg(function, *arg, "max") {
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                }
                function.instruction(&Instruction::Call(import_index));
            }
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(import_index) = self.math_min_import_index(&callee_node) {
            let args: Vec<_> = node.children.iter().skip(1).copied().collect();
            let Some(first_arg) = args.first() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.min requires at least one argument in the current phase; use an explicit argument or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            if let Some(folded) = self.math_extrema_static_literal_value("min", &args) {
                function.instruction(&Instruction::I64Const(folded));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if !self.emit_integer_math_arg(function, *first_arg, "min") {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
            for arg in args.iter().skip(1) {
                if !self.emit_integer_math_arg(function, *arg, "min") {
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                }
                function.instruction(&Instruction::Call(import_index));
            }
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(import_index) = self.math_abs_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(first_arg) = args.next() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.abs requires at least one argument in the current phase; use an explicit argument or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            if let Some(folded) = self.math_abs_static_literal_value(*first_arg) {
                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if !self.emit_integer_math_arg(function, *first_arg, "abs") {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
            function.instruction(&Instruction::Call(import_index));
            for arg in args {
                if !self.emit_integer_math_arg(function, *arg, "abs") {
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

        if let Some(import_index) = self.math_sign_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(first_arg) = args.next() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.sign requires at least one argument in the current phase; use an explicit argument or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            if let Some(folded) = self.math_sign_static_literal_value(*first_arg) {
                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if !self.emit_integer_math_arg(function, *first_arg, "sign") {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
            function.instruction(&Instruction::Call(import_index));
            for arg in args {
                if !self.emit_integer_math_arg(function, *arg, "sign") {
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

        if let Some(import_index) = self.math_imul_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(left) = args.next() else {
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            };
            let Some(right) = args.next() else {
                let produced = self.emit_node(function, *left, true);
                if produced.produced {
                    function.instruction(&Instruction::Drop);
                }
                for arg in args {
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
            };

            if let Some(folded) = self.math_imul_static_literal_value(*left, *right) {
                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if !self.emit_integer_math_arg(function, *left, "imul") {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
            if !self.emit_integer_math_arg(function, *right, "imul") {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
            function.instruction(&Instruction::Call(import_index));
            for arg in args {
                if !self.emit_integer_math_arg(function, *arg, "imul") {
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

        if let Some(import_index) = self.math_round_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(value) = args.next() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.round requires at least one argument in the current phase; use an explicit argument or the later compatibility path",
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            };

            if let Some(folded) = self.math_round_like_static_literal_value("round", *value) {
                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if !self.emit_integer_math_arg(function, *value, "round") {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
            function.instruction(&Instruction::Call(import_index));
            for arg in args {
                if !self.emit_integer_math_arg(function, *arg, "round") {
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

        if let Some(import_index) = self.math_clz32_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(value) = args.next() else {
                function.instruction(&Instruction::I64Const(32));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            };

            if let Some(folded) = self.math_clz32_static_literal_value(*value) {
                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if !self.emit_integer_math_arg(function, *value, "clz32") {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
            function.instruction(&Instruction::Call(import_index));
            for arg in args {
                if !self.emit_integer_math_arg(function, *arg, "clz32") {
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

        if self.math_pow_import_index(&callee_node).is_some() {
            let operands: Vec<_> = node.children.iter().skip(1).copied().collect();
            return self.emit_exponentiation_expression(function, &operands, "Math.pow");
        }

        if matches!(
            callee_node.text.as_deref(),
            Some("floor") | Some("trunc") | Some("ceil")
        ) && self.is_math_object(&callee_node)
        {
            let method = callee_node.text.as_deref().unwrap_or("floor").to_string();
            let mut args = node.children.iter().skip(1);
            let Some(value) = args.next() else {
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            };

            if let Some(folded) = self.math_round_like_static_literal_value(method.as_str(), *value)
            {
                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if !self.emit_integer_math_arg(function, *value, method.as_str()) {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
            for arg in args {
                if !self.emit_integer_math_arg(function, *arg, method.as_str()) {
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

        if let Some(method) = self.math_member_method(&callee_node) {
            if method == "hypot" {
                let args: Vec<_> = node.children.iter().skip(1).copied().collect();
                let Some(root) = self.math_hypot_constant_root(&args) else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        "Math.hypot is unavailable unless every argument is a statically-known integer literal whose squared sum is a perfect-square integer literal in the current phase; use explicit constants or the later compatibility path",
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                function.instruction(&Instruction::I64Const(root));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if method == "atan2" {
                let args: Vec<_> = node.children.iter().skip(1).copied().collect();
                let Some(left) = args.first().copied() else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        "Math.atan2 is unavailable unless the first argument is a statically-known zero numeric literal and the second argument is a statically-known non-negative numeric literal in the current phase; use explicit constants or the later compatibility path".to_string(),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };
                let Some(right) = args.get(1).copied() else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        "Math.atan2 is unavailable unless the first argument is a statically-known zero numeric literal and the second argument is a statically-known non-negative numeric literal in the current phase; use explicit constants or the later compatibility path".to_string(),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                let Some(folded) = self.math_atan2_zero_slice_value(left, right) else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        "Math.atan2 is unavailable unless the first argument is a statically-known zero numeric literal and the second argument is a statically-known non-negative numeric literal in the current phase; use explicit constants or the later compatibility path".to_string(),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                function.instruction(&Instruction::I64Const(folded));
                for arg in args.iter().skip(2).copied() {
                    let _ = self.emit_node(function, arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if method == "exp" || method == "log" || method == "exp2" {
                let mut args = node.children.iter().skip(1);
                let Some(value) = args.next() else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                let folded = match method {
                    "exp" => self.math_exp_constant_value(*value),
                    "log" => self.math_log_constant_value(*value),
                    "exp2" => self.math_exp2_constant_value(*value),
                    _ => unreachable!(),
                };
                let Some(folded) = folded else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                            if method == "log" {
                                "one"
                            } else if method == "exp2" {
                                "non-negative integer literal within the current integer-fold range"
                            } else {
                                "zero"
                            }
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if method == "expm1" || method == "log1p" || method == "fround" {
                let mut args = node.children.iter().skip(1);
                let Some(value) = args.next() else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                let folded = match method {
                    "expm1" => self.math_expm1_constant_value(*value),
                    "log1p" => self.math_log1p_constant_value(*value),
                    "fround" => self.math_fround_zero_constant_value(*value),
                    _ => unreachable!(),
                };
                let Some(folded) = folded else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if method == "asinh" || method == "acosh" || method == "atanh" {
                let mut args = node.children.iter().skip(1);
                let Some(value) = args.next() else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                let folded = self.math_inverse_hyperbolic_constant_value(method, *value);
                let Some(folded) = folded else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                            if method == "acosh" { "one" } else { "zero" }
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if method == "sinh" || method == "cosh" || method == "tanh" {
                let mut args = node.children.iter().skip(1);
                let Some(value) = args.next() else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                let folded = self.math_hyperbolic_zero_constant_value(method, *value);
                let Some(folded) = folded else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if method == "sin" || method == "cos" || method == "tan" {
                let mut args = node.children.iter().skip(1);
                let Some(value) = args.next() else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                let folded = self.math_sin_cos_zero_constant_value(method, *value);
                let Some(folded) = folded else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if method == "asin" || method == "acos" || method == "atan" {
                let mut args = node.children.iter().skip(1);
                let Some(value) = args.next() else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                let folded = self.math_inverse_trig_constant_value(method, *value);
                let Some(folded) = folded else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                            if method == "acos" { "one" } else { "zero" }
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if method == "sin" || method == "cos" || method == "tan" {
                let mut args = node.children.iter().skip(1);
                let Some(value) = args.next() else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                let folded = self.math_sin_cos_zero_constant_value(method, *value);
                let Some(folded) = folded else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                function.instruction(&Instruction::I64Const(folded));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if method == "log2" || method == "log10" {
                let mut args = node.children.iter().skip(1);
                let Some(value) = args.next() else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} is unavailable unless the argument is a statically-known {} integer literal in the current phase; use an explicit constant or the later compatibility path",
                            if method == "log2" { "positive power-of-two" } else { "positive power-of-ten" }
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                let literal_root = if method == "log2" {
                    self.math_log2_constant_exponent(*value)
                } else {
                    self.math_log10_constant_exponent(*value)
                };
                let Some(exponent) = literal_root else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} is unavailable unless the argument is a statically-known {} integer literal in the current phase; use an explicit constant or the later compatibility path",
                            if method == "log2" { "positive power-of-two" } else { "positive power-of-ten" }
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                function.instruction(&Instruction::I64Const(exponent));
                for arg in args {
                    let _ = self.emit_node(function, *arg, true);
                    function.instruction(&Instruction::Drop);
                }
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }

            if method == "sqrt" || method == "cbrt" {
                let mut args = node.children.iter().skip(1);
                let Some(value) = args.next() else {
                    self.diagnostics.push(Diagnostic::error(
                        e5::FEATURE_UNAVAILABLE as u32,
                        format!(
                            "Math.{method} is unavailable unless the argument is a statically-known {} integer literal in the current phase; use an explicit constant or the later compatibility path",
                            if method == "sqrt" { "perfect-square" } else { "perfect-cube" }
                        ),
                    ));
                    function.instruction(&Instruction::Unreachable);
                    return EmittedValue {
                        produced: false,
                        shape: ValueShape::Unknown,
                    };
                };

                let root = if method == "sqrt" {
                    self.math_sqrt_constant_root(*value)
                } else {
                    self.math_cbrt_constant_root(*value)
                };
                if let Some(root) = root {
                    function.instruction(&Instruction::I64Const(root));
                    for arg in args {
                        let _ = self.emit_node(function, *arg, true);
                        function.instruction(&Instruction::Drop);
                    }
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Scalar,
                    };
                }

                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable unless the argument is a statically-known {} integer literal in the current phase; use an explicit constant or the later compatibility path",
                        if method == "sqrt" { "perfect-square" } else { "perfect-cube" }
                    ),
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }

            if !matches!(
                method,
                "max"
                    | "min"
                    | "abs"
                    | "sign"
                    | "imul"
                    | "round"
                    | "clz32"
                    | "pow"
                    | "trunc"
                    | "floor"
                    | "tan"
                    | "asin"
                    | "acos"
                    | "atan"
                    | "asinh"
                    | "acosh"
                    | "atanh"
            ) {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable in the current phase; use a supported Math builtin or the later compatibility path"
                    ),
                ));
                function.instruction(&Instruction::Unreachable);
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
        }

        if let Some(import_index) = self.cwd_import_index(&callee_node) {
            let env_buffer_offset = 0i32;
            let env_buffer_capacity = ENV_GET_BUFFER_RESERVED as i32;
            function.instruction(&Instruction::I32Const(0));
            function.instruction(&Instruction::I32Const(0));
            function.instruction(&Instruction::I32Const(env_buffer_offset));
            function.instruction(&Instruction::I32Const(env_buffer_capacity));
            function.instruction(&Instruction::Call(import_index));
            function.instruction(&Instruction::I64ExtendI32U);
            let temp_local = self.locals.len() as u32;
            function.instruction(&Instruction::LocalTee(temp_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(temp_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::I64Const(STRING_HANDLE_TAG as i64));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::End);
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(import_index) = self.env_set_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(key_expr) = args.next() else {
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };
            let Some(value_expr) = args.next() else {
                self.push_placeholder_fallback_diagnostic("call target", "Deno.env.set");
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };

            let Some(key_text) = self.render_static_value(*key_expr) else {
                self.push_placeholder_fallback_diagnostic("call target", "Deno.env.set");
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };
            let Some(value_text) = self.render_static_value(*value_expr) else {
                self.push_placeholder_fallback_diagnostic("call target", "Deno.env.set");
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };

            let (key_offset, key_len) = self.strings.intern(&key_text);
            let (value_offset, value_len) = self.strings.intern(&value_text);
            function.instruction(&Instruction::I32Const(key_offset as i32));
            function.instruction(&Instruction::I32Const(key_len as i32));
            function.instruction(&Instruction::I32Const(value_offset as i32));
            function.instruction(&Instruction::I32Const(value_len as i32));
            function.instruction(&Instruction::Call(import_index));
            function.instruction(&Instruction::Drop);
            for arg in args {
                let _ = self.emit_node(function, *arg, true);
                function.instruction(&Instruction::Drop);
            }
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(import_index) = self.env_delete_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(key_expr) = args.next() else {
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };

            let Some(key_text) = self.render_static_value(*key_expr) else {
                self.push_placeholder_fallback_diagnostic("call target", "Deno.env.delete");
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
            for arg in args {
                let _ = self.emit_node(function, *arg, true);
                function.instruction(&Instruction::Drop);
            }
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(import_index) = self.env_get_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(key_expr) = args.next() else {
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };

            let Some(key_text) = self.render_static_value(*key_expr) else {
                self.push_placeholder_fallback_diagnostic("call target", "Deno.env.get");
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };

            let (key_offset, key_len) = self.strings.intern(&key_text);
            let env_buffer_offset = 0i32;
            let env_buffer_capacity = ENV_GET_BUFFER_RESERVED as i32;
            function.instruction(&Instruction::I32Const(key_offset as i32));
            function.instruction(&Instruction::I32Const(key_len as i32));
            function.instruction(&Instruction::I32Const(env_buffer_offset));
            function.instruction(&Instruction::I32Const(env_buffer_capacity));
            function.instruction(&Instruction::Call(import_index));
            function.instruction(&Instruction::I64ExtendI32U);
            let temp_local = self.locals.len() as u32;
            function.instruction(&Instruction::LocalTee(temp_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::LocalGet(temp_local));
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::I64Sub);
            function.instruction(&Instruction::I64Const(STRING_HANDLE_TAG as i64));
            function.instruction(&Instruction::I64Or);
            function.instruction(&Instruction::End);
            for arg in args {
                let _ = self.emit_node(function, *arg, true);
                function.instruction(&Instruction::Drop);
            }
            return EmittedValue {
                produced: true,
                shape: ValueShape::Scalar,
            };
        }

        if let Some(import_index) = self.env_has_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(key_expr) = args.next() else {
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };

            let Some(key_text) = self.render_static_value(*key_expr) else {
                self.push_placeholder_fallback_diagnostic("call target", "Deno.env.has");
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };

            let (key_offset, key_len) = self.strings.intern(&key_text);
            function.instruction(&Instruction::I32Const(key_offset as i32));
            function.instruction(&Instruction::I32Const(key_len as i32));
            function.instruction(&Instruction::Call(import_index));
            function.instruction(&Instruction::I64ExtendI32U);
            for arg in args {
                let _ = self.emit_node(function, *arg, true);
                function.instruction(&Instruction::Drop);
            }
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }

        if let Some(import_index) = self.cwd_set_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(path_expr) = args.next() else {
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };

            let _ = self.emit_node(function, *path_expr, true);
            function.instruction(&Instruction::Call(import_index));
            function.instruction(&Instruction::Drop);
            for arg in args {
                let _ = self.emit_node(function, *arg, true);
                function.instruction(&Instruction::Drop);
            }
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        }

        if let Some(import_index) = self.process_exit_import_index(&callee_node) {
            let mut args = node.children.iter().skip(1);
            if let Some(code_expr) = args.next() {
                let _ = self.emit_node(function, *code_expr, true);
            } else {
                function.instruction(&Instruction::I64Const(0));
            }
            for arg in args {
                let _ = self.emit_node(function, *arg, true);
                function.instruction(&Instruction::Drop);
            }
            function.instruction(&Instruction::Call(import_index));
            function.instruction(&Instruction::I64Const(0));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        }

        if self.is_process_kill(&callee_node) {
            let mut args = node.children.iter().skip(1);
            let Some(pid_expr) = args.next() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    kali_common::process_kill_zero_probe_unavailable_message(),
                ));
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };
            let Some(pid_text) = self.render_static_value(*pid_expr) else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    kali_common::process_kill_zero_probe_unavailable_message(),
                ));
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            };
            if parse_number_literal(&pid_text) != Some(0) || args.next().is_some() {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    kali_common::process_kill_zero_probe_unavailable_message(),
                ));
                function.instruction(&Instruction::I64Const(0));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Unknown,
                };
            }
            function.instruction(&Instruction::I64Const(1));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Boolean,
            };
        }

        for arg in node.children.iter().skip(1) {
            let _ = self.emit_node(function, *arg, true);
        }

        if let Some(index) = resolved {
            function.instruction(&Instruction::Call(index));
            return EmittedValue {
                produced: true,
                shape: ValueShape::Unknown,
            };
        }

        if self.has_semver_import() {
            if let Some(rendered) = self.render_semver_intrinsic(callee_name, node) {
                for _ in node.children.iter().skip(1) {
                    function.instruction(&Instruction::Drop);
                }
                if rendered == "0" || rendered == "1" {
                    let value = rendered.parse::<i64>().unwrap_or(0);
                    function.instruction(&Instruction::I64Const(value));
                    return EmittedValue {
                        produced: true,
                        shape: ValueShape::Boolean,
                    };
                }
                let (offset, len) = self.strings.intern(&rendered);
                function.instruction(&Instruction::I64Const(encode_string_handle(offset, len)));
                return EmittedValue {
                    produced: true,
                    shape: ValueShape::Scalar,
                };
            }
        }

        self.push_placeholder_fallback_diagnostic("call target", callee_name);
        for _ in node.children.iter().skip(1) {
            function.instruction(&Instruction::Drop);
        }
        function.instruction(&Instruction::I64Const(0));
        EmittedValue {
            produced: true,
            shape: ValueShape::Unknown,
        }
    }

    pub(crate) fn resolve_static_index_member(
        &self,
        node: &LirNode,
    ) -> Option<StaticIndexMemberResult> {
        if node.kind != LirNodeKind::Value || node.children.len() != 1 {
            return None;
        }

        let index = self.static_member_index(node.text.as_deref()?)?;
        let source = node.children[0];

        if let Some(result) = self.resolve_static_array_concat_element(source, index) {
            return Some(result);
        }

        if let Some(parts) = self.resolve_static_string_split_parts_from_id(source) {
            return Some(
                parts
                    .get(index)
                    .cloned()
                    .map(StaticIndexMemberResult::String)
                    .unwrap_or(StaticIndexMemberResult::Undefined),
            );
        }

        let source = self.resolve_literal_aggregate(source)?;
        let source_node = self.node(source);
        if self.is_array_literal(source_node) {
            return Some(
                source_node
                    .children
                    .get(index)
                    .copied()
                    .map(StaticIndexMemberResult::Node)
                    .unwrap_or(StaticIndexMemberResult::Undefined),
            );
        }

        None
    }

    pub(crate) fn static_member_index(&self, text: &str) -> Option<usize> {
        let index = parse_number_literal(text)?;
        (index >= 0 && text.chars().all(|ch| ch.is_ascii_digit())).then_some(index as usize)
    }

    pub(crate) fn resolve_static_reference_root_name(&self, id: LirNodeId) -> Option<String> {
        let id = self.resolve_bound_node(id);
        let id = self.unwrap_transparent_value_node(id);
        let node = self.node(id);

        if self.is_object_freeze_call(node) {
            return node
                .children
                .get(1)
                .copied()
                .and_then(|child| self.resolve_static_reference_root_name(child));
        }

        if node.kind == LirNodeKind::Value && node.children.len() == 2 {
            match node.text.as_deref() {
                Some("??") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    if left.is_nullish() {
                        return self.resolve_static_reference_root_name(node.children[1]);
                    }
                    return self.resolve_static_reference_root_name(node.children[0]);
                }
                Some("&&") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    match left.truthiness() {
                        Some(true) => {
                            return self.resolve_static_reference_root_name(node.children[1]);
                        }
                        Some(false) => {
                            return self.resolve_static_reference_root_name(node.children[0]);
                        }
                        None => {
                            let left_root =
                                self.resolve_static_reference_root_name(node.children[0]);
                            let right_root =
                                self.resolve_static_reference_root_name(node.children[1]);
                            if left_root.is_some() && left_root == right_root {
                                return left_root;
                            }
                        }
                    }
                }
                Some("||") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    match left.truthiness() {
                        Some(true) => {
                            return self.resolve_static_reference_root_name(node.children[0]);
                        }
                        Some(false) => {
                            return self.resolve_static_reference_root_name(node.children[1]);
                        }
                        None => {
                            let left_root =
                                self.resolve_static_reference_root_name(node.children[0]);
                            let right_root =
                                self.resolve_static_reference_root_name(node.children[1]);
                            if left_root.is_some() && left_root == right_root {
                                return left_root;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        match node.kind {
            LirNodeKind::Value if node.children.is_empty() => {
                let text = node.text.as_deref()?;
                if let Some(bound) = self.bindings.get(text).copied() {
                    return self.resolve_static_reference_root_name(bound);
                }
                Some(text.to_string())
            }
            LirNodeKind::Value if node.children.len() == 1 => {
                let object = self.resolve_static_reference_root_name(node.children[0])?;
                let property = node.text.as_deref()?;
                Some(format!("{object}.{property}"))
            }
            _ => None,
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

    pub(crate) fn unwrap_transparent_value_node(&self, mut id: LirNodeId) -> LirNodeId {
        loop {
            let node = self.node(id);
            if node.kind == LirNodeKind::Value
                && node.children.len() == 1
                && node.text.as_deref().is_none_or(|text| text.is_empty())
            {
                id = node.children[0];
                continue;
            }

            return id;
        }
    }

    pub(crate) fn is_supported_callable_reference(&self, node: &LirNode) -> bool {
        if node.kind != LirNodeKind::Value || node.children.len() != 1 {
            return false;
        }

        match node.text.as_deref() {
            Some("is") => self.is_object_identity_object(node),
            Some("isFinite") | Some("isNaN") | Some("isInteger") | Some("isSafeInteger") => {
                self.is_number_object(node)
            }
            _ => false,
        }
    }

    pub(crate) fn resolve_bound_node(&self, mut id: LirNodeId) -> LirNodeId {
        let mut seen = HashSet::new();

        loop {
            if !seen.insert(id) {
                return id;
            }

            let node = self.node(id);
            if node.kind == LirNodeKind::Value && node.children.is_empty() {
                if let Some(text) = node.text.as_deref() {
                    if let Some(bound) = self.bindings.get(text).copied() {
                        id = bound;
                        continue;
                    }
                }
            }

            return id;
        }
    }

    pub(crate) fn resolve_bound_member_callable_node(&self, id: LirNodeId) -> Option<LirNodeId> {
        let bound = self.resolve_bound_node(id);
        let bound = self.unwrap_transparent_value_node(bound);
        let node = self.node(bound);
        if node.kind == LirNodeKind::Value && node.children.len() == 2 {
            match node.text.as_deref() {
                Some("??") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    if left.is_nullish() {
                        return self.resolve_bound_member_callable_node(node.children[1]);
                    }
                    return self.resolve_bound_member_callable_node(node.children[0]);
                }
                Some("&&") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    match left.truthiness() {
                        Some(true) => {
                            return self.resolve_bound_member_callable_node(node.children[1])
                        }
                        Some(false) => {
                            return self.resolve_bound_member_callable_node(node.children[0])
                        }
                        None => {
                            let consequent =
                                self.resolve_bound_member_callable_node(node.children[0]);
                            let alternate =
                                self.resolve_bound_member_callable_node(node.children[1]);
                            if consequent.is_some() && consequent == alternate {
                                return consequent;
                            }
                        }
                    }
                }
                Some("||") => {
                    let left = self.resolve_static_object_identity_value(node.children[0])?;
                    match left.truthiness() {
                        Some(true) => {
                            return self.resolve_bound_member_callable_node(node.children[0])
                        }
                        Some(false) => {
                            return self.resolve_bound_member_callable_node(node.children[1])
                        }
                        None => {
                            let consequent =
                                self.resolve_bound_member_callable_node(node.children[0]);
                            let alternate =
                                self.resolve_bound_member_callable_node(node.children[1]);
                            if consequent.is_some() && consequent == alternate {
                                return consequent;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        if node.kind == LirNodeKind::Value && node.children.len() == 3 {
            let consequent = self.resolve_bound_member_callable_node(node.children[1])?;
            let alternate = self.resolve_bound_member_callable_node(node.children[2])?;
            if self.node(consequent).text.as_deref() == self.node(alternate).text.as_deref() {
                return Some(consequent);
            }
        }
        if node.text.is_some() && !node.children.is_empty() {
            Some(bound)
        } else if self.is_object_freeze_call(node) {
            node.children
                .get(1)
                .copied()
                .and_then(|child| self.resolve_bound_member_callable_node(child))
        } else if node.kind == LirNodeKind::Value && node.children.len() == 1 {
            self.resolve_bound_member_callable_node(node.children[0])
        } else {
            None
        }
    }

    pub(crate) fn resolve_transparent_callable_node(&self, id: LirNodeId) -> Option<LirNodeId> {
        let mut id = self.resolve_bound_node(id);
        let mut seen = HashSet::new();

        loop {
            if !seen.insert(id.0) {
                return None;
            }

            let node = self.node(id);
            if node.kind == LirNodeKind::Value
                && node.children.len() == 1
                && node.text.as_deref().is_none_or(|text| text.is_empty())
            {
                id = node.children[0];
                continue;
            }

            if self.is_object_freeze_call(node) {
                id = node.children.get(1).copied()?;
                continue;
            }

            if node.kind == LirNodeKind::Value && node.children.len() == 2 {
                match node.text.as_deref() {
                    Some("??") => {
                        let left = self.resolve_static_object_identity_value(node.children[0])?;
                        if left.is_nullish() {
                            id = node.children[1];
                            continue;
                        }

                        id = node.children[0];
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
                                let consequent =
                                    self.resolve_transparent_callable_node(node.children[0]);
                                let alternate =
                                    self.resolve_transparent_callable_node(node.children[1]);
                                if consequent.is_some() && consequent == alternate {
                                    return consequent;
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
                                let consequent =
                                    self.resolve_transparent_callable_node(node.children[0]);
                                let alternate =
                                    self.resolve_transparent_callable_node(node.children[1]);
                                if consequent.is_some() && consequent == alternate {
                                    return consequent;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            if node.kind == LirNodeKind::Value && node.children.len() == 3 {
                let consequent = self.resolve_transparent_callable_node(node.children[1])?;
                let alternate = self.resolve_transparent_callable_node(node.children[2])?;
                if self.node(consequent).text.as_deref() == self.node(alternate).text.as_deref() {
                    return Some(consequent);
                }
            }

            if node.text.is_some() {
                return Some(id);
            }

            return None;
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
            ValueShape::Scalar | ValueShape::Unknown => {
                function.instruction(&Instruction::I64Eqz);
                function.instruction(&Instruction::I32Eqz);
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
