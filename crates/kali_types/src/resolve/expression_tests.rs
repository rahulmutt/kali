use crate::test_support::*;
use crate::*;
use kali_ast::{
    AssignmentExpression, AssignmentOperator, BinaryExpression, BlockStatement, CallExpression,
    DecoratedExpression, ExportDefaultDeclaration, ExportNamedDeclaration, ExportSpecifier,
    Expression, ExpressionStatement, FunctionDeclaration, LiteralValue, LogicalExpression,
    LogicalOperator, MemberExpression, ObjectExpression, ObjectProperty, ObjectPropertyKind,
    ParenthesizedExpression, PropertyName, TemplateElement, TemplateLiteral, UnaryExpression,
    UpdateExpression, UpdateOperator, VariableDeclaration, VariableDeclarator,
};
use kali_error::_error_codes::{e3, e5};
use kali_test_support::fixtures;
use std::fs;

#[path = "expression_tests/exports.rs"]
mod exports;

#[path = "expression_tests/operators.rs"]
mod operators;

#[path = "expression_tests/dynamic_import.rs"]
mod dynamic_import;

#[test]
fn a_one_element_array_literal_of_an_allocation_is_refused() {
    // `[new Array(3)]` lowers to the same LIR node as `new Array(3)` itself
    // (`crates/kali_mir/src/lower.rs:108`, `:116` erase the HIR distinction), so
    // codegen reads the literal as the allocation and answers its length.
    let statements = vec![Statement::VariableDeclaration(VariableDeclaration {
        kind: "const".to_string(),
        declarations: vec![VariableDeclarator {
            id: "xs".to_string(),
            init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                elements: vec![Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::NewExpression(Box::new(kali_ast::NewExpression {
                        callee: Expression::CallExpression(Box::new(CallExpression {
                            callee: Expression::Identifier("Array".to_string()),
                            args: vec![Expression::Literal(LiteralValue::Number(3.0))],
                        })),
                        args: vec![],
                    })),
                ))],
            })),
        }],
    })];

    let result = assert_resolution!(statements, diagnostics: 1);
    assert!(
        result.diagnostics[0]
            .message
            .contains("lowers to the same node as the allocation itself"),
        "unexpected message: {}",
        result.diagnostics[0].message
    );
}

#[test]
fn a_two_element_array_literal_of_allocations_is_admitted() {
    // Only the ONE-element literal collides: a two-child text-less `Value` is not
    // a shape `resolve_array_alloc_call` accepts.
    let allocation = || {
        Expression::NewExpression(Box::new(kali_ast::NewExpression {
            callee: Expression::CallExpression(Box::new(CallExpression {
                callee: Expression::Identifier("Array".to_string()),
                args: vec![Expression::Literal(LiteralValue::Number(3.0))],
            })),
            args: vec![],
        }))
    };
    let statements = vec![Statement::VariableDeclaration(VariableDeclaration {
        kind: "const".to_string(),
        declarations: vec![VariableDeclarator {
            id: "xs".to_string(),
            init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                elements: vec![
                    Some(kali_ast::ExpressionOrSpread::Expression(allocation())),
                    Some(kali_ast::ExpressionOrSpread::Expression(allocation())),
                ],
            })),
        }],
    })];

    assert_resolution!(statements, diagnostics: 0);
}

#[test]
fn a_one_element_array_literal_of_a_non_allocation_is_admitted() {
    // The over-refusal direction: a one-element literal whose element is not
    // an array allocation at all must stay silent on this gate.
    let statements = vec![Statement::VariableDeclaration(VariableDeclaration {
        kind: "const".to_string(),
        declarations: vec![VariableDeclarator {
            id: "xs".to_string(),
            init: Some(Expression::ArrayExpression(kali_ast::ArrayExpression {
                elements: vec![Some(kali_ast::ExpressionOrSpread::Expression(
                    Expression::Literal(LiteralValue::Number(1.0)),
                ))],
            })),
        }],
    })];

    assert_resolution!(statements, diagnostics: 0);
}
