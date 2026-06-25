//! String-literal normalization and property-name helpers.

use crate::Parser;
use kali_ast::{Expression, LiteralValue};

pub(crate) fn unquote_string_literal(value: &str) -> String {
    let trimmed = value.trim();
    let Some(first) = trimmed.chars().next() else {
        return trimmed.to_string();
    };
    let Some(last) = trimmed.chars().last() else {
        return trimmed.to_string();
    };

    if (first == '"' && last == '"')
        || (first == '\'' && last == '\'')
        || (first == '`' && last == '`')
    {
        trimmed[1..trimmed.len().saturating_sub(1)].to_string()
    } else {
        trimmed.to_string()
    }
}

impl Parser {
    pub(crate) fn expression_to_property_name(expr: &Expression) -> String {
        match expr {
            Expression::ParenthesizedExpression(parenthesized) => {
                Self::expression_to_property_name(&parenthesized.expression)
            }
            Expression::SequenceExpression(sequence) => sequence
                .expressions
                .last()
                .map(Self::expression_to_property_name)
                .unwrap_or_else(|| "index".to_string()),
            Expression::UnaryExpression(unary)
                if unary.operator == "+" || unary.operator == "-" =>
            {
                let inner = Self::expression_to_property_name(&unary.argument);
                let Some(value) = inner.parse::<f64>().ok() else {
                    return "index".to_string();
                };
                let value = if unary.operator == "+" { value } else { -value };
                if value == 0.0 {
                    "0".to_string()
                } else if value.fract() == 0.0 {
                    format!("{value:.0}")
                } else {
                    value.to_string()
                }
            }
            Expression::Identifier(s) => s.clone(),
            Expression::Literal(LiteralValue::String(s)) => Self::normalize_string_literal(s),
            Expression::Literal(LiteralValue::Number(n)) if n.fract() == 0.0 => {
                if *n == 0.0 {
                    "0".to_string()
                } else {
                    format!("{n:.0}")
                }
            }
            Expression::Literal(LiteralValue::Number(n)) => n.to_string(),
            _ => "index".to_string(),
        }
    }

    pub(crate) fn normalize_string_literal(value: &str) -> String {
        let Some(first) = value.chars().next() else {
            return value.to_string();
        };
        let Some(last) = value.chars().last() else {
            return value.to_string();
        };

        if value.len() >= 2 && matches!((first, last), ('"', '"') | ('\'', '\'') | ('`', '`')) {
            value[1..value.len() - 1].to_string()
        } else {
            value.to_string()
        }
    }
}
