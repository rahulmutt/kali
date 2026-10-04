use super::*;
use crate::build::block_scope_rename::test_support::table_of;

fn renamed(source: &str) -> Vec<(ScopeId, String, String)> {
    plan_renames(&table_of(source))
        .into_iter()
        .map(|((s, n), new)| (s, n, new))
        .collect()
}

#[test]
fn nothing_shadows_nothing_is_renamed() {
    assert!(renamed("function f(a){ let b=a+1; { let c=b*2; return c; } }").is_empty());
    assert!(renamed("function a(){ let i=0; } function b(){ let i=1; }").is_empty());
}

#[test]
fn a_block_shadow_of_a_module_binding_is_renamed() {
    assert_eq!(
        renamed("let x=1; { let x=2; }"),
        vec![(1, "x".into(), "x{b0}".into())]
    );
}

#[test]
fn a_block_shadow_of_a_parameter_is_renamed() {
    assert_eq!(
        renamed("function f(c){ { const c=7; } }"),
        vec![(2, "c".into(), "c{b0}".into())]
    );
}

#[test]
fn among_siblings_the_first_keeps_its_name() {
    assert_eq!(
        renamed("function f(){ { let a=1; } { let a=\"s\"; } }"),
        vec![(3, "a".into(), "a{b0}".into())]
    );
}

#[test]
fn a_shallower_sibling_branch_keeps_its_name() {
    assert_eq!(
        renamed("function f(){ { { let a=1; } } { let a=2; } }"),
        vec![(3, "a".into(), "a{b0}".into())]
    );
}

#[test]
fn var_in_a_block_keeps_its_name_against_a_sibling_let() {
    assert_eq!(
        renamed("function f(){ { var v=1; } { let v=2; } return v; }"),
        vec![(3, "v".into(), "v{b0}".into())]
    );
}

#[test]
fn a_nested_function_parameter_sharing_a_module_name_is_renamed() {
    assert_eq!(
        renamed("const n=1; function f(n){ return n; }"),
        vec![(1, "n".into(), "n{b0}".into())]
    );
}

#[test]
fn same_named_nested_functions_are_renamed_program_wide() {
    let r = renamed(
        "function a(){ function h(){ return 1; } return h(); } function b(){ function h(){ return 2; } return h(); }",
    );
    assert_eq!(r, vec![(3, "h".into(), "h{b0}".into())]);
}

#[test]
fn a_top_level_function_keeps_its_name_against_a_nested_one() {
    let r = renamed("function g(){ function h(){} } function h(){}");
    assert_eq!(r, vec![(1, "h".into(), "h{b0}".into())]);
}

#[test]
fn a_same_scope_redeclaration_is_left_for_the_resolver() {
    assert!(renamed("function f(){ let a=1; let a=2; }").is_empty());
}

#[test]
fn numbering_follows_declaration_order() {
    let r = renamed("let x=1; let y=1; { let y=2; } { let x=2; }");
    assert_eq!(
        r,
        vec![
            (1, "y".into(), "y{b0}".into()),
            (2, "x".into(), "x{b1}".into())
        ]
    );
}
