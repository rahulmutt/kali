use super::*;

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
