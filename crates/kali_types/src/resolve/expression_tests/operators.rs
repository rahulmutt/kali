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

// Helper: `<lhs> + <rhs>` as a top-level expression statement.
#[cfg(test)]
fn plus_statement(lhs: Expression, rhs: Expression) -> Statement {
    Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::BinaryExpression(Box::new(BinaryExpression {
            operator: "+".to_string(),
            left: lhs,
            right: rhs,
        }))),
    })
}

#[cfg(test)]
fn let_string(name: &str, value: &str) -> Statement {
    Statement::VariableDeclaration(VariableDeclaration {
        kind: "let".to_string(),
        declarations: vec![VariableDeclarator {
            id: name.to_string(),
            init: Some(Expression::Literal(LiteralValue::String(value.to_string()))),
        }],
    })
}

#[cfg(test)]
fn let_number(name: &str, value: f64) -> Statement {
    Statement::VariableDeclaration(VariableDeclaration {
        kind: "let".to_string(),
        declarations: vec![VariableDeclarator {
            id: name.to_string(),
            init: Some(Expression::Literal(LiteralValue::Number(value))),
        }],
    })
}

#[test]
fn test_resolution_rejects_string_variable_plus_number() {
    // `let s = "x"; s + 3` — string-typed variable added to an integer.
    let mut ctx = TypeContext::new();
    let statements = vec![
        let_string("s", "x"),
        plus_statement(
            Expression::Identifier("s".to_string()),
            Expression::Literal(LiteralValue::Number(3.0)),
        ),
    ];
    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].code, Some(e3::TYPE_MISMATCH as u32));

    // `let s = "x"; 3 + s` — operand order is symmetric.
    let mut ctx = TypeContext::new();
    let statements = vec![
        let_string("s", "x"),
        plus_statement(
            Expression::Literal(LiteralValue::Number(3.0)),
            Expression::Identifier("s".to_string()),
        ),
    ];
    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].code, Some(e3::TYPE_MISMATCH as u32));

    // `let s = "x"; let n = 3; s + n` — string variable + numeric variable.
    let mut ctx = TypeContext::new();
    let statements = vec![
        let_string("s", "x"),
        let_number("n", 3.0),
        plus_statement(
            Expression::Identifier("s".to_string()),
            Expression::Identifier("n".to_string()),
        ),
    ];
    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
    assert_eq!(result.diagnostics[0].code, Some(e3::TYPE_MISMATCH as u32));
}

#[test]
fn test_resolution_accepts_supported_addition_shapes() {
    // Literal-rooted concatenation `"x" + 3` stays supported (codegen concatenates).
    let mut ctx = TypeContext::new();
    let statements = vec![plus_statement(
        Expression::Literal(LiteralValue::String("x".to_string())),
        Expression::Literal(LiteralValue::Number(3.0)),
    )];
    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);

    // Literal-rooted concatenation with a numeric variable `"n=" + n`.
    let mut ctx = TypeContext::new();
    let statements = vec![
        let_number("n", 7.0),
        plus_statement(
            Expression::Literal(LiteralValue::String("n=".to_string())),
            Expression::Identifier("n".to_string()),
        ),
    ];
    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);

    // Pure integer arithmetic `let a = 1; let b = 2; a + b` is untouched.
    let mut ctx = TypeContext::new();
    let statements = vec![
        let_number("a", 1.0),
        let_number("b", 2.0),
        plus_statement(
            Expression::Identifier("a".to_string()),
            Expression::Identifier("b".to_string()),
        ),
    ];
    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);

    // Reassigning the string variable to a number clears its static string type,
    // so `let s = "x"; s = 5; s + 1` is NOT rejected (and stays correct at runtime).
    let mut ctx = TypeContext::new();
    let statements = vec![
        let_string("s", "x"),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                AssignmentExpression {
                    operator: AssignmentOperator::Assign,
                    left: Expression::Identifier("s".to_string()),
                    right: Expression::Literal(LiteralValue::Number(5.0)),
                },
            ))),
        }),
        plus_statement(
            Expression::Identifier("s".to_string()),
            Expression::Literal(LiteralValue::Number(1.0)),
        ),
    ];
    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}
