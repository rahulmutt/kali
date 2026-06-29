use super::*;

#[test]
fn test_parse_prefix_update_expression() {
    let tokens = lex("++value;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    let Statement::ExpressionStatement(expr_stmt) = &output.statements[0] else {
        panic!("Expected ExpressionStatement");
    };
    let Expression::UpdateExpression(update) = expr_stmt.expression.as_ref() else {
        panic!("Expected UpdateExpression");
    };
    assert!(update.prefix);
    assert!(matches!(update.operator, UpdateOperator::Increment));
    assert!(matches!(update.argument, Expression::Identifier(_)));
}

#[test]
fn test_parse_void_unary_expression() {
    let tokens = lex("void 0;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    let Statement::ExpressionStatement(expr_stmt) = &output.statements[0] else {
        panic!("Expected ExpressionStatement");
    };
    let Expression::UnaryExpression(unary) = expr_stmt.expression.as_ref() else {
        panic!("Expected UnaryExpression");
    };
    assert_eq!(unary.operator, "void");
    assert!(matches!(unary.argument, Expression::Literal(_)));
}

#[test]
fn test_parse_bitwise_not_unary_expression() {
    let tokens = lex("~value;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    let Statement::ExpressionStatement(expr_stmt) = &output.statements[0] else {
        panic!("Expected ExpressionStatement");
    };
    let Expression::UnaryExpression(unary) = expr_stmt.expression.as_ref() else {
        panic!("Expected UnaryExpression, got {:?}", expr_stmt.expression);
    };
    assert_eq!(unary.operator, "~");
    assert!(matches!(unary.argument, Expression::Identifier(_)));
}

#[test]
fn test_parse_postfix_update_expression() {
    let tokens = lex("value--;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    let Statement::ExpressionStatement(expr_stmt) = &output.statements[0] else {
        panic!("Expected ExpressionStatement");
    };
    let Expression::UpdateExpression(update) = expr_stmt.expression.as_ref() else {
        panic!("Expected UpdateExpression");
    };
    assert!(!update.prefix);
    assert!(matches!(update.operator, UpdateOperator::Decrement));
    assert!(matches!(update.argument, Expression::Identifier(_)));
}
