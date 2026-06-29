use super::*;

#[test]
fn test_resolution_rejects_class_method_generator_lowering() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");

    let statements = vec![Statement::ClassDeclaration(ClassDeclaration {
        name: "Example".to_string(),
        body: Box::new(ClassBody {
            methods: vec![MethodDefinition {
                name: "main".to_string(),
                params: vec![],
                body: Some(Box::new(BlockStatement {
                    body: vec![Statement::ExpressionStatement(ExpressionStatement {
                        expression: Box::new(Expression::YieldExpression(Box::new(
                            YieldExpression {
                                delegate: true,
                                argument: Some(Expression::ArrayExpression(
                                    kali_ast::ArrayExpression { elements: vec![] },
                                )),
                            },
                        ))),
                    })],
                })),
                is_async: false,
                generator: true,
            }],
        }),
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
        .contains("generator class method lowering is unavailable"));
}

#[test]
fn test_resolution_rejects_async_class_method_generator_lowering() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");

    let statements = vec![Statement::ClassDeclaration(ClassDeclaration {
        name: "Example".to_string(),
        body: Box::new(ClassBody {
            methods: vec![MethodDefinition {
                name: "main".to_string(),
                params: vec![],
                body: Some(Box::new(BlockStatement {
                    body: vec![Statement::ExpressionStatement(ExpressionStatement {
                        expression: Box::new(Expression::YieldExpression(Box::new(
                            YieldExpression {
                                delegate: true,
                                argument: Some(Expression::ArrayExpression(
                                    kali_ast::ArrayExpression { elements: vec![] },
                                )),
                            },
                        ))),
                    })],
                })),
                is_async: true,
                generator: true,
            }],
        }),
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
        .contains("async-generator class method lowering is unavailable"));
}

#[test]
fn test_resolution_collapses_mixed_generator_class_method_lowering() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");

    let statements = vec![Statement::ClassDeclaration(ClassDeclaration {
        name: "Example".to_string(),
        body: Box::new(ClassBody {
            methods: vec![
                MethodDefinition {
                    name: "syncGen".to_string(),
                    params: vec![],
                    body: Some(Box::new(BlockStatement {
                        body: vec![Statement::ExpressionStatement(ExpressionStatement {
                            expression: Box::new(Expression::YieldExpression(Box::new(
                                YieldExpression {
                                    delegate: true,
                                    argument: Some(Expression::ArrayExpression(
                                        kali_ast::ArrayExpression { elements: vec![] },
                                    )),
                                },
                            ))),
                        })],
                    })),
                    is_async: false,
                    generator: true,
                },
                MethodDefinition {
                    name: "asyncGen".to_string(),
                    params: vec![],
                    body: Some(Box::new(BlockStatement {
                        body: vec![Statement::ExpressionStatement(ExpressionStatement {
                            expression: Box::new(Expression::YieldExpression(Box::new(
                                YieldExpression {
                                    delegate: true,
                                    argument: Some(Expression::ArrayExpression(
                                        kali_ast::ArrayExpression { elements: vec![] },
                                    )),
                                },
                            ))),
                        })],
                    })),
                    is_async: true,
                    generator: true,
                },
                MethodDefinition {
                    name: "plain".to_string(),
                    params: vec![],
                    body: Some(Box::new(BlockStatement {
                        body: vec![Statement::ReturnStatement(kali_ast::ReturnStatement {
                            argument: Some(Expression::Literal(LiteralValue::Number(1.0))),
                        })],
                    })),
                    is_async: false,
                    generator: false,
                },
            ],
        }),
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
        .contains("generator and async-generator class method lowering is unavailable"));
}

#[test]
fn test_resolution_rejects_generator_class_expression_lowering() {
    let cases = [
        (
            Statement::VariableDeclaration(VariableDeclaration {
                declarations: vec![VariableDeclarator {
                    id: "Example".to_string(),
                    init: Some(Expression::ClassExpression(Box::new(ClassExpression {
                        id: Some("NamedExample".to_string()),
                        body: Box::new(ClassBody {
                            methods: vec![MethodDefinition {
                                name: "main".to_string(),
                                params: vec![],
                                body: Some(Box::new(BlockStatement {
                                    body: vec![Statement::ExpressionStatement(
                                        ExpressionStatement {
                                            expression: Box::new(Expression::YieldExpression(
                                                Box::new(YieldExpression {
                                                    delegate: true,
                                                    argument: Some(Expression::ArrayExpression(
                                                        kali_ast::ArrayExpression {
                                                            elements: vec![],
                                                        },
                                                    )),
                                                }),
                                            )),
                                        },
                                    )],
                                })),
                                is_async: false,
                                generator: true,
                            }],
                        }),
                    }))),
                }],
                kind: "const".to_string(),
            }),
            "generator class method lowering is unavailable",
        ),
        (
            Statement::VariableDeclaration(VariableDeclaration {
                declarations: vec![VariableDeclarator {
                    id: "Example".to_string(),
                    init: Some(Expression::ClassExpression(Box::new(ClassExpression {
                        id: Some("NamedExample".to_string()),
                        body: Box::new(ClassBody {
                            methods: vec![MethodDefinition {
                                name: "main".to_string(),
                                params: vec![],
                                body: Some(Box::new(BlockStatement {
                                    body: vec![Statement::ExpressionStatement(
                                        ExpressionStatement {
                                            expression: Box::new(Expression::YieldExpression(
                                                Box::new(YieldExpression {
                                                    delegate: true,
                                                    argument: Some(Expression::ArrayExpression(
                                                        kali_ast::ArrayExpression {
                                                            elements: vec![],
                                                        },
                                                    )),
                                                }),
                                            )),
                                        },
                                    )],
                                })),
                                is_async: true,
                                generator: true,
                            }],
                        }),
                    }))),
                }],
                kind: "const".to_string(),
            }),
            "async-generator class method lowering is unavailable",
        ),
        (
            Statement::ExportDefault(ExportDefaultDeclaration::Expression(
                Expression::ClassExpression(Box::new(ClassExpression {
                    id: Some("NamedExample".to_string()),
                    body: Box::new(ClassBody {
                        methods: vec![MethodDefinition {
                            name: "main".to_string(),
                            params: vec![],
                            body: Some(Box::new(BlockStatement {
                                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                                    expression: Box::new(Expression::YieldExpression(Box::new(
                                        YieldExpression {
                                            delegate: true,
                                            argument: Some(Expression::ArrayExpression(
                                                kali_ast::ArrayExpression { elements: vec![] },
                                            )),
                                        },
                                    ))),
                                })],
                            })),
                            is_async: false,
                            generator: true,
                        }],
                    }),
                })),
            )),
            "generator class method lowering is unavailable",
        ),
        (
            Statement::ExportDefault(ExportDefaultDeclaration::Expression(
                Expression::ClassExpression(Box::new(ClassExpression {
                    id: Some("NamedExample".to_string()),
                    body: Box::new(ClassBody {
                        methods: vec![MethodDefinition {
                            name: "main".to_string(),
                            params: vec![],
                            body: Some(Box::new(BlockStatement {
                                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                                    expression: Box::new(Expression::YieldExpression(Box::new(
                                        YieldExpression {
                                            delegate: true,
                                            argument: Some(Expression::ArrayExpression(
                                                kali_ast::ArrayExpression { elements: vec![] },
                                            )),
                                        },
                                    ))),
                                })],
                            })),
                            is_async: true,
                            generator: true,
                        }],
                    }),
                })),
            )),
            "async-generator class method lowering is unavailable",
        ),
    ];

    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = fixtures::tempdir();
        let source_path = dir.path().join(format!("main.{extension}"));

        for (statement, expected_message) in cases.iter() {
            let mut ctx = TypeContext::with_base_path(&source_path);
            let result =
                ctx.resolve_statements_at_path(Some(&source_path), std::slice::from_ref(statement));
            assert_eq!(
                result.diagnostics.len(),
                1,
                "unexpected diagnostics for {extension}: {:?}",
                result.diagnostics
            );
            assert_eq!(
                result.diagnostics[0].code,
                Some(e5::FEATURE_UNAVAILABLE as u32)
            );
            assert!(result.diagnostics[0].message.contains(expected_message));
        }
    }
}

#[test]
fn test_resolution_collapses_mixed_generator_class_expression_lowering() {
    let statement = Statement::VariableDeclaration(VariableDeclaration {
        declarations: vec![VariableDeclarator {
            id: "Example".to_string(),
            init: Some(Expression::ClassExpression(Box::new(ClassExpression {
                id: Some("NamedExample".to_string()),
                body: Box::new(ClassBody {
                    methods: vec![
                        MethodDefinition {
                            name: "syncGen".to_string(),
                            params: vec![],
                            body: Some(Box::new(BlockStatement {
                                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                                    expression: Box::new(Expression::YieldExpression(Box::new(
                                        YieldExpression {
                                            delegate: true,
                                            argument: Some(Expression::ArrayExpression(
                                                kali_ast::ArrayExpression { elements: vec![] },
                                            )),
                                        },
                                    ))),
                                })],
                            })),
                            is_async: false,
                            generator: true,
                        },
                        MethodDefinition {
                            name: "asyncGen".to_string(),
                            params: vec![],
                            body: Some(Box::new(BlockStatement {
                                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                                    expression: Box::new(Expression::YieldExpression(Box::new(
                                        YieldExpression {
                                            delegate: true,
                                            argument: Some(Expression::ArrayExpression(
                                                kali_ast::ArrayExpression { elements: vec![] },
                                            )),
                                        },
                                    ))),
                                })],
                            })),
                            is_async: true,
                            generator: true,
                        },
                    ],
                }),
            }))),
        }],
        kind: "const".to_string(),
    });

    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = fixtures::tempdir();
        let source_path = dir.path().join(format!("main.{extension}"));

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result =
            ctx.resolve_statements_at_path(Some(&source_path), std::slice::from_ref(&statement));
        assert_eq!(
            result.diagnostics.len(),
            1,
            "unexpected diagnostics for {extension}: {:?}",
            result.diagnostics
        );
        assert_eq!(
            result.diagnostics[0].code,
            Some(e5::FEATURE_UNAVAILABLE as u32)
        );
        assert!(result.diagnostics[0]
            .message
            .contains("generator and async-generator class method lowering is unavailable"));
    }
}

#[test]
fn test_resolution_supports_async_class_method_lowering() {
    let statements = vec![Statement::ClassDeclaration(ClassDeclaration {
        name: "Example".to_string(),
        body: Box::new(ClassBody {
            methods: vec![MethodDefinition {
                name: "main".to_string(),
                params: vec![],
                body: Some(Box::new(BlockStatement {
                    body: vec![Statement::ReturnStatement(kali_ast::ReturnStatement {
                        argument: Some(Expression::Literal(LiteralValue::Number(1.0))),
                    })],
                })),
                is_async: true,
                generator: false,
            }],
        }),
    })];

    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = fixtures::tempdir();
        let source_path = dir.path().join(format!("main.{extension}"));

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {extension}: {:?}",
            result.diagnostics
        );
    }
}
