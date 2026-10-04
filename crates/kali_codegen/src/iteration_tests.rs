use super::*;
use crate::test_support::parse_and_lower_lir_with_env_plans;

fn loops(nodes: &[LirNode]) -> Vec<LirNodeId> {
    nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| {
            n.kind == LirNodeKind::Branch
                && matches!(
                    n.text.as_deref(),
                    Some("for" | "while" | "do-while" | "for-of" | "for-await-of" | "for-in")
                )
        })
        .map(|(i, _)| LirNodeId(i as u32))
        .collect()
}

#[test]
fn the_owner_loop_is_found_by_its_declared_cells() {
    let (lir, plans) = parse_and_lower_lir_with_env_plans(
        "function m(){ for(let i=0;i<3;i++){ setTimeout(()=>console.log(i), 0); } } m();",
    );
    let mine = iteration_plans_of(&plans, "m");
    assert_eq!(mine.len(), 1);
    let found: Vec<_> = loops(&lir.nodes)
        .into_iter()
        .filter_map(|l| iteration_label_for_loop(&lir.nodes, l, &mine))
        .collect();
    assert_eq!(found, vec!["m{iter0}"]);
}

#[test]
fn a_nested_loop_declaration_belongs_to_the_inner_loop_only() {
    let (lir, _) = parse_and_lower_lir_with_env_plans(
        "function m(){ for(let i=0;i<2;i++){ for(let j=0;j<2;j++){ const k=j; } } } m();",
    );
    let ls = loops(&lir.nodes);
    let sets: Vec<_> = ls
        .iter()
        .map(|l| names_declared_in_loop(&lir.nodes, *l))
        .collect();
    assert!(sets.contains(&["i".to_string()].into_iter().collect()));
    assert!(sets.contains(&["j".to_string(), "k".to_string()].into_iter().collect()));
}

#[test]
fn a_depth_two_capture_through_a_record_is_refused() {
    let (_, plans) = parse_and_lower_lir_with_env_plans(
        "function m(){ let a=10; for(let i=0;i<2;i++){ setTimeout(()=>console.log(a+i),0); } } m();",
    );
    let d = iteration_capture_diagnostics(&plans);
    assert_eq!(d.len(), 1);
    assert!(
        d[0].message
            .contains("captures `a` through a per-iteration record"),
        "{}",
        d[0].message
    );
}

#[test]
fn the_module_root_plan_key_is_empty() {
    assert_eq!(plan_key("_start"), "");
    assert_eq!(plan_key("m"), "m");
}

#[test]
fn a_module_loop_owner_is_found_under_the_empty_plan_key() {
    let (lir, plans) = parse_and_lower_lir_with_env_plans(
        "for(let i=0;i<3;i++){ setTimeout(()=>console.log(i), 0); }",
    );
    let root = iteration_plans_of(&plans, plan_key("_start"));
    let found: Vec<_> = loops(&lir.nodes)
        .into_iter()
        .filter_map(|l| iteration_label_for_loop(&lir.nodes, l, &root))
        .collect();
    assert_eq!(found, vec!["{iter0}"]);
    assert_eq!(owner_repr_namespace(&plans, "{iter0}"), "_start");
}

#[test]
fn an_owner_label_resolves_to_its_functions_repr_namespace() {
    let (_, plans) = parse_and_lower_lir_with_env_plans(
        "function m(){ for(let i=0;i<3;i++){ setTimeout(()=>console.log(i), 0); } } m();",
    );
    assert_eq!(owner_repr_namespace(&plans, "m{iter0}"), "m");
    assert_eq!(owner_repr_namespace(&plans, "m"), "m");
}

#[test]
fn a_nested_function_declaration_is_not_declared_in_the_loop() {
    let (lir, _) = parse_and_lower_lir_with_env_plans(
        "function m(){ for(let i=0;i<2;i++){ if (i) { let b=i; } const f=()=>{ let x=1; return x; }; f(); } } m();",
    );
    let sets: Vec<_> = loops(&lir.nodes)
        .iter()
        .map(|l| names_declared_in_loop(&lir.nodes, *l))
        .collect();
    let expected: BTreeSet<String> = ["i", "b", "f"].iter().map(|s| s.to_string()).collect();
    assert_eq!(sets, vec![expected]);
}

#[test]
fn a_for_of_head_declaration_is_declared_in_the_loop() {
    let (lir, _) = parse_and_lower_lir_with_env_plans(
        "function m(a){ for(const x of a){ let y=x; } } m([1]);",
    );
    let sets: Vec<_> = loops(&lir.nodes)
        .iter()
        .map(|l| names_declared_in_loop(&lir.nodes, *l))
        .collect();
    let expected: BTreeSet<String> = ["x", "y"].iter().map(|s| s.to_string()).collect();
    assert_eq!(sets, vec![expected]);
}

#[test]
fn a_program_without_owners_has_no_iteration_plans_or_refusals() {
    let (_, plans) = parse_and_lower_lir_with_env_plans(
        "function m(){ let a=1; const g=()=>a; for(let i=0;i<2;i++){ console.log(g()+i); } } m();",
    );
    assert!(iteration_plans_of(&plans, "m").is_empty());
    assert!(iteration_capture_diagnostics(&plans).is_empty());
}

#[test]
fn save_local_names_cannot_collide_with_source_identifiers() {
    assert_eq!(
        iteration_save_local_name("m{iter0}"),
        "__iter_savem{iter0}#env"
    );
    assert!(iteration_prev_local_name().ends_with("#env"));
}

#[test]
fn lowering_reports_the_depth_two_refusal() {
    let source =
        "function m(){ let a=10; for(let i=0;i<2;i++){ setTimeout(()=>console.log(a+i),0); } } m();";
    let (program, env_plans) = parse_and_lower_lir_with_env_plans(source);
    let mut ctx = crate::CodegenCtx::new(crate::TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    ctx.env_plans = env_plans;
    let diagnostics = crate::lower_lir_to_wasm(&mut ctx, &program).diagnostics;
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code == Some(e5::FEATURE_UNAVAILABLE as u32)
                && d.message
                    .contains("captures `a` through a per-iteration record")),
        "{diagnostics:?}"
    );
}
