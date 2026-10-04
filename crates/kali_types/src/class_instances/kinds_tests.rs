use super::classes::plan_classes;
use super::kinds::{field_kinds, Kind};
use super::provenance::{build_env, reassociate_new, Provenance};
use super::scopes::Scopes;
use crate::test_support::parse_statements;

/// `(class, field, kind)` for every field of every rewritten class.
fn kinds(src: &str) -> Vec<(String, String, Kind)> {
    let mut stmts = parse_statements(src);
    let spelled = Scopes::build(&mut stmts).spelled().clone();
    let plans = plan_classes(&mut stmts, &spelled);
    reassociate_new(&mut stmts, &plans);
    let scopes = Scopes::build(&mut stmts);
    let env = build_env(&mut stmts, &scopes, &plans);
    let prov = Provenance::solve(&mut stmts, &env);
    field_kinds(&mut stmts, &env, &prov)
        .into_iter()
        .map(|((c, f), k)| (c, f, k))
        .collect()
}

fn kind_of(src: &str) -> Kind {
    let all = kinds(src);
    assert_eq!(all.len(), 1, "{all:?}");
    all[0].2
}

#[test]
fn numbers_booleans_and_arithmetic_over_them_are_proven() {
    for src in [
        "class C { n = 1; } new C();",
        "class C { constructor(){ this.v = true; } } new C();",
        "class C { constructor(){ this.n = 0; } add(x){ this.n = this.n + x; } } const c = new C(); c.add(3);",
        "class C { constructor(v){ this.n = v; } } function mk(v){ return new C(v); } const c = mk(7);",
        "class C { constructor(){ this.n = 2; this.m = this.n * 3 > 4; } } new C();",
        "class C { constructor(v){ this.n = -v; } } new C(Math.floor(2.5));",
    ] {
        let all = kinds(src);
        assert!(all.iter().all(|(_, _, k)| *k == Kind::NumBool), "{src}: {all:?}");
    }
}

#[test]
fn strings_null_undefined_bigint_and_aggregates_are_not_proven() {
    for src in [
        "class P { name = \"bob\"; } new P();",
        "class P { constructor(n){ this.name = n; } } new P(\"bob\");",
        "class P { constructor(){ this.n = 1; } set(v){ this.n = v; } } const p = new P(); p.set(\"x\");",
        "class C { constructor(){ this.v = null; } } new C();",
        "class C { constructor(){ this.v = undefined; } } new C();",
        "class C { constructor(a){ this.a = a; } } new C(10n);",
        "class C { constructor(a){ this.a = a; } } new C();",
        "class C { constructor(){ this.a = []; } } new C();",
        "class C { constructor(){ this.a = {}; } } new C();",
        "class C { constructor(){ this.n = 1; } } const c = new C(); c.n = `t`;",
        "class C { constructor(){ this.n = 1; } } const c = new C(); c.n = c.n + \"!\";",
        "class C { constructor(v){ this.n = v; } } function f(x){ return new C(x); } const g = f; g(1);",
    ] {
        assert_eq!(kind_of(src), Kind::Other, "{src}");
    }
}
