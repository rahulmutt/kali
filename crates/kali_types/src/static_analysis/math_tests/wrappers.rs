use super::*;

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
fn test_resolution_supports_frozen_math_expm1_and_log1p_identity_helpers_across_js_like_extensions()
{
    let source = r#"const zero = 0;
const frozenDotRoot = Object.freeze(globalThis.Math);
const frozenMixedRoot = Object.freeze(globalThis[\"Math\"]);
const frozenDirectRoot = Object.freeze(Math);
frozenDotRoot.expm1(zero);
frozenMixedRoot.expm1(zero);
frozenDirectRoot.expm1(zero);
frozenDotRoot.log1p(zero);
frozenMixedRoot.log1p(zero);
frozenDirectRoot.log1p(zero);
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
fn test_resolution_supports_frozen_math_pow_callable_aliases_across_js_like_extensions() {
    let source = r#"const exponent = 3;
const frozenDotRoot = Object.freeze(Math.pow);
const frozenGlobalDotRoot = Object.freeze(globalThis.Math.pow);
const frozenBracketedRoot = Object.freeze(globalThis["Math"]["pow"]);
const frozenSingleQuotedBracketedRoot = Object.freeze(globalThis['Math']['pow']);
const frozenSingleQuotedMathRoot = Object.freeze(Math['pow']);
const frozenParenthesizedDotRoot = Object.freeze((Math.pow));
const frozenParenthesizedGlobalThisDotRoot = Object.freeze((globalThis.Math))["pow"];
const frozenParenthesizedSingleQuotedGlobalThisDotRoot = Object.freeze((globalThis.Math))['pow'];
const frozenParenthesizedGlobalThisBracketedRoot = Object.freeze((globalThis["Math"]))["pow"];
const frozenParenthesizedSingleQuotedGlobalThisBracketedRoot = Object.freeze((globalThis['Math']))['pow'];
frozenDotRoot(2, exponent);
frozenGlobalDotRoot(2, exponent);
frozenBracketedRoot(2, exponent);
frozenSingleQuotedBracketedRoot(2, exponent);
frozenSingleQuotedMathRoot(2, exponent);
frozenParenthesizedDotRoot(2, exponent);
frozenParenthesizedGlobalThisDotRoot(2, exponent);
frozenParenthesizedSingleQuotedGlobalThisDotRoot(2, exponent);
frozenParenthesizedGlobalThisBracketedRoot(2, exponent);
frozenParenthesizedSingleQuotedGlobalThisBracketedRoot(2, exponent);
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
fn test_resolution_supports_frozen_math_abs_and_sign_callable_aliases_across_js_like_extensions() {
    let source = format!(
        "const alias = 1;\n{}",
        math_abs_sign_frozen_callable_invocation_source()
    );

    for extension in ["js", "jsx", "ts", "tsx"] {
        let dir = fixtures::tempdir();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, &source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.clone());
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
fn test_resolution_supports_frozen_math_round_callable_aliases_across_js_like_extensions() {
    let source = format!(
        "const value = 1.6;\n{}",
        math_round_frozen_callable_invocation_source()
    );

    for extension in ["js", "jsx", "ts", "tsx"] {
        let dir = fixtures::tempdir();
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, &source).unwrap();

        let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.clone());
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
