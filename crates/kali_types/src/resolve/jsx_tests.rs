use crate::*;
use kali_ast::{
    BlockStatement, Expression, ExpressionStatement, FunctionDeclaration, LiteralValue,
    YieldExpression,
};
use kali_error::_error_codes::e5;
use kali_test_support::fixtures;
use std::fs;

#[test]
fn test_resolution_accepts_directory_index_dynamic_import_targets_in_jsx_files() {
    let dir = fixtures::tempdir();
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
fn test_resolution_rejects_generator_function_lowering_in_jsx_input() {
    let dir = fixtures::tempdir();
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
