use super::scopes::{BindingId, Resolved, Scopes};
use super::walk::{walk, Cx, Pos, Visitor};
use crate::test_support::parse_statements;
use kali_ast::Expression;

fn resolutions(src: &str, name: &str) -> Vec<Resolved> {
    let mut stmts = parse_statements(src);
    let scopes = Scopes::build(&mut stmts);
    struct R<'a> { scopes: &'a Scopes, name: &'a str, out: Vec<Resolved> }
    impl Visitor for R<'_> {
        fn expr(&mut self, expr: &mut Expression, _: &Pos, cx: &Cx) {
            if matches!(expr, Expression::Identifier(n) if n == self.name) {
                self.out.push(self.scopes.resolve(self.name, cx));
            }
        }
    }
    let mut r = R { scopes: &scopes, name, out: Vec::new() };
    walk(&mut stmts, &mut r);
    r.out
}

fn binding(frame: &str, name: &str) -> Resolved {
    Resolved::Binding(BindingId { frame: frame.into(), name: name.into() })
}

#[test]
fn an_arrow_reaches_the_outer_binding() {
    let got = resolutions("const s = 1; const f = () => { s; };", "s");
    assert_eq!(got, [binding("", "s")]);
}

#[test]
fn a_parameter_shadows_the_outer_binding() {
    let got = resolutions("const s = 1; function g(s){ return s; } s;", "s");
    assert_eq!(got, [binding("g", "s"), binding("", "s")]);
}

#[test]
fn a_name_declared_twice_in_one_frame_is_ambiguous() {
    let got = resolutions("function g(){ { const s = 1; s; } { const s = 2; s; } }", "s");
    assert_eq!(got, [Resolved::Ambiguous, Resolved::Ambiguous]);
}

#[test]
fn an_undeclared_name_is_free() {
    assert_eq!(resolutions("console.log(1);", "console"), [Resolved::Free]);
}

#[test]
fn spelled_names_include_params_and_classes() {
    let mut stmts = parse_statements("class C { m(__this){ return q; } }");
    let scopes = Scopes::build(&mut stmts);
    assert!(scopes.spelled().contains("__this"));
    assert!(scopes.spelled().contains("C"));
    assert!(scopes.spelled().contains("q"));
}
