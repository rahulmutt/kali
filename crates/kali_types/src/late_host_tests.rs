use crate::test_support::*;
use kali_test_support::fixtures;
use crate::*;
use kali_ast::{
    AssignmentExpression, AssignmentOperator, AwaitExpression, CallExpression, DecoratedExpression,
    Expression, ExpressionStatement, LiteralValue, MemberExpression, ObjectExpression,
    ObjectProperty, ObjectPropertyKind, ParenthesizedExpression, PropertyName, SatisfiesExpression,
    UnaryExpression, VariableDeclaration, VariableDeclarator,
};
use kali_common::process_kill_zero_probe_source;
use kali_error::_error_codes::{e3, e5};
use std::fs;

#[path = "late_host_tests/globals.rs"]
mod globals;

#[path = "late_host_tests/process_env.rs"]
mod process_env;

#[path = "late_host_tests/permissions.rs"]
mod permissions;

#[path = "late_host_tests/intl_imports_kill.rs"]
mod intl_imports_kill;
