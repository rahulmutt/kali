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
fn a_depth_two_capture_owned_by_a_record_is_refused() {
    // Ruling R13 (`rv/d06.js`, node `1 8`): `h` reads `k`, owned by the loop's
    // record, through `cb`'s own env. The path crosses no OTHER record, so
    // `through_iteration` is false; the owner being a record refuses it.
    let (_, plans) = parse_and_lower_lir_with_env_plans(
        "function m(){ for(let i=0;i<2;i++){ const k=i*7; queueMicrotask(function cb(){ let z=1; function h(){ return z+k; } console.log(h()); }); } } m();",
    );
    let d = iteration_capture_diagnostics(&plans);
    assert!(
        d.iter().any(|d| d
            .message
            .contains("captures `k` through a per-iteration record")),
        "{d:?}"
    );
}

#[test]
fn a_depth_one_capture_owned_by_a_record_is_not_refused() {
    let (_, plans) = parse_and_lower_lir_with_env_plans(
        "function m(){ for(let i=0;i<2;i++){ const k=i*7; queueMicrotask(()=>console.log(k)); } } m();",
    );
    assert!(iteration_capture_diagnostics(&plans).is_empty());
}

#[test]
fn the_call_switch_hold_name_cannot_collide_with_source_identifiers() {
    assert!(iteration_call_save_local_name().ends_with("#env"));
    assert_ne!(
        iteration_call_save_local_name(),
        iteration_prev_local_name()
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

fn lowering_diagnostics(
    source: &str,
    edit_plans: impl FnOnce(&mut BTreeMap<String, EnvPlan>),
) -> Vec<Diagnostic> {
    let (program, mut env_plans) = parse_and_lower_lir_with_env_plans(source);
    edit_plans(&mut env_plans);
    let mut ctx = crate::CodegenCtx::new(crate::TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    ctx.env_plans = env_plans;
    crate::lower_lir_to_wasm(&mut ctx, &program).diagnostics
}

fn has_e5506(diagnostics: &[Diagnostic], needle: &str) -> bool {
    diagnostics
        .iter()
        .any(|d| d.code == Some(e5::FEATURE_UNAVAILABLE as u32) && d.message.contains(needle))
}

#[test]
fn a_for_head_declares_only_its_init_names() {
    let (lir, _) = parse_and_lower_lir_with_env_plans(
        "function m(){ for(let i=0; i<2; i++){ const k=i; } } m();",
    );
    let for_loop = loops(&lir.nodes)[0];
    let init = lir.nodes[for_loop.0 as usize].children[0];
    let expected: BTreeSet<String> = ["i"].iter().map(|s| s.to_string()).collect();
    assert_eq!(names_declared_in(&lir.nodes, init), expected);
    let whole: BTreeSet<String> = ["i", "k"].iter().map(|s| s.to_string()).collect();
    assert_eq!(names_declared_in_loop(&lir.nodes, for_loop), whole);
}

#[test]
fn a_direct_continue_is_found_but_not_one_in_a_nested_loop_or_function() {
    let body_of = |source: &str| {
        let (lir, _) = parse_and_lower_lir_with_env_plans(source);
        let outer = loops(&lir.nodes)[0];
        let body = *lir.nodes[outer.0 as usize].children.last().unwrap();
        contains_direct_continue(&lir.nodes, body)
    };
    assert!(body_of(
        "function m(){ for(let i=0;i<3;i++){ if(i===1) continue; } } m();"
    ));
    assert!(body_of(
        "function m(){ for(let i=0;i<3;i++){ switch(i){ case 1: continue; default: break; } } } m();"
    ));
    assert!(!body_of(
        "function m(){ for(let i=0;i<3;i++){ for(let j=0;j<3;j++){ continue; } } } m();"
    ));
    assert!(!body_of(
        "function m(){ for(let i=0;i<3;i++){ const f=()=>{ for(;;){ continue; } }; } } m();"
    ));
}

#[test]
fn lowering_an_owner_loop_places_its_plan() {
    let diagnostics = lowering_diagnostics(
        "function m(){ for(let i=0;i<3;i++){ setTimeout(()=>console.log(i), 0); } } m();",
        |_| {},
    );
    assert!(
        diagnostics
            .iter()
            .all(|d| d.code != Some(e5::FEATURE_UNAVAILABLE as u32)),
        "{diagnostics:?}"
    );
}

#[test]
fn lowering_refuses_continue_in_an_owner_for() {
    let diagnostics = lowering_diagnostics(
        "function m(){ for(let i=0;i<3;i++){ if(i===1) continue; queueMicrotask(()=>console.log(i)); } } m();",
        |_| {},
    );
    assert!(
        has_e5506(&diagnostics, "`continue` in a `for` loop"),
        "{diagnostics:?}"
    );
    assert!(
        !has_e5506(&diagnostics, "was planned but no loop declared"),
        "{diagnostics:?}"
    );
}

#[test]
fn lowering_refuses_an_unrolled_for_of_owner() {
    let diagnostics = lowering_diagnostics(
        "function m(){ for(const x of [5,6]){ queueMicrotask(()=>console.log(x)); } } m();",
        |_| {},
    );
    assert!(
        has_e5506(&diagnostics, "compile-time iterable that captures `x`"),
        "{diagnostics:?}"
    );
}

#[test]
fn an_iteration_plan_no_loop_declares_is_refused_by_the_backstop() {
    let diagnostics = lowering_diagnostics(
        "function m(){ for(let i=0;i<3;i++){ console.log(i); } } m();",
        |plans| {
            plans.insert(
                "m{iter7}".to_string(),
                EnvPlan {
                    owns_env: true,
                    cells: vec![kali_mir::EnvCell {
                        name: "nowhere".to_string(),
                        offset: 0,
                        is_scalar: true,
                    }],
                    captured: Vec::new(),
                    iteration_of: Some("m".to_string()),
                },
            );
        },
    );
    assert!(
        has_e5506(
            &diagnostics,
            "per-iteration closure record `m{iter7}` was planned"
        ),
        "{diagnostics:?}"
    );
}
