use super::*;

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
                        computed_index: None,
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
