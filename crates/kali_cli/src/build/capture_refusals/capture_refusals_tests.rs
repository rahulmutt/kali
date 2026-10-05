use super::*;
use crate::build::block_scope_rename::test_support::parse;

fn refusals(source: &str, phase: Phase) -> Vec<String> {
    let mut statements = parse(source);
    crate::build::name_anon_functions::name_anonymous_functions(&mut statements);
    let table = kali_types::infer_reprs(&statements);
    capture_refusals(&mut statements, &table, phase)
        .into_iter()
        .map(|d| d.message)
        .collect()
}

#[test]
fn a_captured_parameter_is_refused_in_phase_one() {
    let found = refusals(
        "function f(k){ const g=()=>k; return g(); } f(5);",
        Phase::One,
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("`k` is a parameter of `f`"), "{found:?}");
}

#[test]
fn a_function_declaration_capturer_is_keyed_by_name() {
    let found = refusals(
        "function f(k){ function g(){ return k; } return g(); } f(5);",
        Phase::One,
    );
    assert!(
        found[0].contains("a closure `g` that captures `k`"),
        "{found:?}"
    );
}

#[test]
fn a_string_local_capture_is_refused() {
    let found = refusals(
        "function f(){ let s=\"hi\"; const g=()=>s; return g(); } f();",
        Phase::One,
    );
    assert!(
        found[0].contains("its value type has no closure cell"),
        "{found:?}"
    );
}

#[test]
fn a_depth_two_capture_is_refused() {
    let found = refusals(
        "function m(){ let a=5; const o=()=>{ let z=10; const h=()=>z+a; return h(); }; return o(); } m();",
        Phase::One,
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains("`a` is two or more closures away"),
        "{found:?}"
    );
}

#[test]
fn an_intermediate_function_without_captured_bindings_is_transparent() {
    // `mid` owns nothing captured, so `a` is one env record away (MIR §3.4).
    let found = refusals(
        "function m(){ let a=5; function mid(){ const h=()=>a; return h(); } return mid(); } m();",
        Phase::One,
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn an_i64_local_capture_is_admitted() {
    assert!(refusals(
        "function f(){ let c=0; const inc=()=>{ c+=1; }; inc(); return c; } f();",
        Phase::One
    )
    .is_empty());
}

#[test]
fn a_boolean_const_capture_is_refused_in_phase_one() {
    let found = refusals(
        "function f(){ const b=true; const g=()=>b; return g(); } f();",
        Phase::One,
    );
    assert!(found[0].contains("that captures `b`"), "{found:?}");
}

#[test]
fn module_bindings_are_not_captures() {
    assert!(refusals("let s=\"x\"; function f(){ return s; } f();", Phase::One).is_empty());
}

#[test]
fn a_capture_of_a_per_iteration_binding_is_left_to_block_scoping() {
    // A-2.5: `k` lives in the loop's record (the loop registers a closure).
    let found = refusals(
        "function m(){ for (let i = 0; i < 2; i++) { const k = i * 7; queueMicrotask(function cb() { let z = 1; function h() { return z + k; } console.log(h()); }); } } m();",
        Phase::One,
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn a_capture_through_a_registering_loop_is_left_to_block_scoping() {
    // A-2.5: `g` is created in an iteration-record loop, so its path to `s`
    // crosses the record (`through_iteration`).
    let found = refusals(
        "function m(){ let s=\"x\"; for(let i=0;i<2;i++){ setTimeout(()=>console.log(i),0); const g=()=>s; g(); } } m();",
        Phase::One,
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn a_loop_without_a_registration_owns_no_record() {
    let found = refusals(
        "function m(){ for(let i=0;i<2;i++){ let s=\"x\"; const g=()=>s; g(); } } m();",
        Phase::One,
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains("its value type has no closure cell"),
        "{found:?}"
    );
}
