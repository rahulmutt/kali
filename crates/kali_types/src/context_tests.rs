use crate::*;
use kali_ast::{Expression, ExpressionStatement, LiteralValue, VariableDeclaration, VariableDeclarator};
use kali_error::_error_codes::e3;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_type_context() {
    let mut ctx = TypeContext::new();
    assert!(ctx.is_defined("Kali"));
    assert!(!ctx.is_defined("x"));

    let _module = ctx.push_scope(ScopeType::Module);
    let binding = ctx.define("x");
    assert_eq!(binding.name(), "x");
    assert!(ctx.resolve_name("x").is_some());
}

#[test]
fn test_resolution_finds_bound_names() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::VariableDeclaration(VariableDeclaration {
        kind: "let".to_string(),
        declarations: vec![VariableDeclarator {
            id: "value".to_string(),
            init: Some(Expression::Literal(LiteralValue::Number(1.0))),
        }],
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert!(result.scopes.values().any(|scope| scope.contains("value")));
}

#[test]
fn test_resolution_reports_unresolved_identifiers() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::Identifier("missing".to_string())),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::UNDEFINED_IDENTIFIER as u32)
    );
}

#[test]
fn test_resolution_uses_project_root_for_materialized_packages() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "devDependencies": {
    "@types/lodash": "1.0.0"
  }
}"#,
    )
    .unwrap();

    let src_dir = dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let source_path = src_dir.join("main.ts");
    fs::write(&source_path, "import lodash from 'lodash';\n").unwrap();

    let types_dir = dir.path().join("node_modules/@types/lodash");
    fs::create_dir_all(&types_dir).unwrap();
    fs::write(
        types_dir.join("package.json"),
        r#"{"name":"@types/lodash","types":"index.d.ts"}"#,
    )
    .unwrap();
    fs::write(types_dir.join("index.d.ts"), "declare const _: number;").unwrap();

    let mut ctx = TypeContext::with_base_path(&source_path);
    let statements = vec![Statement::ImportDeclaration(ImportDeclaration {
        specifiers: vec![ImportSpecifier::Default("lodash".to_string())],
        source: "lodash".to_string(),
    })];

    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}
