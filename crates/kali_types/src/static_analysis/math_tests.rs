use crate::test_support::*;
use kali_test_support::fixtures;
use crate::*;
use kali_ast::{
    CallExpression, ConditionalExpression, DecoratedExpression, Expression, ExpressionStatement,
    LiteralValue, MemberExpression, ObjectExpression, ObjectProperty, ObjectPropertyKind,
    ParenthesizedExpression, PropertyName, VariableDeclaration, VariableDeclarator,
};
use kali_common::{
    math_abs_sign_frozen_callable_invocation_source, math_round_frozen_callable_invocation_source,
};
use kali_error::_error_codes::{e3, e5};
use std::fs;

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
fn test_resolution_supports_wrapped_call_targets_for_object_model_and_math_helpers() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::DecoratedExpression(DecoratedExpression {
                    expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                        ParenthesizedExpression {
                            expression: Box::new(Expression::MemberExpression(Box::new(
                                MemberExpression {
                                    object: Expression::Identifier("Object".to_string()),
                                    property: "hasOwn".to_string(),
                                },
                            ))),
                        },
                    ))),
                }),
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
                callee: Expression::DecoratedExpression(DecoratedExpression {
                    expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                        ParenthesizedExpression {
                            expression: Box::new(Expression::MemberExpression(Box::new(
                                MemberExpression {
                                    object: Expression::Identifier("Math".to_string()),
                                    property: "floor".to_string(),
                                },
                            ))),
                        },
                    ))),
                }),
                args: vec![Expression::Literal(LiteralValue::Number(1.6))],
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
fn test_resolution_supports_math_round_member_calls_through_optional_chain_wrappers() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "round".to_string(),
            })),
            args: vec![Expression::OptionalChainExpression(Box::new(
                OptionalChainExpression {
                    inner: Box::new(OptionalChainInner::NonNull {
                        object: Box::new(Expression::Literal(LiteralValue::Number(1.6))),
                        optional: true,
                    }),
                },
            ))],
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
fn test_resolution_supports_math_round_member_calls_through_sequence_wrappers() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "round".to_string(),
            })),
            args: vec![sequence_expression(vec![
                Expression::Literal(LiteralValue::Number(0.0)),
                Expression::Literal(LiteralValue::Number(1.6)),
            ])],
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
fn test_resolution_supports_math_round_member_calls_through_conditional_callable_wrappers() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::ConditionalExpression(Box::new(ConditionalExpression {
                test: Box::new(Expression::Literal(LiteralValue::Boolean(true))),
                consequent: Box::new(Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "round".to_string(),
                }))),
                alternate: Box::new(Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("Object".to_string()),
                        property: "freeze".to_string(),
                    })),
                    args: vec![Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("Math".to_string()),
                        property: "round".to_string(),
                    }))],
                }))),
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
fn test_resolution_supports_global_this_math_builtin_slices_for_supported_methods() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Math".to_string(),
                    })),
                    property: "min".to_string(),
                })),
                args: vec![
                    Expression::Literal(LiteralValue::Number(3.0)),
                    Expression::Literal(LiteralValue::Number(2.0)),
                    Expression::Literal(LiteralValue::Number(1.0)),
                ],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Math".to_string(),
                    })),
                    property: "abs".to_string(),
                })),
                args: vec![Expression::UnaryExpression(Box::new(
                    kali_ast::UnaryExpression {
                        operator: "-".to_string(),
                        argument: Expression::Literal(LiteralValue::Number(4.0)),
                    },
                ))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Math".to_string(),
                    })),
                    property: "sign".to_string(),
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
fn test_resolution_supports_math_max_and_min_member_calls_through_object_freeze_wrappers() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("Object".to_string()),
                        property: "freeze".to_string(),
                    })),
                    args: vec![Expression::MemberExpression(Box::new(MemberExpression {
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
                        object: Expression::Identifier("Object".to_string()),
                        property: "freeze".to_string(),
                    })),
                    args: vec![Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::MemberExpression(Box::new(MemberExpression {
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
fn test_resolution_supports_math_pow_member_calls_with_non_integer_base_for_zero_exponent() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
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
                        object: Expression::Identifier("Object".to_string()),
                        property: "freeze".to_string(),
                    })),
                    args: vec![Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::Identifier("Math".to_string()),
                    property: "imul".to_string(),
                })),
                args: vec![],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
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
fn test_resolution_supports_global_this_math_hypot_member_calls_with_empty_argument_list() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("globalThis".to_string()),
                    property: "Math".to_string(),
                })),
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
fn test_resolution_reports_unsupported_math_hypot_member_calls_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::Identifier("Math".to_string()),
                    property: "exp".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::Identifier("Math".to_string()),
                    property: "expm1".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "log1p".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::Identifier("Math".to_string()),
                    property: "expm1".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "log1p".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::Identifier("Math".to_string()),
                    property: "expm1".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "log1p".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::Identifier("Math".to_string()),
                    property: "asin".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "acos".to_string(),
                })),
                args: vec![Expression::Identifier("one".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::Identifier("Math".to_string()),
                    property: "asinh".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "acosh".to_string(),
                })),
                args: vec![Expression::Identifier("one".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::Identifier("Math".to_string()),
                    property: "sinh".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "cosh".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::Identifier("Math".to_string()),
                    property: "sinh".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "cosh".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::Identifier("Math".to_string()),
                    property: "asinh".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "acosh".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(0.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
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

#[test]
fn test_resolution_supports_math_tan_zero_literal_member_calls() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::Identifier("Math".to_string()),
                    property: "sin".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(0.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
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
fn test_resolution_supports_math_clz32_zero_argument_member_calls() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "clz32".to_string(),
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
fn test_resolution_supports_math_clz32_non_integer_literal_member_calls() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "clz32".to_string(),
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
fn test_resolution_rejects_non_zero_literals_in_math_tan_member_calls_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::Identifier("Math".to_string()),
                    property: "sin".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::Identifier("Math".to_string()),
                    property: "asin".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "acos".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(0.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
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
fn test_resolution_supports_non_integer_numeric_literals_in_math_sign_member_calls() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "sign".to_string(),
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
fn test_resolution_supports_frozen_math_expm1_and_log1p_identity_helpers_across_js_like_extensions()
{
    let source = r#"const zero = 0;
const frozenDotRoot = Object.freeze(globalThis.Math);
const frozenMixedRoot = Object.freeze(globalThis[\"Math\"]);
const frozenDirectRoot = Object.freeze(Math);
frozenDotRoot.expm1(zero);
frozenMixedRoot.expm1(zero);
frozenDirectRoot.expm1(zero);
frozenDotRoot.log1p(zero);
frozenMixedRoot.log1p(zero);
frozenDirectRoot.log1p(zero);
"#;

    for extension in ["js", "jsx", "ts", "tsx"] {
        let dir = fixtures::tempdir();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {extension}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_supports_frozen_math_pow_callable_aliases_across_js_like_extensions() {
    let source = r#"const exponent = 3;
const frozenDotRoot = Object.freeze(Math.pow);
const frozenGlobalDotRoot = Object.freeze(globalThis.Math.pow);
const frozenBracketedRoot = Object.freeze(globalThis["Math"]["pow"]);
const frozenSingleQuotedBracketedRoot = Object.freeze(globalThis['Math']['pow']);
const frozenSingleQuotedMathRoot = Object.freeze(Math['pow']);
const frozenParenthesizedDotRoot = Object.freeze((Math.pow));
const frozenParenthesizedGlobalThisDotRoot = Object.freeze((globalThis.Math))["pow"];
const frozenParenthesizedSingleQuotedGlobalThisDotRoot = Object.freeze((globalThis.Math))['pow'];
const frozenParenthesizedGlobalThisBracketedRoot = Object.freeze((globalThis["Math"]))["pow"];
const frozenParenthesizedSingleQuotedGlobalThisBracketedRoot = Object.freeze((globalThis['Math']))['pow'];
frozenDotRoot(2, exponent);
frozenGlobalDotRoot(2, exponent);
frozenBracketedRoot(2, exponent);
frozenSingleQuotedBracketedRoot(2, exponent);
frozenSingleQuotedMathRoot(2, exponent);
frozenParenthesizedDotRoot(2, exponent);
frozenParenthesizedGlobalThisDotRoot(2, exponent);
frozenParenthesizedSingleQuotedGlobalThisDotRoot(2, exponent);
frozenParenthesizedGlobalThisBracketedRoot(2, exponent);
frozenParenthesizedSingleQuotedGlobalThisBracketedRoot(2, exponent);
"#;

    for extension in ["js", "jsx", "ts", "tsx"] {
        let dir = fixtures::tempdir();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {extension}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_supports_frozen_math_abs_and_sign_callable_aliases_across_js_like_extensions() {
    let source = format!(
        "const alias = 1;\n{}",
        math_abs_sign_frozen_callable_invocation_source()
    );

    for extension in ["js", "jsx", "ts", "tsx"] {
        let dir = fixtures::tempdir();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, &source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.clone());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {extension}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_supports_frozen_math_round_callable_aliases_across_js_like_extensions() {
    let source = format!(
        "const value = 1.6;\n{}",
        math_round_frozen_callable_invocation_source()
    );

    for extension in ["js", "jsx", "ts", "tsx"] {
        let dir = fixtures::tempdir();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, &source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.clone());
        let tokens = lexer.lex_all().tokens;
        let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
        let statements = parser.parse(None).statements;

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {extension}: {:?}",
            result.diagnostics
        );
    }
}
