use super::*;

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
fn test_resolution_supports_for_of_object_entries_iteration() {
    let dir = fixtures::tempdir();
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

#[test]
fn test_resolution_accepts_frozen_object_enumeration_callable_aliases_in_js_like_input() {
    let source = r#"async function main() {
    const obj = Object.fromEntries([["b", 1], ["a", 2]]);
    const frozenKeys = Object.freeze(Object["keys"])(obj);
    const frozenValues = Object.freeze(globalThis.Object["values"])(obj);
    const mixedBracketedKeys = Object.freeze(globalThis["Object"]['keys'])(obj);
    const mixedSingleQuotedKeys = Object.freeze(globalThis['Object']["keys"])(obj);
    const mixedBracketedValues = Object.freeze(globalThis["Object"]['values'])(obj);
    const mixedSingleQuotedValues = Object.freeze(globalThis['Object']["values"])(obj);
    const mixedBracketedEntries = Object.freeze(globalThis["Object"]['entries'])(obj);
    const mixedSingleQuotedEntries = Object.freeze(globalThis['Object']["entries"])(obj);
    const mixedBracketedOwnKeys = Object.freeze(globalThis["Reflect"]['ownKeys'])(obj);
    const mixedSingleQuotedOwnKeys = Object.freeze(globalThis['Reflect']["ownKeys"])(obj);
    const frozenBracketedKeys = Object.freeze(globalThis["Object"]["keys"])(obj);
    const parenthesizedSingleQuotedReceiverBracketedKeys = Object.freeze((globalThis['Object'])['keys'])(obj);
    const frozenBracketedValues = Object.freeze(globalThis["Object"]["values"])(obj);
    const frozenEntries = Object.freeze(globalThis["Object"]["entries"])(obj);
    const parenthesizedBracketedKeys = Object.freeze((globalThis["Object"]).keys)(obj);
    const parenthesizedBracketedValues = Object.freeze((globalThis["Object"]).values)(obj);
    const parenthesizedBracketedEntries = Object.freeze((globalThis["Object"]).entries)(obj);
    const parenthesizedSingleQuotedBracketedValues = Object.freeze((globalThis['Object'])["values"])(obj);
    const parenthesizedSingleQuotedBracketedEntries = Object.freeze((globalThis['Object'])["entries"])(obj);
    const parenthesizedSingleQuotedBracketedKeys = Object.freeze((globalThis['Object'])['keys'])(obj);
    const frozenLogicalAndCallableKeys = Object.freeze((true && Object.keys))(obj);
    const frozenLogicalOrCallableKeys = Object.freeze((false || Object.keys))(obj);
    const frozenOwnKeys = Object.freeze(Reflect.ownKeys)(obj);
    const frozenParenOwnKeys = Object.freeze((Reflect.ownKeys))(obj);
    const frozenBracketedRootOwnKeys = Object.freeze((globalThis["Reflect"]).ownKeys)(obj);
    const frozenDoubleQuotedBracketedRootOwnKeys = Object.freeze((globalThis["Reflect"])["ownKeys"])(obj);
    const frozenMixedDotRootOwnKeys = Object.freeze(globalThis.Reflect["ownKeys"])(obj);
    const frozenParenthesizedMixedDotRootOwnKeys = Object.freeze((globalThis.Reflect["ownKeys"]))(obj);
    const frozenSingleQuotedDotRootOwnKeys = Object.freeze(globalThis.Reflect['ownKeys'])(obj);
    const frozenParenthesizedSingleQuotedDotRootOwnKeys = Object.freeze((globalThis.Reflect['ownKeys']))(obj);
    const frozenSingleQuotedBracketedRootOwnKeys = Object.freeze((globalThis['Reflect'])['ownKeys'])(obj);
    const frozenNullishCallableKeys = Object.freeze((null ?? Object.keys))(obj);
    const frozenLogicalAndCallableValues = Object.freeze((true && Object.values))(obj);
    const frozenLogicalOrCallableEntries = Object.freeze((false || Object.entries))(obj);
    const frozenNullishCallableOwnKeys = Object.freeze((null ?? Reflect.ownKeys))(obj);
    const frozenLogicalAndCallableOwnKeys = Object.freeze((true && Reflect.ownKeys))(obj);
    const frozenLogicalOrCallableOwnKeys = Object.freeze((false || Reflect.ownKeys))(obj);
    for (const key of mixedBracketedKeys) {
        console.log(key);
    }
    for (const key of mixedSingleQuotedKeys) {
        console.log(key);
    }
    for (const value of mixedBracketedValues) {
        console.log(value);
    }
    for (const value of mixedSingleQuotedValues) {
        console.log(value);
    }
    for (const entry of mixedBracketedEntries) {
        console.log(entry[0], entry[1]);
    }
    for (const entry of mixedSingleQuotedEntries) {
        console.log(entry[0], entry[1]);
    }
    for (const key of mixedBracketedOwnKeys) {
        console.log(key);
    }
    for (const key of mixedSingleQuotedOwnKeys) {
        console.log(key);
    }
    for (const key of frozenKeys) {
        console.log(key);
    }
    for (const key of parenthesizedSingleQuotedBracketedKeys) {
        console.log(key);
    }
    for (const key of frozenLogicalAndCallableKeys) {
        console.log(key);
    }
    for (const key of frozenLogicalOrCallableKeys) {
        console.log(key);
    }
    for (const key of parenthesizedSingleQuotedReceiverBracketedKeys) {
        console.log(key);
    }
    for (const value of frozenValues) {
        console.log(value);
    }
    for (const entry of frozenEntries) {
        console.log(entry[0], entry[1]);
    }
    for (const key of parenthesizedBracketedKeys) {
        console.log(key);
    }
    for (const value of parenthesizedBracketedValues) {
        console.log(value);
    }
    for (const entry of parenthesizedBracketedEntries) {
        console.log(entry[0], entry[1]);
    }
    for (const value of parenthesizedSingleQuotedBracketedValues) {
        console.log(value);
    }
    for (const entry of parenthesizedSingleQuotedBracketedEntries) {
        console.log(entry[0], entry[1]);
    }
    for (const key of frozenOwnKeys) {
        console.log(key);
    }
    for (const key of frozenParenOwnKeys) {
        console.log(key);
    }
    for (const key of frozenBracketedRootOwnKeys) {
        console.log(key);
    }
    for (const key of frozenMixedDotRootOwnKeys) {
        console.log(key);
    }
    for (const key of frozenParenthesizedMixedDotRootOwnKeys) {
        console.log(key);
    }
    for (const key of frozenSingleQuotedDotRootOwnKeys) {
        console.log(key);
    }
    for (const key of frozenParenthesizedSingleQuotedDotRootOwnKeys) {
        console.log(key);
    }
    for (const key of frozenSingleQuotedBracketedRootOwnKeys) {
        console.log(key);
    }
    for (const key of frozenNullishCallableKeys) {
        console.log(key);
    }
    for (const value of frozenLogicalAndCallableValues) {
        console.log(value);
    }
    for (const entry of frozenLogicalOrCallableEntries) {
        console.log(entry[0], entry[1]);
    }
    for (const key of frozenNullishCallableOwnKeys) {
        console.log(key);
    }
    for (const key of frozenLogicalAndCallableOwnKeys) {
        console.log(key);
    }
    for (const key of frozenLogicalOrCallableOwnKeys) {
        console.log(key);
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
fn test_resolution_supports_object_keys_iteration_with_let_binding_in_js_input() {
    let dir = fixtures::tempdir();
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
    let dir = fixtures::tempdir();
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
fn test_resolution_supports_single_quoted_bracket_root_object_enumeration_aliases_in_js_input() {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"let keys = { a: 1 }; let values = { b: 2 }; let entries = { c: 3 };
for (const key of globalThis['Object']['keys'](keys)) { console.log(key); }
for (const value of globalThis['Object']['values'](values)) { console.log(value); }
for (const entry of globalThis['Object']['entries'](entries)) { console.log(entry); }"#,
    )
    .unwrap();

    let lexer = kali_lexer::Lexer::new(
        kali_common::FileId::new(0),
        r#"let keys = { a: 1 }; let values = { b: 2 }; let entries = { c: 3 };
for (const key of globalThis['Object']['keys'](keys)) { console.log(key); }
for (const value of globalThis['Object']['values'](values)) { console.log(value); }
for (const entry of globalThis['Object']['entries'](entries)) { console.log(entry); }"#
            .to_string(),
    );
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
