use super::captures::check_captures;
use super::classes::plan_classes;
use super::scopes::Scopes;
use crate::test_support::parse_statements;
use kali_common::{
    class_construction_unavailable_message, CLASS_REASON_ENCLOSING_LOCAL,
    CLASS_REASON_INITIALIZER_SCOPE,
};

fn captures(src: &str) -> Vec<String> {
    let mut stmts = parse_statements(src);
    let spelled = Scopes::build(&mut stmts).spelled().clone();
    let plans = plan_classes(&mut stmts, &spelled);
    let scopes = Scopes::build(&mut stmts);
    check_captures(&mut stmts, &scopes, &plans)
        .into_iter()
        .map(|d| d.message)
        .collect()
}

// Ruling R-25.
#[test]
fn a_nested_class_reading_an_enclosing_local_refuses() {
    let want = class_construction_unavailable_message("C", CLASS_REASON_ENCLOSING_LOCAL);
    for program in [
        "function make(k){ class C { constructor(){ this.n = k; } } return new C(); } make(1);",
        "function make(k){ class C { constructor(){ this.n = 1; } get(){ return this.n + k; } } return new C().get(); } make(1);",
        "function make(){ const k = 2; class C { n = k; } return new C(); } make();",
        "function make(k){ class C { constructor(){ this.n = 1; } get(){ const f = () => k; return f(); } } return new C(); } make(1);",
        "function make(){ function h(){ return 1; } class C { m(){ return h(); } } return new C(); } make();",
    ] {
        assert_eq!(captures(program), [want.clone()], "{program}");
    }
}

#[test]
fn own_params_locals_and_program_names_are_not_captures() {
    for program in [
        "const k = 3; function g(){ return 1; } class C { n = k; constructor(v){ this.m = v + g(); } get(x){ const y = x; return y + this.n; } } new C(1);",
        "function main(){ class C { constructor(v){ this.v = v; } get(){ const t = 1; return this.v + t; } } const c = new C(2); return c.get(); } main();",
    ] {
        assert!(captures(program).is_empty(), "{program}: {:?}", captures(program));
    }
}

// Ruling R-26.
#[test]
fn a_field_initializer_naming_a_constructor_binding_refuses() {
    let want = class_construction_unavailable_message("C", CLASS_REASON_INITIALIZER_SCOPE);
    for program in [
        "const x = 10; class C { a = x; constructor(x){ this.b = x; } } new C(3);",
        "const x = 10; class C { a = x + 1; constructor(){ const x = 2; this.b = x; } } new C();",
        "class C { a = x; constructor(){ var x = 2; } } new C();",
    ] {
        assert!(
            captures(program).contains(&want),
            "{program}: {:?}",
            captures(program)
        );
    }
    // An initializer naming something the constructor does not declare is fine.
    assert!(
        captures("const x = 10; class C { a = x; constructor(y){ this.b = y; } } new C(3);")
            .is_empty()
    );
}
