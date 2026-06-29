use super::*;

#[test]
fn test_parse_export_async_function_declaration() {
    let tokens = lex("export async function main() { await value; }");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::FunctionDeclaration(decl) => {
            assert_eq!(decl.name, "main");
            assert!(decl.is_async);
        }
        other => panic!("Expected FunctionDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_async_await_expression() {
    let tokens = lex("async function main() { await Promise.resolve(7); }");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    let Statement::FunctionDeclaration(decl) = &output.statements[0] else {
        panic!(
            "Expected FunctionDeclaration, got {:?}",
            output.statements[0]
        );
    };
    assert!(decl.is_async, "expected async flag to be preserved");
    assert_eq!(decl.body.body.len(), 1);

    let Statement::ExpressionStatement(expr_stmt) = &decl.body.body[0] else {
        panic!("Expected ExpressionStatement, got {:?}", decl.body.body[0]);
    };
    let Expression::AwaitExpression(await_expr) = expr_stmt.expression.as_ref() else {
        panic!("Expected AwaitExpression, got {:?}", expr_stmt.expression);
    };
    assert!(matches!(await_expr.argument, Expression::CallExpression(_)));
}

#[test]
fn test_parse_function_declaration_stops_before_following_statement() {
    let tokens = lex("function add(a, b) { return a + b; } add(1, 2);");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 2);

    let Statement::FunctionDeclaration(decl) = &output.statements[0] else {
        panic!(
            "Expected FunctionDeclaration, got {:?}",
            output.statements[0]
        );
    };
    assert_eq!(decl.body.body.len(), 1);

    let Statement::ReturnStatement(return_stmt) = &decl.body.body[0] else {
        panic!("Expected ReturnStatement, got {:?}", decl.body.body[0]);
    };
    assert!(return_stmt.argument.is_some());

    let Statement::ExpressionStatement(expr_stmt) = &output.statements[1] else {
        panic!(
            "Expected ExpressionStatement, got {:?}",
            output.statements[1]
        );
    };
    assert!(matches!(
        expr_stmt.expression.as_ref(),
        Expression::CallExpression(_)
    ));
}

#[test]
fn test_parse_async_function_expression() {
    let tokens = lex("const make = async function() { return 1; };");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::VariableDeclaration(decl) => {
            let init = decl.declarations[0].init.as_ref().expect("initializer");
            match init {
                Expression::FunctionExpression(func) => {
                    assert!(func.is_async, "expected async flag to be preserved");
                    assert!(!func.generator, "expected generator flag to be false");
                    assert!(func.body.as_ref().is_some());
                }
                other => panic!("Expected FunctionExpression, got {other:?}"),
            }
        }
        other => panic!("Expected VariableDeclaration, got {other:?}"),
    }
}
