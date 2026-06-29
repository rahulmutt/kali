use super::*;

#[test]
fn test_resolution_accepts_new_set_iteration_target_in_js_input() {
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
