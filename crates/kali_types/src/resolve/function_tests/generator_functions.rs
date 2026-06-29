use super::*;

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
    assert_eq!(
        result.diagnostics.len(),
        3,
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert_eq!(
        result
            .diagnostics
            .iter()
            .filter(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32))
            .count(),
        3
    );
    assert!(result.diagnostics.iter().any(|diag| diag
        .message
        .contains("generator and async-generator function lowering is unavailable")));
    assert!(result.diagnostics.iter().any(|diag| diag
        .message
        .contains("generator function lowering is unavailable")));
}

#[test]
fn test_resolution_rejects_generator_function_lowering() {
    let dir = fixtures::tempdir();
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
    assert!(result.diagnostics[0].message.contains("yield* delegation"));
}

#[test]
fn test_resolution_rejects_generator_yield_delegation_lowering() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "function main() { yield* []; }\nmain();").unwrap();

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
        generator: false,
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
    assert!(result.diagnostics[0].message.contains("yield* delegation"));
    assert!(result.diagnostics[0]
        .message
        .contains("generator function lowering is unavailable"));
}

#[test]
fn test_resolution_rejects_generator_function_lowering_in_js_input() {
    let dir = fixtures::tempdir();
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
fn test_resolution_rejects_async_generator_function_lowering_in_js_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "async function* main() { yield 1; }\nmain();").unwrap();

    let statements = vec![Statement::FunctionDeclaration(FunctionDeclaration {
        name: "main".to_string(),
        params: vec![],
        body: Box::new(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::YieldExpression(Box::new(YieldExpression {
                    delegate: false,
                    argument: Some(Expression::Literal(LiteralValue::Number(1.0))),
                }))),
            })],
        }),
        is_async: true,
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
        .contains("async-generator function lowering is unavailable"));
}

#[test]
fn test_resolution_rejects_mixed_generator_function_lowering_in_js_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "function* syncMain() { yield* []; }\nasync function* asyncMain() { yield 1; }\nsyncMain();\nasyncMain();",
    )
    .unwrap();

    let statements = vec![
        Statement::FunctionDeclaration(FunctionDeclaration {
            name: "syncMain".to_string(),
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
        }),
        Statement::FunctionDeclaration(FunctionDeclaration {
            name: "asyncMain".to_string(),
            params: vec![],
            body: Box::new(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::YieldExpression(Box::new(YieldExpression {
                        delegate: false,
                        argument: Some(Expression::Literal(LiteralValue::Number(1.0))),
                    }))),
                })],
            }),
            is_async: true,
            generator: true,
        }),
    ];

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
        .contains("generator and async-generator function lowering is unavailable"));
}

#[test]
fn test_resolution_rejects_generator_function_lowering_in_tsx_input() {
    let dir = fixtures::tempdir();
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
