//! What kind of value a default is (default-parameters spec §3.2 step 1, A-7).

use kali_ast::{Expression, LiteralValue};

#[cfg(test)]
#[path = "literal_tests.rs"]
mod literal_tests;

/// The three kinds of default the pass distinguishes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DefaultKind {
    /// A number, string, boolean, `null` or BigInt literal, or unary `-`/`+`
    /// on a number. It has no side effects and reads no binding, so
    /// evaluating a clone of it at the call site is not observable (spec
    /// §3.2, "Evaluation order").
    Scalar,
    /// An object or array literal. Refused (spec A-7): kali refuses an object
    /// or array literal passed directly as a call argument, which is what the
    /// fill would produce.
    Composite,
    /// Anything else.
    Other,
}

pub(crate) fn classify_default(expression: &Expression) -> DefaultKind {
    match expression {
        Expression::Literal(LiteralValue::Regex { .. }) => DefaultKind::Other,
        Expression::Literal(_) | Expression::BigIntLiteral(_) => DefaultKind::Scalar,
        Expression::UnaryExpression(unary)
            if (unary.operator == "-" || unary.operator == "+")
                && matches!(unary.argument, Expression::Literal(LiteralValue::Number(_))) =>
        {
            DefaultKind::Scalar
        }
        Expression::ArrayExpression(_) | Expression::ObjectExpression(_) => DefaultKind::Composite,
        _ => DefaultKind::Other,
    }
}
