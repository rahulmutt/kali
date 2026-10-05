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

/// The original quadratic statement of the two rules (review I-3): every
/// binding scans every other binding. `plan_renames` must agree with it.
fn reference_plan(table: &ScopeTable) -> RenamePlan {
    let all: Vec<(ScopeId, &str, &Binding)> = table
        .scopes
        .iter()
        .enumerate()
        .flat_map(|(id, scope)| {
            scope
                .bindings
                .iter()
                .map(move |(name, binding)| (id, name.as_str(), binding))
        })
        .collect();
    let rank = |scope: ScopeId, ordinal: u32| {
        let s = &table.scopes[scope];
        (s.frame_level, s.depth, ordinal)
    };
    let mut chosen: Vec<(u32, ScopeId, &str)> = Vec::new();
    for &(s, name, b) in &all {
        if s == 0 {
            continue;
        }
        let chain = frame_chain(table, s);
        let mine = rank(s, b.ordinal);
        let variable_rival = all.iter().any(|&(o, n, ob)| {
            n == name
                && o != s
                && chain.contains(&table.scopes[o].frame)
                && rank(o, ob.ordinal) < mine
        });
        let program_rival = b.kind.is_program_wide()
            && all.iter().any(|&(o, n, ob)| {
                n == name
                    && o != s
                    && ob.kind.is_program_wide()
                    && (o != 0, ob.ordinal) < (true, b.ordinal)
            });
        if variable_rival || program_rival {
            chosen.push((b.ordinal, s, name));
        }
    }
    chosen.sort();
    chosen
        .into_iter()
        .enumerate()
        .map(|(n, (_, s, name))| ((s, name.to_string()), format!("{name}{{b{n}}}")))
        .collect()
}

#[test]
fn the_indexed_plan_matches_the_quadratic_reference() {
    let sources = [
        "let x=1; { let x=2; { let x=3; } } function f(x){ { let x=4; } function x2(){ let x=5; { const x=6; } } }",
        "function f(){ { let a=1; } { let a=\"s\"; } { function g(){} } { function g(){} } } function g(){}",
        "class C {} { class C {} } function h(){ class C {} { let C=1; } }",
        "for(let i=0;i<2;i++){ let i2=i; { let i=7; } } function k(i){ for(let i=0;i<1;i++){ { let i=2; } } }",
        "var v=1; function a(){ var v=2; { let v=3; } } function b(){ { var w=1; } { let w=2; } }",
    ];
    for source in sources {
        let table = table_of(source);
        assert_eq!(plan_renames(&table), reference_plan(&table), "{source}");
    }
}

#[test]
fn the_indexed_plan_matches_the_reference_on_many_frames() {
    // 300 functions, each shadowing a module binding, a parameter and a
    // sibling block, plus a block function per function (program-wide rule).
    let mut source = String::from("let a=0; function g(){} ");
    for n in 0..300 {
        source.push_str(&format!(
            "function f{n}(p){{ let a=p; {{ let a=1; {{ const p=2; }} }} {{ let q=3; }} {{ let q=4; }} {{ function g(){{}} }} }} "
        ));
    }
    let table = table_of(&source);
    let plan = plan_renames(&table);
    assert!(plan.len() > 900, "{}", plan.len());
    assert_eq!(plan, reference_plan(&table));
}
