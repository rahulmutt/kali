//! Static recognition and constant-folding of Number intrinsic call shapes.
use crate::*;

pub(crate) fn parse_number_literal(text: &str) -> Option<i64> {
    if let Some(stripped) = text.strip_suffix('n') {
        return stripped.parse::<i64>().ok();
    }
    text.parse::<i64>().ok()
}

pub(crate) fn parse_numeric_literal_value(text: &str) -> Option<f64> {
    if let Some(stripped) = text.strip_suffix('n') {
        return stripped.parse::<f64>().ok();
    }
    text.parse::<f64>().ok()
}

pub(crate) fn is_supported_static_ascii_char_code(value: f64) -> bool {
    value.is_finite() && value.fract() == 0.0 && (0.0..=127.0).contains(&value)
}

pub(crate) fn static_parse_float_ascii_integer(source: &str) -> Option<i64> {
    if !source.is_ascii() {
        return None;
    }

    let trimmed = source.trim_start_matches(|ch: char| ch.is_ascii_whitespace());
    let mut end = 0;
    let bytes = trimmed.as_bytes();
    if matches!(bytes.get(end), Some(b'+' | b'-')) {
        end += 1;
    }

    let digits_before = bytes[end..]
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    end += digits_before;

    let mut digits_after = 0;
    if bytes.get(end) == Some(&b'.') {
        end += 1;
        digits_after = bytes[end..]
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count();
        end += digits_after;
    }

    if digits_before + digits_after == 0 {
        return None;
    }

    if matches!(bytes.get(end), Some(b'e' | b'E')) {
        let exponent_marker = end;
        end += 1;
        if matches!(bytes.get(end), Some(b'+' | b'-')) {
            end += 1;
        }
        let exponent_digits = bytes[end..]
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count();
        if exponent_digits == 0 {
            end = exponent_marker;
        } else {
            end += exponent_digits;
        }
    }

    let value = trimmed[..end].parse::<f64>().ok()?;
    if !value.is_finite()
        || value.fract() != 0.0
        || value < i64::MIN as f64
        || value > i64::MAX as f64
    {
        return None;
    }

    Some(value as i64)
}

pub(crate) fn static_parse_int_ascii(source: &str, radix: u32) -> Option<i64> {
    if !source.is_ascii() || !(radix == 0 || (2..=36).contains(&radix)) {
        return None;
    }

    let trimmed = source.trim_start_matches(|ch: char| ch.is_ascii_whitespace());
    let (negative, rest) = if let Some(rest) = trimmed.strip_prefix('-') {
        (true, rest)
    } else if let Some(rest) = trimmed.strip_prefix('+') {
        (false, rest)
    } else {
        (false, trimmed)
    };

    let (radix, digits) = if radix == 0 {
        if let Some(rest) = rest.strip_prefix("0x").or_else(|| rest.strip_prefix("0X")) {
            (16, rest)
        } else {
            (10, rest)
        }
    } else if radix == 16 {
        (
            16,
            rest.strip_prefix("0x")
                .or_else(|| rest.strip_prefix("0X"))
                .unwrap_or(rest),
        )
    } else {
        (radix, rest)
    };

    let mut value: i64 = 0;
    let mut consumed = false;
    for ch in digits.chars() {
        let Some(digit) = ch.to_digit(radix) else {
            break;
        };
        consumed = true;
        value = value.checked_mul(radix as i64)?.checked_add(digit as i64)?;
    }

    if !consumed {
        return None;
    }

    if negative {
        value.checked_neg()
    } else {
        Some(value)
    }
}

impl<'a> FunctionEmitter<'a> {
    pub(crate) fn resolve_static_global_number_predicate_call(
        &self,
        node: &LirNode,
        callee_node: &LirNode,
    ) -> Option<bool> {
        if node.kind != LirNodeKind::Call || node.children.len() < 2 {
            return None;
        }

        let callee_id = *node.children.first()?;
        let callee_node = self
            .resolve_transparent_callable_node(callee_id)
            .map(|id| self.node(id))
            .unwrap_or(callee_node);
        let method = self.global_number_predicate_callable_method(callee_node)?;
        let StaticObjectIdentityValue::Number(number) =
            self.resolve_static_object_identity_value(*node.children.get(1)?)?
        else {
            return None;
        };

        match method {
            "isFinite" => Some(number.is_finite()),
            "isNaN" => Some(number.is_nan()),
            _ => None,
        }
    }

    pub(crate) fn resolve_static_parse_int_call(
        &self,
        node: &LirNode,
        callee_node: &LirNode,
    ) -> Option<i64> {
        if node.kind != LirNodeKind::Call || !(2..=3).contains(&node.children.len()) {
            return None;
        }

        let callee_id = *node.children.first()?;
        let callee_node = self
            .resolve_transparent_callable_node(callee_id)
            .map(|id| self.node(id))
            .unwrap_or(callee_node);
        if !self.is_parse_int_callable(callee_node) {
            return None;
        }

        let source = match self.resolve_static_object_identity_value(*node.children.get(1)?)? {
            StaticObjectIdentityValue::String(value) if value.is_ascii() => value,
            _ => return None,
        };
        let radix = match node.children.get(2) {
            Some(id) => {
                let radix = self.resolve_static_numeric_value(*id)?;
                if !radix.is_finite() || radix.fract() != 0.0 {
                    return None;
                }
                radix as u32
            }
            None => 0,
        };
        static_parse_int_ascii(&source, radix)
    }

    pub(crate) fn resolve_static_parse_float_call(
        &self,
        node: &LirNode,
        callee_node: &LirNode,
    ) -> Option<i64> {
        if node.kind != LirNodeKind::Call || node.children.len() != 2 {
            return None;
        }

        let callee_id = *node.children.first()?;
        let callee_node = self
            .resolve_transparent_callable_node(callee_id)
            .map(|id| self.node(id))
            .unwrap_or(callee_node);
        if !self.is_parse_float_callable(callee_node) {
            return None;
        }

        let source = match self.resolve_static_object_identity_value(*node.children.get(1)?)? {
            StaticObjectIdentityValue::String(value) if value.is_ascii() => value,
            _ => return None,
        };
        static_parse_float_ascii_integer(&source)
    }

    pub(crate) fn global_number_predicate_callable_method<'b>(
        &self,
        callee_node: &'b LirNode,
    ) -> Option<&'b str> {
        let method = callee_node.text.as_deref()?;
        if method == "isFinite" || method == "isNaN" {
            if callee_node.children.is_empty() {
                return Some(method);
            }

            let object = callee_node.children.first().copied()?;
            let object = self.resolve_transparent_object_root_node(object)?;
            if matches!(self.node(object).text.as_deref(), Some("globalThis")) {
                return Some(method);
            }
        }

        None
    }

    pub(crate) fn is_parse_int_callable(&self, callee_node: &LirNode) -> bool {
        self.is_number_parse_callable(callee_node, "parseInt")
    }

    pub(crate) fn is_parse_float_callable(&self, callee_node: &LirNode) -> bool {
        self.is_number_parse_callable(callee_node, "parseFloat")
    }

    pub(crate) fn is_number_parse_callable(&self, callee_node: &LirNode, expected: &str) -> bool {
        let Some(method) = callee_node.text.as_deref() else {
            return false;
        };

        if method == expected && callee_node.children.is_empty() {
            return true;
        }

        if method != expected {
            return false;
        }

        let Some(object) = callee_node.children.first().copied() else {
            return false;
        };
        let Some(object) = self.resolve_transparent_object_root_node(object) else {
            return false;
        };
        matches!(
            self.node(object).text.as_deref(),
            Some("Number")
                | Some("globalThis.Number")
                | Some(r#"globalThis["Number"]"#)
                | Some(r#"globalThis['Number']"#)
                | Some("globalThis")
        )
    }

    pub(crate) fn resolve_static_numeric_reducer_callback(
        &self,
        callback: LirNodeId,
        accumulator: f64,
        current: f64,
    ) -> Option<f64> {
        let callback = self.resolve_transparent_callable_node(callback)?;
        let callback_node = self.node(callback);
        if callback_node.function_flavor != Some(FunctionFlavor::Sync)
            || callback_node.kind != LirNodeKind::Instruction
            || callback_node.children.len() < 3
        {
            return None;
        }

        let accumulator_name = self.node(callback_node.children[0]).text.as_deref()?;
        let current_name = self.node(callback_node.children[1]).text.as_deref()?;
        let body_node = self.node(*callback_node.children.last()?);
        let body_expr = body_node.children.first().copied()?;
        self.resolve_static_numeric_reducer_expr(
            body_expr,
            accumulator_name,
            current_name,
            accumulator,
            current,
        )
    }

    pub(crate) fn resolve_static_numeric_reducer_expr(
        &self,
        id: LirNodeId,
        accumulator_name: &str,
        current_name: &str,
        accumulator: f64,
        current: f64,
    ) -> Option<f64> {
        let node = self.node(id);
        if node.kind == LirNodeKind::Value
            && node.children.len() == 1
            && node.text.as_deref().is_none_or(|text| text.is_empty())
        {
            return self.resolve_static_numeric_reducer_expr(
                node.children[0],
                accumulator_name,
                current_name,
                accumulator,
                current,
            );
        }

        match node.kind {
            LirNodeKind::Literal => node.text.as_deref().and_then(parse_numeric_literal_value),
            LirNodeKind::Value if node.children.is_empty() => match node.text.as_deref()? {
                name if name == accumulator_name => Some(accumulator),
                name if name == current_name => Some(current),
                _ => self.resolve_static_numeric_value(id),
            },
            LirNodeKind::Value if node.children.len() == 1 => match node.text.as_deref() {
                None | Some("") | Some("+") => self.resolve_static_numeric_reducer_expr(
                    node.children[0],
                    accumulator_name,
                    current_name,
                    accumulator,
                    current,
                ),
                Some("-") => self
                    .resolve_static_numeric_reducer_expr(
                        node.children[0],
                        accumulator_name,
                        current_name,
                        accumulator,
                        current,
                    )
                    .map(|value| -value),
                _ => None,
            },
            LirNodeKind::Value if node.children.len() == 2 => {
                let left = self.resolve_static_numeric_reducer_expr(
                    node.children[0],
                    accumulator_name,
                    current_name,
                    accumulator,
                    current,
                )?;
                let right = self.resolve_static_numeric_reducer_expr(
                    node.children[1],
                    accumulator_name,
                    current_name,
                    accumulator,
                    current,
                )?;
                match node.text.as_deref() {
                    Some("+") => Some(left + right),
                    Some("-") => Some(left - right),
                    Some("*") => Some(left * right),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn resolve_static_numeric_value(&self, id: LirNodeId) -> Option<f64> {
        let node = self.node(id);
        if self.is_object_freeze_call(node) {
            return node
                .children
                .get(1)
                .copied()
                .and_then(|child| self.resolve_static_numeric_value(child));
        }
        match node.kind {
            LirNodeKind::Literal => node.text.as_deref().and_then(parse_numeric_literal_value),
            LirNodeKind::Value if node.children.is_empty() => {
                let text = node.text.as_deref()?;
                if let Some(bound) = self.bindings.get(text).copied() {
                    return self.resolve_static_numeric_value(bound);
                }
                parse_numeric_literal_value(text)
            }
            LirNodeKind::Value if node.children.len() == 1 => match node.text.as_deref() {
                None | Some("") => self.resolve_static_numeric_value(node.children[0]),
                Some("+") => self.resolve_static_numeric_value(node.children[0]),
                Some("-") => self
                    .resolve_static_numeric_value(node.children[0])
                    .map(|value| if value == 0.0 { -0.0 } else { -value }),
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn emit_integer_math_arg(
        &mut self,
        function: &mut Function,
        arg: LirNodeId,
        method: &str,
    ) -> bool {
        if self.contains_non_integer_numeric_literal(arg) {
            self.diagnostics.push(Diagnostic::error(
                e5::FEATURE_UNAVAILABLE as u32,
                format!(
                    "Math.{method} is unavailable for non-integer numeric literals in the current phase; use an integer-valued expression or the later compatibility path"
                ),
            ));
            function.instruction(&Instruction::Unreachable);
            return false;
        }

        let _ = self.emit_node(function, arg, true);
        true
    }

    pub(crate) fn static_bigint_literal_value(&self, id: LirNodeId) -> Option<i64> {
        let id = self.resolve_bound_node(id);
        let node = self.node(id);
        if node.kind != LirNodeKind::Literal {
            return None;
        }

        parse_number_literal(node.text.as_deref()?)
    }

    pub(crate) fn contains_negative_numeric_literal(&self, id: LirNodeId) -> bool {
        self.render_static_value(id)
            .and_then(|rendered| parse_number_literal(&rendered))
            .is_some_and(|value| value < 0)
    }

    pub(crate) fn to_uint32_literal_value(&self, value: f64) -> Option<u32> {
        if !value.is_finite() {
            return Some(0);
        }

        let truncated = value.trunc();
        let modulo = truncated.rem_euclid(4_294_967_296.0);
        Some(modulo as u32)
    }

    pub(crate) fn contains_non_integer_numeric_literal(&self, arg: LirNodeId) -> bool {
        self.render_static_value(arg)
            .and_then(|rendered| parse_numeric_literal_value(&rendered))
            .is_some_and(|value| value.fract() != 0.0)
    }
}

#[cfg(test)]
#[path = "number_tests.rs"]
mod number_tests;
