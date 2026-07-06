use super::*;

#[test]
fn test_parse_conditional_expression() {
    let tokens = lex("let x = a ? 1 : 2;");
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
    let Expression::ConditionalExpression(cond) = init else {
        panic!("expected ConditionalExpression, got {init:?}");
    };
    assert!(matches!(*cond.test, Expression::Identifier(ref n) if n == "a"));
    assert!(matches!(*cond.consequent, Expression::Literal(_)));
    assert!(matches!(*cond.alternate, Expression::Literal(_)));
}

#[test]
fn test_conditional_is_right_associative() {
    let tokens = lex("let x = a ? 1 : b ? 2 : 3;");
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
    let Expression::ConditionalExpression(outer) = init else {
        panic!("expected outer conditional, got {init:?}");
    };
    assert!(
        matches!(*outer.alternate, Expression::ConditionalExpression(_)),
        "alternate must nest the second conditional"
    );
}

#[test]
fn test_conditional_nests_in_consequent() {
    // Mirror of `test_conditional_is_right_associative` for the CONSEQUENT
    // position: `a ? b ? 10 : 11 : 12` parses the inner ternary as the
    // consequent of the outer one. (e2e with a=1,b=0 prints 11.)
    let tokens = lex("let x = a ? b ? 10 : 11 : 12;");
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
    let Expression::ConditionalExpression(outer) = init else {
        panic!("expected outer conditional, got {init:?}");
    };
    assert!(
        matches!(*outer.consequent, Expression::ConditionalExpression(_)),
        "consequent must nest the inner conditional"
    );
    assert!(
        matches!(*outer.alternate, Expression::Literal(_)),
        "outer alternate is the trailing literal"
    );
}

#[test]
fn test_conditional_nests_inside_assignment_rhs() {
    // `x = a ? 1 : 2` — the ternary binds tighter than `=`.
    let tokens = lex("x = a ? 1 : 2;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    let Statement::ExpressionStatement(expr_stmt) = &output.statements[0] else {
        panic!(
            "Expected ExpressionStatement, got {:?}",
            output.statements[0]
        );
    };
    let Expression::AssignmentExpression(assign) = expr_stmt.expression.as_ref() else {
        panic!(
            "Expected AssignmentExpression, got {:?}",
            expr_stmt.expression
        );
    };
    assert!(matches!(assign.right, Expression::ConditionalExpression(_)));
}

#[test]
fn test_optional_chain_is_not_a_conditional() {
    // `a?.b` lexes QuestionDot — must stay a chain, not a ternary.
    let tokens = lex("a?.b;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert_eq!(output.statements.len(), 1);

    let Statement::ExpressionStatement(expr_stmt) = &output.statements[0] else {
        panic!(
            "Expected ExpressionStatement, got {:?}",
            output.statements[0]
        );
    };
    assert!(!matches!(
        expr_stmt.expression.as_ref(),
        Expression::ConditionalExpression(_)
    ));
}
