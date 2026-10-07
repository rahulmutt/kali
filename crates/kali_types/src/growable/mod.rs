//! Growable runtime arrays: the fact walk (`facts`), the component solve (`flow`)
//! and the position refusals (`positions`), plus the object-field shape helper
//! [`array_literal_of_scalar_seeds`].

use kali_ast::{Expression, ExpressionOrSpread, LiteralValue};

pub(crate) mod elem_proof;
pub(crate) mod facts;
pub(crate) mod flow;
pub(crate) mod positions;

/// True for the scalar-shaped expressions admitted as push arguments, array
/// seeds, and computed indices in Task 2's i64 lane: numeric literals, bare
/// identifiers (`allow_identifiers` — repr-checked later at `emit_table`),
/// and unary/binary arithmetic over such. Everything else (calls, members,
/// objects, arrays, strings, booleans, templates, …) is out — it could
/// deliver a value this lane would store raw and read back wrong.
fn scalar_value_shape_ok(expr: &Expression, allow_identifiers: bool) -> bool {
    match expr {
        Expression::Literal(LiteralValue::Number(_)) => true,
        Expression::Identifier(_) => allow_identifiers,
        Expression::ParenthesizedExpression(inner) => {
            scalar_value_shape_ok(&inner.expression, allow_identifiers)
        }
        Expression::UnaryExpression(unary) => {
            matches!(unary.operator.as_str(), "-" | "+" | "~")
                && scalar_value_shape_ok(&unary.argument, allow_identifiers)
        }
        Expression::BinaryExpression(binary) => {
            matches!(
                binary.operator.as_str(),
                "+" | "-" | "*" | "/" | "%" | "**" | "&" | "|" | "^" | "<<" | ">>" | ">>>"
            ) && scalar_value_shape_ok(&binary.left, allow_identifiers)
                && scalar_value_shape_ok(&binary.right, allow_identifiers)
        }
        _ => false,
    }
}

/// The growable-i64 array SHAPE predicate: true when `expr` is an array
/// LITERAL every element of which is a scalar-shaped seed (numeric literal or
/// arithmetic over literals — NO identifiers, whose value the emit-time repr
/// gate cannot see on the element axis, and no strings/objects/arrays). This
/// is the exact declarator-init check `variable_declaration` applies to bare
/// bindings; `kali_types::repr_infer` reuses it to classify object-literal
/// array FIELDS (Stage P2 Lane 1 Task 3) off the SAME provenance — an
/// array-shaped field that fails this predicate fails closed (a shape
/// conflict) rather than silently interning a scalar field repr.
pub(crate) fn array_literal_of_scalar_seeds(expr: &Expression) -> bool {
    matches!(expr, Expression::ArrayExpression(array)
    if array.elements.iter().all(|element| matches!(
        element,
        Some(ExpressionOrSpread::Expression(inner))
            if scalar_value_shape_ok(inner, false)
    )))
}
