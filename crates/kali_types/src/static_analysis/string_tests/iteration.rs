use super::*;

#[test]
fn test_resolution_supports_for_of_array_iteration_with_const_string_alias_in_js_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = \"hello\"; const alias = value; for (const item of [alias]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::String(
                    "hello".to_string(),
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
                            computed_index: None,
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
fn test_resolution_supports_for_of_string_concatenation_iteration_in_js_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const prefix = \"he\"; const suffix = \"llo\"; for (const ch of prefix + suffix) { console.log(ch); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "prefix".to_string(),
                init: Some(Expression::Literal(LiteralValue::String("he".to_string()))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "suffix".to_string(),
                init: Some(Expression::Literal(LiteralValue::String("llo".to_string()))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "ch".to_string(),
                    init: None,
                }],
            }),
            right: Expression::BinaryExpression(Box::new(BinaryExpression {
                left: Expression::Identifier("prefix".to_string()),
                operator: "+".to_string(),
                right: Expression::Identifier("suffix".to_string()),
            })),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            computed_index: None,
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("ch".to_string())],
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
fn test_resolution_supports_for_await_string_concatenation_iteration_in_js_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const prefix = \"he\"; const suffix = \"llo\"; for await (const ch of prefix + suffix) { console.log(ch); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "prefix".to_string(),
                init: Some(Expression::Literal(LiteralValue::String("he".to_string()))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "suffix".to_string(),
                init: Some(Expression::Literal(LiteralValue::String("llo".to_string()))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "ch".to_string(),
                    init: None,
                }],
            }),
            right: Expression::BinaryExpression(Box::new(BinaryExpression {
                left: Expression::Identifier("prefix".to_string()),
                operator: "+".to_string(),
                right: Expression::Identifier("suffix".to_string()),
            })),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            computed_index: None,
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("ch".to_string())],
                    }))),
                })],
            })),
            is_await: true,
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
fn test_resolution_supports_for_of_template_literal_string_iteration_in_js_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "for (const ch of `hello`) { console.log(ch); }",
    )
    .unwrap();

    let statements = vec![Statement::ForOfStatement(ForOfStatement {
        left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "ch".to_string(),
                init: None,
            }],
        }),
        right: Expression::TemplateLiteral(kali_ast::TemplateLiteral {
            quasis: vec![kali_ast::TemplateElement {
                value: "hello".to_string(),
                tail: true,
            }],
            expressions: vec![],
        }),
        body: Box::new(Statement::BlockStatement(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::Identifier("console".to_string()),
                        property: "log".to_string(),
                    })),
                    args: vec![Expression::Identifier("ch".to_string())],
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
fn test_resolution_supports_for_await_of_array_iteration_with_const_string_alias_in_ts_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const value = \"hello\"; const alias = value; for await (const item of [alias]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::String(
                    "hello".to_string(),
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
                            computed_index: None,
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("item".to_string())],
                    }))),
                })],
            })),
            is_await: true,
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
fn test_resolution_supports_for_await_of_array_iteration_with_const_string_alias_in_js_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = \"hello\"; const alias = value; for await (const item of [alias]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::String(
                    "hello".to_string(),
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
                            computed_index: None,
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("item".to_string())],
                    }))),
                })],
            })),
            is_await: true,
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
