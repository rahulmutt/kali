use super::*;

#[test]
fn test_resolution_supports_bracketed_reflect_own_keys_iteration_target_in_js_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    let source = r#"for (const key of globalThis["Reflect"]["ownKeys"]({ a: 1 })) {
    console.log(key);
}
for (const key of globalThis["Reflect"].ownKeys({ a: 1 })) {
    console.log(key);
}
for (const key of globalThis["Reflect"]['ownKeys']({ a: 1 })) {
    console.log(key);
}
for (const key of globalThis['Reflect'].ownKeys({ a: 1 })) {
    console.log(key);
}
for (const key of globalThis['Reflect']["ownKeys"]({ a: 1 })) {
    console.log(key);
}
for (const key of globalThis['Reflect']['ownKeys']({ a: 1 })) {
    console.log(key);
}
"#;
    fs::write(&source_path, source).unwrap();

    let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let result = TypeContext::with_base_path(&source_path)
        .resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_accepts_object_freeze_wrapped_object_helper_iteration_targets_in_js_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    let source = r#"const object = Object.fromEntries([["b", 1], ["a", 2]]);
async function main() {
    const conditionalKeys = Object.freeze((true ? Object.keys : Object.keys));
    const conditionalValues = Object.freeze((true ? Object.values : Object.values));
    const conditionalEntries = Object.freeze((true ? Object.entries : Object.entries));
    const keys = conditionalKeys(object);
    const values = conditionalValues(object);
    const entries = conditionalEntries(object);
    if (
        keys.length !== 2 ||
        keys[0] !== "b" ||
        keys[1] !== "a" ||
        values.length !== 2 ||
        values[0] !== 1 ||
        values[1] !== 2 ||
        entries.length !== 2 ||
        entries[0][0] !== "b" ||
        entries[0][1] !== 1 ||
        entries[1][0] !== "a" ||
        entries[1][1] !== 2
    ) {
        throw new Error("unexpected conditional Object.keys/Object.values/Object.entries helper result");
    }
}
main();
"#;
    fs::write(&source_path, source).unwrap();

    let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_accepts_await_wrapped_numeric_literals_in_static_literal_paths() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "exp".to_string(),
                })),
                args: vec![Expression::AwaitExpression(Box::new(AwaitExpression {
                    argument: Expression::Literal(LiteralValue::Number(0.0)),
                }))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "is".to_string(),
                })),
                args: vec![
                    Expression::AwaitExpression(Box::new(AwaitExpression {
                        argument: Expression::Literal(LiteralValue::Number(0.0)),
                    })),
                    Expression::AwaitExpression(Box::new(AwaitExpression {
                        argument: Expression::Literal(LiteralValue::Number(0.0)),
                    })),
                ],
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_resolution_accepts_transparent_decorated_wrappers_for_static_object_helpers() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "object".to_string(),
                init: Some(Expression::DecoratedExpression(DecoratedExpression {
                    expression: Box::new(Expression::ObjectExpression(ObjectExpression {
                        properties: vec![ObjectProperty {
                            key: PropertyName::Identifier("a".to_string()),
                            value: Expression::Literal(LiteralValue::Number(1.0)),
                            kind: ObjectPropertyKind::Init,
                        }],
                    })),
                })),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "hasOwn".to_string(),
                })),
                args: vec![
                    Expression::DecoratedExpression(DecoratedExpression {
                        expression: Box::new(Expression::Identifier("object".to_string())),
                    }),
                    Expression::DecoratedExpression(DecoratedExpression {
                        expression: Box::new(Expression::Literal(LiteralValue::String(
                            "a".to_string(),
                        ))),
                    }),
                ],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "is".to_string(),
                })),
                args: vec![
                    Expression::DecoratedExpression(DecoratedExpression {
                        expression: Box::new(Expression::Literal(LiteralValue::Boolean(true))),
                    }),
                    Expression::DecoratedExpression(DecoratedExpression {
                        expression: Box::new(Expression::Literal(LiteralValue::Boolean(true))),
                    }),
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
fn test_resolution_accepts_object_freeze_wrappers_for_static_object_helpers() {
    let mut ctx = TypeContext::new();
    let frozen_object = Expression::CallExpression(Box::new(CallExpression {
        callee: Expression::MemberExpression(Box::new(MemberExpression {
            object: Expression::Identifier("Object".to_string()),
            property: "freeze".to_string(),
        })),
        args: vec![Expression::ObjectExpression(ObjectExpression {
            properties: vec![
                ObjectProperty {
                    key: PropertyName::Identifier("b".to_string()),
                    value: Expression::Literal(LiteralValue::Number(1.0)),
                    kind: ObjectPropertyKind::Init,
                },
                ObjectProperty {
                    key: PropertyName::Identifier("a".to_string()),
                    value: Expression::Literal(LiteralValue::Number(2.0)),
                    kind: ObjectPropertyKind::Init,
                },
            ],
        })],
    }));

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "frozen".to_string(),
                init: Some(frozen_object),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "hasOwn".to_string(),
                })),
                args: vec![
                    Expression::Identifier("frozen".to_string()),
                    Expression::Literal(LiteralValue::String("a".to_string())),
                ],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "keys".to_string(),
                })),
                args: vec![Expression::Identifier("frozen".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "values".to_string(),
                })),
                args: vec![Expression::Identifier("frozen".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "entries".to_string(),
                })),
                args: vec![Expression::Identifier("frozen".to_string())],
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
fn test_resolution_reports_late_object_model_globals_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("Proxy".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("WeakRef".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::NewExpression(Box::new(
                kali_ast::NewExpression {
                    callee: Expression::Identifier("WeakMap".to_string()),
                    args: Vec::new(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::NewExpression(Box::new(
                kali_ast::NewExpression {
                    callee: Expression::Identifier("WeakSet".to_string()),
                    args: Vec::new(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::NewExpression(Box::new(
                kali_ast::NewExpression {
                    callee: Expression::Identifier("FinalizationRegistry".to_string()),
                    args: Vec::new(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("globalThis".to_string()),
                    property: "Proxy".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Proxy".to_string(),
                    })),
                    property: "revocable".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("globalThis".to_string()),
                    property: "WeakMap".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "WeakMap".to_string(),
                    })),
                    property: "value".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("globalThis".to_string()),
                    property: "WeakSet".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "WeakSet".to_string(),
                    })),
                    property: "value".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("globalThis".to_string()),
                    property: "WeakRef".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "WeakRef".to_string(),
                    })),
                    property: "value".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("globalThis".to_string()),
                    property: "FinalizationRegistry".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "FinalizationRegistry".to_string(),
                    })),
                    property: "value".to_string(),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 15);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    for expected in [
        "Proxy",
        "WeakRef",
        "WeakMap",
        "WeakSet",
        "FinalizationRegistry",
        "globalThis.Proxy",
        r#"globalThis["Proxy"]"#,
        r#"globalThis["WeakMap"]"#,
        r#"globalThis["WeakSet"]"#,
        r#"globalThis["WeakRef"]"#,
        r#"globalThis["FinalizationRegistry"]"#,
    ] {
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.message.contains(expected)),
            "missing {expected} in {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_reports_proxy_revocable_member_access_as_late_object_model_api() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Proxy".to_string()),
                    property: "revocable".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Proxy".to_string(),
                    })),
                    property: "revocable".to_string(),
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
        .any(|diag| diag.message.contains("Proxy.revocable")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("globalThis.Proxy.revocable")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| { diag.message.contains(r#"globalThis["Proxy"]["revocable"]"#) }));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains(r#"globalThis['Proxy']['revocable']"#)));
}

#[test]
fn test_resolution_reports_single_quoted_proxy_revocable_aliases_as_late_object_model_api() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"globalThis['Proxy']['revocable']; Object.freeze((globalThis['Proxy'])['revocable']);"#,
    )
    .unwrap();

    let lexer = kali_lexer::Lexer::new(
        kali_common::FileId::new(0),
        r#"globalThis['Proxy']['revocable']; Object.freeze((globalThis['Proxy'])['revocable']);"#
            .to_string(),
    );
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(result.diagnostics.len(), 2);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Proxy.revocable")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains(r#"globalThis["Proxy"]["revocable"]"#)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains(r#"globalThis['Proxy']['revocable']"#)));
}

#[test]
fn test_resolution_reports_frozen_proxy_revocable_aliases_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "freeze".to_string(),
                })),
                args: vec![Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Proxy".to_string()),
                    property: "revocable".to_string(),
                }))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "freeze".to_string(),
                })),
                args: vec![Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Proxy".to_string(),
                    })),
                    property: "revocable".to_string(),
                }))],
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
        .any(|diag| diag.message.contains("Proxy.revocable")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("globalThis.Proxy.revocable")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains(r#"globalThis['Proxy']['revocable']"#)));
}

#[test]
fn test_resolution_reports_frozen_optional_chain_proxy_revocable_aliases_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "freeze".to_string(),
                })),
                args: vec![Expression::OptionalChainExpression(Box::new(
                    OptionalChainExpression {
                        inner: Box::new(OptionalChainInner::NonNull {
                            object: Box::new(Expression::MemberExpression(Box::new(
                                MemberExpression {
                                    object: Expression::Identifier("globalThis".to_string()),
                                    property: "Proxy".to_string(),
                                },
                            ))),
                            optional: true,
                        }),
                    },
                ))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "freeze".to_string(),
                })),
                args: vec![Expression::OptionalChainExpression(Box::new(
                    OptionalChainExpression {
                        inner: Box::new(OptionalChainInner::NonNull {
                            object: Box::new(Expression::MemberExpression(Box::new(
                                MemberExpression {
                                    object: Expression::MemberExpression(Box::new(
                                        MemberExpression {
                                            object: Expression::Identifier(
                                                "globalThis".to_string(),
                                            ),
                                            property: "Proxy".to_string(),
                                        },
                                    )),
                                    property: "revocable".to_string(),
                                },
                            ))),
                            optional: true,
                        }),
                    },
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
        .any(|diag| diag.message.contains("globalThis.Proxy.revocable")));
}

#[test]
fn test_resolution_accepts_object_freeze_wrapped_dynamic_import_targets() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    let chunk_path = dir.path().join("lazy.ts");
    fs::write(&chunk_path, "export const lazy = true;").unwrap();
    fs::write(
        &source_path,
        "const specifier = Object.freeze(\"./lazy.ts\"); import(specifier);",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "specifier".to_string(),
                init: Some(Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("Object".to_string()),
                        property: "freeze".to_string(),
                    })),
                    args: vec![Expression::Literal(LiteralValue::String(
                        "./lazy.ts".to_string(),
                    ))],
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
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_process_kill_zero_probe_object_freeze_wrappers_on_node_surface() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    let source = r#"Object.freeze(process.kill)(0); Object.freeze((process.kill))(0); Object.freeze((process.kill))(+0); Object.freeze(globalThis.process.kill)(0); Object.freeze(globalThis.process.kill)(+0); Object.freeze(globalThis[\"process\"][\"kill\"])(0); Object.freeze(globalThis[\"process\"].kill)(0); Object.freeze(process)[\"kill\"](0); Object.freeze(globalThis.process)[\"kill\"](0); Object.freeze(globalThis.process)[\"kill\"](+0); Object.freeze(globalThis[\"process\"])[\"kill\"](0); Object.freeze(globalThis[\"process\"])[\"kill\"](+0); Object.freeze(globalThis[\"process\"].kill)(0); Object.freeze(globalThis[\"process\"][\"kill\"])(0); Object.freeze((globalThis.process.kill))(0); Object.freeze((globalThis.process.kill))(+0); Object.freeze((globalThis[\"process\"][\"kill\"]))(0); Object.freeze((globalThis[\"process\"][\"kill\"]))(+0); Object.freeze((globalThis[\"process\"].kill))(0); Object.freeze((globalThis[\"process\"].kill))(+0); Object.freeze((globalThis.process[\"kill\"]))(0); Object.freeze((globalThis.process[\"kill\"]))(+0); Object.freeze((process))[\"kill\"](0); Object.freeze((process))[\"kill\"](+0); Object.freeze((globalThis.process))[\"kill\"](0); Object.freeze((globalThis.process))[\"kill\"](+0); Object.freeze((globalThis["process"]))[\"kill\"](0); Object.freeze((globalThis["process"]))[\"kill\"](+0);"#;
    fs::write(&source_path, source).unwrap();

    let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let mut ctx = TypeContext::with_base_path_and_api_surface(&source_path, "node");
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_accepts_object_freeze_wrapped_set_constructor_targets_in_js_like_input() {
    let source = r#"async function main() {
    for (const value of Object.freeze(new Set([1, 2, 1]))) {
        console.log(value);
    }
}
main();
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
fn test_resolution_accepts_object_freeze_wrapped_map_constructor_targets_in_js_like_input() {
    let source = r#"async function main() {
    for await (const entry of Object.freeze(new Map([[1, 2], [1, 3], [4, 5]]))) {
        console.log(entry[0], entry[1]);
    }
}
main();
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
fn test_resolution_accepts_parenthesized_object_freeze_wrapped_set_constructor_targets_in_js_like_input(
) {
    let source = r#"async function main() {
    for (const value of Object.freeze((new Set([1, 2, 1])))) {
        console.log(value);
    }
}
main();
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
fn test_resolution_accepts_parenthesized_object_freeze_wrapped_map_constructor_targets_in_js_like_input(
) {
    let source = r#"async function main() {
    for await (const entry of Object.freeze((new Map([[1, 2], [1, 3], [4, 5]])))) {
        console.log(entry[0], entry[1]);
    }
}
main();
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
fn test_resolution_accepts_nullish_and_logical_wrapped_object_freeze_wrapped_set_and_map_constructor_results_in_js_like_input(
) {
    let source = r#"async function main() {
    for (const value of Object.freeze((null ?? new Set([1, 2, 1])))) {
        console.log(value);
    }
    for (const value of Object.freeze((true && new Set([1, 2, 1])))) {
        console.log(value);
    }
    for (const value of Object.freeze((false || new Set([1, 2, 1])))) {
        console.log(value);
    }
    for await (const entry of Object.freeze((null ?? new Map([[1, 2], [1, 3], [4, 5]])))) {
        console.log(entry[0], entry[1]);
    }
    for await (const entry of Object.freeze((true && new Map([[1, 2], [1, 3], [4, 5]])))) {
        console.log(entry[0], entry[1]);
    }
    for await (const entry of Object.freeze((false || new Map([[1, 2], [1, 3], [4, 5]])))) {
        console.log(entry[0], entry[1]);
    }
}
main();
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
fn test_resolution_accepts_nullish_and_logical_wrapped_object_freeze_wrapped_set_and_map_constructor_targets_in_js_like_input(
) {
    let source = r#"async function main() {
    for (const value of new (null ?? Set)([1, 2, 1])) {
        console.log(value);
    }
    for (const value of new (true && Set)([1, 2, 1])) {
        console.log(value);
    }
    for (const value of new (false || Set)([1, 2, 1])) {
        console.log(value);
    }
    for await (const entry of new (null ?? Map)([[1, 2], [1, 3], [4, 5]])) {
        console.log(entry[0], entry[1]);
    }
    for await (const entry of new (true && Map)([[1, 2], [1, 3], [4, 5]])) {
        console.log(entry[0], entry[1]);
    }
    for await (const entry of new (false || Map)([[1, 2], [1, 3], [4, 5]])) {
        console.log(entry[0], entry[1]);
    }
}
main();
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
fn test_resolution_supports_await_wrapped_static_helper_inputs_across_js_like_extensions() {
    let source = r#"async function main() {
    console.log(Object.is(await 1, await 1));
    console.log(Object.is(await globalThis.Object, await globalThis.Object));
    console.log(Object.is(await globalThis["Object"], await globalThis["Object"]));
    console.log(Object.is(Object.freeze(+1), Object.freeze(1)));
    console.log(Number.isSafeInteger(await 1));
    console.log(Number.isFinite(Object.freeze(1)));
    console.log(Math.atan2(await 0, await 1));
    console.log(Object.keys(await { a: 1 }));
    console.log(Object["keys"](await { a: 1 }));
    console.log(globalThis.Object["values"](await { a: 1 }));
    console.log(Reflect.ownKeys(await Object.freeze({ b: 1, a: 2 })));
    console.log(globalThis['Reflect']['ownKeys'](await Object.freeze({ c: 3, a: 1 })));
    console.log(Object.hasOwn(await Object.freeze({ d: 4 }), 'd'));
    console.log(Object.prototype.hasOwnProperty.call(await Object.freeze({ e: 5 }), 'e'));
}
main();
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
