use super::*;

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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
fn test_resolution_recognizes_single_quoted_bracketed_array_from_callable_name_in_js_input() {
    let dir = fixtures::tempdir();
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
