use super::*;

#[test]
fn test_resolution_allows_browser_file_reader_global() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::Identifier("FileReader".to_string())),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_allows_browser_stub_globals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("FormData".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("URLSearchParams".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("WebSocket".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("ReadableStream".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("TransformStream".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("WritableStream".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("Worker".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("BroadcastChannel".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("indexedDB".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("localStorage".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("sessionStorage".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("navigator".to_string())),
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
fn test_resolution_allows_shared_web_baseline_globals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("structuredClone".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("AbortController".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("AbortSignal".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("Event".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("EventTarget".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("CustomEvent".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("URL".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("URLSearchParams".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("TextEncoder".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("TextDecoder".to_string())),
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
fn test_resolution_allows_browser_baseline_host_globals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("fetch".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("Headers".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("Request".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("Response".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("Blob".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("File".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("performance".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("crypto".to_string())),
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
fn test_resolution_reports_threaded_runtime_globals_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("SharedArrayBuffer".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("Atomics".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("globalThis".to_string()),
                    property: "SharedArrayBuffer".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("globalThis".to_string()),
                    property: "Atomics".to_string(),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 4);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
}

#[test]
fn test_resolution_accepts_threaded_runtime_globals_when_profile_is_enabled() {
    let mut ctx = TypeContext::with_api_surface_and_runtime_profiles(
        "deno",
        vec!["wasm-threads".to_string()],
    );
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("SharedArrayBuffer".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::Identifier("Atomics".to_string())),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("globalThis".to_string()),
                    property: "SharedArrayBuffer".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("globalThis".to_string()),
                    property: "Atomics".to_string(),
                },
            ))),
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
fn test_resolution_reports_late_host_control_globals_as_unavailable() {
    let mut ctx = TypeContext::with_base_path_and_api_surface(".", "node");
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Deno".to_string()),
                    property: "pid".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Deno".to_string(),
                    })),
                    property: "pid".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Deno".to_string(),
                    })),
                    property: "cwd".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Deno".to_string()),
                    property: "chdir".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Deno".to_string(),
                    })),
                    property: "chdir".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Deno".to_string(),
                    })),
                    property: "exit".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("process".to_string()),
                    property: "pid".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "process".to_string(),
                    })),
                    property: "pid".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "process".to_string(),
                    })),
                    property: "cwd".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("process".to_string()),
                    property: "chdir".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "process".to_string(),
                    })),
                    property: "chdir".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "process".to_string(),
                    })),
                    property: "exit".to_string(),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.len() >= 4);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    for expected in ["globalThis.Deno.cwd", "globalThis.Deno.exit"] {
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
fn test_resolution_reports_late_host_control_globals_through_await_wrapped_receivers_as_unavailable(
) {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    let mut ctx = TypeContext::with_base_path_and_api_surface(&source_path, "browser");
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::MemberExpression(Box::new(
            kali_ast::MemberExpression {
                object: Expression::AwaitExpression(Box::new(AwaitExpression {
                    argument: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "process".to_string(),
                    })),
                })),
                property: "kill".to_string(),
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
        result.diagnostics[0]
            .message
            .contains("globalThis.process.kill"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_late_subprocess_and_network_globals_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Deno".to_string()),
                    property: "connect".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Deno".to_string(),
                    })),
                    property: "connect".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Deno".to_string()),
                    property: "listen".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Deno".to_string(),
                    })),
                    property: "listen".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Deno".to_string()),
                    property: "serve".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Deno".to_string(),
                    })),
                    property: "serve".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::NewExpression(Box::new(
                kali_ast::NewExpression {
                    callee: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("Deno".to_string()),
                        property: "Command".to_string(),
                    })),
                    args: vec![Expression::Literal(LiteralValue::String("sh".to_string()))],
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 7, "{:?}", result.diagnostics);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    for expected in [
        "Deno.connect",
        "globalThis.Deno.connect",
        "Deno.listen",
        "globalThis.Deno.listen",
        "Deno.serve",
        "globalThis.Deno.serve",
        "Deno.Command",
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
fn test_resolution_reports_bracketed_late_network_aliases_as_unavailable_in_browser_api_surface() {
    let mut ctx = TypeContext::with_api_surface("browser");
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Deno".to_string(),
                    })),
                    property: "connect".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Deno".to_string(),
                    })),
                    property: "listen".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Deno".to_string(),
                    })),
                    property: "serve".to_string(),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 3, "{:?}", result.diagnostics);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    for expected in [
        r#"globalThis["Deno"]["connect"]"#,
        r#"globalThis["Deno"]["listen"]"#,
        r#"globalThis["Deno"]["serve"]"#,
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
