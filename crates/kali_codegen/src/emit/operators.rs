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

    /// Unwraps transparent single-child `Value` wrappers (no operator text) so a
    /// string-classification query inspects the underlying literal/expression.
    fn unwrap_transparent(&self, mut id: LirNodeId) -> LirNodeId {
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
    /// template) literal, or a `+` expression whose either operand is string-valued.
    pub(crate) fn is_string_valued(&self, id: LirNodeId) -> bool {
        let id = self.unwrap_transparent(id);
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
            _ => false,
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
        let node = self.node(id);
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
                self.node(callee)
                    .text
                    .as_deref()
                    .is_some_and(|name| self.repr_table.return_repr(name) == kali_common::Repr::F64)
            }
            LirNodeKind::Value => match node.children.len() {
                0 => node.text.as_deref().is_some_and(|text| {
                    Self::is_float_literal_text(text)
                        || self.scalar_repr(text) == kali_common::Repr::F64
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
                            "/" => true,
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
                _ => false,
            },
            _ => false,
        }
    }

    /// Emits operand `id`, inserting an `f64.convert_i64_s` promotion when the
    /// surrounding operation is float-typed (`float_op`) but this operand is itself
    /// integer-valued. Per-side so mixed `int <op> float` operands both land as
    /// `f64` on the stack before the float instruction.
    fn emit_float_operand(&mut self, function: &mut Function, id: LirNodeId, float_op: bool) {
        let operand_is_float = self.is_float_valued(id);
        let _ = self.emit_node(function, id, true);
        if float_op && !operand_is_float {
            function.instruction(&Instruction::F64ConvertI64S);
        }
    }

    /// Emits `id` as a string handle: if it is already string-valued the emitted
    /// value is a handle; otherwise the produced i64 is coerced to a decimal-string
    /// handle via `int_to_string`.
    pub(crate) fn emit_as_string(&mut self, function: &mut Function, id: LirNodeId) {
        let is_string = self.is_string_valued(id);
        let produced = self.emit_node(function, id, true);
        if !produced.produced {
            function.instruction(&Instruction::I64Const(0));
        }
        if !is_string {
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

        // Repr-directed float selection. Arithmetic `+ - *` is float when either
        // operand is float; `/` is always float (JS division yields a double in
        // this model); relational ops compare as doubles when either operand is
        // float. `%`, logical, `??`, and `**` stay on the integer path. For an
        // all-integer program every operand is integer-valued and `/` never
        // reaches here (no float seeds), so `float_op` is always false and the
        // emitted code is byte-identical to the pre-repr path.
        let operand_float = self.is_float_valued(left) || self.is_float_valued(right);
        let float_op = match op {
            "/" => true,
            "+" | "-" | "*" => operand_float,
            "<" | "<=" | ">" | ">=" | "==" | "===" | "!=" | "!==" => operand_float,
            _ => false,
        };

        if op != "??" && op != "**" {
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
                // `float_op` is always true here.
                function.instruction(&Instruction::F64Div);
                EmittedValue {
                    produced: true,
                    shape: ValueShape::Float,
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
