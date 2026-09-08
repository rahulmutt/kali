//! String-literal normalization and property-name helpers.

use crate::Parser;
use kali_ast::{Expression, LiteralValue};
use kali_common::js_number::format_js_number;

/// Strip a string literal's delimiters. **It does NOT decode escape sequences,
/// and that is register entry R-57** (§2, Tier 2, filed 2026-09-08 at
/// `dde0f083c0`): the lexer deliberately keeps the raw escape in the token's
/// value (`crates/kali_lexer/src/string.rs:23-35`, so `kali_fmt` can re-emit it
/// verbatim), and this function only removes the outer quotes -- so a property
/// key spelled `"a\"b"` becomes the FOUR characters `a\"b`, where the property
/// name JavaScript denotes is the three characters `a"b`. Every downstream
/// comparison then compares the wrong text, and `Object.hasOwn(o, 'a"b')`
/// answers `false` for a property the object has. Measured at `dde0f083c0`
/// against node v26.8.1, both scopes; pinned by `r57a_*` in
/// `crates/kali_cli/tests/cases/oracle/tier2.toml` and by
/// `escaped_quote_in_a_key_is_stored_undecoded_*` in
/// `crates/kali_cli/tests/cases/object/property_key_identity.toml`.
///
/// A decoder exists -- `decode_string_escapes`
/// (`crates/kali_codegen/src/ctx.rs:160`), applied when a string is interned at
/// `ctx.rs:222` -- and it is downstream of the key path, which is why a string
/// VALUE renders correctly while a key does not. **Do not fix R-57 by
/// un-escaping at a comparison site**; see the entry's Fix direction, and note
/// that it also owes `object_fold.rs`'s two `__proto__` guards a move onto the
/// decoded name, which are sound today only because this function never decodes
/// (§4.1 of `docs/superpowers/followups/property-key-trim-site-classification.md`).
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
    /// `String(index)` for the shapes this phase can read statically --
    /// a statically-foldable literal, or a folded `+`/`-` unary applied to
    /// one. For every OTHER shape this does NOT decline: it falls back to a
    /// fabricated name that is not the property JavaScript would read.
    ///
    /// **That fabrication is register entry R-59** (§2, Tier 2, filed
    /// 2026-09-08 at `dde0f083c0`), and the entry is filed for the case the
    /// paragraphs below understate: when the fabricated name COLLIDES with a
    /// property the receiver really has, the read does not fall to a
    /// placeholder `0` -- it returns another property's VALUE, at exit 0, with
    /// no diagnostic. Measured against node v26.8.1 in both scopes; pinned by
    /// `r59a_*` in `crates/kali_cli/tests/cases/oracle/tier2.toml` and by
    /// `computed_member_index_is_fabricated_from_the_index_expression_*` in
    /// `crates/kali_cli/tests/cases/object/property_key_identity.toml`.
    ///
    /// An `Identifier` (`o[i]`) returns the identifier's own TEXT, not
    /// `String(i)`'s runtime value -- `const o = {index: 9, i: 7}; let i = 1;
    /// o[i]` reads the property literally named `i` (kali: `7`, node:
    /// `undefined`), because this function cannot evaluate a binding and
    /// falls to `Expression::Identifier(s) => s.clone()` instead of
    /// declining. Every other unreadable shape (a `SequenceExpression` with
    /// no last element, an unrecognized unary argument, or the catch-all
    /// `_` arm) fabricates the literal string `"index"` instead, and the
    /// catch-all is the easiest one to reach: an index spelled as any
    /// BINARY expression lands there, so in the same program `o[i + 0]`
    /// reads the property named `index` (kali: `9`, node: `undefined`).
    /// Both are measured, not hypothetical -- pinned in both scopes as
    /// `computed_member_index_is_fabricated_from_the_index_expression_*` in
    /// `crates/kali_cli/tests/cases/object/property_key_identity.toml`.
    ///
    /// This is the same fabricated-key class Task 5 deleted from the
    /// numeric-key arm one file over (`object.rs`'s un-quoting sites), and it
    /// undercuts this project's architectural claim that a probe and the key
    /// it probes are "one currency by construction": that is true only for
    /// the shapes this function actually reads statically, not for every
    /// shape it is called on.
    ///
    /// A NUMBER is rendered with `format_js_number`, the same function
    /// `kali_hir`'s `lower_property_name` stores a numeric KEY with, so a probe
    /// and the key it probes land on one spelling BY CONSTRUCTION rather than
    /// by two formatters happening to agree -- for the numeric shapes this
    /// function actually reads.
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
    /// comparison site: one formatter is what makes the agreement structural
    /// on the shapes it covers.
    ///
    /// RECOMMENDATION, not done here (a behaviour change with its own
    /// verification surface): return `Option<String>` so a caller can decline
    /// on the unreadable shapes instead of comparing against a fabricated
    /// name. That is the eventual fix; this comment only corrects what the
    /// function does today.
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
