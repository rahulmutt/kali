
use super::*;
use kali_ast::{
    BinaryExpression, LiteralValue, ParenthesizedExpression, TypeAliasDeclaration,
    VariableDeclarator,
};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_scope_creation() {
    let scope = Scope::new(ScopeType::Global, None);
    assert_eq!(scope.scope_type, ScopeType::Global);
    assert!(scope.parent.is_none());
}

#[test]
fn test_scope_binding() {
    let mut scope = Scope::new(ScopeType::Module, None);
    scope.bind("x", NodeId::new(1));
    scope.bind("y", NodeId::new(2));

    assert!(scope.contains("x"));
    assert!(scope.contains("y"));
    assert!(!scope.contains("z"));
}

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
fn test_type_annotation_resolution_accepts_known_names() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "Foo".to_string(),
            type_params: vec![],
            type_annotation: "string".to_string(),
        }),
        Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "Box".to_string(),
            type_params: vec![],
            type_annotation: "Foo | Array<string>".to_string(),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_type_annotation_resolution_reports_unknown_names() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::TypeAliasDeclaration(TypeAliasDeclaration {
        name: "Box".to_string(),
        type_params: vec![],
        type_annotation: "Missing | string".to_string(),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.code == Some(e3::UNDEFINED_IDENTIFIER as u32)));
}

#[test]
fn test_type_checker_collects_annotation_diagnostics() {
    let mut checker = TypeChecker::new();
    checker.check_type_annotation(NodeId::new(1), "Missing | string");

    let diagnostics = checker.typecheck(NodeId::new(0));
    assert!(diagnostics
        .iter()
        .any(|diag| diag.code == Some(e3::UNDEFINED_IDENTIFIER as u32)));
}

#[test]
fn test_type_checker_typecheck_drains_pending_context_diagnostics() {
    let mut checker = TypeChecker::new();
    checker
        .context
        .resolve_type_annotation_text("Missing | string");

    let diagnostics = checker.typecheck(NodeId::new(0));
    assert!(diagnostics
        .iter()
        .any(|diag| diag.code == Some(e3::UNDEFINED_IDENTIFIER as u32)));
    assert!(checker.context.diagnostics().is_empty());
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
fn test_resolution_reports_duplicate_bindings() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::BlockStatement(BlockStatement {
        body: vec![
            Statement::VariableDeclaration(VariableDeclaration {
                kind: "let".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "x".to_string(),
                    init: None,
                }],
            }),
            Statement::VariableDeclaration(VariableDeclaration {
                kind: "let".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "x".to_string(),
                    init: None,
                }],
            }),
        ],
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.code == Some(e3::DUPLICATE_BINDING as u32)));
}

#[test]
fn test_resolution_reports_missing_imports() {
    let mut ctx = TypeContext::with_base_path(".");
    let statements = vec![Statement::ImportDeclaration(ImportDeclaration {
        specifiers: vec![ImportSpecifier::Default("value".to_string())],
        source: "./definitely-missing-file.ts".to_string(),
    })];

    let result = ctx.resolve_statements_at_path(Some("."), &statements);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::IMPORT_NOT_FOUND as u32)
    );
}

#[test]
fn test_resolution_allows_browser_file_reader_global() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::Identifier("FileReader".to_string())),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_allows_browser_stub_globals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("FormData".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("URLSearchParams".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("WebSocket".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("Worker".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("BroadcastChannel".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("indexedDB".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("navigator".to_string())),
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
fn test_resolution_allows_node_builtin_imports_in_node_context() {
    let mut ctx = TypeContext::with_base_path_and_api_surface(".", "node");
    assert!(ctx.is_defined("process"));

    let statements = vec![Statement::ImportDeclaration(ImportDeclaration {
        specifiers: vec![ImportSpecifier::Default("fs".to_string())],
        source: "node:fs/promises".to_string(),
    })];

    let result = ctx.resolve_statements_at_path(Some("."), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_rejects_node_builtin_imports_outside_node_context() {
    let mut ctx = TypeContext::with_base_path(".");
    let statements = vec![Statement::ImportDeclaration(ImportDeclaration {
        specifiers: vec![ImportSpecifier::Default("fs".to_string())],
        source: "node:fs/promises".to_string(),
    })];

    let result = ctx.resolve_statements_at_path(Some("."), &statements);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::IMPORT_NOT_FOUND as u32)
    );
}

#[test]
fn test_resolution_allows_static_dynamic_import_targets() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(dir.path().join("lazy.ts"), "export const lazy = 7;").unwrap();
    fs::write(&source_path, "const lazy = import(\"./\" + \"lazy.ts\");").unwrap();

    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
            source: Expression::BinaryExpression(Box::new(BinaryExpression {
                operator: "+".to_string(),
                left: Expression::Literal(LiteralValue::String("./".to_string())),
                right: Expression::Literal(LiteralValue::String("lazy.ts".to_string())),
            })),
        }))),
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
fn test_resolution_allows_const_bound_dynamic_import_targets() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(dir.path().join("lazy.ts"), "export const lazy = 7;").unwrap();
    fs::write(
        &source_path,
        "const name = \"lazy.ts\"; const root = \"./\"; import(root + name);",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "name".to_string(),
                init: Some(Expression::Literal(LiteralValue::String(
                    "lazy.ts".to_string(),
                ))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "root".to_string(),
                init: Some(Expression::Literal(LiteralValue::String("./".to_string()))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
                source: Expression::BinaryExpression(Box::new(BinaryExpression {
                    operator: "+".to_string(),
                    left: Expression::Identifier("root".to_string()),
                    right: Expression::Identifier("name".to_string()),
                })),
            }))),
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
fn test_resolution_allows_parenthesized_dynamic_import_targets() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(dir.path().join("lazy.ts"), "export const lazy = 7;").unwrap();
    fs::write(
        &source_path,
        "const name = \"lazy.ts\"; const root = \"./\"; import((root + name));",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "name".to_string(),
                init: Some(Expression::Literal(LiteralValue::String(
                    "lazy.ts".to_string(),
                ))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "root".to_string(),
                init: Some(Expression::Literal(LiteralValue::String("./".to_string()))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
                source: Expression::ParenthesizedExpression(Box::new(ParenthesizedExpression {
                    expression: Box::new(Expression::BinaryExpression(Box::new(
                        BinaryExpression {
                            operator: "+".to_string(),
                            left: Expression::Identifier("root".to_string()),
                            right: Expression::Identifier("name".to_string()),
                        },
                    ))),
                })),
            }))),
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
fn test_resolution_reports_unknown_dynamic_import_targets() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const name = \"lazy.ts\"; import(\"./\" + name);",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "name".to_string(),
                init: Some(Expression::Literal(LiteralValue::String(
                    "lazy.ts".to_string(),
                ))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
                source: Expression::BinaryExpression(Box::new(BinaryExpression {
                    operator: "+".to_string(),
                    left: Expression::Literal(LiteralValue::String("./".to_string())),
                    right: Expression::Identifier("name".to_string()),
                })),
            }))),
        }),
    ];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e4::DYNAMIC_IMPORT_NOT_IN_LINKED_GRAPH as u32)
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
