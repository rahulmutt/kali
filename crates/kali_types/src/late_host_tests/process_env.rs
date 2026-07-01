use super::*;

#[test]
fn test_resolution_reports_deno_args_as_unavailable_on_browser_surface() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    let mut ctx = TypeContext::with_base_path_and_api_surface(&source_path, "browser");
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::MemberExpression(Box::new(
            kali_ast::MemberExpression {
                computed_index: None,
                object: Expression::Identifier("Deno".to_string()),
                property: "args".to_string(),
            },
        ))),
    })];

    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("Deno.args"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_allows_process_pid_query_in_node_api_surface() {
    let mut ctx = TypeContext::with_base_path_and_api_surface(".", "node");
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("process".to_string()),
                    property: "pid".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "process".to_string(),
                    })),
                    property: "pid".to_string(),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_resolution_allows_process_cwd_query_in_node_api_surface() {
    let mut ctx = TypeContext::with_base_path_and_api_surface(".", "node");
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("process".to_string()),
                    property: "cwd".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "process".to_string(),
                    })),
                    property: "cwd".to_string(),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_resolution_allows_process_chdir_mutation_in_node_api_surface() {
    let mut ctx = TypeContext::with_base_path_and_api_surface(".", "node");
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("process".to_string()),
                    property: "chdir".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "process".to_string(),
                    })),
                    property: "chdir".to_string(),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_resolution_allows_deno_cwd_query_in_default_standalone_surface() {
    let mut ctx = TypeContext::with_base_path_and_api_surface(".", "deno");
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Deno".to_string()),
                    property: "cwd".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Deno".to_string(),
                    })),
                    property: "cwd".to_string(),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_resolution_allows_deno_chdir_mutation_in_default_standalone_surface() {
    let mut ctx = TypeContext::with_base_path_and_api_surface(".", "deno");
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Deno".to_string()),
                    property: "chdir".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Deno".to_string(),
                    })),
                    property: "chdir".to_string(),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_resolution_allows_deno_exit_termination_in_default_standalone_surface() {
    let mut ctx = TypeContext::with_base_path_and_api_surface(".", "deno");
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Deno".to_string()),
                    property: "exit".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Deno".to_string(),
                    })),
                    property: "exit".to_string(),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_resolution_supports_env_snapshot_materialization_on_default_surface() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("Deno".to_string()),
                        property: "env".to_string(),
                    })),
                    property: "toObject".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: sequence_expression(vec![
                        Expression::Literal(LiteralValue::Number(0.0)),
                        Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                            computed_index: None,
                            object: Expression::MemberExpression(Box::new(
                                kali_ast::MemberExpression {
                                    computed_index: None,
                                    object: Expression::Identifier("globalThis".to_string()),
                                    property: "Deno".to_string(),
                                },
                            )),
                            property: "env".to_string(),
                        })),
                    ]),
                    property: "toObject".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("Deno".to_string()),
                        property: "env".to_string(),
                    })),
                    property: "toObject".to_string(),
                })),
                args: vec![],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::MemberExpression(Box::new(MemberExpression {
                            computed_index: None,
                            object: Expression::Identifier("globalThis".to_string()),
                            property: "Deno".to_string(),
                        })),
                        property: "env".to_string(),
                    })),
                    property: "toObject".to_string(),
                })),
                args: vec![],
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_resolution_rejects_process_env_assignment_as_unavailable_in_node_api_surface() {
    let mut ctx = TypeContext::with_api_surface("node");
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                AssignmentExpression {
                    operator: AssignmentOperator::Assign,
                    left: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("process".to_string()),
                        property: "env".to_string(),
                    })),
                    right: Expression::Literal(LiteralValue::Number(1.0)),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                AssignmentExpression {
                    operator: AssignmentOperator::Assign,
                    left: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::MemberExpression(Box::new(MemberExpression {
                            computed_index: None,
                            object: Expression::Identifier("globalThis".to_string()),
                            property: "process".to_string(),
                        })),
                        property: "env".to_string(),
                    })),
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
        .any(|diag| diag.message.contains("process.env")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("globalThis.process.env")));
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.message.contains("later mutable env path")));
}

#[test]
fn test_resolution_allows_bracketed_deno_env_mutation_in_default_standalone_surface() {
    let mut ctx = TypeContext::with_base_path_and_api_surface(".", "deno");
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("Deno".to_string()),
                        property: "env".to_string(),
                    })),
                    property: "set".to_string(),
                })),
                args: vec![
                    Expression::Literal(LiteralValue::String("KALI_FLAG".to_string())),
                    Expression::Literal(LiteralValue::String("1".to_string())),
                ],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::MemberExpression(Box::new(MemberExpression {
                            computed_index: None,
                            object: Expression::Identifier("globalThis".to_string()),
                            property: "Deno".to_string(),
                        })),
                        property: "env".to_string(),
                    })),
                    property: "delete".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::String(
                    "KALI_FLAG".to_string(),
                ))],
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_resolution_rejects_bracketed_env_mutation_as_unavailable_in_browser_api_surface() {
    let mut ctx = TypeContext::with_api_surface("browser");
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("Deno".to_string()),
                        property: "env".to_string(),
                    })),
                    property: "set".to_string(),
                })),
                args: vec![
                    Expression::Literal(LiteralValue::String("KALI_FLAG".to_string())),
                    Expression::Literal(LiteralValue::String("1".to_string())),
                ],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::MemberExpression(Box::new(MemberExpression {
                            computed_index: None,
                            object: Expression::Identifier("globalThis".to_string()),
                            property: "Deno".to_string(),
                        })),
                        property: "env".to_string(),
                    })),
                    property: "delete".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::String(
                    "KALI_FLAG".to_string(),
                ))],
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
        .any(|diag| diag.message.contains(r#"Deno["env"]["set"]"#)));
    assert!(result.diagnostics.iter().any(|diag| diag
        .message
        .contains(r#"globalThis["Deno"]["env"]["delete"]"#)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("browser API surface")));
}

#[test]
fn test_resolution_rejects_process_env_property_mutation_as_unavailable_in_browser_api_surface() {
    let mut ctx = TypeContext::with_api_surface("browser");
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                AssignmentExpression {
                    operator: AssignmentOperator::Assign,
                    left: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::MemberExpression(Box::new(MemberExpression {
                            computed_index: None,
                            object: Expression::MemberExpression(Box::new(MemberExpression {
                                computed_index: None,
                                object: Expression::Identifier("globalThis".to_string()),
                                property: "process".to_string(),
                            })),
                            property: "env".to_string(),
                        })),
                        property: "KALI_BROWSER_ENV_MUTATION".to_string(),
                    })),
                    right: Expression::Literal(LiteralValue::String("set".to_string())),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::UnaryExpression(Box::new(UnaryExpression {
                operator: "delete".to_string(),
                argument: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::MemberExpression(Box::new(MemberExpression {
                            computed_index: None,
                            object: Expression::Identifier("globalThis".to_string()),
                            property: "process".to_string(),
                        })),
                        property: "env".to_string(),
                    })),
                    property: "KALI_BROWSER_ENV_DELETE".to_string(),
                })),
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 2);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result.diagnostics.iter().any(|diag| diag
        .message
        .contains("globalThis.process.env.KALI_BROWSER_ENV_MUTATION")));
    assert!(result.diagnostics.iter().any(|diag| diag
        .message
        .contains("globalThis.process.env.KALI_BROWSER_ENV_DELETE")));
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.message.contains("browser API surface")));
}
