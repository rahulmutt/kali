use super::*;

#[test]
fn test_resolution_supports_math_pow_member_calls_for_integer_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                computed_index: None,
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
fn test_resolution_supports_math_pow_member_calls_with_non_integer_base_for_zero_exponent() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                computed_index: None,
                object: Expression::Identifier("Math".to_string()),
                property: "pow".to_string(),
            })),
            args: vec![
                Expression::Literal(LiteralValue::Number(1.6)),
                Expression::Literal(LiteralValue::Number(0.0)),
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
fn test_resolution_supports_math_pow_member_calls_with_zero_base_and_positive_integer_exponent() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "exponent".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(3.0))),
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
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "pow".to_string(),
                })),
                args: vec![
                    Expression::Literal(LiteralValue::Number(0.0)),
                    Expression::Identifier("alias".to_string()),
                ],
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
fn test_resolution_supports_math_pow_member_calls_with_const_numeric_alias_exponents() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "exponent".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(3.0))),
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
                    computed_index: None,
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
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_math_pow_member_calls_with_negative_integer_base_and_const_numeric_alias_exponents(
) {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "exponent".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(3.0))),
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
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "pow".to_string(),
                })),
                args: vec![
                    Expression::UnaryExpression(Box::new(kali_ast::UnaryExpression {
                        operator: "-".to_string(),
                        argument: Expression::Literal(LiteralValue::Number(2.0)),
                    })),
                    Expression::Identifier("alias".to_string()),
                ],
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
fn test_resolution_supports_math_pow_member_calls_with_negative_integer_exponent_for_unit_bases() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "negative_exponent".to_string(),
                init: Some(Expression::UnaryExpression(Box::new(
                    kali_ast::UnaryExpression {
                        operator: "-".to_string(),
                        argument: Expression::Literal(LiteralValue::Number(3.0)),
                    },
                ))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Identifier("negative_exponent".to_string())),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "pow".to_string(),
                })),
                args: vec![
                    Expression::Literal(LiteralValue::Number(1.0)),
                    Expression::Identifier("alias".to_string()),
                ],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "pow".to_string(),
                })),
                args: vec![
                    Expression::UnaryExpression(Box::new(kali_ast::UnaryExpression {
                        operator: "-".to_string(),
                        argument: Expression::Literal(LiteralValue::Number(1.0)),
                    })),
                    Expression::Identifier("alias".to_string()),
                ],
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
fn test_resolution_reports_unsupported_math_pow_negative_exponents_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                computed_index: None,
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
fn test_resolution_reports_optional_chain_wrapped_math_pow_member_calls_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: optional_chain_global_this_math(),
                    property: "pow".to_string(),
                })),
                args: vec![
                    Expression::Literal(LiteralValue::Number(2.0)),
                    Expression::Literal(LiteralValue::Number(3.0)),
                ],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("Object".to_string()),
                        property: "freeze".to_string(),
                    })),
                    args: vec![Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: optional_chain_global_this_math(),
                        property: "pow".to_string(),
                    }))],
                })),
                args: vec![
                    Expression::Literal(LiteralValue::Number(2.0)),
                    Expression::Literal(LiteralValue::Number(3.0)),
                ],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: optional_chain_global_this_math_pow(),
                    property: "call".to_string(),
                })),
                args: vec![
                    Expression::Literal(LiteralValue::Number(2.0)),
                    Expression::Literal(LiteralValue::Number(3.0)),
                ],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("Object".to_string()),
                        property: "freeze".to_string(),
                    })),
                    args: vec![optional_chain_global_this_math_pow()],
                })),
                args: vec![
                    Expression::Literal(LiteralValue::Number(2.0)),
                    Expression::Literal(LiteralValue::Number(3.0)),
                ],
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 4);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("optional-chain wrappers")));
}

#[test]
fn test_resolution_reports_math_pow_with_single_argument_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                computed_index: None,
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
                    computed_index: None,
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
fn test_resolution_rejects_non_integer_const_numeric_alias_exponents_in_math_pow_member_calls_as_unavailable(
) {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "exponent".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.6))),
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
                    computed_index: None,
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
        .contains("non-integer numeric literals"));
}
