use crate::*;
use kali_ast::{CallExpression, Expression, ExpressionStatement, LiteralValue, MemberExpression};
use std::fs;

#[test]
fn test_resolution_supports_promise_all_settled_member_calls() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Promise".to_string()),
                    property: "allSettled".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Promise".to_string(),
                    })),
                    property: "allSettled".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(2.0))],
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
fn test_resolution_supports_promise_any_member_calls() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Promise".to_string()),
                    property: "any".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Promise".to_string(),
                    })),
                    property: "any".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(2.0))],
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
fn test_resolution_recognizes_bracketed_promise_combinator_callable_names_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"Object.freeze((globalThis["Promise"]).any)([Promise.resolve(1)]); Object.freeze((globalThis['Promise'])['race'])([Promise.resolve(1)]);"#,
    )
    .unwrap();

    let lexer = kali_lexer::Lexer::new(
        kali_common::FileId::new(0),
        r#"Object.freeze((globalThis["Promise"]).any)([Promise.resolve(1)]); Object.freeze((globalThis['Promise'])['race'])([Promise.resolve(1)]);"#
            .to_string(),
    );
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let ctx = TypeContext::with_base_path(&source_path);

    let Statement::ExpressionStatement(ExpressionStatement { expression }) = &statements[0] else {
        panic!("expected expression statement");
    };
    let Expression::CallExpression(call) = expression.as_ref() else {
        panic!("unexpected expression: {expression:?}");
    };
    assert_eq!(
        ctx.resolve_static_callable_name(&call.callee).as_deref(),
        Some("globalThis.Promise.any")
    );

    let Statement::ExpressionStatement(ExpressionStatement { expression }) = &statements[1] else {
        panic!("expected expression statement");
    };
    let Expression::CallExpression(call) = expression.as_ref() else {
        panic!("unexpected expression: {expression:?}");
    };
    assert_eq!(
        ctx.resolve_static_callable_name(&call.callee).as_deref(),
        Some("globalThis.Promise.race")
    );
}
