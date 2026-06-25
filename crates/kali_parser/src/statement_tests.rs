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
fn test_parse_try_finally_statement() {
    let tokens = lex("try { value; } finally { other; }");
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);

    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_eq!(output.statements.len(), 1);

    match &output.statements[0] {
        Statement::TryStatement(stmt) => {
            assert!(
                stmt.handler.is_none(),
                "unexpected catch clause: {:?}",
                stmt.handler
            );
            assert!(stmt.finalizer.is_some(), "expected finally block");
            assert_eq!(stmt.block.body.len(), 1);
            assert_eq!(stmt.finalizer.as_ref().unwrap().body.len(), 1);
        }
        other => panic!("Expected TryStatement, got {other:?}"),
    }
}
