use crate::test_support::lex;
use crate::*;
use kali_ast::{Expression, Statement};

#[path = "call_tests/member.rs"]
mod member;

#[path = "call_tests/optional_chain.rs"]
mod optional_chain;

#[path = "call_tests/dynamic_import.rs"]
mod dynamic_import;
