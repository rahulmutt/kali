use super::*;

#[test]
fn test_resolution_accepts_transparent_wrappers_around_permission_query_descriptors() {
    let mut ctx = TypeContext::new();
    let wrapped_descriptor = Expression::DecoratedExpression(DecoratedExpression {
        expression: Box::new(Expression::ParenthesizedExpression(Box::new(
            ParenthesizedExpression {
                expression: Box::new(Expression::ObjectExpression(ObjectExpression {
                    properties: vec![ObjectProperty {
                        key: PropertyName::Identifier("name".to_string()),
                        value: Expression::Literal(LiteralValue::String("env".to_string())),
                        kind: ObjectPropertyKind::Init,
                    }],
                })),
            },
        ))),
    });

    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Deno".to_string()),
                    property: "permissions".to_string(),
                })),
                property: "query".to_string(),
            })),
            args: vec![wrapped_descriptor],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_resolution_rejects_unsupported_permission_query_descriptors() {
    let mut ctx = TypeContext::new();
    let wrapped_ffi_descriptor =
        Expression::ParenthesizedExpression(Box::new(ParenthesizedExpression {
            expression: Box::new(Expression::ObjectExpression(ObjectExpression {
                properties: vec![ObjectProperty {
                    key: PropertyName::Identifier("name".to_string()),
                    value: Expression::Literal(LiteralValue::String("ffi".to_string())),
                    kind: ObjectPropertyKind::Init,
                }],
            })),
        }));
    let wrapped_sys_descriptor = Expression::DecoratedExpression(DecoratedExpression {
        expression: Box::new(Expression::ParenthesizedExpression(Box::new(
            ParenthesizedExpression {
                expression: Box::new(Expression::ObjectExpression(ObjectExpression {
                    properties: vec![ObjectProperty {
                        key: PropertyName::String("name".to_string()),
                        value: Expression::Literal(LiteralValue::String("sys".to_string())),
                        kind: ObjectPropertyKind::Init,
                    }],
                })),
            },
        ))),
    });
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("Deno".to_string()),
                        property: "permissions".to_string(),
                    })),
                    property: "query".to_string(),
                })),
                args: vec![Expression::ObjectExpression(ObjectExpression {
                    properties: vec![ObjectProperty {
                        key: PropertyName::Identifier("name".to_string()),
                        value: Expression::Literal(LiteralValue::String("env".to_string())),
                        kind: ObjectPropertyKind::Init,
                    }],
                })],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("globalThis".to_string()),
                            property: "Deno".to_string(),
                        })),
                        property: "permissions".to_string(),
                    })),
                    property: "query".to_string(),
                })),
                args: vec![Expression::ObjectExpression(ObjectExpression {
                    properties: vec![ObjectProperty {
                        key: PropertyName::String("name".to_string()),
                        value: Expression::Literal(LiteralValue::String("ffi".to_string())),
                        kind: ObjectPropertyKind::Init,
                    }],
                })],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("globalThis".to_string()),
                            property: "Deno".to_string(),
                        })),
                        property: "permissions".to_string(),
                    })),
                    property: "query".to_string(),
                })),
                args: vec![wrapped_ffi_descriptor],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("globalThis".to_string()),
                            property: "Deno".to_string(),
                        })),
                        property: "permissions".to_string(),
                    })),
                    property: "query".to_string(),
                })),
                args: vec![wrapped_sys_descriptor],
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
        .any(|diag| diag.message.contains("permission query descriptor 'ffi'")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("permission query descriptor 'sys'")));
}

#[test]
fn test_resolution_accepts_supported_permission_query_descriptors_with_const_bindings_in_js_input()
{
    fn member(object: Expression, property: &str) -> Expression {
        Expression::MemberExpression(Box::new(MemberExpression {
            object,
            property: property.to_string(),
        }))
    }

    fn const_descriptor(name: &str, value: &str) -> Statement {
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: name.to_string(),
                init: Some(Expression::Literal(LiteralValue::String(value.to_string()))),
            }],
        })
    }

    fn permission_query(root: Expression, descriptor: &str) -> Statement {
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: member(member(root, "permissions"), "query"),
                args: vec![Expression::ObjectExpression(ObjectExpression {
                    properties: vec![ObjectProperty {
                        key: PropertyName::Identifier("name".to_string()),
                        value: Expression::Identifier(descriptor.to_string()),
                        kind: ObjectPropertyKind::Init,
                    }],
                })],
            }))),
        })
    }

    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "const descriptor = 'read';\n").expect("write source");

    let mut ctx = TypeContext::with_base_path(&source_path);
    let statements = vec![
        const_descriptor("read_descriptor", "read"),
        permission_query(
            Expression::Identifier("Deno".to_string()),
            "read_descriptor",
        ),
        const_descriptor("write_descriptor", "write"),
        permission_query(
            member(Expression::Identifier("globalThis".to_string()), "Deno"),
            "write_descriptor",
        ),
        const_descriptor("net_descriptor", "net"),
        permission_query(Expression::Identifier("Deno".to_string()), "net_descriptor"),
        const_descriptor("env_descriptor", "env"),
        permission_query(
            member(Expression::Identifier("globalThis".to_string()), "Deno"),
            "env_descriptor",
        ),
    ];

    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_resolution_accepts_supported_permission_query_descriptors_with_const_bindings_in_ts_input()
{
    fn member(object: Expression, property: &str) -> Expression {
        Expression::MemberExpression(Box::new(MemberExpression {
            object,
            property: property.to_string(),
        }))
    }

    fn const_descriptor(name: &str, value: &str) -> Statement {
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: name.to_string(),
                init: Some(Expression::Literal(LiteralValue::String(value.to_string()))),
            }],
        })
    }

    fn permission_query(root: Expression, descriptor: &str) -> Statement {
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: member(member(root, "permissions"), "query"),
                args: vec![Expression::ObjectExpression(ObjectExpression {
                    properties: vec![ObjectProperty {
                        key: PropertyName::Identifier("name".to_string()),
                        value: Expression::Identifier(descriptor.to_string()),
                        kind: ObjectPropertyKind::Init,
                    }],
                })],
            }))),
        })
    }

    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const descriptor = 'read';\n").expect("write source");

    let mut ctx = TypeContext::with_base_path(&source_path);
    let statements = vec![
        const_descriptor("read_descriptor", "read"),
        permission_query(
            Expression::Identifier("Deno".to_string()),
            "read_descriptor",
        ),
        const_descriptor("write_descriptor", "write"),
        permission_query(
            member(Expression::Identifier("globalThis".to_string()), "Deno"),
            "write_descriptor",
        ),
        const_descriptor("net_descriptor", "net"),
        permission_query(Expression::Identifier("Deno".to_string()), "net_descriptor"),
        const_descriptor("env_descriptor", "env"),
        permission_query(
            member(Expression::Identifier("globalThis".to_string()), "Deno"),
            "env_descriptor",
        ),
    ];

    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_resolution_reports_permission_escalation_members_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("Deno".to_string()),
                        property: "permissions".to_string(),
                    })),
                    property: "request".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("Deno".to_string()),
                        property: "permissions".to_string(),
                    })),
                    property: "revoke".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::MemberExpression(Box::new(
                            kali_ast::MemberExpression {
                                object: Expression::Identifier("globalThis".to_string()),
                                property: "Deno".to_string(),
                            },
                        )),
                        property: "permissions".to_string(),
                    })),
                    property: "request".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::MemberExpression(Box::new(
                            kali_ast::MemberExpression {
                                object: Expression::Identifier("globalThis".to_string()),
                                property: "Deno".to_string(),
                            },
                        )),
                        property: "permissions".to_string(),
                    })),
                    property: "revoke".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("Deno".to_string()),
                        property: "permissions".to_string(),
                    })),
                    property: "request".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("Deno".to_string()),
                        property: "permissions".to_string(),
                    })),
                    property: "revoke".to_string(),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 6);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    for expected in [
        "Deno.permissions.request",
        "Deno.permissions.revoke",
        "globalThis.Deno.permissions.request",
        "globalThis.Deno.permissions.revoke",
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
fn test_resolution_reports_bracketed_permission_escalation_members_as_unavailable() {
    let mut ctx = TypeContext::new();
    let bracketed_request = Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
        object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
            object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                object: Expression::Identifier("globalThis".to_string()),
                property: "Deno".to_string(),
            })),
            property: "permissions".to_string(),
        })),
        property: "request".to_string(),
    }));
    let bracketed_revoke = Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
        object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
            object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                object: Expression::Identifier("globalThis".to_string()),
                property: "Deno".to_string(),
            })),
            property: "permissions".to_string(),
        })),
        property: "revoke".to_string(),
    }));

    let bracketed_request_member = match &bracketed_request {
        Expression::MemberExpression(member) => member.as_ref(),
        _ => unreachable!(),
    };
    assert_eq!(
        TypeContext::member_access_name(bracketed_request_member).as_deref(),
        Some("globalThis.Deno.permissions.request")
    );
    assert_eq!(
        TypeContext::member_access_name_bracketed(bracketed_request_member).as_deref(),
        Some(r#"globalThis["Deno"]["permissions"]["request"]"#)
    );

    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(bracketed_request),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(bracketed_revoke),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 2);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    for expected in [
        "globalThis.Deno.permissions.request",
        r#"globalThis["Deno"]["permissions"]["request"]"#,
        "globalThis.Deno.permissions.revoke",
        r#"globalThis["Deno"]["permissions"]["revoke"]"#,
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
