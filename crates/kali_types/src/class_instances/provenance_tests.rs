use super::classes::plan_classes;
use super::provenance::{build_env, reassociate_new, Provenance, Val};
use super::scopes::{BindingId, Scopes};
use crate::test_support::parse_statements;

fn solve(src: &str) -> (Provenance, Vec<kali_ast::Statement>) {
    let mut stmts = parse_statements(src);
    let spelled = Scopes::build(&mut stmts).spelled().clone();
    let plans = plan_classes(&mut stmts, &spelled);
    reassociate_new(&mut stmts, &plans);
    let scopes = Scopes::build(&mut stmts);
    let env = build_env(&mut stmts, &scopes, &plans);
    (Provenance::solve(&mut stmts, &env), stmts)
}

fn b(frame: &str, name: &str) -> BindingId {
    BindingId {
        frame: frame.into(),
        name: name.into(),
    }
}

#[test]
fn bindings_parameters_and_returns_resolve() {
    let (p, _) = solve(
        "class C { constructor(v){ this.v = v; } } function mk(v){ return new C(v); } function use(x){ return x.v; } const c = mk(1); use(c);",
    );
    assert_eq!(p.binding(&b("", "c")), Val::Inst("C".into()));
    assert_eq!(p.binding(&b("use", "x")), Val::Inst("C".into()));
    assert_eq!(p.returns("mk"), Val::Inst("C".into()));
}

#[test]
fn two_classes_into_one_parameter_is_unknown() {
    let (p, _) = solve(
        "class A{ constructor(){ this.n=1; } } class B{ constructor(){ this.n=2; } } function f(x){ return x.n; } f(new A()); f(new B());",
    );
    assert_eq!(p.binding(&b("f", "x")), Val::Unknown);
}

#[test]
fn an_escaped_function_has_unknown_parameters() {
    let (p, _) = solve("class C{ constructor(){ this.n=1; } } function f(x){ return x.n; } const g = f; g(new C());");
    assert_eq!(p.binding(&b("f", "x")), Val::Unknown);
}

#[test]
fn a_recursive_method_returning_this_converges() {
    let (p, _) = solve(
        "class C { constructor(){ this.n = 0; } add(x){ this.n = this.n + x; if (x > 0) { return this.add(x - 1); } return this; } } const c = new C(); const d = c.add(3);",
    );
    assert_eq!(p.binding(&b("", "d")), Val::Inst("C".into()));
}

#[test]
fn method_arguments_flow_to_method_parameters() {
    let (p, _) = solve(
        "class P{ constructor(){ this.x=1; } } class C{ constructor(){ this.n=0; } take(p){ return p.x; } } const c = new C(); c.take(new P());",
    );
    assert_eq!(p.binding(&b("C#take", "p")), Val::Inst("P".into()));
}

#[test]
fn reassociation_puts_new_at_the_chain_root() {
    use kali_ast::{Expression, Statement};
    let (_, stmts) = solve("class S { m(v){ return v + 1; } } new S().m(1);");
    let Statement::ExpressionStatement(stmt) = &stmts[1] else {
        panic!("{:?}", stmts[1])
    };
    let Expression::CallExpression(call) = stmt.expression.as_ref() else {
        panic!("{stmt:?}")
    };
    assert_eq!(call.args.len(), 1);
    let Expression::MemberExpression(member) = &call.callee else {
        panic!("{call:?}")
    };
    assert_eq!(member.property.as_deref(), Some("m"));
    let Expression::NewExpression(new) = &member.object else {
        panic!("{member:?}")
    };
    assert_eq!(new.callee, Expression::Identifier("S".into()));
    assert!(new.args.is_empty());
}

#[test]
fn a_captured_instance_resolves_inside_an_arrow() {
    let (p, _) = solve("class C{ constructor(){ this.n=0; } add(x){} } const s = new C(); const f = () => { s.add(4); }; f();");
    assert_eq!(p.binding(&b("", "s")), Val::Inst("C".into()));
}

#[test]
fn a_callback_parameter_with_unknown_call_sites_is_unknown() {
    let (p, _) = solve("class C{ constructor(){ this.n=0; } } const xs = [1]; xs.map(function cb(y){ return y; });");
    assert_eq!(p.binding(&b("cb", "y")), Val::Unknown);
}

#[test]
fn async_returns_and_bare_returns_are_not_instances() {
    let (p, _) = solve(
        "class C{ constructor(){ this.n=0; } } async function a(){ return new C(); } function g(x){ if (x) { return; } return new C(); } const r = g(1);",
    );
    assert_eq!(p.returns("a"), Val::Unknown);
    assert_eq!(p.binding(&b("", "r")), Val::Unknown);
}

#[test]
fn a_reassigned_function_is_not_a_known_function() {
    let (p, _) = solve(
        "class C{ constructor(){ this.n=0; } } function f(){ return new C(); } f = function h(){ return 1; }; const r = f();",
    );
    assert_eq!(p.binding(&b("", "r")), Val::Unknown);
}

#[test]
fn logical_and_compound_assignments_to_a_binding() {
    let (p, _) = solve(
        "class C{ constructor(){ this.n=0; } } let a = 0; a ||= new C(); let k = new C(); k += 1;",
    );
    assert_eq!(p.binding(&b("", "a")), Val::Unknown);
    assert_eq!(p.binding(&b("", "k")), Val::Unknown);
}

#[test]
fn a_shadowed_class_name_is_not_an_instance() {
    let (p, _) = solve(
        "class C{ constructor(){ this.n=0; } } new C(); function f(C){ const x = new C(); return x; }",
    );
    assert_eq!(p.binding(&b("f", "x")), Val::Unknown);
}

#[test]
fn an_export_default_function_escapes() {
    let (p, _) = solve("class C{constructor(){this.n=0;} m(){}} export default function f(x){ x.m(); } f(new C());");
    assert_eq!(p.binding(&b("f", "x")), Val::Unknown);
}
