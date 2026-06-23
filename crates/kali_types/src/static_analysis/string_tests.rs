use crate::*;
use kali_ast::{BinaryExpression, BlockStatement, CallExpression, Expression, ExpressionStatement, ForOfLefthand, ForOfStatement, LiteralValue, MemberExpression, VariableDeclaration, VariableDeclarator};
use kali_error::_error_codes::e5;
use std::fs;

#[test]
fn test_resolution_supports_for_of_array_iteration_with_const_string_alias_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = \"hello\"; const alias = value; for (const item of [alias]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::String(
                    "hello".to_string(),
                ))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Identifier("value".to_string())),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "item".to_string(),
                    init: None,
                }],
            }),
            right: Expression::ArrayExpression(kali_ast::ArrayExpression {
                elements: vec![Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::Identifier("alias".to_string()),
                ))],
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("item".to_string())],
                    }))),
                })],
            })),
            is_await: false,
        }),
    ];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_for_of_string_concatenation_iteration_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const prefix = \"he\"; const suffix = \"llo\"; for (const ch of prefix + suffix) { console.log(ch); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "prefix".to_string(),
                init: Some(Expression::Literal(LiteralValue::String("he".to_string()))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "suffix".to_string(),
                init: Some(Expression::Literal(LiteralValue::String("llo".to_string()))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "ch".to_string(),
                    init: None,
                }],
            }),
            right: Expression::BinaryExpression(Box::new(BinaryExpression {
                left: Expression::Identifier("prefix".to_string()),
                operator: "+".to_string(),
                right: Expression::Identifier("suffix".to_string()),
            })),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("ch".to_string())],
                    }))),
                })],
            })),
            is_await: false,
        }),
    ];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_for_await_string_concatenation_iteration_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const prefix = \"he\"; const suffix = \"llo\"; for await (const ch of prefix + suffix) { console.log(ch); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "prefix".to_string(),
                init: Some(Expression::Literal(LiteralValue::String("he".to_string()))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "suffix".to_string(),
                init: Some(Expression::Literal(LiteralValue::String("llo".to_string()))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "ch".to_string(),
                    init: None,
                }],
            }),
            right: Expression::BinaryExpression(Box::new(BinaryExpression {
                left: Expression::Identifier("prefix".to_string()),
                operator: "+".to_string(),
                right: Expression::Identifier("suffix".to_string()),
            })),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("ch".to_string())],
                    }))),
                })],
            })),
            is_await: true,
        }),
    ];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_for_of_template_literal_string_iteration_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "for (const ch of `hello`) { console.log(ch); }",
    )
    .unwrap();

    let statements = vec![Statement::ForOfStatement(ForOfStatement {
        left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "ch".to_string(),
                init: None,
            }],
        }),
        right: Expression::TemplateLiteral(kali_ast::TemplateLiteral {
            quasis: vec![kali_ast::TemplateElement {
                value: "hello".to_string(),
                tail: true,
            }],
            expressions: vec![],
        }),
        body: Box::new(Statement::BlockStatement(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("console".to_string()),
                        property: "log".to_string(),
                    })),
                    args: vec![Expression::Identifier("ch".to_string())],
                }))),
            })],
        })),
        is_await: false,
    })];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_for_await_of_array_iteration_with_const_string_alias_in_ts_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const value = \"hello\"; const alias = value; for await (const item of [alias]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::String(
                    "hello".to_string(),
                ))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Identifier("value".to_string())),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "item".to_string(),
                    init: None,
                }],
            }),
            right: Expression::ArrayExpression(kali_ast::ArrayExpression {
                elements: vec![Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::Identifier("alias".to_string()),
                ))],
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("item".to_string())],
                    }))),
                })],
            })),
            is_await: true,
        }),
    ];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_for_await_of_array_iteration_with_const_string_alias_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = \"hello\"; const alias = value; for await (const item of [alias]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::String(
                    "hello".to_string(),
                ))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Identifier("value".to_string())),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "item".to_string(),
                    init: None,
                }],
            }),
            right: Expression::ArrayExpression(kali_ast::ArrayExpression {
                elements: vec![Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::Identifier("alias".to_string()),
                ))],
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("item".to_string())],
                    }))),
                })],
            })),
            is_await: true,
        }),
    ];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_allows_static_ascii_string_search_family_in_non_browser_surface() {
    for source in [
        "const result = \"hello\".includes(\"ell\");",
        "const result = \"hello\".indexOf(\"l\", 3);",
        "const result = \"hello\".lastIndexOf(\"l\");",
        "const source = \"hello\"; const result = Object.freeze(source).includes(Object.freeze(\"he\"));",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {source}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_rejects_argument_bearing_static_array_to_string_in_non_browser_surface() {
    let source = "const result = [0, 1, 2].toString('-');";
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, source).unwrap();

    let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)
                && diag.message.contains("Array.prototype.toString")),
        "expected Array.prototype.toString feature gate for {source}, got {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_rejects_dynamic_or_non_ascii_string_search_family_in_non_browser_surface() {
    for source in [
        "function has(needle) { return \"hello\".includes(needle); }",
        "function at(from) { return \"hello\".indexOf(\"l\", from); }",
        "const result = \"héllo\".lastIndexOf(\"l\");",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)
                    && diag.message.contains("string search method")),
            "expected string search feature gate for {source}, got {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_allows_static_ascii_string_slice_in_non_browser_surface() {
    for source in [
        "const result = 'hello'.slice(1);",
        "const result = 'hello'.slice(1, 4);",
        "const result = 'hello'.slice(1.5, 4.9);",
        "const result = 'hello'.slice(-4, -1);",
        "const source = 'hello'; const result = Object.freeze(source).slice(Object.freeze(1.5), 4.9);",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {source}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_allows_static_ascii_string_substring_in_non_browser_surface() {
    for source in [
        "const result = 'hello'.substring(1);",
        "const result = 'hello'.substring(1, 4);",
        "const result = 'hello'.substring(1.5, 4.9);",
        "const result = 'hello'.substring(4, 1);",
        "const source = 'hello'; const result = Object.freeze(source).substring(Object.freeze(1.5), 4.9);"
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {source}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_rejects_dynamic_or_non_ascii_string_substring_in_non_browser_surface() {
    for source in [
        "function cut(start) { return 'hello'.substring(start); }",
        "function cut(value) { return value.substring(1); }",
        "const result = 'hello'.substring(Infinity);",
        "const result = 'héllo'.substring(1);",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)
                    && diag.message.contains("String.prototype.substring")),
            "expected String.prototype.substring feature gate for {source}, got {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_allows_static_ascii_string_concat_in_non_browser_surface() {
    for source in [
        "const result = 'he'.concat('llo');",
        "const result = 'he'.concat('l', 'lo');",
        "const result = 'hello'.concat();",
        "const suffix = 'llo'; const result = Object.freeze('he').concat(Object.freeze(suffix));",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {source}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_allows_static_ascii_string_search_omitted_search_in_non_browser_surface() {
    for source in [
        "const result = 'hello'.includes();",
        "const result = 'hello'.indexOf();",
        "const result = 'hello'.lastIndexOf();",
        "const result = 'hello'.startsWith();",
        "const result = 'hello'.endsWith();",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {source}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_rejects_dynamic_or_non_ascii_string_concat_in_non_browser_surface() {
    for source in [
        "function join(suffix) { return 'he'.concat(suffix); }",
        "function join(value) { return value.concat('llo'); }",
        "const result = 'hé'.concat('llo');",
        "const result = 'he'.concat('lló');",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)
                    && diag.message.contains("String.prototype.concat")),
            "expected String.prototype.concat feature gate for {source}, got {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_allows_static_ascii_string_at_in_non_browser_surface() {
    for source in [
        "const result = 'hello'.at();",
        "const result = 'hello'.at(1);",
        "const result = 'hello'.at(-1);",
        "const result = 'hello'.at(99);",
        "const source = 'hello'; const result = Object.freeze(source).at(Object.freeze(4));",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {source}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_rejects_dynamic_or_non_ascii_string_at_in_non_browser_surface() {
    for source in [
        "function pick(index) { return 'hello'.at(index); }",
        "function pick(value) { return value.at(1); }",
        "const result = 'hello'.at(1.5);",
        "const result = 'héllo'.at(1);",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)
                    && diag.message.contains("String.prototype.at")),
            "expected String.prototype.at feature gate for {source}, got {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_allows_static_ascii_string_char_at_in_non_browser_surface() {
    for source in [
        "const result = 'hello'.charAt();",
        "const result = 'hello'.charAt(1);",
        "const result = 'hello'.charAt(-1);",
        "const source = 'hello'; const result = Object.freeze(source).charAt(Object.freeze(4));",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {source}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_rejects_dynamic_or_non_ascii_string_char_at_in_non_browser_surface() {
    for source in [
        "function pick(index) { return 'hello'.charAt(index); }",
        "function pick(value) { return value.charAt(1); }",
        "const result = 'hello'.charAt(1.5);",
        "const result = 'héllo'.charAt(1);",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)
                    && diag.message.contains("String.prototype.charAt")),
            "expected String.prototype.charAt feature gate for {source}, got {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_allows_static_ascii_string_char_code_at_in_non_browser_surface() {
    for source in [
        "const result = 'hello'.charCodeAt();",
        "const result = 'hello'.charCodeAt(1);",
        "const result = 'hello'.charCodeAt(-1);",
        "const result = 'hello'.charCodeAt(99);",
        "const source = 'hello'; const result = Object.freeze(source).charCodeAt(Object.freeze(4));",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {source}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_rejects_dynamic_or_non_ascii_string_char_code_at_in_non_browser_surface() {
    for source in [
        "function pick(index) { return 'hello'.charCodeAt(index); }",
        "function pick(value) { return value.charCodeAt(1); }",
        "const result = 'hello'.charCodeAt(1.5);",
        "const result = 'héllo'.charCodeAt(1);",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)
                    && diag.message.contains("String.prototype.charCodeAt")),
            "expected String.prototype.charCodeAt feature gate for {source}, got {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_allows_static_ascii_string_trim_family_in_non_browser_surface() {
    for source in [
        "const result = '  hello  '.trim();",
        "const result = '  hello  '.trimStart();",
        "const result = '  hello  '.trimEnd();",
        "const result = '  hello  '.trimLeft();",
        "const result = '  hello  '.trimRight();",
        "const source = ' hello '; const result = Object.freeze(source).trim();",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {source}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_allows_static_ascii_string_case_family_in_non_browser_surface() {
    for source in [
        "const result = 'Hello'.toLowerCase();",
        "const result = 'Hello'.toUpperCase();",
        "const result = 'Hello'.toLocaleLowerCase();",
        "const result = 'Hello'.toLocaleUpperCase();",
        "const source = 'Hello'; const result = Object.freeze(source).toLowerCase();",
        "const source = 'Hello'; const result = Object.freeze(source).toLocaleUpperCase();",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {source}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_allows_static_ascii_string_replace_in_non_browser_surface() {
    for source in [
        "const result = 'hello hello'.replace('hello', 'hi');",
        "const result = 'abc'.replace('', 'X');",
        "const source = 'hello'; const result = Object.freeze(source).replace(Object.freeze('h'), 'j');",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {source}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_allows_static_ascii_string_split_in_non_browser_surface() {
    for source in [
        "const result = 'a,b,c'.split(',');",
        "const result = 'abc'.split('', 2);",
        "const whole = 'abc'.split();",
        "const source = 'a-b'; const result = Object.freeze(source).split(Object.freeze('-'));",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {source}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_rejects_dynamic_or_non_ascii_string_split_in_non_browser_surface() {
    for source in [
        "function split(separator) { return 'a,b'.split(separator); }",
        "function split(value) { return value.split(','); }",
        "const result = 'a,b'.split(',', value);",
        "const result = 'á,b'.split(',');",
        "const result = 'a,b'.split('é');",
        "const result = 'a,b'.split(',', -1);",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)
                    && diag.message.contains("String.prototype.split")),
            "expected String.prototype.split feature gate for {source}, got {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_rejects_dynamic_or_non_ascii_string_replace_in_non_browser_surface() {
    for source in [
        "function replace(search) { return 'hello'.replace(search, 'x'); }",
        "function replace(value) { return value.replace('h', 'j'); }",
        "const result = 'hello'.replace('h', value);",
        "const result = 'héllo'.replace('h', 'j');",
        "const result = 'hello'.replace('h', 'é');",
        "const result = 'hello'.replace('h', '$&');",
        "const result = 'hello'.replaceAll('h', '$&');",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)
                    && diag.message.contains("String.prototype.replace")),
            "expected String.prototype.replace feature gate for {source}, got {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_rejects_non_ascii_or_argument_string_trim_family_in_non_browser_surface() {
    for source in [
        "const result = '  héllo  '.trim();",
        "const result = '  hello  '.trimStart(1);",
        "const result = '  hello  '.trimEnd(1);",
        "const result = '  hello  '.trimLeft(1);",
        "const result = '  hello  '.trimRight(1);",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)
                    && diag.message.contains("String.prototype.")),
            "expected String.prototype trim feature gate for {source}, got {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_rejects_non_ascii_or_argument_string_case_family_in_non_browser_surface() {
    for source in [
        "const result = 'héllo'.toLowerCase();",
        "const result = 'hello'.toUpperCase(1);",
        "const result = 'hello'.toLocaleLowerCase('tr');",
        "const result = 'héllo'.toLocaleUpperCase();",
        "function convert(value) { return value.toLowerCase(); }",
        "function convert(value) { return value.toLocaleLowerCase(); }",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)
                    && diag.message.contains("String.prototype.")),
            "expected String.prototype case feature gate for {source}, got {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_rejects_dynamic_or_non_ascii_string_slice_in_non_browser_surface() {
    for source in [
        "function cut(start) { return 'hello'.slice(start); }",
        "function cut(end) { return 'hello'.slice(1, end); }",
        "const result = 'héllo'.slice(1);",
        "const result = 'hello'.slice(1 / 0);",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)
                    && diag.message.contains("String.prototype.slice")),
            "expected String.prototype.slice feature gate for {source}, got {:?}",
            result.diagnostics
        );
    }
}
