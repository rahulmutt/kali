use crate::*;

impl Optimizer {
    pub(crate) fn optimize_constant_expression(
        &self,
        program: &mut LirProgram,
        id: LirNodeId,
        tracker: &mut SpecializationTracker,
        owner: &str,
    ) -> bool {
        let snapshot = program.nodes[id.0 as usize].clone();
        match snapshot.kind {
            LirNodeKind::Literal => false,
            LirNodeKind::Value => {
                let Some(op) = snapshot.text.as_deref() else {
                    return false;
                };

                match snapshot.children.len() {
                    1 => {
                        let Some(value) = literal_value(program, snapshot.children[0]) else {
                            return false;
                        };
                        if let Some(folded) = fold_unary(op, value) {
                            let key = format!(
                                "unary:{}:{}",
                                op,
                                node_signature(program, snapshot.children[0])
                            );
                            if !tracker.allow(owner, key) {
                                return false;
                            }
                            program.nodes[id.0 as usize] =
                                LirNode::with_text(LirNodeKind::Literal, literal_text(folded));
                            return true;
                        }
                    }
                    2 => {
                        let left = literal_value(program, snapshot.children[0]);
                        let right = literal_value(program, snapshot.children[1]);
                        if let (Some(left), Some(right)) = (left, right) {
                            if let Some(folded) = fold_binary(op, left, right) {
                                let key = format!(
                                    "binary:{}:{}:{}",
                                    op,
                                    node_signature(program, snapshot.children[0]),
                                    node_signature(program, snapshot.children[1])
                                );
                                if !tracker.allow(owner, key) {
                                    return false;
                                }
                                program.nodes[id.0 as usize] =
                                    LirNode::with_text(LirNodeKind::Literal, literal_text(folded));
                                return true;
                            }
                        }
                    }
                    _ => {}
                }
                false
            }
            LirNodeKind::Branch => {
                let Some(cond_id) = snapshot.children.first().copied() else {
                    return false;
                };
                let Some(condition) = literal_value(program, cond_id) else {
                    return false;
                };
                let truthy = condition.truthy();
                let chosen = if truthy {
                    snapshot.children.get(1).copied()
                } else {
                    snapshot.children.get(2).copied()
                };

                let Some(chosen) = chosen else {
                    let key = format!("branch:{}", node_signature(program, cond_id));
                    if !tracker.allow(owner, key) {
                        return false;
                    }
                    program.nodes[id.0 as usize] =
                        LirNode::with_text(LirNodeKind::Literal, if truthy { "1" } else { "0" });
                    return true;
                };

                let key = format!("branch:{}:{}", node_signature(program, cond_id), truthy);
                if !tracker.allow(owner, key) {
                    return false;
                }
                program.nodes[id.0 as usize] = program.nodes[chosen.0 as usize].clone();
                true
            }
            _ => false,
        }
    }

    pub(crate) fn optimize_algebraic_identity(
        &self,
        program: &mut LirProgram,
        id: LirNodeId,
        tracker: &mut SpecializationTracker,
        owner: &str,
    ) -> bool {
        let snapshot = program.nodes[id.0 as usize].clone();
        let Some(op) = snapshot.text.as_deref() else {
            return false;
        };

        match (op, snapshot.children.as_slice()) {
            ("+", [left, right]) => {
                let key = format!(
                    "identity:+:{}:{}",
                    node_signature(program, *left),
                    node_signature(program, *right)
                );
                if !tracker.allow(owner, key) {
                    return false;
                }
                if is_zero_constant(literal_value(program, *left)) {
                    program.nodes[id.0 as usize] = program.nodes[right.0 as usize].clone();
                    return true;
                }
                if is_zero_constant(literal_value(program, *right)) {
                    program.nodes[id.0 as usize] = program.nodes[left.0 as usize].clone();
                    return true;
                }
                false
            }
            ("-", [left, right]) => {
                let key = format!(
                    "identity:-:{}:{}",
                    node_signature(program, *left),
                    node_signature(program, *right)
                );
                if !tracker.allow(owner, key) {
                    return false;
                }
                if is_zero_constant(literal_value(program, *right)) {
                    program.nodes[id.0 as usize] = program.nodes[left.0 as usize].clone();
                    return true;
                }
                false
            }
            ("/", [left, right]) => {
                let key = format!(
                    "identity/:{}:{}",
                    node_signature(program, *left),
                    node_signature(program, *right)
                );
                if !tracker.allow(owner, key) {
                    return false;
                }
                if is_one_constant(literal_value(program, *right)) {
                    program.nodes[id.0 as usize] = program.nodes[left.0 as usize].clone();
                    return true;
                }
                false
            }
            ("*", [left, right]) => {
                let key = format!(
                    "identity:*:{}:{}",
                    node_signature(program, *left),
                    node_signature(program, *right)
                );
                if !tracker.allow(owner, key) {
                    return false;
                }
                if is_zero_constant(literal_value(program, *left))
                    || is_zero_constant(literal_value(program, *right))
                {
                    program.nodes[id.0 as usize] = LirNode::with_text(LirNodeKind::Literal, "0");
                    return true;
                }
                if is_one_constant(literal_value(program, *left)) {
                    program.nodes[id.0 as usize] = program.nodes[right.0 as usize].clone();
                    return true;
                }
                if is_one_constant(literal_value(program, *right)) {
                    program.nodes[id.0 as usize] = program.nodes[left.0 as usize].clone();
                    return true;
                }
                false
            }
            ("&&", [left, right]) => {
                let key = format!(
                    "identity:&&:{}:{}",
                    node_signature(program, *left),
                    node_signature(program, *right)
                );
                if !tracker.allow(owner, key) {
                    return false;
                }
                match literal_value(program, *left) {
                    Some(ConstantValue::Boolean(false)) => {
                        program.nodes[id.0 as usize] =
                            LirNode::with_text(LirNodeKind::Literal, "false");
                    }
                    Some(ConstantValue::Boolean(true)) => {
                        program.nodes[id.0 as usize] = program.nodes[right.0 as usize].clone();
                    }
                    _ => {}
                }

                match literal_value(program, *right) {
                    Some(ConstantValue::Boolean(false)) => {
                        program.nodes[id.0 as usize] =
                            LirNode::with_text(LirNodeKind::Literal, "false");
                        true
                    }
                    Some(ConstantValue::Boolean(true)) => {
                        program.nodes[id.0 as usize] = program.nodes[left.0 as usize].clone();
                        true
                    }
                    _ => false,
                }
            }
            ("||", [left, right]) => {
                let key = format!(
                    "identity:||:{}:{}",
                    node_signature(program, *left),
                    node_signature(program, *right)
                );
                if !tracker.allow(owner, key) {
                    return false;
                }
                match literal_value(program, *left) {
                    Some(ConstantValue::Boolean(true)) => {
                        program.nodes[id.0 as usize] =
                            LirNode::with_text(LirNodeKind::Literal, "true");
                    }
                    Some(ConstantValue::Boolean(false)) => {
                        program.nodes[id.0 as usize] = program.nodes[right.0 as usize].clone();
                    }
                    _ => {}
                }

                match literal_value(program, *right) {
                    Some(ConstantValue::Boolean(true)) => {
                        program.nodes[id.0 as usize] =
                            LirNode::with_text(LirNodeKind::Literal, "true");
                        true
                    }
                    Some(ConstantValue::Boolean(false)) => {
                        program.nodes[id.0 as usize] = program.nodes[left.0 as usize].clone();
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ConstantValue {
    Number(i64),
    BigInt(i64),
    Boolean(bool),
    String(String),
    RegExp { pattern: String, flags: String },
    Null,
    Undefined,
    NegativeZero,
    Infinity,
    NegativeInfinity,
    NaN,
}

impl ConstantValue {
    pub(crate) fn truthy(self) -> bool {
        match self {
            ConstantValue::Number(value) | ConstantValue::BigInt(value) => value != 0,
            ConstantValue::Boolean(value) => value,
            ConstantValue::String(value) => !value.is_empty(),
            ConstantValue::RegExp { .. } => true,
            ConstantValue::Null
            | ConstantValue::Undefined
            | ConstantValue::NegativeZero
            | ConstantValue::NaN => false,
            ConstantValue::Infinity | ConstantValue::NegativeInfinity => true,
        }
    }
}

pub(crate) fn is_zero_constant(value: Option<ConstantValue>) -> bool {
    matches!(
        value,
        Some(ConstantValue::Number(0) | ConstantValue::BigInt(0) | ConstantValue::NegativeZero)
    )
}

pub(crate) fn is_one_constant(value: Option<ConstantValue>) -> bool {
    matches!(
        value,
        Some(ConstantValue::Number(1) | ConstantValue::BigInt(1))
    )
}

pub(crate) fn literal_value(program: &LirProgram, id: LirNodeId) -> Option<ConstantValue> {
    let node = program.nodes.get(id.0 as usize)?;
    match node.kind {
        LirNodeKind::Literal => parse_literal_text(node.text.as_deref()),
        LirNodeKind::Value if node.children.is_empty() => parse_literal_text(node.text.as_deref()),
        _ => None,
    }
}

pub(crate) fn parse_literal_text(text: Option<&str>) -> Option<ConstantValue> {
    let text = text?;
    match text {
        "true" => Some(ConstantValue::Boolean(true)),
        "false" => Some(ConstantValue::Boolean(false)),
        "null" => Some(ConstantValue::Null),
        "undefined" => Some(ConstantValue::Undefined),
        "-0" => Some(ConstantValue::NegativeZero),
        "Infinity" => Some(ConstantValue::Infinity),
        "-Infinity" => Some(ConstantValue::NegativeInfinity),
        "NaN" => Some(ConstantValue::NaN),
        _ => parse_regex_literal(text)
            .map(|(pattern, flags)| ConstantValue::RegExp { pattern, flags })
            .or_else(|| parse_string_literal(text).map(ConstantValue::String))
            .or_else(|| {
                if let Some(stripped) = text.strip_suffix('n') {
                    stripped.parse::<i64>().ok().map(ConstantValue::BigInt)
                } else {
                    parse_number_literal(text).map(ConstantValue::Number)
                }
            }),
    }
}

pub(crate) fn fold_unary(op: &str, value: ConstantValue) -> Option<ConstantValue> {
    match (op, value) {
        ("-", ConstantValue::Number(0)) => Some(ConstantValue::NegativeZero),
        ("-", ConstantValue::NegativeZero) => Some(ConstantValue::Number(0)),
        ("-", ConstantValue::Number(value)) => value.checked_neg().map(ConstantValue::Number),
        ("-", ConstantValue::BigInt(value)) => value.checked_neg().map(ConstantValue::BigInt),
        ("-", ConstantValue::Infinity) => Some(ConstantValue::NegativeInfinity),
        ("-", ConstantValue::NegativeInfinity) => Some(ConstantValue::Infinity),
        ("-", ConstantValue::NaN) => Some(ConstantValue::NaN),
        ("!", value) => Some(ConstantValue::Boolean(!value.truthy())),
        _ => None,
    }
}

pub(crate) fn fold_binary(
    op: &str,
    left: ConstantValue,
    right: ConstantValue,
) -> Option<ConstantValue> {
    fn as_number(value: ConstantValue) -> Option<i64> {
        match value {
            ConstantValue::Number(value) => Some(value),
            ConstantValue::NegativeZero => Some(0),
            _ => None,
        }
    }

    match (op, left, right) {
        ("+", ConstantValue::BigInt(left), ConstantValue::BigInt(right)) => {
            left.checked_add(right).map(ConstantValue::BigInt)
        }
        ("+", ConstantValue::String(left), ConstantValue::String(right)) => {
            Some(ConstantValue::String(format!("{left}{right}")))
        }
        ("-", ConstantValue::BigInt(left), ConstantValue::BigInt(right)) => {
            left.checked_sub(right).map(ConstantValue::BigInt)
        }
        ("*", ConstantValue::BigInt(left), ConstantValue::BigInt(right)) => {
            left.checked_mul(right).map(ConstantValue::BigInt)
        }
        ("/", ConstantValue::BigInt(left), ConstantValue::BigInt(right)) => {
            if right == 0 {
                None
            } else {
                Some(ConstantValue::BigInt(left / right))
            }
        }
        ("%", ConstantValue::BigInt(left), ConstantValue::BigInt(right)) => {
            if right == 0 {
                None
            } else {
                Some(ConstantValue::BigInt(left % right))
            }
        }
        ("==", ConstantValue::BigInt(left), ConstantValue::BigInt(right)) => {
            Some(ConstantValue::Boolean(left == right))
        }
        ("+", left, right) => match (as_number(left), as_number(right)) {
            (Some(left), Some(right)) => left.checked_add(right).map(ConstantValue::Number),
            _ => None,
        },
        ("-", left, right) => match (as_number(left), as_number(right)) {
            (Some(left), Some(right)) => left.checked_sub(right).map(ConstantValue::Number),
            _ => None,
        },
        ("*", left, right) => match (as_number(left), as_number(right)) {
            (Some(left), Some(right)) => left.checked_mul(right).map(ConstantValue::Number),
            _ => None,
        },
        ("/", left, right) => match (as_number(left), as_number(right)) {
            (Some(left), Some(right)) => {
                if right == 0 {
                    None
                } else {
                    Some(ConstantValue::Number(left / right))
                }
            }
            _ => None,
        },
        ("%", left, right) => match (as_number(left), as_number(right)) {
            (Some(left), Some(right)) => {
                if right == 0 {
                    None
                } else {
                    Some(ConstantValue::Number(left % right))
                }
            }
            _ => None,
        },
        ("==", left, right) => match (left, right) {
            (ConstantValue::Number(left), ConstantValue::Number(right)) => {
                Some(ConstantValue::Boolean(left == right))
            }
            (ConstantValue::Number(left), ConstantValue::NegativeZero)
            | (ConstantValue::NegativeZero, ConstantValue::Number(left)) => {
                Some(ConstantValue::Boolean(left == 0))
            }
            (ConstantValue::NegativeZero, ConstantValue::NegativeZero) => {
                Some(ConstantValue::Boolean(true))
            }
            (ConstantValue::Boolean(left), ConstantValue::Boolean(right)) => {
                Some(ConstantValue::Boolean(left == right))
            }
            (ConstantValue::String(left), ConstantValue::String(right)) => {
                Some(ConstantValue::Boolean(left == right))
            }
            (ConstantValue::Null, ConstantValue::Null)
            | (ConstantValue::Undefined, ConstantValue::Undefined)
            | (ConstantValue::Null, ConstantValue::Undefined)
            | (ConstantValue::Undefined, ConstantValue::Null) => Some(ConstantValue::Boolean(true)),
            _ => None,
        },
        ("&&", left, right) => Some(ConstantValue::Boolean(left.truthy() && right.truthy())),
        ("||", left, right) => Some(ConstantValue::Boolean(left.truthy() || right.truthy())),
        _ => None,
    }
}

pub(crate) fn literal_text(value: ConstantValue) -> String {
    match value {
        ConstantValue::Number(value) => value.to_string(),
        ConstantValue::BigInt(value) => format!("{value}n"),
        ConstantValue::Boolean(value) => value.to_string(),
        ConstantValue::String(value) => {
            format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
        }
        ConstantValue::RegExp { pattern, flags } => format!("/{pattern}/{flags}"),
        ConstantValue::Null => "null".to_string(),
        ConstantValue::Undefined => "undefined".to_string(),
        ConstantValue::NegativeZero => "-0".to_string(),
        ConstantValue::Infinity => "Infinity".to_string(),
        ConstantValue::NegativeInfinity => "-Infinity".to_string(),
        ConstantValue::NaN => "NaN".to_string(),
    }
}

pub(crate) fn parse_number_literal(text: &str) -> Option<i64> {
    if let Some(stripped) = text.strip_suffix('n') {
        return stripped.parse::<i64>().ok();
    }
    text.parse::<i64>().ok()
}

pub(crate) fn parse_string_literal(text: &str) -> Option<String> {
    let (inner, is_template) = text
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .map(|inner| (inner, false))
        .or_else(|| {
            text.strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
                .map(|inner| (inner, false))
        })
        .or_else(|| {
            text.strip_prefix('`')
                .and_then(|value| value.strip_suffix('`'))
                .map(|inner| (inner, true))
        })?;
    let mut value = inner
        .replace("\\\\", "\\")
        .replace("\\\"", "\"")
        .replace("\\'", "'");
    if is_template {
        value = value.replace("\\`", "`");
    }
    Some(value)
}

pub(crate) fn parse_regex_literal(text: &str) -> Option<(String, String)> {
    if !text.starts_with('/') {
        return None;
    }

    let mut escaped = false;
    let mut closing = None;
    for (idx, ch) in text.char_indices().skip(1) {
        if escaped {
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '/' => {
                closing = Some(idx);
                break;
            }
            _ => {}
        }
    }

    let closing = closing?;
    if closing == 0 || closing + 1 > text.len() {
        return None;
    }

    let pattern = text[1..closing].to_string();
    let flags = text[closing + 1..].to_string();
    if !flags.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return None;
    }

    Some((pattern, flags))
}

#[cfg(test)]
#[path = "constant_fold_tests.rs"]
mod constant_fold_tests;
