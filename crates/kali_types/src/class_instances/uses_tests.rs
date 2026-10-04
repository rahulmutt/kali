use super::classes::plan_classes;
use super::provenance::{build_env, reassociate_new, Provenance};
use super::scopes::Scopes;
use super::uses::check_and_rewrite;
use crate::test_support::parse_statements;

fn run(src: &str) -> (Vec<kali_ast::Statement>, Vec<String>) {
    let mut stmts = parse_statements(src);
    let spelled = Scopes::build(&mut stmts).spelled().clone();
    let plans = plan_classes(&mut stmts, &spelled);
    reassociate_new(&mut stmts, &plans);
    let scopes = Scopes::build(&mut stmts);
    let env = build_env(&mut stmts, &scopes, &plans);
    let prov = Provenance::solve(&mut stmts, &env);
    let diags = check_and_rewrite(&mut stmts, &env, &prov);
    (stmts, diags.into_iter().map(|d| d.message).collect())
}

fn tail(src: &str, n: usize) -> Vec<kali_ast::Statement> {
    let stmts = parse_statements(src);
    stmts[stmts.len() - n..].to_vec()
}

#[test]
fn new_and_method_calls_are_rewritten() {
    let (stmts, diags) = run("class C { constructor(v){ this.n = v; } add(x){ this.n = this.n + x; } } const s = new C(1); s.add(2);");
    assert!(diags.is_empty(), "{diags:?}");
    assert_eq!(
        stmts[1..].to_vec(),
        tail("const s = C__new(1); C__add(s, 2);", 2)
    );
}

#[test]
fn this_method_calls_and_compound_assignments_are_rewritten_inside_the_class() {
    let (stmts, diags) = run("class C { constructor(){ this.n = 0; } a(){ this.n += 2; this.n++; return this.b(); } b(){ return 1; } } new C().a();");
    assert!(diags.is_empty(), "{diags:?}");
    let expected = parse_statements("class C { constructor(){ this.n = 0; } a(){ this.n = this.n + 2; this.n = this.n + 1; return C__b(this); } b(){ return 1; } } C__a(C__new());");
    assert_eq!(stmts, expected);
}

#[test]
fn same_named_methods_dispatch_per_class() {
    let (stmts, diags) = run("class A{ f(){ return 1; } } class B{ f(){ return 2; } } const a=new A(); const b=new B(); a.f(); b.f();");
    assert!(diags.is_empty(), "{diags:?}");
    assert_eq!(
        stmts[2..].to_vec(),
        tail("const a=A__new(); const b=B__new(); A__f(a); B__f(b);", 4)
    );
}

#[test]
fn positions_outside_the_allowlist_refuse() {
    let (_, d) = run("class C{ constructor(){ this.n=1; } } const c=new C(); console.log(c);");
    assert_eq!(
        d,
        [kali_common::class_instance_position_message(
            "C",
            "an argument to a call kali cannot resolve to a program function"
        )]
    );
    let (_, d) = run("class C{ constructor(){ this.n=1; } } const xs=[new C()];");
    assert_eq!(
        d,
        [kali_common::class_instance_position_message(
            "C",
            "an array element"
        )]
    );
    let (_, d) = run("class C{ constructor(){ this.n=1; } } const c=new C(); c instanceof C;");
    assert!(
        d.contains(&kali_common::class_instance_position_message(
            "C",
            "an operand of a binary operator"
        )),
        "{d:?}"
    );
    assert!(d.contains(&kali_common::class_value_message("C")), "{d:?}");
}

#[test]
fn field_rules_refuse_at_the_use() {
    let (_, d) = run("class C{ constructor(){ this.n=0; } set(){ this.m=1; } } new C().set();");
    assert_eq!(d, [kali_common::class_field_outside_set_message("C", "m")]);
    let (_, d) = run("class C{ constructor(){ this.n=1; } } const c=new C(); c.zz;");
    assert_eq!(
        d,
        [kali_common::class_field_undeclared_read_message("C", "zz")]
    );
    let (_, d) = run(
        "class C{ constructor(){ this.n=1; } get(){ return 1; } } const c=new C(); const m=c.get;",
    );
    assert!(
        d.contains(&kali_common::class_method_value_message("C", "get")),
        "{d:?}"
    );
    let (_, d) = run("class C{ constructor(){ this.n=1; } } const c=new C(); c.zork();");
    assert_eq!(
        d,
        [kali_common::unresolved_member_call_unavailable_message(
            "zork"
        )]
    );
}

#[test]
fn mixed_and_unresolved_receivers_refuse() {
    let (_, d) = run("class A{ constructor(){ this.n=1; } } class B{ constructor(){ this.n=2; } } function f(x){ return x.n; } f(new A()); f(new B());");
    assert!(
        d.contains(&kali_common::class_instance_mixed_message(
            "A",
            "parameter `x` of `f`"
        )),
        "{d:?}"
    );
    let (_, d) = run("class C{ constructor(){ this.n=1; } get(){ return this.n; } } function f(x){ return x.get(); } const g=f; g(new C());");
    assert!(
        d.contains(&kali_common::class_receiver_unresolved_message("get", "C")),
        "{d:?}"
    );
}

#[test]
fn an_array_push_next_to_a_user_push_is_left_alone() {
    let (stmts, d) = run("class Stack{ constructor(){ this.n=0; } push(v){ this.n=this.n+v; } } const s=new Stack(); const a=[1]; a.push(2); s.push(5);");
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(
        stmts[1..].to_vec(),
        tail(
            "const s=Stack__new(); const a=[1]; a.push(2); Stack__push(s, 5);",
            4
        )
    );
}

#[test]
fn an_update_in_value_position_is_left_for_the_lane_to_refuse() {
    let (stmts, d) =
        run("class C{ constructor(){ this.n=0; } inc(){ return this.n++; } } new C().inc();");
    assert!(d.is_empty(), "{d:?}");
    let expected = parse_statements(
        "class C{ constructor(){ this.n=0; } inc(){ return this.n++; } } C__inc(C__new());",
    );
    assert_eq!(stmts, expected);
}

// Ruling R-12: provenance does not track a rewritten class used as a value,
// so every non-`new` use of the class name must refuse here.
#[test]
fn the_class_as_a_value_refuses_in_every_position() {
    let (_, d) = run("class C{ constructor(v){ this.n=v; } } const K = C; new K(5); new C(1);");
    assert!(d.contains(&kali_common::class_value_message("C")), "{d:?}");
    let (_, d) =
        run("class C{ constructor(){ this.n=1; } m(){ return 1; } } new C(); C.prototype.m;");
    assert!(d.contains(&kali_common::class_value_message("C")), "{d:?}");
    let (_, d) =
        run("class C{ constructor(){ this.n=1; } } new C(); function f(k){ return k; } f(C);");
    assert!(d.contains(&kali_common::class_value_message("C")), "{d:?}");
    let (_, d) = run("class C{ constructor(){ this.n=1; } } new C(); function f(){ return C; }");
    assert!(d.contains(&kali_common::class_value_message("C")), "{d:?}");
    let (_, d) = run("class C{ constructor(){ this.n=1; } } new C(); C();");
    assert!(d.contains(&kali_common::class_value_message("C")), "{d:?}");
}

#[test]
fn a_shadowed_class_name_is_not_the_class() {
    let (stmts, d) =
        run("class C{ constructor(){ this.n=1; } } new C(); function f(C){ return C; } f(2);");
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(stmts[1..2].to_vec(), tail("C__new();", 1));
}

#[test]
fn method_values_refuse_call_apply_and_bind() {
    for tail in ["s.get.call(s);", "s.get.apply(s, []);", "s.get.bind(s);"] {
        let src = format!("class C{{ constructor(){{ this.n=1; }} get(){{ return this.n; }} }} const s=new C(); {tail}");
        let (_, d) = run(&src);
        assert!(
            d.contains(&kali_common::class_method_value_message("C", "get")),
            "{tail}: {d:?}"
        );
    }
}

#[test]
fn arguments_check_the_parameter_they_bind() {
    let (stmts, d) =
        run("class C{ constructor(v){ this.n=v; } } function f(x){ return x.n; } f(new C(1));");
    assert!(d.is_empty(), "{d:?}");
    assert_eq!(stmts[2..].to_vec(), tail("f(C__new(1));", 1));
    let (_, d) = run("class C{ constructor(){ this.n=1; } } function f(){ return 1; } f(new C());");
    assert_eq!(
        d,
        [kali_common::class_instance_position_message(
            "C",
            "an extra argument"
        )]
    );
    let (_, d) = run("class C{ constructor(){ this.n=1; } } new C(new C());");
    assert_eq!(
        d,
        [kali_common::class_instance_position_message(
            "C",
            "an extra argument"
        )]
    );
}

#[test]
fn returns_check_the_function_return() {
    let (_, d) = run("class C{ constructor(){ this.n=1; } } function f(b){ if (b) { return new C(); } return 2; } f(true);");
    assert_eq!(
        d,
        [kali_common::class_instance_mixed_message(
            "C",
            "the return value of `f`"
        )]
    );
    let (_, d) = run(
        "class C{ constructor(){ this.n=1; } } function f(){ return new C(); } const c = f(); c.n;",
    );
    assert!(d.is_empty(), "{d:?}");
}

#[test]
fn a_parenthesized_instance_refuses_once() {
    let (_, d) = run("class C{ constructor(){ this.n=1; } } const c=new C(); const xs=[(c)];");
    assert_eq!(
        d,
        [kali_common::class_instance_position_message(
            "C",
            "an array element"
        )]
    );
}

#[test]
fn a_class_named_as_a_jsx_element_refuses() {
    let (_, d) = run("class C{ constructor(){ this.n=1; } } new C(); const e = <div><C /></div>;");
    assert!(d.contains(&kali_common::class_value_message("C")), "{d:?}");
}

// Ruling R-13: a frame whose call sites are not all tracked returns `Unknown`,
// so returning an instance from it refuses.
#[test]
fn an_instance_returned_from_an_untracked_frame_refuses() {
    const CLASS: &str = "class C { constructor(){ this.n=0; } m(){} } ";
    let mk_return = kali_common::class_instance_mixed_message("C", "the return value of `mk`");
    for program in [
        "function mk(){ return new C(); } const xs=[0].map(mk); xs[0].m(); xs[0].zz = 5; console.log(xs[0]);",
        "function mk(){ return new C(); } const o={ f: mk }; o.f().m(); o.f().zz = 1;",
        "function mk(){ return new C(); } for (const x of [0].map(mk)) { x.m(); }",
        "function mk(){ return new C(); } function ap(g){ return g(); } const y=ap(mk); y.n; y.zz = 1;",
    ] {
        let (_, d) = run(&format!("{CLASS}{program}"));
        assert!(d.contains(&mk_return), "{program}: {d:?}");
    }
}

#[test]
fn a_top_level_return_names_the_program() {
    let (_, d) = run("class C { constructor(){ this.n=0; } } return new C();");
    assert_eq!(
        d,
        [kali_common::class_instance_mixed_message(
            "C",
            "the return value of the program"
        )]
    );
}

// Ruling R-14: `arguments` would hand an instance parameter out untracked.
#[test]
fn arguments_in_an_instance_taking_frame_refuses() {
    let (_, d) = run("class C { constructor(){ this.n=0; } m(){} } function f(a){ return arguments[0]; } const y=f(new C()); y.m();");
    assert!(
        d.contains(&kali_common::class_instance_position_message(
            "C",
            "the `arguments` object"
        )),
        "{d:?}"
    );
    let (_, d) = run("class C { constructor(){ this.n=0; } m(){} } function f(a){ const g = () => arguments[0]; return 1; } f(new C());");
    assert!(
        d.contains(&kali_common::class_instance_position_message(
            "C",
            "the `arguments` object"
        )),
        "{d:?}"
    );
    let (_, d) = run("class C { constructor(){ this.n=0; } } function f(a){ return arguments.length; } f(1); new C();");
    assert!(d.is_empty(), "{d:?}");
}

// Ruling R-15: a field access needs a variable receiver.
#[test]
fn a_field_access_on_a_non_variable_receiver_refuses() {
    const CLASS: &str = "class C { constructor(){ this.v=3; } } ";
    let want = kali_common::class_instance_position_message(
        "C",
        "the receiver of a field access that is not a variable",
    );
    let (_, d) = run(&format!("{CLASS}new C().v;"));
    assert!(d.contains(&want), "{d:?}");
    let (_, d) = run(&format!("{CLASS}const c = new C(); c.v;"));
    assert!(d.is_empty(), "{d:?}");
}

// Ruling R-16: an instance value inside an arrow or a function expression refuses.
#[test]
fn an_instance_inside_an_arrow_or_function_expression_refuses() {
    const CLASS: &str = "class C { constructor(){ this.n=0; } add(x){ this.n=this.n+x; } } ";
    for program in [
        "class E { constructor(){ this.n=1; } bump(){ const f = () => { this.n = 2; }; f(); } } const e = new E(); e.bump(); const c = new C();",
        "const s=new C(); const f=()=>{ s.add(1); }; f();",
        "const f = (x) => x.n; f(new C());",
        "const s=new C(); const f=function(){ s.add(1); }; f();",
    ] {
        let (_, d) = run(&format!("{CLASS}{program}"));
        assert!(d.iter().any(|m| m.contains("a value inside an arrow function or function expression")), "{program}: {d:?}");
    }
}

// Ruling R-16b: an instance captured from an enclosing function refuses.
#[test]
fn an_instance_captured_from_an_enclosing_function_refuses() {
    const CLASS: &str = "class C { constructor(){ this.n=0; } add(x){ this.n=this.n+x; } } ";
    let want = kali_common::class_instance_position_message(
        "C",
        "a value captured from an enclosing function",
    );
    for program in [
        "function main(){ const s=new C(); function g(){ s.add(1); } g(); } main();",
        "function main(){ const s=new C(); function g(){ return s; } const t=g(); t.add(4); } main();",
        "function main(){ const s=new C(); const f=()=>{ function g(){ s.add(4); } g(); }; f(); } main();",
        "const s=new C(); function g(){ s.add(1); } g();",
    ] {
        let (_, d) = run(&format!("{CLASS}{program}"));
        assert!(d.contains(&want), "{program}: {d:?}");
    }
    // A parameter used in its own function, and a local in its own frame, are not captures.
    let (_, d) = run(&format!("{CLASS}function f(p){{ p.add(1); return p.n; }} function main(){{ const s=new C(); f(s); s.add(2); }} main();"));
    assert!(d.is_empty(), "{d:?}");
}

// Ruling R-24: `typeof` of an instance field reads the slot as a number.
#[test]
fn typeof_of_an_instance_field_refuses() {
    let want =
        kali_common::class_instance_position_message("P", kali_common::CLASS_POSITION_TYPEOF_FIELD);
    for program in [
        "class P { constructor(n){ this.n = n; } } const p = new P(3); console.log(typeof p.n);",
        "class P { constructor(){ this.n = 1; } t(){ return typeof this.n; } } const p = new P(); p.t();",
        "class P { constructor(){ this.n = 1; } } const p = new P(); const t = typeof (p.n);",
    ] {
        let (_, d) = run(program);
        assert_eq!(d, std::slice::from_ref(&want), "{program}");
    }
    // `typeof` of something else, and a field read elsewhere, are not refused.
    let (_, d) = run("class P { constructor(){ this.n = 1; } } const p = new P(); const k = 2; console.log(typeof k, p.n);");
    assert!(d.is_empty(), "{d:?}");
}

// Ruling R-28: a TS wrapper around an instance would form an alias kali does not track.
#[test]
fn a_type_assertion_or_satisfies_around_an_instance_refuses() {
    let want = kali_common::class_instance_position_message(
        "C",
        kali_common::CLASS_POSITION_TYPE_ASSERTION,
    );
    for program in [
        "class C { n = 2; m(){ return this.n; } } const c = new C(); console.log((c as any).m());",
        "class C { n = 2; m(){ return this.n; } } const c = new C(); const d = c as any; console.log(d.m());",
        "class C { n = 2; } const c = new C(); const d = c satisfies C; console.log(d.n);",
        "class C { n = 2; } const d = new C() as C; console.log(d.n);",
    ] {
        let (_, d) = run(program);
        assert!(d.contains(&want), "{program}: {d:?}");
    }
    // A wrapper around a non-instance is not refused.
    let (_, d) =
        run("class C { n = 2; } const c = new C(); const k = c.n as number; console.log(k);");
    assert!(d.is_empty(), "{d:?}");
}
