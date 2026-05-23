use std::fs;

use kali_parser::Parser;
use kali_types::TypeContext;
use tempfile::tempdir;

#[test]
fn bracketed_object_enumeration_aliases_are_accepted_in_js_like_input() {
    let source = r#"async function main() {
    const obj = Object.fromEntries([["b", 1], ["a", 2]]);
    const frozenKeys = Object.freeze(Object["keys"])(obj);
    const frozenValues = Object.freeze(Object["values"])(obj);
    for (const key of frozenKeys) {
        console.log(key);
    }
    for (const value of frozenValues) {
        console.log(value);
    }
}
main();
"#;

    for extension in ["js", "jsx", "ts", "tsx"] {
        let dir = tempdir().unwrap();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {extension}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn parenthesized_object_enumeration_aliases_are_accepted_in_js_like_input() {
    let source = r#"async function main() {
    const obj = Object.fromEntries([["b", 1], ["a", 2]]);
    const frozenKeys = Object.freeze((globalThis.Object.keys))(obj);
    const frozenValues = Object.freeze((globalThis.Object.values))(obj);
    const frozenEntries = Object.freeze((globalThis.Object.entries))(obj);
    for (const key of frozenKeys) {
        console.log(key);
    }
    for (const value of frozenValues) {
        console.log(value);
    }
    for (const entry of frozenEntries) {
        console.log(entry[0]);
        console.log(entry[1]);
    }
}
main();
"#;

    for extension in ["js", "jsx", "ts", "tsx"] {
        let dir = tempdir().unwrap();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {extension}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn mixed_quoted_object_enumeration_aliases_are_accepted_in_js_like_input() {
    let source = r#"async function main() {
    const obj = Object.fromEntries([["b", 1], ["a", 2]]);
    const frozenKeys = Object.freeze(globalThis["Object"].keys)(obj);
    const frozenQuotedKeys = Object.freeze(globalThis["Object"]['keys'])(obj);
    const frozenMixedKeys = Object.freeze(globalThis['Object']["keys"])(obj);
    const frozenValues = Object.freeze(globalThis['Object']["values"])(obj);
    const frozenEntries = Object.freeze(globalThis['Object']['entries'])(obj);
    const frozenReflectOwnKeys = Object.freeze(globalThis["Reflect"].ownKeys)(obj);
    const frozenReflectQuotedOwnKeys = Object.freeze(globalThis["Reflect"]['ownKeys'])(obj);
    const frozenReflectMixedOwnKeys = Object.freeze(globalThis['Reflect']["ownKeys"])(obj);
    for (const key of frozenKeys) {
        console.log(key);
    }
    for (const key of frozenQuotedKeys) {
        console.log(key);
    }
    for (const key of frozenMixedKeys) {
        console.log(key);
    }
    for (const value of frozenValues) {
        console.log(value);
    }
    for (const entry of frozenEntries) {
        console.log(entry[0]);
        console.log(entry[1]);
    }
    for (const key of frozenReflectOwnKeys) {
        console.log(key);
    }
    for (const key of frozenReflectQuotedOwnKeys) {
        console.log(key);
    }
    for (const key of frozenReflectMixedOwnKeys) {
        console.log(key);
    }
}
main();
"#;

    for extension in ["js", "jsx", "ts", "tsx"] {
        let dir = tempdir().unwrap();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {extension}: {:?}",
            result.diagnostics
        );
    }
}
