use crate::emit::computed_member::computed_member_tests::{assert_e5506, diagnostics_for};

const UNRES: &str =
    "is unavailable in the current phase: the receiver is a value this program built";

fn assert_not_refused(source: &str) {
    let diagnostics = diagnostics_for(source);
    assert!(
        !diagnostics.iter().any(|d| d.message.contains(UNRES)),
        "{source}: expected no unresolved-member-call refusal, got {diagnostics:?}"
    );
}

#[test]
fn a_program_owned_root_refuses() {
    for source in [
        "const o={k:1}; console.log(o.zork(4));",
        "const o={k:1}; console.log(o[\"zork\"](4));",
        "const o={a:{b:{}}}; console.log(o.a.b.zork());",
        "function main(){ const o={k:1}; console.log(o.zork()); } main();",
        "class C{ f(){return 1;} } const c=new C(); console.log(c.g());",
        "const s=\"abc\"; console.log(s.zork());",
        "const n=5; console.log(n.zork());",
        "const a=[1,2]; console.log(a.zork());",
        "const o={k:1}; const p=o; console.log(p.zork());",
        "function g(x){ return x.zork(); } const o={k:1}; console.log(g(o));",
        "let o={k:1}; o={k:2}; console.log(o.zork());",
        "function mk(){ return {k:1}; } const o=mk(); console.log(o.zork());",
        "const o={k:1}; console.log(o.hasOwnProperty(\"k\"));",
    ] {
        assert_e5506(&diagnostics_for(source), UNRES, source);
    }
}

#[test]
fn a_literal_start_refuses() {
    for source in [
        "console.log(\"abc\".zork());",
        "console.log(({k:1}).zork());",
        "console.log([1,2].zork());",
    ] {
        assert_e5506(&diagnostics_for(source), UNRES, source);
    }
}

#[test]
fn call_and_apply_through_a_program_root_or_an_intrinsic_prototype_refuse() {
    for source in [
        "const a=[1,2,3]; a.push.call(a, 4); console.log(a.length);",
        "const a=[1,2,3]; a.pop.call(a); console.log(a.length);",
        "function main(){ const a=[1,2,3]; a.push.call(a, 4); console.log(a.length); } main();",
        "const a=[1,2,3]; Array.prototype.push.apply(a, [4]); console.log(a.length);",
        "const a=[1,2,3]; Array.prototype.pop.call(a); console.log(a.length);",
    ] {
        assert_e5506(&diagnostics_for(source), UNRES, source);
    }
}

#[test]
fn a_host_root_keeps_its_lowering() {
    for source in [
        "performance.now(); console.log(\"ok\");",
        "const t=globalThis.performance; t.now(); console.log(\"ok\");",
        "let t=globalThis.performance; t.now(); console.log(\"ok\");",
        "function main(){ let el=document.getElementById(\"x\"); el.focus(); } main();",
        "const u=new URLSearchParams(\"a=1\"); u.append(\"b\",\"2\"); console.log(u.toString());",
        "globalThis[\"process\"][\"kill\"](0);",
        "class S { f(){ return 6; } } const s=new S(); console.log(s.f());",
        "class A{ f(){return 4;} } class B extends A{} const b=new B(); console.log(b.f());",
    ] {
        assert_not_refused(source);
    }
}

#[test]
fn a_reassigned_let_bound_from_a_host_is_not_host() {
    let source = "let t=globalThis.performance; t={}; console.log(t.now());";
    assert_e5506(&diagnostics_for(source), UNRES, source);
}

#[test]
fn a_call_result_receiver_keeps_todays_lowering() {
    // §14a's route and `mk().zork()` stop at a call node (spec §3.2).
    assert_not_refused("function mk(){ return {k:1}; } console.log(mk().zork());");
}

#[test]
fn an_alias_cycle_terminates() {
    // Not valid JS at run time (TDZ), but the provenance walk must stop.
    let source = "function main(){ const a=b; const b=a; console.log(a.zork()); } main();";
    assert_e5506(&diagnostics_for(source), UNRES, source);
}
