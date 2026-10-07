use crate::emit::computed_member::computed_member_tests::{assert_e5506, diagnostics_for};
use crate::test_support::parse_and_lower_lir_with_env_plans;
use crate::{lower_lir_to_wasm, CodegenCtx, TargetConfig};
use kali_error::diagnostic::Diagnostic;

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
        // Ruling R7: a host binding of an ENCLOSING function, read in a closure.
        "function main(){ const t = performance; const f = () => { t.now(); }; f(); console.log(\"ok\"); } main();",
        "function main(){ const t = globalThis.performance; function inner(){ t.now(); console.log(\"ok\"); } inner(); } main();",
    ] {
        assert_not_refused(source);
    }
}

#[test]
fn an_enclosing_program_binding_refuses_in_a_closure() {
    // Ruling R7: the enclosing-scope walk finds a program value as well.
    for source in [
        "function main(){ const o={k:1}; const f = () => { console.log(o.zork()); }; f(); } main();",
        "function main(p){ const f = () => { console.log(p.zork()); }; f(); } main({k:1});",
        // `t` is declared in `main`, so its initializer's `o` resolves from `main`,
        // not from the arrow that shadows `o` with a host value.
        "function main(){ const o={k:1}; const t=o; const f = () => { const o=performance; console.log(t.zork()); }; f(); } main();",
    ] {
        assert_e5506(&diagnostics_for(source), UNRES, source);
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

/// `diagnostics_for` with `host_classes` in `ReprTable::host_derived_classes`,
/// the set `infer_reprs` fills from `kali_types::program_classes` (A-3: the
/// plain helper leaves it empty).
fn diagnostics_with_host_classes(source: &str, host_classes: &[&str]) -> Vec<Diagnostic> {
    let (program, env_plans) = parse_and_lower_lir_with_env_plans(source);
    let mut ctx = CodegenCtx::new(TargetConfig {
        max_specializations: 16,
        compat_eval: false,
        coverage: false,
    });
    ctx.env_plans = env_plans;
    for name in host_classes {
        ctx.repr_table.set_host_derived_class(name);
    }
    lower_lir_to_wasm(&mut ctx, &program).diagnostics
}

fn assert_not_refused_in(source: &str, diagnostics: &[Diagnostic]) {
    assert!(
        !diagnostics.iter().any(|d| d.message.contains(UNRES)),
        "{source}: expected no unresolved-member-call refusal, got {diagnostics:?}"
    );
}

#[test]
fn this_in_a_method_of_a_host_derived_class_is_host() {
    // Ruling R8: node prints `1` / `ok`; the baseline printed the same.
    for source in [
        "class X extends EventTarget { fire(){ this.addEventListener(\"t\", () => {}); return 1; } } const x = new X(); console.log(x.fire());",
        "class X extends EventTarget { constructor(){ super(); this.addEventListener(\"t\", () => {}); } } const x = new X(); console.log(\"ok\");",
        // A nameless class expression is named by its declarator.
        "const K = class extends EventTarget { fire(){ this.addEventListener(\"t\", () => {}); return 1; } }; const k = new K(); console.log(k.fire());",
        // R8's stated cost: a missing method on such a `this` stays warn+0.
        "class X extends EventTarget { fire(){ this.zork(); return 1; } } const x = new X(); console.log(x.fire());",
    ] {
        assert_not_refused_in(source, &diagnostics_with_host_classes(source, &["X", "K"]));
    }
}

#[test]
fn this_in_a_method_of_a_program_only_class_still_refuses() {
    for (source, host) in [
        // No host-derived class at all.
        ("class P { fire(){ this.zork(); return 1; } } const p = new P(); console.log(p.fire());", &[][..]),
        // A host-derived class elsewhere does not make `P`'s `this` host.
        ("class X extends EventTarget {} class P { fire(){ this.zork(); return 1; } } const p = new P(); console.log(p.fire());", &["X"][..]),
        // A plain function nested in a module-level function is not a method.
        ("function X(){ function g(){ return this.zork(); } return g(); } console.log(X());", &["X"][..]),
    ] {
        assert_e5506(&diagnostics_with_host_classes(source, host), UNRES, source);
    }
}

#[test]
fn a_host_derived_class_name_rebound_in_scope_is_not_the_class() {
    // Item 4 of the final fix wave: `const C = mk` shadows `class C`, so
    // `C()` is the program function's result. node: `TypeError: o.zork is not
    // a function`; HEAD before the fix printed `0`.
    let source = "class C extends EventTarget {} function main(){ const mk = () => ({k:1}); const C = mk; const o = C(); console.log(o.zork()); } main();";
    assert_e5506(
        &diagnostics_with_host_classes(source, &["C"]),
        UNRES,
        source,
    );
    // The class itself, unshadowed, stays host (A-1), and so does a binding
    // to a host-derived class expression (`const K = class … extends …`).
    for source in [
        "class C extends EventTarget {} function main(){ const o = new C(); o.addEventListener(\"t\", () => {}); console.log(\"ok\"); } main();",
        "const K = class extends EventTarget {}; const k = new K(); k.addEventListener(\"t\", () => {}); console.log(\"ok\");",
    ] {
        let host = if source.contains("const K") { "K" } else { "C" };
        assert_not_refused_in(source, &diagnostics_with_host_classes(source, &[host]));
    }
    // A parameter named like the class is not the class.
    let source = "class C extends EventTarget {} function main(C){ const o = C(); console.log(o.zork()); } main(() => ({k:1}));";
    assert_e5506(
        &diagnostics_with_host_classes(source, &["C"]),
        UNRES,
        source,
    );
}

const UNRES_READ: &str = "no lowering for that read";

fn read_refusals(source: &str) -> usize {
    diagnostics_for(source)
        .iter()
        .filter(|d| d.code == Some(5506) && d.message.contains(UNRES_READ))
        .count()
}

#[test]
fn a_member_read_on_a_program_built_root_refuses() {
    for source in [
        // captured-bindings followups §5.10
        "function mk(){ return {a:1}; } function f(){ let o=mk(); const g=()=>o; return g().a; } console.log(f());",
        "function mk(){ return {a:1}; } function show(z){ console.log(z.a); } function f(){ let o=mk(); const g=()=>{ show(o); }; g(); } f();",
        "function show(z){ return z.n * 10; } function outer(p){ const obj = p; function rd(){ return show(obj); } console.log(rd()); } const x={n:4}; outer(x);",
        "function f(p){ const o=p; const g=()=>o[\"a\"]; return g(); } const x={a:1}; console.log(f(x));",
        "function mk(){ return {a:1, s:\"xy\", arr:[1,2]}; } function f(){ let o=mk(); const g=()=>o[\"a\"]; return g(); } console.log(f());",
        // call roots and absent fields
        "function mk(){ return {a:1}; } console.log(mk().a);",
        "function id(o){ return o; } const x={a:1}; console.log(id(x).a);",
        "const o={a:1}; console.log(\"z=\"+o.z);",
        "const o={a:1}; console.log(\"z=\"+o[\"z\"]);",
        "const o={a:1}; console.log(o.z ?? 5);",
        "const o={a:1}; console.log(\"z=\"+o?.z);",
        // spread and comma reach the same fallback (spec A-1)
        "const a=[1,2]; console.log([...a]);",
        "function main(){ let n = 0; function bump() { n = n + 1; return 5; } let b = (bump(), 7); console.log(\"b=\" + b); } main();",
    ] {
        assert_e5506(&diagnostics_for(source), UNRES_READ, source);
    }
}

#[test]
fn spread_and_comma_get_the_neutral_message() {
    for source in [
        "const a=[1,2]; console.log([...a]);",
        "function main(){ let b = (1, 7); console.log(\"b=\" + b); } main();",
    ] {
        let diagnostics = diagnostics_for(source);
        assert_e5506(&diagnostics, "this expression is unavailable", source);
        assert!(
            !diagnostics.iter().any(|d| d.message.contains("reading `.")),
            "{source}: {diagnostics:?}"
        );
    }
}

#[test]
fn a_host_root_keeps_its_read() {
    for source in [
        "const f = Object.freeze(Math.log2); console.log(f(8));",
        "const finite = Number.isFinite; console.log(finite(1));",
        "const n = Object.freeze(Number[\"isNaN\"]); console.log(n(1));",
        "let t=globalThis.performance; t.now(); console.log(\"ok\");",
        "const p = Object.freeze(globalThis.String.fromCharCode); console.log(p(72));",
        "const o = Object.fromEntries([[\"a\", 1]]); console.log(o.a);",
        "let a = (console.log(\"x\"), 7); console.log(\"a=\" + a);",
        "console.log(Object.fromEntries([[\"a\",1]]).a);",
    ] {
        assert_eq!(read_refusals(source), 0, "{source}: {:?}", diagnostics_for(source));
    }
}

#[test]
fn a_shadowed_global_root_refuses() {
    let source = "let Math = {a:1}; console.log(Math.b);";
    let diagnostics = diagnostics_for(source);
    assert!(
        diagnostics.iter().any(|d| d.code == Some(5506)),
        "{source}: {diagnostics:?}"
    );
}

#[test]
fn a_this_root_in_a_plain_class_refuses() {
    // The unit harness runs no repr inference, so `this.x` reaches the read
    // fallback here; the real pipeline refuses it earlier ("field `x` is not
    // declared on class `C`").
    let source = "class C{ m(){ return this.x; } } console.log(new C().m());";
    assert_e5506(&diagnostics_for(source), UNRES_READ, source);
}

#[test]
fn a_resolved_read_is_not_refused() {
    assert_eq!(read_refusals("const o={a:1}; console.log(o.a);"), 0);
    // Closure reads resolve only through the env plans the real driver derives.
    for source in [
        "function outer(){ const obj={n:4}; function rd(){ return obj.n; } return rd(); } console.log(outer());",
    ] {
        let diagnostics = diagnostics_with_host_classes(source, &[]);
        assert!(
            !diagnostics.iter().any(|d| d.message.contains(UNRES_READ)),
            "{source}: {diagnostics:?}"
        );
    }
}
