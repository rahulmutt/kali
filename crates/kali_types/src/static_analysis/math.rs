//! Math static-analysis helpers.
use crate::*;

impl TypeContext {
    pub(crate) fn resolve_static_numeric_binding(&self, name: &str) -> Option<f64> {
        let mut current = self.current_scope_id();
        while let Some(scope_id) = current {
            let scope = self.scopes.get(&scope_id)?;
            if let Some(value) = scope.static_numeric_values.get(name) {
                return parse_numeric_literal_value(value);
            }
            current = scope.parent;
        }

        self.global_scope
            .static_numeric_values
            .get(name)
            .and_then(|value| parse_numeric_literal_value(value))
    }

    pub(crate) fn resolve_math_member_call(&mut self, expr: &CallExpression) {
        let Some(callee_name) = self.resolve_static_callable_name(&expr.callee) else {
            return;
        };

        let Some(method) = callee_name
            .strip_prefix("Math.")
            .or_else(|| callee_name.strip_prefix("globalThis.Math."))
        else {
            return;
        };

        if method == "hypot" {
            if self
                .resolve_math_hypot_static_literal_root(&expr.args)
                .is_some()
            {
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "Math.hypot is unavailable unless every argument is a statically-known integer literal whose squared sum is a perfect-square integer literal in the current phase; use explicit constants or the later compatibility path",
            ));
            return;
        }

        if method == "sqrt" || method == "cbrt" || method == "log2" || method == "log10" {
            let literal_root = expr.args.first().and_then(|arg| {
                if method == "sqrt" {
                    self.resolve_math_sqrt_static_literal_root(arg)
                } else if method == "cbrt" {
                    self.resolve_math_cbrt_static_literal_root(arg)
                } else if method == "log2" {
                    self.resolve_math_log2_static_literal_exponent(arg)
                } else {
                    self.resolve_math_log10_static_literal_exponent(arg)
                }
            });
            if literal_root.is_some() {
                return;
            }

            if method == "sqrt" {
                // Runtime `Math.sqrt` on a non-perfect-square argument now lowers
                // directly to `F64Sqrt` in codegen (see `emit/call.rs`); only
                // `cbrt`/`log2`/`log10` remain constant-fold-only in this phase.
                return;
            }

            let shape = if method == "cbrt" {
                "perfect-cube"
            } else if method == "log2" {
                "positive power-of-two"
            } else {
                "positive power-of-ten"
            };
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Math.{method} is unavailable unless the argument is a statically-known {shape} integer literal in the current phase; use an explicit constant or the later compatibility path"
                ),
            ));
            return;
        }

        if method == "exp" || method == "log" || method == "exp2" {
            let Some(value) = expr
                .args
                .first()
                .and_then(|arg| self.resolve_static_numeric_literal_value(arg))
            else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                        match method {
                            "exp" => "zero",
                            "exp2" => "non-negative integer literal within the current integer-fold range",
                            _ => "one",
                        }
                    ),
                ));
                return;
            };

            if (method == "exp" && value == 0.0)
                || (method == "log" && value == 1.0)
                || (method == "exp2"
                    && value.is_finite()
                    && value.fract() == 0.0
                    && (0.0..=62.0).contains(&value))
            {
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                    match method {
                        "exp" => "zero",
                        "exp2" => "non-negative integer literal within the current integer-fold range",
                        _ => "one",
                    }
                ),
            ));
            return;
        }

        if method == "expm1" || method == "log1p" || method == "fround" {
            let Some(value) = expr
                .args
                .first()
                .and_then(|arg| self.resolve_static_numeric_literal_value(arg))
            else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                    ),
                ));
                return;
            };

            if value == 0.0 {
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                ),
            ));
            return;
        }

        if method == "asin" || method == "acos" || method == "atan" {
            let Some(value) = expr
                .args
                .first()
                .and_then(|arg| self.resolve_static_numeric_literal_value(arg))
            else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                        if method == "acos" { "one" } else { "zero" }
                    ),
                ));
                return;
            };

            if (method == "acos" && value == 1.0) || (method != "acos" && value == 0.0) {
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                    if method == "acos" { "one" } else { "zero" }
                ),
            ));
            return;
        }

        if method == "atan2" {
            let atan2_message = "Math.atan2 is unavailable unless the first argument is a statically-known zero numeric literal and the second argument is a statically-known non-negative numeric literal in the current phase; use explicit constants or the later compatibility path".to_string();
            let Some(y) = expr
                .args
                .first()
                .and_then(|arg| self.resolve_static_numeric_literal_value(arg))
            else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    atan2_message,
                ));
                return;
            };

            let Some(x) = expr
                .args
                .get(1)
                .and_then(|arg| self.resolve_static_numeric_literal_value(arg))
            else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    atan2_message,
                ));
                return;
            };

            if y == 0.0 && x.is_finite() && x >= 0.0 {
                for arg in expr.args.iter().skip(2) {
                    self.resolve_expression(arg);
                }
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                atan2_message,
            ));
            return;
        }

        if method == "sin" || method == "cos" || method == "tan" {
            let Some(argument) = expr.args.first() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                    ),
                ));
                return;
            };

            let Some(value) = self.resolve_static_numeric_literal_value(argument) else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                    ),
                ));
                return;
            };

            if value == 0.0 {
                self.resolve_expression(argument);
                for arg in expr.args.iter().skip(1) {
                    self.resolve_expression(arg);
                }
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                ),
            ));
            return;
        }

        if method == "asinh" || method == "acosh" || method == "atanh" {
            let Some(argument) = expr.args.first() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                        if method == "acosh" { "one" } else { "zero" }
                    ),
                ));
                return;
            };

            if self
                .resolve_math_inverse_hyperbolic_constant_value(method, argument)
                .is_some()
            {
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Math.{method} is unavailable unless the argument is a statically-known {} numeric literal in the current phase; use an explicit constant or the later compatibility path",
                    if method == "acosh" { "one" } else { "zero" }
                ),
            ));
            return;
        }

        if method == "sinh" || method == "cosh" || method == "tanh" {
            let Some(argument) = expr.args.first() else {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                    ),
                ));
                return;
            };

            if self
                .resolve_math_hyperbolic_zero_constant_value(method, argument)
                .is_some()
            {
                return;
            }

            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Math.{method} is unavailable unless the argument is a statically-known zero numeric literal in the current phase; use an explicit constant or the later compatibility path"
                ),
            ));
            return;
        }

        if method == "max" || method == "min" {
            if expr.args.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                    ),
                ));
                return;
            }

            if let Some(_folded) =
                self.resolve_math_extrema_static_literal_value(method, &expr.args)
            {
                return;
            }

            if expr
                .args
                .iter()
                .any(|arg| self.contains_non_integer_numeric_literal(arg))
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable for non-integer numeric literals in the current phase; use an integer-valued expression or the later compatibility path"
                    ),
                ));
            }
            return;
        }

        if method == "pow" {
            if Self::contains_optional_chain(&expr.callee) {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.pow is unavailable through optional-chain wrappers in the current phase; use a direct call or the later compatibility path",
                ));
                return;
            }

            if expr.args.len() < 2 {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.pow requires at least two arguments in the current phase; use explicit operands or the later compatibility path",
                ));
                return;
            }

            let base_value = expr
                .args
                .first()
                .and_then(|arg| self.resolve_static_numeric_literal_value(arg));
            let exponent_value = expr
                .args
                .get(1)
                .and_then(|arg| self.resolve_static_numeric_literal_value(arg));
            let exponent_is_static_zero = exponent_value.is_some_and(|value| value == 0.0);
            let base_is_static_zero = base_value.is_some_and(|value| value == 0.0);
            let base_is_static_unit = base_value.is_some_and(|value| value == 1.0 || value == -1.0);
            let exponent_is_positive_integer =
                exponent_value.is_some_and(|value| value > 0.0 && value.fract() == 0.0);
            let exponent_is_negative_integer =
                exponent_value.is_some_and(|value| value < 0.0 && value.fract() == 0.0);
            if base_is_static_zero && exponent_is_positive_integer {
                return;
            }

            if base_is_static_unit && exponent_is_negative_integer {
                return;
            }

            if !exponent_is_static_zero
                && expr
                    .args
                    .iter()
                    .any(|arg| self.contains_non_integer_numeric_literal(arg))
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.pow is unavailable for non-integer numeric literals in the current phase; use an integer-valued exponent or the later compatibility path",
                ));
                return;
            }

            if expr
                .args
                .get(1)
                .is_some_and(|arg| self.contains_negative_numeric_literal(arg))
                && !base_is_static_unit
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.pow is unavailable for negative numeric literals unless the base is a statically-known ±1 in the current phase; use a non-negative exponent or the later compatibility path",
                ));
            }
            return;
        }

        if method == "round" {
            if expr.args.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.round requires at least one argument in the current phase; use an explicit argument or the later compatibility path",
                ));
                return;
            }

            if self
                .resolve_math_round_like_static_literal_value(method, expr.args.first())
                .is_some()
            {
                return;
            }

            if expr
                .args
                .iter()
                .any(|arg| self.contains_non_integer_numeric_literal(arg))
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.round is unavailable for non-integer numeric literals in the current phase; use an integer-valued expression or the later compatibility path",
                ));
            }
            return;
        }

        if method == "floor" {
            if expr.args.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.floor requires at least one argument in the current phase; use an explicit argument or the later compatibility path",
                ));
                return;
            }

            if self
                .resolve_math_round_like_static_literal_value(method, expr.args.first())
                .is_some()
            {
                return;
            }

            if expr
                .args
                .iter()
                .any(|arg| self.contains_non_integer_numeric_literal(arg))
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.floor is unavailable for non-integer numeric literals in the current phase; use an integer-valued expression or the later compatibility path",
                ));
            }
            return;
        }

        if matches!(method, "trunc" | "ceil") {
            if expr.args.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                    ),
                ));
                return;
            }

            if self
                .resolve_math_round_like_static_literal_value(method, expr.args.first())
                .is_some()
            {
                return;
            }

            if expr
                .args
                .iter()
                .any(|arg| self.contains_non_integer_numeric_literal(arg))
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable for non-integer numeric literals in the current phase; use an integer-valued expression or the later compatibility path"
                    ),
                ));
            }
            return;
        }

        if method == "sign" {
            if expr.args.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.sign requires at least one argument in the current phase; use an explicit argument or the later compatibility path",
                ));
                return;
            }

            return;
        }

        if matches!(method, "max" | "min" | "abs" | "asinh" | "acosh" | "atanh") {
            if expr.args.is_empty() {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} requires at least one argument in the current phase; use an explicit argument or the later compatibility path"
                    ),
                ));
                return;
            }

            if expr
                .args
                .iter()
                .any(|arg| self.contains_non_integer_numeric_literal(arg))
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    format!(
                        "Math.{method} is unavailable for non-integer numeric literals in the current phase; use an integer-valued expression or the later compatibility path"
                    ),
                ));
            }
            return;
        }

        if method == "imul" {
            if expr.args.len() < 2 {
                for arg in &expr.args {
                    self.resolve_expression(arg);
                }
                return;
            }

            if expr
                .args
                .iter()
                .any(|arg| self.contains_non_integer_numeric_literal(arg))
            {
                self.diagnostics.push(Diagnostic::error(
                    e5::FEATURE_UNAVAILABLE as u32,
                    "Math.imul is unavailable for non-integer numeric literals in the current phase; use integer-valued operands or the later compatibility path",
                ));
            }
            return;
        }

        if method == "clz32" {
            return;
        }

        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            format!(
                "Math.{method} is unavailable in the current phase; use a supported Math builtin or the later compatibility path"
            ),
        ));
    }

    pub(crate) fn contains_non_integer_numeric_literal(&self, expression: &Expression) -> bool {
        self.resolve_static_numeric_literal_value(expression)
            .is_some_and(|value| value.fract() != 0.0)
    }

    pub(crate) fn resolve_static_numeric_literal_value(
        &self,
        expression: &Expression,
    ) -> Option<f64> {
        match expression {
            Expression::Literal(LiteralValue::Number(value)) => Some(*value),
            Expression::ParenthesizedExpression(expr) => {
                self.resolve_static_numeric_literal_value(&expr.expression)
            }
            Expression::AwaitExpression(expr) => {
                self.resolve_static_numeric_literal_value(&expr.argument)
            }
            Expression::UnaryExpression(expr) if expr.operator == "+" => {
                self.resolve_static_numeric_literal_value(&expr.argument)
            }
            Expression::UnaryExpression(expr) if expr.operator == "-" => self
                .resolve_static_numeric_literal_value(&expr.argument)
                .map(|value| -value),
            Expression::TypeAssertion(expr) => {
                self.resolve_static_numeric_literal_value(&expr.expression)
            }
            Expression::SatisfiesExpression(expr) => {
                self.resolve_static_numeric_literal_value(&expr.expression)
            }
            Expression::OptionalChainExpression(expr) => match expr.inner.as_ref() {
                OptionalChainInner::NonNull { object, .. } => {
                    self.resolve_static_numeric_literal_value(object)
                }
            },
            Expression::ChainExpression(expr) => {
                self.resolve_static_numeric_literal_value(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                self.resolve_static_numeric_literal_value(&expr.expression)
            }
            Expression::SequenceExpression(expr) => expr
                .expressions
                .last()
                .and_then(|expression| self.resolve_static_numeric_literal_value(expression)),
            Expression::ConditionalExpression(expr) => {
                match self.resolve_static_object_identity_literal_value(&expr.test) {
                    Some(StaticObjectIdentityValue::Boolean(true)) => {
                        self.resolve_static_numeric_literal_value(&expr.consequent)
                    }
                    Some(StaticObjectIdentityValue::Boolean(false)) => {
                        self.resolve_static_numeric_literal_value(&expr.alternate)
                    }
                    _ => {
                        let consequent =
                            self.resolve_static_numeric_literal_value(&expr.consequent);
                        let alternate = self.resolve_static_numeric_literal_value(&expr.alternate);
                        match (consequent, alternate) {
                            (Some(consequent), Some(alternate)) if consequent == alternate => {
                                Some(consequent)
                            }
                            _ => None,
                        }
                    }
                }
            }
            Expression::CallExpression(call) if Self::is_object_freeze_call(call) => call
                .args
                .first()
                .and_then(|argument| self.resolve_static_numeric_literal_value(argument)),
            Expression::Identifier(name) => self.resolve_static_numeric_binding(name),
            _ => None,
        }
    }

    pub(crate) fn resolve_math_round_like_static_literal_value(
        &self,
        method: &str,
        expression: Option<&Expression>,
    ) -> Option<i64> {
        let value = self.resolve_static_numeric_literal_value(expression?)?;
        let folded = match method {
            "round" => (value + 0.5).floor(),
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

    pub(crate) fn contains_negative_numeric_literal(&self, expression: &Expression) -> bool {
        self.resolve_static_numeric_literal_value(expression)
            .is_some_and(|value| value < 0.0)
    }

    pub(crate) fn resolve_math_extrema_static_literal_value(
        &self,
        method: &str,
        expressions: &[Expression],
    ) -> Option<i64> {
        let mut values = expressions.iter().map(|expression| {
            let value = self.resolve_static_numeric_literal_value(expression)?;
            if !value.is_finite()
                || value.fract() != 0.0
                || value < i64::MIN as f64
                || value > i64::MAX as f64
            {
                return None;
            }
            Some(value as i64)
        });

        let first = values.next().flatten()?;
        let mut folded = first;

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

    pub(crate) fn resolve_math_inverse_hyperbolic_constant_value(
        &self,
        method: &str,
        expression: &Expression,
    ) -> Option<i64> {
        let value = self.resolve_static_numeric_literal_value(expression)?;

        match method {
            "acosh" if value == 1.0 => Some(0),
            "asinh" | "atanh" if value == 0.0 => Some(0),
            _ => None,
        }
    }

    pub(crate) fn resolve_math_hyperbolic_zero_constant_value(
        &self,
        method: &str,
        expression: &Expression,
    ) -> Option<i64> {
        let value = self.resolve_static_numeric_literal_value(expression)?;
        if value != 0.0 {
            return None;
        }

        Some(if method == "cosh" { 1 } else { 0 })
    }

    pub(crate) fn resolve_math_sqrt_static_literal_root(
        &self,
        expression: &Expression,
    ) -> Option<i64> {
        let value = self.resolve_static_numeric_literal_value(expression)?;
        if !value.is_finite() || value.fract() != 0.0 || value < 0.0 || value > i64::MAX as f64 {
            return None;
        }

        let value = value as i64;
        let root = (value as f64).sqrt() as i64;
        if root.checked_mul(root) == Some(value) {
            Some(root)
        } else {
            None
        }
    }

    pub(crate) fn resolve_math_cbrt_static_literal_root(
        &self,
        expression: &Expression,
    ) -> Option<i64> {
        let value = self.resolve_static_numeric_literal_value(expression)?;
        if !value.is_finite()
            || value.fract() != 0.0
            || value < i64::MIN as f64
            || value > i64::MAX as f64
        {
            return None;
        }

        let value = value as i64;
        let root = (value as f64).cbrt().round() as i64;
        if i128::from(root).pow(3) == i128::from(value) {
            Some(root)
        } else {
            None
        }
    }

    pub(crate) fn resolve_math_log2_static_literal_exponent(
        &self,
        expression: &Expression,
    ) -> Option<i64> {
        let value = self.resolve_static_numeric_literal_value(expression)?;
        if !value.is_finite() || value.fract() != 0.0 || value <= 0.0 || value > u64::MAX as f64 {
            return None;
        }

        let value = value as u64;
        if value.is_power_of_two() {
            Some(i64::from(value.trailing_zeros()))
        } else {
            None
        }
    }

    pub(crate) fn resolve_math_log10_static_literal_exponent(
        &self,
        expression: &Expression,
    ) -> Option<i64> {
        let value = self.resolve_static_numeric_literal_value(expression)?;
        if !value.is_finite() || value.fract() != 0.0 || value <= 0.0 || value > i64::MAX as f64 {
            return None;
        }

        let mut value = value as i64;
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

    pub(crate) fn resolve_math_hypot_static_literal_root(
        &self,
        expressions: &[Expression],
    ) -> Option<i64> {
        if expressions.is_empty() {
            return Some(0);
        }

        let mut sum = 0_i128;
        for expression in expressions {
            let value = self.resolve_static_numeric_literal_value(expression)?;
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

        self.resolve_perfect_square_i128(sum)
    }

    pub(crate) fn resolve_perfect_square_i128(&self, value: i128) -> Option<i64> {
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
#[path = "math_tests.rs"]
mod math_tests;
