use super::*;

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
                    computed_index: None,
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
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::MemberExpression(Box::new(MemberExpression {
                            computed_index: None,
                            object: Expression::Identifier("Object".to_string()),
                            property: "prototype".to_string(),
                        })),
                        property: "hasOwnProperty".to_string(),
                    })),
                    property: "call".to_string(),
                }))),
            }],
        }),
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "sequenced_object".to_string(),
                init: Some(sequence_expression(vec![
                    Expression::Literal(LiteralValue::Number(0.0)),
                    Expression::Identifier("object".to_string()),
                ])),
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
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::Identifier("has_own".to_string()),
                args: vec![
                    Expression::Identifier("sequenced_object".to_string()),
                    Expression::Literal(LiteralValue::String("a".to_string())),
                ],
            }))),
        }),
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::Identifier("has_own_property_call".to_string()),
                args: vec![
                    Expression::Identifier("sequenced_object".to_string()),
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
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const object = { a: 1, "b": 2 };
const alias = object;
Object.hasOwn(alias, "a");
globalThis.Object.hasOwnProperty.call(alias, "a");
globalThis["Object"]["hasOwnProperty"].call(alias, "a");
globalThis["Object"].hasOwnProperty.call(alias, "a");
globalThis["Object"]["hasOwnProperty"]["call"](alias, "a");
Object["hasOwnProperty"].call(alias, "a");
Object["hasOwnProperty"]["call"](alias, "a");
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
                    computed_index: None,
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
                    computed_index: None,
                    object: Expression::MemberExpression(Box::new(MemberExpression {
                        computed_index: None,
                        object: Expression::MemberExpression(Box::new(MemberExpression {
                            computed_index: None,
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
fn test_resolution_supports_bracketed_and_frozen_object_has_own_aliases_in_js_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    let source = r#"const object = Object.fromEntries([["a", 1], ["b", 2]]);
const alias = object;
const bracketedHasOwn = Object["hasOwn"];
const globalThisBracketedHasOwn = globalThis["Object"]["hasOwn"];
const frozenBracketedHasOwn = Object.freeze(globalThis["Object"]["hasOwn"]);
const frozenParenthesizedBracketedHasOwn = Object.freeze((globalThis["Object"])["hasOwn"]);
bracketedHasOwn(alias, "a");
globalThisBracketedHasOwn(alias, "a");
frozenBracketedHasOwn(alias, "a");
frozenParenthesizedBracketedHasOwn(alias, "a");
Object["hasOwn"](alias, "a");
globalThis["Object"]["hasOwn"](alias, "a");
Object.freeze(globalThis["Object"]["hasOwn"])(alias, "a");
Object.freeze((globalThis["Object"])["hasOwn"])(alias, "a");
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
fn test_resolution_supports_object_from_entries_with_conditional_wrapper_in_js_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const conditionalEntries = (true ? [[\"b\", 1], [\"a\", 2]] : [[\"x\", 9]]); const fromEntries = Object.fromEntries(conditionalEntries);",
    )
    .unwrap();

    let lexer = kali_lexer::Lexer::new(
        kali_common::FileId::new(0),
        fs::read_to_string(&source_path).unwrap(),
    );
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
fn test_resolution_supports_object_from_entries_with_satisfies_wrapper_in_ts_input() {
    let dir = fixtures::tempdir();
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
                        computed_index: None,
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
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    let source = r#"const frozenEntries = Object.freeze([["b", 1], ["a", 2]]);
const fromEntries = Object.fromEntries(frozenEntries);
Object.hasOwn(fromEntries, "a");
Object.prototype.hasOwnProperty.call(fromEntries, "b");
Object["hasOwnProperty"]["call"](fromEntries, "b");
Object.hasOwn(Object.fromEntries(Object.freeze([["c", 3], ["d", 4]])), "c");
Object.prototype.hasOwnProperty.call(Object.fromEntries(Object.freeze([["e", 5], ["f", 6]])), "e");
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
fn test_resolution_supports_same_branch_conditional_string_keys_for_object_has_own_in_js_input() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                computed_index: None,
                object: Expression::Identifier("Object".to_string()),
                property: "hasOwn".to_string(),
            })),
            args: vec![
                Expression::ObjectExpression(ObjectExpression {
                    properties: vec![ObjectProperty {
                        key: PropertyName::Identifier("a".to_string()),
                        value: Expression::Literal(LiteralValue::Number(1.0)),
                        kind: ObjectPropertyKind::Init,
                    }],
                }),
                Expression::ConditionalExpression(Box::new(kali_ast::ConditionalExpression {
                    test: Box::new(Expression::Literal(LiteralValue::Boolean(true))),
                    consequent: Box::new(Expression::Literal(LiteralValue::String(
                        "a".to_string(),
                    ))),
                    alternate: Box::new(Expression::Literal(LiteralValue::String("a".to_string()))),
                })),
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
                        computed_index: None,
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
                        computed_index: None,
                        object: Expression::MemberExpression(Box::new(
                            kali_ast::MemberExpression {
                                computed_index: None,
                                object: Expression::MemberExpression(Box::new(
                                    kali_ast::MemberExpression {
                                        computed_index: None,
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
