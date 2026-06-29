use crate::test_support::*;
use kali_test_support::fixtures;
use crate::*;
use kali_ast::{
    AssignmentExpression, AssignmentOperator, BlockStatement, CallExpression, DecoratedExpression,
    Expression, ExpressionStatement, ForOfLefthand, ForOfStatement, LiteralValue, MemberExpression,
    VariableDeclaration, VariableDeclarator,
};
use kali_error::_error_codes::e5;
use std::fs;

fn assert_resolution_accepts_frozen_iterator_protocol_edge(source_filename: &str, source: &str) {
    let dir = fixtures::tempdir();
    let source_path = dir.path().join(source_filename);
    fs::write(&source_path, source).unwrap();

    let lexer = kali_lexer::Lexer::new(kali_common::FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = kali_parser::Parser::new(kali_common::FileId::new(0), tokens);
    let statements = parser.parse(None).statements;

    let mut ctx = TypeContext::with_base_path(&source_path);
    let result = ctx.resolve_statements_at_path(Some(&source_path), &statements);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[path = "array_tests/set_map_targets.rs"]
mod set_map_targets;

#[path = "array_tests/array_from.rs"]
mod array_from;

#[path = "array_tests/for_of.rs"]
mod for_of;

#[path = "array_tests/for_await.rs"]
mod for_await;

#[path = "array_tests/methods.rs"]
mod methods;
