use super::*;

#[test]
fn test_resolution_reports_math_sqrt_as_available_for_perfect_square_integer_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                computed_index: None,
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
fn test_resolution_supports_math_max_and_min_member_calls_through_object_freeze_wrappers() {
    let mut ctx = TypeContext::new();
    let statements = vec![
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
                        object: Expression::Identifier("Math".to_string()),
                        property: "max".to_string(),
                    }))],
                })),
                args: vec![
                    Expression::Literal(LiteralValue::Number(1.0)),
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
                        object: Expression::MemberExpression(Box::new(MemberExpression {
                            computed_index: None,
                            object: Expression::Identifier("globalThis".to_string()),
                            property: "Math".to_string(),
                        })),
                        property: "min".to_string(),
                    }))],
                })),
                args: vec![
                    Expression::Literal(LiteralValue::Number(3.0)),
                    Expression::Literal(LiteralValue::Number(2.0)),
                    Expression::Literal(LiteralValue::Number(1.0)),
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
fn test_resolution_supports_math_cbrt_member_calls_for_perfect_cube_integer_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                computed_index: None,
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
                computed_index: None,
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
                computed_index: None,
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
fn test_resolution_supports_math_hypot_member_calls_with_const_numeric_alias_chain() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(3.0))),
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
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "hypot".to_string(),
                })),
                args: vec![
                    Expression::Identifier("alias".to_string()),
                    Expression::Literal(LiteralValue::Number(4.0)),
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
fn test_resolution_supports_math_hypot_member_calls_with_empty_argument_list() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                computed_index: None,
                object: Expression::Identifier("Math".to_string()),
                property: "hypot".to_string(),
            })),
            args: vec![],
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
fn test_resolution_supports_math_imul_with_omitted_operands() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "imul".to_string(),
                })),
                args: vec![],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "imul".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(7.0))],
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
fn test_resolution_reports_unsupported_math_cbrt_member_calls_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                computed_index: None,
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
fn test_resolution_reports_unsupported_math_hypot_member_calls_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                computed_index: None,
                object: Expression::Identifier("Math".to_string()),
                property: "hypot".to_string(),
            })),
            args: vec![
                Expression::Literal(LiteralValue::Number(1.6)),
                Expression::Literal(LiteralValue::Number(2.0)),
            ],
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
        .any(|diag| diag.message.contains("Math.hypot")
            && diag.message.contains("perfect-square integer literal")));
}

#[test]
fn test_resolution_reports_unsupported_math_log2_member_calls_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                computed_index: None,
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
                computed_index: None,
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
fn test_resolution_supports_math_exp_and_log_exact_identity_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "one".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "exp".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "log".to_string(),
                })),
                args: vec![Expression::Identifier("one".to_string())],
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
fn test_resolution_supports_math_exp2_exact_identity_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Identifier("zero".to_string())),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "exp2".to_string(),
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
fn test_resolution_supports_math_exp2_non_negative_integer_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "exponent".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(2.0))),
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
                    property: "exp2".to_string(),
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
fn test_resolution_rejects_math_exp2_non_integer_literals_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                computed_index: None,
                object: Expression::Identifier("Math".to_string()),
                property: "exp2".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(1.5))],
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
        .any(|diag| diag.message.contains("Math.exp2")
            && diag.message.contains("non-negative integer")));
}

#[test]
fn test_resolution_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "one".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "atan2".to_string(),
                })),
                args: vec![
                    Expression::Identifier("zero".to_string()),
                    Expression::Identifier("one".to_string()),
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
fn test_resolution_traverses_extra_math_atan2_arguments_after_the_supported_slice() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "one".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "atan2".to_string(),
                })),
                args: vec![
                    Expression::Identifier("zero".to_string()),
                    Expression::Identifier("one".to_string()),
                    Expression::Identifier("missing".to_string()),
                ],
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diag| diag.code == Some(e3::UNDEFINED_IDENTIFIER as u32)),
        "expected the trailing argument to be resolved: {:?}",
        result.diagnostics
    );
    assert!(
        !result
            .diagnostics
            .iter()
            .any(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)),
        "unexpected unsupported-feature diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_traverses_extra_math_tan_arguments_after_the_supported_slice() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "tan".to_string(),
                })),
                args: vec![
                    Expression::Identifier("zero".to_string()),
                    Expression::Identifier("missing".to_string()),
                ],
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diag| diag.code == Some(e3::UNDEFINED_IDENTIFIER as u32)),
        "expected the trailing argument to be resolved: {:?}",
        result.diagnostics
    );
    assert!(
        !result
            .diagnostics
            .iter()
            .any(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)),
        "unexpected unsupported-feature diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_math_expm1_log1p_and_fround_exact_zero_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "expm1".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "log1p".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "fround".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
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
fn test_resolution_supports_math_expm1_log1p_and_fround_const_numeric_alias_chain_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Identifier("zero".to_string())),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "expm1".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "log1p".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "fround".to_string(),
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
fn test_resolution_reports_math_expm1_log1p_and_fround_non_identity_literals_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "expm1".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "log1p".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "fround".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 3);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.expm1")
            && diag.message.contains("zero numeric literal")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.log1p")
            && diag.message.contains("zero numeric literal")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.fround")
            && diag.message.contains("zero numeric literal")));
}

#[test]
fn test_resolution_supports_math_asin_acos_atan_exact_identity_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "one".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "asin".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "acos".to_string(),
                })),
                args: vec![Expression::Identifier("one".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "atan".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
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
fn test_resolution_supports_math_asinh_acosh_atanh_exact_identity_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "one".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "asinh".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "acosh".to_string(),
                })),
                args: vec![Expression::Identifier("one".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "atanh".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
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
fn test_resolution_supports_math_sinh_cosh_tanh_exact_identity_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "sinh".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "cosh".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "tanh".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
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
fn test_resolution_reports_math_sinh_cosh_tanh_non_identity_literals_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "sinh".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "cosh".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "tanh".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 3);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.sinh")
            && diag.message.contains("zero numeric literal")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.cosh")
            && diag.message.contains("zero numeric literal")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.tanh")
            && diag.message.contains("zero numeric literal")));
}

#[test]
fn test_resolution_reports_math_asinh_acosh_atanh_non_identity_literals_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "asinh".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "acosh".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(0.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "atanh".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 3);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.asinh")
            && diag.message.contains("zero numeric literal")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.acosh")
            && diag.message.contains("one numeric literal")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.atanh")
            && diag.message.contains("zero numeric literal")));
}

#[test]
fn test_resolution_reports_math_atan2_non_matching_literals_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                computed_index: None,
                object: Expression::Identifier("Math".to_string()),
                property: "atan2".to_string(),
            })),
            args: vec![
                Expression::Literal(LiteralValue::Number(1.0)),
                Expression::Literal(LiteralValue::Number(1.0)),
            ],
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
        .any(|diag| diag.message.contains("Math.atan2")));
}

#[test]
fn test_resolution_supports_math_atan2_member_calls_with_const_numeric_alias_chain() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero_alias".to_string(),
                init: Some(Expression::Identifier("zero".to_string())),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "one".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "one_alias".to_string(),
                init: Some(Expression::Identifier("one".to_string())),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "atan2".to_string(),
                })),
                args: vec![
                    Expression::Identifier("zero_alias".to_string()),
                    Expression::Identifier("one_alias".to_string()),
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
fn test_resolution_reports_math_max_without_arguments_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                computed_index: None,
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
                    computed_index: None,
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
                    computed_index: None,
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
fn test_resolution_supports_math_tan_zero_literal_member_calls() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                computed_index: None,
                object: Expression::Identifier("Math".to_string()),
                property: "tan".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(0.0))],
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
fn test_resolution_supports_math_sin_cos_zero_literal_member_calls() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "sin".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(0.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "cos".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(0.0))],
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
fn test_resolution_rejects_non_zero_literals_in_math_tan_member_calls_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                computed_index: None,
                object: Expression::Identifier("Math".to_string()),
                property: "tan".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(1.0))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(result.diagnostics[0].message.contains("Math.tan"));
}

#[test]
fn test_resolution_rejects_non_zero_literals_in_math_sin_cos_member_calls_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "sin".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "cos".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
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
        .any(|diag| diag.message.contains("Math.sin")
            && diag.message.contains("zero numeric literal")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.cos")
            && diag.message.contains("zero numeric literal")));
}

#[test]
fn test_resolution_reports_non_identity_literals_in_math_asin_acos_atan_member_calls_as_unavailable(
) {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "asin".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "acos".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(0.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Math".to_string()),
                    property: "atan".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 3);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.asin")
            && diag.message.contains("zero numeric literal")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.acos")
            && diag.message.contains("one numeric literal")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.atan")
            && diag.message.contains("zero numeric literal")));
}
