use crate::test_support::lex;
use crate::*;
use kali_ast::{AssignmentOperator, Expression, Statement, UpdateOperator};

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

#[test]
fn test_parse_nullish_coalescing_expression() {
    let tokens = lex("const value = null ?? 1;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    let Statement::VariableDeclaration(vd) = &output.statements[0] else {
        panic!(
            "Expected VariableDeclaration, got {:?}",
            output.statements[0]
        );
    };
    let init = vd.declarations[0].init.as_ref().expect("initializer");
    let Expression::BinaryExpression(expr) = init else {
        panic!("Expected BinaryExpression, got {init:?}");
    };
    assert_eq!(expr.operator, "??");
}

#[test]
fn test_parse_exponentiation_expression() {
    let tokens = lex("const value = 2 ** 3 ** 2;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    let Statement::VariableDeclaration(vd) = &output.statements[0] else {
        panic!(
            "Expected VariableDeclaration, got {:?}",
            output.statements[0]
        );
    };
    let init = vd.declarations[0].init.as_ref().expect("initializer");
    let Expression::BinaryExpression(expr) = init else {
        panic!("Expected BinaryExpression, got {init:?}");
    };
    assert_eq!(expr.operator, "**");
    let Expression::BinaryExpression(right_expr) = expr.right.as_ref() else {
        panic!(
            "Expected nested BinaryExpression on the right, got {:?}",
            expr.right
        );
    };
    assert_eq!(right_expr.operator, "**");
}

#[test]
fn test_parse_modulo_expression() {
    let tokens = lex("const value = 3n % 2n;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );

    let Statement::VariableDeclaration(vd) = &output.statements[0] else {
        panic!(
            "Expected VariableDeclaration, got {:?}",
            output.statements[0]
        );
    };
    let init = vd.declarations[0].init.as_ref().expect("initializer");
    let Expression::BinaryExpression(expr) = init else {
        panic!("Expected BinaryExpression, got {init:?}");
    };
    assert_eq!(expr.operator, "%");
}

#[test]
fn test_parse_compound_assignment_expression() {
    let tokens = lex("value += 1; value **= 2; value %= 3; value &&= 4; value ||= 5;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 5);

    let Statement::ExpressionStatement(first) = &output.statements[0] else {
        panic!(
            "Expected ExpressionStatement, got {:?}",
            output.statements[0]
        );
    };
    let Expression::AssignmentExpression(first_assign) = first.expression.as_ref() else {
        panic!("Expected AssignmentExpression, got {:?}", first.expression);
    };
    assert!(matches!(
        first_assign.operator,
        AssignmentOperator::AddAssign
    ));

    let Statement::ExpressionStatement(second) = &output.statements[1] else {
        panic!(
            "Expected ExpressionStatement, got {:?}",
            output.statements[1]
        );
    };
    let Expression::AssignmentExpression(second_assign) = second.expression.as_ref() else {
        panic!("Expected AssignmentExpression, got {:?}", second.expression);
    };
    assert!(matches!(
        second_assign.operator,
        AssignmentOperator::ExponentAssign
    ));

    let Statement::ExpressionStatement(third) = &output.statements[2] else {
        panic!(
            "Expected ExpressionStatement, got {:?}",
            output.statements[2]
        );
    };
    let Expression::AssignmentExpression(third_assign) = third.expression.as_ref() else {
        panic!("Expected AssignmentExpression, got {:?}", third.expression);
    };
    assert!(matches!(
        third_assign.operator,
        AssignmentOperator::ModuloAssign
    ));

    let Statement::ExpressionStatement(fourth) = &output.statements[3] else {
        panic!(
            "Expected ExpressionStatement, got {:?}",
            output.statements[3]
        );
    };
    let Expression::AssignmentExpression(fourth_assign) = fourth.expression.as_ref() else {
        panic!("Expected AssignmentExpression, got {:?}", fourth.expression);
    };
    assert!(matches!(
        fourth_assign.operator,
        AssignmentOperator::AndAssign
    ));

    let Statement::ExpressionStatement(fifth) = &output.statements[4] else {
        panic!(
            "Expected ExpressionStatement, got {:?}",
            output.statements[4]
        );
    };
    let Expression::AssignmentExpression(fifth_assign) = fifth.expression.as_ref() else {
        panic!("Expected AssignmentExpression, got {:?}", fifth.expression);
    };
    assert!(matches!(
        fifth_assign.operator,
        AssignmentOperator::OrAssign
    ));
}

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
