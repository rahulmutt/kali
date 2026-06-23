use crate::test_support::*;
use crate::*;
use kali_ast::{
    ArrowFunctionExpression, BinaryExpression, CallExpression, Expression, ExpressionStatement,
    LiteralValue, MemberExpression, SatisfiesExpression, TemplateElement, TemplateLiteral,
    TypeAssertion, VariableDeclaration, VariableDeclarator,
};

#[test]
fn test_resolution_accepts_wrapped_call_targets_with_type_assertions_and_satisfies() {
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

    assert_resolution!(statements, diagnostics: 0);
}

#[test]
fn test_resolution_accepts_wrapped_local_function_call_targets() {
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

    assert_resolution!(statements, diagnostics: 0);
}
