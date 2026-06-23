use crate::*;
use crate::test_support::*;
use kali_ast::{ArrayExpression, AssignmentExpression, AssignmentOperator, AwaitExpression, BlockStatement, CallExpression, DecoratedExpression, Expression, ExpressionOrSpread, ExpressionStatement, ForOfLefthand, ForOfStatement, LiteralValue, MemberExpression, ObjectExpression, ObjectProperty, ObjectPropertyKind, ParenthesizedExpression, PropertyName, SatisfiesExpression, TemplateElement, TemplateLiteral, TypeAliasDeclaration, UnaryExpression, VariableDeclaration, VariableDeclarator};
use kali_error::_error_codes::e5;
use std::fs;
use tempfile::tempdir;

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
    let dir = tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
fn test_resolution_supports_bracketed_and_frozen_object_has_own_aliases_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
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
    let dir = tempdir().unwrap();
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
fn test_resolution_supports_bracketed_object_is_and_number_predicate_alias_spelling_in_js_input() {
    let dir = tempdir().unwrap();
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
        let dir = tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
fn test_resolution_accepts_object_freeze_wrapped_dynamic_import_targets() {
    let dir = tempfile::tempdir().unwrap();
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
    let dir = tempfile::tempdir().unwrap();
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
        let dir = tempfile::tempdir().unwrap();
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
fn test_resolution_accepts_object_freeze_wrapped_set_constructor_targets_in_js_like_input() {
    let source = r#"async function main() {
    for (const value of Object.freeze(new Set([1, 2, 1]))) {
        console.log(value);
    }
}
main();
"#;

    for extension in ["js", "jsx", "ts", "tsx"] {
        let dir = tempdir().unwrap();
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
        let dir = tempdir().unwrap();
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
        let dir = tempdir().unwrap();
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
        let dir = tempdir().unwrap();
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
        let dir = tempdir().unwrap();
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
        let dir = tempdir().unwrap();
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
        let dir = tempdir().unwrap();
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
fn test_resolution_supports_single_quoted_bracket_root_object_enumeration_aliases_in_js_input() {
    let dir = tempfile::tempdir().unwrap();
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
