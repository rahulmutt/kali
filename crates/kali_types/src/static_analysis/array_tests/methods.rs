use super::*;

#[test]
fn test_resolution_recognizes_bracketed_global_this_array_bracket_from_callable_name_in_js_input() {
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
fn test_resolution_allows_static_array_reduce_without_initial_value_on_non_empty_numeric_literals()
{
    for method in ["reduce", "reduceRight"] {
        let dir = fixtures::tempdir();
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
        let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
fn test_resolution_allows_static_array_join_in_non_browser_surface() {
    for source in [
        "const result = [0, true, null, 'x'].join('-');",
        "const separator = '-'; const result = [0, 1, 2].join(separator);",
        "const result = ['a', 'b'].join();",
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
fn test_resolution_rejects_dynamic_array_join_in_non_browser_surface() {
    let source = "function join(separator) { return [0, 1, 2].join(separator); }";
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
fn test_resolution_rejects_dynamic_array_concat_in_non_browser_surface() {
    for source in [
        "function join(values) { return values.concat([1]); }",
        "function join(value) { return [0].concat(value); }",
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
fn test_resolution_rejects_dynamic_array_at_in_non_browser_surface() {
    for source in [
        "function get(index) { return [0, 1, 2].at(index); }",
        "function get(values) { return values.at(1); }",
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
        let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
            "unexpected diagnostics: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_allows_truthy_identity_array_filter_in_non_browser_surface() {
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
