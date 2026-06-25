use crate::test_support::lex;
use crate::*;
use kali_ast::{Expression, Statement};

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
fn test_parse_parenthesized_arrow_function_expression() {
    let tokens = lex("const add = (left, right) => left + right;");
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
fn test_parse_async_arrow_function_expression() {
    let tokens = lex("const add = async (left, right) => left + right;");
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
                Expression::ArrowFunctionExpression(func) => {
                    assert!(func.is_async, "expected async flag to be preserved");
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
fn test_parse_async_arrow_function_return_type_annotation_with_multiple_params() {
    let tokens = lex("const add = async (left, right): number => left + right;");
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
                Expression::ArrowFunctionExpression(func) => {
                    assert!(func.is_async, "expected async flag to be preserved");
                    assert_eq!(func.params.len(), 2);
                    assert_eq!(func.params[0].name, "left");
                    assert_eq!(func.params[1].name, "right");
                    assert_eq!(func.returnType.as_deref(), Some("number"));
                    assert!(matches!(func.body, Expression::BinaryExpression(_)));
                }
                other => panic!("Expected ArrowFunctionExpression, got {other:?}"),
            }
        }
        other => panic!("Expected VariableDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_async_single_parameter_arrow_function_expression() {
    let tokens = lex("const identity = async value => value;");
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
                Expression::ArrowFunctionExpression(func) => {
                    assert!(func.is_async, "expected async flag to be preserved");
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
fn test_parse_async_arrow_function_return_type_annotation() {
    let tokens = lex("const identity = async (value): number => value;");
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
                Expression::ArrowFunctionExpression(func) => {
                    assert!(func.is_async, "expected async flag to be preserved");
                    assert_eq!(func.params.len(), 1);
                    assert_eq!(func.params[0].name, "value");
                    assert_eq!(func.returnType.as_deref(), Some("number"));
                    assert!(matches!(&func.body, Expression::Identifier(name) if name == "value"));
                }
                other => panic!("Expected ArrowFunctionExpression, got {other:?}"),
            }
        }
        other => panic!("Expected VariableDeclaration, got {other:?}"),
    }
}

#[test]
fn test_parse_arrow_function_return_type_annotation() {
    let tokens = lex("const identity = (value): number => value;");
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
                Expression::ArrowFunctionExpression(func) => {
                    assert!(!func.is_async, "expected async flag to be false");
                    assert_eq!(func.params.len(), 1);
                    assert_eq!(func.params[0].name, "value");
                    assert_eq!(func.returnType.as_deref(), Some("number"));
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

fn assert_parse_class_method_modifiers_are_preserved(
    source: &str,
    is_async: bool,
    generator: bool,
) {
    let tokens = lex(source);
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
            assert_eq!(method.name, "main");
            assert_eq!(method.is_async, is_async);
            assert_eq!(method.generator, generator);
            assert!(
                method.body.is_some(),
                "expected class method body to be preserved"
            );
        }
        other => panic!("Expected ClassDeclaration, got {other:?}"),
    }
}

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
