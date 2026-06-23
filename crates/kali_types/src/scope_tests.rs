use crate::*;
use kali_ast::{BlockStatement, VariableDeclaration, VariableDeclarator};
use kali_error::_error_codes::e3;

#[test]
fn test_scope_creation() {
    let scope = Scope::new(ScopeType::Global, None);
    assert_eq!(scope.scope_type, ScopeType::Global);
    assert!(scope.parent.is_none());
}

#[test]
fn test_scope_binding() {
    let mut scope = Scope::new(ScopeType::Module, None);
    scope.bind("x", NodeId::new(1));
    scope.bind("y", NodeId::new(2));

    assert!(scope.contains("x"));
    assert!(scope.contains("y"));
    assert!(!scope.contains("z"));
}

#[test]
fn test_resolution_reports_duplicate_bindings() {
    let mut ctx = TypeContext::new();
    let statements = vec![Statement::BlockStatement(BlockStatement {
        body: vec![
            Statement::VariableDeclaration(VariableDeclaration {
                kind: "let".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "x".to_string(),
                    init: None,
                }],
            }),
            Statement::VariableDeclaration(VariableDeclaration {
                kind: "let".to_string(),
                declarations: vec![VariableDeclarator {
                    id: "x".to_string(),
                    init: None,
                }],
            }),
        ],
    })];

    let result = ctx.resolve_statements(&statements);
    assert!(result
        .diagnostics
        .iter()
        .any(|diag| diag.code == Some(e3::DUPLICATE_BINDING as u32)));
}
