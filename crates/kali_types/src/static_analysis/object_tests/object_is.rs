use super::*;

#[test]
fn test_resolution_supports_bracketed_object_is_and_number_predicate_alias_spelling_in_js_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    let source = r#"const object = { a: 1 };
const objectAlias = object;
const numeric = 1;
const numericAlias = numeric;
const safeInteger = Number.isSafeInteger;
const globalFinite = isFinite;
const globalNaN = Object.freeze(globalThis["isNaN"]);
isFinite(numericAlias);
globalThis.isFinite(numericAlias);
globalThis["isFinite"](numericAlias);
globalFinite(numericAlias);
globalThis.isNaN(NaN);
globalThis["isNaN"](NaN);
globalNaN(NaN);
globalThis["Object"]["is"](objectAlias, object);
globalThis.Object["is"](object, object);
globalThis["Object"].is(objectAlias, object);
Object["is"](objectAlias, object);
globalThis.Number["isFinite"](numericAlias);
globalThis["Number"]["isFinite"](numericAlias);
safeInteger(numericAlias);
globalThis["Number"].isInteger(numericAlias);
globalThis["Number"]["isInteger"](numericAlias);
globalThis.Number["isInteger"](numericAlias);
globalThis["Number"].isSafeInteger(numericAlias);
globalThis.Number["isSafeInteger"](numericAlias);
globalThis["Number"]["isSafeInteger"](numericAlias);
globalThis.Number["isNaN"](NaN);
globalThis["Number"]["isNaN"](NaN);
Number.isSafeInteger(numericAlias);
Number["isFinite"](numericAlias);
const frozenFinite = Object.freeze(Number.isFinite);
const frozenNaN = Object.freeze(Number.isNaN);
const frozenInteger = Object.freeze(Number.isInteger);
const frozenSafeInteger = Object.freeze(Number.isSafeInteger);
frozenFinite(numericAlias);
frozenNaN(NaN);
frozenInteger(numericAlias);
frozenSafeInteger(numericAlias);
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
fn test_resolution_supports_object_is_through_static_member_roots() {
    let source = r#"Object.is(globalThis.Object, globalThis.Object);
Object.is(globalThis["Object"], globalThis["Object"]);
Object.is(globalThis['Object'], globalThis['Object']);
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
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "is".to_string(),
                })),
                args: vec![
                    Expression::TemplateLiteral(TemplateLiteral {
                        quasis: vec![TemplateElement {
                            value: "hello".to_string(),
                            tail: true,
                        }],
                        expressions: vec![],
                    }),
                    Expression::TemplateLiteral(TemplateLiteral {
                        quasis: vec![TemplateElement {
                            value: "hello".to_string(),
                            tail: true,
                        }],
                        expressions: vec![],
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
fn test_resolution_accepts_object_is_signed_zero_literal_pairs() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::ExpressionStatement(ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::MemberExpression(Box::new(MemberExpression {
                    object: Expression::Identifier("Object".to_string()),
                    property: "is".to_string(),
                })),
                args: vec![
                    Expression::UnaryExpression(Box::new(UnaryExpression {
                        operator: "-".to_string(),
                        argument: Expression::Literal(LiteralValue::Number(0.0)),
                    })),
                    Expression::UnaryExpression(Box::new(UnaryExpression {
                        operator: "+".to_string(),
                        argument: Expression::Literal(LiteralValue::Number(0.0)),
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
                        operator: "-".to_string(),
                        argument: Expression::Literal(LiteralValue::Number(0.0)),
                    })),
                    Expression::UnaryExpression(Box::new(UnaryExpression {
                        operator: "-".to_string(),
                        argument: Expression::Literal(LiteralValue::Number(0.0)),
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
fn test_resolution_accepts_object_is_with_optional_chain_wrapped_static_reference() {
    let mut ctx = TypeContext::new();
    let statements = vec![
        Statement::VariableDeclaration(VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![VariableDeclarator {
                id: "object".to_string(),
                init: Some(Expression::OptionalChainExpression(Box::new(
                    OptionalChainExpression {
                        inner: Box::new(OptionalChainInner::NonNull {
                            object: Box::new(Expression::MemberExpression(Box::new(
                                MemberExpression {
                                    object: Expression::Identifier("globalThis".to_string()),
                                    property: "Object".to_string(),
                                },
                            ))),
                            optional: true,
                        }),
                    },
                ))),
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
    ];

    let result = ctx.resolve_statements(&statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_resolution_accepts_object_is_with_optional_chain_wrapped_same_reference() {
    let mut ctx = TypeContext::new();
    let object_root = Expression::MemberExpression(Box::new(MemberExpression {
        object: Expression::Identifier("globalThis".to_string()),
        property: "Object".to_string(),
    }));
    let optional_chain_root =
        Expression::OptionalChainExpression(Box::new(OptionalChainExpression {
            inner: Box::new(OptionalChainInner::NonNull {
                object: Box::new(object_root.clone()),
                optional: true,
            }),
        }));
    let statements = vec![Statement::ExpressionStatement(ExpressionStatement {
        expression: Box::new(Expression::CallExpression(Box::new(CallExpression {
            callee: Expression::MemberExpression(Box::new(MemberExpression {
                object: Expression::Identifier("Object".to_string()),
                property: "is".to_string(),
            })),
            args: vec![optional_chain_root, object_root],
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
