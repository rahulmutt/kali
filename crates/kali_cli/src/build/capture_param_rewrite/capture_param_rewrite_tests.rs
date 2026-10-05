use super::*;
use crate::build::block_scope_rename::test_support::parse;

fn rewritten(source: &str) -> (usize, Vec<Statement>) {
    let mut statements = parse(source);
    let n = rewrite_captured_params(&mut statements);
    (n, statements)
}

#[test]
fn a_captured_parameter_becomes_a_let_from_the_renamed_parameter() {
    let (n, got) = rewritten("function f(k){ const g=()=>k; return g(); }");
    assert_eq!(n, 1);
    let Statement::FunctionDeclaration(f) = &got[0] else {
        panic!("{got:?}")
    };
    assert_eq!(f.params, vec!["k{p}".to_string()]);
    let Statement::VariableDeclaration(d) = &f.body.body[0] else {
        panic!("{:?}", f.body.body)
    };
    assert_eq!(d.kind, "let");
    assert_eq!(d.declarations[0].id, "k");
    assert_eq!(
        d.declarations[0].init,
        Some(Expression::Identifier("k{p}".into()))
    );
}

#[test]
fn an_uncaptured_parameter_is_untouched() {
    let source = "function f(k){ return k + 1; }";
    let (n, got) = rewritten(source);
    assert_eq!(n, 0);
    assert_eq!(got, parse(source));
}

#[test]
fn an_arrow_with_an_expression_body_gets_a_block() {
    let (n, got) = rewritten("const f=(k)=>()=>k;");
    assert_eq!(n, 1);
    let Statement::VariableDeclaration(d) = &got[0] else {
        panic!()
    };
    let Some(Expression::FunctionExpression(f)) = &d.declarations[0].init else {
        panic!("{:?}", d.declarations[0].init)
    };
    assert!(f.is_arrow);
    assert_eq!(f.params[0].name, "k{p}");
    let body = &f.body.as_ref().expect("block").body;
    assert!(matches!(&body[0], Statement::VariableDeclaration(_)));
    assert!(matches!(&body[1], Statement::ReturnStatement(_)));
}

#[test]
fn methods_and_function_expressions_are_rewritten() {
    let (n, _) = rewritten(
        "class C { m(k){ const g=()=>k; return g(); } } const h=function(j){ return ()=>j; };",
    );
    assert_eq!(n, 2);
}

#[test]
fn a_parameter_shadowed_in_the_closure_is_not_captured() {
    let (n, _) = rewritten("function f(k){ const g=(k)=>k; return g(1); }");
    assert_eq!(n, 0);
}

#[test]
fn the_rewrite_is_idempotent() {
    let mut statements = parse("function f(k){ const g=()=>k; return g(); }");
    assert_eq!(rewrite_captured_params(&mut statements), 1);
    assert_eq!(rewrite_captured_params(&mut statements), 0);
}

#[test]
fn frames_are_counted_in_the_walks_enter_order() {
    // `walk_class_body` enters methods before field initializers, whatever
    // their source order; pass B must count the same way or it rewrites the
    // wrong frame.
    let (n, got) = rewritten(
        "class C { x = (a)=>()=>a; m(k){ const g=()=>k; return g(); } n(j){ return j; } }",
    );
    assert_eq!(n, 2);
    let Statement::ClassDeclaration(c) = &got[0] else {
        panic!("{got:?}")
    };
    let params: Vec<_> = c.body.methods.iter().map(|m| m.params.clone()).collect();
    assert_eq!(
        params,
        vec![vec!["k{p}".to_string()], vec!["j".to_string()]]
    );
    let Some(Expression::FunctionExpression(f)) = &c.body.fields[0].value else {
        panic!("{:?}", c.body.fields[0].value)
    };
    assert_eq!(f.params[0].name, "a{p}");
}
