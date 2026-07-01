use crate::*;
use kali_ast::{
    BinaryExpression, BlockStatement, CallExpression, Expression, ExpressionStatement,
    ForOfLefthand, ForOfStatement, LiteralValue, MemberExpression, VariableDeclaration,
    VariableDeclarator,
};
use kali_error::_error_codes::e5;
use kali_test_support::fixtures;
use std::fs;

#[path = "string_tests/iteration.rs"]
mod iteration;

#[path = "string_tests/methods.rs"]
mod methods;
