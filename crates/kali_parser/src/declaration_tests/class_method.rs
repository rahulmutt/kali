use super::*;

#[test]
fn test_parse_generator_class_method_preserves_generator_flag() {
    assert_parse_class_method_modifiers_are_preserved(
        "class Example { *main() { yield 1; } }",
        false,
        true,
    );
}

#[test]
fn test_parse_generator_class_method_delegating_yield_expression() {
    let tokens = lex("class Example { *main() { yield* other(); } }");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ClassDeclaration(class_decl) => {
            assert_eq!(class_decl.body.methods.len(), 1);
            let method = &class_decl.body.methods[0];
            assert!(method.generator, "expected generator flag to be preserved");
            let body = method.body.as_ref().expect("method body");
            assert_eq!(body.body.len(), 1);
            match &body.body[0] {
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
        other => panic!("Expected ClassDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_async_generator_class_method_preserves_generator_flags() {
    assert_parse_class_method_modifiers_are_preserved(
        "class Example { async *main() { yield 1; } }",
        true,
        true,
    );
}

#[test]
fn test_parse_class_expression_preserves_method_modifiers() {
    let tokens = lex("const Example = class NamedExample { async *main() { yield* other(); } };");
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
                Expression::ClassExpression(class_expr) => {
                    assert_eq!(class_expr.id.as_deref(), Some("NamedExample"));
                    assert_eq!(class_expr.body.methods.len(), 1);
                    let method = &class_expr.body.methods[0];
                    assert_eq!(method.name, "main");
                    assert!(method.is_async, "expected async flag to be preserved");
                    assert!(method.generator, "expected generator flag to be preserved");
                    let body = method.body.as_ref().expect("method body");
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
                other => panic!("Expected ClassExpression, got {other:?}"),
            }
        }
        other => panic!("Expected VariableDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_default_export_class_expression_preserves_method_modifiers() {
    let tokens = lex("export default (class NamedExample { async *main() { yield* other(); } });");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ExportDefault(decl) => match decl {
            kali_ast::ExportDefaultDeclaration::Expression(expr) => {
                let mut expr = expr;
                loop {
                    match expr {
                        Expression::ParenthesizedExpression(parenthesized) => {
                            expr = parenthesized.expression.as_ref();
                        }
                        Expression::ClassExpression(class_expr) => {
                            assert_eq!(class_expr.id.as_deref(), Some("NamedExample"));
                            assert_eq!(class_expr.body.methods.len(), 1);
                            let method = &class_expr.body.methods[0];
                            assert_eq!(method.name, "main");
                            assert!(method.is_async, "expected async flag to be preserved");
                            assert!(method.generator, "expected generator flag to be preserved");
                            let body = method.body.as_ref().expect("method body");
                            assert_eq!(body.body.len(), 1);
                            match &body.body[0] {
                                Statement::ExpressionStatement(expr_stmt) => {
                                    match expr_stmt.expression.as_ref() {
                                        Expression::YieldExpression(yield_expr) => {
                                            assert!(
                                                yield_expr.delegate,
                                                "expected yield* delegation to be preserved"
                                            );
                                            let argument = yield_expr
                                                .argument
                                                .as_ref()
                                                .expect("yield argument");
                                            match argument {
                                                Expression::CallExpression(call_expr) => {
                                                    assert_eq!(call_expr.args.len(), 0)
                                                }
                                                other => {
                                                    panic!("unexpected yield* argument: {other:?}")
                                                }
                                            }
                                        }
                                        other => panic!("Expected YieldExpression, got {other:?}"),
                                    }
                                }
                                other => panic!("Expected ExpressionStatement, got {other:?}"),
                            }
                            break;
                        }
                        other => panic!("Expected default-export class expression, got {other:?}"),
                    }
                }
            }
            other => panic!("Expected default-export class expression, got {other:?}"),
        },
        other => panic!("Expected ExportDefaultDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_default_export_class_declaration_preserves_method_modifiers() {
    let tokens = lex("export default class NamedDeclExample { async *main() { yield* other(); } }");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ExportDefault(decl) => match decl {
            kali_ast::ExportDefaultDeclaration::ClassDeclaration(class_decl) => {
                assert_eq!(class_decl.name, "NamedDeclExample");
                assert_eq!(class_decl.body.methods.len(), 1);
                let method = &class_decl.body.methods[0];
                assert_eq!(method.name, "main");
                assert!(method.is_async, "expected async flag to be preserved");
                assert!(method.generator, "expected generator flag to be preserved");
                let body = method.body.as_ref().expect("method body");
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
            other => panic!("Expected default-export class declaration, got {other:?}"),
        },
        other => panic!("Expected ExportDefaultDeclaration, got {other:?}"),
    }
}
