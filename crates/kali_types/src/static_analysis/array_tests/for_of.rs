use super::*;

#[test]
fn test_resolution_supports_for_of_array_iteration() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "for (const value of [1, 2]) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![Statement::ForOfStatement(ForOfStatement {
        left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: None,
            }],
        }),
        right: Expression::ArrayExpression(kali_ast::ArrayExpression {
            elements: vec![
                Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::Literal(LiteralValue::Number(1.0)),
                )),
                Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::Literal(LiteralValue::Number(2.0)),
                )),
            ],
        }),
        body: Box::new(Statement::BlockStatement(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("console".to_string()),
                        property: "log".to_string(),
                    })),
                    args: vec![Expression::Identifier("value".to_string())],
                }))),
            })],
        })),
        is_await: false,
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
fn test_resolution_supports_for_of_array_iteration_in_js_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "for (const value of [1, 2]) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![Statement::ForOfStatement(ForOfStatement {
        left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: None,
            }],
        }),
        right: Expression::ArrayExpression(kali_ast::ArrayExpression {
            elements: vec![
                Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::Literal(LiteralValue::Number(1.0)),
                )),
                Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::Literal(LiteralValue::Number(2.0)),
                )),
            ],
        }),
        body: Box::new(Statement::BlockStatement(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("console".to_string()),
                        property: "log".to_string(),
                    })),
                    args: vec![Expression::Identifier("value".to_string())],
                }))),
            })],
        })),
        is_await: false,
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
fn test_resolution_supports_for_of_array_iteration_with_sequence_wrappers_in_js_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "let value = 0; for ((0, value) of (0, [(0, 1), (0, 2)])) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(sequence_expression(vec![
                Expression::Literal(LiteralValue::Number(0.0)),
                Expression::Identifier("value".to_string()),
            ])),
            right: sequence_expression(vec![
                Expression::Literal(LiteralValue::Number(0.0)),
                Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            sequence_expression(vec![
                                Expression::Literal(LiteralValue::Number(0.0)),
                                Expression::Literal(LiteralValue::Number(1.0)),
                            ]),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            sequence_expression(vec![
                                Expression::Literal(LiteralValue::Number(0.0)),
                                Expression::Literal(LiteralValue::Number(2.0)),
                            ]),
                        )),
                    ],
                }),
            ]),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("value".to_string())],
                    }))),
                })],
            })),
            is_await: false,
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
fn test_resolution_rejects_for_of_non_literal_iterable_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                kali_ast::AssignmentExpression {
                    operator: AssignmentOperator::Assign,
                    left: Expression::Identifier("values".to_string()),
                    right: Expression::ArrayExpression(kali_ast::ArrayExpression {
                        elements: vec![
                            Some(kali_ast::ExpressionOrSpread::Expression(
                                Expression::Literal(LiteralValue::Number(3.0)),
                            )),
                            Some(kali_ast::ExpressionOrSpread::Expression(
                                Expression::Literal(LiteralValue::Number(4.0)),
                            )),
                        ],
                    }),
                },
            ))),
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "item".to_string(),
                    init: None,
                }],
            }),
            right: Expression::Identifier("values".to_string()),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::Identifier("item".to_string())),
                })],
            })),
            is_await: false,
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("literal array"),
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_for_of_array_iteration_with_parenthesized_binding_in_js_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "let value = 0; for ((value) of [1, 2]) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(Expression::ParenthesizedExpression(Box::new(
                kali_ast::ParenthesizedExpression {
                    expression: Box::new(Expression::Identifier("value".to_string())),
                },
            ))),
            right: Expression::ArrayExpression(kali_ast::ArrayExpression {
                elements: vec![
                    Some(kali_ast::ExpressionOrSpread::Expression(
                        Expression::Literal(LiteralValue::Number(1.0)),
                    )),
                    Some(kali_ast::ExpressionOrSpread::Expression(
                        Expression::Literal(LiteralValue::Number(2.0)),
                    )),
                ],
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("value".to_string())],
                    }))),
                })],
            })),
            is_await: false,
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
fn test_resolution_supports_for_of_array_iteration_with_parenthesized_binding_in_ts_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "let value = 0; for ((value) of [1, 2]) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(Expression::ParenthesizedExpression(Box::new(
                kali_ast::ParenthesizedExpression {
                    expression: Box::new(Expression::Identifier("value".to_string())),
                },
            ))),
            right: Expression::ArrayExpression(kali_ast::ArrayExpression {
                elements: vec![
                    Some(kali_ast::ExpressionOrSpread::Expression(
                        Expression::Literal(LiteralValue::Number(1.0)),
                    )),
                    Some(kali_ast::ExpressionOrSpread::Expression(
                        Expression::Literal(LiteralValue::Number(2.0)),
                    )),
                ],
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("value".to_string())],
                    }))),
                })],
            })),
            is_await: false,
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
fn test_resolution_rejects_for_of_array_iteration_with_identifier_iterable() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const items = [1, 2];\nfor (const value of items) { console.log(value); }\n",
    )
    .unwrap();

    let statements = vec![Statement::ForOfStatement(ForOfStatement {
        left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: None,
            }],
        }),
        right: Expression::Identifier("items".to_string()),
        body: Box::new(Statement::BlockStatement(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("console".to_string()),
                        property: "log".to_string(),
                    })),
                    args: vec![Expression::Identifier("value".to_string())],
                }))),
            })],
        })),
        is_await: false,
    })];

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
}

#[test]
fn test_resolution_supports_for_of_array_iteration_with_const_alias_in_js_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const values = [1, 2]; for (const value of values) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "value".to_string(),
                    init: None,
                }],
            }),
            right: Expression::Identifier("values".to_string()),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("value".to_string())],
                    }))),
                })],
            })),
            is_await: false,
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
fn test_resolution_supports_for_of_array_iteration_with_const_alias_in_ts_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const values = [1, 2]; for (const value of values) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "value".to_string(),
                    init: None,
                }],
            }),
            right: Expression::Identifier("values".to_string()),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("value".to_string())],
                    }))),
                })],
            })),
            is_await: false,
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
fn test_resolution_supports_for_of_array_iteration_with_let_binding_in_js_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "let values = [1, 2]; for (const value of values) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "value".to_string(),
                    init: None,
                }],
            }),
            right: Expression::Identifier("values".to_string()),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("value".to_string())],
                    }))),
                })],
            })),
            is_await: false,
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
fn test_resolution_rejects_for_of_array_iteration_with_let_binding_rebound_before_use_in_js_input()
{
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "let values = [1, 2]; values = [3, 4]; for (const value of values) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                AssignmentExpression {
                    operator: AssignmentOperator::Assign,
                    left: Expression::Identifier("values".to_string()),
                    right: Expression::ArrayExpression(kali_ast::ArrayExpression {
                        elements: vec![
                            Some(kali_ast::ExpressionOrSpread::Expression(
                                Expression::Literal(LiteralValue::Number(3.0)),
                            )),
                            Some(kali_ast::ExpressionOrSpread::Expression(
                                Expression::Literal(LiteralValue::Number(4.0)),
                            )),
                        ],
                    }),
                },
            ))),
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "value".to_string(),
                    init: None,
                }],
            }),
            right: Expression::Identifier("values".to_string()),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("value".to_string())],
                    }))),
                })],
            })),
            is_await: false,
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
}

#[test]
fn test_resolution_supports_for_of_array_iteration_with_const_numeric_alias_in_js_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = 1; const alias = value; for (const item of [alias]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Identifier("value".to_string())),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "item".to_string(),
                    init: None,
                }],
            }),
            right: Expression::ArrayExpression(kali_ast::ArrayExpression {
                elements: vec![Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::Identifier("alias".to_string()),
                ))],
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("item".to_string())],
                    }))),
                })],
            })),
            is_await: false,
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
fn test_resolution_supports_for_of_array_iteration_with_const_numeric_alias_in_ts_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const value = 1; const alias = value; for (const item of [alias]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Identifier("value".to_string())),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "item".to_string(),
                    init: None,
                }],
            }),
            right: Expression::ArrayExpression(kali_ast::ArrayExpression {
                elements: vec![Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::Identifier("alias".to_string()),
                ))],
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("item".to_string())],
                    }))),
                })],
            })),
            is_await: false,
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
fn test_resolution_supports_for_of_array_iteration_with_decorated_wrappers_in_js_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "let item = 0; for ((item) of [1, 2]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "item".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::Identifier("item".to_string())),
                    },
                ))),
            })),
            right: Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    1.0,
                                ))),
                            }),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    2.0,
                                ))),
                            }),
                        )),
                    ],
                })),
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("item".to_string())],
                    }))),
                })],
            })),
            is_await: false,
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
fn test_resolution_supports_for_of_array_iteration_with_decorated_wrappers_in_ts_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "let item = 0; for ((item) of [1, 2]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "item".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::Identifier("item".to_string())),
                    },
                ))),
            })),
            right: Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    1.0,
                                ))),
                            }),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    2.0,
                                ))),
                            }),
                        )),
                    ],
                })),
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("item".to_string())],
                    }))),
                })],
            })),
            is_await: false,
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
fn test_resolution_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_js_input(
) {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const values = [1, 2]; for (const item of [...values]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "item".to_string(),
                    init: None,
                }],
            }),
            right: Expression::ArrayExpression(kali_ast::ArrayExpression {
                elements: vec![Some(kali_ast::ExpressionOrSpread::Spread(
                    kali_ast::SpreadElement {
                        argument: Expression::Identifier("values".to_string()),
                    },
                ))],
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("item".to_string())],
                    }))),
                })],
            })),
            is_await: false,
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
fn test_resolution_supports_for_of_array_iteration_with_decorated_spread_targets_in_ts_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "item".to_string(),
                    init: None,
                }],
            }),
            right: Expression::ArrayExpression(kali_ast::ArrayExpression {
                elements: vec![Some(kali_ast::ExpressionOrSpread::Spread(
                    kali_ast::SpreadElement {
                        argument: Expression::DecoratedExpression(DecoratedExpression {
                            expression: Box::new(Expression::Identifier("values".to_string())),
                        }),
                    },
                ))],
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("item".to_string())],
                    }))),
                })],
            })),
            is_await: false,
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
fn test_resolution_supports_for_of_array_iteration_with_decorated_spread_targets_in_js_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "item".to_string(),
                    init: None,
                }],
            }),
            right: Expression::ArrayExpression(kali_ast::ArrayExpression {
                elements: vec![Some(kali_ast::ExpressionOrSpread::Spread(
                    kali_ast::SpreadElement {
                        argument: Expression::DecoratedExpression(DecoratedExpression {
                            expression: Box::new(Expression::Identifier("values".to_string())),
                        }),
                    },
                ))],
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("item".to_string())],
                    }))),
                })],
            })),
            is_await: false,
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
