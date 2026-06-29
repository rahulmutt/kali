use super::*;

#[test]
fn collect_library_exports_rejects_generator_default_export_expression() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::FunctionExpression(Box::new(
            kali_ast::FunctionExpression {
                id: None,
                params: vec![],
                body: Some(Box::new(kali_ast::BlockStatement { body: vec![] })),
                is_async: false,
                generator: true,
            },
        ))),
    )];

    let error = collect_library_exports_from_statements(&statements, &source_path)
        .expect_err("generator default exports should fail");
    assert!(
        error.iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "unexpected diagnostics: {error:?}"
    );
    assert!(
        error
            .iter()
            .any(|diagnostic| diagnostic.message.contains("generator function lowering")),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn collect_library_exports_rejects_generator_default_export_expression_through_parentheses() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::ParenthesizedExpression(
            Box::new(kali_ast::ParenthesizedExpression {
                expression: Box::new(Expression::FunctionExpression(Box::new(
                    kali_ast::FunctionExpression {
                        id: None,
                        params: vec![],
                        body: Some(Box::new(kali_ast::BlockStatement { body: vec![] })),
                        is_async: false,
                        generator: true,
                    },
                ))),
            }),
        )),
    )];

    let error = collect_library_exports_from_statements(&statements, &source_path)
        .expect_err("parenthesized generator default exports should fail");
    assert!(
        error.iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "unexpected diagnostics: {error:?}"
    );
    assert!(
        error
            .iter()
            .any(|diagnostic| diagnostic.message.contains("generator function lowering")),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn collect_library_exports_rejects_generator_default_export_expression_through_sequence() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::SequenceExpression(Box::new(
            kali_ast::SequenceExpression {
                expressions: vec![
                    Expression::Literal(kali_ast::LiteralValue::Number(0.0)),
                    Expression::FunctionExpression(Box::new(kali_ast::FunctionExpression {
                        id: None,
                        params: vec![],
                        body: Some(Box::new(kali_ast::BlockStatement { body: vec![] })),
                        is_async: false,
                        generator: true,
                    })),
                ],
            },
        ))),
    )];

    let error = collect_library_exports_from_statements(&statements, &source_path)
        .expect_err("sequence-wrapped generator default exports should fail");
    assert!(
        error.iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "unexpected diagnostics: {error:?}"
    );
    assert!(
        error
            .iter()
            .any(|diagnostic| diagnostic.message.contains("generator function lowering")),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn collect_library_exports_rejects_async_generator_default_export_expression_through_sequence() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::SequenceExpression(Box::new(
            kali_ast::SequenceExpression {
                expressions: vec![
                    Expression::Literal(kali_ast::LiteralValue::Number(0.0)),
                    Expression::FunctionExpression(Box::new(kali_ast::FunctionExpression {
                        id: None,
                        params: vec![],
                        body: Some(Box::new(kali_ast::BlockStatement { body: vec![] })),
                        is_async: true,
                        generator: true,
                    })),
                ],
            },
        ))),
    )];

    let error = collect_library_exports_from_statements(&statements, &source_path)
        .expect_err("sequence-wrapped async generator default exports should fail");
    assert!(
        error.iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "unexpected diagnostics: {error:?}"
    );
    assert!(
        error.iter().any(|diagnostic| diagnostic
            .message
            .contains("async-generator function lowering")),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn collect_library_exports_rejects_generator_exported_binding() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");

    let statements = vec![
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![kali_ast::VariableDeclarator {
                id: "exported".to_string(),
                init: Some(Expression::FunctionExpression(Box::new(
                    kali_ast::FunctionExpression {
                        id: None,
                        params: vec![],
                        body: Some(Box::new(kali_ast::BlockStatement { body: vec![] })),
                        is_async: false,
                        generator: true,
                    },
                ))),
            }],
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![kali_ast::ExportSpecifier {
                local: "exported".to_string(),
                exported: "exported".to_string(),
            }],
            source: None,
        }),
    ];

    let error = collect_library_exports_from_statements(&statements, &source_path)
        .expect_err("generator exported bindings should fail");
    assert!(
        error.iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "unexpected diagnostics: {error:?}"
    );
    assert!(
        error
            .iter()
            .any(|diagnostic| diagnostic.message.contains("generator function lowering")),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn collect_library_exports_rejects_async_generator_default_export_expression() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::FunctionExpression(Box::new(
            kali_ast::FunctionExpression {
                id: None,
                params: vec![],
                body: Some(Box::new(kali_ast::BlockStatement { body: vec![] })),
                is_async: true,
                generator: true,
            },
        ))),
    )];

    let error = collect_library_exports_from_statements(&statements, &source_path)
        .expect_err("async generator default exports should fail");
    assert!(
        error.iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "unexpected diagnostics: {error:?}"
    );
    assert!(
        error.iter().any(|diagnostic| diagnostic
            .message
            .contains("async-generator function lowering")),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn collect_library_exports_rejects_generator_default_export_declaration() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::FunctionDeclaration(kali_ast::FunctionDeclaration {
            name: "main".to_string(),
            params: vec![],
            body: Box::new(kali_ast::BlockStatement { body: vec![] }),
            is_async: false,
            generator: true,
        }),
    )];

    let error = collect_library_exports_from_statements(&statements, &source_path)
        .expect_err("generator default export declarations should fail");
    assert!(
        error.iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "unexpected diagnostics: {error:?}"
    );
    assert!(
        error
            .iter()
            .any(|diagnostic| diagnostic.message.contains("generator function lowering")),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn collect_library_exports_rejects_async_generator_default_export_declaration() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::FunctionDeclaration(kali_ast::FunctionDeclaration {
            name: "main".to_string(),
            params: vec![],
            body: Box::new(kali_ast::BlockStatement { body: vec![] }),
            is_async: true,
            generator: true,
        }),
    )];

    let error = collect_library_exports_from_statements(&statements, &source_path)
        .expect_err("async generator default export declarations should fail");
    assert!(
        error.iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "unexpected diagnostics: {error:?}"
    );
    assert!(
        error.iter().any(|diagnostic| diagnostic
            .message
            .contains("async-generator function lowering")),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn collect_library_exports_rejects_async_generator_exported_binding() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");

    let statements = vec![
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![kali_ast::VariableDeclarator {
                id: "exported".to_string(),
                init: Some(Expression::FunctionExpression(Box::new(
                    kali_ast::FunctionExpression {
                        id: None,
                        params: vec![],
                        body: Some(Box::new(kali_ast::BlockStatement { body: vec![] })),
                        is_async: true,
                        generator: true,
                    },
                ))),
            }],
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![kali_ast::ExportSpecifier {
                local: "exported".to_string(),
                exported: "exported".to_string(),
            }],
            source: None,
        }),
    ];

    let error = collect_library_exports_from_statements(&statements, &source_path)
        .expect_err("async generator exported bindings should fail");
    assert!(
        error.iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "unexpected diagnostics: {error:?}"
    );
    assert!(
        error.iter().any(|diagnostic| diagnostic
            .message
            .contains("async-generator function lowering")),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn collect_library_exports_infers_literal_return_types_for_function_declarations_and_aliases() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "export function main(input) { return 1; } export { main as alias };",
    )
    .expect("write source");

    let statements = vec![
        Statement::FunctionDeclaration(kali_ast::FunctionDeclaration {
            name: "main".to_string(),
            params: vec!["input".to_string()],
            body: Box::new(kali_ast::BlockStatement {
                body: vec![Statement::ReturnStatement(kali_ast::ReturnStatement {
                    argument: Some(Expression::Literal(kali_ast::LiteralValue::Number(1.0))),
                })],
            }),
            is_async: false,
            generator: false,
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![kali_ast::ExportSpecifier {
                local: "main".to_string(),
                exported: "alias".to_string(),
            }],
            source: None,
        }),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 2, "exports: {exports:?}");
    assert!(exports
        .iter()
        .any(|export| { export.name == "main" && export.signature == "(input) => number" }));
    assert!(exports
        .iter()
        .any(|export| { export.name == "alias" && export.signature == "(input) => number" }));
}

#[test]
fn collect_library_exports_infers_template_literal_return_types_for_function_declarations_and_aliases(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "export function main(input) { return `${input}`; } export { main as alias };",
    )
    .expect("write source");

    let statements = vec![
        Statement::FunctionDeclaration(kali_ast::FunctionDeclaration {
            name: "main".to_string(),
            params: vec!["input".to_string()],
            body: Box::new(kali_ast::BlockStatement {
                body: vec![Statement::ReturnStatement(kali_ast::ReturnStatement {
                    argument: Some(Expression::TemplateLiteral(kali_ast::TemplateLiteral {
                        quasis: vec![kali_ast::TemplateElement {
                            value: String::new(),
                            tail: true,
                        }],
                        expressions: vec![Expression::Identifier("input".to_string())],
                    })),
                })],
            }),
            is_async: false,
            generator: false,
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![kali_ast::ExportSpecifier {
                local: "main".to_string(),
                exported: "alias".to_string(),
            }],
            source: None,
        }),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 2, "exports: {exports:?}");
    assert!(exports
        .iter()
        .any(|export| { export.name == "main" && export.signature == "(input) => string" }));
    assert!(exports
        .iter()
        .any(|export| { export.name == "alias" && export.signature == "(input) => string" }));
}

#[test]
fn collect_library_exports_resolves_named_re_exports_across_source_graph() {
    let dir = tempdir().expect("tempdir");
    let helper_path = dir.path().join("helper.ts");
    let bridge_path = dir.path().join("bridge.ts");

    fs::write(
        &helper_path,
        "export function quadruple(value) { return value + value; }\n",
    )
    .expect("write helper source");
    fs::write(&bridge_path, "export { quadruple } from './helper.ts';\n")
        .expect("write bridge source");

    let exports = collect_library_exports(&bridge_path, ApiSurface::Deno, &[])
        .expect("library exports should resolve through re-exports");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert!(exports
        .iter()
        .any(|export| { export.name == "quadruple" && export.signature == "(value) => unknown" }));
}

#[test]
fn collect_library_exports_resolves_default_export_aliases_across_source_graph() {
    let dir = tempdir().expect("tempdir");
    let helper_path = dir.path().join("helper.ts");
    let bridge_path = dir.path().join("bridge.ts");
    let entry_path = dir.path().join("entry.ts");

    fs::write(
        &helper_path,
        "const main = (input) => 1; export default main;\n",
    )
    .expect("write helper source");
    fs::write(
        &bridge_path,
        "export { default as bridged } from './helper.ts';\n",
    )
    .expect("write bridge source");
    fs::write(
        &entry_path,
        "export { bridged as final } from './bridge.ts';\n",
    )
    .expect("write entry source");

    let helper_exports = collect_library_exports(&helper_path, ApiSurface::Deno, &[])
        .expect("helper library exports should resolve");
    assert_eq!(
        helper_exports.len(),
        1,
        "helper exports: {helper_exports:?}"
    );
    assert!(helper_exports
        .iter()
        .any(|export| { export.name == "default" && export.signature == "(input) => number" }));

    let exports = collect_library_exports(&entry_path, ApiSurface::Deno, &[])
        .expect("library exports should resolve through default export aliases");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert!(exports
        .iter()
        .any(|export| { export.name == "final" && export.signature == "(input) => number" }));
}

#[test]
fn collect_library_exports_resolves_default_re_exports_across_source_graph() {
    let dir = tempdir().expect("tempdir");
    let helper_path = dir.path().join("helper.ts");
    let bridge_path = dir.path().join("bridge.ts");

    fs::write(
        &helper_path,
        "const main = (input) => 1; export default main;\n",
    )
    .expect("write helper source");
    fs::write(&bridge_path, "export { default } from './helper.ts';\n")
        .expect("write bridge source");

    let exports = collect_library_exports(&bridge_path, ApiSurface::Deno, &[])
        .expect("library exports should resolve through default re-exports");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert!(exports
        .iter()
        .any(|export| { export.name == "default" && export.signature == "(input) => number" }));
}

#[test]
fn collect_library_exports_resolves_named_re_exports_as_default_across_source_graph() {
    let dir = tempdir().expect("tempdir");
    let helper_path = dir.path().join("helper.ts");
    let bridge_path = dir.path().join("bridge.ts");
    let entry_path = dir.path().join("entry.ts");

    fs::write(
        &helper_path,
        "const main = (input) => 1; export default main;\n",
    )
    .expect("write helper source");
    fs::write(
        &bridge_path,
        "export { default as bridged } from './helper.ts';\n",
    )
    .expect("write bridge source");
    fs::write(
        &entry_path,
        "export { bridged as default } from './bridge.ts';\n",
    )
    .expect("write entry source");

    let exports = collect_library_exports(&entry_path, ApiSurface::Deno, &[])
        .expect("library exports should resolve default aliases across the source graph");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert!(exports
        .iter()
        .any(|export| { export.name == "default" && export.signature == "(input) => number" }));
}

#[test]
fn collect_library_exports_resolves_named_re_exports_across_multi_hop_source_graph() {
    let dir = tempdir().expect("tempdir");
    let helper_path = dir.path().join("helper.ts");
    let bridge_path = dir.path().join("bridge.ts");
    let entry_path = dir.path().join("entry.ts");

    fs::write(
        &helper_path,
        "export function quadruple(value) { return 1; }\n",
    )
    .expect("write helper source");
    fs::write(
        &bridge_path,
        "export { quadruple as bridged } from './helper.ts';\n",
    )
    .expect("write bridge source");
    fs::write(
        &entry_path,
        "export { bridged as final } from './bridge.ts';\n",
    )
    .expect("write entry source");

    let exports = collect_library_exports(&entry_path, ApiSurface::Deno, &[])
        .expect("library exports should resolve through multi-hop re-exports");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert!(exports
        .iter()
        .any(|export| { export.name == "final" && export.signature == "(value) => number" }));
}

#[test]
fn collect_library_exports_resolves_export_all_re_exports_across_source_graph() {
    let dir = tempdir().expect("tempdir");
    let helper_path = dir.path().join("helper.ts");
    let bridge_path = dir.path().join("bridge.ts");

    fs::write(
        &helper_path,
        "export function quadruple(value) { return value + value; }\nexport default function ignored() { return 1; }\n",
    )
    .expect("write helper source");
    fs::write(&bridge_path, "export * from './helper.ts';\n").expect("write bridge source");

    let exports = collect_library_exports(&bridge_path, ApiSurface::Deno, &[])
        .expect("library exports should resolve through export-all re-exports");

    assert_eq!(exports.len(), 2, "exports: {exports:?}");
    assert!(exports
        .iter()
        .any(|export| { export.name == "quadruple" && export.signature == "(value) => unknown" }));
    assert!(exports
        .iter()
        .any(|export| { export.name == "ignored" && export.signature == "() => number" }));
}

#[test]
fn collect_library_exports_infers_const_function_expression_bindings_and_aliases() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const main = (input) => 1; const helper = function(input) { return 2; }; export { main, helper as alias };",
    )
    .expect("write source");

    let statements = vec![
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![
                kali_ast::VariableDeclarator {
                    id: "main".to_string(),
                    init: Some(Expression::ParenthesizedExpression(Box::new(
                        kali_ast::ParenthesizedExpression {
                            expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                                kali_ast::ArrowFunctionExpression {
                                    params: vec![kali_ast::FunctionParam {
                                        name: "input".to_string(),
                                    }],
                                    body: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                                    is_async: false,
                                    returnType: None,
                                },
                            ))),
                        },
                    ))),
                },
                kali_ast::VariableDeclarator {
                    id: "helper".to_string(),
                    init: Some(Expression::FunctionExpression(Box::new(
                        kali_ast::FunctionExpression {
                            id: None,
                            params: vec![kali_ast::FunctionParam {
                                name: "input".to_string(),
                            }],
                            body: Some(Box::new(kali_ast::BlockStatement {
                                body: vec![Statement::ReturnStatement(kali_ast::ReturnStatement {
                                    argument: Some(Expression::Literal(
                                        kali_ast::LiteralValue::Number(2.0),
                                    )),
                                })],
                            })),
                            is_async: false,
                            generator: false,
                        },
                    ))),
                },
            ],
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![
                kali_ast::ExportSpecifier {
                    local: "main".to_string(),
                    exported: "main".to_string(),
                },
                kali_ast::ExportSpecifier {
                    local: "helper".to_string(),
                    exported: "alias".to_string(),
                },
            ],
            source: None,
        }),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 2, "exports: {exports:?}");
    assert!(exports
        .iter()
        .any(|export| { export.name == "main" && export.signature == "(input) => number" }));
    assert!(exports
        .iter()
        .any(|export| { export.name == "alias" && export.signature == "(input) => number" }));
}

#[test]
fn collect_library_exports_infers_async_function_declarations_and_aliases() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "export async function main(input) { return await 1; } export { main as alias };",
    )
    .expect("write source");

    let statements = vec![
        Statement::FunctionDeclaration(kali_ast::FunctionDeclaration {
            name: "main".to_string(),
            params: vec!["input".to_string()],
            body: Box::new(kali_ast::BlockStatement {
                body: vec![Statement::ReturnStatement(kali_ast::ReturnStatement {
                    argument: Some(Expression::AwaitExpression(Box::new(
                        kali_ast::AwaitExpression {
                            argument: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                        },
                    ))),
                })],
            }),
            is_async: true,
            generator: false,
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![kali_ast::ExportSpecifier {
                local: "main".to_string(),
                exported: "alias".to_string(),
            }],
            source: None,
        }),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 2, "exports: {exports:?}");
    assert!(exports.iter().any(|export| {
        export.name == "main" && export.signature == "(input) => Promise<number>"
    }));
    assert!(exports.iter().any(|export| {
        export.name == "alias" && export.signature == "(input) => Promise<number>"
    }));
}

#[test]
fn collect_library_exports_infers_default_async_function_expression_exports_through_await() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default async (input) => await 1;").expect("write source");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::ArrowFunctionExpression(
            Box::new(kali_ast::ArrowFunctionExpression {
                params: vec![kali_ast::FunctionParam {
                    name: "input".to_string(),
                }],
                body: Expression::AwaitExpression(Box::new(kali_ast::AwaitExpression {
                    argument: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                })),
                is_async: true,
                returnType: None,
            }),
        )),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => Promise<number>");
}

#[test]
fn collect_library_exports_infers_default_async_function_expression_exports() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default async (input) => 1;").expect("write source");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::ArrowFunctionExpression(
            Box::new(kali_ast::ArrowFunctionExpression {
                params: vec![kali_ast::FunctionParam {
                    name: "input".to_string(),
                }],
                body: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                is_async: true,
                returnType: None,
            }),
        )),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => Promise<number>");
}

#[test]
fn collect_library_exports_infers_default_async_function_expression_exports_through_chain_wrapper()
{
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default async (input) => (1);").expect("write source");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::ArrowFunctionExpression(
            Box::new(kali_ast::ArrowFunctionExpression {
                params: vec![kali_ast::FunctionParam {
                    name: "input".to_string(),
                }],
                body: Expression::ChainExpression(Box::new(kali_ast::ChainExpression {
                    expression: Box::new(Expression::Literal(kali_ast::LiteralValue::Number(1.0))),
                })),
                is_async: true,
                returnType: None,
            }),
        )),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => Promise<number>");
}

#[test]
fn collect_library_exports_infers_default_async_function_expression_exports_through_decorated_wrapper(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default ((async (input) => 1));").expect("write source");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::DecoratedExpression(
            kali_ast::DecoratedExpression {
                expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                            kali_ast::ArrowFunctionExpression {
                                params: vec![kali_ast::FunctionParam {
                                    name: "input".to_string(),
                                }],
                                body: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                                is_async: true,
                                returnType: None,
                            },
                        ))),
                    },
                ))),
            },
        )),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => Promise<number>");
}

#[test]
fn collect_library_exports_infers_default_function_expression_exports_through_await_wrapper() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default await ((input) => 1);").expect("write source");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::AwaitExpression(Box::new(
            kali_ast::AwaitExpression {
                argument: Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                            kali_ast::ArrowFunctionExpression {
                                params: vec![kali_ast::FunctionParam {
                                    name: "input".to_string(),
                                }],
                                body: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                                is_async: false,
                                returnType: None,
                            },
                        ))),
                    },
                )),
            },
        ))),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_const_function_expression_bindings_through_freeze_wrapper() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const main = Object.freeze((input) => 1); export { main };",
    )
    .expect("write source");

    let exports = collect_library_exports(&source_path, ApiSurface::Deno, &[])
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert!(exports
        .iter()
        .any(|export| { export.name == "main" && export.signature == "(input) => number" }));
}

#[test]
fn collect_library_exports_infers_default_function_expression_exports_through_freeze_wrapper() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default Object.freeze((input) => 1);").expect("write source");

    let exports = collect_library_exports(&source_path, ApiSurface::Deno, &[])
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_default_function_expression_exports_through_freeze_wrapper_across_source_graph(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let helper_path = dir.path().join(format!("helper.{extension}"));
        let bridge_path = dir.path().join(format!("bridge.{extension}"));
        let entry_path = dir.path().join(format!("entry.{extension}"));

        fs::write(&helper_path, "export default Object.freeze((input) => 1);")
            .expect("write helper source");
        fs::write(
            &bridge_path,
            format!("export {{ default as bridged }} from './helper.{extension}';"),
        )
        .expect("write bridge source");
        fs::write(
            &entry_path,
            format!("export {{ bridged as final }} from './bridge.{extension}';"),
        )
        .expect("write entry source");

        let exports = collect_library_exports(&entry_path, ApiSurface::Deno, &[])
            .expect("library exports should resolve through freeze-wrapped source graph aliases");

        assert_eq!(exports.len(), 1, "exports for {extension}: {exports:?}");
        assert_eq!(exports[0].name, "final");
        assert_eq!(exports[0].signature, "(input) => number");
    }
}

#[test]
fn collect_library_exports_infers_const_function_expression_exports_through_freeze_wrapper_across_source_graph(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let helper_path = dir.path().join(format!("helper.{extension}"));
        let bridge_path = dir.path().join(format!("bridge.{extension}"));
        let entry_path = dir.path().join(format!("entry.{extension}"));

        fs::write(
            &helper_path,
            "const main = Object.freeze((input) => 1); export default main;",
        )
        .expect("write helper source");
        fs::write(
            &bridge_path,
            format!("export {{ default as bridged }} from './helper.{extension}';"),
        )
        .expect("write bridge source");
        fs::write(
            &entry_path,
            format!("export {{ bridged as final }} from './bridge.{extension}';"),
        )
        .expect("write entry source");

        let exports = collect_library_exports(&entry_path, ApiSurface::Deno, &[])
            .expect("library exports should resolve through freeze-wrapped source graph aliases");

        assert_eq!(exports.len(), 1, "exports for {extension}: {exports:?}");
        assert_eq!(exports[0].name, "final");
        assert_eq!(exports[0].signature, "(input) => number");
    }
}

#[test]
fn collect_library_exports_infers_default_function_expression_exports_through_bracketed_freeze_wrapper(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "export default Object[\"freeze\"]((input) => 1);",
    )
    .expect("write source");

    let exports = collect_library_exports(&source_path, ApiSurface::Deno, &[])
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_default_function_expression_exports_through_bracketed_freeze_wrapper_across_source_graph(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let helper_path = dir.path().join(format!("helper.{extension}"));
        let bridge_path = dir.path().join(format!("bridge.{extension}"));
        let entry_path = dir.path().join(format!("entry.{extension}"));

        fs::write(
            &helper_path,
            "export default globalThis[\"Object\"][\"freeze\"]((input) => 1);",
        )
        .expect("write helper source");
        fs::write(
            &bridge_path,
            format!("export {{ default as bridged }} from './helper.{extension}';"),
        )
        .expect("write bridge source");
        fs::write(
            &entry_path,
            format!("export {{ bridged as final }} from './bridge.{extension}';"),
        )
        .expect("write entry source");

        let exports = collect_library_exports(&entry_path, ApiSurface::Deno, &[]).expect(
            "library exports should resolve through bracketed freeze-wrapped source graph aliases",
        );

        assert_eq!(exports.len(), 1, "exports for {extension}: {exports:?}");
        assert_eq!(exports[0].name, "final");
        assert_eq!(exports[0].signature, "(input) => number");
    }
}

#[test]
fn collect_library_exports_infers_default_function_expression_exports_through_nullish_wrapper() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default null ?? ((input) => 1);").expect("write source");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::BinaryExpression(Box::new(
            kali_ast::BinaryExpression {
                operator: "??".to_string(),
                left: Expression::Literal(kali_ast::LiteralValue::Null),
                right: Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                            kali_ast::ArrowFunctionExpression {
                                params: vec![kali_ast::FunctionParam {
                                    name: "input".to_string(),
                                }],
                                body: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                                is_async: false,
                                returnType: None,
                            },
                        ))),
                    },
                )),
            },
        ))),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_default_async_function_expression_exports_through_nullish_wrapper(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "export default void 0 ?? (async (input) => 1);",
    )
    .expect("write source");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::BinaryExpression(Box::new(
            kali_ast::BinaryExpression {
                operator: "??".to_string(),
                left: Expression::UnaryExpression(Box::new(kali_ast::UnaryExpression {
                    operator: "void".to_string(),
                    argument: Expression::Literal(kali_ast::LiteralValue::Number(0.0)),
                })),
                right: Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                            kali_ast::ArrowFunctionExpression {
                                params: vec![kali_ast::FunctionParam {
                                    name: "input".to_string(),
                                }],
                                body: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                                is_async: true,
                                returnType: None,
                            },
                        ))),
                    },
                )),
            },
        ))),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => Promise<number>");
}

#[test]
fn collect_library_exports_infers_default_function_expression_exports_through_logical_or_wrapper() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "export default false || (async (input) => 1);",
    )
    .expect("write source");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::LogicalExpression(Box::new(
            kali_ast::LogicalExpression {
                operator: kali_ast::LogicalOperator::Or,
                left: Box::new(Expression::Literal(kali_ast::LiteralValue::Boolean(false))),
                right: Box::new(Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                            kali_ast::ArrowFunctionExpression {
                                params: vec![kali_ast::FunctionParam {
                                    name: "input".to_string(),
                                }],
                                body: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                                is_async: true,
                                returnType: None,
                            },
                        ))),
                    },
                ))),
            },
        ))),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => Promise<number>");
}

#[test]
fn collect_library_exports_infers_const_function_expression_bindings_through_logical_and_wrapper() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const logicalWrapped = true && ((input) => 1); export { logicalWrapped as default };",
    )
    .expect("write source");

    let statements = vec![
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            declarations: vec![kali_ast::VariableDeclarator {
                id: "logicalWrapped".to_string(),
                init: Some(Expression::LogicalExpression(Box::new(
                    kali_ast::LogicalExpression {
                        operator: kali_ast::LogicalOperator::And,
                        left: Box::new(Expression::Literal(kali_ast::LiteralValue::Boolean(true))),
                        right: Box::new(Expression::ParenthesizedExpression(Box::new(
                            kali_ast::ParenthesizedExpression {
                                expression: Box::new(Expression::ArrowFunctionExpression(
                                    Box::new(kali_ast::ArrowFunctionExpression {
                                        params: vec![kali_ast::FunctionParam {
                                            name: "input".to_string(),
                                        }],
                                        body: Expression::Literal(kali_ast::LiteralValue::Number(
                                            1.0,
                                        )),
                                        is_async: false,
                                        returnType: None,
                                    }),
                                )),
                            },
                        ))),
                    },
                ))),
            }],
            kind: "const".to_string(),
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![kali_ast::ExportSpecifier {
                local: "logicalWrapped".to_string(),
                exported: "default".to_string(),
            }],
            source: None,
        }),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_default_async_function_expression_exports_through_await_wrapper()
{
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default await ((async (input) => 1));").expect("write source");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::AwaitExpression(Box::new(
            kali_ast::AwaitExpression {
                argument: Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                            kali_ast::ArrowFunctionExpression {
                                params: vec![kali_ast::FunctionParam {
                                    name: "input".to_string(),
                                }],
                                body: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                                is_async: true,
                                returnType: None,
                            },
                        ))),
                    },
                )),
            },
        ))),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => Promise<number>");
}

#[test]
fn collect_library_exports_infers_default_function_expression_exports_through_decorated_wrapper() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default ((input) => 1);").expect("write source");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::DecoratedExpression(
            kali_ast::DecoratedExpression {
                expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                            kali_ast::ArrowFunctionExpression {
                                params: vec![kali_ast::FunctionParam {
                                    name: "input".to_string(),
                                }],
                                body: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                                is_async: false,
                                returnType: None,
                            },
                        ))),
                    },
                ))),
            },
        )),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_function_binding_signatures_through_await_wrapper() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const main = await ((input) => 1); export { main as alias };",
    )
    .expect("write source");

    let statements = vec![
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![kali_ast::VariableDeclarator {
                id: "main".to_string(),
                init: Some(Expression::AwaitExpression(Box::new(
                    kali_ast::AwaitExpression {
                        argument: Expression::ParenthesizedExpression(Box::new(
                            kali_ast::ParenthesizedExpression {
                                expression: Box::new(Expression::ArrowFunctionExpression(
                                    Box::new(kali_ast::ArrowFunctionExpression {
                                        params: vec![kali_ast::FunctionParam {
                                            name: "input".to_string(),
                                        }],
                                        body: Expression::Literal(kali_ast::LiteralValue::Number(
                                            1.0,
                                        )),
                                        is_async: false,
                                        returnType: None,
                                    }),
                                )),
                            },
                        )),
                    },
                ))),
            }],
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![kali_ast::ExportSpecifier {
                local: "main".to_string(),
                exported: "alias".to_string(),
            }],
            source: None,
        }),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "alias");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_async_function_expression_bindings_and_aliases() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const main = true ? async (input) => await 1 : async (input) => await 1; export { main as alias };",
    )
    .expect("write source");

    let async_function_expression = |value| {
        Expression::ParenthesizedExpression(Box::new(kali_ast::ParenthesizedExpression {
            expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                kali_ast::ArrowFunctionExpression {
                    params: vec![kali_ast::FunctionParam {
                        name: "input".to_string(),
                    }],
                    body: Expression::AwaitExpression(Box::new(kali_ast::AwaitExpression {
                        argument: Expression::Literal(kali_ast::LiteralValue::Number(value)),
                    })),
                    is_async: true,
                    returnType: None,
                },
            ))),
        }))
    };

    let statements = vec![
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![kali_ast::VariableDeclarator {
                id: "main".to_string(),
                init: Some(Expression::ConditionalExpression(Box::new(
                    kali_ast::ConditionalExpression {
                        test: Box::new(Expression::Literal(kali_ast::LiteralValue::Boolean(true))),
                        consequent: Box::new(async_function_expression(1.0)),
                        alternate: Box::new(async_function_expression(1.0)),
                    },
                ))),
            }],
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![
                kali_ast::ExportSpecifier {
                    local: "main".to_string(),
                    exported: "main".to_string(),
                },
                kali_ast::ExportSpecifier {
                    local: "main".to_string(),
                    exported: "alias".to_string(),
                },
            ],
            source: None,
        }),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 2, "exports: {exports:?}");
    assert!(exports.iter().any(|export| {
        export.name == "main" && export.signature == "(input) => Promise<number>"
    }));
    assert!(exports.iter().any(|export| {
        export.name == "alias" && export.signature == "(input) => Promise<number>"
    }));
}

#[test]
fn collect_library_exports_infers_default_function_expression_exports() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default (input) => 1;").expect("write source");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::ParenthesizedExpression(
            Box::new(kali_ast::ParenthesizedExpression {
                expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                    kali_ast::ArrowFunctionExpression {
                        params: vec![kali_ast::FunctionParam {
                            name: "input".to_string(),
                        }],
                        body: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                        is_async: false,
                        returnType: None,
                    },
                ))),
            }),
        )),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_default_function_binding_exports_through_declared_alias() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const main = (input) => 1; export default main;",
    )
    .expect("write source");

    let statements = vec![
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![kali_ast::VariableDeclarator {
                id: "main".to_string(),
                init: Some(Expression::ArrowFunctionExpression(Box::new(
                    kali_ast::ArrowFunctionExpression {
                        params: vec![kali_ast::FunctionParam {
                            name: "input".to_string(),
                        }],
                        body: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                        is_async: false,
                        returnType: None,
                    },
                ))),
            }],
        }),
        Statement::ExportDefault(kali_ast::ExportDefaultDeclaration::Expression(
            Expression::Identifier("main".to_string()),
        )),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_default_function_binding_exports_through_declared_alias_chain() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const main = (input) => 1; const helper = main; export default helper;",
    )
    .expect("write source");

    let statements = vec![
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![kali_ast::VariableDeclarator {
                id: "main".to_string(),
                init: Some(Expression::ArrowFunctionExpression(Box::new(
                    kali_ast::ArrowFunctionExpression {
                        params: vec![kali_ast::FunctionParam {
                            name: "input".to_string(),
                        }],
                        body: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                        is_async: false,
                        returnType: None,
                    },
                ))),
            }],
        }),
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![kali_ast::VariableDeclarator {
                id: "helper".to_string(),
                init: Some(Expression::Identifier("main".to_string())),
            }],
        }),
        Statement::ExportDefault(kali_ast::ExportDefaultDeclaration::Expression(
            Expression::Identifier("helper".to_string()),
        )),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_default_function_binding_exports_through_declared_alias_chain_in_jsx_and_tsx_input(
) {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            "const main = (input) => 1; const helper = main; export default helper;",
        )
        .expect("write source");

        let statements = vec![
            Statement::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![kali_ast::VariableDeclarator {
                    id: "main".to_string(),
                    init: Some(Expression::ArrowFunctionExpression(Box::new(
                        kali_ast::ArrowFunctionExpression {
                            params: vec![kali_ast::FunctionParam {
                                name: "input".to_string(),
                            }],
                            body: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                            is_async: false,
                            returnType: None,
                        },
                    ))),
                }],
            }),
            Statement::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![kali_ast::VariableDeclarator {
                    id: "helper".to_string(),
                    init: Some(Expression::Identifier("main".to_string())),
                }],
            }),
            Statement::ExportDefault(kali_ast::ExportDefaultDeclaration::Expression(
                Expression::Identifier("helper".to_string()),
            )),
        ];

        let exports = collect_library_exports_from_statements(&statements, &source_path)
            .expect("library exports should collect");

        assert_eq!(exports.len(), 1, "exports: {exports:?}");
        assert_eq!(exports[0].name, "default");
        assert_eq!(exports[0].signature, "(input) => number");
    }
}

#[test]
fn collect_library_exports_infers_named_function_binding_exports_through_declared_alias_chain() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const main = (input) => 1; const helper = main; export { helper as alias };",
    )
    .expect("write source");

    let statements = vec![
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![
                kali_ast::VariableDeclarator {
                    id: "main".to_string(),
                    init: Some(Expression::ArrowFunctionExpression(Box::new(
                        kali_ast::ArrowFunctionExpression {
                            params: vec![kali_ast::FunctionParam {
                                name: "input".to_string(),
                            }],
                            body: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                            is_async: false,
                            returnType: None,
                        },
                    ))),
                },
                kali_ast::VariableDeclarator {
                    id: "helper".to_string(),
                    init: Some(Expression::Identifier("main".to_string())),
                },
            ],
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![kali_ast::ExportSpecifier {
                local: "helper".to_string(),
                exported: "alias".to_string(),
            }],
            source: None,
        }),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "alias");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_default_function_expression_exports_through_chain_wrapper() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default (input) => 1;").expect("write source");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::ArrowFunctionExpression(
            Box::new(kali_ast::ArrowFunctionExpression {
                params: vec![kali_ast::FunctionParam {
                    name: "input".to_string(),
                }],
                body: Expression::ChainExpression(Box::new(kali_ast::ChainExpression {
                    expression: Box::new(Expression::Literal(kali_ast::LiteralValue::Number(1.0))),
                })),
                is_async: false,
                returnType: None,
            }),
        )),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_default_function_expression_exports_through_optional_chain_wrapper(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default (input) => 1;").expect("write source");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::OptionalChainExpression(
            Box::new(kali_ast::OptionalChainExpression {
                inner: Box::new(kali_ast::OptionalChainInner::NonNull {
                    object: Box::new(Expression::ArrowFunctionExpression(Box::new(
                        kali_ast::ArrowFunctionExpression {
                            params: vec![kali_ast::FunctionParam {
                                name: "input".to_string(),
                            }],
                            body: Expression::OptionalChainExpression(Box::new(
                                kali_ast::OptionalChainExpression {
                                    inner: Box::new(kali_ast::OptionalChainInner::NonNull {
                                        object: Box::new(Expression::Literal(
                                            kali_ast::LiteralValue::Number(1.0),
                                        )),
                                        optional: true,
                                    }),
                                },
                            )),
                            is_async: false,
                            returnType: None,
                        },
                    ))),
                    optional: true,
                }),
            }),
        )),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_function_binding_signatures_through_satisfies_wrapper() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const main = (input) => 1; export { main as alias };",
    )
    .expect("write source");

    let statements = vec![
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![kali_ast::VariableDeclarator {
                id: "main".to_string(),
                init: Some(Expression::SatisfiesExpression(Box::new(
                    kali_ast::SatisfiesExpression {
                        type_name: "unknown".to_string(),
                        expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                            kali_ast::ArrowFunctionExpression {
                                params: vec![kali_ast::FunctionParam {
                                    name: "input".to_string(),
                                }],
                                body: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                                is_async: false,
                                returnType: None,
                            },
                        ))),
                    },
                ))),
            }],
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![kali_ast::ExportSpecifier {
                local: "main".to_string(),
                exported: "alias".to_string(),
            }],
            source: None,
        }),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "alias");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_default_function_expression_exports_through_conditional_wrapper()
{
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "export default true ? ((input) => 1) : ((input) => 1);",
    )
    .expect("write source");

    let function_expression = |value| {
        Expression::ParenthesizedExpression(Box::new(kali_ast::ParenthesizedExpression {
            expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                kali_ast::ArrowFunctionExpression {
                    params: vec![kali_ast::FunctionParam {
                        name: "input".to_string(),
                    }],
                    body: Expression::Literal(kali_ast::LiteralValue::Number(value)),
                    is_async: false,
                    returnType: None,
                },
            ))),
        }))
    };

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::ConditionalExpression(
            Box::new(kali_ast::ConditionalExpression {
                test: Box::new(Expression::Literal(kali_ast::LiteralValue::Boolean(true))),
                consequent: Box::new(function_expression(1.0)),
                alternate: Box::new(function_expression(1.0)),
            }),
        )),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_function_binding_signatures_through_optional_chain_wrapper() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const main = (input) => 1;").expect("write source");

    let statements = vec![
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![kali_ast::VariableDeclarator {
                id: "main".to_string(),
                init: Some(Expression::OptionalChainExpression(Box::new(
                    kali_ast::OptionalChainExpression {
                        inner: Box::new(kali_ast::OptionalChainInner::NonNull {
                            object: Box::new(Expression::ArrowFunctionExpression(Box::new(
                                kali_ast::ArrowFunctionExpression {
                                    params: vec![kali_ast::FunctionParam {
                                        name: "input".to_string(),
                                    }],
                                    body: Expression::OptionalChainExpression(Box::new(
                                        kali_ast::OptionalChainExpression {
                                            inner: Box::new(
                                                kali_ast::OptionalChainInner::NonNull {
                                                    object: Box::new(Expression::Literal(
                                                        kali_ast::LiteralValue::Number(1.0),
                                                    )),
                                                    optional: true,
                                                },
                                            ),
                                        },
                                    )),
                                    is_async: false,
                                    returnType: None,
                                },
                            ))),
                            optional: true,
                        }),
                    },
                ))),
            }],
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![kali_ast::ExportSpecifier {
                local: "main".to_string(),
                exported: "alias".to_string(),
            }],
            source: None,
        }),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "alias");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_function_binding_signatures_through_nullish_coalescing_wrappers()
{
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const main = null ?? ((input) => 1); const helper = void 0 ?? ((value) => 2); const undefinedMain = undefined ?? ((text) => 3); const asyncMain = null ?? (async (input) => 4); const asyncHelper = void 0 ?? (async (value) => 5); const undefinedAsync = undefined ?? (async (text) => 6);",
    )
    .expect("write source");

    let arrow_function = |param: &str, value: f64, is_async: bool| {
        Expression::ParenthesizedExpression(Box::new(kali_ast::ParenthesizedExpression {
            expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                kali_ast::ArrowFunctionExpression {
                    params: vec![kali_ast::FunctionParam {
                        name: param.to_string(),
                    }],
                    body: Expression::Literal(kali_ast::LiteralValue::Number(value)),
                    is_async,
                    returnType: None,
                },
            ))),
        }))
    };

    let nullish_expression = |left: Expression, right: Expression| {
        Expression::BinaryExpression(Box::new(kali_ast::BinaryExpression {
            operator: "??".to_string(),
            left,
            right,
        }))
    };

    let statements = vec![
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![
                kali_ast::VariableDeclarator {
                    id: "main".to_string(),
                    init: Some(nullish_expression(
                        Expression::Literal(kali_ast::LiteralValue::Null),
                        arrow_function("input", 1.0, false),
                    )),
                },
                kali_ast::VariableDeclarator {
                    id: "helper".to_string(),
                    init: Some(nullish_expression(
                        Expression::UnaryExpression(Box::new(kali_ast::UnaryExpression {
                            operator: "void".to_string(),
                            argument: Expression::Literal(kali_ast::LiteralValue::Number(0.0)),
                        })),
                        arrow_function("value", 2.0, false),
                    )),
                },
                kali_ast::VariableDeclarator {
                    id: "undefined_main".to_string(),
                    init: Some(nullish_expression(
                        Expression::Identifier("undefined".to_string()),
                        arrow_function("text", 3.0, false),
                    )),
                },
                kali_ast::VariableDeclarator {
                    id: "async_main".to_string(),
                    init: Some(nullish_expression(
                        Expression::Literal(kali_ast::LiteralValue::Null),
                        arrow_function("input", 4.0, true),
                    )),
                },
                kali_ast::VariableDeclarator {
                    id: "async_helper".to_string(),
                    init: Some(nullish_expression(
                        Expression::UnaryExpression(Box::new(kali_ast::UnaryExpression {
                            operator: "void".to_string(),
                            argument: Expression::Literal(kali_ast::LiteralValue::Number(0.0)),
                        })),
                        arrow_function("value", 5.0, true),
                    )),
                },
                kali_ast::VariableDeclarator {
                    id: "undefined_async".to_string(),
                    init: Some(nullish_expression(
                        Expression::Identifier("undefined".to_string()),
                        arrow_function("text", 6.0, true),
                    )),
                },
            ],
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![
                kali_ast::ExportSpecifier {
                    local: "main".to_string(),
                    exported: "alias".to_string(),
                },
                kali_ast::ExportSpecifier {
                    local: "helper".to_string(),
                    exported: "secondary".to_string(),
                },
                kali_ast::ExportSpecifier {
                    local: "async_main".to_string(),
                    exported: "async_alias".to_string(),
                },
                kali_ast::ExportSpecifier {
                    local: "async_helper".to_string(),
                    exported: "async_secondary".to_string(),
                },
                kali_ast::ExportSpecifier {
                    local: "undefined_main".to_string(),
                    exported: "undefined_alias".to_string(),
                },
                kali_ast::ExportSpecifier {
                    local: "undefined_async".to_string(),
                    exported: "undefined_async_alias".to_string(),
                },
            ],
            source: None,
        }),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 6, "exports: {exports:?}");
    assert!(exports
        .iter()
        .any(|export| export.name == "alias" && export.signature == "(input) => number"));
    assert!(exports
        .iter()
        .any(|export| { export.name == "secondary" && export.signature == "(value) => number" }));
    assert!(exports.iter().any(|export| {
        export.name == "async_alias" && export.signature == "(input) => Promise<number>"
    }));
    assert!(exports.iter().any(|export| {
        export.name == "async_secondary" && export.signature == "(value) => Promise<number>"
    }));
    assert!(exports.iter().any(|export| {
        export.name == "undefined_alias" && export.signature == "(text) => number"
    }));
    assert!(exports.iter().any(|export| {
        export.name == "undefined_async_alias" && export.signature == "(text) => Promise<number>"
    }));
}

#[test]
fn collect_library_exports_infers_function_binding_signatures_through_sequence_and_conditional_wrappers(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const main = 0; const helper = 1;").expect("write source");

    let statements = vec![
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![
                kali_ast::VariableDeclarator {
                    id: "main".to_string(),
                    init: Some(Expression::SequenceExpression(Box::new(
                        kali_ast::SequenceExpression {
                            expressions: vec![
                                Expression::Literal(kali_ast::LiteralValue::Number(0.0)),
                                Expression::ParenthesizedExpression(Box::new(
                                    kali_ast::ParenthesizedExpression {
                                        expression: Box::new(Expression::ArrowFunctionExpression(
                                            Box::new(kali_ast::ArrowFunctionExpression {
                                                params: vec![kali_ast::FunctionParam {
                                                    name: "input".to_string(),
                                                }],
                                                body: Expression::Literal(
                                                    kali_ast::LiteralValue::Number(1.0),
                                                ),
                                                is_async: false,
                                                returnType: None,
                                            }),
                                        )),
                                    },
                                )),
                            ],
                        },
                    ))),
                },
                kali_ast::VariableDeclarator {
                    id: "helper".to_string(),
                    init: Some(Expression::ConditionalExpression(Box::new(
                        kali_ast::ConditionalExpression {
                            test: Box::new(Expression::Literal(kali_ast::LiteralValue::Boolean(
                                true,
                            ))),
                            consequent: Box::new(Expression::ParenthesizedExpression(Box::new(
                                kali_ast::ParenthesizedExpression {
                                    expression: Box::new(Expression::FunctionExpression(Box::new(
                                        kali_ast::FunctionExpression {
                                            id: None,
                                            params: vec![kali_ast::FunctionParam {
                                                name: "input".to_string(),
                                            }],
                                            body: Some(Box::new(kali_ast::BlockStatement {
                                                body: vec![Statement::ReturnStatement(
                                                    kali_ast::ReturnStatement {
                                                        argument: Some(Expression::Literal(
                                                            kali_ast::LiteralValue::Number(2.0),
                                                        )),
                                                    },
                                                )],
                                            })),
                                            is_async: false,
                                            generator: false,
                                        },
                                    ))),
                                },
                            ))),
                            alternate: Box::new(Expression::ParenthesizedExpression(Box::new(
                                kali_ast::ParenthesizedExpression {
                                    expression: Box::new(Expression::ArrowFunctionExpression(
                                        Box::new(kali_ast::ArrowFunctionExpression {
                                            params: vec![kali_ast::FunctionParam {
                                                name: "input".to_string(),
                                            }],
                                            body: Expression::Literal(
                                                kali_ast::LiteralValue::Number(2.0),
                                            ),
                                            is_async: false,
                                            returnType: None,
                                        }),
                                    )),
                                },
                            ))),
                        },
                    ))),
                },
            ],
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![
                kali_ast::ExportSpecifier {
                    local: "main".to_string(),
                    exported: "main".to_string(),
                },
                kali_ast::ExportSpecifier {
                    local: "helper".to_string(),
                    exported: "alias".to_string(),
                },
            ],
            source: None,
        }),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 2, "exports: {exports:?}");
    assert!(exports
        .iter()
        .any(|export| { export.name == "main" && export.signature == "(input) => number" }));
    assert!(exports
        .iter()
        .any(|export| { export.name == "alias" && export.signature == "(input) => number" }));
}

#[test]
fn collect_library_exports_infers_function_binding_signatures_through_decorated_wrappers() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const main = 0; export { main as alias };").expect("write source");

    let statements = vec![
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![kali_ast::VariableDeclarator {
                id: "main".to_string(),
                init: Some(Expression::DecoratedExpression(
                    kali_ast::DecoratedExpression {
                        expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                            kali_ast::ParenthesizedExpression {
                                expression: Box::new(Expression::ArrowFunctionExpression(
                                    Box::new(kali_ast::ArrowFunctionExpression {
                                        params: vec![kali_ast::FunctionParam {
                                            name: "input".to_string(),
                                        }],
                                        body: Expression::DecoratedExpression(
                                            kali_ast::DecoratedExpression {
                                                expression: Box::new(Expression::Literal(
                                                    kali_ast::LiteralValue::Number(1.0),
                                                )),
                                            },
                                        ),
                                        is_async: false,
                                        returnType: None,
                                    }),
                                )),
                            },
                        ))),
                    },
                )),
            }],
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![kali_ast::ExportSpecifier {
                local: "main".to_string(),
                exported: "alias".to_string(),
            }],
            source: None,
        }),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert!(exports
        .iter()
        .any(|export| { export.name == "alias" && export.signature == "(input) => number" }));
}

#[test]
fn collect_library_exports_preserves_unknown_signature_for_mixed_conditional_binding_exports_in_js_jsx_ts_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            "const main = true ? ((input) => 1) : ((input, extra) => 2); export { main as alias };",
        )
        .expect("write source");

        let function_expression = |params: Vec<&str>, value: f64| {
            Expression::ParenthesizedExpression(Box::new(kali_ast::ParenthesizedExpression {
                expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                    kali_ast::ArrowFunctionExpression {
                        params: params
                            .into_iter()
                            .map(|name| kali_ast::FunctionParam {
                                name: name.to_string(),
                            })
                            .collect(),
                        body: Expression::Literal(kali_ast::LiteralValue::Number(value)),
                        is_async: false,
                        returnType: None,
                    },
                ))),
            }))
        };

        let statements = vec![
            Statement::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![kali_ast::VariableDeclarator {
                    id: "main".to_string(),
                    init: Some(Expression::ConditionalExpression(Box::new(
                        kali_ast::ConditionalExpression {
                            test: Box::new(Expression::Literal(kali_ast::LiteralValue::Boolean(
                                true,
                            ))),
                            consequent: Box::new(function_expression(vec!["input"], 1.0)),
                            alternate: Box::new(function_expression(vec!["input", "extra"], 2.0)),
                        },
                    ))),
                }],
            }),
            Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
                specifiers: vec![kali_ast::ExportSpecifier {
                    local: "main".to_string(),
                    exported: "alias".to_string(),
                }],
                source: None,
            }),
        ];

        let exports = collect_library_exports_from_statements(&statements, &source_path)
            .expect("library exports should collect");

        assert_eq!(exports.len(), 1, "exports for {extension}: {exports:?}");
        assert_eq!(exports[0].name, "alias");
        assert_eq!(exports[0].signature, "(main) => unknown");
    }
}

#[test]
fn collect_direct_bundle_calls_from_statements_peels_transparent_call_wrappers() {
    let candidate_names = ["helper".to_string(), "sequence_helper".to_string()]
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();

    let statements = vec![
        Statement::ReturnStatement(kali_ast::ReturnStatement {
            argument: Some(Expression::CallExpression(Box::new(
                kali_ast::CallExpression {
                    callee: Expression::ParenthesizedExpression(Box::new(
                        kali_ast::ParenthesizedExpression {
                            expression: Box::new(Expression::Identifier("helper".to_string())),
                        },
                    )),
                    args: vec![],
                },
            ))),
        }),
        Statement::ExpressionStatement(kali_ast::ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(
                kali_ast::CallExpression {
                    callee: Expression::SequenceExpression(Box::new(
                        kali_ast::SequenceExpression {
                            expressions: vec![
                                Expression::Identifier("ignored".to_string()),
                                Expression::Identifier("sequence_helper".to_string()),
                            ],
                        },
                    )),
                    args: vec![],
                },
            ))),
        }),
    ];

    let calls = collect_direct_bundle_calls_from_statements(&statements, &candidate_names);

    assert_eq!(
        calls,
        ["helper".to_string(), "sequence_helper".to_string()]
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>()
    );
}
