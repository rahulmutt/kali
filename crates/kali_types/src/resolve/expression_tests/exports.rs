use super::*;

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
