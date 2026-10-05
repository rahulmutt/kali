use super::*;
use crate::build::block_scope_rename::test_support::table_of;
use crate::build::block_scope_rename::walk::{BindKind, ScopeKind};

fn names(table: &ScopeTable, id: ScopeId) -> Vec<&str> {
    table.scopes[id]
        .bindings
        .keys()
        .map(String::as_str)
        .collect()
}

#[test]
fn module_block_and_function_scopes_are_numbered_in_entry_order() {
    let t = table_of("let x=1; { let x=2; } function f(a){ { const b=1; } }");
    assert_eq!(t.scopes[0].kind, ScopeKind::Module);
    assert_eq!(names(&t, 0), vec!["f", "x"]);
    assert_eq!(t.scopes[1].kind, ScopeKind::Block);
    assert_eq!(names(&t, 1), vec!["x"]);
    assert_eq!(t.scopes[2].kind, ScopeKind::Function);
    assert_eq!(names(&t, 2), vec!["a"]);
    assert_eq!(t.scopes[3].kind, ScopeKind::Block);
    assert_eq!(t.scopes[3].depth, 1);
    assert_eq!(t.scopes[3].frame, 2);
    assert_eq!(t.scopes[3].frame_level, 1);
}

#[test]
fn var_hoists_to_the_frame_and_let_stays_in_the_block() {
    let t = table_of("function f(){ { var v=1; let w=2; } }");
    assert_eq!(names(&t, 1), vec!["v"]);
    assert_eq!(names(&t, 2), vec!["w"]);
}

#[test]
fn a_for_head_is_its_own_block_and_the_body_nests_in_it() {
    let t = table_of("for(let i=0;i<2;i++){ const v=i; }");
    assert_eq!(names(&t, 1), vec!["i"]);
    assert_eq!(t.scopes[2].parent, Some(1));
    assert_eq!(names(&t, 2), vec!["v"]);
}

#[test]
fn a_function_declaration_binds_in_its_block_and_its_params_inside() {
    let t = table_of("{ function h(p){ return p; } }");
    assert_eq!(names(&t, 1), vec!["h"]);
    assert_eq!(t.scopes[1].bindings["h"].kind, BindKind::FunctionDecl);
    assert_eq!(names(&t, 2), vec!["p"]);
}

#[test]
fn a_named_function_expression_binds_its_id_inside_itself() {
    let t = table_of("const f = function g(){ return 1; };");
    assert_eq!(names(&t, 0), vec!["f"]);
    assert_eq!(t.scopes[1].bindings["g"].kind, BindKind::FunctionExprId);
}

#[test]
fn imports_bind_in_the_module_scope() {
    let t = table_of(r#"import { a as b } from "./m.js"; import * as ns from "./n.js";"#);
    assert_eq!(names(&t, 0), vec!["b", "ns"]);
}

#[test]
fn a_same_scope_redeclaration_keeps_the_first_binding() {
    let t = table_of("var v=1; var v=2;");
    assert_eq!(t.scopes[0].bindings["v"].ordinal, 0);
}

#[test]
fn resolve_walks_parents_and_target_scope_hoists_var() {
    let t = table_of("let x=1; { let y=2; }");
    assert_eq!(t.resolve(1, "x"), Some(0));
    assert_eq!(t.resolve(1, "y"), Some(1));
    assert_eq!(t.resolve(1, "z"), None);
    assert_eq!(t.target_scope(1, BindKind::Var), 0);
    assert_eq!(t.target_scope(1, BindKind::Lexical), 1);
}
