use super::*;

#[test]
fn test_parse_generator_function_declaration() {
    let tokens = lex("function* main() { yield 1; }");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::FunctionDeclaration(decl) => {
            assert!(!decl.is_async, "expected async flag to be false");
            assert!(decl.generator, "expected generator flag to be preserved");
            assert_eq!(decl.name, "main");
            assert_eq!(decl.params.len(), 0);
            assert_eq!(decl.body.body.len(), 1);
            match &decl.body.body[0] {
                Statement::ExpressionStatement(expr_stmt) => match expr_stmt.expression.as_ref() {
                    Expression::YieldExpression(yield_expr) => {
                        assert!(!yield_expr.delegate);
                        let argument = yield_expr.argument.as_ref().expect("yield argument");
                        match argument {
                            Expression::Literal(kali_ast::LiteralValue::Number(value)) => {
                                assert_eq!(*value, 1.0)
                            }
                            other => panic!("unexpected yield argument: {other:?}"),
                        }
                    }
                    other => panic!("Expected YieldExpression, got {other:?}"),
                },
                other => panic!("Expected ExpressionStatement, got {other:?}"),
            }
        }
        other => panic!("Expected FunctionDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_generator_delegating_yield_expression() {
    let tokens = lex("function* main() { yield* other(); }");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::FunctionDeclaration(decl) => {
            assert!(decl.generator, "expected generator flag to be preserved");
            assert_eq!(decl.body.body.len(), 1);
            match &decl.body.body[0] {
                Statement::ExpressionStatement(expr_stmt) => match expr_stmt.expression.as_ref() {
                    Expression::YieldExpression(yield_expr) => {
                        assert!(
                            yield_expr.delegate,
                            "expected yield* delegation to be preserved"
                        );
                        let argument = yield_expr.argument.as_ref().expect("yield argument");
                        match argument {
                            Expression::CallExpression(call_expr) => {
                                assert_eq!(call_expr.args.len(), 0)
                            }
                            other => panic!("unexpected yield* argument: {other:?}"),
                        }
                    }
                    other => panic!("Expected YieldExpression, got {other:?}"),
                },
                other => panic!("Expected ExpressionStatement, got {other:?}"),
            }
        }
        other => panic!("Expected FunctionDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_generator_function_expression() {
    let tokens = lex("const make = function*() { yield 1; };");
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
                    assert!(!func.is_async, "expected async flag to be false");
                    assert!(func.generator, "expected generator flag to be preserved");
                    assert!(func.body.as_ref().is_some());
                }
                other => panic!("Expected FunctionExpression, got {other:?}"),
            }
        }
        other => panic!("Expected VariableDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_async_generator_function_declaration() {
    let tokens = lex("async function* main() { yield 1; }");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::FunctionDeclaration(decl) => {
            assert!(decl.is_async, "expected async flag to be preserved");
            assert!(decl.generator, "expected generator flag to be preserved");
            assert_eq!(decl.name, "main");
            assert_eq!(decl.params.len(), 0);
            assert_eq!(decl.body.body.len(), 1);
            match &decl.body.body[0] {
                Statement::ExpressionStatement(expr_stmt) => match expr_stmt.expression.as_ref() {
                    Expression::YieldExpression(yield_expr) => {
                        assert!(!yield_expr.delegate);
                        let argument = yield_expr.argument.as_ref().expect("yield argument");
                        match argument {
                            Expression::Literal(kali_ast::LiteralValue::Number(value)) => {
                                assert_eq!(*value, 1.0)
                            }
                            other => panic!("unexpected yield argument: {other:?}"),
                        }
                    }
                    other => panic!("Expected YieldExpression, got {other:?}"),
                },
                other => panic!("Expected ExpressionStatement, got {other:?}"),
            }
        }
        other => panic!("Expected FunctionDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_async_generator_function_expression() {
    let tokens = lex("const make = async function*() { yield* other(); };");
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
                    assert!(func.generator, "expected generator flag to be preserved");
                    assert!(func.body.as_ref().is_some());
                    let body = func.body.as_ref().expect("function body");
                    assert_eq!(body.body.len(), 1);
                    match &body.body[0] {
                        Statement::ExpressionStatement(expr_stmt) => {
                            match expr_stmt.expression.as_ref() {
                                Expression::YieldExpression(yield_expr) => {
                                    assert!(
                                        yield_expr.delegate,
                                        "expected yield* delegation to be preserved"
                                    );
                                    let argument =
                                        yield_expr.argument.as_ref().expect("yield argument");
                                    match argument {
                                        Expression::CallExpression(call_expr) => {
                                            assert_eq!(call_expr.args.len(), 0)
                                        }
                                        other => panic!("unexpected yield* argument: {other:?}"),
                                    }
                                }
                                other => panic!("Expected YieldExpression, got {other:?}"),
                            }
                        }
                        other => panic!("Expected ExpressionStatement, got {other:?}"),
                    }
                }
                other => panic!("Expected FunctionExpression, got {other:?}"),
            }
        }
        other => panic!("Expected VariableDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_yield_expression_outside_generator_remains_identifier() {
    let tokens = lex("yield;");
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
            Expression::Identifier(name) => assert_eq!(name, "yield"),
            other => panic!("Expected Identifier, got {other:?}"),
        },
        other => panic!("Expected ExpressionStatement, got {other:?}"),
    }
}
