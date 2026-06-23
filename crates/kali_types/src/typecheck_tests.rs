use crate::*;
use kali_ast::{
    ArrowFunctionExpression, Expression, ExpressionStatement, LiteralValue, SatisfiesExpression,
    TypeAliasDeclaration, TypeAssertion, VariableDeclaration, VariableDeclarator,
};
use kali_error::_error_codes::e3;
use std::fs;
use tempfile::tempdir;

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
