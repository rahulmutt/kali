use crate::test_support::analyze;

#[test]
fn a_registered_closure_over_a_for_let_makes_the_loop_an_owner() {
    let p =
        analyze("function m(){ for(let i=0;i<3;i++){ setTimeout(()=>console.log(i), 0); } } m();");
    assert_eq!(p.iteration_scopes.len(), 1);
    let s = &p.iteration_scopes[0];
    assert_eq!(s.label, "m{iter0}");
    assert_eq!(s.function, "m");
    assert_eq!(s.cells, vec!["i".to_string()]);
    assert_eq!(
        p.parent_labels.get("m{iter0}"),
        Some(&Some("m".to_string()))
    );
    let closure = p
        .parent_labels
        .iter()
        .find(|(_, parent)| parent.as_deref() == Some("m{iter0}"));
    assert!(closure.is_some(), "{:?}", p.parent_labels);
}

#[test]
fn a_module_loop_owner_has_no_parent() {
    let p = analyze("for(let i=0;i<3;i++){ queueMicrotask(()=>console.log(i)); }");
    assert_eq!(p.iteration_scopes[0].label, "{iter0}");
    assert_eq!(p.iteration_scopes[0].function, "");
    assert_eq!(p.parent_labels.get("{iter0}"), Some(&None));
}

#[test]
fn a_body_const_is_a_cell_and_the_loop_counter_is_not_captured() {
    let p = analyze("function m(){ for(let i=0;i<2;i++){ const k=i*3; queueMicrotask(()=>console.log(k)); } } m();");
    assert_eq!(p.iteration_scopes[0].cells, vec!["k".to_string()]);
}

#[test]
fn a_synchronous_closure_does_not_make_an_owner() {
    let p = analyze(
        "function m(){ let s=0; for(let i=0;i<3;i++){ const g=()=>i; s+=g(); } return s; } m();",
    );
    assert!(p.iteration_scopes.is_empty());
}

#[test]
fn a_registration_without_a_loop_capture_does_not_make_an_owner() {
    let p = analyze(
        "function m(){ let a=1; for(let i=0;i<2;i++){ setTimeout(()=>console.log(a), 0); } } m();",
    );
    assert!(p.iteration_scopes.is_empty());
}

#[test]
fn an_add_event_listener_member_call_registers() {
    let p = analyze("function m(t){ for(let i=0;i<2;i++){ t.addEventListener(\"x\", ()=>console.log(i)); } } m(new EventTarget());");
    assert_eq!(p.iteration_scopes.len(), 1);
}

#[test]
fn nested_owner_loops_are_two_owners_and_the_inner_parents_to_the_outer() {
    let p = analyze(
        "for(let i=0;i<2;i++){ for(let j=0;j<2;j++){ setTimeout(()=>console.log(i,j),0); } }",
    );
    let labels: Vec<&str> = p
        .iteration_scopes
        .iter()
        .map(|s| s.label.as_str())
        .collect();
    assert_eq!(labels, vec!["{iter1}", "{iter0}"]); // closed inner-first
    assert_eq!(
        p.parent_labels.get("{iter1}"),
        Some(&Some("{iter0}".to_string()))
    );
    assert_eq!(p.parent_labels.get("{iter0}"), Some(&None));
    assert!(p
        .parent_labels
        .values()
        .any(|parent| parent.as_deref() == Some("{iter1}")));
}

#[test]
fn a_loop_in_a_nested_function_belongs_to_that_function() {
    let p = analyze("function m(){ function g(){ for(let i=0;i<2;i++){ setTimeout(()=>console.log(i),0); } } g(); } m();");
    assert_eq!(p.iteration_scopes[0].function, "g");
    assert_eq!(p.iteration_scopes[0].label, "g{iter0}");
}

#[test]
fn for_of_and_for_in_loops_are_candidates_too() {
    let p = analyze("function m(xs){ for(const x of xs){ setTimeout(()=>console.log(x),0); } for(const k in xs){ setTimeout(()=>console.log(k),0); } } m([1]);");
    let labels: Vec<&str> = p
        .iteration_scopes
        .iter()
        .map(|s| s.label.as_str())
        .collect();
    assert_eq!(labels, vec!["m{iter0}", "m{iter1}"]);
}
