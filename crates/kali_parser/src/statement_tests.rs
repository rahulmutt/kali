use crate::test_support::lex;
use crate::*;
use kali_ast::{Expression, ForOfLefthand, Statement};

#[test]
fn test_parse_var_declaration() {
    let tokens = lex("var x = 1;");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::VariableDeclaration(vd) => {
            assert_eq!(vd.kind, "var");
            assert_eq!(vd.declarations.len(), 1);
        }
        _ => panic!("Expected VariableDeclaration"),
    }
}

#[test]
fn test_parse_for_of_statement() {
    let tokens = lex("for (const value of items) { console.log(value); }");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::ForOfStatement(stmt) => {
            match &stmt.left {
                ForOfLefthand::VariableDeclaration(decl) => {
                    assert_eq!(decl.kind, "const");
                    assert_eq!(decl.declarations.len(), 1);
                    assert_eq!(decl.declarations[0].id, "value");
                    assert!(decl.declarations[0].init.is_none());
                }
                other => panic!("Expected variable declaration left-hand, got {other:?}"),
            }
            match &stmt.right {
                Expression::Identifier(name) => assert_eq!(name, "items"),
                other => panic!("Expected identifier right-hand, got {other:?}"),
            }
        }
        other => panic!("Expected ForOfStatement, got {other:?}"),
    }
}

#[test]
fn test_parse_for_await_of_statement() {
    let tokens = lex("async function main() { for await (const item of items) { item; } }");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    let Statement::FunctionDeclaration(decl) = &output.statements[0] else {
        panic!(
            "Expected FunctionDeclaration, got {:?}",
            output.statements[0]
        );
    };
    assert!(decl.is_async, "expected async flag to be preserved");
    assert_eq!(decl.body.body.len(), 1);

    let Statement::ForOfStatement(stmt) = &decl.body.body[0] else {
        panic!("Expected ForOfStatement, got {:?}", decl.body.body[0]);
    };
    assert!(stmt.is_await, "expected for-await-of flag to be preserved");
    match &stmt.left {
        ForOfLefthand::VariableDeclaration(decl) => {
            assert_eq!(decl.kind, "const");
            assert_eq!(decl.declarations[0].id, "item");
        }
        other => panic!("Expected variable declaration left-hand, got {other:?}"),
    }
}

#[test]
fn test_parse_for_await_of_statement_accepts_await_wrapped_literal_arrays() {
    let tokens = lex("for await (const item of await [1, 2]) { item; }");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    let Statement::ForOfStatement(stmt) = &output.statements[0] else {
        panic!("Expected ForOfStatement, got {:?}", output.statements[0]);
    };
    assert!(stmt.is_await, "expected for-await-of flag to be preserved");
    match &stmt.right {
        Expression::ArrayExpression(array) => {
            assert_eq!(array.elements.len(), 2);
        }
        other => panic!("Expected ArrayExpression right-hand, got {other:?}"),
    }
}

#[test]
fn test_parse_try_finally_statement_rejects_fail_closed() {
    // kali has no exception machinery. try/catch/finally is rejected
    // fail-closed (E5506 FEATURE_UNAVAILABLE) rather than miscompiled to a
    // bogus if-shaped branch. The tokens are still consumed so the parse
    // recovers to a single (rejected) statement without a secondary cascade.
    let tokens = lex("try { value; } finally { other; }");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output
            .diagnostics
            .iter()
            .any(|d| d.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected FEATURE_UNAVAILABLE reject, got: {:?}",
        output.diagnostics
    );
}
