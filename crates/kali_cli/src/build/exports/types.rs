//! Static type inference for export/signature collection.

use kali_ast::{BlockStatement, Expression, LiteralValue, OptionalChainInner, Statement};

pub(crate) fn infer_block_return_type(body: &BlockStatement) -> Option<&'static str> {
    if body.body.len() != 1 {
        return None;
    }

    let Statement::ReturnStatement(return_statement) = &body.body[0] else {
        return None;
    };

    match &return_statement.argument {
        Some(expression) => infer_expression_type(expression),
        None => Some("void"),
    }
}

pub(crate) fn infer_static_truthiness(expression: &Expression) -> Option<bool> {
    match expression {
        Expression::Literal(kali_ast::LiteralValue::Boolean(value)) => Some(*value),
        Expression::Literal(kali_ast::LiteralValue::Number(value)) => {
            Some(*value != 0.0 && !value.is_nan())
        }
        Expression::Literal(kali_ast::LiteralValue::String(value)) => Some(!value.is_empty()),
        Expression::Literal(kali_ast::LiteralValue::Regex { .. }) => Some(true),
        Expression::Literal(kali_ast::LiteralValue::Null) => Some(false),
        Expression::Identifier(name) if name == "undefined" => Some(false),
        Expression::UnaryExpression(unary) if unary.operator == "void" => Some(false),
        Expression::UnaryExpression(unary) if unary.operator == "!" => {
            infer_static_truthiness(&unary.argument).map(|value| !value)
        }
        Expression::ParenthesizedExpression(parenthesized) => {
            infer_static_truthiness(&parenthesized.expression)
        }
        Expression::AwaitExpression(await_expression) => {
            infer_static_truthiness(&await_expression.argument)
        }
        Expression::TypeAssertion(type_assertion) => {
            infer_static_truthiness(&type_assertion.expression)
        }
        Expression::SatisfiesExpression(satisfies_expression) => {
            infer_static_truthiness(&satisfies_expression.expression)
        }
        Expression::OptionalChainExpression(optional_chain) => {
            match optional_chain.inner.as_ref() {
                OptionalChainInner::NonNull { object, .. } => infer_static_truthiness(object),
            }
        }
        Expression::ChainExpression(chain_expression) => {
            infer_static_truthiness(&chain_expression.expression)
        }
        Expression::SequenceExpression(sequence) => sequence
            .expressions
            .last()
            .and_then(infer_static_truthiness),
        Expression::DecoratedExpression(decorated_expression) => {
            infer_static_truthiness(&decorated_expression.expression)
        }
        _ => None,
    }
}

pub(crate) fn infer_expression_type(expression: &Expression) -> Option<&'static str> {
    match expression {
        Expression::Literal(value) => infer_literal_type(value),
        Expression::Identifier(name) if name == "undefined" => Some("undefined"),
        Expression::BigIntLiteral(_) => Some("bigint"),
        Expression::TemplateLiteral(_) => Some("string"),
        Expression::ParenthesizedExpression(parenthesized) => {
            infer_expression_type(&parenthesized.expression)
        }
        Expression::AwaitExpression(await_expression) => {
            infer_expression_type(&await_expression.argument)
        }
        Expression::TypeAssertion(type_assertion) => {
            infer_expression_type(&type_assertion.expression)
        }
        Expression::SatisfiesExpression(satisfies_expression) => {
            infer_expression_type(&satisfies_expression.expression)
        }
        Expression::OptionalChainExpression(optional_chain) => {
            match optional_chain.inner.as_ref() {
                OptionalChainInner::NonNull { object, .. } => infer_expression_type(object),
            }
        }
        Expression::ChainExpression(chain_expression) => {
            infer_expression_type(&chain_expression.expression)
        }
        Expression::SequenceExpression(sequence) => {
            sequence.expressions.last().and_then(infer_expression_type)
        }
        Expression::DecoratedExpression(decorated_expression) => {
            infer_expression_type(&decorated_expression.expression)
        }
        Expression::ConditionalExpression(condition) => {
            let consequent = infer_expression_type(&condition.consequent);
            let alternate = infer_expression_type(&condition.alternate);
            if consequent.is_some() && consequent == alternate {
                consequent
            } else {
                None
            }
        }
        Expression::UnaryExpression(unary) => infer_unary_expression_type(unary),
        Expression::BinaryExpression(binary) => infer_binary_expression_type(binary),
        _ => None,
    }
}

fn infer_unary_expression_type(unary: &kali_ast::UnaryExpression) -> Option<&'static str> {
    match unary.operator.as_str() {
        "!" => Some("boolean"),
        "+" | "-" => infer_expression_type(&unary.argument)
            .filter(|type_name| matches!(*type_name, "number" | "bigint")),
        "void" => Some("void"),
        _ => None,
    }
}

fn infer_binary_expression_type(binary: &kali_ast::BinaryExpression) -> Option<&'static str> {
    let left = infer_expression_type(&binary.left);
    let right = infer_expression_type(&binary.right);
    match binary.operator.as_str() {
        "+" => {
            if left == Some("string") || right == Some("string") {
                Some("string")
            } else if is_numeric_like_type(left) && is_numeric_like_type(right) {
                Some("number")
            } else {
                None
            }
        }
        "-" | "*" | "/" | "%" | "**" | "<<" | ">>" | ">>>" | "&" | "|" | "^" => {
            if is_numeric_like_type(left) && is_numeric_like_type(right) {
                Some("number")
            } else {
                None
            }
        }
        "??" => {
            if matches!(left, Some("null" | "undefined" | "void")) {
                right
            } else if left.is_some() {
                left
            } else {
                None
            }
        }
        "&&" | "||" => {
            if left.is_some() && left == right {
                left
            } else {
                None
            }
        }
        _ => None,
    }
}

fn is_numeric_like_type(type_name: Option<&str>) -> bool {
    matches!(type_name, Some("number" | "bigint"))
}

fn infer_literal_type(value: &LiteralValue) -> Option<&'static str> {
    match value {
        LiteralValue::Boolean(_) => Some("boolean"),
        LiteralValue::Number(_) => Some("number"),
        LiteralValue::String(_) => Some("string"),
        LiteralValue::Regex { .. } => Some("RegExp"),
        LiteralValue::Null => Some("null"),
    }
}
