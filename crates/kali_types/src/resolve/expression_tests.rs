use crate::test_support::*;
use kali_test_support::fixtures;
use crate::*;
use kali_ast::{
    AssignmentExpression, AssignmentOperator, BinaryExpression, BlockStatement,
    DecoratedExpression, ExportDefaultDeclaration, ExportNamedDeclaration, ExportSpecifier,
    Expression, ExpressionStatement, FunctionDeclaration, LiteralValue, LogicalExpression,
    LogicalOperator, MemberExpression, ObjectExpression, ObjectProperty, ObjectPropertyKind,
    ParenthesizedExpression, PropertyName, TemplateElement, TemplateLiteral, UnaryExpression,
    UpdateExpression, UpdateOperator, VariableDeclaration, VariableDeclarator,
};
use kali_error::_error_codes::{e3, e5};
use std::fs;

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
    let dir = fixtures::tempdir();
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
fn test_resolution_resolves_export_all_sources_in_js_input() {
    let dir = fixtures::tempdir();
    let helper_path = dir.path().join("helper.js");
    let source_path = dir.path().join("main.js");
    fs::write(
        &helper_path,
        "export function quadruple(value) { return value + value; }",
    )
    .unwrap();
    fs::write(&source_path, "export * from './helper.js';").unwrap();

    let statements = vec![Statement::ExportAll(kali_ast::ExportAllDeclaration {
        source: "./helper.js".to_string(),
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
fn test_resolution_reports_unresolved_public_exports_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
        let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
        let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
        let dir = fixtures::tempdir();
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
fn test_resolution_allows_nullish_coalescing_with_void_and_undefined_fallbacks() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::BinaryExpression(Box::new(BinaryExpression {
                operator: "??".to_string(),
                left: Expression::UnaryExpression(Box::new(UnaryExpression {
                    operator: "void".to_string(),
                    argument: Expression::Literal(LiteralValue::Number(0.0)),
                })),
                right: Expression::Literal(LiteralValue::Number(1.0)),
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::BinaryExpression(Box::new(BinaryExpression {
                operator: "??".to_string(),
                left: Expression::Identifier("undefined".to_string()),
                right: Expression::Literal(LiteralValue::Number(2.0)),
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
fn test_resolution_allows_remainder_operator() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::BinaryExpression(Box::new(BinaryExpression {
                operator: "%".to_string(),
                left: Expression::Literal(LiteralValue::Number(7.0)),
                right: Expression::Literal(LiteralValue::Number(3.0)),
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::BinaryExpression(Box::new(BinaryExpression {
                operator: "%".to_string(),
                left: Expression::BigIntLiteral("7n".to_string()),
                right: Expression::BigIntLiteral("3n".to_string()),
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
fn test_resolution_supports_update_expressions_on_mutable_bindings() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::UpdateExpression(Box::new(UpdateExpression {
                operator: UpdateOperator::Increment,
                argument: Expression::Identifier("value".to_string()),
                prefix: true,
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::UpdateExpression(Box::new(UpdateExpression {
                operator: UpdateOperator::Decrement,
                argument: Expression::Identifier("value".to_string()),
                prefix: false,
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                AssignmentExpression {
                    operator: AssignmentOperator::ExponentAssign,
                    left: Expression::ParenthesizedExpression(Box::new(ParenthesizedExpression {
                        expression: Box::new(Expression::Identifier("value".to_string())),
                    })),
                    right: Expression::Literal(LiteralValue::Number(2.0)),
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
fn test_resolution_rejects_update_expressions_on_immutable_bindings() {
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
            expression: Box::new(Expression::UpdateExpression(Box::new(UpdateExpression {
                operator: UpdateOperator::Increment,
                argument: Expression::Identifier("value".to_string()),
                prefix: true,
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
        .contains("mutable local binding"));
}

#[test]
fn test_resolution_rejects_compound_assignment_on_non_local_targets_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "target".to_string(),
                init: Some(Expression::ObjectExpression(ObjectExpression {
                    properties: vec![ObjectProperty {
                        kind: ObjectPropertyKind::Init,
                        key: PropertyName::Identifier("value".to_string()),
                        value: Expression::Literal(LiteralValue::Number(1.0)),
                    }],
                })),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                AssignmentExpression {
                    operator: AssignmentOperator::AddAssign,
                    left: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("target".to_string()),
                        property: "value".to_string(),
                    })),
                    right: Expression::Literal(LiteralValue::Number(2.0)),
                },
            ))),
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                AssignmentExpression {
                    operator: AssignmentOperator::AddAssign,
                    left: Expression::Identifier("value".to_string()),
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
        .any(|diag| diag.message.contains("compound assignment lowering")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("binding 'value'")));
}

#[test]
fn test_resolution_accepts_decorated_wrappers_for_update_targets() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                AssignmentExpression {
                    operator: AssignmentOperator::AddAssign,
                    left: Expression::DecoratedExpression(DecoratedExpression {
                        expression: Box::new(Expression::Identifier("value".to_string())),
                    }),
                    right: Expression::Literal(LiteralValue::Number(2.0)),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::UpdateExpression(Box::new(UpdateExpression {
                operator: UpdateOperator::Increment,
                argument: Expression::DecoratedExpression(DecoratedExpression {
                    expression: Box::new(Expression::Identifier("value".to_string())),
                }),
                prefix: true,
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_resolution_allows_static_dynamic_import_targets() {
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
fn test_resolution_allows_template_literal_dynamic_import_targets() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    fs::write(dir.path().join("lazy.ts"), "export const lazy = 7;").unwrap();
    fs::write(
        &source_path,
        "const name = \"lazy.ts\"; import(`./${name}`);",
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
                source: Expression::TemplateLiteral(TemplateLiteral {
                    quasis: vec![
                        TemplateElement {
                            value: "./".to_string(),
                            tail: false,
                        },
                        TemplateElement {
                            value: "".to_string(),
                            tail: true,
                        },
                    ],
                    expressions: vec![Expression::Identifier("name".to_string())],
                }),
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
fn test_resolution_allows_template_literal_dynamic_import_targets_in_js_files() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(dir.path().join("lazy.js"), "export const lazy = 7;").unwrap();
    fs::write(
        &source_path,
        "const name = \"lazy.js\"; import(`./${name}`);",
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
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
                source: Expression::TemplateLiteral(TemplateLiteral {
                    quasis: vec![
                        TemplateElement {
                            value: "./".to_string(),
                            tail: false,
                        },
                        TemplateElement {
                            value: "".to_string(),
                            tail: true,
                        },
                    ],
                    expressions: vec![Expression::Identifier("name".to_string())],
                }),
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
fn test_resolution_allows_sequence_wrapped_template_literal_dynamic_import_targets_in_js_files() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(dir.path().join("lazy.js"), "export const lazy = 7;").unwrap();
    fs::write(
        &source_path,
        "const name = \"lazy.js\"; import((0, `./${name}`));",
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
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
                source: sequence_expression(vec![
                    Expression::Literal(LiteralValue::Number(0.0)),
                    Expression::TemplateLiteral(TemplateLiteral {
                        quasis: vec![
                            TemplateElement {
                                value: "./".to_string(),
                                tail: false,
                            },
                            TemplateElement {
                                value: "".to_string(),
                                tail: true,
                            },
                        ],
                        expressions: vec![Expression::Identifier("name".to_string())],
                    }),
                ]),
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
fn test_resolution_allows_const_bound_dynamic_import_targets() {
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
fn test_resolution_allows_sequence_wrapped_dynamic_import_targets_in_js_files() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(dir.path().join("lazy.js"), "export const lazy = 7;").unwrap();
    fs::write(&source_path, "import((0, \"./lazy.js\"));").unwrap();

    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
            source: sequence_expression(vec![
                Expression::Literal(LiteralValue::Number(0.0)),
                Expression::Literal(LiteralValue::String("./lazy.js".to_string())),
            ]),
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
fn test_resolution_accepts_directory_index_dynamic_import_targets() {
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
fn test_resolution_accepts_directory_index_dynamic_import_targets_in_tsx_files() {
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
fn test_resolution_accepts_constant_template_dynamic_import_targets() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    let chunk_path = dir.path().join("lazy.ts");
    fs::write(&chunk_path, "export const lazy = true;").unwrap();
    fs::write(
        &source_path,
        "const name = \"lazy.ts\"; import(`./${name}`);",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "name".to_string(),
                init: Some(Expression::Literal(LiteralValue::String(
                    "\"lazy.ts\"".to_string(),
                ))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
                source: Expression::Literal(LiteralValue::String("`./${name}`".to_string())),
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
fn test_resolution_accepts_logical_wrapped_dynamic_import_targets() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    let chunk_path = dir.path().join("lazy.ts");
    fs::write(&chunk_path, "export const lazy = true;").unwrap();
    fs::write(
        &source_path,
        "const specifier = true && './lazy.ts'; import(specifier);",
    )
    .unwrap();

    for (operator, left, source) in [
        (
            LogicalOperator::And,
            Expression::Literal(LiteralValue::Boolean(true)),
            "true && './lazy.ts'",
        ),
        (
            LogicalOperator::Or,
            Expression::Literal(LiteralValue::Boolean(false)),
            "false || './lazy.ts'",
        ),
        (
            LogicalOperator::Coalesce,
            Expression::Literal(LiteralValue::Null),
            "null ?? './lazy.ts'",
        ),
    ] {
        let statements = vec![
            Statement::VariableDeclaration(VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "specifier".to_string(),
                    init: Some(Expression::LogicalExpression(Box::new(LogicalExpression {
                        operator: operator.clone(),
                        left: Box::new(left.clone()),
                        right: Box::new(Expression::Literal(LiteralValue::String(
                            "./lazy.ts".to_string(),
                        ))),
                    }))),
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
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {source}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_rejects_non_literal_dynamic_import_targets() {
    let dir = fixtures::tempdir();
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
