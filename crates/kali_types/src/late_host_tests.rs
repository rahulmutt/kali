use crate::test_support::*;
use crate::*;
use kali_ast::{
    AssignmentExpression, AssignmentOperator, AwaitExpression, CallExpression, DecoratedExpression,
    Expression, ExpressionStatement, LiteralValue, MemberExpression, ObjectExpression,
    ObjectProperty, ObjectPropertyKind, ParenthesizedExpression, PropertyName, SatisfiesExpression,
    UnaryExpression, VariableDeclaration, VariableDeclarator,
};
use kali_common::process_kill_zero_probe_source;
use kali_error::_error_codes::{e3, e5};
use std::fs;
use tempfile::tempdir;

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
    let dir = tempfile::tempdir().unwrap();
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
fn test_resolution_reports_deno_args_as_unavailable_on_browser_surface() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    let mut ctx = TypeContext::with_base_path_and_api_surface(&source_path, "browser");
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::MemberExpression(Box::new(
            kali_ast::MemberExpression {
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

#[test]
fn test_resolution_allows_process_pid_query_in_node_api_surface() {
    let mut ctx = TypeContext::with_base_path_and_api_surface(".", "node");
    let statements = vec![
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
                    object: Expression::Identifier("process".to_string()),
                    property: "cwd".to_string(),
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
                    object: Expression::Identifier("Deno".to_string()),
                    property: "cwd".to_string(),
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
                    object: Expression::Identifier("Deno".to_string()),
                    property: "exit".to_string(),
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
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
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
                    object: sequence_expression(vec![
                        Expression::Literal(LiteralValue::Number(0.0)),
                        Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                            object: Expression::MemberExpression(Box::new(
                                kali_ast::MemberExpression {
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
                    object: Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::MemberExpression(Box::new(MemberExpression {
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
                        object: Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::MemberExpression(Box::new(MemberExpression {
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
                        object: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::MemberExpression(Box::new(MemberExpression {
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

    let dir = tempdir().expect("tempdir");
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

    let dir = tempdir().expect("tempdir");
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
                    object: Expression::Identifier("Intl".to_string()),
                    property: "NumberFormat".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Intl".to_string()),
                    property: "DisplayNames".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Intl".to_string()),
                    property: "Locale".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
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
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
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
                    object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
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
                    object: Expression::Identifier("Intl".to_string()),
                    property: "NumberFormat".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Intl".to_string()),
                    property: "RelativeTimeFormat".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Intl".to_string()),
                    property: "Collator".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Intl".to_string()),
                    property: "DisplayNames".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Intl".to_string()),
                    property: "Segmenter".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::Identifier("Intl".to_string()),
                    property: "Locale".to_string(),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::MemberExpression(Box::new(
                kali_ast::MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::MemberExpression(Box::new(MemberExpression {
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
    let dir = tempfile::tempdir().unwrap();
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
                    object: Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::MemberExpression(Box::new(MemberExpression {
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
                            object: Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::MemberExpression(Box::new(MemberExpression {
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
    let dir = tempfile::tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
                    object: Expression::Identifier("process".to_string()),
                    property: "kill".to_string(),
                })),
                args: vec![Expression::Identifier("zeroAlias".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
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
    let dir = tempfile::tempdir().unwrap();
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
                    object: Expression::Identifier("process".to_string()),
                    property: "kill".to_string(),
                })),
                args: vec![satisfies_zero()],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::MemberExpression(Box::new(MemberExpression {
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
                    object: Expression::MemberExpression(Box::new(MemberExpression {
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
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "process.kill(1);").unwrap();

    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
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
