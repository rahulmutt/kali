use super::{fold_const_initializer, fold_nameless_computed_index};
use kali_ast::{Expression, LiteralValue};

fn ident(name: &str) -> Expression {
    Expression::Identifier(name.to_string())
}

#[test]
fn a_string_or_number_literal_initializer_folds_to_the_property_name() {
    assert_eq!(
        fold_const_initializer(&Expression::Literal(LiteralValue::String(
            "\"b\"".to_string()
        )))
        .as_deref(),
        Some("b")
    );
    assert_eq!(
        fold_const_initializer(&Expression::Literal(LiteralValue::Number(1.0))).as_deref(),
        Some("1")
    );
    assert_eq!(
        fold_const_initializer(&Expression::Literal(LiteralValue::Number(1e21))).as_deref(),
        Some("1e+21")
    );
}

#[test]
fn any_other_initializer_does_not_fold() {
    assert_eq!(
        fold_const_initializer(&Expression::Literal(LiteralValue::Boolean(true))),
        None
    );
    assert_eq!(
        fold_const_initializer(&Expression::Literal(LiteralValue::Null)),
        None
    );
    assert_eq!(fold_const_initializer(&ident("other")), None);
}

#[test]
fn only_a_bare_identifier_with_a_recorded_name_folds() {
    let lookup = |name: &str| (name == "k").then(|| "b".to_string());
    assert_eq!(
        fold_nameless_computed_index(&ident("k"), lookup).as_deref(),
        Some("b")
    );
    assert_eq!(fold_nameless_computed_index(&ident("j"), lookup), None);
    let parenthesized =
        Expression::ParenthesizedExpression(Box::new(kali_ast::ParenthesizedExpression {
            expression: Box::new(ident("k")),
        }));
    assert_eq!(
        fold_nameless_computed_index(&parenthesized, lookup),
        None,
        "a parenthesized identifier is not the admitted shape"
    );
}
