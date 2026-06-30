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
            "<" => {
                function.instruction(&Instruction::I64LtS);
                function.instruction(&Instruction::I64ExtendI32U);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                }
            }
            "<=" => {
                function.instruction(&Instruction::I64LeS);
                function.instruction(&Instruction::I64ExtendI32U);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                }
            }
            ">" => {
                function.instruction(&Instruction::I64GtS);
                function.instruction(&Instruction::I64ExtendI32U);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Boolean,
                }
            }
            ">=" => {
                function.instruction(&Instruction::I64GeS);
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
