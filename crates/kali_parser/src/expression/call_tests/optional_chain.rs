use super::*;

#[test]
fn test_parse_optional_chain_member_expression() {
    let tokens = lex("minVersion(\"^1.2.3\")?.version;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    // `a?.b` preserves the accessed property as a `MemberExpression` whose
    // receiver is the short-circuit `OptionalChainExpression` marker. (The
    // property used to be DROPPED, collapsing `a?.b` to `a` — a silent
    // miscompile of optional member access.)
    match &output.statements[0] {
        Statement::ExpressionStatement(expr_stmt) => match expr_stmt.expression.as_ref() {
            Expression::MemberExpression(member) => {
                assert_eq!(member.property, "version");
                assert!(member.computed_index.is_none());
                assert!(
                    matches!(member.object, Expression::OptionalChainExpression(_)),
                    "expected optional-chain receiver, got {:?}",
                    member.object
                );
            }
            other => panic!("Expected MemberExpression, got {other:?}"),
        },
        other => panic!("Expected ExpressionStatement, got {other:?}"),
    }
}

#[test]
fn test_parse_optional_chain_index_expression() {
    let tokens = lex("call()?.[expr];");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    // `a?.[expr]` preserves the computed index as a `MemberExpression` whose
    // receiver is the short-circuit `OptionalChainExpression` marker.
    match &output.statements[0] {
        Statement::ExpressionStatement(expr_stmt) => match expr_stmt.expression.as_ref() {
            Expression::MemberExpression(member) => {
                assert!(member.computed_index.is_some());
                assert!(
                    matches!(member.object, Expression::OptionalChainExpression(_)),
                    "expected optional-chain receiver, got {:?}",
                    member.object
                );
            }
            other => panic!("Expected MemberExpression, got {other:?}"),
        },
        other => panic!("Expected ExpressionStatement, got {other:?}"),
    }
}
