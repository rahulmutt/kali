use crate::*;
use kali_ast::{
    CallExpression, Expression, ExpressionStatement, LiteralValue, MemberExpression,
    VariableDeclaration, VariableDeclarator,
};
use kali_error::_error_codes::e5;
use kali_test_support::fixtures;
use std::fs;

#[test]
fn test_resolution_accepts_number_is_finite_is_integer_and_is_nan_static_values() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Number".to_string()),
                    property: "isFinite".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Number".to_string()),
                    property: "isInteger".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Number".to_string()),
                    property: "isSafeInteger".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Number".to_string()),
                    property: "isSafeInteger".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.5))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Number".to_string(),
                    })),
                    property: "isNaN".to_string(),
                })),
                args: vec![Expression::Identifier("NaN".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Number".to_string(),
                    })),
                    property: "isInteger".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Number".to_string(),
                    })),
                    property: "isSafeInteger".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_accepts_number_is_alias_spellings() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Number".to_string(),
                    })),
                    property: "isFinite".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Number".to_string(),
                    })),
                    property: "isInteger".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Number".to_string(),
                    })),
                    property: "isSafeInteger".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_rejects_number_is_integer_with_dynamic_values_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                computed_index: None,
                object: Expression::Identifier("Number".to_string()),
                property: "isInteger".to_string(),
            })),
            args: vec![Expression::Identifier("value".to_string())],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(result.diagnostics[0].message.contains(
        "Number.isInteger is unavailable unless the argument is a statically-known primitive value"
    ));
}

#[test]
fn test_resolution_rejects_number_is_safe_integer_with_dynamic_values_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                computed_index: None,
                object: Expression::Identifier("Number".to_string()),
                property: "isSafeInteger".to_string(),
            })),
            args: vec![Expression::Identifier("value".to_string())],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(result.diagnostics[0].message.contains(
        "Number.isSafeInteger is unavailable unless the argument is a statically-known primitive value"
    ));
}

#[test]
fn test_resolution_allows_static_ascii_parse_int_in_non_browser_surface() {
    for source in [
        "const result = parseInt('42');",
        "const result = globalThis.parseInt('-0x10');",
        "const result = Number.parseInt('ff', 16);",
        "const source = '101'; const result = globalThis[\"Number\"][\"parseInt\"](Object.freeze(source), Object.freeze(2));",
        "const parse = Object.freeze(parseInt); const result = parse(Object.freeze('77'), 8);",
        "const parse = Object.freeze(globalThis[\"Number\"][\"parseInt\"]); const result = parse('10', Object.freeze(2));",
    ] {
        let dir = fixtures::tempdir();
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
fn test_resolution_allows_static_ascii_parse_float_integer_in_non_browser_surface() {
    for source in [
        "const result = parseFloat('42.0px');",
        "const result = globalThis.parseFloat('-1.2e1tail');",
        "const result = Number.parseFloat('7.000');",
        "const source = '6.02e2'; const result = globalThis[\"Number\"][\"parseFloat\"](Object.freeze(source));",
        "const parse = Object.freeze(parseFloat); const result = parse(Object.freeze('77.0'));",
        "const parse = Object.freeze(globalThis[\"Number\"][\"parseFloat\"]); const result = parse('10e1');",
    ] {
        let dir = fixtures::tempdir();
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
fn test_resolution_rejects_dynamic_or_invalid_parse_int_in_non_browser_surface() {
    for source in [
        "function parse(value) { return parseInt(value); }",
        "const result = parseInt('é');",
        "const result = parseInt('nope');",
        "const result = parseInt('10', 1);",
        "const result = parseInt('10', radix);",
    ] {
        let dir = fixtures::tempdir();
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
                    && diag.message.contains("parseInt")),
            "expected parseInt feature gate for {source}, got {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_rejects_dynamic_or_invalid_parse_float_in_non_browser_surface() {
    for source in [
        "function parse(value) { return parseFloat(value); }",
        "const result = parseFloat('é');",
        "const result = parseFloat('nope');",
        "const result = parseFloat('1.5');",
        "const result = parseFloat('10', 10);",
    ] {
        let dir = fixtures::tempdir();
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
                    && diag.message.contains("parseFloat")),
            "expected parseFloat feature gate for {source}, got {:?}",
            result.diagnostics
        );
    }
}
