use super::*;

#[test]
fn test_parse_type_assertion_expression() {
    let tokens = lex("value as Foo;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ExpressionStatement(expr_stmt) => match expr_stmt.expression.as_ref() {
            Expression::TypeAssertion(assertion) => {
                assert_eq!(assertion.type_name, "Foo");
                match assertion.expression.as_ref() {
                    Expression::Identifier(name) => assert_eq!(name, "value"),
                    other => panic!("Expected Identifier, got {other:?}"),
                }
            }
            other => panic!("Expected TypeAssertion, got {other:?}"),
        },
        other => panic!("Expected ExpressionStatement, got {other:?}"),
    }
}

#[test]
fn test_parse_satisfies_expression() {
    let tokens = lex("value satisfies Foo;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ExpressionStatement(expr_stmt) => match expr_stmt.expression.as_ref() {
            Expression::SatisfiesExpression(satisfies) => {
                assert_eq!(satisfies.type_name, "Foo");
                match satisfies.expression.as_ref() {
                    Expression::Identifier(name) => assert_eq!(name, "value"),
                    other => panic!("Expected Identifier, got {other:?}"),
                }
            }
            other => panic!("Expected SatisfiesExpression, got {other:?}"),
        },
        other => panic!("Expected ExpressionStatement, got {other:?}"),
    }
}
