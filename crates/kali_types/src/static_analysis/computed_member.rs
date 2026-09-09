//! The one fold rule for a computed member index the parser could not name.
//!
//! Three passes must agree on what folds — the resolver (this crate), the
//! materialization pass (`repr_infer`, this crate) and codegen — so the rule
//! is deliberately narrow and defined once: a BARE identifier naming a
//! `const` whose initializer is a string or number LITERAL. The two checker
//! passes call these functions with their own lookup; codegen mirrors the rule
//! on LIR through its `const`-only `bindings` map.
//!
//! Spec: docs/superpowers/specs/2026-09-08-computed-member-static-name-design.md §4.4.

use kali_ast::{Expression, LiteralValue};
use kali_common::js_number::format_js_number;
use kali_parser::Parser;

/// The property name a `const` declarator's initializer denotes when it is a
/// string or number literal; `None` for every other initializer.
pub(crate) fn fold_const_initializer(init: &Expression) -> Option<String> {
    match init {
        Expression::Literal(LiteralValue::String(raw)) => {
            Some(Parser::normalize_string_literal(raw))
        }
        Expression::Literal(LiteralValue::Number(value)) => Some(format_js_number(*value)),
        _ => None,
    }
}

/// The property name a nameless computed index denotes: only a bare identifier
/// whose recorded `const` literal name `lookup` returns. Parenthesized, unary
/// and sequence spellings of the identifier are declined on purpose — the
/// parser already reads those forms when they wrap a literal, and widening
/// them here would have to be mirrored in three places.
pub(crate) fn fold_nameless_computed_index(
    index: &Expression,
    lookup: impl Fn(&str) -> Option<String>,
) -> Option<String> {
    let Expression::Identifier(name) = index else {
        return None;
    };
    lookup(name)
}

#[cfg(test)]
#[path = "computed_member_tests.rs"]
mod computed_member_tests;
