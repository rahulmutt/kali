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

/// Spec 4a Task 2: the `for..in` key-provenance registry. (1) `for (var c in
/// table)` tags `c` with `table`'s object shape; (2) `last = c` propagates
/// the same shape to `last` (the bare-identifier-alias path in
/// `resolve/expression.rs`'s `AssignmentExpression` arm); (3) `d`, never
/// assigned from a key, stays unregistered. Dormant registry: nothing reads
/// it yet, so this test inspects `Scope::for_in_key_bindings` directly via
/// the `ResolutionResult::scopes`/`global_scope` snapshot rather than calling
/// `TypeContext::for_in_key_shape` post-hoc — that method's scope-walk keys
/// off `current_scope_id()`, which is only meaningful mid-traversal (it is
/// exercised for real, live, by the `last = c` propagation call itself
/// during resolution).
#[test]
fn for_in_key_provenance_registers_and_propagates_through_alias() {
    let statements = crate::test_support::parse_statements(
        "function m() {\n\
             const table = { a: 1, b: 2 };\n\
             table.a = table.a + 1;\n\
             let last = 0;\n\
             let d = 0;\n\
             for (var c in table) {\n\
                 last = c;\n\
             }\n\
         }\n\
         m();\n",
    );

    let mut ctx = TypeContext::new();
    let result = ctx.resolve_statements(&statements);

    let find_shape = |name: &str| {
        result
            .scopes
            .values()
            .find_map(|scope| scope.for_in_key_bindings.get(name).copied())
            .or_else(|| result.global_scope.for_in_key_bindings.get(name).copied())
    };

    let c_shape = find_shape("c");
    assert!(
        c_shape.is_some(),
        "expected `c` to be registered as a for..in key binding over `table`'s shape"
    );
    assert_eq!(
        find_shape("last"),
        c_shape,
        "expected `last = c` to propagate the same shape to `last`"
    );
    assert_eq!(
        find_shape("d"),
        None,
        "expected `d` (never assigned from a key) to stay unregistered"
    );
}
