use crate::test_support::*;
use crate::*;
use kali_ast::{
    AssignmentExpression, AssignmentOperator, BinaryExpression, BlockStatement,
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
