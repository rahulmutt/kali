//! Static recognition and constant-folding of Math intrinsic call shapes.
use crate::*;

impl<'a> FunctionEmitter<'a> {
    pub(crate) fn math_max_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "max" {
            Some(MATH_MAX_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_min_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "min" {
            Some(MATH_MIN_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_abs_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "abs" {
            Some(MATH_ABS_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_sign_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "sign" {
            Some(MATH_SIGN_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_imul_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "imul" {
            Some(MATH_IMUL_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_round_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "round" {
            Some(MATH_ROUND_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_clz32_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "clz32" {
            Some(MATH_CLZ32_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_pow_import_index(&self, callee_node: &LirNode) -> Option<u32> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) && method == "pow" {
            Some(MATH_POW_IMPORT_INDEX)
        } else {
            None
        }
    }

    pub(crate) fn math_member_method<'b>(&self, callee_node: &'b LirNode) -> Option<&'b str> {
        let method = callee_node.text.as_deref()?;
        if self.is_math_object(callee_node) {
            Some(method)
        } else {
            None
        }
    }

    pub(crate) fn math_exp_constant_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        if value == 0 {
            Some(1)
        } else {
            None
        }
    }

    pub(crate) fn math_log_constant_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        if value == 1 {
            Some(0)
        } else {
            None
        }
    }

    pub(crate) fn math_exp2_constant_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        if !(0..=62).contains(&value) {
            return None;
        }

        Some(1_i64 << (value as u32))
    }

    pub(crate) fn math_expm1_constant_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        if value == 0 {
            Some(0)
        } else {
            None
        }
    }

    pub(crate) fn math_log1p_constant_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        if value == 0 {
            Some(0)
        } else {
            None
        }
    }

    pub(crate) fn math_fround_zero_constant_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_numeric_literal_value(&rendered)?;
        if value == 0.0 {
            Some(0)
        } else {
            None
        }
    }

    pub(crate) fn math_sin_cos_zero_constant_value(
        &self,
        method: &str,
        arg: LirNodeId,
    ) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_numeric_literal_value(&rendered)?;
        if value != 0.0 {
            return None;
        }

        Some(if method == "cos" { 1 } else { 0 })
    }

    pub(crate) fn math_hyperbolic_zero_constant_value(
        &self,
        method: &str,
        arg: LirNodeId,
    ) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_numeric_literal_value(&rendered)?;
        if value != 0.0 {
            return None;
        }

        Some(if method == "cosh" { 1 } else { 0 })
    }

    pub(crate) fn math_inverse_trig_constant_value(
        &self,
        method: &str,
        arg: LirNodeId,
    ) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_numeric_literal_value(&rendered)?;

        match method {
            "asin" | "atan" if value == 0.0 => Some(0),
            "acos" if value == 1.0 => Some(0),
            _ => None,
        }
    }

    pub(crate) fn math_atan2_zero_slice_value(&self, y: LirNodeId, x: LirNodeId) -> Option<i64> {
        let y = self.render_static_value(y)?;
        let x = self.render_static_value(x)?;
        let y = parse_numeric_literal_value(&y)?;
        let x = parse_numeric_literal_value(&x)?;
        if y == 0.0 && x.is_finite() && x >= 0.0 {
            Some(0)
        } else {
            None
        }
    }

    pub(crate) fn math_inverse_hyperbolic_constant_value(
        &self,
        method: &str,
        arg: LirNodeId,
    ) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_numeric_literal_value(&rendered)?;

        match method {
            "acosh" if value == 1.0 => Some(0),
            "asinh" | "atanh" if value == 0.0 => Some(0),
            _ => None,
        }
    }

    pub(crate) fn math_sqrt_constant_root(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        if value < 0 {
            return None;
        }

        let root = (value as f64).sqrt() as i64;
        if root.checked_mul(root) == Some(value) {
            Some(root)
        } else {
            None
        }
    }

    pub(crate) fn math_cbrt_constant_root(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        let root = (value as f64).cbrt().round() as i64;
        if i128::from(root).pow(3) == i128::from(value) {
            Some(root)
        } else {
            None
        }
    }

    pub(crate) fn math_log2_constant_exponent(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_number_literal(&rendered)?;
        if value <= 0 {
            return None;
        }

        let value = value as u64;
        if value.is_power_of_two() {
            Some(i64::from(value.trailing_zeros()))
        } else {
            None
        }
    }

    pub(crate) fn math_log10_constant_exponent(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let mut value = parse_number_literal(&rendered)?;
        if value <= 0 {
            return None;
        }

        let mut exponent = 0;
        while value % 10 == 0 {
            value /= 10;
            exponent += 1;
        }

        if value == 1 {
            Some(exponent)
        } else {
            None
        }
    }

    pub(crate) fn math_hypot_constant_root(&self, args: &[LirNodeId]) -> Option<i64> {
        if args.is_empty() {
            return Some(0);
        }

        let mut sum = 0_i128;
        for arg in args {
            let rendered = self.render_static_value(*arg)?;
            let value = parse_numeric_literal_value(&rendered)?;
            if !value.is_finite()
                || value.fract() != 0.0
                || value < i64::MIN as f64
                || value > i64::MAX as f64
            {
                return None;
            }

            let value = value as i128;
            sum = sum.checked_add(value.checked_mul(value)?)?;
        }

        self.perfect_square_root_i128(sum)
    }

    pub(crate) fn math_round_like_static_literal_value(
        &self,
        method: &str,
        arg: LirNodeId,
    ) -> Option<i64> {
        // Residual round 1: a module `const` of a literal read from a
        // function folds too (inference treats it as a compile-time number,
        // `is_static_numeric`; it used to reach the integer lane as an f64).
        let value = match self.render_static_value(arg) {
            Some(rendered) => parse_numeric_literal_value(&rendered)?,
            None => self.static_numeric_chain(arg)?,
        };
        let folded = match method {
            "round" => {
                if value.fract() == 0.0 {
                    value
                } else {
                    (value + 0.5).floor()
                }
            }
            "trunc" => value.trunc(),
            "ceil" => value.ceil(),
            "floor" => value.floor(),
            _ => return None,
        };

        if !folded.is_finite() || folded < i64::MIN as f64 || folded > i64::MAX as f64 {
            return None;
        }

        Some(folded as i64)
    }

    pub(crate) fn math_extrema_static_literal_value(
        &self,
        method: &str,
        args: &[LirNodeId],
    ) -> Option<i64> {
        let mut values = args.iter().map(|arg| {
            let rendered = self.render_static_value(*arg)?;
            let value = parse_numeric_literal_value(&rendered)?;
            if !value.is_finite()
                || value.fract() != 0.0
                || value < i64::MIN as f64
                || value > i64::MAX as f64
            {
                return None;
            }
            Some(value as i64)
        });

        let mut folded = values.next().flatten()?;
        for value in values {
            let value = value?;
            folded = if method == "max" {
                folded.max(value)
            } else {
                folded.min(value)
            };
        }

        Some(folded)
    }

    /// Residual rounds 1-2 (spec A-41): the value of a compile-time number
    /// — a numeric literal, a unary sign, `Object.freeze(…)` or a transparent
    /// wrapper of one, or an identifier bound to one through a chain of local
    /// `const` fold aliases (`self.bindings`) and, from a function, module
    /// `const`s (`module_const_inits`, whose own identifiers resolve at
    /// module scope only). Cycle-safe by depth. The codegen half of
    /// `repr_infer::is_static_numeric`, which follows the same chains over
    /// the AST, so a rounding call over it takes the i64 fold on both sides.
    pub(crate) fn static_numeric_chain(&self, id: LirNodeId) -> Option<f64> {
        self.static_numeric_chain_at(id, false, 0)
    }

    fn static_numeric_chain_at(
        &self,
        id: LirNodeId,
        module_only: bool,
        depth: usize,
    ) -> Option<f64> {
        if depth > 32 {
            return None;
        }
        let node = self.node(id);
        if self.is_object_freeze_call(node) {
            let arg = *node.children.get(1)?;
            return self.static_numeric_chain_at(arg, module_only, depth + 1);
        }
        match node.kind {
            LirNodeKind::Literal => node.text.as_deref().and_then(parse_numeric_literal_value),
            LirNodeKind::Value if node.children.is_empty() => {
                let name = node.text.as_deref()?;
                if !module_only {
                    if let Some(&bound) = self.bindings.get(name) {
                        return self.static_numeric_chain_at(bound, false, depth + 1);
                    }
                }
                let module_scope = module_only
                    || (!self.locals.contains_key(name) && self.function_name != "_start");
                if module_scope {
                    if let Some(&init) = self.module_const_inits.get(name) {
                        return self.static_numeric_chain_at(init, true, depth + 1);
                    }
                }
                parse_numeric_literal_value(name)
            }
            LirNodeKind::Value if node.children.len() == 1 => match node.text.as_deref() {
                None | Some("") | Some("await") | Some("+") => {
                    self.static_numeric_chain_at(node.children[0], module_only, depth + 1)
                }
                Some("-") => self
                    .static_numeric_chain_at(node.children[0], module_only, depth + 1)
                    .map(|v| -v),
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn math_abs_static_literal_value(&self, arg: LirNodeId) -> Option<i64> {
        let Some(rendered) = self.render_static_value(arg) else {
            // Residual round 2: a module `const` chain read from a function.
            let value = self.static_numeric_chain(arg)?;
            if value.fract() != 0.0 || !value.is_finite() || value.abs() > i64::MAX as f64 {
                return None;
            }
            return (value as i64).checked_abs();
        };
        parse_number_literal(&rendered)?.checked_abs()
    }

    pub(crate) fn math_imul_static_literal_value(
        &self,
        left: LirNodeId,
        right: LirNodeId,
    ) -> Option<i64> {
        let rendered_left = self.render_static_value(left)?;
        let rendered_right = self.render_static_value(right)?;
        let left = parse_number_literal(&rendered_left)? as i32;
        let right = parse_number_literal(&rendered_right)? as i32;
        Some(i64::from(left.wrapping_mul(right)))
    }

    pub(crate) fn math_clz32_static_literal_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_numeric_literal_value(&rendered)?;
        let uint32 = self.to_uint32_literal_value(value)?;
        Some(i64::from(uint32.leading_zeros()))
    }

    pub(crate) fn math_sign_static_literal_value(&self, arg: LirNodeId) -> Option<i64> {
        let rendered = self.render_static_value(arg)?;
        let value = parse_numeric_literal_value(&rendered)?;
        Some(if value == 0.0 {
            0
        } else if value.is_sign_negative() {
            -1
        } else {
            1
        })
    }
}

#[cfg(test)]
#[path = "math_tests.rs"]
mod math_tests;

/// Growable-runtime-arrays residual R2 (spec A-39): the `Math` methods that
/// take the f64 lane when an operand is a floating-point value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MathFloatMethod {
    Floor,
    Ceil,
    Trunc,
    Round,
    Abs,
    Min,
    Max,
}

impl MathFloatMethod {
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "floor" => Self::Floor,
            "ceil" => Self::Ceil,
            "trunc" => Self::Trunc,
            "round" => Self::Round,
            "abs" => Self::Abs,
            "min" => Self::Min,
            "max" => Self::Max,
            _ => return None,
        })
    }

    fn name(self) -> &'static str {
        match self {
            Self::Floor => "floor",
            Self::Ceil => "ceil",
            Self::Trunc => "trunc",
            Self::Round => "round",
            Self::Abs => "abs",
            Self::Min => "min",
            Self::Max => "max",
        }
    }

    /// `min`/`max` read every argument; the others read the first only.
    fn is_extremum(self) -> bool {
        matches!(self, Self::Min | Self::Max)
    }
}

impl<'a> FunctionEmitter<'a> {
    /// Spec A-39: the method when `node` is a `Math.floor`/`ceil`/`trunc`/
    /// `round`/`abs` call whose first argument is a floating-point value
    /// other than a signed numeric literal (a literal keeps the constant-fold
    /// lane), or a `Math.min`/`Math.max` call with any floating-point
    /// argument. The result is then an f64. Mirrors `repr_infer`'s float
    /// edges for the same calls (`math_float_operand_edges`), so the two
    /// agree on which results are f64.
    pub(crate) fn math_float_lane(&self, node: &LirNode) -> Option<MathFloatMethod> {
        if node.kind != LirNodeKind::Call {
            return None;
        }
        let callee = self.unwrap_transparent(*node.children.first()?);
        let callee_node = self.node(callee);
        // Only the bare `Math.<method>(…)` spelling, which is the one
        // `repr_infer` gives float edges.
        let object = self.node(self.unwrap_transparent(*callee_node.children.first()?));
        if object.kind != LirNodeKind::Value
            || !object.children.is_empty()
            || object.text.as_deref() != Some("Math")
            || !self.is_math_object(callee_node)
        {
            return None;
        }
        let method = MathFloatMethod::from_name(callee_node.text.as_deref()?)?;
        let args = &node.children[1..];
        let float = if method.is_extremum() {
            args.iter().any(|&arg| self.is_float_valued(arg))
        } else {
            args.first().is_some_and(|&arg| {
                // Residual round 1: a compile-time number (a literal, or a
                // `const` alias chain of one) keeps the i64 fold lane, as on
                // `main`; `repr_infer` gives it no float edge either
                // (`is_static_numeric`).
                self.is_float_valued(arg) && !self.is_static_numeric_arg(arg)
            })
        };
        float.then_some(method)
    }

    /// A compile-time number: a literal, a unary sign or `Object.freeze` of
    /// one, a local `const` fold alias of one, or (from a function) a
    /// module `const` whose initializer is one. Mirrors `repr_infer`'s
    /// `is_static_numeric`.
    fn is_static_numeric_arg(&self, id: LirNodeId) -> bool {
        self.static_numeric_chain(id).is_some()
    }

    /// Spec A-39: lower a `math_float_lane` call. Every argument is
    /// evaluated left to right through `emit_integer_math_arg` (which keeps
    /// its string and non-integer-literal refusals), an integer one promoted
    /// to f64; `min`/`max` fold with `f64.min`/`f64.max` (NaN-propagating,
    /// `-0 < +0`, as JS), the others apply to the first argument and evaluate
    /// and drop the rest. `round` is JS's: `floor(x)`, plus one when
    /// `x - floor(x) >= 0.5`, with the sign of `x` (so `-0.25` gives `-0`).
    /// Leaves an f64.
    pub(crate) fn emit_math_float_lane(
        &mut self,
        function: &mut Function,
        node: &LirNode,
        method: MathFloatMethod,
    ) -> EmittedValue {
        let args: Vec<LirNodeId> = node.children[1..].to_vec();
        for (position, &arg) in args.iter().enumerate() {
            let is_float = self.is_float_valued(arg);
            if !self.emit_integer_math_arg(function, arg, method.name()) {
                return EmittedValue {
                    produced: false,
                    shape: ValueShape::Unknown,
                };
            }
            if position > 0 && !method.is_extremum() {
                function.instruction(&Instruction::Drop);
                continue;
            }
            if !is_float {
                function.instruction(&Instruction::F64ConvertI64S);
            }
            match method {
                MathFloatMethod::Min if position > 0 => {
                    function.instruction(&Instruction::F64Min);
                }
                MathFloatMethod::Max if position > 0 => {
                    function.instruction(&Instruction::F64Max);
                }
                MathFloatMethod::Floor => {
                    function.instruction(&Instruction::F64Floor);
                }
                MathFloatMethod::Ceil => {
                    function.instruction(&Instruction::F64Ceil);
                }
                MathFloatMethod::Trunc => {
                    function.instruction(&Instruction::F64Trunc);
                }
                MathFloatMethod::Abs => {
                    function.instruction(&Instruction::F64Abs);
                }
                MathFloatMethod::Round => self.emit_f64_js_round(function),
                MathFloatMethod::Min | MathFloatMethod::Max => {}
            }
        }
        EmittedValue {
            produced: true,
            shape: ValueShape::Float,
        }
    }

    /// JS `Math.round` of the f64 on top of the stack. Holds the operand's
    /// bits in the dedicated `__math_round_scratch` local for the few
    /// instructions below only; nothing is emitted in between, so a nested
    /// `Math.round` (already finished) cannot clobber it.
    fn emit_f64_js_round(&self, function: &mut Function) {
        let scratch = self
            .locals
            .get(crate::lower::math_round_scratch_local_name().as_str())
            .copied()
            .expect("the `Math.round` scratch is reserved for any body that reads `.round`");
        let x = |function: &mut Function| {
            function.instruction(&Instruction::LocalGet(scratch));
            function.instruction(&Instruction::F64ReinterpretI64);
        };
        function.instruction(&Instruction::I64ReinterpretF64);
        function.instruction(&Instruction::LocalSet(scratch));
        // floor(x)
        x(function);
        function.instruction(&Instruction::F64Floor);
        // + (x - floor(x) >= 0.5)
        x(function);
        x(function);
        function.instruction(&Instruction::F64Floor);
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::F64Const(0.5.into()));
        function.instruction(&Instruction::F64Ge);
        function.instruction(&Instruction::F64ConvertI32U);
        function.instruction(&Instruction::F64Add);
        // with the sign of x
        x(function);
        function.instruction(&Instruction::F64Copysign);
    }
}
