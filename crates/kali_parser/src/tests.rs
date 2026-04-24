use super::*;
use kali_lexer::Lexer;

fn lex(source: &str) -> Vec<Token> {
    let lexer = Lexer::new(FileId::new(0), source.to_string());
    let result = lexer.lex_all();
    result.tokens
}

#[test]
fn test_parse_var_declaration() {
    let tokens = lex("var x = 1;");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::VariableDeclaration(vd) => {
            assert_eq!(vd.kind, "var");
            assert_eq!(vd.declarations.len(), 1);
        }
        _ => panic!("Expected VariableDeclaration"),
    }
}

#[test]
fn test_parse_side_effect_import_declaration() {
    let tokens = lex("import \"mod\";");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ImportDeclaration(decl) => {
            assert_eq!(decl.source, "mod");
            assert_eq!(decl.specifiers, vec![ImportSpecifier::SideEffect]);
        }
        _ => panic!("Expected ImportDeclaration"),
    }
}

#[test]
fn test_parse_default_import_declaration() {
    let tokens = lex("import value from \"mod\";");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ImportDeclaration(decl) => {
            assert_eq!(decl.source, "mod");
            assert_eq!(
                decl.specifiers,
                vec![ImportSpecifier::Default("value".to_string())]
            );
        }
        _ => panic!("Expected ImportDeclaration"),
    }
}

#[test]
fn test_parse_dynamic_import_expression() {
    let tokens = lex("const mod = import(\"./lazy\");");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::VariableDeclaration(vd) => {
            let init = vd.declarations[0].init.as_ref().expect("initializer");
            match init {
                Expression::ImportExpression(expr) => match &expr.source {
                    Expression::Literal(kali_ast::LiteralValue::String(source)) => {
                        assert_eq!(source, "\"./lazy\"")
                    }
                    other => panic!("unexpected import source: {other:?}"),
                },
                other => panic!("Expected ImportExpression, got {other:?}"),
            }
        }
        _ => panic!("Expected VariableDeclaration"),
    }
}

#[test]
fn test_parse_bigint_literal_expression() {
    let tokens = lex("const value = 42n;");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::VariableDeclaration(vd) => {
            let init = vd.declarations[0].init.as_ref().expect("initializer");
            match init {
                Expression::BigIntLiteral(value) => assert_eq!(value, "42n"),
                other => panic!("Expected BigIntLiteral, got {other:?}"),
            }
        }
        other => panic!("Expected VariableDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_optional_chain_member_expression() {
    let tokens = lex("minVersion(\"^1.2.3\")?.version;");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ExpressionStatement(expr_stmt) => match expr_stmt.expression.as_ref() {
            Expression::OptionalChainExpression(_) => {}
            other => panic!("Expected OptionalChainExpression, got {other:?}"),
        },
        other => panic!("Expected ExpressionStatement, got {other:?}"),
    }
}

#[test]
fn test_parse_optional_chain_index_expression() {
    let tokens = lex("call()?.[expr];");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ExpressionStatement(expr_stmt) => match expr_stmt.expression.as_ref() {
            Expression::OptionalChainExpression(_) => {}
            other => panic!("Expected OptionalChainExpression, got {other:?}"),
        },
        other => panic!("Expected ExpressionStatement, got {other:?}"),
    }
}

#[test]
fn test_parse_generator_function_declaration() {
    let tokens = lex("function* main() { yield 1; }");
    let mut parser = Parser::new(FileId::new(0), tokens);
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
    let mut parser = Parser::new(FileId::new(0), tokens);
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
    let mut parser = Parser::new(FileId::new(0), tokens);
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
fn test_parse_parenthesized_arrow_function_expression() {
    let tokens = lex("const add = (left, right) => left + right;");
    let mut parser = Parser::new(FileId::new(0), tokens);
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
                Expression::ArrowFunctionExpression(func) => {
                    assert!(!func.is_async, "expected async flag to be false");
                    assert_eq!(func.params.len(), 2);
                    assert_eq!(func.params[0].name, "left");
                    assert_eq!(func.params[1].name, "right");
                    assert!(matches!(func.body, Expression::BinaryExpression(_)));
                }
                other => panic!("Expected ArrowFunctionExpression, got {other:?}"),
            }
        }
        other => panic!("Expected VariableDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_single_parameter_arrow_function_expression() {
    let tokens = lex("const identity = value => value;");
    let mut parser = Parser::new(FileId::new(0), tokens);
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
                Expression::ArrowFunctionExpression(func) => {
                    assert_eq!(func.params.len(), 1);
                    assert_eq!(func.params[0].name, "value");
                    assert!(matches!(&func.body, Expression::Identifier(name) if name == "value"));
                }
                other => panic!("Expected ArrowFunctionExpression, got {other:?}"),
            }
        }
        other => panic!("Expected VariableDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_async_generator_function_declaration() {
    let tokens = lex("async function* main() { yield 1; }");
    let mut parser = Parser::new(FileId::new(0), tokens);
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
    let mut parser = Parser::new(FileId::new(0), tokens);
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
fn test_parse_async_function_expression() {
    let tokens = lex("const make = async function() { return 1; };");
    let mut parser = Parser::new(FileId::new(0), tokens);
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

#[test]
fn test_parse_yield_expression_outside_generator_remains_identifier() {
    let tokens = lex("yield;");
    let mut parser = Parser::new(FileId::new(0), tokens);
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

#[test]
fn test_parse_try_finally_statement() {
    let tokens = lex("try { value; } finally { other; }");
    let mut parser = Parser::new(FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::TryStatement(stmt) => {
            assert!(
                stmt.handler.is_none(),
                "unexpected catch clause: {:?}",
                stmt.handler
            );
            assert!(stmt.finalizer.is_some(), "expected finally block");
            assert_eq!(stmt.block.body.len(), 1);
            assert_eq!(stmt.finalizer.as_ref().unwrap().body.len(), 1);
        }
        other => panic!("Expected TryStatement, got {other:?}"),
    }
}
