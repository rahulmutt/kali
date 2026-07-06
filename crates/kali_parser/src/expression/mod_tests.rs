use crate::test_support::lex;
use crate::*;
use kali_ast::{AssignmentOperator, Expression, Statement, UpdateOperator};

#[path = "mod_tests/unary.rs"]
mod unary;

#[path = "mod_tests/binary.rs"]
mod binary;

#[path = "mod_tests/type_ops.rs"]
mod type_ops;

#[path = "mod_tests/conditional.rs"]
mod conditional;
