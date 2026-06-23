//! Type-checking helpers: annotation parser free-functions, TypeChecker facade, and TypeContext typecheck methods.

use super::*;

pub(crate) fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_ascii_alphabetic()
}

pub(crate) fn is_ident_continue(ch: char) -> bool {
    is_ident_start(ch) || ch.is_ascii_digit()
}

pub(crate) fn is_type_annotation_keyword(ident: &str) -> bool {
    matches!(
        ident,
        "any"
            | "as"
            | "bigint"
            | "boolean"
            | "const"
            | "extends"
            | "false"
            | "infer"
            | "in"
            | "intrinsic"
            | "is"
            | "keyof"
            | "never"
            | "null"
            | "number"
            | "object"
            | "out"
            | "readonly"
            | "string"
            | "symbol"
            | "this"
            | "true"
            | "typeof"
            | "undefined"
            | "unique"
            | "unknown"
            | "void"
    )
}

pub(crate) fn is_property_name_context(chars: &[char], start: usize, end: usize) -> bool {
    if matches!(next_non_whitespace_char(chars, end), Some(':')) {
        return true;
    }

    if matches!(next_non_whitespace_char(chars, end), Some('?')) {
        let mut index = end + 1;
        while index < chars.len() && chars[index].is_whitespace() {
            index += 1;
        }
        return matches!(chars.get(index), Some(':'));
    }

    if start > 0 {
        let mut index = start;
        while index > 0 {
            index -= 1;
            if chars[index].is_whitespace() {
                continue;
            }
            return matches!(chars.get(index), Some('.'));
        }
    }

    false
}

pub(crate) fn next_non_whitespace_char(chars: &[char], mut index: usize) -> Option<char> {
    while index < chars.len() {
        let ch = chars[index];
        if !ch.is_whitespace() {
            return Some(ch);
        }
        index += 1;
    }
    None
}

pub(crate) fn skip_quoted_annotation_segment(chars: &[char], start: usize) -> usize {
    let quote = chars[start];
    let mut index = start + 1;
    while index < chars.len() {
        let ch = chars[index];
        if ch == '\\' {
            index = index.saturating_add(2);
            continue;
        }
        if ch == quote {
            return index + 1;
        }
        index += 1;
    }
    chars.len()
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

/// A lightweight type-checking facade.
#[derive(Default)]
pub struct TypeChecker {
    pub(crate) context: TypeContext,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn clear_diagnostics(&mut self) {
        self.context.clear_diagnostics();
        self.diagnostics.clear();
    }

    pub fn check_type_annotation(&mut self, _node_id: NodeId, annotation: &str) {
        self.context.resolve_type_annotation_text(annotation);
        self.diagnostics.extend(self.context.drain_diagnostics());
    }

    pub fn check_node(&mut self, _node_id: NodeId) {
        let _ = &self.context;
    }

    pub fn typecheck(&mut self, _program_root: NodeId) -> Vec<Diagnostic> {
        self.diagnostics.extend(self.context.drain_diagnostics());
        self.diagnostics.clone()
    }
}

impl TypeContext {
    pub fn check_type_annotation(&mut self, _node_id: NodeId, annotation: &str) {
        self.resolve_type_annotation_text(annotation);
    }

    pub fn check_node(&mut self, _node_id: NodeId) {}

    pub fn typecheck(&mut self, _program_root: NodeId) -> Vec<Diagnostic> {
        self.clear_diagnostics();
        self.diagnostics.clone()
    }

    pub(crate) fn resolve_type_annotation_text(&mut self, annotation: &str) {
        let annotation = annotation.trim();
        if annotation.is_empty() {
            return;
        }

        let chars: Vec<char> = annotation.chars().collect();
        let mut index = 0;
        while index < chars.len() {
            let ch = chars[index];
            if matches!(ch, '\'' | '"' | '`') {
                index = skip_quoted_annotation_segment(&chars, index);
                continue;
            }

            if is_ident_start(ch) {
                let start = index;
                index += 1;
                while index < chars.len() && is_ident_continue(chars[index]) {
                    index += 1;
                }

                let ident: String = chars[start..index].iter().collect();
                if !is_type_annotation_keyword(&ident)
                    && !is_property_name_context(&chars, start, index)
                    && self.resolve_name(&ident).is_none()
                {
                    self.diagnostics.push(
                        Diagnostic::error(
                            e3::UNDEFINED_IDENTIFIER as u32,
                            format!("undefined type reference '{}'", ident),
                        )
                        .with_suggestion(
                            "declare the type or import it before using it in an annotation",
                        ),
                    );
                }
                continue;
            }

            index += 1;
        }
    }
}

#[cfg(test)]
#[path = "typecheck_tests.rs"]
mod typecheck_tests;
