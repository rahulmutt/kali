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
                            return true;
                        }
                    }
                }
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
}

#[cfg(test)]
#[path = "literal_tests.rs"]
mod literal_tests;
