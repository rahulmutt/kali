use crate::*;
use kali_test_support::fixtures;
use kali_ast::{
    BlockStatement, ClassBody, ClassDeclaration, ClassExpression, ExportDefaultDeclaration,
    Expression, ExpressionStatement, FunctionDeclaration, FunctionExpression, LiteralValue,
    MethodDefinition, VariableDeclaration, VariableDeclarator, YieldExpression,
};
use kali_error::_error_codes::e5;
use std::fs;

#[path = "function_tests/generator_functions.rs"]
mod generator_functions;

#[path = "function_tests/class_methods.rs"]
mod class_methods;
