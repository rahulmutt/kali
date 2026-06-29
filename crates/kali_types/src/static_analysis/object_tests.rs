use crate::test_support::*;
use kali_test_support::fixtures;
use crate::*;
use kali_ast::{
    ArrayExpression, AssignmentExpression, AssignmentOperator, AwaitExpression, BlockStatement,
    CallExpression, DecoratedExpression, Expression, ExpressionOrSpread, ExpressionStatement,
    ForOfLefthand, ForOfStatement, LiteralValue, MemberExpression, ObjectExpression,
    ObjectProperty, ObjectPropertyKind, ParenthesizedExpression, PropertyName, SatisfiesExpression,
    TemplateElement, TemplateLiteral, TypeAliasDeclaration, UnaryExpression, VariableDeclaration,
    VariableDeclarator,
};
use kali_error::_error_codes::e5;
use std::fs;

fn assert_object_helper_iteration_with_let_binding_in_js_input(helper: &str, rebound: bool) {
    let dir = fixtures::tempdir();
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

#[path = "object_tests/object_is.rs"]
mod object_is;

#[path = "object_tests/has_own_entries.rs"]
mod has_own_entries;

#[path = "object_tests/enumeration.rs"]
mod enumeration;

#[path = "object_tests/freeze_late_model.rs"]
mod freeze_late_model;
