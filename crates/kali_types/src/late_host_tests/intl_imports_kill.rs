use super::*;

#[test]
fn test_resolution_reports_broader_intl_support_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("Intl".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Intl".to_string()),
                    property: "NumberFormat".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Intl".to_string()),
                    property: "DisplayNames".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Intl".to_string()),
                    property: "Locale".to_string(),
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
                        property: "Intl".to_string(),
                    })),
                    property: "NumberFormat".to_string(),
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
                        property: "Intl".to_string(),
                    })),
                    property: "DisplayNames".to_string(),
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
                        property: "Intl".to_string(),
                    })),
                    property: "Locale".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("globalThis".to_string()),
                    property: "Intl".to_string(),
                },
            ))),
        }),
    ];
    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.len() >= 2);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.message.contains("Intl")));
    assert!(result.diagnostics.iter().any(|diag| {
        diag.message
            .contains(r#"globalThis["Intl"]["NumberFormat"]"#)
    }));
    assert!(result.diagnostics.iter().any(|diag| {
        diag.message
            .contains(r#"globalThis["Intl"]["DisplayNames"]"#)
    }));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| { diag.message.contains(r#"globalThis["Intl"]["Locale"]"#) }));
}

#[test]
fn test_resolution_reports_global_this_intl_root_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::MemberExpression(Box::new(
            kali_ast::MemberExpression {
                computed_index: None,
                object: Expression::Identifier("globalThis".to_string()),
                property: "Intl".to_string(),
            },
        ))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(result.diagnostics[0].message.contains("globalThis.Intl"));
}

#[test]
fn test_resolution_reports_late_intl_member_access_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Intl".to_string()),
                    property: "NumberFormat".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Intl".to_string()),
                    property: "RelativeTimeFormat".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Intl".to_string()),
                    property: "Collator".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Intl".to_string()),
                    property: "DisplayNames".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Intl".to_string()),
                    property: "Segmenter".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("Intl".to_string()),
                    property: "Locale".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Intl".to_string(),
                    })),
                    property: "DisplayNames".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Intl".to_string(),
                    })),
                    property: "Locale".to_string(),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 8);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    for expected in [
        "Intl.NumberFormat",
        "Intl.RelativeTimeFormat",
        "Intl.Collator",
        "Intl.DisplayNames",
        "Intl.Segmenter",
        "Intl.Locale",
    ] {
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.message.contains(expected)),
            "missing diagnostic for {expected}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_allows_node_builtin_imports_in_node_context() {
    let mut ctx = TypeContext::with_base_path_and_api_surface(".", "node");
    assert!(ctx.is_defined("process"));

    let statements = vec![Statement::ImportDeclaration(ImportDeclaration {
        specifiers: vec![ImportSpecifier::Default("fs".to_string())],
        source: "node:fs/promises".to_string(),
    })];

    let result = ctx.resolve_statements_at_path(Some("."), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_allows_node_timers_imports_in_node_context() {
    let mut ctx = TypeContext::with_base_path_and_api_surface(".", "node");
    let statements = vec![Statement::ImportDeclaration(ImportDeclaration {
        specifiers: vec![ImportSpecifier::Default("timers".to_string())],
        source: "node:timers".to_string(),
    })];

    let result = ctx.resolve_statements_at_path(Some("."), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_rejects_node_builtin_imports_outside_node_context() {
    let mut ctx = TypeContext::with_base_path(".");
    let statements = vec![Statement::ImportDeclaration(ImportDeclaration {
        specifiers: vec![ImportSpecifier::Default("fs".to_string())],
        source: "node:fs/promises".to_string(),
    })];

    let result = ctx.resolve_statements_at_path(Some("."), &statements);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::IMPORT_NOT_FOUND as u32)
    );
}

#[test]
fn test_resolution_supports_process_kill_zero_probe_wrappers_on_node_surface() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "process.kill((0)); globalThis.process.kill(+0); globalThis.process.kill(0); globalThis.process[\"kill\"](0); globalThis.process[\"kill\"](+0); globalThis[\"process\"].kill(0); globalThis[\"process\"].kill(+0); process[\"kill\"]((0)); ((globalThis.process.kill))(0); ((globalThis[\"process\"][\"kill\"]))(+0);",
    )
    .unwrap();

    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("process".to_string()),
                    property: "kill".to_string(),
                })),
                args: vec![Expression::ParenthesizedExpression(Box::new(
                    ParenthesizedExpression {
                        expression: Box::new(Expression::Literal(LiteralValue::Number(0.0))),
                    },
                ))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "process".to_string(),
                    })),
                    property: "kill".to_string(),
                })),
                args: vec![Expression::UnaryExpression(Box::new(UnaryExpression {
                    operator: "+".to_string(),
                    argument: Expression::Literal(LiteralValue::Number(0.0)),
                }))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("process".to_string()),
                    property: "kill".to_string(),
                })),
                args: vec![Expression::ParenthesizedExpression(Box::new(
                    ParenthesizedExpression {
                        expression: Box::new(Expression::Literal(LiteralValue::Number(0.0))),
                    },
                ))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "process".to_string(),
                    })),
                    property: "kill".to_string(),
                })),
                args: vec![Expression::UnaryExpression(Box::new(UnaryExpression {
                    operator: "+".to_string(),
                    argument: Expression::Literal(LiteralValue::Number(0.0)),
                }))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::ParenthesizedExpression(Box::new(ParenthesizedExpression {
                    expression: Box::new(Expression::MemberExpression(Box::new(
                        MemberExpression {
                            computed_index: None,
                            object: Expression::MemberExpression(Box::new(MemberExpression {
                                computed_index: None,
                                object: Expression::Identifier("globalThis".to_string()),
                                property: "process".to_string(),
                            })),
                            property: "kill".to_string(),
                        },
                    ))),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(0.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "process".to_string(),
                    })),
                    property: "kill".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(0.0))],
            }))),
        }),
    ];

    let mut ctx = TypeContext::with_base_path_and_api_surface(&source_path, "node");
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_bracketed_process_kill_zero_probe_wrappers_in_js_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    let mut source = process_kill_zero_probe_source();
    source.push_str(" const killer = process.kill; const bracketedKiller = globalThis[\"process\"][\"kill\"]; const sequenceKiller = (process.kill, process.kill); killer(0); bracketedKiller(+0); sequenceKiller(0);");
    fs::write(&source_path, &source).unwrap();

    let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let result = TypeContext::with_base_path_and_api_surface(&source_path, "node")
        .resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_process_kill_zero_probe_through_static_zero_aliases_on_node_surface() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const zero = 0; const zeroAlias = zero; process.kill(zeroAlias); globalThis.process.kill(+zero); globalThis.process[\"kill\"](+0); globalThis[\"process\"][\"kill\"](+0);",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![
                VariableDeclarator {
                    id: "zero".to_string(),
                    init: Some(Expression::Literal(LiteralValue::Number(0.0))),
                },
                VariableDeclarator {
                    id: "zeroAlias".to_string(),
                    init: Some(Expression::Identifier("zero".to_string())),
                },
            ],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("process".to_string()),
                    property: "kill".to_string(),
                })),
                args: vec![Expression::Identifier("zeroAlias".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "process".to_string(),
                    })),
                    property: "kill".to_string(),
                })),
                args: vec![Expression::UnaryExpression(Box::new(UnaryExpression {
                    operator: "+".to_string(),
                    argument: Expression::Identifier("zero".to_string()),
                }))],
            }))),
        }),
    ];

    let mut ctx = TypeContext::with_base_path_and_api_surface(&source_path, "node");
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_process_kill_zero_probe_satisfies_wrappers_on_node_surface() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    let source = kali_common::process_kill_zero_probe_satisfies_source();
    fs::write(&source_path, source).unwrap();

    let satisfies_zero = || {
        Expression::SatisfiesExpression(Box::new(SatisfiesExpression {
            type_name: "number".to_string(),
            expression: Box::new(Expression::Literal(LiteralValue::Number(0.0))),
        }))
    };

    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::Identifier("process".to_string()),
                    property: "kill".to_string(),
                })),
                args: vec![satisfies_zero()],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "process".to_string(),
                    })),
                    property: "kill".to_string(),
                })),
                args: vec![satisfies_zero()],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "process".to_string(),
                    })),
                    property: "kill".to_string(),
                })),
                args: vec![Expression::SatisfiesExpression(Box::new(
                    SatisfiesExpression {
                        type_name: "number".to_string(),
                        expression: Box::new(Expression::Literal(LiteralValue::Number(0.0))),
                    },
                ))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "process".to_string(),
                    })),
                    property: "kill".to_string(),
                })),
                args: vec![Expression::SatisfiesExpression(Box::new(
                    SatisfiesExpression {
                        type_name: "number".to_string(),
                        expression: Box::new(Expression::Literal(LiteralValue::Number(0.0))),
                    },
                ))],
            }))),
        }),
    ];

    let mut ctx = TypeContext::with_base_path_and_api_surface(&source_path, "node");
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_rejects_process_kill_non_zero_literal_on_node_surface() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "process.kill(1);").unwrap();

    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                computed_index: None,
                object: Expression::Identifier("process".to_string()),
                property: "kill".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(1.0))],
        }))),
    })];

    let mut ctx = TypeContext::with_base_path_and_api_surface(&source_path, "node");
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
    assert!(
        result.diagnostics[0].message.contains("process.kill(0)"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"process["kill"](0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"globalThis.process["kill"](0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"globalThis.process.kill((0))"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"globalThis["process"].kill((0))"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"globalThis["process"].kill(0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"globalThis["process"]["kill"](0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"Object.freeze(globalThis.process["kill"])(0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"Object.freeze(globalThis.process["kill"])(+0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"Object.freeze((process))["kill"](0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"Object.freeze((process))["kill"](+0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"Object.freeze((globalThis.process))["kill"](0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"Object.freeze((globalThis.process))["kill"](+0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"Object.freeze((globalThis["process"]))["kill"](0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"Object.freeze((globalThis["process"]))["kill"](+0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"Object.freeze(process)["kill"](+0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"Object.freeze((process)["kill"])(0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"Object.freeze((process)["kill"])(+0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"Object.freeze(globalThis.process)["kill"](+0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"Object.freeze(globalThis["process"].kill)(0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"Object.freeze(globalThis["process"].kill)(+0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"((process["kill"]))(0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"((process["kill"]))(+0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"Object.freeze((globalThis.process["kill"]))(0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"Object.freeze((globalThis.process["kill"]))(+0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"((globalThis["process"]["kill"]))(0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains(r#"((globalThis["process"]["kill"]))(+0)"#),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}
