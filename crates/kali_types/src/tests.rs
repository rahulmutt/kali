use super::*;
use kali_ast::{
    ArrowFunctionExpression, AssignmentExpression, AssignmentOperator, BinaryExpression,
    BlockStatement, CallExpression, ExportDefaultDeclaration, ExportNamedDeclaration,
    ExportSpecifier, Expression, ExpressionStatement, ForOfLefthand, ForOfStatement,
    FunctionDeclaration, FunctionExpression, LiteralValue, MemberExpression, ObjectExpression,
    ObjectProperty, ObjectPropertyKind, ParenthesizedExpression, PropertyName, SatisfiesExpression,
    TypeAliasDeclaration, TypeAssertion, VariableDeclaration, VariableDeclarator, YieldExpression,
};
use kali_error::_error_codes::{e3, e5};
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
fn test_resolution_accepts_type_assertion_and_satisfies_with_known_type_names() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "Foo".to_string(),
            type_params: vec![],
            type_annotation: "string".to_string(),
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::TypeAssertion(Box::new(TypeAssertion {
                type_name: "Foo".to_string(),
                expression: Box::new(Expression::Identifier("value".to_string())),
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::SatisfiesExpression(Box::new(
                SatisfiesExpression {
                    type_name: "Foo".to_string(),
                    expression: Box::new(Expression::Identifier("value".to_string())),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_resolution_accepts_arrow_function_return_type_annotations() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "Foo".to_string(),
            type_params: vec![],
            type_annotation: "string".to_string(),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                ArrowFunctionExpression {
                    params: vec![FunctionParam {
                        name: "value".to_string(),
                    }],
                    body: Expression::Identifier("value".to_string()),
                    is_async: false,
                    returnType: Some("Foo".to_string()),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_resolution_accepts_async_arrow_function_return_type_annotations() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "Foo".to_string(),
            type_params: vec![],
            type_annotation: "string".to_string(),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                ArrowFunctionExpression {
                    params: vec![FunctionParam {
                        name: "value".to_string(),
                    }],
                    body: Expression::Identifier("value".to_string()),
                    is_async: true,
                    returnType: Some("Foo".to_string()),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_resolution_reports_unknown_type_names_in_type_assertion_and_satisfies_expressions() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::TypeAssertion(Box::new(TypeAssertion {
                type_name: "Missing".to_string(),
                expression: Box::new(Expression::Identifier("value".to_string())),
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::SatisfiesExpression(Box::new(
                SatisfiesExpression {
                    type_name: "Missing".to_string(),
                    expression: Box::new(Expression::Identifier("value".to_string())),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.code == Some(e3::UNDEFINED_IDENTIFIER as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Missing")));
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
fn test_type_annotation_resolution_reports_unknown_names_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "const value = 1;").unwrap();

    let mut ctx = TypeContext::with_base_path(&source_path);
    let statements = vec![Statement::TypeAliasDeclaration(TypeAliasDeclaration {
        name: "Box".to_string(),
        type_params: vec![],
        type_annotation: "Missing | string".to_string(),
    })];

    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.code == Some(e3::UNDEFINED_IDENTIFIER as u32)));
}

#[test]
fn test_type_annotation_resolution_reports_unknown_names_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, "const value = 1;").unwrap();

        let mut ctx = TypeContext::with_base_path(&source_path);
        let statements = vec![Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "Box".to_string(),
            type_params: vec![],
            type_annotation: "Missing | string".to_string(),
        })];

        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.code == Some(e3::UNDEFINED_IDENTIFIER as u32)),
            "{extension}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_type_annotation_resolution_accepts_known_names_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "const value = 1;").unwrap();

    let mut ctx = TypeContext::with_base_path(&source_path);
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

    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_type_annotation_resolution_accepts_nested_known_names_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "const value = 1;").unwrap();

    let mut ctx = TypeContext::with_base_path(&source_path);
    let statements = vec![
        Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "Foo".to_string(),
            type_params: vec![],
            type_annotation: "string".to_string(),
        }),
        Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "Box".to_string(),
            type_params: vec![],
            type_annotation: "Promise<Array<Foo>>".to_string(),
        }),
    ];

    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_type_annotation_resolution_accepts_known_names_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().unwrap();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, "const value = 1;").unwrap();

        let mut ctx = TypeContext::with_base_path(&source_path);
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

        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "{extension}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_type_annotation_resolution_accepts_deeper_nested_known_names_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "const value = 1;").unwrap();

    let mut ctx = TypeContext::with_base_path(&source_path);
    let statements = vec![
        Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "Foo".to_string(),
            type_params: vec![],
            type_annotation: "string".to_string(),
        }),
        Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "Box".to_string(),
            type_params: vec![],
            type_annotation: "Promise<Array<Promise<Foo>>>".to_string(),
        }),
    ];

    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_type_annotation_resolution_accepts_deeper_nested_known_names_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().unwrap();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, "const value = 1;").unwrap();

        let mut ctx = TypeContext::with_base_path(&source_path);
        let statements = vec![
            Statement::TypeAliasDeclaration(TypeAliasDeclaration {
                name: "Foo".to_string(),
                type_params: vec![],
                type_annotation: "string".to_string(),
            }),
            Statement::TypeAliasDeclaration(TypeAliasDeclaration {
                name: "Box".to_string(),
                type_params: vec![],
                type_annotation: "Promise<Array<Promise<Foo>>>".to_string(),
            }),
        ];

        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "{extension}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_type_annotation_resolution_accepts_mixed_union_nested_known_names_in_js_jsx_and_tsx_input()
{
    for extension in ["js", "jsx", "tsx"] {
        let dir = tempdir().unwrap();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, "const value = 1;").unwrap();

        let mut ctx = TypeContext::with_base_path(&source_path);
        let statements = vec![
            Statement::TypeAliasDeclaration(TypeAliasDeclaration {
                name: "Foo".to_string(),
                type_params: vec![],
                type_annotation: "string".to_string(),
            }),
            Statement::TypeAliasDeclaration(TypeAliasDeclaration {
                name: "Bar".to_string(),
                type_params: vec![],
                type_annotation: "number".to_string(),
            }),
            Statement::TypeAliasDeclaration(TypeAliasDeclaration {
                name: "Box".to_string(),
                type_params: vec![],
                type_annotation: "Promise<Array<Foo | Bar>>".to_string(),
            }),
        ];

        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "{extension}: {:?}",
            result.diagnostics
        );
    }
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
fn test_resolution_reports_unresolved_public_exports() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExportNamed(ExportNamedDeclaration {
        specifiers: vec![ExportSpecifier {
            local: "missing".to_string(),
            exported: "missing".to_string(),
        }],
        source: None,
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::UNDEFINED_IDENTIFIER as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("missing"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_unresolved_public_exports_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "export { missing };").unwrap();

    let statements = vec![Statement::ExportNamed(ExportNamedDeclaration {
        specifiers: vec![ExportSpecifier {
            local: "missing".to_string(),
            exported: "missing".to_string(),
        }],
        source: None,
    })];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::UNDEFINED_IDENTIFIER as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("missing"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_unresolved_public_exports_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().unwrap();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, "export { missing };").unwrap();

        let statements = vec![Statement::ExportNamed(ExportNamedDeclaration {
            specifiers: vec![ExportSpecifier {
                local: "missing".to_string(),
                exported: "missing".to_string(),
            }],
            source: None,
        })];

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].code,
            Some(e3::UNDEFINED_IDENTIFIER as u32)
        );
        assert!(
            result.diagnostics[0].message.contains("missing"),
            "unexpected diagnostics for {extension}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_reports_unresolved_public_export_aliases_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "export { missing as renamed };").unwrap();

    let statements = vec![Statement::ExportNamed(ExportNamedDeclaration {
        specifiers: vec![ExportSpecifier {
            local: "missing".to_string(),
            exported: "renamed".to_string(),
        }],
        source: None,
    })];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::UNDEFINED_IDENTIFIER as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("missing"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_unresolved_default_exports_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "export default missing;").unwrap();

    let statements = vec![Statement::ExportDefault(
        ExportDefaultDeclaration::Expression(Expression::Identifier("missing".to_string())),
    )];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::UNDEFINED_IDENTIFIER as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("missing"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_unresolved_default_exports_in_ts_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default missing;").unwrap();

    let statements = vec![Statement::ExportDefault(
        ExportDefaultDeclaration::Expression(Expression::Identifier("missing".to_string())),
    )];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::UNDEFINED_IDENTIFIER as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("missing"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_unresolved_default_export_aliases_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "export { missing as default };").unwrap();

    let statements = vec![Statement::ExportNamed(ExportNamedDeclaration {
        specifiers: vec![ExportSpecifier {
            local: "missing".to_string(),
            exported: "default".to_string(),
        }],
        source: None,
    })];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::UNDEFINED_IDENTIFIER as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("missing"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_unresolved_default_export_aliases_in_ts_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export { missing as default };").unwrap();

    let statements = vec![Statement::ExportNamed(ExportNamedDeclaration {
        specifiers: vec![ExportSpecifier {
            local: "missing".to_string(),
            exported: "default".to_string(),
        }],
        source: None,
    })];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::UNDEFINED_IDENTIFIER as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("missing"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_unresolved_default_export_aliases_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().unwrap();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, "export { missing as default };").unwrap();

        let statements = vec![Statement::ExportNamed(ExportNamedDeclaration {
            specifiers: vec![ExportSpecifier {
                local: "missing".to_string(),
                exported: "default".to_string(),
            }],
            source: None,
        })];

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].code,
            Some(e3::UNDEFINED_IDENTIFIER as u32)
        );
        assert!(
            result.diagnostics[0].message.contains("missing"),
            "unexpected diagnostics for {extension}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_reports_unresolved_identifiers_inside_default_export_function_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "export default function describe() { missing; }",
    )
    .unwrap();

    let statements = vec![Statement::ExportDefault(
        ExportDefaultDeclaration::FunctionDeclaration(FunctionDeclaration {
            name: "describe".to_string(),
            params: vec![],
            body: Box::new(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::Identifier("missing".to_string())),
                })],
            }),
            is_async: false,
            generator: false,
        }),
    )];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::UNDEFINED_IDENTIFIER as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("missing"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_unresolved_identifiers_inside_default_export_function_in_ts_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "export default function describe() { missing; }",
    )
    .unwrap();

    let statements = vec![Statement::ExportDefault(
        ExportDefaultDeclaration::FunctionDeclaration(FunctionDeclaration {
            name: "describe".to_string(),
            params: vec![],
            body: Box::new(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::Identifier("missing".to_string())),
                })],
            }),
            is_async: false,
            generator: false,
        }),
    )];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::UNDEFINED_IDENTIFIER as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("missing"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_unresolved_identifiers_inside_default_export_function_in_jsx_and_tsx_input(
) {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().unwrap();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            "export default function describe() { missing; }",
        )
        .unwrap();

        let statements = vec![Statement::ExportDefault(
            ExportDefaultDeclaration::FunctionDeclaration(FunctionDeclaration {
                name: "describe".to_string(),
                params: vec![],
                body: Box::new(BlockStatement {
                    body: vec![Statement::ExpressionStatement(ExpressionStatement {
                        expression: Box::new(Expression::Identifier("missing".to_string())),
                    })],
                }),
                is_async: false,
                generator: false,
            }),
        )];

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].code,
            Some(e3::UNDEFINED_IDENTIFIER as u32)
        );
        assert!(
            result.diagnostics[0].message.contains("missing"),
            "unexpected diagnostics for {extension}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_reports_missing_re_export_sources() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export { missing } from './missing.ts';").unwrap();

    let statements = vec![Statement::ExportNamed(ExportNamedDeclaration {
        specifiers: vec![ExportSpecifier {
            local: "missing".to_string(),
            exported: "missing".to_string(),
        }],
        source: Some("./missing.ts".to_string()),
    })];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::IMPORT_NOT_FOUND as u32)
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains("could not be resolved"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_missing_re_export_sources_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "export { missing } from './missing.js';").unwrap();

    let statements = vec![Statement::ExportNamed(ExportNamedDeclaration {
        specifiers: vec![ExportSpecifier {
            local: "missing".to_string(),
            exported: "missing".to_string(),
        }],
        source: Some("./missing.js".to_string()),
    })];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::IMPORT_NOT_FOUND as u32)
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains("could not be resolved"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_missing_re_export_sources_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().unwrap();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            format!("export {{ missing }} from './missing.{extension}';"),
        )
        .unwrap();

        let statements = vec![Statement::ExportNamed(ExportNamedDeclaration {
            specifiers: vec![ExportSpecifier {
                local: "missing".to_string(),
                exported: "missing".to_string(),
            }],
            source: Some(format!("./missing.{extension}")),
        })];

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].code,
            Some(e3::IMPORT_NOT_FOUND as u32)
        );
        assert!(
            result.diagnostics[0]
                .message
                .contains("could not be resolved"),
            "unexpected diagnostics for {extension}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_reports_math_floor_as_available_for_integer_inputs() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "floor".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(1.0))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_math_sqrt_as_available_for_perfect_square_integer_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "sqrt".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(4.0))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_allows_nullish_coalescing() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::BinaryExpression(Box::new(BinaryExpression {
            operator: "??".to_string(),
            left: Expression::Literal(LiteralValue::Null),
            right: Expression::Literal(LiteralValue::Number(1.0)),
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
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
            expression: Box::new(Expression::Identifier("ReadableStream".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("TransformStream".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("WritableStream".to_string())),
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
            expression: Box::new(Expression::Identifier("localStorage".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("sessionStorage".to_string())),
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
fn test_resolution_allows_shared_web_baseline_globals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("structuredClone".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("AbortController".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("AbortSignal".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("Event".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("EventTarget".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("CustomEvent".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("URL".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("URLSearchParams".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("TextEncoder".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("TextDecoder".to_string())),
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
fn test_resolution_allows_browser_baseline_host_globals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("fetch".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("Headers".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("Request".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("Response".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("Blob".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("File".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("performance".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("crypto".to_string())),
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
fn test_resolution_reports_threaded_runtime_globals_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("SharedArrayBuffer".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("Atomics".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("globalThis".to_string()),
                    property: "SharedArrayBuffer".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("globalThis".to_string()),
                    property: "Atomics".to_string(),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 4);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
}

#[test]
fn test_resolution_accepts_threaded_runtime_globals_when_profile_is_enabled() {
    let mut ctx = TypeContext::with_api_surface_and_runtime_profiles(
        "deno",
        vec!["wasm-threads".to_string()],
    );
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("SharedArrayBuffer".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("Atomics".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("globalThis".to_string()),
                    property: "SharedArrayBuffer".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("globalThis".to_string()),
                    property: "Atomics".to_string(),
                },
            ))),
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
fn test_resolution_reports_generator_lowering_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::FunctionDeclaration(FunctionDeclaration {
            name: "main".to_string(),
            params: vec![],
            body: Box::new(BlockStatement { body: vec![] }),
            is_async: false,
            generator: true,
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::YieldExpression(Box::new(YieldExpression {
                delegate: false,
                argument: None,
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::YieldExpression(Box::new(YieldExpression {
                delegate: true,
                argument: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![],
                })),
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::FunctionExpression(Box::new(
                FunctionExpression {
                    id: Some("inner".to_string()),
                    params: vec![],
                    body: Some(Box::new(BlockStatement { body: vec![] })),
                    is_async: false,
                    generator: true,
                },
            ))),
        }),
        Statement::FunctionDeclaration(FunctionDeclaration {
            name: "asyncMain".to_string(),
            params: vec![],
            body: Box::new(BlockStatement { body: vec![] }),
            is_async: true,
            generator: true,
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::FunctionExpression(Box::new(
                FunctionExpression {
                    id: Some("asyncInner".to_string()),
                    params: vec![],
                    body: Some(Box::new(BlockStatement { body: vec![] })),
                    is_async: true,
                    generator: true,
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.len() >= 6);
    assert!(
        result
            .diagnostics
            .iter()
            .filter(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32))
            .count()
            >= 6
    );
}

#[test]
fn test_resolution_reports_late_host_control_globals_as_unavailable() {
    let mut ctx = TypeContext::with_base_path_and_api_surface(".", "node");
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Deno".to_string()),
                    property: "pid".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Deno".to_string(),
                    })),
                    property: "pid".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Deno".to_string(),
                    })),
                    property: "cwd".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Deno".to_string()),
                    property: "chdir".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Deno".to_string(),
                    })),
                    property: "chdir".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Deno".to_string(),
                    })),
                    property: "exit".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("process".to_string()),
                    property: "pid".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "process".to_string(),
                    })),
                    property: "pid".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "process".to_string(),
                    })),
                    property: "cwd".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("process".to_string()),
                    property: "chdir".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "process".to_string(),
                    })),
                    property: "chdir".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "process".to_string(),
                    })),
                    property: "exit".to_string(),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.len() >= 10);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    for expected in [
        "globalThis.Deno.cwd",
        "Deno.chdir",
        "globalThis.Deno.chdir",
        "globalThis.Deno.exit",
        "process.pid",
        "globalThis.process.pid",
        "globalThis.process.cwd",
        "process.chdir",
        "globalThis.process.chdir",
        "globalThis.process.exit",
    ] {
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.message.contains(expected)),
            "missing {expected} in {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_rejects_env_snapshot_materialization_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("Deno".to_string()),
                        property: "env".to_string(),
                    })),
                    property: "toObject".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::MemberExpression(Box::new(
                            kali_ast::MemberExpression {
                                object: Expression::Identifier("globalThis".to_string()),
                                property: "Deno".to_string(),
                            },
                        )),
                        property: "env".to_string(),
                    })),
                    property: "toObject".to_string(),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 2);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Deno.env.toObject")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("globalThis.Deno.env.toObject")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Deno[\"env\"][\"toObject\"]")));
    assert!(result.diagnostics.iter().any(|diag| diag
        .message
        .contains("globalThis[\"Deno\"][\"env\"][\"toObject\"]")));
}

#[test]
fn test_resolution_rejects_process_env_assignment_as_unavailable_in_node_api_surface() {
    let mut ctx = TypeContext::with_api_surface("node");
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                AssignmentExpression {
                    operator: AssignmentOperator::Assign,
                    left: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("process".to_string()),
                        property: "env".to_string(),
                    })),
                    right: Expression::Literal(LiteralValue::Number(1.0)),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                AssignmentExpression {
                    operator: AssignmentOperator::Assign,
                    left: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("globalThis".to_string()),
                            property: "process".to_string(),
                        })),
                        property: "env".to_string(),
                    })),
                    right: Expression::Literal(LiteralValue::Number(2.0)),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 2);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("process.env")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("globalThis.process.env")));
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.message.contains("later mutable env path")));
}

#[test]
fn test_resolution_rejects_env_mutation_as_unavailable_in_browser_api_surface() {
    let mut ctx = TypeContext::with_api_surface("browser");
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("Deno".to_string()),
                        property: "env".to_string(),
                    })),
                    property: "set".to_string(),
                })),
                args: vec![
                    Expression::Literal(LiteralValue::String("KALI_FLAG".to_string())),
                    Expression::Literal(LiteralValue::String("1".to_string())),
                ],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("globalThis".to_string()),
                            property: "Deno".to_string(),
                        })),
                        property: "env".to_string(),
                    })),
                    property: "delete".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::String(
                    "KALI_FLAG".to_string(),
                ))],
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 2);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Deno.env.set")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("globalThis.Deno.env.delete")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("browser API surface")));
}

#[test]
fn test_resolution_rejects_object_has_own_as_unavailable_in_browser_api_surface() {
    let mut ctx = TypeContext::with_api_surface("browser");
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "hasOwn".to_string(),
                })),
                args: vec![
                    Expression::ObjectExpression(ObjectExpression {
                        properties: vec![ObjectProperty {
                            key: PropertyName::Identifier("a".to_string()),
                            value: Expression::Literal(LiteralValue::Number(1.0)),
                            kind: ObjectPropertyKind::Init,
                        }],
                    }),
                    Expression::Literal(LiteralValue::String("a".to_string())),
                ],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Object".to_string(),
                    })),
                    property: "hasOwn".to_string(),
                })),
                args: vec![
                    Expression::ObjectExpression(ObjectExpression {
                        properties: vec![ObjectProperty {
                            key: PropertyName::Identifier("a".to_string()),
                            value: Expression::Literal(LiteralValue::Number(1.0)),
                            kind: ObjectPropertyKind::Init,
                        }],
                    }),
                    Expression::Literal(LiteralValue::String("a".to_string())),
                ],
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 2);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Object.hasOwn")));
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.message.contains("Object.hasOwn")));
}

#[test]
fn test_resolution_rejects_unsupported_permission_query_descriptors() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("Deno".to_string()),
                        property: "permissions".to_string(),
                    })),
                    property: "query".to_string(),
                })),
                args: vec![Expression::ObjectExpression(ObjectExpression {
                    properties: vec![ObjectProperty {
                        key: PropertyName::Identifier("name".to_string()),
                        value: Expression::Literal(LiteralValue::String("env".to_string())),
                        kind: ObjectPropertyKind::Init,
                    }],
                })],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("globalThis".to_string()),
                            property: "Deno".to_string(),
                        })),
                        property: "permissions".to_string(),
                    })),
                    property: "query".to_string(),
                })),
                args: vec![Expression::ObjectExpression(ObjectExpression {
                    properties: vec![ObjectProperty {
                        key: PropertyName::String("name".to_string()),
                        value: Expression::Literal(LiteralValue::String("ffi".to_string())),
                        kind: ObjectPropertyKind::Init,
                    }],
                })],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("globalThis".to_string()),
                            property: "Deno".to_string(),
                        })),
                        property: "permissions".to_string(),
                    })),
                    property: "query".to_string(),
                })),
                args: vec![Expression::ObjectExpression(ObjectExpression {
                    properties: vec![ObjectProperty {
                        key: PropertyName::String("name".to_string()),
                        value: Expression::Literal(LiteralValue::String("sys".to_string())),
                        kind: ObjectPropertyKind::Init,
                    }],
                })],
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 2);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("permission query descriptor 'ffi'")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("permission query descriptor 'sys'")));
}

#[test]
fn test_resolution_accepts_supported_permission_query_descriptors_with_const_bindings_in_js_input()
{
    fn member(object: Expression, property: &str) -> Expression {
        Expression::MemberExpression(Box::new(MemberExpression {
            object,
            property: property.to_string(),
        }))
    }

    fn const_descriptor(name: &str, value: &str) -> Statement {
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: name.to_string(),
                init: Some(Expression::Literal(LiteralValue::String(value.to_string()))),
            }],
        })
    }

    fn permission_query(root: Expression, descriptor: &str) -> Statement {
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: member(member(root, "permissions"), "query"),
                args: vec![Expression::ObjectExpression(ObjectExpression {
                    properties: vec![ObjectProperty {
                        key: PropertyName::Identifier("name".to_string()),
                        value: Expression::Identifier(descriptor.to_string()),
                        kind: ObjectPropertyKind::Init,
                    }],
                })],
            }))),
        })
    }

    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "const descriptor = 'read';\n").expect("write source");

    let mut ctx = TypeContext::with_base_path(&source_path);
    let statements = vec![
        const_descriptor("read_descriptor", "read"),
        permission_query(
            Expression::Identifier("Deno".to_string()),
            "read_descriptor",
        ),
        const_descriptor("write_descriptor", "write"),
        permission_query(
            member(Expression::Identifier("globalThis".to_string()), "Deno"),
            "write_descriptor",
        ),
        const_descriptor("net_descriptor", "net"),
        permission_query(Expression::Identifier("Deno".to_string()), "net_descriptor"),
        const_descriptor("env_descriptor", "env"),
        permission_query(
            member(Expression::Identifier("globalThis".to_string()), "Deno"),
            "env_descriptor",
        ),
    ];

    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_resolution_accepts_supported_permission_query_descriptors_with_const_bindings_in_ts_input()
{
    fn member(object: Expression, property: &str) -> Expression {
        Expression::MemberExpression(Box::new(MemberExpression {
            object,
            property: property.to_string(),
        }))
    }

    fn const_descriptor(name: &str, value: &str) -> Statement {
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: name.to_string(),
                init: Some(Expression::Literal(LiteralValue::String(value.to_string()))),
            }],
        })
    }

    fn permission_query(root: Expression, descriptor: &str) -> Statement {
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: member(member(root, "permissions"), "query"),
                args: vec![Expression::ObjectExpression(ObjectExpression {
                    properties: vec![ObjectProperty {
                        key: PropertyName::Identifier("name".to_string()),
                        value: Expression::Identifier(descriptor.to_string()),
                        kind: ObjectPropertyKind::Init,
                    }],
                })],
            }))),
        })
    }

    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const descriptor = 'read';\n").expect("write source");

    let mut ctx = TypeContext::with_base_path(&source_path);
    let statements = vec![
        const_descriptor("read_descriptor", "read"),
        permission_query(
            Expression::Identifier("Deno".to_string()),
            "read_descriptor",
        ),
        const_descriptor("write_descriptor", "write"),
        permission_query(
            member(Expression::Identifier("globalThis".to_string()), "Deno"),
            "write_descriptor",
        ),
        const_descriptor("net_descriptor", "net"),
        permission_query(Expression::Identifier("Deno".to_string()), "net_descriptor"),
        const_descriptor("env_descriptor", "env"),
        permission_query(
            member(Expression::Identifier("globalThis".to_string()), "Deno"),
            "env_descriptor",
        ),
    ];

    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_resolution_reports_permission_escalation_members_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("Deno".to_string()),
                        property: "permissions".to_string(),
                    })),
                    property: "request".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("Deno".to_string()),
                        property: "permissions".to_string(),
                    })),
                    property: "revoke".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::MemberExpression(Box::new(
                            kali_ast::MemberExpression {
                                object: Expression::Identifier("globalThis".to_string()),
                                property: "Deno".to_string(),
                            },
                        )),
                        property: "permissions".to_string(),
                    })),
                    property: "request".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::MemberExpression(Box::new(
                            kali_ast::MemberExpression {
                                object: Expression::Identifier("globalThis".to_string()),
                                property: "Deno".to_string(),
                            },
                        )),
                        property: "permissions".to_string(),
                    })),
                    property: "revoke".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("Deno".to_string()),
                        property: "permissions".to_string(),
                    })),
                    property: "request".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("Deno".to_string()),
                        property: "permissions".to_string(),
                    })),
                    property: "revoke".to_string(),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 6);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    for expected in [
        "Deno.permissions.request",
        "Deno.permissions.revoke",
        "globalThis.Deno.permissions.request",
        "globalThis.Deno.permissions.revoke",
    ] {
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.message.contains(expected)),
            "missing {expected} in {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_reports_broader_intl_support_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("Intl".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Intl".to_string()),
                    property: "NumberFormat".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Intl".to_string()),
                    property: "DisplayNames".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Intl".to_string()),
                    property: "Locale".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Intl".to_string(),
                    })),
                    property: "NumberFormat".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Intl".to_string(),
                    })),
                    property: "DisplayNames".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Intl".to_string(),
                    })),
                    property: "Locale".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("globalThis".to_string()),
                    property: "Intl".to_string(),
                },
            ))),
        }),
    ];
    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.len() >= 2);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.message.contains("Intl")));
    assert!(result.diagnostics.iter().any(|diag| {
        diag.message
            .contains(r#"globalThis["Intl"]["NumberFormat"]"#)
    }));
    assert!(result.diagnostics.iter().any(|diag| {
        diag.message
            .contains(r#"globalThis["Intl"]["DisplayNames"]"#)
    }));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| { diag.message.contains(r#"globalThis["Intl"]["Locale"]"#) }));
}

#[test]
fn test_resolution_reports_global_this_intl_root_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::MemberExpression(Box::new(
            kali_ast::MemberExpression {
                object: Expression::Identifier("globalThis".to_string()),
                property: "Intl".to_string(),
            },
        ))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(result.diagnostics[0].message.contains("globalThis.Intl"));
}

#[test]
fn test_resolution_reports_late_intl_member_access_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Intl".to_string()),
                    property: "NumberFormat".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Intl".to_string()),
                    property: "RelativeTimeFormat".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Intl".to_string()),
                    property: "Collator".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Intl".to_string()),
                    property: "DisplayNames".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Intl".to_string()),
                    property: "Segmenter".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Intl".to_string()),
                    property: "Locale".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Intl".to_string(),
                    })),
                    property: "DisplayNames".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Intl".to_string(),
                    })),
                    property: "Locale".to_string(),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 8);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    for expected in [
        "Intl.NumberFormat",
        "Intl.RelativeTimeFormat",
        "Intl.Collator",
        "Intl.DisplayNames",
        "Intl.Segmenter",
        "Intl.Locale",
    ] {
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.message.contains(expected)),
            "missing diagnostic for {expected}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_reports_late_object_model_globals_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("Proxy".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("WeakRef".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::NewExpression(Box::new(
                kali_ast::NewExpression {
                    callee: Expression::Identifier("WeakMap".to_string()),
                    args: Vec::new(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::NewExpression(Box::new(
                kali_ast::NewExpression {
                    callee: Expression::Identifier("WeakSet".to_string()),
                    args: Vec::new(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::NewExpression(Box::new(
                kali_ast::NewExpression {
                    callee: Expression::Identifier("FinalizationRegistry".to_string()),
                    args: Vec::new(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("globalThis".to_string()),
                    property: "Proxy".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Proxy".to_string(),
                    })),
                    property: "revocable".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("globalThis".to_string()),
                    property: "WeakMap".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "WeakMap".to_string(),
                    })),
                    property: "value".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("globalThis".to_string()),
                    property: "WeakSet".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "WeakSet".to_string(),
                    })),
                    property: "value".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("globalThis".to_string()),
                    property: "WeakRef".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "WeakRef".to_string(),
                    })),
                    property: "value".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("globalThis".to_string()),
                    property: "FinalizationRegistry".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "FinalizationRegistry".to_string(),
                    })),
                    property: "value".to_string(),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 15);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    for expected in [
        "Proxy",
        "WeakRef",
        "WeakMap",
        "WeakSet",
        "FinalizationRegistry",
        "globalThis.Proxy",
        r#"globalThis["Proxy"]"#,
        r#"globalThis["WeakMap"]"#,
        r#"globalThis["WeakSet"]"#,
        r#"globalThis["WeakRef"]"#,
        r#"globalThis["FinalizationRegistry"]"#,
    ] {
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.message.contains(expected)),
            "missing {expected} in {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_supports_math_round_member_calls_for_non_integer_numeric_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "round".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.6))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "round".to_string(),
                })),
                args: vec![Expression::UnaryExpression(Box::new(
                    kali_ast::UnaryExpression {
                        operator: "-".to_string(),
                        argument: Expression::Literal(LiteralValue::Number(1.5)),
                    },
                ))],
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
fn test_resolution_supports_math_cbrt_member_calls_for_perfect_cube_integer_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "cbrt".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(27.0))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_math_log2_member_calls_for_positive_power_of_two_integer_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "log2".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(8.0))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_math_log10_member_calls_for_positive_power_of_ten_integer_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "log10".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(1000.0))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_math_pow_member_calls_for_integer_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "pow".to_string(),
            })),
            args: vec![
                Expression::Literal(LiteralValue::Number(2.0)),
                Expression::Literal(LiteralValue::Number(3.0)),
            ],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_unsupported_math_pow_negative_exponents_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "pow".to_string(),
            })),
            args: vec![
                Expression::Literal(LiteralValue::Number(2.0)),
                Expression::UnaryExpression(Box::new(kali_ast::UnaryExpression {
                    operator: "-".to_string(),
                    argument: Expression::Literal(LiteralValue::Number(1.0)),
                })),
            ],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(result.diagnostics[0]
        .message
        .contains("negative numeric literals"));
}

#[test]
fn test_resolution_reports_unsupported_math_cbrt_member_calls_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "cbrt".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(28.0))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.cbrt")));
}

#[test]
fn test_resolution_reports_unsupported_math_log2_member_calls_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "log2".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(12.0))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.log2")
            && diag.message.contains("positive power-of-two")));
}

#[test]
fn test_resolution_reports_unsupported_math_log10_member_calls_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "log10".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(12.0))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.log10")
            && diag.message.contains("positive power-of-ten")));
}

#[test]
fn test_resolution_reports_math_max_without_arguments_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "max".to_string(),
            })),
            args: vec![],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(result.diagnostics[0]
        .message
        .contains("requires at least one argument"));
}

#[test]
fn test_resolution_reports_math_pow_with_single_argument_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "pow".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(2.0))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(result.diagnostics[0]
        .message
        .contains("requires at least two arguments"));
}

#[test]
fn test_resolution_supports_math_sqrt_member_calls_with_const_numeric_alias_chain() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(4.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Identifier("value".to_string())),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "sqrt".to_string(),
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
fn test_resolution_supports_math_cbrt_member_calls_with_negative_const_numeric_alias_chain() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::UnaryExpression(Box::new(
                    kali_ast::UnaryExpression {
                        operator: "-".to_string(),
                        argument: Expression::Literal(LiteralValue::Number(27.0)),
                    },
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
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "cbrt".to_string(),
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
fn test_resolution_rejects_negative_const_numeric_alias_exponents_in_math_pow_member_calls_as_unavailable(
) {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "exponent".to_string(),
                init: Some(Expression::UnaryExpression(Box::new(
                    kali_ast::UnaryExpression {
                        operator: "-".to_string(),
                        argument: Expression::Literal(LiteralValue::Number(1.0)),
                    },
                ))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Identifier("exponent".to_string())),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "pow".to_string(),
                })),
                args: vec![
                    Expression::Literal(LiteralValue::Number(2.0)),
                    Expression::Identifier("alias".to_string()),
                ],
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(result.diagnostics[0]
        .message
        .contains("negative numeric literals"));
}

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
fn test_resolution_supports_non_integer_numeric_literals_in_math_ceil_member_calls() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "ceil".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(1.6))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_non_integer_numeric_literals_in_math_trunc_member_calls() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "trunc".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(1.6))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_proxy_revocable_member_access_as_late_object_model_api() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Proxy".to_string()),
                    property: "revocable".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Proxy".to_string(),
                    })),
                    property: "revocable".to_string(),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 2);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Proxy.revocable")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("globalThis.Proxy.revocable")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| { diag.message.contains(r#"globalThis["Proxy"]["revocable"]"#) }));
}

#[test]
fn test_resolution_reports_object_has_own_helpers_as_late_object_model_api() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "hasOwn".to_string(),
                })),
                args: vec![
                    Expression::ObjectExpression(ObjectExpression {
                        properties: vec![ObjectProperty {
                            key: PropertyName::Identifier("a".to_string()),
                            value: Expression::Literal(LiteralValue::Number(1.0)),
                            kind: ObjectPropertyKind::Init,
                        }],
                    }),
                    Expression::Literal(LiteralValue::String("a".to_string())),
                ],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::MemberExpression(Box::new(
                            kali_ast::MemberExpression {
                                object: Expression::Identifier("Object".to_string()),
                                property: "prototype".to_string(),
                            },
                        )),
                        property: "hasOwnProperty".to_string(),
                    })),
                    property: "call".to_string(),
                })),
                args: vec![
                    Expression::ObjectExpression(ObjectExpression {
                        properties: vec![ObjectProperty {
                            key: PropertyName::Identifier("a".to_string()),
                            value: Expression::Literal(LiteralValue::Number(1.0)),
                            kind: ObjectPropertyKind::Init,
                        }],
                    }),
                    Expression::Literal(LiteralValue::String("a".to_string())),
                ],
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 2);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Object.hasOwn")));
    assert!(result.diagnostics.iter().any(|diag| diag
        .message
        .contains("Object.prototype.hasOwnProperty.call")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| { diag.message.contains(r#"Object["hasOwn"]"#) }));
    assert!(result.diagnostics.iter().any(|diag| {
        diag.message
            .contains(r#"Object["prototype"]["hasOwnProperty"]["call"]"#)
    }));
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
fn test_resolution_allows_node_timers_imports_in_node_context() {
    let mut ctx = TypeContext::with_base_path_and_api_surface(".", "node");
    let statements = vec![Statement::ImportDeclaration(ImportDeclaration {
        specifiers: vec![ImportSpecifier::Default("timers".to_string())],
        source: "node:timers".to_string(),
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
fn test_resolution_allows_static_dynamic_import_targets_in_js_files() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(dir.path().join("lazy.js"), "export const lazy = 7;").unwrap();
    fs::write(&source_path, "const lazy = import(\"./lazy.js\");").unwrap();

    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
            source: Expression::Literal(LiteralValue::String("./lazy.js".to_string())),
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
fn test_resolution_allows_parenthesized_dynamic_import_targets_in_js_files() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(dir.path().join("lazy.js"), "export const lazy = 7;").unwrap();
    fs::write(
        &source_path,
        "const name = \"lazy.js\"; const root = \"./\"; import((root + name));",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "name".to_string(),
                init: Some(Expression::Literal(LiteralValue::String(
                    "lazy.js".to_string(),
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
fn test_resolution_accepts_directory_index_dynamic_import_targets() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).unwrap();
    fs::write(lazy_dir.join("index.ts"), "export const lazy = 7;").unwrap();
    fs::write(&source_path, "import(\"./lazy\");").unwrap();

    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
            source: Expression::Literal(LiteralValue::String("./lazy".to_string())),
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
fn test_resolution_accepts_directory_index_dynamic_import_targets_in_js_files() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).unwrap();
    fs::write(lazy_dir.join("index.js"), "export const lazy = 7;").unwrap();
    fs::write(&source_path, "import(\"./lazy\");").unwrap();

    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
            source: Expression::Literal(LiteralValue::String("./lazy".to_string())),
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
fn test_resolution_accepts_directory_index_dynamic_import_targets_in_jsx_files() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.jsx");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).unwrap();
    fs::write(lazy_dir.join("index.jsx"), "export const lazy = 7;").unwrap();
    fs::write(&source_path, "import(\"./lazy\");").unwrap();

    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
            source: Expression::Literal(LiteralValue::String("./lazy".to_string())),
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
fn test_resolution_accepts_directory_index_dynamic_import_targets_in_tsx_files() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.tsx");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).unwrap();
    fs::write(lazy_dir.join("index.tsx"), "export const lazy = 7;").unwrap();
    fs::write(&source_path, "import(\"./lazy\");").unwrap();

    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
            source: Expression::Literal(LiteralValue::String("./lazy".to_string())),
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
fn test_resolution_rejects_directory_dynamic_import_targets_without_index_in_js_files() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).unwrap();
    fs::write(&source_path, "import(\"./lazy\");").unwrap();

    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
            source: Expression::Literal(LiteralValue::String("./lazy".to_string())),
        }))),
    })];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e4::DYNAMIC_IMPORT_NOT_IN_LINKED_GRAPH as u32)
    );
}

#[test]
fn test_resolution_rejects_directory_dynamic_import_targets_without_index() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).unwrap();
    fs::write(&source_path, "import(\"./lazy\");").unwrap();

    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
            source: Expression::Literal(LiteralValue::String("./lazy".to_string())),
        }))),
    })];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e4::DYNAMIC_IMPORT_NOT_IN_LINKED_GRAPH as u32)
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
fn test_resolution_rejects_non_literal_dynamic_import_targets() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let specifier; import(specifier);").unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "specifier".to_string(),
                init: None,
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
                source: Expression::Identifier("specifier".to_string()),
            }))),
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
    assert!(result.diagnostics.iter().any(|diag| {
        diag.message.contains("non-literal dynamic import()")
            || diag.message.contains("statically known import specifier")
    }));
}

#[test]
fn test_resolution_rejects_generator_function_lowering() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "function* main() { yield* []; }\nmain();").unwrap();

    let statements = vec![Statement::FunctionDeclaration(FunctionDeclaration {
        name: "main".to_string(),
        params: vec![],
        body: Box::new(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::YieldExpression(Box::new(YieldExpression {
                    delegate: true,
                    argument: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                        elements: vec![],
                    })),
                }))),
            })],
        }),
        is_async: false,
        generator: true,
    })];

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
    assert!(result.diagnostics[0]
        .message
        .contains("generator function lowering is unavailable"));
}

#[test]
fn test_resolution_rejects_generator_function_lowering_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "function* main() { yield* []; }\nmain();").unwrap();

    let statements = vec![Statement::FunctionDeclaration(FunctionDeclaration {
        name: "main".to_string(),
        params: vec![],
        body: Box::new(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::YieldExpression(Box::new(YieldExpression {
                    delegate: true,
                    argument: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                        elements: vec![],
                    })),
                }))),
            })],
        }),
        is_async: false,
        generator: true,
    })];

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
    assert!(result.diagnostics[0]
        .message
        .contains("generator function lowering is unavailable"));
}

#[test]
fn test_resolution_rejects_generator_function_lowering_in_jsx_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.jsx");
    fs::write(&source_path, "function* main() { yield* []; }\nmain();").unwrap();

    let statements = vec![Statement::FunctionDeclaration(FunctionDeclaration {
        name: "main".to_string(),
        params: vec![],
        body: Box::new(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::YieldExpression(Box::new(YieldExpression {
                    delegate: true,
                    argument: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                        elements: vec![],
                    })),
                }))),
            })],
        }),
        is_async: false,
        generator: true,
    })];

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
    assert!(result.diagnostics[0]
        .message
        .contains("generator function lowering is unavailable"));
}

#[test]
fn test_resolution_rejects_generator_function_lowering_in_tsx_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.tsx");
    fs::write(&source_path, "function* main() { yield* []; }\nmain();").unwrap();

    let statements = vec![Statement::FunctionDeclaration(FunctionDeclaration {
        name: "main".to_string(),
        params: vec![],
        body: Box::new(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::YieldExpression(Box::new(YieldExpression {
                    delegate: true,
                    argument: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                        elements: vec![],
                    })),
                }))),
            })],
        }),
        is_async: false,
        generator: true,
    })];

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
    assert!(result.diagnostics[0]
        .message
        .contains("generator function lowering is unavailable"));
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
