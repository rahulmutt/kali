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

#[test]
fn an_uninitialized_field_is_not_bound_for_this_reads() {
    assert_eq!(messages("class C { n; constructor(){ this.m = this.n; this.n = 1; } } new C();"),
        [kali_common::class_field_without_initial_value_message("C", "n")]);
    assert_eq!(plan("class C { n; constructor(){ this.m = this.n; this.n = 1; } } new C();").rewritten["C"].leading_run, 0);
    assert_eq!(messages("class C { n; m = this.n; constructor(){ this.n = 1; } } new C();"),
        [kali_common::class_field_initializer_this_message("C", "m")]);
}

#[test]
fn this_in_a_nested_function_ends_the_run_or_refuses_an_initializer() {
    let p = plan("class C { constructor(){ this.n = 1; this.cb = () => this.n; } } new C();");
    assert_eq!(p.rewritten["C"].fields, ["n".to_string()]);
    assert_eq!(p.rewritten["C"].leading_run, 1);
    assert_eq!(messages("class C { n = 1; cb = () => this.n; } new C();"),
        [kali_common::class_field_initializer_this_message("C", "cb")]);
}

#[test]
fn a_return_in_a_nested_arrow_does_not_refuse_and_a_run_stops_on_an_unbound_read() {
    assert!(messages("class C { constructor(){ this.n = 1; const f = () => { return 2; }; } } new C();").is_empty());
    let p = plan("class C { constructor(){ this.a = 1; this.b = this.g; this.g = 2; } } new C();");
    assert_eq!(p.rewritten["C"].leading_run, 1);
}

// Ruling R-27.
#[test]
fn a_field_and_a_method_with_the_same_name_refuse() {
    let want = kali_common::class_construction_unavailable_message("C", kali_common::CLASS_REASON_FIELD_METHOD);
    assert_eq!(messages("class C { constructor(){ this.go = 7; } go(){ return 1; } } new C().go();"), [want.clone()]);
    assert_eq!(messages("class C { go = 7; go(){ return 1; } } new C();"), [want]);
}

// Ruling R-29: `new X().m()` of a program class kali does not rewrite.
#[test]
fn a_chained_new_of_an_out_of_slice_class_refuses() {
    let msg = |c: &str| kali_common::class_construction_unavailable_message(c, kali_common::CLASS_REASON_SAME_EXPRESSION);
    assert_eq!(messages("class S { static k(){ return 0; } push(v){ return v + 1; } } console.log(new S().push(1));"), [msg("S")]);
    assert_eq!(messages("const K = class { push(v){ return v + 1; } }; console.log(new K().push(1));"), [msg("K")]);
    assert_eq!(messages("const K = class X { f(){ return 1; } }; new X().f();"), [msg("X")]);
    // Bound to a variable first, an out-of-slice stateless class is unchanged (A-1).
    assert!(messages("class S { static k(){ return 0; } push(v){ return v + 1; } } const s = new S(); s.push(1);").is_empty());
    // A rewritten class and a host-derived one are not refused here.
    assert!(messages("class S { push(v){ return v + 1; } } new S().push(1);").is_empty());
    assert!(messages("class X extends EventTarget { f(){ return 1; } } new X().f();").is_empty());
    // A stateless class in an `extends` chain is out of the slice too.
    assert_eq!(messages("class A{ f(){return 4;} } class B extends A{} new B().f();"), [msg("B")]);
    // `new S()` with no arguments parses as `new (S())`: that is not a chain.
    assert!(messages("class S { static k(){ return 0; } } const s = new S(); const t = new (S());").is_empty());
}

// Ruling R-30r: generated names must be unique across classes too.
#[test]
fn generated_names_colliding_across_classes_refuse() {
    let collision = kali_common::class_generated_name_collision_message;
    let b1 = messages("class A { constructor(){ this.n = 1; } _x(){ return 1; } } class A_ { constructor(){ this.n = 2; } x(){ return 2; } } new A(); new A_();");
    assert!(b1.contains(&collision("A___x", "A")) && b1.contains(&collision("A___x", "A_")), "{b1:?}");
    let b3 = messages("class A { constructor(){ this.n = 1; } _new(){ return 1; } } class A_ { constructor(){ this.n = 2; } } new A(); new A_();");
    assert!(b3.contains(&collision("A___new", "A")) && b3.contains(&collision("A___new", "A_")), "{b3:?}");
    // (A method spelled `new` never reaches a plan: the parser skips that member.)
    // Two classes sharing a method name generate distinct names.
    assert!(messages("class A { f(){ return 1; } } class B { f(){ return 2; } } new A(); new B();").is_empty());
}

#[test]
fn a_function_expression_or_arrow_id_or_import_local_counts_as_spelled() {
    let collision = kali_common::class_generated_name_collision_message;
    assert_eq!(messages("class C { m(){} } const f = function C__m(){ return 1; }; new C();"), [collision("C__m", "C")]);
    assert_eq!(messages("import { C__new } from \"./x\"; class C { m(){} } new C();"), [collision("C__new", "C")]);
    assert_eq!(messages("import C__m from \"./x\"; class C { m(){} } new C();"), [collision("C__m", "C")]);
}
