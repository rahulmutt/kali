use super::walk::{walk, Cx, FrameKind, Pos, Visitor};
use crate::test_support::parse_statements;
use kali_ast::Expression;

#[derive(Default)]
struct Record(Vec<(String, Pos, String)>);

impl Visitor for Record {
    fn expr(&mut self, expr: &mut Expression, pos: &Pos, cx: &Cx) {
        let label = match expr {
            Expression::Identifier(n) => n.clone(),
            Expression::ThisExpression => "this".into(),
            _ => return,
        };
        self.0.push((label, pos.clone(), cx.key().to_string()));
    }
}

fn record(src: &str) -> Vec<(String, Pos, String)> {
    let mut stmts = parse_statements(src);
    let mut r = Record::default();
    walk(&mut stmts, &mut r);
    r.0
}

#[test]
fn positions_follow_the_table() {
    let got = record("const s = a; s = b; f(c, d); o.m(e); p.f = q; r.n++; return2(t[u]);");
    let want = [
        ("a", Pos::BindingInit("s".into())),
        ("s", Pos::AssignTarget),
        ("b", Pos::BindingAssign("s".into())),
        ("f", Pos::Callee),
    ];
    for (i, (name, pos)) in want.iter().enumerate() {
        assert_eq!(
            (&got[i].0, &got[i].1),
            (&name.to_string(), pos),
            "item {i}: {got:?}"
        );
    }
    assert!(got
        .iter()
        .any(|(n, p, _)| n == "c" && matches!(p, Pos::CallArg { index: 0, .. })));
    assert!(got.iter().any(|(n, p, _)| n == "o"
        && *p
            == Pos::MemberObject {
                property: Some("m".into()),
                call: true,
                write: false
            }));
    assert!(got.iter().any(|(n, p, _)| n == "p"
        && *p
            == Pos::MemberObject {
                property: Some("f".into()),
                call: false,
                write: true
            }));
    assert!(got.iter().any(|(n, p, _)| n == "r"
        && *p
            == Pos::MemberObject {
                property: Some("n".into()),
                call: false,
                write: true
            }));
    assert!(got
        .iter()
        .any(|(n, p, _)| n == "u" && *p == Pos::Other("a computed key")));
}

#[test]
fn frames_key_methods_constructors_and_arrows() {
    let got = record(
        "class C { n = this.k; constructor(){ this.k = 1; } m(){ const f = () => { return this; }; return this; } }",
    );
    let keys: Vec<_> = got
        .iter()
        .filter(|(n, _, _)| n == "this")
        .map(|(_, _, k)| k.clone())
        .collect();
    assert_eq!(keys, ["C#fields", "C#constructor", "C#m/<anon>", "C#m"]);
}

#[test]
fn this_class_skips_arrows_but_not_functions() {
    struct ThisClass(Vec<Option<String>>);
    impl Visitor for ThisClass {
        fn expr(&mut self, expr: &mut Expression, _: &Pos, cx: &Cx) {
            if matches!(expr, Expression::ThisExpression) {
                self.0.push(cx.this_class().map(str::to_string));
            }
        }
    }
    let mut stmts = parse_statements(
        "class C { m(){ const a = () => { return this; }; const g = function(){ return this; }; return this; } static s(){ return this; } }",
    );
    let mut v = ThisClass(Vec::new());
    walk(&mut stmts, &mut v);
    assert_eq!(v.0, [Some("C".into()), None, Some("C".into()), None]);
}

#[test]
fn a_replacement_is_descended_into() {
    struct Replace(usize);
    impl Visitor for Replace {
        fn expr(&mut self, expr: &mut Expression, _: &Pos, _: &Cx) {
            if let Expression::Identifier(n) = expr {
                if n == "x" {
                    *expr = crate::test_support::parse_statements("const z = [y, y];")
                        .into_iter()
                        .find_map(|s| match s {
                            kali_ast::Statement::VariableDeclaration(d) => {
                                d.declarations.into_iter().next().and_then(|d| d.init)
                            }
                            _ => None,
                        })
                        .unwrap();
                } else if n == "y" {
                    self.0 += 1;
                }
            }
        }
    }
    let mut stmts = parse_statements("f(x);");
    let mut v = Replace(0);
    walk(&mut stmts, &mut v);
    assert_eq!(v.0, 2);
}

#[test]
fn frame_kinds_are_reported() {
    struct Kinds(Vec<FrameKind>);
    impl Visitor for Kinds {
        fn enter_frame(&mut self, cx: &Cx, _: &[String]) {
            self.0.push(cx.frames.last().unwrap().kind.clone());
        }
    }
    let mut stmts = parse_statements(
        "async function f(){} const g = function(){}; class C { constructor(){} static s(){} }",
    );
    let mut v = Kinds(Vec::new());
    walk(&mut stmts, &mut v);
    assert_eq!(
        v.0,
        [
            FrameKind::Function {
                is_async_or_generator: true,
                is_expression: false
            },
            FrameKind::Function {
                is_async_or_generator: false,
                is_expression: true
            },
            FrameKind::Constructor { class: "C".into() },
            FrameKind::Method {
                class: "C".into(),
                method: "s".into(),
                is_static: true
            },
        ]
    );
}
