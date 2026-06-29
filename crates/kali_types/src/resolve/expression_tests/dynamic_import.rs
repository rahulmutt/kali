use super::*;

#[test]
fn test_resolution_allows_static_dynamic_import_targets() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    fs::write(dir.path().join("lazy.ts"), "export const lazy = 7;").unwrap();
    fs::write(&source_path, "const lazy = import(\"./\" + \"lazy.ts\");").unwrap();

    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
            source: Expression::BinaryExpression(Box::new(BinaryExpression {
                operator: "+".to_string(),
                left: Expression::Literal(LiteralValue::String("./".to_string())),
                right: Expression::Literal(LiteralValue::String("lazy.ts".to_string())),
            })),
        }))),
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
fn test_resolution_allows_static_dynamic_import_targets_in_js_files() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(dir.path().join("lazy.js"), "export const lazy = 7;").unwrap();
    fs::write(&source_path, "const lazy = import(\"./lazy.js\");").unwrap();

    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
            source: Expression::Literal(LiteralValue::String("./lazy.js".to_string())),
        }))),
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
fn test_resolution_allows_template_literal_dynamic_import_targets() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    fs::write(dir.path().join("lazy.ts"), "export const lazy = 7;").unwrap();
    fs::write(
        &source_path,
        "const name = \"lazy.ts\"; import(`./${name}`);",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "name".to_string(),
                init: Some(Expression::Literal(LiteralValue::String(
                    "lazy.ts".to_string(),
                ))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
                source: Expression::TemplateLiteral(TemplateLiteral {
                    quasis: vec![
                        TemplateElement {
                            value: "./".to_string(),
                            tail: false,
                        },
                        TemplateElement {
                            value: "".to_string(),
                            tail: true,
                        },
                    ],
                    expressions: vec![Expression::Identifier("name".to_string())],
                }),
            }))),
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
fn test_resolution_allows_template_literal_dynamic_import_targets_in_js_files() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(dir.path().join("lazy.js"), "export const lazy = 7;").unwrap();
    fs::write(
        &source_path,
        "const name = \"lazy.js\"; import(`./${name}`);",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "name".to_string(),
                init: Some(Expression::Literal(LiteralValue::String(
                    "lazy.js".to_string(),
                ))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
                source: Expression::TemplateLiteral(TemplateLiteral {
                    quasis: vec![
                        TemplateElement {
                            value: "./".to_string(),
                            tail: false,
                        },
                        TemplateElement {
                            value: "".to_string(),
                            tail: true,
                        },
                    ],
                    expressions: vec![Expression::Identifier("name".to_string())],
                }),
            }))),
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
fn test_resolution_allows_sequence_wrapped_template_literal_dynamic_import_targets_in_js_files() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(dir.path().join("lazy.js"), "export const lazy = 7;").unwrap();
    fs::write(
        &source_path,
        "const name = \"lazy.js\"; import((0, `./${name}`));",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "name".to_string(),
                init: Some(Expression::Literal(LiteralValue::String(
                    "lazy.js".to_string(),
                ))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
                source: sequence_expression(vec![
                    Expression::Literal(LiteralValue::Number(0.0)),
                    Expression::TemplateLiteral(TemplateLiteral {
                        quasis: vec![
                            TemplateElement {
                                value: "./".to_string(),
                                tail: false,
                            },
                            TemplateElement {
                                value: "".to_string(),
                                tail: true,
                            },
                        ],
                        expressions: vec![Expression::Identifier("name".to_string())],
                    }),
                ]),
            }))),
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
fn test_resolution_allows_const_bound_dynamic_import_targets() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    fs::write(dir.path().join("lazy.ts"), "export const lazy = 7;").unwrap();
    fs::write(
        &source_path,
        "const name = \"lazy.ts\"; const root = \"./\"; import(root + name);",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "name".to_string(),
                init: Some(Expression::Literal(LiteralValue::String(
                    "lazy.ts".to_string(),
                ))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "root".to_string(),
                init: Some(Expression::Literal(LiteralValue::String("./".to_string()))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
                source: Expression::BinaryExpression(Box::new(BinaryExpression {
                    operator: "+".to_string(),
                    left: Expression::Identifier("root".to_string()),
                    right: Expression::Identifier("name".to_string()),
                })),
            }))),
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
fn test_resolution_allows_parenthesized_dynamic_import_targets() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    fs::write(dir.path().join("lazy.ts"), "export const lazy = 7;").unwrap();
    fs::write(
        &source_path,
        "const name = \"lazy.ts\"; const root = \"./\"; import((root + name));",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "name".to_string(),
                init: Some(Expression::Literal(LiteralValue::String(
                    "lazy.ts".to_string(),
                ))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "root".to_string(),
                init: Some(Expression::Literal(LiteralValue::String("./".to_string()))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
                source: Expression::ParenthesizedExpression(Box::new(ParenthesizedExpression {
                    expression: Box::new(Expression::BinaryExpression(Box::new(
                        BinaryExpression {
                            operator: "+".to_string(),
                            left: Expression::Identifier("root".to_string()),
                            right: Expression::Identifier("name".to_string()),
                        },
                    ))),
                })),
            }))),
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
fn test_resolution_allows_parenthesized_dynamic_import_targets_in_js_files() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(dir.path().join("lazy.js"), "export const lazy = 7;").unwrap();
    fs::write(
        &source_path,
        "const name = \"lazy.js\"; const root = \"./\"; import((root + name));",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "name".to_string(),
                init: Some(Expression::Literal(LiteralValue::String(
                    "lazy.js".to_string(),
                ))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "root".to_string(),
                init: Some(Expression::Literal(LiteralValue::String("./".to_string()))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
                source: Expression::ParenthesizedExpression(Box::new(ParenthesizedExpression {
                    expression: Box::new(Expression::BinaryExpression(Box::new(
                        BinaryExpression {
                            operator: "+".to_string(),
                            left: Expression::Identifier("root".to_string()),
                            right: Expression::Identifier("name".to_string()),
                        },
                    ))),
                })),
            }))),
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
fn test_resolution_allows_sequence_wrapped_dynamic_import_targets_in_js_files() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(dir.path().join("lazy.js"), "export const lazy = 7;").unwrap();
    fs::write(&source_path, "import((0, \"./lazy.js\"));").unwrap();

    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
            source: sequence_expression(vec![
                Expression::Literal(LiteralValue::Number(0.0)),
                Expression::Literal(LiteralValue::String("./lazy.js".to_string())),
            ]),
        }))),
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
fn test_resolution_accepts_directory_index_dynamic_import_targets() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).unwrap();
    fs::write(lazy_dir.join("index.ts"), "export const lazy = 7;").unwrap();
    fs::write(&source_path, "import(\"./lazy\");").unwrap();

    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
            source: Expression::Literal(LiteralValue::String("./lazy".to_string())),
        }))),
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
fn test_resolution_accepts_directory_index_dynamic_import_targets_in_js_files() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).unwrap();
    fs::write(lazy_dir.join("index.js"), "export const lazy = 7;").unwrap();
    fs::write(&source_path, "import(\"./lazy\");").unwrap();

    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
            source: Expression::Literal(LiteralValue::String("./lazy".to_string())),
        }))),
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
fn test_resolution_accepts_directory_index_dynamic_import_targets_in_tsx_files() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.tsx");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).unwrap();
    fs::write(lazy_dir.join("index.tsx"), "export const lazy = 7;").unwrap();
    fs::write(&source_path, "import(\"./lazy\");").unwrap();

    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
            source: Expression::Literal(LiteralValue::String("./lazy".to_string())),
        }))),
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
fn test_resolution_rejects_directory_dynamic_import_targets_without_index_in_js_files() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).unwrap();
    fs::write(&source_path, "import(\"./lazy\");").unwrap();

    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
            source: Expression::Literal(LiteralValue::String("./lazy".to_string())),
        }))),
    })];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e4::DYNAMIC_IMPORT_NOT_IN_LINKED_GRAPH as u32)
    );
}

#[test]
fn test_resolution_rejects_directory_dynamic_import_targets_without_index() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).unwrap();
    fs::write(&source_path, "import(\"./lazy\");").unwrap();

    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
            source: Expression::Literal(LiteralValue::String("./lazy".to_string())),
        }))),
    })];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e4::DYNAMIC_IMPORT_NOT_IN_LINKED_GRAPH as u32)
    );
}

#[test]
fn test_resolution_reports_unknown_dynamic_import_targets() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const name = \"lazy.ts\"; import(\"./\" + name);",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "name".to_string(),
                init: Some(Expression::Literal(LiteralValue::String(
                    "lazy.ts".to_string(),
                ))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
                source: Expression::BinaryExpression(Box::new(BinaryExpression {
                    operator: "+".to_string(),
                    left: Expression::Literal(LiteralValue::String("./".to_string())),
                    right: Expression::Identifier("name".to_string()),
                })),
            }))),
        }),
    ];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e4::DYNAMIC_IMPORT_NOT_IN_LINKED_GRAPH as u32)
    );
}

#[test]
fn test_resolution_accepts_constant_template_dynamic_import_targets() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    let chunk_path = dir.path().join("lazy.ts");
    fs::write(&chunk_path, "export const lazy = true;").unwrap();
    fs::write(
        &source_path,
        "const name = \"lazy.ts\"; import(`./${name}`);",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "name".to_string(),
                init: Some(Expression::Literal(LiteralValue::String(
                    "\"lazy.ts\"".to_string(),
                ))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
                source: Expression::Literal(LiteralValue::String("`./${name}`".to_string())),
            }))),
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
fn test_resolution_accepts_logical_wrapped_dynamic_import_targets() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    let chunk_path = dir.path().join("lazy.ts");
    fs::write(&chunk_path, "export const lazy = true;").unwrap();
    fs::write(
        &source_path,
        "const specifier = true && './lazy.ts'; import(specifier);",
    )
    .unwrap();

    for (operator, left, source) in [
        (
            LogicalOperator::And,
            Expression::Literal(LiteralValue::Boolean(true)),
            "true && './lazy.ts'",
        ),
        (
            LogicalOperator::Or,
            Expression::Literal(LiteralValue::Boolean(false)),
            "false || './lazy.ts'",
        ),
        (
            LogicalOperator::Coalesce,
            Expression::Literal(LiteralValue::Null),
            "null ?? './lazy.ts'",
        ),
    ] {
        let statements = vec![
            Statement::VariableDeclaration(VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "specifier".to_string(),
                    init: Some(Expression::LogicalExpression(Box::new(LogicalExpression {
                        operator: operator.clone(),
                        left: Box::new(left.clone()),
                        right: Box::new(Expression::Literal(LiteralValue::String(
                            "./lazy.ts".to_string(),
                        ))),
                    }))),
                }],
            }),
            Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
                    source: Expression::Identifier("specifier".to_string()),
                }))),
            }),
        ];

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {source}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_rejects_non_literal_dynamic_import_targets() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "let specifier; import(specifier);").unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "specifier".to_string(),
                init: None,
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::ImportExpression(Box::new(ImportExpression {
                source: Expression::Identifier("specifier".to_string()),
            }))),
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
    assert!(result.diagnostics.iter().any(|diag| {
        diag.message.contains("non-literal dynamic import()")
            || diag.message.contains("statically known import specifier")
    }));
}
