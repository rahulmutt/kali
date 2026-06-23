use crate::*;
use crate::test_support::*;
use kali_ast::{AssignmentExpression, AssignmentOperator, BlockStatement, CallExpression, DecoratedExpression, Expression, ExpressionStatement, ForOfLefthand, ForOfStatement, LiteralValue, MemberExpression, VariableDeclaration, VariableDeclarator};
use kali_error::_error_codes::e5;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_resolution_accepts_new_set_iteration_target_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    let source = r#"for (const value of new Set([1, 2, 1])) {
    console.log(value);
}
"#;
    fs::write(&source_path, source).unwrap();

    let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let Statement::ForOfStatement(ForOfStatement { right, .. }) = &statements[0] else {
        panic!("expected for-of statement");
    };

    let mut ctx = TypeContext::with_base_path(&source_path);
    assert!(ctx.is_static_array_iteration_target(right));

    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_accepts_new_set_iteration_target_via_builtin_alias_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    let source = r#"const setAlias = Set; const wrappedSetAlias = (setAlias); const values = [1, 2, 1]; for (const value of new setAlias(values)) { console.log(value); } for await (const value of new (wrappedSetAlias)(values)) { console.log(value); }
"#;
    fs::write(&source_path, source).unwrap();

    let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

fn assert_resolution_accepts_frozen_iterator_protocol_edge(source_filename: &str, source: &str) {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join(source_filename);
    fs::write(&source_path, source).unwrap();

    let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_accepts_frozen_set_iteration_in_js_input() {
    assert_resolution_accepts_frozen_iterator_protocol_edge(
        "main.js",
        "for (const item of new Set(Object.freeze([1, 2, 1]))) { console.log(item); }",
    );
}

#[test]
fn test_resolution_accepts_frozen_map_iteration_in_ts_input() {
    assert_resolution_accepts_frozen_iterator_protocol_edge(
        "main.ts",
        "for (const entry of new Map(Object.freeze([[1, 2], [1, 3], [4, 5]]))) { console.log(entry[0], entry[1]); }",
    );
}

#[test]
fn test_resolution_accepts_new_map_iteration_target_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    let source = r#"for (const entry of new Map([[1, 2], [1, 3], [4, 5]])) {
    console.log(entry[0], entry[1]);
}
"#;
    fs::write(&source_path, source).unwrap();

    let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let Statement::ForOfStatement(ForOfStatement { right, .. }) = &statements[0] else {
        panic!("expected for-of statement");
    };

    let mut ctx = TypeContext::with_base_path(&source_path);
    assert!(ctx.is_static_array_iteration_target(right));

    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_accepts_new_map_iteration_target_via_builtin_alias_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    let source = r#"const mapAlias = Map; const wrappedMapAlias = (mapAlias); const values = [[1, 2], [1, 3], [4, 5]]; for (const entry of new mapAlias(values)) { console.log(entry[0], entry[1]); } for await (const entry of new (wrappedMapAlias)(values)) { console.log(entry[0], entry[1]); }
"#;
    fs::write(&source_path, source).unwrap();

    let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_accepts_global_this_set_and_map_iteration_targets_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    let source = r#"for (const value of new globalThis.Set([1, 2, 1])) {
    console.log(value);
}
for (const value of new (globalThis["Set"])([1, 2, 1])) {
    console.log(value);
}
for (const value of new (globalThis['Set'])([1, 2, 1])) {
    console.log(value);
}
for (const value of new globalThis["Set"]([1, 2, 1])) {
    console.log(value);
}
for await (const entry of new globalThis.Map([[1, 2], [1, 3], [4, 5]])) {
    console.log(entry[0], entry[1]);
}
for await (const entry of new (globalThis["Map"])([[1, 2], [1, 3], [4, 5]])) {
    console.log(entry[0], entry[1]);
}
for await (const entry of new (globalThis['Map'])([[1, 2], [1, 3], [4, 5]])) {
    console.log(entry[0], entry[1]);
}
for await (const entry of new globalThis['Map']([[1, 2], [1, 3], [4, 5]])) {
    console.log(entry[0], entry[1]);
}
"#;
    fs::write(&source_path, source).unwrap();

    let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_for_of_array_iteration() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "for (const value of [1, 2]) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![Statement::ForOfStatement(ForOfStatement {
        left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: None,
            }],
        }),
        right: Expression::ArrayExpression(kali_ast::ArrayExpression {
            elements: vec![
                Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::Literal(LiteralValue::Number(1.0)),
                )),
                Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::Literal(LiteralValue::Number(2.0)),
                )),
            ],
        }),
        body: Box::new(Statement::BlockStatement(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("console".to_string()),
                        property: "log".to_string(),
                    })),
                    args: vec![Expression::Identifier("value".to_string())],
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
fn test_resolution_supports_for_of_array_iteration_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "for (const value of [1, 2]) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![Statement::ForOfStatement(ForOfStatement {
        left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: None,
            }],
        }),
        right: Expression::ArrayExpression(kali_ast::ArrayExpression {
            elements: vec![
                Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::Literal(LiteralValue::Number(1.0)),
                )),
                Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::Literal(LiteralValue::Number(2.0)),
                )),
            ],
        }),
        body: Box::new(Statement::BlockStatement(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("console".to_string()),
                        property: "log".to_string(),
                    })),
                    args: vec![Expression::Identifier("value".to_string())],
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
fn test_resolution_supports_for_of_array_from_iteration_in_js_input() {
    assert_resolution_accepts_frozen_iterator_protocol_edge(
        "main.js",
        "for (const value of Array.from([1, 2])) { console.log(value); }",
    );
}

#[test]
fn test_resolution_supports_frozen_for_of_array_from_iteration_in_js_input() {
    assert_resolution_accepts_frozen_iterator_protocol_edge(
        "main.js",
        "const values = [1, 2]; for (const value of Object.freeze(Array.from)(values)) { console.log(value); }",
    );
}

#[test]
fn test_resolution_supports_frozen_global_this_array_from_iteration_in_js_input() {
    assert_resolution_accepts_frozen_iterator_protocol_edge(
        "main.js",
        "const values = [1, 2]; for (const value of Object.freeze(globalThis.Array.from)(values)) { console.log(value); }",
    );
}

#[test]
fn test_resolution_supports_frozen_bracketed_global_this_array_from_dot_iteration_in_js_input() {
    assert_resolution_accepts_frozen_iterator_protocol_edge(
        "main.js",
        r#"const values = [1, 2]; for (const value of Object.freeze(globalThis["Array"].from)(values)) { console.log(value); }"#,
    );
}

#[test]
fn test_resolution_supports_frozen_bracketed_global_this_array_from_iteration_in_js_input() {
    assert_resolution_accepts_frozen_iterator_protocol_edge(
        "main.js",
        "const values = [1, 2]; for (const value of Object.freeze((globalThis[\"Array\"]))[\"from\"](values)) { console.log(value); }",
    );
}

#[test]
fn test_resolution_supports_parenthesized_bracketed_frozen_global_this_array_from_iteration_in_js_input(
) {
    assert_resolution_accepts_frozen_iterator_protocol_edge(
        "main.js",
        "const values = [1, 2]; for (const value of Object.freeze((globalThis[\"Array\"])[\"from\"])(values)) { console.log(value); }",
    );
}

#[test]
fn test_resolution_supports_parenthesized_single_quoted_bracketed_frozen_global_this_array_from_iteration_in_js_input(
) {
    assert_resolution_accepts_frozen_iterator_protocol_edge(
        "main.js",
        "const values = [1, 2]; for (const value of Object.freeze((globalThis['Array'])[\"from\"])(values)) { console.log(value); }",
    );
}

#[test]
fn test_resolution_recognizes_parenthesized_bracketed_frozen_global_this_array_from_dot_callable_name_in_js_input(
) {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const values = [1, 2]; for (const value of Object.freeze((globalThis["Array"]).from)(values)) { console.log(value); }"#,
    )
    .unwrap();

    let lexer = kali_lexer::Lexer::new(
        kali_common::FileId::new(0),
        r#"const values = [1, 2]; for (const value of Object.freeze((globalThis["Array"]).from)(values)) { console.log(value); }"#
            .to_string(),
    );
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let Statement::ForOfStatement(ForOfStatement { right, .. }) = &statements[1] else {
        panic!("expected for-of statement");
    };

    let ctx = TypeContext::with_base_path(&source_path);
    let Expression::CallExpression(call) = right else {
        panic!("unexpected right expression: {right:?}");
    };

    assert_eq!(
        ctx.resolve_static_callable_name(&call.callee).as_deref(),
        Some("globalThis.Array.from")
    );
}

#[test]
fn test_resolution_recognizes_parenthesized_single_quoted_frozen_global_this_array_from_dot_callable_name_in_js_input(
) {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const values = [1, 2]; for (const value of Object.freeze((globalThis.Array))['from'](values)) { console.log(value); }"#,
    )
    .unwrap();

    let lexer = kali_lexer::Lexer::new(
        kali_common::FileId::new(0),
        r#"const values = [1, 2]; for (const value of Object.freeze((globalThis.Array))['from'](values)) { console.log(value); }"#
            .to_string(),
    );
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let Statement::ForOfStatement(ForOfStatement { right, .. }) = &statements[1] else {
        panic!("expected for-of statement");
    };

    let ctx = TypeContext::with_base_path(&source_path);
    let Expression::CallExpression(call) = right else {
        panic!("unexpected right expression: {right:?}");
    };

    assert_eq!(
        ctx.resolve_static_callable_name(&call.callee).as_deref(),
        Some("globalThis.Array.from")
    );
}

#[test]
fn test_resolution_supports_parenthesized_bracketed_and_single_quoted_frozen_global_this_array_from_dot_iteration_in_js_input(
) {
    assert_resolution_accepts_frozen_iterator_protocol_edge(
        "main.js",
        r#"const values = [1, 2]; for (const value of Object.freeze((globalThis["Array"]).from)(values)) { console.log(value); } for (const value of Object.freeze((globalThis['Array']).from)(values)) { console.log(value); }"#,
    );
}

#[test]
fn test_resolution_supports_parenthesized_frozen_array_from_iteration_in_js_input() {
    assert_resolution_accepts_frozen_iterator_protocol_edge(
        "main.js",
        "const values = [1, 2]; for (const value of Object.freeze((Array.from))(values)) { console.log(value); }",
    );
}

#[test]
fn test_resolution_supports_sequence_wrapped_frozen_array_from_iteration_in_js_input() {
    assert_resolution_accepts_frozen_iterator_protocol_edge(
        "main.js",
        "const values = [1, 2]; for (const value of Object.freeze((Array.from, Array.from))(values)) { console.log(value); }",
    );
}

#[test]
fn test_resolution_recognizes_nullish_wrapped_array_from_callable_name_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "for (const value of Object.freeze((null ?? Array.from))([1, 2])) { console.log(value); }",
    )
    .unwrap();

    let lexer = kali_lexer::Lexer::new(
        kali_common::FileId::new(0),
        "for (const value of Object.freeze((null ?? Array.from))([1, 2])) { console.log(value); }"
            .to_string(),
    );
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let Statement::ForOfStatement(ForOfStatement { right, .. }) = &statements[0] else {
        panic!("expected for-of statement");
    };

    let ctx = TypeContext::with_base_path(&source_path);
    let Expression::CallExpression(call) = right else {
        panic!("unexpected right expression: {right:?}");
    };

    assert_eq!(
        ctx.resolve_static_callable_name(&call.callee).as_deref(),
        Some("Array.from")
    );
}

#[test]
fn test_resolution_recognizes_frozen_array_from_callable_name_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const alias = Object.freeze((Array.from)); alias([1, 2]);",
    )
    .unwrap();

    let lexer = kali_lexer::Lexer::new(
        kali_common::FileId::new(0),
        "const alias = Object.freeze((Array.from)); alias([1, 2]);".to_string(),
    );
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let Statement::VariableDeclaration(variable) = &statements[0] else {
        panic!("expected variable declaration");
    };
    let declarator = variable.declarations.first().expect("declarator");
    let Some(initializer) = declarator.init.as_ref() else {
        panic!("expected variable initializer");
    };
    let ctx = TypeContext::with_base_path(&source_path);

    assert_eq!(
        ctx.resolve_static_callable_name(initializer).as_deref(),
        Some("Array.from")
    );
}

#[test]
fn test_resolution_recognizes_and_wrapped_array_from_callable_name_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "for (const value of Object.freeze((true && Array.from))([1, 2])) { console.log(value); }",
    )
    .unwrap();

    let lexer = kali_lexer::Lexer::new(
        kali_common::FileId::new(0),
        "for (const value of Object.freeze((true && Array.from))([1, 2])) { console.log(value); }"
            .to_string(),
    );
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let Statement::ForOfStatement(ForOfStatement { right, .. }) = &statements[0] else {
        panic!("expected for-of statement");
    };

    let ctx = TypeContext::with_base_path(&source_path);
    let Expression::CallExpression(call) = right else {
        panic!("unexpected right expression: {right:?}");
    };

    assert_eq!(
        ctx.resolve_static_callable_name(&call.callee).as_deref(),
        Some("Array.from")
    );
}

#[test]
fn test_resolution_recognizes_or_wrapped_array_from_callable_name_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "for (const value of Object.freeze((false || Array.from))([1, 2])) { console.log(value); }",
    )
    .unwrap();

    let lexer = kali_lexer::Lexer::new(
        kali_common::FileId::new(0),
        "for (const value of Object.freeze((false || Array.from))([1, 2])) { console.log(value); }"
            .to_string(),
    );
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let Statement::ForOfStatement(ForOfStatement { right, .. }) = &statements[0] else {
        panic!("expected for-of statement");
    };

    let ctx = TypeContext::with_base_path(&source_path);
    let Expression::CallExpression(call) = right else {
        panic!("unexpected right expression: {right:?}");
    };

    assert_eq!(
        ctx.resolve_static_callable_name(&call.callee).as_deref(),
        Some("Array.from")
    );
}

#[test]
fn test_resolution_recognizes_and_wrapped_bracketed_global_this_array_from_callable_name_in_js_input(
) {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"for (const value of Object.freeze((true && globalThis["Array"].from))([1, 2])) { console.log(value); }"#,
    )
    .unwrap();

    let lexer = kali_lexer::Lexer::new(
        kali_common::FileId::new(0),
        r#"for (const value of Object.freeze((true && globalThis["Array"].from))([1, 2])) { console.log(value); }"#
            .to_string(),
    );
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let Statement::ForOfStatement(ForOfStatement { right, .. }) = &statements[0] else {
        panic!("expected for-of statement");
    };

    let ctx = TypeContext::with_base_path(&source_path);
    let Expression::CallExpression(call) = right else {
        panic!("unexpected right expression: {right:?}");
    };

    assert_eq!(
        ctx.resolve_static_callable_name(&call.callee).as_deref(),
        Some("globalThis.Array.from")
    );
}

#[test]
fn test_resolution_recognizes_nullish_wrapped_bracketed_global_this_array_from_callable_name_in_js_input(
) {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"for (const value of Object.freeze((null ?? globalThis["Array"].from))([1, 2])) { console.log(value); }"#,
    )
    .unwrap();

    let lexer = kali_lexer::Lexer::new(
        kali_common::FileId::new(0),
        r#"for (const value of Object.freeze((null ?? globalThis["Array"].from))([1, 2])) { console.log(value); }"#
            .to_string(),
    );
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let Statement::ForOfStatement(ForOfStatement { right, .. }) = &statements[0] else {
        panic!("expected for-of statement");
    };

    let ctx = TypeContext::with_base_path(&source_path);
    let Expression::CallExpression(call) = right else {
        panic!("unexpected right expression: {right:?}");
    };

    assert_eq!(
        ctx.resolve_static_callable_name(&call.callee).as_deref(),
        Some("globalThis.Array.from")
    );
}

#[test]
fn test_resolution_recognizes_nullish_wrapped_fully_bracketed_global_this_array_from_callable_name_in_js_input(
) {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"for (const value of Object.freeze((null ?? globalThis["Array"]["from"]))([1, 2])) { console.log(value); }"#,
    )
    .unwrap();

    let lexer = kali_lexer::Lexer::new(
        kali_common::FileId::new(0),
        r#"for (const value of Object.freeze((null ?? globalThis["Array"]["from"]))([1, 2])) { console.log(value); }"#
            .to_string(),
    );
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let Statement::ForOfStatement(ForOfStatement { right, .. }) = &statements[0] else {
        panic!("expected for-of statement");
    };

    let ctx = TypeContext::with_base_path(&source_path);
    let Expression::CallExpression(call) = right else {
        panic!("unexpected right expression: {right:?}");
    };

    assert_eq!(
        ctx.resolve_static_callable_name(&call.callee).as_deref(),
        Some("globalThis.Array.from")
    );
}

#[test]
fn test_resolution_recognizes_nullish_wrapped_single_quoted_fully_bracketed_global_this_array_from_callable_name_in_js_input(
) {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"for (const value of Object.freeze((null ?? globalThis['Array']['from']))([1, 2])) { console.log(value); }"#,
    )
    .unwrap();

    let lexer = kali_lexer::Lexer::new(
        kali_common::FileId::new(0),
        r#"for (const value of Object.freeze((null ?? globalThis['Array']['from']))([1, 2])) { console.log(value); }"#
            .to_string(),
    );
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let Statement::ForOfStatement(ForOfStatement { right, .. }) = &statements[0] else {
        panic!("expected for-of statement");
    };

    let ctx = TypeContext::with_base_path(&source_path);
    let Expression::CallExpression(call) = right else {
        panic!("unexpected right expression: {right:?}");
    };

    assert_eq!(
        ctx.resolve_static_callable_name(&call.callee).as_deref(),
        Some("globalThis.Array.from")
    );
}

#[test]
fn test_resolution_recognizes_array_from_callable_name_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "for (const value of Array.from([1, 2])) { console.log(value); }",
    )
    .unwrap();

    let lexer = kali_lexer::Lexer::new(
        kali_common::FileId::new(0),
        "for (const value of Array.from([1, 2])) { console.log(value); }".to_string(),
    );
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let Statement::ForOfStatement(ForOfStatement { right, .. }) = &statements[0] else {
        panic!("expected for-of statement");
    };

    let ctx = TypeContext::with_base_path(&source_path);
    let Expression::CallExpression(call) = right else {
        panic!("unexpected right expression: {right:?}");
    };

    assert_eq!(
        ctx.resolve_static_callable_name(&call.callee).as_deref(),
        Some("Array.from")
    );
}

#[test]
fn test_resolution_recognizes_global_this_array_from_callable_name_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "for (const value of globalThis.Array.from([1, 2])) { console.log(value); }",
    )
    .unwrap();

    let lexer = kali_lexer::Lexer::new(
        kali_common::FileId::new(0),
        "for (const value of globalThis.Array.from([1, 2])) { console.log(value); }".to_string(),
    );
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let Statement::ForOfStatement(ForOfStatement { right, .. }) = &statements[0] else {
        panic!("expected for-of statement");
    };

    let ctx = TypeContext::with_base_path(&source_path);
    let Expression::CallExpression(call) = right else {
        panic!("unexpected right expression: {right:?}");
    };

    assert_eq!(
        ctx.resolve_static_callable_name(&call.callee).as_deref(),
        Some("globalThis.Array.from")
    );
}

#[test]
fn test_resolution_recognizes_bracketed_global_this_array_from_callable_name_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"for (const value of globalThis["Array"].from([1, 2])) { console.log(value); }"#,
    )
    .unwrap();

    let lexer = kali_lexer::Lexer::new(
        kali_common::FileId::new(0),
        r#"for (const value of globalThis["Array"].from([1, 2])) { console.log(value); }"#
            .to_string(),
    );
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let Statement::ForOfStatement(ForOfStatement { right, .. }) = &statements[0] else {
        panic!("expected for-of statement");
    };

    let ctx = TypeContext::with_base_path(&source_path);
    let Expression::CallExpression(call) = right else {
        panic!("unexpected right expression: {right:?}");
    };

    assert_eq!(
        ctx.resolve_static_callable_name(&call.callee).as_deref(),
        Some("globalThis.Array.from")
    );
}

#[test]
fn test_resolution_recognizes_single_quoted_bracketed_global_this_array_from_callable_name_in_js_input(
) {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"for (const value of globalThis['Array'].from([1, 2])) { console.log(value); }"#,
    )
    .unwrap();

    let lexer = kali_lexer::Lexer::new(
        kali_common::FileId::new(0),
        r#"for (const value of globalThis['Array'].from([1, 2])) { console.log(value); }"#
            .to_string(),
    );
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let Statement::ForOfStatement(ForOfStatement { right, .. }) = &statements[0] else {
        panic!("expected for-of statement");
    };

    let ctx = TypeContext::with_base_path(&source_path);
    let Expression::CallExpression(call) = right else {
        panic!("unexpected right expression: {right:?}");
    };

    assert_eq!(
        ctx.resolve_static_callable_name(&call.callee).as_deref(),
        Some("globalThis.Array.from")
    );
}

#[test]
fn test_resolution_recognizes_bracketed_global_this_array_bracket_from_callable_name_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"for (const value of globalThis["Array"]["from"]([1, 2])) { console.log(value); }"#,
    )
    .unwrap();

    let lexer = kali_lexer::Lexer::new(
        kali_common::FileId::new(0),
        r#"for (const value of globalThis["Array"]["from"]([1, 2])) { console.log(value); }"#
            .to_string(),
    );
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let Statement::ForOfStatement(ForOfStatement { right, .. }) = &statements[0] else {
        panic!("expected for-of statement");
    };

    let ctx = TypeContext::with_base_path(&source_path);
    let Expression::CallExpression(call) = right else {
        panic!("unexpected right expression: {right:?}");
    };

    assert_eq!(
        ctx.resolve_static_callable_name(&call.callee).as_deref(),
        Some("globalThis.Array.from")
    );
}

#[test]
fn test_resolution_recognizes_single_quoted_bracketed_global_this_array_bracket_from_callable_name_in_js_input(
) {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"for (const value of globalThis['Array']['from']([1, 2])) { console.log(value); }"#,
    )
    .unwrap();

    let lexer = kali_lexer::Lexer::new(
        kali_common::FileId::new(0),
        r#"for (const value of globalThis['Array']['from']([1, 2])) { console.log(value); }"#
            .to_string(),
    );
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let Statement::ForOfStatement(ForOfStatement { right, .. }) = &statements[0] else {
        panic!("expected for-of statement");
    };

    let ctx = TypeContext::with_base_path(&source_path);
    let Expression::CallExpression(call) = right else {
        panic!("unexpected right expression: {right:?}");
    };

    assert_eq!(
        ctx.resolve_static_callable_name(&call.callee).as_deref(),
        Some("globalThis.Array.from")
    );
}

#[test]
fn test_resolution_recognizes_nullish_wrapped_bracketed_global_this_array_bracket_from_callable_name_in_js_input(
) {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"for (const value of Object.freeze((null ?? globalThis["Array"]["from"]))([1, 2])) { console.log(value); }"#,
    )
    .unwrap();

    let lexer = kali_lexer::Lexer::new(
        kali_common::FileId::new(0),
        r#"for (const value of Object.freeze((null ?? globalThis["Array"]["from"]))([1, 2])) { console.log(value); }"#
            .to_string(),
    );
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let Statement::ForOfStatement(ForOfStatement { right, .. }) = &statements[0] else {
        panic!("expected for-of statement");
    };

    let ctx = TypeContext::with_base_path(&source_path);
    let Expression::CallExpression(call) = right else {
        panic!("unexpected right expression: {right:?}");
    };

    assert_eq!(
        ctx.resolve_static_callable_name(&call.callee).as_deref(),
        Some("globalThis.Array.from")
    );
}

#[test]
fn test_resolution_recognizes_single_quoted_bracketed_array_from_callable_name_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"for (const value of Array['from']([1, 2])) { console.log(value); }"#,
    )
    .unwrap();

    let lexer = kali_lexer::Lexer::new(
        kali_common::FileId::new(0),
        r#"for (const value of Array['from']([1, 2])) { console.log(value); }"#.to_string(),
    );
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let Statement::ForOfStatement(ForOfStatement { right, .. }) = &statements[0] else {
        panic!("expected for-of statement");
    };

    let ctx = TypeContext::with_base_path(&source_path);
    let Expression::CallExpression(call) = right else {
        panic!("unexpected right expression: {right:?}");
    };

    assert_eq!(
        ctx.resolve_static_callable_name(&call.callee).as_deref(),
        Some("Array.from")
    );
}

#[test]
fn test_resolution_supports_for_await_array_from_iteration_in_js_input() {
    assert_resolution_accepts_frozen_iterator_protocol_edge(
        "main.js",
        "for await (const value of Array.from([1, 2])) { console.log(value); }",
    );
}

#[test]
fn test_resolution_supports_for_await_array_from_iteration_in_ts_input() {
    assert_resolution_accepts_frozen_iterator_protocol_edge(
        "main.ts",
        "for await (const value of Array.from([1, 2])) { console.log(value); }",
    );
}

#[test]
fn test_resolution_supports_array_from_new_set_and_new_map_iteration_in_js_input() {
    assert_resolution_accepts_frozen_iterator_protocol_edge(
        "main.js",
        "for (const value of Array.from(new Set([1, 2, 1]))) { console.log(value); }\nfor await (const entry of Array.from(new Map([[1, 2], [1, 3], [4, 5]]))) { console.log(entry[0], entry[1]); }",
    );
}

#[test]
fn test_resolution_supports_array_from_new_set_and_new_map_iteration_in_ts_input() {
    assert_resolution_accepts_frozen_iterator_protocol_edge(
        "main.ts",
        "for (const value of Array.from(new Set([1, 2, 1]))) { console.log(value); }\nfor await (const entry of Array.from(new Map([[1, 2], [1, 3], [4, 5]]))) { console.log(entry[0], entry[1]); }",
    );
}

#[test]
fn test_resolution_supports_frozen_single_quoted_bracketed_array_from_iteration_in_js_input() {
    assert_resolution_accepts_frozen_iterator_protocol_edge(
        "main.js",
        "const values = [1, 2]; for (const value of Object.freeze(globalThis['Array']['from'])(values)) { console.log(value); }",
    );
}

#[test]
fn test_resolution_supports_parenthesized_receiver_frozen_bracketed_array_from_iteration_in_js_input(
) {
    assert_resolution_accepts_frozen_iterator_protocol_edge(
        "main.js",
        "const values = [1, 2]; for (const value of Object.freeze((globalThis[\"Array\"]).from)(values)) { console.log(value); } for (const value of Object.freeze((globalThis['Array']).from)(values)) { console.log(value); }",
    );
}

#[test]
fn test_resolution_supports_parenthesized_frozen_dot_root_array_from_iteration_in_js_input() {
    assert_resolution_accepts_frozen_iterator_protocol_edge(
        "main.js",
        "const values = [1, 2]; for (const value of Object.freeze((globalThis.Array))[\"from\"](values)) { console.log(value); }",
    );
}

#[test]
fn test_resolution_supports_parenthesized_mixed_quoted_bracket_root_array_from_iteration_in_js_input(
) {
    assert_resolution_accepts_frozen_iterator_protocol_edge(
        "main.js",
        r#"const values = [1, 2]; for (const value of Object.freeze((globalThis["Array"])["from"])(values)) { console.log(value); } for (const value of Object.freeze((globalThis['Array'])['from'])(values)) { console.log(value); }"#,
    );
}

#[test]
fn test_resolution_supports_for_of_array_iteration_with_sequence_wrappers_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "let value = 0; for ((0, value) of (0, [(0, 1), (0, 2)])) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(sequence_expression(vec![
                Expression::Literal(LiteralValue::Number(0.0)),
                Expression::Identifier("value".to_string()),
            ])),
            right: sequence_expression(vec![
                Expression::Literal(LiteralValue::Number(0.0)),
                Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            sequence_expression(vec![
                                Expression::Literal(LiteralValue::Number(0.0)),
                                Expression::Literal(LiteralValue::Number(1.0)),
                            ]),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            sequence_expression(vec![
                                Expression::Literal(LiteralValue::Number(0.0)),
                                Expression::Literal(LiteralValue::Number(2.0)),
                            ]),
                        )),
                    ],
                }),
            ]),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("value".to_string())],
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
fn test_resolution_rejects_for_of_non_literal_iterable_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                kali_ast::AssignmentExpression {
                    operator: AssignmentOperator::Assign,
                    left: Expression::Identifier("values".to_string()),
                    right: Expression::ArrayExpression(kali_ast::ArrayExpression {
                        elements: vec![
                            Some(kali_ast::ExpressionOrSpread::Expression(
                                Expression::Literal(LiteralValue::Number(3.0)),
                            )),
                            Some(kali_ast::ExpressionOrSpread::Expression(
                                Expression::Literal(LiteralValue::Number(4.0)),
                            )),
                        ],
                    }),
                },
            ))),
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "item".to_string(),
                    init: None,
                }],
            }),
            right: Expression::Identifier("values".to_string()),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::Identifier("item".to_string())),
                })],
            })),
            is_await: false,
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("literal array"),
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_rejects_for_await_non_literal_iterable_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                kali_ast::AssignmentExpression {
                    operator: AssignmentOperator::Assign,
                    left: Expression::Identifier("values".to_string()),
                    right: Expression::ArrayExpression(kali_ast::ArrayExpression {
                        elements: vec![
                            Some(kali_ast::ExpressionOrSpread::Expression(
                                Expression::Literal(LiteralValue::Number(3.0)),
                            )),
                            Some(kali_ast::ExpressionOrSpread::Expression(
                                Expression::Literal(LiteralValue::Number(4.0)),
                            )),
                        ],
                    }),
                },
            ))),
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "item".to_string(),
                    init: None,
                }],
            }),
            right: Expression::Identifier("values".to_string()),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::Identifier("item".to_string())),
                })],
            })),
            is_await: true,
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("literal array"),
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_for_of_array_iteration_with_parenthesized_binding_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "let value = 0; for ((value) of [1, 2]) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(Expression::ParenthesizedExpression(Box::new(
                kali_ast::ParenthesizedExpression {
                    expression: Box::new(Expression::Identifier("value".to_string())),
                },
            ))),
            right: Expression::ArrayExpression(kali_ast::ArrayExpression {
                elements: vec![
                    Some(kali_ast::ExpressionOrSpread::Expression(
                        Expression::Literal(LiteralValue::Number(1.0)),
                    )),
                    Some(kali_ast::ExpressionOrSpread::Expression(
                        Expression::Literal(LiteralValue::Number(2.0)),
                    )),
                ],
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("value".to_string())],
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
fn test_resolution_supports_for_of_array_iteration_with_parenthesized_binding_in_ts_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "let value = 0; for ((value) of [1, 2]) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(Expression::ParenthesizedExpression(Box::new(
                kali_ast::ParenthesizedExpression {
                    expression: Box::new(Expression::Identifier("value".to_string())),
                },
            ))),
            right: Expression::ArrayExpression(kali_ast::ArrayExpression {
                elements: vec![
                    Some(kali_ast::ExpressionOrSpread::Expression(
                        Expression::Literal(LiteralValue::Number(1.0)),
                    )),
                    Some(kali_ast::ExpressionOrSpread::Expression(
                        Expression::Literal(LiteralValue::Number(2.0)),
                    )),
                ],
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("value".to_string())],
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
fn test_resolution_rejects_for_of_array_iteration_with_identifier_iterable() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const items = [1, 2];\nfor (const value of items) { console.log(value); }\n",
    )
    .unwrap();

    let statements = vec![Statement::ForOfStatement(ForOfStatement {
        left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: None,
            }],
        }),
        right: Expression::Identifier("items".to_string()),
        body: Box::new(Statement::BlockStatement(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("console".to_string()),
                        property: "log".to_string(),
                    })),
                    args: vec![Expression::Identifier("value".to_string())],
                }))),
            })],
        })),
        is_await: false,
    })];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_for_of_array_iteration_with_const_alias_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const values = [1, 2]; for (const value of values) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "value".to_string(),
                    init: None,
                }],
            }),
            right: Expression::Identifier("values".to_string()),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("value".to_string())],
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
fn test_resolution_supports_for_of_array_iteration_with_const_alias_in_ts_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const values = [1, 2]; for (const value of values) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "value".to_string(),
                    init: None,
                }],
            }),
            right: Expression::Identifier("values".to_string()),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("value".to_string())],
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
fn test_resolution_supports_for_of_array_iteration_with_let_binding_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "let values = [1, 2]; for (const value of values) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "value".to_string(),
                    init: None,
                }],
            }),
            right: Expression::Identifier("values".to_string()),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("value".to_string())],
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
fn test_resolution_rejects_for_of_array_iteration_with_let_binding_rebound_before_use_in_js_input()
{
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "let values = [1, 2]; values = [3, 4]; for (const value of values) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                AssignmentExpression {
                    operator: AssignmentOperator::Assign,
                    left: Expression::Identifier("values".to_string()),
                    right: Expression::ArrayExpression(kali_ast::ArrayExpression {
                        elements: vec![
                            Some(kali_ast::ExpressionOrSpread::Expression(
                                Expression::Literal(LiteralValue::Number(3.0)),
                            )),
                            Some(kali_ast::ExpressionOrSpread::Expression(
                                Expression::Literal(LiteralValue::Number(4.0)),
                            )),
                        ],
                    }),
                },
            ))),
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "value".to_string(),
                    init: None,
                }],
            }),
            right: Expression::Identifier("values".to_string()),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("value".to_string())],
                    }))),
                })],
            })),
            is_await: false,
        }),
    ];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_for_of_array_iteration_with_const_numeric_alias_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = 1; const alias = value; for (const item of [alias]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
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
fn test_resolution_supports_for_of_array_iteration_with_const_numeric_alias_in_ts_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const value = 1; const alias = value; for (const item of [alias]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
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
fn test_resolution_supports_for_of_array_iteration_with_decorated_wrappers_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "let item = 0; for ((item) of [1, 2]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "item".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::Identifier("item".to_string())),
                    },
                ))),
            })),
            right: Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    1.0,
                                ))),
                            }),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    2.0,
                                ))),
                            }),
                        )),
                    ],
                })),
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
fn test_resolution_supports_for_of_array_iteration_with_decorated_wrappers_in_ts_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "let item = 0; for ((item) of [1, 2]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "item".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::Identifier("item".to_string())),
                    },
                ))),
            })),
            right: Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    1.0,
                                ))),
                            }),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    2.0,
                                ))),
                            }),
                        )),
                    ],
                })),
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
fn test_resolution_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_js_input(
) {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const values = [1, 2]; for (const item of [...values]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
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
                elements: vec![Some(kali_ast::ExpressionOrSpread::Spread(
                    kali_ast::SpreadElement {
                        argument: Expression::Identifier("values".to_string()),
                    },
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
fn test_resolution_supports_for_of_array_iteration_with_decorated_spread_targets_in_ts_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
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
                elements: vec![Some(kali_ast::ExpressionOrSpread::Spread(
                    kali_ast::SpreadElement {
                        argument: Expression::DecoratedExpression(DecoratedExpression {
                            expression: Box::new(Expression::Identifier("values".to_string())),
                        }),
                    },
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
fn test_resolution_supports_for_of_array_iteration_with_decorated_spread_targets_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
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
                elements: vec![Some(kali_ast::ExpressionOrSpread::Spread(
                    kali_ast::SpreadElement {
                        argument: Expression::DecoratedExpression(DecoratedExpression {
                            expression: Box::new(Expression::Identifier("values".to_string())),
                        }),
                    },
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
fn test_resolution_supports_for_await_of_array_iteration_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "for await (const value of [1, 2]) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![Statement::ForOfStatement(ForOfStatement {
        left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: None,
            }],
        }),
        right: Expression::ArrayExpression(kali_ast::ArrayExpression {
            elements: vec![
                Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::Literal(LiteralValue::Number(1.0)),
                )),
                Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::Literal(LiteralValue::Number(2.0)),
                )),
            ],
        }),
        body: Box::new(Statement::BlockStatement(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("console".to_string()),
                        property: "log".to_string(),
                    })),
                    args: vec![Expression::Identifier("value".to_string())],
                }))),
            })],
        })),
        is_await: true,
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
fn test_resolution_supports_for_await_of_array_iteration_with_sequence_wrappers_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "let item = 0; for await ((0, item) of (0, [(0, 1), (0, 2)])) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "item".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(sequence_expression(vec![
                Expression::Literal(LiteralValue::Number(0.0)),
                Expression::Identifier("item".to_string()),
            ])),
            right: sequence_expression(vec![
                Expression::Literal(LiteralValue::Number(0.0)),
                Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            sequence_expression(vec![
                                Expression::Literal(LiteralValue::Number(0.0)),
                                Expression::Literal(LiteralValue::Number(1.0)),
                            ]),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            sequence_expression(vec![
                                Expression::Literal(LiteralValue::Number(0.0)),
                                Expression::Literal(LiteralValue::Number(2.0)),
                            ]),
                        )),
                    ],
                }),
            ]),
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
fn test_resolution_supports_for_await_of_array_iteration_with_decorated_wrappers_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "let item = 0; for await ((item) of [1, 2]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "item".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::Identifier("item".to_string())),
                    },
                ))),
            })),
            right: Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    1.0,
                                ))),
                            }),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    2.0,
                                ))),
                            }),
                        )),
                    ],
                })),
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
fn test_resolution_supports_for_await_of_array_iteration_with_decorated_wrappers_in_jsx_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.jsx");
    fs::write(
        &source_path,
        "let item = 0; for await ((item) of [1, 2]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "item".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::Identifier("item".to_string())),
                    },
                ))),
            })),
            right: Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    1.0,
                                ))),
                            }),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    2.0,
                                ))),
                            }),
                        )),
                    ],
                })),
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
fn test_resolution_supports_for_await_of_array_iteration_with_decorated_wrappers_in_ts_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "let item = 0; for await ((item) of [1, 2]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "item".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::Identifier("item".to_string())),
                    },
                ))),
            })),
            right: Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    1.0,
                                ))),
                            }),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    2.0,
                                ))),
                            }),
                        )),
                    ],
                })),
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
fn test_resolution_supports_for_await_of_array_iteration_with_decorated_wrappers_in_tsx_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.tsx");
    fs::write(
        &source_path,
        "let item = 0; for await ((item) of [1, 2]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "item".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::Identifier("item".to_string())),
                    },
                ))),
            })),
            right: Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    1.0,
                                ))),
                            }),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    2.0,
                                ))),
                            }),
                        )),
                    ],
                })),
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
fn test_resolution_supports_for_await_of_await_wrapped_iterables_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "for await (const value of await [1, 2]) { console.log(value); }",
    )
    .unwrap();

    let lexer = kali_lexer::Lexer::new(
        kali_common::FileId::new(0),
        "for await (const value of await [1, 2]) { console.log(value); }".to_string(),
    );
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_for_await_of_array_iteration_with_decorated_spread_targets_in_js_input()
{
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
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
                elements: vec![Some(kali_ast::ExpressionOrSpread::Spread(
                    kali_ast::SpreadElement {
                        argument: Expression::DecoratedExpression(DecoratedExpression {
                            expression: Box::new(Expression::Identifier("values".to_string())),
                        }),
                    },
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
fn test_resolution_supports_for_await_of_array_iteration_with_decorated_spread_targets_in_ts_input()
{
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
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
                elements: vec![Some(kali_ast::ExpressionOrSpread::Spread(
                    kali_ast::SpreadElement {
                        argument: Expression::DecoratedExpression(DecoratedExpression {
                            expression: Box::new(Expression::Identifier("values".to_string())),
                        }),
                    },
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
fn test_resolution_supports_for_await_of_array_iteration_with_const_alias_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = 1; const alias = value; for await (const item of [alias]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
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
fn test_resolution_supports_for_await_of_array_iteration_with_const_alias_in_ts_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const value = 1; const alias = value; for await (const item of [alias]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
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
fn test_resolution_allows_static_array_reduce_without_initial_value_on_non_empty_numeric_literals()
{
    for method in ["reduce", "reduceRight"] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        let source = format!(
            "const result = [1, 2, 3].{method}((accumulator, value) => accumulator + value);"
        );
        fs::write(&source_path, &source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source);
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_rejects_static_array_reduce_without_initial_value_on_empty_literals() {
    for method in ["reduce", "reduceRight"] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        let source =
            format!("const result = [].{method}((accumulator, value) => accumulator + value);");
        fs::write(&source_path, &source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert_eq!(
            result.diagnostics.len(),
            1,
            "unexpected diagnostics: {:?}",
            result.diagnostics
        );
        assert_eq!(
            result.diagnostics[0].code,
            Some(e5::FEATURE_UNAVAILABLE as u32)
        );
        assert!(
            result.diagnostics[0]
                .message
                .contains(&format!("array callback method '{method}'")),
            "unexpected diagnostics: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_allows_static_predicate_array_filter_in_non_browser_surface() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    let source =
        "for (const value of [1, 2, 3].filter((value) => value > 1)) { console.log(value); }"
            .to_string();
    fs::write(&source_path, &source).unwrap();

    let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source);
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_allows_static_array_search_family_in_non_browser_surface() {
    for source in [
        "const result = [0, 1, 2].includes(1);",
        "const needle = 1; const result = [0, 1, 2].indexOf(needle, 1);",
        "const result = [0, 1, 2, 1].lastIndexOf(1, 2);",
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
fn test_resolution_allows_static_array_join_in_non_browser_surface() {
    for source in [
        "const result = [0, true, null, 'x'].join('-');",
        "const separator = '-'; const result = [0, 1, 2].join(separator);",
        "const result = ['a', 'b'].join();",
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
fn test_resolution_rejects_dynamic_array_join_in_non_browser_surface() {
    let source = "function join(separator) { return [0, 1, 2].join(separator); }";
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
                && diag.message.contains("Array.prototype.join")),
        "expected Array.prototype.join feature gate for {source}, got {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_allows_static_array_concat_in_non_browser_surface() {
    for source in [
        "const result = [0, 1].concat([2, 3]);",
        "const values = [0, 1]; const result = values.concat(2, [3]);",
        "const result = [0].concat(Object.freeze(1));",
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
fn test_resolution_rejects_dynamic_array_concat_in_non_browser_surface() {
    for source in [
        "function join(values) { return values.concat([1]); }",
        "function join(value) { return [0].concat(value); }",
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
                    && diag.message.contains("Array.prototype.concat")),
            "expected Array.prototype.concat feature gate for {source}, got {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_allows_static_array_at_in_non_browser_surface() {
    for source in [
        "const result = [0, 1, 2].at(1);",
        "const result = [0, 1, 2].at(-1);",
        "const result = [0, 1, 2].at(3);",
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
fn test_resolution_rejects_dynamic_array_at_in_non_browser_surface() {
    for source in [
        "function get(index) { return [0, 1, 2].at(index); }",
        "function get(values) { return values.at(1); }",
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
                    && diag.message.contains("Array.prototype.at")),
            "expected Array.prototype.at feature gate for {source}, got {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_rejects_dynamic_array_search_family_in_non_browser_surface() {
    for source in [
        "function has(needle) { return [0, 1, 2].includes(needle); }",
        "function at(from) { return [0, 1, 2].indexOf(1, from); }",
        "function find(values) { return values.lastIndexOf(1); }",
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
                    && diag.message.contains("array search method")),
            "expected array search feature gate for {source}, got {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_allows_array_find_family_in_non_browser_surface() {
    for (method, callback) in [
        ("find", "(value) => value > 1"),
        ("findIndex", "(value) => value > 1"),
        ("findLast", "(value) => value > 1"),
        ("findLastIndex", "(value) => value > 1"),
        ("find", "(value) => value === 2"),
        ("findIndex", "(value) => value !== 1"),
        ("findLast", "(value) => value === 2"),
        ("findLastIndex", "(value) => value !== 1"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join("main.js");
        let source = format!("const result = [0, 1, 2, 3].{method}({callback});");
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(
            kali_common::FileId::new(0),
            fs::read_to_string(&source_path).unwrap(),
        );
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {method}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_allows_identity_array_map_in_non_browser_surface() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    let source = "const result = [1, 2, 3].map((value) => value);".to_string();
    fs::write(&source_path, &source).unwrap();

    let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source);
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_allows_identity_array_some_in_non_browser_surface() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    let source = "const result = [0, 1].some((value) => value);".to_string();
    fs::write(&source_path, &source).unwrap();

    let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source);
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_allows_identity_array_every_in_non_browser_surface() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    let source = "const result = [1, 2].every((value) => value);".to_string();
    fs::write(&source_path, &source).unwrap();

    let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source);
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_allows_number_predicate_array_some_every_in_non_browser_surface() {
    for source in [
        "const result = [0, 1, 2].some((value) => value > 1);",
        "const result = [2, 3].every((value) => value > 1);",
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
            "unexpected diagnostics: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_allows_truthy_identity_array_filter_in_non_browser_surface() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    let source = "const result = [1, 2].filter((value) => value);".to_string();
    fs::write(&source_path, &source).unwrap();

    let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source);
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_allows_identity_array_flat_map_in_non_browser_surface() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    let source = "const result = [1, 2].flatMap((value) => [value]);".to_string();
    fs::write(&source_path, &source).unwrap();

    let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source);
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}
