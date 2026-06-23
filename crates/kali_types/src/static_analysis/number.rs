//! Number static-analysis helpers.
use crate::*;

impl TypeContext {
    pub(crate) fn resolve_number_identity_call(&mut self, expr: &CallExpression) -> bool {
        let Some(callee_name) = self.resolve_static_callable_name(&expr.callee) else {
            return false;
        };

        let Some(method) = callee_name
            .strip_prefix("Number.")
            .or_else(|| callee_name.strip_prefix("globalThis.Number."))
            .or_else(|| callee_name.strip_prefix(r#"globalThis["Number"]."#))
            .or_else(|| callee_name.strip_prefix(r#"globalThis['Number']."#))
        else {
            return false;
        };

        if !matches!(method, "isFinite" | "isNaN" | "isInteger" | "isSafeInteger") {
            return false;
        }

        let Some(value_expr) = expr.args.first() else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Number.{method} requires at least one statically-known primitive value in the current phase; use an explicit constant or the later compatibility path"
                ),
            ));
            return true;
        };

        let Some(value) = self.resolve_static_object_identity_literal_value(value_expr) else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Number.{method} is unavailable unless the argument is a statically-known primitive value in the current phase; use an explicit constant or the later compatibility path"
                ),
            ));
            return true;
        };

        self.resolve_expression(value_expr);
        for arg in expr.args.iter().skip(1) {
            self.resolve_expression(arg);
        }

        let _ = match value {
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

        true
    }

    pub(crate) fn resolve_global_number_predicate_call(&mut self, expr: &CallExpression) -> bool {
        let Some(callee_name) =
            self.resolve_static_callable_name(&expr.callee)
                .or_else(|| match &expr.callee {
                    Expression::Identifier(name) => Some(name.clone()),
                    _ => None,
                })
        else {
            return false;
        };

        let method = match callee_name.as_str() {
            "isFinite"
            | "globalThis.isFinite"
            | r#"globalThis["isFinite"]"#
            | r#"globalThis['isFinite']"# => "isFinite",
            "isNaN" | "globalThis.isNaN" | r#"globalThis["isNaN"]"# | r#"globalThis['isNaN']"# => {
                "isNaN"
            }
            _ => return false,
        };

        let Some(value_expr) = expr.args.first() else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "global {method} requires at least one statically-known numeric value in the current phase; use an explicit numeric constant or the later compatibility path"
                ),
            ));
            return true;
        };

        let Some(StaticObjectIdentityValue::Number(number)) =
            self.resolve_static_object_identity_literal_value(value_expr)
        else {
            self.resolve_expression(value_expr);
            for arg in expr.args.iter().skip(1) {
                self.resolve_expression(arg);
            }
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "global {method} is unavailable unless the argument is a statically-known numeric value in the current direct-runtime path; use an explicit numeric constant or the later compatibility path"
                ),
            ));
            return true;
        };

        self.resolve_expression(value_expr);
        for arg in expr.args.iter().skip(1) {
            self.resolve_expression(arg);
        }

        let _ = match method {
            "isFinite" => number.is_finite(),
            "isNaN" => number.is_nan(),
            _ => false,
        };

        true
    }

    pub(crate) fn resolve_number_parse_int_call(&mut self, expr: &CallExpression) -> bool {
        let Some(callee_name) =
            self.resolve_static_callable_name(&expr.callee)
                .or_else(|| match &expr.callee {
                    Expression::Identifier(name) => Some(name.clone()),
                    _ => None,
                })
        else {
            return false;
        };

        if !matches!(
            callee_name.as_str(),
            "parseInt"
                | "globalThis.parseInt"
                | r#"globalThis["parseInt"]"#
                | r#"globalThis['parseInt']"#
                | "Number.parseInt"
                | "globalThis.Number.parseInt"
                | r#"globalThis["Number"].parseInt"#
                | r#"globalThis['Number'].parseInt"#
                | r#"Number["parseInt"]"#
                | r#"Number['parseInt']"#
                | r#"globalThis.Number["parseInt"]"#
                | r#"globalThis.Number['parseInt']"#
                | r#"globalThis["Number"]["parseInt"]"#
                | r#"globalThis['Number']['parseInt']"#
        ) {
            return false;
        }

        let Some(value_expr) = expr.args.first() else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "parseInt requires at least one statically-known ASCII string argument in the current phase; use an explicit literal or the later compatibility path",
            ));
            return true;
        };

        let source = self.resolve_static_string_expression(value_expr);
        let radix = expr
            .args
            .get(1)
            .and_then(|argument| self.resolve_static_numeric_literal_value(argument));
        let supported_radix = expr.args.get(1).is_none_or(|_| {
            radix.is_some_and(|radix| {
                radix.is_finite()
                    && radix.fract() == 0.0
                    && (radix == 0.0 || (2.0..=36.0).contains(&radix))
            })
        });

        if matches!(expr.args.len(), 1 | 2)
            && source.as_ref().is_some_and(|source| source.is_ascii())
            && supported_radix
            && source
                .as_ref()
                .zip(Some(radix.unwrap_or(0.0)))
                .is_some_and(|(source, radix)| {
                    static_parse_int_ascii(source, radix as u32).is_some()
                })
        {
            self.resolve_expression(value_expr);
            for arg in expr.args.iter().skip(1) {
                self.resolve_expression(arg);
            }
            return true;
        }

        self.resolve_expression(value_expr);
        for arg in expr.args.iter().skip(1) {
            self.resolve_expression(arg);
        }
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "parseInt is unavailable unless the input is a statically-known ASCII string that yields an integer result and the optional radix is omitted, 0, or a statically-known integer from 2 through 36 in the current direct-runtime path; use explicit literals or the later compatibility path",
        ));
        true
    }

    pub(crate) fn resolve_number_parse_float_call(&mut self, expr: &CallExpression) -> bool {
        let Some(callee_name) =
            self.resolve_static_callable_name(&expr.callee)
                .or_else(|| match &expr.callee {
                    Expression::Identifier(name) => Some(name.clone()),
                    _ => None,
                })
        else {
            return false;
        };

        if !matches!(
            callee_name.as_str(),
            "parseFloat"
                | "globalThis.parseFloat"
                | r#"globalThis["parseFloat"]"#
                | r#"globalThis['parseFloat']"#
                | "Number.parseFloat"
                | "globalThis.Number.parseFloat"
                | r#"globalThis["Number"].parseFloat"#
                | r#"globalThis['Number'].parseFloat"#
                | r#"Number["parseFloat"]"#
                | r#"Number['parseFloat']"#
                | r#"globalThis.Number["parseFloat"]"#
                | r#"globalThis.Number['parseFloat']"#
                | r#"globalThis["Number"]["parseFloat"]"#
                | r#"globalThis['Number']['parseFloat']"#
        ) {
            return false;
        }

        let Some(value_expr) = expr.args.first() else {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                "parseFloat requires at least one statically-known ASCII string argument in the current phase; use an explicit literal or the later compatibility path",
            ));
            return true;
        };

        let source = self.resolve_static_string_expression(value_expr);
        if expr.args.len() == 1
            && source.as_ref().is_some_and(|source| source.is_ascii())
            && source
                .as_ref()
                .is_some_and(|source| static_parse_float_ascii_integer(source).is_some())
        {
            self.resolve_expression(value_expr);
            return true;
        }

        self.resolve_expression(value_expr);
        for arg in expr.args.iter().skip(1) {
            self.resolve_expression(arg);
        }
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            "parseFloat is unavailable unless the input is a statically-known ASCII string that yields a bounded integer result in the current direct-runtime path; use explicit literals or the later compatibility path",
        ));
        true
    }
}

#[cfg(test)]
#[path = "number_tests.rs"]
mod number_tests;
