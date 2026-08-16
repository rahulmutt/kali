//! String-literal normalization and property-name helpers.

use crate::Parser;
use kali_ast::{Expression, LiteralValue};
use kali_common::js_number::format_js_number;

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
    /// The JAVASCRIPT PROPERTY NAME a computed member-access index denotes --
    /// `String(index)` for the shapes this phase can read statically.
    ///
    /// A NUMBER is rendered with `format_js_number`, the same function
    /// `kali_hir`'s `lower_property_name` stores a numeric KEY with, so a probe
    /// and the key it probes land on one spelling BY CONSTRUCTION rather than
    /// by two formatters happening to agree.
    ///
    /// They stopped agreeing once. This function used to spell a number with
    /// `format!("{n:.0}")` / `Display`, which matches Rust and not JavaScript
    /// above 1e21, below 1e-6, and at the infinities. While HIR stored a key
    /// the same way the two sides matched by accident; when HIR moved to the
    /// JavaScript spelling and this one did not, `{1e21: 1}` stored the key
    /// `1e+21` while `o[1e21]` probed for `1000000000000000000000`, the
    /// object-literal field lookup missed, and the member read emitted a
    /// fabricated `0` at exit 0 -- in a program whose `Object.hasOwn` and
    /// `Object.keys` were both already right. Do not reintroduce a second
    /// number formatter here, and do not translate between them at the
    /// comparison site: one formatter is what makes the agreement structural.
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
                // `format_js_number` renders both zeros as "0", so the signed
                // zero a `-0` index folds to needs no separate branch.
                format_js_number(value)
            }
            Expression::Identifier(s) => s.clone(),
            Expression::Literal(LiteralValue::String(s)) => Self::normalize_string_literal(s),
            Expression::Literal(LiteralValue::Number(n)) => format_js_number(*n),
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
