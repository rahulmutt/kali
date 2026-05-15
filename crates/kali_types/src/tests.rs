use super::*;
use kali_ast::{
    ArrayExpression, ArrowFunctionExpression, AssignmentExpression, AssignmentOperator,
    BinaryExpression, BlockStatement, CallExpression, ClassBody, ClassDeclaration,
    DecoratedExpression, ExportDefaultDeclaration, ExportNamedDeclaration, ExportSpecifier,
    Expression, ExpressionOrSpread, ExpressionStatement, ForOfLefthand, ForOfStatement,
    FunctionDeclaration, FunctionExpression, LiteralValue, MemberExpression, MethodDefinition,
    ObjectExpression, ObjectProperty, ObjectPropertyKind, ParenthesizedExpression, PropertyName,
    SatisfiesExpression, TemplateElement, TemplateLiteral, TypeAliasDeclaration, TypeAssertion,
    UnaryExpression, UpdateExpression, UpdateOperator, VariableDeclaration, VariableDeclarator,
    YieldExpression,
};
use kali_error::_error_codes::{e3, e5};
use std::fs;
use tempfile::tempdir;

fn sequence_expression(expressions: Vec<Expression>) -> Expression {
    Expression::SequenceExpression(Box::new(kali_ast::SequenceExpression { expressions }))
}

#[test]
fn test_scope_creation() {
    let scope = Scope::new(ScopeType::Global, None);
    assert_eq!(scope.scope_type, ScopeType::Global);
    assert!(scope.parent.is_none());
}

#[test]
fn test_scope_binding() {
    let mut scope = Scope::new(ScopeType::Module, None);
    scope.bind("x", NodeId::new(1));
    scope.bind("y", NodeId::new(2));

    assert!(scope.contains("x"));
    assert!(scope.contains("y"));
    assert!(!scope.contains("z"));
}

#[test]
fn test_type_context() {
    let mut ctx = TypeContext::new();
    assert!(ctx.is_defined("Kali"));
    assert!(!ctx.is_defined("x"));

    let _module = ctx.push_scope(ScopeType::Module);
    let binding = ctx.define("x");
    assert_eq!(binding.name(), "x");
    assert!(ctx.resolve_name("x").is_some());
}

#[test]
fn test_type_annotation_resolution_accepts_known_names() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "Foo".to_string(),
            type_params: vec![],
            type_annotation: "string".to_string(),
        }),
        Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "Box".to_string(),
            type_params: vec![],
            type_annotation: "Foo | Array<string>".to_string(),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_resolution_accepts_wrapped_call_targets_with_type_assertions_and_satisfies() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "add".to_string(),
                init: Some(Expression::ArrowFunctionExpression(Box::new(
                    ArrowFunctionExpression {
                        params: vec![
                            FunctionParam {
                                name: "left".to_string(),
                            },
                            FunctionParam {
                                name: "right".to_string(),
                            },
                        ],
                        returnType: None,
                        body: Expression::BinaryExpression(Box::new(BinaryExpression {
                            operator: "+".to_string(),
                            left: Expression::Identifier("left".to_string()),
                            right: Expression::Identifier("right".to_string()),
                        })),
                        is_async: false,
                    },
                ))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::TypeAssertion(Box::new(TypeAssertion {
                    type_name: "unknown".to_string(),
                    expression: Box::new(Expression::Identifier("add".to_string())),
                })),
                args: vec![
                    Expression::Literal(LiteralValue::Number(1.0)),
                    Expression::Literal(LiteralValue::Number(2.0)),
                ],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::SatisfiesExpression(Box::new(SatisfiesExpression {
                    type_name: "unknown".to_string(),
                    expression: Box::new(Expression::Identifier("add".to_string())),
                })),
                args: vec![
                    Expression::Literal(LiteralValue::Number(3.0)),
                    Expression::Literal(LiteralValue::Number(4.0)),
                ],
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_static_object_enumeration_iteration_target_accepts_object_entries() {
    let ctx = TypeContext::new();
    let call = CallExpression {
        callee: Expression::MemberExpression(Box::new(MemberExpression {
            object: Expression::Identifier("Object".to_string()),
            property: "entries".to_string(),
        })),
        args: vec![Expression::ObjectExpression(ObjectExpression {
            properties: vec![ObjectProperty {
                key: PropertyName::String("b".to_string()),
                value: Expression::Literal(LiteralValue::Number(1.0)),
                kind: ObjectPropertyKind::Init,
            }],
        })],
    };

    assert!(ctx.is_static_object_enumeration_iteration_target(&call));
}

#[test]
fn test_resolution_supports_bracketed_reflect_own_keys_iteration_target_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    let source = r#"for (const key of globalThis["Reflect"]["ownKeys"]({ a: 1 })) {
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
fn test_resolution_accepts_new_set_iteration_target_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    let source = r#"for (const value of new Set([1, 2, 1])) {
    console.log(value);
}
"#;
    fs::write(&source_path, source).unwrap();

    let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let Statement::ForOfStatement(ForOfStatement { right, .. }) = &statements[0] else {
        panic!("expected for-of statement");
    };

    let mut ctx = TypeContext::with_base_path(&source_path);
    assert!(ctx.is_static_array_iteration_target(right));

    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_accepts_type_assertion_and_satisfies_with_known_type_names() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "Foo".to_string(),
            type_params: vec![],
            type_annotation: "string".to_string(),
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::TypeAssertion(Box::new(TypeAssertion {
                type_name: "Foo".to_string(),
                expression: Box::new(Expression::Identifier("value".to_string())),
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::SatisfiesExpression(Box::new(
                SatisfiesExpression {
                    type_name: "Foo".to_string(),
                    expression: Box::new(Expression::Identifier("value".to_string())),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_resolution_accepts_arrow_function_return_type_annotations() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "Foo".to_string(),
            type_params: vec![],
            type_annotation: "string".to_string(),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                ArrowFunctionExpression {
                    params: vec![FunctionParam {
                        name: "value".to_string(),
                    }],
                    body: Expression::Identifier("value".to_string()),
                    is_async: false,
                    returnType: Some("Foo".to_string()),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_resolution_accepts_async_arrow_function_return_type_annotations() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "Foo".to_string(),
            type_params: vec![],
            type_annotation: "string".to_string(),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                ArrowFunctionExpression {
                    params: vec![FunctionParam {
                        name: "value".to_string(),
                    }],
                    body: Expression::Identifier("value".to_string()),
                    is_async: true,
                    returnType: Some("Foo".to_string()),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_resolution_reports_unknown_type_names_in_type_assertion_and_satisfies_expressions() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::TypeAssertion(Box::new(TypeAssertion {
                type_name: "Missing".to_string(),
                expression: Box::new(Expression::Identifier("value".to_string())),
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::SatisfiesExpression(Box::new(
                SatisfiesExpression {
                    type_name: "Missing".to_string(),
                    expression: Box::new(Expression::Identifier("value".to_string())),
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.code == Some(e3::UNDEFINED_IDENTIFIER as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Missing")));
}

#[test]
fn test_type_annotation_resolution_reports_unknown_names() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::TypeAliasDeclaration(TypeAliasDeclaration {
        name: "Box".to_string(),
        type_params: vec![],
        type_annotation: "Missing | string".to_string(),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.code == Some(e3::UNDEFINED_IDENTIFIER as u32)));
}

#[test]
fn test_type_annotation_resolution_reports_unknown_names_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "const value = 1;").unwrap();

    let mut ctx = TypeContext::with_base_path(&source_path);
    let statements = vec![Statement::TypeAliasDeclaration(TypeAliasDeclaration {
        name: "Box".to_string(),
        type_params: vec![],
        type_annotation: "Missing | string".to_string(),
    })];

    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.code == Some(e3::UNDEFINED_IDENTIFIER as u32)));
}

#[test]
fn test_type_annotation_resolution_reports_unknown_names_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        let dir = tempfile::tempdir().unwrap();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, "const value = 1;").unwrap();

        let mut ctx = TypeContext::with_base_path(&source_path);
        let statements = vec![Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "Box".to_string(),
            type_params: vec![],
            type_annotation: "Missing | string".to_string(),
        })];

        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.code == Some(e3::UNDEFINED_IDENTIFIER as u32)),
            "{extension}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_type_annotation_resolution_accepts_known_names_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "const value = 1;").unwrap();

    let mut ctx = TypeContext::with_base_path(&source_path);
    let statements = vec![
        Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "Foo".to_string(),
            type_params: vec![],
            type_annotation: "string".to_string(),
        }),
        Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "Box".to_string(),
            type_params: vec![],
            type_annotation: "Foo | Array<string>".to_string(),
        }),
    ];

    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_type_annotation_resolution_accepts_nested_known_names_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "const value = 1;").unwrap();

    let mut ctx = TypeContext::with_base_path(&source_path);
    let statements = vec![
        Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "Foo".to_string(),
            type_params: vec![],
            type_annotation: "string".to_string(),
        }),
        Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "Box".to_string(),
            type_params: vec![],
            type_annotation: "Promise<Array<Foo>>".to_string(),
        }),
    ];

    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_type_annotation_resolution_accepts_known_names_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().unwrap();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, "const value = 1;").unwrap();

        let mut ctx = TypeContext::with_base_path(&source_path);
        let statements = vec![
            Statement::TypeAliasDeclaration(TypeAliasDeclaration {
                name: "Foo".to_string(),
                type_params: vec![],
                type_annotation: "string".to_string(),
            }),
            Statement::TypeAliasDeclaration(TypeAliasDeclaration {
                name: "Box".to_string(),
                type_params: vec![],
                type_annotation: "Foo | Array<string>".to_string(),
            }),
        ];

        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "{extension}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_type_annotation_resolution_accepts_deeper_nested_known_names_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "const value = 1;").unwrap();

    let mut ctx = TypeContext::with_base_path(&source_path);
    let statements = vec![
        Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "Foo".to_string(),
            type_params: vec![],
            type_annotation: "string".to_string(),
        }),
        Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "Box".to_string(),
            type_params: vec![],
            type_annotation: "Promise<Array<Promise<Foo>>>".to_string(),
        }),
    ];

    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_type_annotation_resolution_accepts_deeper_nested_known_names_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().unwrap();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, "const value = 1;").unwrap();

        let mut ctx = TypeContext::with_base_path(&source_path);
        let statements = vec![
            Statement::TypeAliasDeclaration(TypeAliasDeclaration {
                name: "Foo".to_string(),
                type_params: vec![],
                type_annotation: "string".to_string(),
            }),
            Statement::TypeAliasDeclaration(TypeAliasDeclaration {
                name: "Box".to_string(),
                type_params: vec![],
                type_annotation: "Promise<Array<Promise<Foo>>>".to_string(),
            }),
        ];

        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "{extension}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_type_annotation_resolution_accepts_mixed_union_nested_known_names_in_js_jsx_and_tsx_input()
{
    for extension in ["js", "jsx", "tsx"] {
        let dir = tempdir().unwrap();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, "const value = 1;").unwrap();

        let mut ctx = TypeContext::with_base_path(&source_path);
        let statements = vec![
            Statement::TypeAliasDeclaration(TypeAliasDeclaration {
                name: "Foo".to_string(),
                type_params: vec![],
                type_annotation: "string".to_string(),
            }),
            Statement::TypeAliasDeclaration(TypeAliasDeclaration {
                name: "Bar".to_string(),
                type_params: vec![],
                type_annotation: "number".to_string(),
            }),
            Statement::TypeAliasDeclaration(TypeAliasDeclaration {
                name: "Box".to_string(),
                type_params: vec![],
                type_annotation: "Promise<Array<Foo | Bar>>".to_string(),
            }),
        ];

        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert!(
            result.diagnostics.is_empty(),
            "{extension}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_type_checker_collects_annotation_diagnostics() {
    let mut checker = TypeChecker::new();
    checker.check_type_annotation(NodeId::new(1), "Missing | string");

    let diagnostics = checker.typecheck(NodeId::new(0));
    assert!(diagnostics
        .iter()
        .any(|diag| diag.code == Some(e3::UNDEFINED_IDENTIFIER as u32)));
}

#[test]
fn test_type_checker_typecheck_drains_pending_context_diagnostics() {
    let mut checker = TypeChecker::new();
    checker
        .context
        .resolve_type_annotation_text("Missing | string");

    let diagnostics = checker.typecheck(NodeId::new(0));
    assert!(diagnostics
        .iter()
        .any(|diag| diag.code == Some(e3::UNDEFINED_IDENTIFIER as u32)));
    assert!(checker.context.diagnostics().is_empty());
}

#[test]
fn test_resolution_finds_bound_names() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::VariableDeclaration(VariableDeclaration {
        kind: "let".to_string(),
        declarations: vec![VariableDeclarator {
            id: "value".to_string(),
            init: Some(Expression::Literal(LiteralValue::Number(1.0))),
        }],
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert!(result.scopes.values().any(|scope| scope.contains("value")));
}

#[test]
fn test_resolution_reports_unresolved_identifiers() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::Identifier("missing".to_string())),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::UNDEFINED_IDENTIFIER as u32)
    );
}

#[test]
fn test_resolution_reports_duplicate_bindings() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::BlockStatement(BlockStatement {
        body: vec![
            Statement::VariableDeclaration(VariableDeclaration {
                kind: "let".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "x".to_string(),
                    init: None,
                }],
            }),
            Statement::VariableDeclaration(VariableDeclaration {
                kind: "let".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "x".to_string(),
                    init: None,
                }],
            }),
        ],
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.code == Some(e3::DUPLICATE_BINDING as u32)));
}

#[test]
fn test_resolution_reports_unresolved_public_exports() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExportNamed(ExportNamedDeclaration {
        specifiers: vec![ExportSpecifier {
            local: "missing".to_string(),
            exported: "missing".to_string(),
        }],
        source: None,
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::UNDEFINED_IDENTIFIER as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("missing"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_unresolved_public_exports_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "export { missing };").unwrap();

    let statements = vec![Statement::ExportNamed(ExportNamedDeclaration {
        specifiers: vec![ExportSpecifier {
            local: "missing".to_string(),
            exported: "missing".to_string(),
        }],
        source: None,
    })];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::UNDEFINED_IDENTIFIER as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("missing"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_resolves_export_all_sources_in_js_input() {
    let dir = tempdir().unwrap();
    let helper_path = dir.path().join("helper.js");
    let source_path = dir.path().join("main.js");
    fs::write(
        &helper_path,
        "export function quadruple(value) { return value + value; }",
    )
    .unwrap();
    fs::write(&source_path, "export * from './helper.js';").unwrap();

    let statements = vec![Statement::ExportAll(kali_ast::ExportAllDeclaration {
        source: "./helper.js".to_string(),
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
fn test_resolution_reports_unresolved_public_exports_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().unwrap();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, "export { missing };").unwrap();

        let statements = vec![Statement::ExportNamed(ExportNamedDeclaration {
            specifiers: vec![ExportSpecifier {
                local: "missing".to_string(),
                exported: "missing".to_string(),
            }],
            source: None,
        })];

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].code,
            Some(e3::UNDEFINED_IDENTIFIER as u32)
        );
        assert!(
            result.diagnostics[0].message.contains("missing"),
            "unexpected diagnostics for {extension}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_reports_unresolved_public_export_aliases_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "export { missing as renamed };").unwrap();

    let statements = vec![Statement::ExportNamed(ExportNamedDeclaration {
        specifiers: vec![ExportSpecifier {
            local: "missing".to_string(),
            exported: "renamed".to_string(),
        }],
        source: None,
    })];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::UNDEFINED_IDENTIFIER as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("missing"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_unresolved_default_exports_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "export default missing;").unwrap();

    let statements = vec![Statement::ExportDefault(
        ExportDefaultDeclaration::Expression(Expression::Identifier("missing".to_string())),
    )];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::UNDEFINED_IDENTIFIER as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("missing"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_unresolved_default_exports_in_ts_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default missing;").unwrap();

    let statements = vec![Statement::ExportDefault(
        ExportDefaultDeclaration::Expression(Expression::Identifier("missing".to_string())),
    )];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::UNDEFINED_IDENTIFIER as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("missing"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_unresolved_default_export_aliases_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "export { missing as default };").unwrap();

    let statements = vec![Statement::ExportNamed(ExportNamedDeclaration {
        specifiers: vec![ExportSpecifier {
            local: "missing".to_string(),
            exported: "default".to_string(),
        }],
        source: None,
    })];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::UNDEFINED_IDENTIFIER as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("missing"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_unresolved_default_export_aliases_in_ts_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export { missing as default };").unwrap();

    let statements = vec![Statement::ExportNamed(ExportNamedDeclaration {
        specifiers: vec![ExportSpecifier {
            local: "missing".to_string(),
            exported: "default".to_string(),
        }],
        source: None,
    })];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::UNDEFINED_IDENTIFIER as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("missing"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_unresolved_default_export_aliases_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().unwrap();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, "export { missing as default };").unwrap();

        let statements = vec![Statement::ExportNamed(ExportNamedDeclaration {
            specifiers: vec![ExportSpecifier {
                local: "missing".to_string(),
                exported: "default".to_string(),
            }],
            source: None,
        })];

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].code,
            Some(e3::UNDEFINED_IDENTIFIER as u32)
        );
        assert!(
            result.diagnostics[0].message.contains("missing"),
            "unexpected diagnostics for {extension}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_reports_unresolved_identifiers_inside_default_export_function_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "export default function describe() { missing; }",
    )
    .unwrap();

    let statements = vec![Statement::ExportDefault(
        ExportDefaultDeclaration::FunctionDeclaration(FunctionDeclaration {
            name: "describe".to_string(),
            params: vec![],
            body: Box::new(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::Identifier("missing".to_string())),
                })],
            }),
            is_async: false,
            generator: false,
        }),
    )];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::UNDEFINED_IDENTIFIER as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("missing"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_unresolved_identifiers_inside_default_export_function_in_ts_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "export default function describe() { missing; }",
    )
    .unwrap();

    let statements = vec![Statement::ExportDefault(
        ExportDefaultDeclaration::FunctionDeclaration(FunctionDeclaration {
            name: "describe".to_string(),
            params: vec![],
            body: Box::new(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::Identifier("missing".to_string())),
                })],
            }),
            is_async: false,
            generator: false,
        }),
    )];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::UNDEFINED_IDENTIFIER as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("missing"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_unresolved_identifiers_inside_default_export_function_in_jsx_and_tsx_input(
) {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().unwrap();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            "export default function describe() { missing; }",
        )
        .unwrap();

        let statements = vec![Statement::ExportDefault(
            ExportDefaultDeclaration::FunctionDeclaration(FunctionDeclaration {
                name: "describe".to_string(),
                params: vec![],
                body: Box::new(BlockStatement {
                    body: vec![Statement::ExpressionStatement(ExpressionStatement {
                        expression: Box::new(Expression::Identifier("missing".to_string())),
                    })],
                }),
                is_async: false,
                generator: false,
            }),
        )];

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].code,
            Some(e3::UNDEFINED_IDENTIFIER as u32)
        );
        assert!(
            result.diagnostics[0].message.contains("missing"),
            "unexpected diagnostics for {extension}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_reports_missing_re_export_sources() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export { missing } from './missing.ts';").unwrap();

    let statements = vec![Statement::ExportNamed(ExportNamedDeclaration {
        specifiers: vec![ExportSpecifier {
            local: "missing".to_string(),
            exported: "missing".to_string(),
        }],
        source: Some("./missing.ts".to_string()),
    })];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::IMPORT_NOT_FOUND as u32)
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains("could not be resolved"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_missing_re_export_sources_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "export { missing } from './missing.js';").unwrap();

    let statements = vec![Statement::ExportNamed(ExportNamedDeclaration {
        specifiers: vec![ExportSpecifier {
            local: "missing".to_string(),
            exported: "missing".to_string(),
        }],
        source: Some("./missing.js".to_string()),
    })];

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::IMPORT_NOT_FOUND as u32)
    );
    assert!(
        result.diagnostics[0]
            .message
            .contains("could not be resolved"),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_missing_re_export_sources_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().unwrap();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            format!("export {{ missing }} from './missing.{extension}';"),
        )
        .unwrap();

        let statements = vec![Statement::ExportNamed(ExportNamedDeclaration {
            specifiers: vec![ExportSpecifier {
                local: "missing".to_string(),
                exported: "missing".to_string(),
            }],
            source: Some(format!("./missing.{extension}")),
        })];

        let mut ctx = TypeContext::with_base_path(&source_path);
        let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].code,
            Some(e3::IMPORT_NOT_FOUND as u32)
        );
        assert!(
            result.diagnostics[0]
                .message
                .contains("could not be resolved"),
            "unexpected diagnostics for {extension}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_reports_math_floor_as_available_for_integer_inputs() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "floor".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(1.0))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_math_sqrt_as_available_for_perfect_square_integer_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "sqrt".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(4.0))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_allows_nullish_coalescing() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::BinaryExpression(Box::new(BinaryExpression {
            operator: "??".to_string(),
            left: Expression::Literal(LiteralValue::Null),
            right: Expression::Literal(LiteralValue::Number(1.0)),
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_allows_nullish_coalescing_with_void_and_undefined_fallbacks() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::BinaryExpression(Box::new(BinaryExpression {
                operator: "??".to_string(),
                left: Expression::UnaryExpression(Box::new(UnaryExpression {
                    operator: "void".to_string(),
                    argument: Expression::Literal(LiteralValue::Number(0.0)),
                })),
                right: Expression::Literal(LiteralValue::Number(1.0)),
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::BinaryExpression(Box::new(BinaryExpression {
                operator: "??".to_string(),
                left: Expression::Identifier("undefined".to_string()),
                right: Expression::Literal(LiteralValue::Number(2.0)),
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
fn test_resolution_reports_missing_imports() {
    let mut ctx = TypeContext::with_base_path(".");
    let statements = vec![Statement::ImportDeclaration(ImportDeclaration {
        specifiers: vec![ImportSpecifier::Default("value".to_string())],
        source: "./definitely-missing-file.ts".to_string(),
    })];

    let result = ctx.resolve_statements_at_path(Some("."), &statements);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e3::IMPORT_NOT_FOUND as u32)
    );
}

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
fn test_resolution_reports_generator_lowering_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::FunctionDeclaration(FunctionDeclaration {
            name: "main".to_string(),
            params: vec![],
            body: Box::new(BlockStatement { body: vec![] }),
            is_async: false,
            generator: true,
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::YieldExpression(Box::new(YieldExpression {
                delegate: false,
                argument: None,
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::YieldExpression(Box::new(YieldExpression {
                delegate: true,
                argument: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![],
                })),
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::FunctionExpression(Box::new(
                FunctionExpression {
                    id: Some("inner".to_string()),
                    params: vec![],
                    body: Some(Box::new(BlockStatement { body: vec![] })),
                    is_async: false,
                    generator: true,
                },
            ))),
        }),
        Statement::FunctionDeclaration(FunctionDeclaration {
            name: "asyncMain".to_string(),
            params: vec![],
            body: Box::new(BlockStatement { body: vec![] }),
            is_async: true,
            generator: true,
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::FunctionExpression(Box::new(
                FunctionExpression {
                    id: Some("asyncInner".to_string()),
                    params: vec![],
                    body: Some(Box::new(BlockStatement { body: vec![] })),
                    is_async: true,
                    generator: true,
                },
            ))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.len() >= 6);
    assert!(
        result
            .diagnostics
            .iter()
            .filter(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32))
            .count()
            >= 6
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
fn test_member_access_bracketed_name_for_env_snapshot_materialization() {
    let expr = kali_ast::MemberExpression {
        object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
            object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
                object: Expression::Identifier("globalThis".to_string()),
                property: "Deno".to_string(),
            })),
            property: "env".to_string(),
        })),
        property: "toObject".to_string(),
    };

    assert_eq!(
        TypeContext::member_access_name(&expr).as_deref(),
        Some("globalThis.Deno.env.toObject")
    );
    assert_eq!(
        TypeContext::member_access_name_bracketed(&expr).as_deref(),
        Some(r#"globalThis["Deno"]["env"]["toObject"]"#)
    );

    let mixed_expr = kali_ast::MemberExpression {
        object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
            object: Expression::Identifier("Deno".to_string()),
            property: "env".to_string(),
        })),
        property: "toObject".to_string(),
    };

    assert_eq!(
        TypeContext::member_access_name(&mixed_expr).as_deref(),
        Some("Deno.env.toObject")
    );
    assert_eq!(
        TypeContext::member_access_name_bracketed(&mixed_expr).as_deref(),
        Some(r#"Deno["env"]["toObject"]"#)
    );

    let wrapped_object = Expression::DecoratedExpression(DecoratedExpression {
        expression: Box::new(Expression::ParenthesizedExpression(Box::new(
            ParenthesizedExpression {
                expression: Box::new(Expression::MemberExpression(Box::new(
                    kali_ast::MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Deno".to_string(),
                    },
                ))),
            },
        ))),
    });
    let sequence_wrapped_object = sequence_expression(vec![
        Expression::Literal(LiteralValue::Number(0.0)),
        wrapped_object,
    ]);

    assert_eq!(
        TypeContext::member_object_name(&sequence_wrapped_object).as_deref(),
        Some("Deno")
    );

    let wrapped_expr = kali_ast::MemberExpression {
        object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
            object: sequence_wrapped_object,
            property: "env".to_string(),
        })),
        property: "toObject".to_string(),
    };

    assert_eq!(
        TypeContext::member_access_name(&wrapped_expr).as_deref(),
        Some("globalThis.Deno.env.toObject")
    );
    assert_eq!(
        TypeContext::member_access_name_bracketed(&wrapped_expr).as_deref(),
        Some(r#"globalThis["Deno"]["env"]["toObject"]"#)
    );
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
fn test_resolution_supports_object_has_own_as_static_object_model_callable_in_browser_api_surface()
{
    let mut ctx = TypeContext::with_api_surface("browser");
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "object".to_string(),
                init: Some(Expression::ObjectExpression(ObjectExpression {
                    properties: vec![ObjectProperty {
                        key: PropertyName::Identifier("a".to_string()),
                        value: Expression::Literal(LiteralValue::Number(1.0)),
                        kind: ObjectPropertyKind::Init,
                    }],
                })),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "has_own".to_string(),
                init: Some(Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "hasOwn".to_string(),
                }))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "has_own_property_call".to_string(),
                init: Some(Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("Object".to_string()),
                            property: "prototype".to_string(),
                        })),
                        property: "hasOwnProperty".to_string(),
                    })),
                    property: "call".to_string(),
                }))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::Identifier("has_own".to_string()),
                args: vec![
                    Expression::Identifier("object".to_string()),
                    Expression::Literal(LiteralValue::String("a".to_string())),
                ],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::Identifier("has_own_property_call".to_string()),
                args: vec![
                    Expression::Identifier("object".to_string()),
                    Expression::Literal(LiteralValue::String("a".to_string())),
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
fn test_resolution_supports_object_has_own_helpers_for_static_object_literals_and_alias_chains_in_js_input(
) {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const object = { a: 1, "b": 2 };
const alias = object;
Object.hasOwn(alias, "a");
Object.prototype.hasOwnProperty.call(alias, "a");
"#,
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "object".to_string(),
                init: Some(Expression::ObjectExpression(ObjectExpression {
                    properties: vec![
                        ObjectProperty {
                            key: PropertyName::Identifier("a".to_string()),
                            value: Expression::Literal(LiteralValue::Number(1.0)),
                            kind: ObjectPropertyKind::Init,
                        },
                        ObjectProperty {
                            key: PropertyName::String("b".to_string()),
                            value: Expression::Literal(LiteralValue::Number(2.0)),
                            kind: ObjectPropertyKind::Init,
                        },
                    ],
                })),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Identifier("object".to_string())),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "has_own".to_string(),
                init: Some(Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "hasOwn".to_string(),
                }))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "has_own_property_call".to_string(),
                init: Some(Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("Object".to_string()),
                            property: "prototype".to_string(),
                        })),
                        property: "hasOwnProperty".to_string(),
                    })),
                    property: "call".to_string(),
                }))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::Identifier("has_own".to_string()),
                args: vec![
                    Expression::Identifier("alias".to_string()),
                    Expression::Literal(LiteralValue::String("a".to_string())),
                ],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::Identifier("has_own_property_call".to_string()),
                args: vec![
                    Expression::Identifier("alias".to_string()),
                    Expression::Literal(LiteralValue::String("a".to_string())),
                ],
            }))),
        }),
    ];

    let result = TypeContext::with_base_path(&source_path)
        .resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_object_from_entries_with_satisfies_wrapper_in_ts_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "type EntryShape = unknown; const wrappedEntries = ([['b', 1], ['a', 2]] satisfies EntryShape); const fromEntries = Object.fromEntries(wrappedEntries);",
    )
    .unwrap();

    let statements = vec![
        Statement::TypeAliasDeclaration(TypeAliasDeclaration {
            name: "EntryShape".to_string(),
            type_params: vec![],
            type_annotation: "unknown".to_string(),
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "wrappedEntries".to_string(),
                init: Some(Expression::SatisfiesExpression(Box::new(
                    SatisfiesExpression {
                        type_name: "EntryShape".to_string(),
                        expression: Box::new(Expression::ArrayExpression(
                            kali_ast::ArrayExpression {
                                elements: vec![
                                    Some(kali_ast::ExpressionOrSpread::Expression(
                                        Expression::ArrayExpression(kali_ast::ArrayExpression {
                                            elements: vec![
                                                Some(kali_ast::ExpressionOrSpread::Expression(
                                                    Expression::Literal(LiteralValue::String(
                                                        "b".to_string(),
                                                    )),
                                                )),
                                                Some(kali_ast::ExpressionOrSpread::Expression(
                                                    Expression::Literal(LiteralValue::Number(1.0)),
                                                )),
                                            ],
                                        }),
                                    )),
                                    Some(kali_ast::ExpressionOrSpread::Expression(
                                        Expression::ArrayExpression(kali_ast::ArrayExpression {
                                            elements: vec![
                                                Some(kali_ast::ExpressionOrSpread::Expression(
                                                    Expression::Literal(LiteralValue::String(
                                                        "a".to_string(),
                                                    )),
                                                )),
                                                Some(kali_ast::ExpressionOrSpread::Expression(
                                                    Expression::Literal(LiteralValue::Number(2.0)),
                                                )),
                                            ],
                                        }),
                                    )),
                                ],
                            },
                        )),
                    },
                ))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "fromEntries".to_string(),
                init: Some(Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("Object".to_string()),
                        property: "fromEntries".to_string(),
                    })),
                    args: vec![Expression::Identifier("wrappedEntries".to_string())],
                }))),
            }],
        }),
    ];

    let result = TypeContext::with_base_path(&source_path)
        .resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_object_has_own_on_object_from_entries_results_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    let source = r#"const fromEntries = Object.fromEntries([["b", 1], ["a", 2]]);
Object.hasOwn(fromEntries, "a");
Object.prototype.hasOwnProperty.call(fromEntries, "b");
Object.hasOwn(Object.fromEntries([["c", 3], ["d", 4]]), "c");
Object.prototype.hasOwnProperty.call(Object.fromEntries([["e", 5], ["f", 6]]), "e");
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
fn test_resolution_supports_bracketed_object_is_and_number_predicate_alias_spelling_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    let source = r#"const object = { a: 1 };
const objectAlias = object;
const numeric = 1;
const numericAlias = numeric;
globalThis["Object"]["is"](objectAlias, object);
globalThis.Object["is"](object, object);
globalThis["Object"].is(objectAlias, object);
Object["is"](objectAlias, object);
globalThis.Number["isFinite"](numericAlias);
globalThis["Number"].isInteger(numericAlias);
globalThis["Number"].isSafeInteger(numericAlias);
globalThis.Number["isSafeInteger"](numericAlias);
globalThis["Number"]["isSafeInteger"](numericAlias);
globalThis["Number"]["isNaN"](NaN);
Number.isSafeInteger(numericAlias);
Number["isFinite"](numericAlias);
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
fn test_resolution_supports_wrapped_call_targets_for_object_model_and_math_helpers() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::DecoratedExpression(DecoratedExpression {
                    expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                        ParenthesizedExpression {
                            expression: Box::new(Expression::MemberExpression(Box::new(
                                MemberExpression {
                                    object: Expression::Identifier("Object".to_string()),
                                    property: "hasOwn".to_string(),
                                },
                            ))),
                        },
                    ))),
                }),
                args: vec![
                    Expression::ObjectExpression(ObjectExpression {
                        properties: vec![ObjectProperty {
                            key: PropertyName::Identifier("a".to_string()),
                            value: Expression::Literal(LiteralValue::Number(1.0)),
                            kind: ObjectPropertyKind::Init,
                        }],
                    }),
                    Expression::Literal(LiteralValue::String("a".to_string())),
                ],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::DecoratedExpression(DecoratedExpression {
                    expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                        ParenthesizedExpression {
                            expression: Box::new(Expression::MemberExpression(Box::new(
                                MemberExpression {
                                    object: Expression::Identifier("Math".to_string()),
                                    property: "floor".to_string(),
                                },
                            ))),
                        },
                    ))),
                }),
                args: vec![Expression::Literal(LiteralValue::Number(1.6))],
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
fn test_resolution_accepts_wrapped_local_function_call_targets() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "constant".to_string(),
                init: Some(Expression::ArrowFunctionExpression(Box::new(
                    ArrowFunctionExpression {
                        params: vec![],
                        body: Expression::Literal(LiteralValue::Number(7.0)),
                        is_async: false,
                        returnType: None,
                    },
                ))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::TypeAssertion(Box::new(TypeAssertion {
                    type_name: "unknown".to_string(),
                    expression: Box::new(Expression::Identifier("constant".to_string())),
                })),
                args: vec![],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::SatisfiesExpression(Box::new(SatisfiesExpression {
                    type_name: "unknown".to_string(),
                    expression: Box::new(Expression::Identifier("constant".to_string())),
                })),
                args: vec![],
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
fn test_resolution_supports_object_is_numeric_literal_member_calls() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero_alias".to_string(),
                init: Some(Expression::Identifier("zero".to_string())),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "is".to_string(),
                })),
                args: vec![
                    Expression::Identifier("zero_alias".to_string()),
                    Expression::UnaryExpression(Box::new(UnaryExpression {
                        operator: "-".to_string(),
                        argument: Expression::Literal(LiteralValue::Number(0.0)),
                    })),
                ],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Object".to_string(),
                    })),
                    property: "is".to_string(),
                })),
                args: vec![
                    Expression::Literal(LiteralValue::Number(1.0)),
                    Expression::ParenthesizedExpression(Box::new(ParenthesizedExpression {
                        expression: Box::new(Expression::Identifier("zero_alias".to_string())),
                    })),
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
                    Expression::UnaryExpression(Box::new(UnaryExpression {
                        operator: "+".to_string(),
                        argument: Expression::Literal(LiteralValue::Number(1.0)),
                    })),
                    Expression::Literal(LiteralValue::Number(1.0)),
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
fn test_resolution_supports_object_is_through_object_freeze_same_reference() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "object".to_string(),
                init: Some(Expression::ObjectExpression(ObjectExpression {
                    properties: vec![ObjectProperty {
                        kind: ObjectPropertyKind::Init,
                        key: PropertyName::Identifier("a".to_string()),
                        value: Expression::Literal(LiteralValue::Number(1.0)),
                    }],
                })),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "frozen".to_string(),
                init: Some(Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("Object".to_string()),
                        property: "freeze".to_string(),
                    })),
                    args: vec![Expression::Identifier("object".to_string())],
                }))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "is".to_string(),
                })),
                args: vec![
                    Expression::Identifier("frozen".to_string()),
                    Expression::Identifier("object".to_string()),
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
                    Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("Object".to_string()),
                            property: "freeze".to_string(),
                        })),
                        args: vec![Expression::Identifier("object".to_string())],
                    })),
                    Expression::Identifier("object".to_string()),
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
fn test_resolution_accepts_object_is_alias_spellings_for_primitive_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Object".to_string(),
                    })),
                    property: "is".to_string(),
                })),
                args: vec![
                    Expression::Literal(LiteralValue::Boolean(true)),
                    Expression::Literal(LiteralValue::Boolean(true)),
                ],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Object".to_string(),
                    })),
                    property: "is".to_string(),
                })),
                args: vec![
                    Expression::Literal(LiteralValue::String("hello".to_string())),
                    Expression::Literal(LiteralValue::String("hello".to_string())),
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
fn test_resolution_rejects_object_is_with_non_primitive_literals_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Object".to_string()),
                property: "is".to_string(),
            })),
            args: vec![
                Expression::Identifier("value".to_string()),
                Expression::Literal(LiteralValue::Null),
            ],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(result.diagnostics[0].message.contains(
        "Object.is is unavailable unless both arguments are statically-known primitive literals or the same statically-known reference"
    ));
}

#[test]
fn test_resolution_accepts_number_is_finite_is_integer_and_is_nan_static_values() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Number".to_string()),
                    property: "isFinite".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Number".to_string()),
                    property: "isInteger".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Number".to_string()),
                    property: "isSafeInteger".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Number".to_string()),
                    property: "isSafeInteger".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.5))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Number".to_string(),
                    })),
                    property: "isNaN".to_string(),
                })),
                args: vec![Expression::Identifier("NaN".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Number".to_string(),
                    })),
                    property: "isInteger".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Number".to_string(),
                    })),
                    property: "isSafeInteger".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
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
fn test_resolution_accepts_number_is_alias_spellings() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Number".to_string(),
                    })),
                    property: "isFinite".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Number".to_string(),
                    })),
                    property: "isInteger".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Number".to_string(),
                    })),
                    property: "isSafeInteger".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
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
fn test_resolution_rejects_number_is_integer_with_dynamic_values_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Number".to_string()),
                property: "isInteger".to_string(),
            })),
            args: vec![Expression::Identifier("value".to_string())],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(result.diagnostics[0].message.contains(
        "Number.isInteger is unavailable unless the argument is a statically-known primitive value"
    ));
}

#[test]
fn test_resolution_rejects_number_is_safe_integer_with_dynamic_values_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Number".to_string()),
                property: "isSafeInteger".to_string(),
            })),
            args: vec![Expression::Identifier("value".to_string())],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(result.diagnostics[0].message.contains(
        "Number.isSafeInteger is unavailable unless the argument is a statically-known primitive value"
    ));
}

#[test]
fn test_resolution_accepts_object_is_with_void_undefined_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::UnaryExpression(Box::new(UnaryExpression {
                    operator: "void".to_string(),
                    argument: Expression::Literal(LiteralValue::Number(0.0)),
                }))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "is".to_string(),
                })),
                args: vec![
                    Expression::Identifier("alias".to_string()),
                    Expression::UnaryExpression(Box::new(UnaryExpression {
                        operator: "void".to_string(),
                        argument: Expression::Literal(LiteralValue::Number(1.0)),
                    })),
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
fn test_resolution_accepts_object_is_alias_spellings() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "object".to_string(),
                init: Some(Expression::ObjectExpression(ObjectExpression {
                    properties: vec![ObjectProperty {
                        kind: ObjectPropertyKind::Init,
                        key: PropertyName::Identifier("a".to_string()),
                        value: Expression::Literal(LiteralValue::Number(1.0)),
                    }],
                })),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Object".to_string(),
                    })),
                    property: "is".to_string(),
                })),
                args: vec![
                    Expression::Identifier("object".to_string()),
                    Expression::Identifier("object".to_string()),
                ],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Object".to_string(),
                    })),
                    property: "is".to_string(),
                })),
                args: vec![
                    Expression::ParenthesizedExpression(Box::new(ParenthesizedExpression {
                        expression: Box::new(Expression::Identifier("object".to_string())),
                    })),
                    Expression::ParenthesizedExpression(Box::new(ParenthesizedExpression {
                        expression: Box::new(Expression::Identifier("object".to_string())),
                    })),
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
fn test_resolution_accepts_object_is_for_distinct_object_and_array_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "is".to_string(),
                })),
                args: vec![
                    Expression::ObjectExpression(ObjectExpression {
                        properties: vec![ObjectProperty {
                            kind: ObjectPropertyKind::Init,
                            key: PropertyName::Identifier("a".to_string()),
                            value: Expression::Literal(LiteralValue::Number(1.0)),
                        }],
                    }),
                    Expression::ObjectExpression(ObjectExpression {
                        properties: vec![ObjectProperty {
                            kind: ObjectPropertyKind::Init,
                            key: PropertyName::Identifier("a".to_string()),
                            value: Expression::Literal(LiteralValue::Number(1.0)),
                        }],
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
                    Expression::ArrayExpression(ArrayExpression {
                        elements: vec![Some(ExpressionOrSpread::Expression(Expression::Literal(
                            LiteralValue::Number(1.0),
                        )))],
                    }),
                    Expression::ArrayExpression(ArrayExpression {
                        elements: vec![Some(ExpressionOrSpread::Expression(Expression::Literal(
                            LiteralValue::Number(1.0),
                        )))],
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
fn test_resolution_accepts_object_is_with_static_primitive_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "flag".to_string(),
                init: Some(Expression::Literal(LiteralValue::Boolean(true))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "text".to_string(),
                init: Some(Expression::Literal(LiteralValue::String(
                    "hello".to_string(),
                ))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "bigint".to_string(),
                init: Some(Expression::BigIntLiteral("1n".to_string())),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "infinity".to_string(),
                init: Some(Expression::Identifier("Infinity".to_string())),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "nan".to_string(),
                init: Some(Expression::Identifier("NaN".to_string())),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "is".to_string(),
                })),
                args: vec![
                    Expression::Identifier("flag".to_string()),
                    Expression::Literal(LiteralValue::Boolean(true)),
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
                    Expression::Identifier("text".to_string()),
                    Expression::Literal(LiteralValue::String("hello".to_string())),
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
                    Expression::Identifier("bigint".to_string()),
                    Expression::BigIntLiteral("1n".to_string()),
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
                    Expression::Identifier("infinity".to_string()),
                    Expression::Identifier("Infinity".to_string()),
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
                    Expression::Identifier("nan".to_string()),
                    Expression::Identifier("NaN".to_string()),
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
                    Expression::Literal(LiteralValue::Null),
                    Expression::Literal(LiteralValue::Null),
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
fn test_resolution_accepts_object_is_with_same_static_reference() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "object".to_string(),
                init: Some(Expression::ObjectExpression(ObjectExpression {
                    properties: vec![ObjectProperty {
                        key: PropertyName::Identifier("a".to_string()),
                        value: Expression::Literal(LiteralValue::Number(1.0)),
                        kind: ObjectPropertyKind::Init,
                    }],
                })),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Identifier("object".to_string())),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "is".to_string(),
                })),
                args: vec![
                    Expression::Identifier("alias".to_string()),
                    Expression::Identifier("object".to_string()),
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
                    Expression::Identifier("object".to_string()),
                    Expression::Identifier("object".to_string()),
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
fn test_resolution_accepts_object_is_with_sequence_wrapped_static_primitive_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "flag".to_string(),
                init: Some(Expression::Literal(LiteralValue::Boolean(true))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "text".to_string(),
                init: Some(Expression::Literal(LiteralValue::String(
                    "hello".to_string(),
                ))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "is".to_string(),
                })),
                args: vec![
                    sequence_expression(vec![
                        Expression::Literal(LiteralValue::Boolean(false)),
                        Expression::Identifier("flag".to_string()),
                    ]),
                    sequence_expression(vec![
                        Expression::Literal(LiteralValue::String("ignored".to_string())),
                        Expression::Identifier("text".to_string()),
                    ]),
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
fn test_member_access_bracketed_name_for_permission_escalation() {
    let expr = kali_ast::MemberExpression {
        object: Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
            object: Expression::Identifier("Deno".to_string()),
            property: "permissions".to_string(),
        })),
        property: "request".to_string(),
    };

    assert_eq!(
        TypeContext::member_access_name(&expr).as_deref(),
        Some("Deno.permissions.request")
    );
    assert_eq!(
        TypeContext::member_access_name_bracketed(&expr).as_deref(),
        Some(r#"Deno["permissions"]["request"]"#)
    );
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
fn test_resolution_supports_math_round_member_calls_for_non_integer_numeric_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "round".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.6))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "round".to_string(),
                })),
                args: vec![Expression::UnaryExpression(Box::new(
                    kali_ast::UnaryExpression {
                        operator: "-".to_string(),
                        argument: Expression::Literal(LiteralValue::Number(1.5)),
                    },
                ))],
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
fn test_resolution_supports_math_round_member_calls_through_optional_chain_wrappers() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "round".to_string(),
            })),
            args: vec![Expression::OptionalChainExpression(Box::new(
                OptionalChainExpression {
                    inner: Box::new(OptionalChainInner::NonNull {
                        object: Box::new(Expression::Literal(LiteralValue::Number(1.6))),
                        optional: true,
                    }),
                },
            ))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_math_round_member_calls_through_sequence_wrappers() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "round".to_string(),
            })),
            args: vec![sequence_expression(vec![
                Expression::Literal(LiteralValue::Number(0.0)),
                Expression::Literal(LiteralValue::Number(1.6)),
            ])],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_global_this_math_builtin_slices_for_supported_methods() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Math".to_string(),
                    })),
                    property: "min".to_string(),
                })),
                args: vec![
                    Expression::Literal(LiteralValue::Number(3.0)),
                    Expression::Literal(LiteralValue::Number(2.0)),
                    Expression::Literal(LiteralValue::Number(1.0)),
                ],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Math".to_string(),
                    })),
                    property: "abs".to_string(),
                })),
                args: vec![Expression::UnaryExpression(Box::new(
                    kali_ast::UnaryExpression {
                        operator: "-".to_string(),
                        argument: Expression::Literal(LiteralValue::Number(4.0)),
                    },
                ))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Math".to_string(),
                    })),
                    property: "sign".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(0.0))],
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
fn test_resolution_supports_math_cbrt_member_calls_for_perfect_cube_integer_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "cbrt".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(27.0))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_math_log2_member_calls_for_positive_power_of_two_integer_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "log2".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(8.0))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_math_log10_member_calls_for_positive_power_of_ten_integer_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "log10".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(1000.0))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_math_pow_member_calls_for_integer_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "pow".to_string(),
            })),
            args: vec![
                Expression::Literal(LiteralValue::Number(2.0)),
                Expression::Literal(LiteralValue::Number(3.0)),
            ],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_math_pow_member_calls_with_non_integer_base_for_zero_exponent() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "pow".to_string(),
            })),
            args: vec![
                Expression::Literal(LiteralValue::Number(1.6)),
                Expression::Literal(LiteralValue::Number(0.0)),
            ],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_math_pow_member_calls_with_zero_base_and_positive_integer_exponent() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "exponent".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(3.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Identifier("exponent".to_string())),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "pow".to_string(),
                })),
                args: vec![
                    Expression::Literal(LiteralValue::Number(0.0)),
                    Expression::Identifier("alias".to_string()),
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
fn test_resolution_supports_math_pow_member_calls_with_const_numeric_alias_exponents() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "exponent".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(3.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Identifier("exponent".to_string())),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "pow".to_string(),
                })),
                args: vec![
                    Expression::Literal(LiteralValue::Number(2.0)),
                    Expression::Identifier("alias".to_string()),
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
fn test_resolution_supports_math_pow_member_calls_with_negative_integer_base_and_const_numeric_alias_exponents(
) {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "exponent".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(3.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Identifier("exponent".to_string())),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "pow".to_string(),
                })),
                args: vec![
                    Expression::UnaryExpression(Box::new(kali_ast::UnaryExpression {
                        operator: "-".to_string(),
                        argument: Expression::Literal(LiteralValue::Number(2.0)),
                    })),
                    Expression::Identifier("alias".to_string()),
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
fn test_resolution_supports_math_pow_member_calls_with_negative_integer_exponent_for_unit_bases() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "negative_exponent".to_string(),
                init: Some(Expression::UnaryExpression(Box::new(
                    kali_ast::UnaryExpression {
                        operator: "-".to_string(),
                        argument: Expression::Literal(LiteralValue::Number(3.0)),
                    },
                ))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Identifier("negative_exponent".to_string())),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "pow".to_string(),
                })),
                args: vec![
                    Expression::Literal(LiteralValue::Number(1.0)),
                    Expression::Identifier("alias".to_string()),
                ],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "pow".to_string(),
                })),
                args: vec![
                    Expression::UnaryExpression(Box::new(kali_ast::UnaryExpression {
                        operator: "-".to_string(),
                        argument: Expression::Literal(LiteralValue::Number(1.0)),
                    })),
                    Expression::Identifier("alias".to_string()),
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
fn test_resolution_reports_unsupported_math_pow_negative_exponents_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "pow".to_string(),
            })),
            args: vec![
                Expression::Literal(LiteralValue::Number(2.0)),
                Expression::UnaryExpression(Box::new(kali_ast::UnaryExpression {
                    operator: "-".to_string(),
                    argument: Expression::Literal(LiteralValue::Number(1.0)),
                })),
            ],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(result.diagnostics[0]
        .message
        .contains("negative numeric literals"));
}

#[test]
fn test_resolution_supports_math_hypot_member_calls_with_const_numeric_alias_chain() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(3.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Identifier("value".to_string())),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "hypot".to_string(),
                })),
                args: vec![
                    Expression::Identifier("alias".to_string()),
                    Expression::Literal(LiteralValue::Number(4.0)),
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
fn test_resolution_supports_math_hypot_member_calls_with_empty_argument_list() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "hypot".to_string(),
            })),
            args: vec![],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_math_imul_with_omitted_operands() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "imul".to_string(),
                })),
                args: vec![],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "imul".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(7.0))],
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
fn test_resolution_supports_global_this_math_hypot_member_calls_with_empty_argument_list() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("globalThis".to_string()),
                    property: "Math".to_string(),
                })),
                property: "hypot".to_string(),
            })),
            args: vec![],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_reports_unsupported_math_cbrt_member_calls_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "cbrt".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(28.0))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.cbrt")));
}

#[test]
fn test_resolution_reports_unsupported_math_hypot_member_calls_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "hypot".to_string(),
            })),
            args: vec![
                Expression::Literal(LiteralValue::Number(1.6)),
                Expression::Literal(LiteralValue::Number(2.0)),
            ],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.hypot")
            && diag.message.contains("perfect-square integer literal")));
}

#[test]
fn test_resolution_reports_unsupported_math_log2_member_calls_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "log2".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(12.0))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.log2")
            && diag.message.contains("positive power-of-two")));
}

#[test]
fn test_resolution_reports_unsupported_math_log10_member_calls_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "log10".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(12.0))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.log10")
            && diag.message.contains("positive power-of-ten")));
}

#[test]
fn test_resolution_supports_math_exp_and_log_exact_identity_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "one".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "exp".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "log".to_string(),
                })),
                args: vec![Expression::Identifier("one".to_string())],
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
fn test_resolution_supports_math_exp2_exact_identity_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Identifier("zero".to_string())),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "exp2".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
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
fn test_resolution_supports_math_exp2_non_negative_integer_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "exponent".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(2.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Identifier("exponent".to_string())),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "exp2".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
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
fn test_resolution_rejects_math_exp2_non_integer_literals_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "exp2".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(1.5))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.exp2")
            && diag.message.contains("non-negative integer")));
}

#[test]
fn test_resolution_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "one".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "atan2".to_string(),
                })),
                args: vec![
                    Expression::Identifier("zero".to_string()),
                    Expression::Identifier("one".to_string()),
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
fn test_resolution_traverses_extra_math_atan2_arguments_after_the_supported_slice() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "one".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "atan2".to_string(),
                })),
                args: vec![
                    Expression::Identifier("zero".to_string()),
                    Expression::Identifier("one".to_string()),
                    Expression::Identifier("missing".to_string()),
                ],
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diag| diag.code == Some(e3::UNDEFINED_IDENTIFIER as u32)),
        "expected the trailing argument to be resolved: {:?}",
        result.diagnostics
    );
    assert!(
        !result
            .diagnostics
            .iter()
            .any(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)),
        "unexpected unsupported-feature diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_traverses_extra_math_tan_arguments_after_the_supported_slice() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "tan".to_string(),
                })),
                args: vec![
                    Expression::Identifier("zero".to_string()),
                    Expression::Identifier("missing".to_string()),
                ],
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diag| diag.code == Some(e3::UNDEFINED_IDENTIFIER as u32)),
        "expected the trailing argument to be resolved: {:?}",
        result.diagnostics
    );
    assert!(
        !result
            .diagnostics
            .iter()
            .any(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)),
        "unexpected unsupported-feature diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_math_expm1_and_log1p_exact_identity_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "expm1".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "log1p".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
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
fn test_resolution_supports_math_expm1_and_log1p_const_numeric_alias_chain_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Identifier("zero".to_string())),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "expm1".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "log1p".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
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
fn test_resolution_reports_math_expm1_and_log1p_non_identity_literals_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "expm1".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "log1p".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
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
        .any(|diag| diag.message.contains("Math.expm1")
            && diag.message.contains("zero numeric literal")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.log1p")
            && diag.message.contains("zero numeric literal")));
}

#[test]
fn test_resolution_supports_math_asin_acos_atan_exact_identity_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "one".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "asin".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "acos".to_string(),
                })),
                args: vec![Expression::Identifier("one".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "atan".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
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
fn test_resolution_supports_math_asinh_acosh_atanh_exact_identity_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "one".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "asinh".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "acosh".to_string(),
                })),
                args: vec![Expression::Identifier("one".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "atanh".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
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
fn test_resolution_supports_math_sinh_cosh_tanh_exact_identity_literals() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "sinh".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "cosh".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "tanh".to_string(),
                })),
                args: vec![Expression::Identifier("zero".to_string())],
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
fn test_resolution_reports_math_sinh_cosh_tanh_non_identity_literals_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "sinh".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "cosh".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "tanh".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
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
        .any(|diag| diag.message.contains("Math.sinh")
            && diag.message.contains("zero numeric literal")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.cosh")
            && diag.message.contains("zero numeric literal")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.tanh")
            && diag.message.contains("zero numeric literal")));
}

#[test]
fn test_resolution_reports_math_asinh_acosh_atanh_non_identity_literals_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "asinh".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "acosh".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(0.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "atanh".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
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
        .any(|diag| diag.message.contains("Math.asinh")
            && diag.message.contains("zero numeric literal")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.acosh")
            && diag.message.contains("one numeric literal")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.atanh")
            && diag.message.contains("zero numeric literal")));
}

#[test]
fn test_resolution_reports_math_atan2_non_matching_literals_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "atan2".to_string(),
            })),
            args: vec![
                Expression::Literal(LiteralValue::Number(1.0)),
                Expression::Literal(LiteralValue::Number(1.0)),
            ],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result
        .diagnostics
        .iter()
        .all(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.atan2")));
}

#[test]
fn test_resolution_supports_math_atan2_member_calls_with_const_numeric_alias_chain() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "zero_alias".to_string(),
                init: Some(Expression::Identifier("zero".to_string())),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "one".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "one_alias".to_string(),
                init: Some(Expression::Identifier("one".to_string())),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "atan2".to_string(),
                })),
                args: vec![
                    Expression::Identifier("zero_alias".to_string()),
                    Expression::Identifier("one_alias".to_string()),
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
fn test_resolution_reports_math_max_without_arguments_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "max".to_string(),
            })),
            args: vec![],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(result.diagnostics[0]
        .message
        .contains("requires at least one argument"));
}

#[test]
fn test_resolution_reports_math_pow_with_single_argument_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "pow".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(2.0))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(result.diagnostics[0]
        .message
        .contains("requires at least two arguments"));
}

#[test]
fn test_resolution_supports_math_sqrt_member_calls_with_const_numeric_alias_chain() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(4.0))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Identifier("value".to_string())),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "sqrt".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
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
fn test_resolution_supports_math_cbrt_member_calls_with_negative_const_numeric_alias_chain() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::UnaryExpression(Box::new(
                    kali_ast::UnaryExpression {
                        operator: "-".to_string(),
                        argument: Expression::Literal(LiteralValue::Number(27.0)),
                    },
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
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "cbrt".to_string(),
                })),
                args: vec![Expression::Identifier("alias".to_string())],
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
fn test_resolution_rejects_negative_const_numeric_alias_exponents_in_math_pow_member_calls_as_unavailable(
) {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "exponent".to_string(),
                init: Some(Expression::UnaryExpression(Box::new(
                    kali_ast::UnaryExpression {
                        operator: "-".to_string(),
                        argument: Expression::Literal(LiteralValue::Number(1.0)),
                    },
                ))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Identifier("exponent".to_string())),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "pow".to_string(),
                })),
                args: vec![
                    Expression::Literal(LiteralValue::Number(2.0)),
                    Expression::Identifier("alias".to_string()),
                ],
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(result.diagnostics[0]
        .message
        .contains("negative numeric literals"));
}

#[test]
fn test_resolution_rejects_non_integer_const_numeric_alias_exponents_in_math_pow_member_calls_as_unavailable(
) {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "exponent".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.6))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "alias".to_string(),
                init: Some(Expression::Identifier("exponent".to_string())),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "pow".to_string(),
                })),
                args: vec![
                    Expression::Literal(LiteralValue::Number(2.0)),
                    Expression::Identifier("alias".to_string()),
                ],
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(result.diagnostics[0]
        .message
        .contains("non-integer numeric literals"));
}

#[test]
fn test_resolution_supports_promise_all_settled_member_calls() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Promise".to_string()),
                    property: "allSettled".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("globalThis".to_string()),
                        property: "Promise".to_string(),
                    })),
                    property: "allSettled".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(2.0))],
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
fn test_resolution_supports_math_tan_zero_literal_member_calls() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "tan".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(0.0))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_math_sin_cos_zero_literal_member_calls() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "sin".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(0.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "cos".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(0.0))],
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
fn test_resolution_supports_math_clz32_zero_argument_member_calls() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "clz32".to_string(),
            })),
            args: vec![],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_math_clz32_non_integer_literal_member_calls() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "clz32".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(1.6))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_rejects_non_zero_literals_in_math_tan_member_calls_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "tan".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(1.0))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(result.diagnostics[0].message.contains("Math.tan"));
}

#[test]
fn test_resolution_rejects_non_zero_literals_in_math_sin_cos_member_calls_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "sin".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "cos".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
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
        .any(|diag| diag.message.contains("Math.sin")
            && diag.message.contains("zero numeric literal")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.cos")
            && diag.message.contains("zero numeric literal")));
}

#[test]
fn test_resolution_supports_update_expressions_on_mutable_bindings() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::UpdateExpression(Box::new(UpdateExpression {
                operator: UpdateOperator::Increment,
                argument: Expression::Identifier("value".to_string()),
                prefix: true,
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::UpdateExpression(Box::new(UpdateExpression {
                operator: UpdateOperator::Decrement,
                argument: Expression::Identifier("value".to_string()),
                prefix: false,
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                AssignmentExpression {
                    operator: AssignmentOperator::ExponentAssign,
                    left: Expression::ParenthesizedExpression(Box::new(ParenthesizedExpression {
                        expression: Box::new(Expression::Identifier("value".to_string())),
                    })),
                    right: Expression::Literal(LiteralValue::Number(2.0)),
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
fn test_resolution_rejects_update_expressions_on_immutable_bindings() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::UpdateExpression(Box::new(UpdateExpression {
                operator: UpdateOperator::Increment,
                argument: Expression::Identifier("value".to_string()),
                prefix: true,
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(result.diagnostics[0]
        .message
        .contains("mutable local binding"));
}

#[test]
fn test_resolution_rejects_compound_assignment_on_non_local_targets_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "target".to_string(),
                init: Some(Expression::ObjectExpression(ObjectExpression {
                    properties: vec![ObjectProperty {
                        kind: ObjectPropertyKind::Init,
                        key: PropertyName::Identifier("value".to_string()),
                        value: Expression::Literal(LiteralValue::Number(1.0)),
                    }],
                })),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                AssignmentExpression {
                    operator: AssignmentOperator::AddAssign,
                    left: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("target".to_string()),
                        property: "value".to_string(),
                    })),
                    right: Expression::Literal(LiteralValue::Number(2.0)),
                },
            ))),
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                AssignmentExpression {
                    operator: AssignmentOperator::AddAssign,
                    left: Expression::Identifier("value".to_string()),
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
        .any(|diag| diag.message.contains("compound assignment lowering")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("binding 'value'")));
}

#[test]
fn test_resolution_accepts_decorated_wrappers_for_update_targets() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                AssignmentExpression {
                    operator: AssignmentOperator::AddAssign,
                    left: Expression::DecoratedExpression(DecoratedExpression {
                        expression: Box::new(Expression::Identifier("value".to_string())),
                    }),
                    right: Expression::Literal(LiteralValue::Number(2.0)),
                },
            ))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::UpdateExpression(Box::new(UpdateExpression {
                operator: UpdateOperator::Increment,
                argument: Expression::DecoratedExpression(DecoratedExpression {
                    expression: Box::new(Expression::Identifier("value".to_string())),
                }),
                prefix: true,
            }))),
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

#[test]
fn test_resolution_reports_non_identity_literals_in_math_asin_acos_atan_member_calls_as_unavailable(
) {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "asin".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "acos".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(0.0))],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Math".to_string()),
                    property: "atan".to_string(),
                })),
                args: vec![Expression::Literal(LiteralValue::Number(1.0))],
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
        .any(|diag| diag.message.contains("Math.asin")
            && diag.message.contains("zero numeric literal")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.acos")
            && diag.message.contains("one numeric literal")));
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.message.contains("Math.atan")
            && diag.message.contains("zero numeric literal")));
}

#[test]
fn test_resolution_supports_non_integer_numeric_literals_in_math_ceil_member_calls() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "ceil".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(1.6))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_non_integer_numeric_literals_in_math_trunc_member_calls() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "trunc".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(1.6))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_non_integer_numeric_literals_in_math_sign_member_calls() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Math".to_string()),
                property: "sign".to_string(),
            })),
            args: vec![Expression::Literal(LiteralValue::Number(1.6))],
        }))),
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
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
}

#[test]
fn test_resolution_supports_object_has_own_helpers_for_static_object_literals_and_alias_chains() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "object".to_string(),
                init: Some(Expression::ObjectExpression(ObjectExpression {
                    properties: vec![ObjectProperty {
                        key: PropertyName::Identifier("a".to_string()),
                        value: Expression::Literal(LiteralValue::Number(1.0)),
                        kind: ObjectPropertyKind::Init,
                    }],
                })),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "has_own".to_string(),
                init: Some(Expression::MemberExpression(Box::new(
                    kali_ast::MemberExpression {
                        object: Expression::Identifier("Object".to_string()),
                        property: "hasOwn".to_string(),
                    },
                ))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "has_own_property_call".to_string(),
                init: Some(Expression::MemberExpression(Box::new(
                    kali_ast::MemberExpression {
                        object: Expression::MemberExpression(Box::new(
                            kali_ast::MemberExpression {
                                object: Expression::MemberExpression(Box::new(
                                    kali_ast::MemberExpression {
                                        object: Expression::Identifier("Object".to_string()),
                                        property: "prototype".to_string(),
                                    },
                                )),
                                property: "hasOwnProperty".to_string(),
                            },
                        )),
                        property: "call".to_string(),
                    },
                ))),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::Identifier("has_own".to_string()),
                args: vec![
                    Expression::Identifier("object".to_string()),
                    Expression::Literal(LiteralValue::String("a".to_string())),
                ],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::Identifier("has_own_property_call".to_string()),
                args: vec![
                    Expression::Identifier("object".to_string()),
                    Expression::Literal(LiteralValue::String("a".to_string())),
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
fn test_resolution_allows_static_dynamic_import_targets() {
    let dir = tempfile::tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
fn test_resolution_accepts_directory_index_dynamic_import_targets_in_jsx_files() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.jsx");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).unwrap();
    fs::write(lazy_dir.join("index.jsx"), "export const lazy = 7;").unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
fn test_resolution_rejects_non_literal_dynamic_import_targets() {
    let dir = tempfile::tempdir().unwrap();
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

#[test]
fn test_resolution_rejects_generator_function_lowering() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "function* main() { yield* []; }\nmain();").unwrap();

    let statements = vec![Statement::FunctionDeclaration(FunctionDeclaration {
        name: "main".to_string(),
        params: vec![],
        body: Box::new(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::YieldExpression(Box::new(YieldExpression {
                    delegate: true,
                    argument: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                        elements: vec![],
                    })),
                }))),
            })],
        }),
        is_async: false,
        generator: true,
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
        .contains("generator function lowering is unavailable"));
}

#[test]
fn test_resolution_rejects_generator_function_lowering_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "function* main() { yield* []; }\nmain();").unwrap();

    let statements = vec![Statement::FunctionDeclaration(FunctionDeclaration {
        name: "main".to_string(),
        params: vec![],
        body: Box::new(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::YieldExpression(Box::new(YieldExpression {
                    delegate: true,
                    argument: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                        elements: vec![],
                    })),
                }))),
            })],
        }),
        is_async: false,
        generator: true,
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
        .contains("generator function lowering is unavailable"));
}

#[test]
fn test_resolution_rejects_async_generator_function_lowering_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "async function* main() { yield 1; }\nmain();").unwrap();

    let statements = vec![Statement::FunctionDeclaration(FunctionDeclaration {
        name: "main".to_string(),
        params: vec![],
        body: Box::new(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::YieldExpression(Box::new(YieldExpression {
                    delegate: false,
                    argument: Some(Expression::Literal(LiteralValue::Number(1.0))),
                }))),
            })],
        }),
        is_async: true,
        generator: true,
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
        .contains("async-generator function lowering is unavailable"));
}

#[test]
fn test_resolution_rejects_generator_function_lowering_in_jsx_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.jsx");
    fs::write(&source_path, "function* main() { yield* []; }\nmain();").unwrap();

    let statements = vec![Statement::FunctionDeclaration(FunctionDeclaration {
        name: "main".to_string(),
        params: vec![],
        body: Box::new(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::YieldExpression(Box::new(YieldExpression {
                    delegate: true,
                    argument: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                        elements: vec![],
                    })),
                }))),
            })],
        }),
        is_async: false,
        generator: true,
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
        .contains("generator function lowering is unavailable"));
}

#[test]
fn test_resolution_rejects_generator_function_lowering_in_tsx_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.tsx");
    fs::write(&source_path, "function* main() { yield* []; }\nmain();").unwrap();

    let statements = vec![Statement::FunctionDeclaration(FunctionDeclaration {
        name: "main".to_string(),
        params: vec![],
        body: Box::new(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::YieldExpression(Box::new(YieldExpression {
                    delegate: true,
                    argument: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                        elements: vec![],
                    })),
                }))),
            })],
        }),
        is_async: false,
        generator: true,
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
        .contains("generator function lowering is unavailable"));
}

#[test]
fn test_resolution_rejects_class_method_generator_lowering() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");

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
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");

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
fn test_resolution_supports_async_class_method_lowering() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");

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

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_process_kill_zero_probe_wrappers_on_node_surface() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "process.kill((0)); globalThis.process.kill(+0); process[\"kill\"]((0)); globalThis.process[\"kill\"](+0);",
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
    let source = r#"process["kill"]((0)); globalThis["process"]["kill"](+0); ((process["kill"]))(0); ((globalThis["process"]["kill"]))(0);"#;
    fs::write(&source_path, source).unwrap();

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
        "const zero = 0; const zeroAlias = zero; process.kill(zeroAlias); globalThis.process.kill(+zero);",
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
    fs::write(
        &source_path,
        "process.kill((0 satisfies number)); globalThis.process.kill((0 satisfies number)); globalThis[\"process\"][\"kill\"]((0 satisfies number));",
    )
    .unwrap();

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
}

#[test]
fn test_resolution_supports_for_of_object_entries_iteration() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "for (const entry of Object.entries({ \"b\": 1, \"a\": 2 })) { console.log(entry[0]); console.log(entry[1]); }",
    )
    .unwrap();

    let statements = vec![Statement::ForOfStatement(ForOfStatement {
        left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "entry".to_string(),
                init: None,
            }],
        }),
        right: Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Object".to_string()),
                property: "entries".to_string(),
            })),
            args: vec![Expression::ObjectExpression(ObjectExpression {
                properties: vec![ObjectProperty {
                    key: PropertyName::String("b".to_string()),
                    value: Expression::Literal(LiteralValue::Number(1.0)),
                    kind: ObjectPropertyKind::Init,
                }],
            })],
        })),
        body: Box::new(Statement::BlockStatement(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("console".to_string()),
                        property: "log".to_string(),
                    })),
                    args: vec![Expression::Identifier("entry".to_string())],
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

fn assert_resolution_rejects_unimplemented_iterator_protocol_edge(
    source_filename: &str,
    constructor_name: &str,
) {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join(source_filename);
    fs::write(
        &source_path,
        format!("for (const item of new {constructor_name}()) {{ console.log(item); }}"),
    )
    .unwrap();

    let statements = vec![Statement::ForOfStatement(ForOfStatement {
        left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "item".to_string(),
                init: None,
            }],
        }),
        right: Expression::NewExpression(Box::new(kali_ast::NewExpression {
            callee: Expression::Identifier(constructor_name.to_string()),
            args: vec![],
        })),
        body: Box::new(Statement::BlockStatement(BlockStatement { body: vec![] })),
        is_await: false,
    })];

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
}

#[test]
fn test_resolution_rejects_for_of_set_iteration_in_js_input() {
    assert_resolution_rejects_unimplemented_iterator_protocol_edge("main.js", "Set");
}

#[test]
fn test_resolution_rejects_for_of_map_iteration_in_ts_input() {
    assert_resolution_rejects_unimplemented_iterator_protocol_edge("main.ts", "Map");
}

#[test]
fn test_resolution_accepts_new_map_iteration_target_in_js_input() {
    let dir = tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    let source = r#"for (const entry of new Map([[1, 2], [1, 3], [4, 5]])) {
    console.log(entry[0], entry[1]);
}
"#;
    fs::write(&source_path, source).unwrap();

    let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let Statement::ForOfStatement(ForOfStatement { right, .. }) = &statements[0] else {
        panic!("expected for-of statement");
    };

    let mut ctx = TypeContext::with_base_path(&source_path);
    assert!(ctx.is_static_array_iteration_target(right));

    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_object_keys_iteration_with_let_binding_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "let values = { a: 1 }; for (const key of Object.keys(values)) { console.log(key); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ObjectExpression(ObjectExpression {
                    properties: vec![ObjectProperty {
                        key: PropertyName::Identifier("a".to_string()),
                        value: Expression::Literal(LiteralValue::Number(1.0)),
                        kind: ObjectPropertyKind::Init,
                    }],
                })),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "key".to_string(),
                    init: None,
                }],
            }),
            right: Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "keys".to_string(),
                })),
                args: vec![Expression::Identifier("values".to_string())],
            })),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("key".to_string())],
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
fn test_resolution_rejects_object_keys_iteration_with_let_binding_rebound_before_use_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "let values = { a: 1 }; values = { b: 2 }; for (const key of Object.keys(values)) { console.log(key); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ObjectExpression(ObjectExpression {
                    properties: vec![ObjectProperty {
                        key: PropertyName::Identifier("a".to_string()),
                        value: Expression::Literal(LiteralValue::Number(1.0)),
                        kind: ObjectPropertyKind::Init,
                    }],
                })),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                AssignmentExpression {
                    operator: AssignmentOperator::Assign,
                    left: Expression::Identifier("values".to_string()),
                    right: Expression::ObjectExpression(ObjectExpression {
                        properties: vec![ObjectProperty {
                            key: PropertyName::Identifier("b".to_string()),
                            value: Expression::Literal(LiteralValue::Number(2.0)),
                            kind: ObjectPropertyKind::Init,
                        }],
                    }),
                },
            ))),
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "key".to_string(),
                    init: None,
                }],
            }),
            right: Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "keys".to_string(),
                })),
                args: vec![Expression::Identifier("values".to_string())],
            })),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("key".to_string())],
                    }))),
                })],
            })),
            is_await: false,
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
}

fn assert_object_helper_iteration_with_let_binding_in_js_input(helper: &str, rebound: bool) {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    let source = if rebound {
        format!(
            "let values = {{ a: 1 }}; values = {{ b: 2 }}; for (const item of Object.{helper}(values)) {{ console.log(item); }}",
            helper = helper,
        )
    } else {
        format!(
            "let values = {{ a: 1 }}; for (const item of Object.{helper}(values)) {{ console.log(item); }}",
            helper = helper,
        )
    };
    fs::write(&source_path, source).unwrap();

    let mut statements = vec![Statement::VariableDeclaration(VariableDeclaration {
        kind: "let".to_string(),
        declarations: vec![VariableDeclarator {
            id: "values".to_string(),
            init: Some(Expression::ObjectExpression(ObjectExpression {
                properties: vec![ObjectProperty {
                    key: PropertyName::Identifier("a".to_string()),
                    value: Expression::Literal(LiteralValue::Number(1.0)),
                    kind: ObjectPropertyKind::Init,
                }],
            })),
        }],
    })];

    if rebound {
        statements.push(Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                AssignmentExpression {
                    operator: AssignmentOperator::Assign,
                    left: Expression::Identifier("values".to_string()),
                    right: Expression::ObjectExpression(ObjectExpression {
                        properties: vec![ObjectProperty {
                            key: PropertyName::Identifier("b".to_string()),
                            value: Expression::Literal(LiteralValue::Number(2.0)),
                            kind: ObjectPropertyKind::Init,
                        }],
                    }),
                },
            ))),
        }));
    }

    statements.push(Statement::ForOfStatement(ForOfStatement {
        left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "item".to_string(),
                init: None,
            }],
        }),
        right: Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Object".to_string()),
                property: helper.to_string(),
            })),
            args: vec![Expression::Identifier("values".to_string())],
        })),
        body: Box::new(Statement::BlockStatement(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("console".to_string()),
                        property: "log".to_string(),
                    })),
                    args: vec![Expression::Identifier("item".to_string())],
                }))),
            })],
        })),
        is_await: false,
    }));

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    if rebound {
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.code == Some(e5::FEATURE_UNAVAILABLE as u32)),
            "unexpected diagnostics: {:?}",
            result.diagnostics
        );
    } else {
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_resolution_supports_object_values_iteration_with_let_binding_in_js_input() {
    assert_object_helper_iteration_with_let_binding_in_js_input("values", false);
}

#[test]
fn test_resolution_rejects_object_values_iteration_with_let_binding_rebound_before_use_in_js_input()
{
    assert_object_helper_iteration_with_let_binding_in_js_input("values", true);
}

#[test]
fn test_resolution_supports_object_entries_iteration_with_let_binding_in_js_input() {
    assert_object_helper_iteration_with_let_binding_in_js_input("entries", false);
}

#[test]
fn test_resolution_rejects_object_entries_iteration_with_let_binding_rebound_before_use_in_js_input(
) {
    assert_object_helper_iteration_with_let_binding_in_js_input("entries", true);
}

#[test]
fn test_resolution_supports_for_of_array_iteration() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "for (const value of [1, 2]) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![Statement::ForOfStatement(ForOfStatement {
        left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: None,
            }],
        }),
        right: Expression::ArrayExpression(kali_ast::ArrayExpression {
            elements: vec![
                Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::Literal(LiteralValue::Number(1.0)),
                )),
                Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::Literal(LiteralValue::Number(2.0)),
                )),
            ],
        }),
        body: Box::new(Statement::BlockStatement(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("console".to_string()),
                        property: "log".to_string(),
                    })),
                    args: vec![Expression::Identifier("value".to_string())],
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
fn test_resolution_supports_for_of_array_iteration_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "for (const value of [1, 2]) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![Statement::ForOfStatement(ForOfStatement {
        left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: None,
            }],
        }),
        right: Expression::ArrayExpression(kali_ast::ArrayExpression {
            elements: vec![
                Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::Literal(LiteralValue::Number(1.0)),
                )),
                Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::Literal(LiteralValue::Number(2.0)),
                )),
            ],
        }),
        body: Box::new(Statement::BlockStatement(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("console".to_string()),
                        property: "log".to_string(),
                    })),
                    args: vec![Expression::Identifier("value".to_string())],
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
fn test_resolution_supports_for_of_array_iteration_with_sequence_wrappers_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "let value = 0; for ((0, value) of (0, [(0, 1), (0, 2)])) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(sequence_expression(vec![
                Expression::Literal(LiteralValue::Number(0.0)),
                Expression::Identifier("value".to_string()),
            ])),
            right: sequence_expression(vec![
                Expression::Literal(LiteralValue::Number(0.0)),
                Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            sequence_expression(vec![
                                Expression::Literal(LiteralValue::Number(0.0)),
                                Expression::Literal(LiteralValue::Number(1.0)),
                            ]),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            sequence_expression(vec![
                                Expression::Literal(LiteralValue::Number(0.0)),
                                Expression::Literal(LiteralValue::Number(2.0)),
                            ]),
                        )),
                    ],
                }),
            ]),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("value".to_string())],
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
fn test_resolution_rejects_for_of_non_literal_iterable_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                kali_ast::AssignmentExpression {
                    operator: AssignmentOperator::Assign,
                    left: Expression::Identifier("values".to_string()),
                    right: Expression::ArrayExpression(kali_ast::ArrayExpression {
                        elements: vec![
                            Some(kali_ast::ExpressionOrSpread::Expression(
                                Expression::Literal(LiteralValue::Number(3.0)),
                            )),
                            Some(kali_ast::ExpressionOrSpread::Expression(
                                Expression::Literal(LiteralValue::Number(4.0)),
                            )),
                        ],
                    }),
                },
            ))),
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "item".to_string(),
                    init: None,
                }],
            }),
            right: Expression::Identifier("values".to_string()),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::Identifier("item".to_string())),
                })],
            })),
            is_await: false,
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("literal array"),
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_rejects_for_await_non_literal_iterable_as_unavailable() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                kali_ast::AssignmentExpression {
                    operator: AssignmentOperator::Assign,
                    left: Expression::Identifier("values".to_string()),
                    right: Expression::ArrayExpression(kali_ast::ArrayExpression {
                        elements: vec![
                            Some(kali_ast::ExpressionOrSpread::Expression(
                                Expression::Literal(LiteralValue::Number(3.0)),
                            )),
                            Some(kali_ast::ExpressionOrSpread::Expression(
                                Expression::Literal(LiteralValue::Number(4.0)),
                            )),
                        ],
                    }),
                },
            ))),
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "item".to_string(),
                    init: None,
                }],
            }),
            right: Expression::Identifier("values".to_string()),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::Identifier("item".to_string())),
                })],
            })),
            is_await: true,
        }),
    ];

    let result = ctx.resolve_statements(&statements);
    assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
    assert_eq!(
        result.diagnostics[0].code,
        Some(e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(
        result.diagnostics[0].message.contains("literal array"),
        "{:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_supports_for_of_array_iteration_with_parenthesized_binding_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "let value = 0; for ((value) of [1, 2]) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(Expression::ParenthesizedExpression(Box::new(
                kali_ast::ParenthesizedExpression {
                    expression: Box::new(Expression::Identifier("value".to_string())),
                },
            ))),
            right: Expression::ArrayExpression(kali_ast::ArrayExpression {
                elements: vec![
                    Some(kali_ast::ExpressionOrSpread::Expression(
                        Expression::Literal(LiteralValue::Number(1.0)),
                    )),
                    Some(kali_ast::ExpressionOrSpread::Expression(
                        Expression::Literal(LiteralValue::Number(2.0)),
                    )),
                ],
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("value".to_string())],
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
fn test_resolution_supports_for_of_array_iteration_with_parenthesized_binding_in_ts_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "let value = 0; for ((value) of [1, 2]) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(Expression::ParenthesizedExpression(Box::new(
                kali_ast::ParenthesizedExpression {
                    expression: Box::new(Expression::Identifier("value".to_string())),
                },
            ))),
            right: Expression::ArrayExpression(kali_ast::ArrayExpression {
                elements: vec![
                    Some(kali_ast::ExpressionOrSpread::Expression(
                        Expression::Literal(LiteralValue::Number(1.0)),
                    )),
                    Some(kali_ast::ExpressionOrSpread::Expression(
                        Expression::Literal(LiteralValue::Number(2.0)),
                    )),
                ],
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("value".to_string())],
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
fn test_resolution_rejects_for_of_array_iteration_with_identifier_iterable() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const items = [1, 2];\nfor (const value of items) { console.log(value); }\n",
    )
    .unwrap();

    let statements = vec![Statement::ForOfStatement(ForOfStatement {
        left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: None,
            }],
        }),
        right: Expression::Identifier("items".to_string()),
        body: Box::new(Statement::BlockStatement(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("console".to_string()),
                        property: "log".to_string(),
                    })),
                    args: vec![Expression::Identifier("value".to_string())],
                }))),
            })],
        })),
        is_await: false,
    })];

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
}

#[test]
fn test_resolution_supports_for_of_array_iteration_with_const_alias_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const values = [1, 2]; for (const value of values) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "value".to_string(),
                    init: None,
                }],
            }),
            right: Expression::Identifier("values".to_string()),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("value".to_string())],
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
fn test_resolution_supports_for_of_array_iteration_with_const_alias_in_ts_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const values = [1, 2]; for (const value of values) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "value".to_string(),
                    init: None,
                }],
            }),
            right: Expression::Identifier("values".to_string()),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("value".to_string())],
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
fn test_resolution_supports_for_of_array_iteration_with_let_binding_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "let values = [1, 2]; for (const value of values) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "value".to_string(),
                    init: None,
                }],
            }),
            right: Expression::Identifier("values".to_string()),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("value".to_string())],
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
fn test_resolution_rejects_for_of_array_iteration_with_let_binding_rebound_before_use_in_js_input()
{
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "let values = [1, 2]; values = [3, 4]; for (const value of values) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
            }],
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::AssignmentExpression(Box::new(
                AssignmentExpression {
                    operator: AssignmentOperator::Assign,
                    left: Expression::Identifier("values".to_string()),
                    right: Expression::ArrayExpression(kali_ast::ArrayExpression {
                        elements: vec![
                            Some(kali_ast::ExpressionOrSpread::Expression(
                                Expression::Literal(LiteralValue::Number(3.0)),
                            )),
                            Some(kali_ast::ExpressionOrSpread::Expression(
                                Expression::Literal(LiteralValue::Number(4.0)),
                            )),
                        ],
                    }),
                },
            ))),
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
                kind: "const".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "value".to_string(),
                    init: None,
                }],
            }),
            right: Expression::Identifier("values".to_string()),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
                            object: Expression::Identifier("console".to_string()),
                            property: "log".to_string(),
                        })),
                        args: vec![Expression::Identifier("value".to_string())],
                    }))),
                })],
            })),
            is_await: false,
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
}

#[test]
fn test_resolution_supports_for_of_array_iteration_with_const_string_alias_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
fn test_resolution_supports_for_of_array_iteration_with_const_numeric_alias_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = 1; const alias = value; for (const item of [alias]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
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
fn test_resolution_supports_for_of_array_iteration_with_const_numeric_alias_in_ts_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const value = 1; const alias = value; for (const item of [alias]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
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
fn test_resolution_supports_for_of_array_iteration_with_decorated_wrappers_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "let item = 0; for ((item) of [1, 2]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "item".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::Identifier("item".to_string())),
                    },
                ))),
            })),
            right: Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    1.0,
                                ))),
                            }),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    2.0,
                                ))),
                            }),
                        )),
                    ],
                })),
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
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
fn test_resolution_supports_for_of_array_iteration_with_decorated_wrappers_in_ts_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "let item = 0; for ((item) of [1, 2]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "item".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::Identifier("item".to_string())),
                    },
                ))),
            })),
            right: Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    1.0,
                                ))),
                            }),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    2.0,
                                ))),
                            }),
                        )),
                    ],
                })),
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
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
fn test_resolution_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_js_input(
) {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const values = [1, 2]; for (const item of [...values]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
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
                elements: vec![Some(kali_ast::ExpressionOrSpread::Spread(
                    kali_ast::SpreadElement {
                        argument: Expression::Identifier("values".to_string()),
                    },
                ))],
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
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
fn test_resolution_supports_for_of_array_iteration_with_decorated_spread_targets_in_ts_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
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
                elements: vec![Some(kali_ast::ExpressionOrSpread::Spread(
                    kali_ast::SpreadElement {
                        argument: Expression::DecoratedExpression(DecoratedExpression {
                            expression: Box::new(Expression::Identifier("values".to_string())),
                        }),
                    },
                ))],
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
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
fn test_resolution_supports_for_of_array_iteration_with_decorated_spread_targets_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
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
                elements: vec![Some(kali_ast::ExpressionOrSpread::Spread(
                    kali_ast::SpreadElement {
                        argument: Expression::DecoratedExpression(DecoratedExpression {
                            expression: Box::new(Expression::Identifier("values".to_string())),
                        }),
                    },
                ))],
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
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
fn test_resolution_supports_for_await_of_array_iteration_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "for await (const value of [1, 2]) { console.log(value); }",
    )
    .unwrap();

    let statements = vec![Statement::ForOfStatement(ForOfStatement {
        left: ForOfLefthand::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: None,
            }],
        }),
        right: Expression::ArrayExpression(kali_ast::ArrayExpression {
            elements: vec![
                Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::Literal(LiteralValue::Number(1.0)),
                )),
                Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::Literal(LiteralValue::Number(2.0)),
                )),
            ],
        }),
        body: Box::new(Statement::BlockStatement(BlockStatement {
            body: vec![Statement::ExpressionStatement(ExpressionStatement {
                expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                    callee: Expression::MemberExpression(Box::new(MemberExpression {
                        object: Expression::Identifier("console".to_string()),
                        property: "log".to_string(),
                    })),
                    args: vec![Expression::Identifier("value".to_string())],
                }))),
            })],
        })),
        is_await: true,
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
fn test_resolution_supports_for_await_of_array_iteration_with_sequence_wrappers_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "let item = 0; for await ((0, item) of (0, [(0, 1), (0, 2)])) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "item".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(sequence_expression(vec![
                Expression::Literal(LiteralValue::Number(0.0)),
                Expression::Identifier("item".to_string()),
            ])),
            right: sequence_expression(vec![
                Expression::Literal(LiteralValue::Number(0.0)),
                Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            sequence_expression(vec![
                                Expression::Literal(LiteralValue::Number(0.0)),
                                Expression::Literal(LiteralValue::Number(1.0)),
                            ]),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            sequence_expression(vec![
                                Expression::Literal(LiteralValue::Number(0.0)),
                                Expression::Literal(LiteralValue::Number(2.0)),
                            ]),
                        )),
                    ],
                }),
            ]),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
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
fn test_resolution_supports_for_await_of_array_iteration_with_decorated_wrappers_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "let item = 0; for await ((item) of [1, 2]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "item".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::Identifier("item".to_string())),
                    },
                ))),
            })),
            right: Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    1.0,
                                ))),
                            }),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    2.0,
                                ))),
                            }),
                        )),
                    ],
                })),
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
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
fn test_resolution_supports_for_await_of_array_iteration_with_decorated_wrappers_in_jsx_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.jsx");
    fs::write(
        &source_path,
        "let item = 0; for await ((item) of [1, 2]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "item".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::Identifier("item".to_string())),
                    },
                ))),
            })),
            right: Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    1.0,
                                ))),
                            }),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    2.0,
                                ))),
                            }),
                        )),
                    ],
                })),
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
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
fn test_resolution_supports_for_await_of_array_iteration_with_decorated_wrappers_in_ts_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "let item = 0; for await ((item) of [1, 2]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "item".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::Identifier("item".to_string())),
                    },
                ))),
            })),
            right: Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    1.0,
                                ))),
                            }),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    2.0,
                                ))),
                            }),
                        )),
                    ],
                })),
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
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
fn test_resolution_supports_for_await_of_array_iteration_with_decorated_wrappers_in_tsx_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.tsx");
    fs::write(
        &source_path,
        "let item = 0; for await ((item) of [1, 2]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "let".to_string(),
            declarations: vec![VariableDeclarator {
                id: "item".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(0.0))),
            }],
        }),
        Statement::ForOfStatement(ForOfStatement {
            left: ForOfLefthand::Expression(Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::Identifier("item".to_string())),
                    },
                ))),
            })),
            right: Expression::DecoratedExpression(DecoratedExpression {
                expression: Box::new(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    1.0,
                                ))),
                            }),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::DecoratedExpression(DecoratedExpression {
                                expression: Box::new(Expression::Literal(LiteralValue::Number(
                                    2.0,
                                ))),
                            }),
                        )),
                    ],
                })),
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
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
fn test_resolution_supports_for_await_of_array_iteration_with_decorated_spread_targets_in_js_input()
{
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
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
                elements: vec![Some(kali_ast::ExpressionOrSpread::Spread(
                    kali_ast::SpreadElement {
                        argument: Expression::DecoratedExpression(DecoratedExpression {
                            expression: Box::new(Expression::Identifier("values".to_string())),
                        }),
                    },
                ))],
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
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
fn test_resolution_supports_for_await_of_array_iteration_with_decorated_spread_targets_in_ts_input()
{
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "values".to_string(),
                init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                    elements: vec![
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(1.0)),
                        )),
                        Some(kali_ast::ExpressionOrSpread::Expression(
                            Expression::Literal(LiteralValue::Number(2.0)),
                        )),
                    ],
                })),
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
                elements: vec![Some(kali_ast::ExpressionOrSpread::Spread(
                    kali_ast::SpreadElement {
                        argument: Expression::DecoratedExpression(DecoratedExpression {
                            expression: Box::new(Expression::Identifier("values".to_string())),
                        }),
                    },
                ))],
            }),
            body: Box::new(Statement::BlockStatement(BlockStatement {
                body: vec![Statement::ExpressionStatement(ExpressionStatement {
                    expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                        callee: Expression::MemberExpression(Box::new(MemberExpression {
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
fn test_resolution_supports_for_await_of_array_iteration_with_const_alias_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = 1; const alias = value; for await (const item of [alias]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
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
fn test_resolution_supports_for_await_of_array_iteration_with_const_alias_in_ts_input() {
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const value = 1; const alias = value; for await (const item of [alias]) { console.log(item); }",
    )
    .unwrap();

    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "value".to_string(),
                init: Some(Expression::Literal(LiteralValue::Number(1.0))),
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
fn test_resolution_supports_for_await_of_array_iteration_with_const_string_alias_in_ts_input() {
    let dir = tempfile::tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
fn test_resolution_uses_project_root_for_materialized_packages() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "devDependencies": {
    "@types/lodash": "1.0.0"
  }
}"#,
    )
    .unwrap();

    let src_dir = dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    let source_path = src_dir.join("main.ts");
    fs::write(&source_path, "import lodash from 'lodash';\n").unwrap();

    let types_dir = dir.path().join("node_modules/@types/lodash");
    fs::create_dir_all(&types_dir).unwrap();
    fs::write(
        types_dir.join("package.json"),
        r#"{"name":"@types/lodash","types":"index.d.ts"}"#,
    )
    .unwrap();
    fs::write(types_dir.join("index.d.ts"), "declare const _: number;").unwrap();

    let mut ctx = TypeContext::with_base_path(&source_path);
    let statements = vec![Statement::ImportDeclaration(ImportDeclaration {
        specifiers: vec![ImportSpecifier::Default("lodash".to_string())],
        source: "lodash".to_string(),
    })];

    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}
