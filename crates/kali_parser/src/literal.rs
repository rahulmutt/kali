//! String-literal normalization and property-name helpers.

use crate::Parser;
use kali_ast::{Expression, LiteralValue};
use kali_common::js_number::format_js_number;

/// Strip a string literal's delimiters. **It does NOT decode escape sequences,
/// and that is register entry R-57** (§2, Tier 2, filed 2026-09-08 at
/// `b13c890330`, off `dde0f083c0`): the lexer deliberately keeps the raw escape in the token's
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
    /// The property name a computed member INDEX denotes, when this parser can
    /// read it: a string literal (delimiters stripped — escape sequences are
    /// NOT decoded, which is register entry R-57), a number literal rendered
    /// by `format_js_number`, and the parenthesized, sequence-last and `+`/`-`
    /// unary forms that recurse into one of those. The `+`/`-` arm reads a
    /// NUMBER-literal source only (see `unary_number_value`); `o[+"inf"]` and
    /// every other string spelling decline.
    ///
    /// `None` for every other shape — a bare identifier, a binary expression,
    /// a boolean/`null`/BigInt/regex literal, an empty sequence, a unary whose
    /// argument is not a number literal under those layers. There is NO
    /// fallback string: this function used to fabricate one (the identifier's
    /// own text, or the literal `"index"`), and the static lanes downstream
    /// read it as a real property name — register entry R-59, closed by the
    /// computed-member-static-name project
    /// (`docs/superpowers/specs/2026-09-08-computed-member-static-name-design.md`).
    /// A `None` here is the whole of what the caller records; the structured
    /// index child is then the only description of the access, and the
    /// checker and codegen either fold it (a bare `const`-with-literal
    /// identifier) or refuse it (E5506).
    ///
    /// A NUMBER is rendered with `format_js_number`, the same function
    /// `kali_hir`'s `lower_property_name` stores a numeric KEY with, so a probe
    /// and the key it probes land on one spelling BY CONSTRUCTION. Do not
    /// reintroduce a second number formatter here, and do not translate
    /// between them at the comparison site.
    pub(crate) fn expression_to_property_name(expr: &Expression) -> Option<String> {
        match expr {
            Expression::ParenthesizedExpression(parenthesized) => {
                Self::expression_to_property_name(&parenthesized.expression)
            }
            Expression::SequenceExpression(sequence) => {
                Self::expression_to_property_name(sequence.expressions.last()?)
            }
            Expression::UnaryExpression(unary)
                if unary.operator == "+" || unary.operator == "-" =>
            {
                // `format_js_number` renders both zeros as "0", so the signed
                // zero a `-0` index folds to needs no separate branch.
                Some(format_js_number(Self::unary_number_value(expr)?))
            }
            Expression::Literal(LiteralValue::String(s)) => Some(Self::normalize_string_literal(s)),
            Expression::Literal(LiteralValue::Number(n)) => Some(format_js_number(*n)),
            _ => None,
        }
    }

    /// The `f64` a `+`/`-` unary index denotes, when its source is a NUMBER
    /// literal — directly, or through the parenthesized, sequence-last and
    /// nested unary layers the readable set already admits.
    ///
    /// It exists so the unary arm never re-parses a RENDERED NAME. That is how
    /// the arm used to work: it called `expression_to_property_name`
    /// recursively and fed the resulting `String` to `str::parse::<f64>()`.
    /// Rust's float parser accepts `inf`, `infinity` and `nan`
    /// case-insensitively; JavaScript's `ToNumber` returns `NaN` for all three.
    /// So `o[+"inf"]` folded to the name `Infinity` and read a real, wrong
    /// property at exit 0 — register entry R-59's exact shape, measured at
    /// `ff8567e7f4` against node v26.8.1: on `const o = {Infinity: 9, NaN: 7}`,
    /// `o[+"inf"]` and `o[+"infinity"]` printed 9 where node prints 7.
    ///
    /// Carrying the NUMBER rather than its rendering also keeps the readable
    /// set from shrinking anywhere else: `+1`, `-1`, `+1e21`, `+(1)`, `-(-1)`,
    /// `+(0, 1)` and `-0` all still read their names, and `+1e400` still reads
    /// `"Infinity"` because that IS its JavaScript value — which is why the
    /// correction is a narrowing of the SOURCE and not an `is_finite` guard on
    /// the result.
    ///
    /// A string spelling now returns `None`, the access keeps no name, and the
    /// shared E5506 gate refuses it in both twins.
    fn unary_number_value(expr: &Expression) -> Option<f64> {
        match expr {
            Expression::ParenthesizedExpression(parenthesized) => {
                Self::unary_number_value(&parenthesized.expression)
            }
            Expression::SequenceExpression(sequence) => {
                Self::unary_number_value(sequence.expressions.last()?)
            }
            Expression::UnaryExpression(unary)
                if unary.operator == "+" || unary.operator == "-" =>
            {
                let value = Self::unary_number_value(&unary.argument)?;
                Some(if unary.operator == "+" { value } else { -value })
            }
            Expression::Literal(LiteralValue::Number(n)) => Some(*n),
            _ => None,
        }
    }

    pub fn normalize_string_literal(value: &str) -> String {
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

#[cfg(test)]
#[path = "literal_tests.rs"]
mod literal_tests;
