use super::rewrite_class_instances;
use crate::test_support::parse_statements;

fn rewrite(src: &str) -> (Vec<kali_ast::Statement>, Vec<String>, bool) {
    let mut stmts = parse_statements(src);
    let r = rewrite_class_instances(&mut stmts);
    (stmts, r.diagnostics.into_iter().map(|d| d.message).collect(), r.changed)
}

#[test]
fn the_spec_example_translates() {
    let (got, d, changed) = rewrite(
        "class C { n = 0; constructor(v){ this.k = v; this.m = this.n + v; log(this); this.n = 1; } } function log(x){ return x.n; } const c = new C(2);",
    );
    assert!(d.is_empty(), "{d:?}");
    assert!(changed);
    let want = parse_statements(
        "function C__new(v){ let __f_n = 0; let __f_k = v; let __f_m = __f_n + v; const __this = { n: __f_n, k: __f_k, m: __f_m }; log(__this); __this.n = 1; return __this; } function log(x){ return x.n; } const c = C__new(2);",
    );
    assert_eq!(got, want);
}

#[test]
fn methods_take_this_and_arrows_keep_it_but_functions_do_not() {
    // R-16 refuses `this` inside an arrow, so the translation is driven directly.
    let src = "class C { constructor(){ this.n = 1; } bump(){ const f = () => { this.n = this.n + 1; }; const g = function(){ return this; }; f(); return this.n; } } const c = new C(); c.bump();";
    let mut got = parse_statements(src);
    let spelled = super::scopes::Scopes::build(&mut got).spelled().clone();
    let plans = super::classes::plan_classes(&mut got, &spelled);
    super::translate::translate_classes(&mut got, &plans);
    let want = parse_statements(
        "function C__new(){ let __f_n = 1; const __this = { n: __f_n }; return __this; } function C__bump(__this){ const f = () => { __this.n = __this.n + 1; }; const g = function(){ return this; }; f(); return __this.n; } const c = new C(); c.bump();",
    );
    assert_eq!(got, want);
    // The full entry refuses the arrow's `this`.
    let (_, d, _) = rewrite(src);
    assert!(
        d.contains(&kali_common::class_instance_position_message("C", "a value inside an arrow function or function expression")),
        "{d:?}"
    );
}

#[test]
fn a_bare_return_in_the_constructor_returns_the_instance() {
    let (got, d, _) = rewrite("class C { constructor(v){ this.n = v; if (v > 1) { return; } this.n = 0; } } new C(2);");
    assert!(d.is_empty(), "{d:?}");
    let want = parse_statements("function C__new(v){ let __f_n = v; const __this = { n: __f_n }; if (v > 1) { return __this; } __this.n = 0; return __this; } C__new(2);");
    assert_eq!(got, want);
}

#[test]
fn a_program_without_rewritable_classes_is_byte_identical() {
    for src in [
        "const o = { n: 1 }; o.n = 2; console.log(o.n);",
        "class X extends EventTarget { fire(){ this.addEventListener('t', () => {}); return 1; } } const x = new X(); x.fire();",
        "class U { static twice(x){ return 2*x; } } U.twice(4);",
        "class A{ f(){return 4;} } class B extends A{} const b = new B(); b.f();",
    ] {
        let before = parse_statements(src);
        let (after, d, changed) = rewrite(src);
        assert!(!changed && d.is_empty(), "{src}: {d:?}");
        assert_eq!(after, before, "{src}");
    }
}

#[test]
fn nested_class_translates_in_place() {
    let (got, d, _) = rewrite("function main(){ class C{ constructor(v){ this.v=v; } } const c=new C(2); return c.v; }");
    assert!(d.is_empty(), "{d:?}");
    let want = parse_statements("function main(){ function C__new(v){ let __f_v = v; const __this = { v: __f_v }; return __this; } const c=C__new(2); return c.v; }");
    assert_eq!(got, want);
}
