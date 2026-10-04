use super::classes::{new_target, plan_classes, RewrittenClass};
use super::scopes::Scopes;
use crate::test_support::parse_statements;

fn plan(src: &str) -> super::classes::ClassPlans {
    let mut stmts = parse_statements(src);
    let scopes = Scopes::build(&mut stmts);
    plan_classes(&mut stmts, scopes.spelled())
}

fn messages(src: &str) -> Vec<String> {
    plan(src).diagnostics.into_iter().map(|d| d.message).collect()
}

#[test]
fn a_constructed_plain_class_is_rewritten_with_its_field_set() {
    let p = plan("class C { n = 0; constructor(v){ this.k = v; this.m = this.n + v; log(this); this.n = 1; } add(x){} } new C(1);");
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    assert_eq!(p.rewritten["C"], RewrittenClass {
        name: "C".into(),
        fields: vec!["n".into(), "k".into(), "m".into()],
        methods: ["add".to_string()].into_iter().collect(),
        leading_run: 2,
        ctor_params: vec!["v".into()],
    });
}

#[test]
fn a_class_never_constructed_is_not_rewritten() {
    assert!(plan("class C { constructor(){ this.n = 1; } }").rewritten.is_empty());
}

#[test]
fn a_stateful_chain_refuses_and_a_stateless_chain_is_untouched() {
    let stateful = messages("class A{ constructor(){ this.n=1; } } class B extends A{ g(){ return this.n; } } new B();");
    assert_eq!(stateful, [kali_common::class_construction_unavailable_message("B", kali_common::CLASS_REASON_EXTENDS)]);
    let stateless = plan("class A{ f(){return 4;} } class B extends A{} new B();");
    assert!(stateless.diagnostics.is_empty());
    assert!(stateless.rewritten.is_empty(), "a base class is never rewritten (A-1)");
}

#[test]
fn a_getter_refuses_and_a_static_only_class_is_untouched() {
    assert_eq!(messages("class A{ get v(){ return 3; } } new A();"),
        [kali_common::class_construction_unavailable_message("A", kali_common::CLASS_REASON_ACCESSOR)]);
    let statics = plan("class U { static twice(x){ return 2*x; } } U.twice(4);");
    assert!(statics.diagnostics.is_empty() && statics.rewritten.is_empty());
}

#[test]
fn an_ambiguous_class_name_refuses_when_stateful() {
    let got = messages("function a(){ class P{ constructor(){ this.n=1; } } return new P(); } function b(){ class P{ constructor(){ this.n=2; } } return new P(); }");
    assert_eq!(got, [kali_common::class_construction_unavailable_message("P", kali_common::CLASS_REASON_AMBIGUOUS)]);
}

#[test]
fn field_rules_refuse() {
    assert_eq!(messages("class C{ n; } new C();"), [kali_common::class_field_without_initial_value_message("C", "n")]);
    assert_eq!(messages("class C{ constructor(){ this.n=1; return {n:2}; } } new C();"), [kali_common::constructor_return_unavailable_message().to_string()]);
    assert_eq!(messages("class C{ n = this.k; constructor(){ this.k = 1; } } new C();"), [kali_common::class_field_initializer_this_message("C", "n")]);
}

#[test]
fn constructing_a_plain_function_refuses() {
    assert_eq!(messages("function Box(v){ this.v=v; } new Box(9);"), [kali_common::plain_function_construction_unavailable_message("Box")]);
}

#[test]
fn a_spelled_generated_name_refuses() {
    assert_eq!(messages("class C { m(){} } const __this = 1; new C();"), [kali_common::class_generated_name_collision_message("__this", "C")]);
    assert_eq!(messages("class C { m(){} } function C__m(){} new C();"), [kali_common::class_generated_name_collision_message("C__m", "C")]);
}

#[test]
fn new_target_reads_both_parse_shapes() {
    let stmts = parse_statements("new C(1); new C(1).m(2); new C().v;");
    let targets: Vec<_> = stmts.iter().map(|s| match s {
        kali_ast::Statement::ExpressionStatement(e) => match e.expression.as_ref() {
            kali_ast::Expression::NewExpression(n) => new_target(n).map(str::to_string),
            other => panic!("{other:?}"),
        },
        other => panic!("{other:?}"),
    }).collect();
    assert_eq!(targets, [Some("C".into()), Some("C".into()), Some("C".into())]);
}
