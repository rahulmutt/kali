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
fn a_boolean_const_capture_is_admitted_in_phase_one() {
    let found = refusals(
        "function f(){ const b=true; const g=()=>b; return g(); } f();",
        Phase::One,
    );
    assert!(found.is_empty(), "{found:?}");
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

#[test]
fn a_captured_function_declaration_makes_its_owner_an_env_owner() {
    // MIR gives the captured `h2` a cell in `o`, so `o` owns an env record
    // and `a` is two env records away from `h`.
    let found = refusals(
        "function m(){ let a=5; function o(){ function h2(){return 1;} function h(){ return a+h2(); } return h(); } return o(); } m();",
        Phase::One,
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains("a closure `h` that captures `a`")
            && found[0].contains("`a` is two or more closures away"),
        "{found:?}"
    );
}

#[test]
fn a_captured_class_declaration_makes_its_owner_an_env_owner() {
    let found = refusals(
        "function m(){ let a=5; function o(){ class K { static v(){ return 1; } } const h=()=>a+K.v(); return h(); } return o(); } m();",
        Phase::One,
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains("`a` is two or more closures away"),
        "{found:?}"
    );
}

#[test]
fn a_captured_array_literal_local_is_refused() {
    let found = refusals(
        "function f(){ let arr=[1,2,3]; const g=()=>arr.length; return g(); } f();",
        Phase::One,
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains("that captures `arr`")
            && found[0].contains("its value type has no closure cell"),
        "{found:?}"
    );
}

#[test]
fn a_captured_function_expression_local_is_refused() {
    let found = refusals(
        "function f(){ let h=function(){ return 2; }; const g=()=>h; return g() ? 1 : 0; } f();",
        Phase::One,
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains("that captures `h`")
            && found[0].contains("its value type has no closure cell"),
        "{found:?}"
    );
}

#[test]
fn a_captured_object_literal_local_is_refused_unless_its_repr_is_an_object() {
    let found = refusals(
        "function f(){ let o={a:1}; const g=()=>o.a; return g(); } f();",
        Phase::One,
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].contains("that captures `o`"), "{found:?}");
}

/// The `run`/`check` pipeline order: the captured-parameter rewrite (spec
/// §3.2, A-1), then repr inference.
fn refusals_after_rewrite(source: &str) -> Vec<String> {
    let mut statements = parse(source);
    crate::build::name_anon_functions::name_anonymous_functions(&mut statements);
    crate::build::capture_param_rewrite::rewrite_captured_params(&mut statements);
    let table = kali_types::infer_reprs(&statements);
    capture_refusals(&mut statements, &table, Phase::Two)
        .into_iter()
        .map(|d| d.message)
        .collect()
}

// A rewritten parameter local has a cell only as a proven numeric I64
// (captured-bindings followups §6 CB-9, CB-13).
#[test]
fn a_rewritten_object_parameter_is_refused_by_value_type() {
    let found = refusals_after_rewrite(
        "function f(o){ const g=()=>o.a; return g(); } const x={a:1}; console.log(f(x));",
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains("that captures `o`")
            && found[0].contains("its value type has no closure cell"),
        "{found:?}"
    );
}

#[test]
fn a_rewritten_boolean_parameter_with_a_numeric_proof_is_admitted() {
    // The numeric proof admits `f(true)`; `run` promotes the cell and renders
    // `1`, as uncaptured (followups §6 CB-11, spec A-2.1).
    let found = refusals_after_rewrite("function f(b){ const g=()=>b; return g(); } f(true);");
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn a_rewritten_numeric_parameter_is_admitted() {
    let found = refusals_after_rewrite("function f(k){ const g=()=>k; return g(); } f(5);");
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn a_rewritten_parameter_without_a_proof_is_refused() {
    // v9: no call edge to the const-bound arrow, so no numeric proof.
    let found = refusals_after_rewrite("const f=(k)=>{ const g=()=>k; return g(); }; f(5);");
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains("its value type has no closure cell"),
        "{found:?}"
    );
}

#[test]
fn a_user_written_copy_of_an_object_parameter_is_admitted() {
    // Only the `{p}` spelling is keyed (followups §6 CB-13); `const o=p` (d6/e2 shape,
    // lowered by `run`'s C2) stays admitted — the A-2.6 residue for ol2.
    let found = refusals_after_rewrite(
        "function outer(p){ let obj = p; function rd(){ return obj.n; } console.log(rd()); } const x={n:4}; outer(x);",
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn a_user_written_copy_of_a_numeric_parameter_is_admitted() {
    // p2: proven numeric, so `run` promotes it.
    let found =
        refusals_after_rewrite("function f(k){ let n=k; const g=()=>n; return g(); } f(5);");
    assert!(found.is_empty(), "{found:?}");
}

// Captured-bindings A-4 (followups §6 CB-14, CB-15): a capturer may only write a
// captured F64, as an assignment statement (`= += -= *= /=`); any other
// reference to it is a read codegen refuses.
fn is_value_type_refusal(found: &[String]) -> bool {
    found.len() == 1 && found[0].contains("its value type has no closure cell")
}

#[test]
fn an_f64_capture_read_is_refused_in_both_phases() {
    let source = "function f(){ let x=1.5; const g=()=>x; return g(); } f();";
    for phase in [Phase::One, Phase::Two] {
        assert!(is_value_type_refusal(&refusals(source, phase)), "{phase:?}");
    }
}

#[test]
fn an_f64_capture_written_by_assignment_statements_is_admitted() {
    let found = refusals(
        "function f(){ let x=1.5; const g=()=>{ x=2.5; x+=1; x-=0.5; x*=2; x/=4; }; g(); return x; } f();",
        Phase::Two,
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn an_f64_write_whose_right_hand_side_reads_the_capture_is_refused() {
    let found = refusals(
        "function f(){ let x=1.5; const g=()=>{ x=x+1.25; }; g(); return x; } f();",
        Phase::Two,
    );
    assert!(is_value_type_refusal(&found), "{found:?}");
}

#[test]
fn an_f64_assignment_used_as_a_value_is_refused() {
    let found = refusals(
        "function f(){ let x=1.5; const g=()=>{ console.log(x=2.5); }; g(); return x; } f();",
        Phase::Two,
    );
    assert!(is_value_type_refusal(&found), "{found:?}");
}

#[test]
fn an_f64_remainder_or_update_is_refused() {
    for body in ["x%=2;", "x++;", "x|=1;"] {
        let source =
            format!("function f(){{ let x=1.5; const g=()=>{{ {body} }}; g(); return x; }} f();");
        assert!(
            is_value_type_refusal(&refusals(&source, Phase::Two)),
            "{body}"
        );
    }
}

#[test]
fn a_boolean_const_capture_is_admitted_in_phase_two() {
    // A-2.1: the capture read carries `ValueShape::Boolean`.
    let found = refusals(
        "function f(){ const b=true; const g=()=>b; return g(); } f();",
        Phase::Two,
    );
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn a_rewritten_f64_parameter_is_refused_even_with_a_numeric_proof() {
    // The `{p}` rule stays I64-with-proof only (spec A-4), even for a
    // write-only capturer.
    let found = refusals_after_rewrite(
        "function f(x){ const g=()=>{ x=2.5; }; g(); return x; } console.log(f(1.5));",
    );
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains("its value type has no closure cell"),
        "{found:?}"
    );
}
