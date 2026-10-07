use super::*;
use crate::build::block_scope_rename::test_support::parse;
use kali_ast::{Expression, LiteralValue, Statement};

fn applied(source: &str) -> (Vec<String>, Vec<Statement>) {
    let mut statements = parse(source);
    let refusals = apply_default_params(&mut statements, false);
    (refusals, statements)
}

/// The arguments of the `n`-th call to `name`, in source order, found by a
/// walk over the rewritten program.
fn call_args(statements: &mut [Statement], name: &str) -> Vec<Vec<Expression>> {
    #[derive(Default)]
    struct Calls {
        name: String,
        found: Vec<Vec<Expression>>,
    }
    impl Hooks for Calls {
        fn enter(&mut self, _: ScopeKind, _: Option<&str>) {}
        fn exit(&mut self) {}
        fn bind(&mut self, _: &mut String, _: BindKind) {}
        fn reference(&mut self, _: &mut String) {}
        fn call_expression(&mut self, call: &mut kali_ast::CallExpression) {
            if matches!(&call.callee, Expression::Identifier(n) if *n == self.name) {
                self.found.push(call.args.clone());
            }
        }
    }
    let mut calls = Calls {
        name: name.to_string(),
        ..Calls::default()
    };
    walk::walk_program(statements, &mut calls);
    calls.found
}

fn number(value: f64) -> Expression {
    Expression::Literal(LiteralValue::Number(value))
}

fn declaration<'s>(statements: &'s [Statement], name: &str) -> &'s kali_ast::FunctionDeclaration {
    statements
        .iter()
        .find_map(|statement| match statement {
            Statement::FunctionDeclaration(f) if f.name == name => Some(f),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no declaration `{name}` in {statements:?}"))
}

#[test]
fn an_omitted_trailing_argument_gets_its_default() {
    let (refusals, mut got) = applied("function f(a, b = 2) { return a + b; } f(1);");
    assert!(refusals.is_empty(), "{refusals:?}");
    assert_eq!(
        call_args(&mut got, "f"),
        vec![vec![number(1.0), number(2.0)]]
    );
    assert!(declaration(&got, "f").defaults.is_empty());
}

#[test]
fn every_omitted_default_is_appended_in_order() {
    let (refusals, mut got) = applied("function f(a = 1, b = 2) { return a + b; } f(); f(5);");
    assert!(refusals.is_empty(), "{refusals:?}");
    assert_eq!(
        call_args(&mut got, "f"),
        vec![
            vec![number(1.0), number(2.0)],
            vec![number(5.0), number(2.0)]
        ]
    );
}

#[test]
fn a_literal_undefined_is_replaced() {
    let (refusals, mut got) = applied("function f(a, b = 2) { return a + b; } f(1, undefined);");
    assert!(refusals.is_empty(), "{refusals:?}");
    assert_eq!(
        call_args(&mut got, "f"),
        vec![vec![number(1.0), number(2.0)]]
    );
}

#[test]
fn a_middle_literal_undefined_is_replaced() {
    let (refusals, mut got) =
        applied("function f(a, b = 2, c = 3) { return a + b + c; } f(1, undefined, 9);");
    assert!(refusals.is_empty(), "{refusals:?}");
    assert_eq!(
        call_args(&mut got, "f"),
        vec![vec![number(1.0), number(2.0), number(9.0)]]
    );
}

#[test]
fn void_of_a_literal_is_replaced() {
    let (refusals, mut got) = applied("function f(a = 7) { return a; } f(void 0);");
    assert!(refusals.is_empty(), "{refusals:?}");
    assert_eq!(call_args(&mut got, "f"), vec![vec![number(7.0)]]);
}

#[test]
fn a_non_literal_argument_passes_through() {
    let (refusals, mut got) = applied("function f(a = 7) { return a; } let x = 3; f(x);");
    assert!(refusals.is_empty(), "{refusals:?}");
    assert_eq!(
        call_args(&mut got, "f"),
        vec![vec![Expression::Identifier("x".to_string())]]
    );
}

#[test]
fn a_recursive_call_is_filled() {
    let (refusals, mut got) = applied(
        "function f(n, acc = 0) { if (n === 0) return acc; return f(n - 1, acc + n); } f(3); f(2);",
    );
    assert!(refusals.is_empty(), "{refusals:?}");
    let calls = call_args(&mut got, "f");
    assert_eq!(calls[1], vec![number(3.0), number(0.0)]);
    assert_eq!(calls[2], vec![number(2.0), number(0.0)]);
}

#[test]
fn a_call_before_the_declaration_is_filled() {
    let (refusals, mut got) = applied("f(); function f(a = 1) { return a; }");
    assert!(refusals.is_empty(), "{refusals:?}");
    assert_eq!(call_args(&mut got, "f"), vec![vec![number(1.0)]]);
}

#[test]
fn a_nested_declaration_is_collected_and_filled() {
    let (refusals, mut got) = applied(
        "function outer() { function inner(a = 4) { return a; } return inner(); } outer();",
    );
    assert!(refusals.is_empty(), "{refusals:?}");
    assert_eq!(call_args(&mut got, "inner"), vec![vec![number(4.0)]]);
}

#[test]
fn a_template_string_default_is_a_literal() {
    let (refusals, _) = applied("function f(a = `x`) { return a; } f();");
    assert!(refusals.is_empty(), "{refusals:?}");
}

#[test]
fn a_program_without_defaults_is_untouched() {
    let source = "function f(a, b) { return a + b; } const g = (x) => f(x, 1); g(2);";
    let (refusals, got) = applied(source);
    assert!(refusals.is_empty(), "{refusals:?}");
    assert_eq!(got, parse(source));
}

fn refused(source: &str) -> Vec<String> {
    let mut statements = parse(source);
    let before = statements.clone();
    let refusals = apply_default_params(&mut statements, false);
    assert!(!refusals.is_empty(), "{source}: expected a refusal");
    assert_eq!(
        statements, before,
        "{source}: a refused program must be left unchanged"
    );
    refusals
}

#[test]
fn a_non_literal_default_is_refused() {
    assert_eq!(
        refused("function f(a, b = a * 2) { return b; } f(1);"),
        vec![kali_common::default_param_not_literal_message("f", "b")]
    );
}

#[test]
fn an_object_or_array_default_is_refused() {
    assert_eq!(
        refused("function f(o = {}) { return 1; } f();"),
        vec![kali_common::default_param_composite_message("f", "o")]
    );
    assert_eq!(
        refused("function f(a, xs = []) { return a; } f(1);"),
        vec![kali_common::default_param_composite_message("f", "xs")]
    );
}

#[test]
fn an_async_or_generator_defaulted_function_is_refused() {
    assert_eq!(
        refused("async function f(a = 1) { return a; } f();"),
        vec![kali_common::default_param_async_or_generator_message("f")]
    );
    assert_eq!(
        refused("function* f(a = 1) { yield a; } f();"),
        vec![kali_common::default_param_async_or_generator_message("f")]
    );
}

#[test]
fn a_value_use_is_refused() {
    for source in [
        "function f(a = 1) { return a; } const g = f; g();",
        "function f(a = 1) { return a; } [1].map(f);",
        "function f(a = 1) { return a; } f.call(null);",
        "function f(a = 1) { return a; } console.log(typeof f);",
        "function f(a = 1) { return a; } f?.();",
    ] {
        assert_eq!(
            refused(source),
            vec![kali_common::default_param_value_use_message("f")],
            "{source}"
        );
    }
}

#[test]
fn an_export_specifier_is_refused() {
    assert_eq!(
        refused("function f(a = 1) { return a; } export { f };"),
        vec![kali_common::default_param_exported_message("f")]
    );
}

/// The parser does not yet produce a spread call argument (`f(...xs)` drops
/// the call), so the AST is built by hand: parse `f(xs)` and wrap the argument.
#[test]
fn a_spread_call_is_refused() {
    let mut statements = parse("function f(a = 1) { return a; } const xs = [2]; f(xs);");
    let Some(Statement::ExpressionStatement(statement)) = statements.last_mut() else {
        panic!("{statements:?}")
    };
    let Expression::CallExpression(call) = &mut *statement.expression else {
        panic!("{statement:?}")
    };
    let argument = call.args.remove(0);
    call.args.push(Expression::SpreadElement(Box::new(
        kali_ast::SpreadElement { argument },
    )));
    let before = statements.clone();
    assert_eq!(
        apply_default_params(&mut statements, false),
        vec![kali_common::default_param_spread_call_message("f")]
    );
    assert_eq!(statements, before);
}

#[test]
fn an_omitted_argument_without_a_default_is_refused() {
    assert_eq!(
        refused("function f(a = 1, b) { return a; } f();"),
        vec![kali_common::default_param_omitted_argument_message(
            "f", "b"
        )]
    );
}

#[test]
fn eval_compat_refuses_any_defaulted_function() {
    let mut statements = parse("function f(a = 1) { return a; } f();");
    let before = statements.clone();
    assert_eq!(
        apply_default_params(&mut statements, true),
        vec![kali_common::default_param_eval_refused_message().to_string()]
    );
    assert_eq!(statements, before);
}

#[test]
fn eval_compat_without_defaults_is_untouched() {
    let mut statements = parse("function f(a) { return a; } f(1);");
    assert!(apply_default_params(&mut statements, true).is_empty());
}

#[test]
fn a_same_name_binding_elsewhere_is_refused() {
    for source in [
        "function h() { function f(a = 41) { return a; } return f(); } \
         function g() { const f = (a) => a; return f(undefined); } \
         console.log(h(), g());",
        "function h() { function f(a = 41) { return a; } return f(); } \
         function g(f) { return f; } console.log(h(), g(3));",
    ] {
        assert_eq!(
            refused(source),
            vec![kali_common::default_param_rebound_name_message("f")],
            "{source}"
        );
    }
}

#[test]
fn sibling_nested_defaulted_functions_are_filled() {
    // The pass runs after the rename, which spells the two `f`s differently.
    let mut got = parse(
        "function h() { function f(a = 1) { return a; } return f(); } \
         function g() { function f(a = 2) { return a; } return f(); } h(); g();",
    );
    crate::build::block_scope_rename::rename_block_scoped_bindings(&mut got);
    let refusals = apply_default_params(&mut got, false);
    assert!(refusals.is_empty(), "{refusals:?}");
    let mut seen: Vec<Expression> = Vec::new();
    #[derive(Default)]
    struct Names(Vec<String>);
    impl Hooks for Names {
        fn enter(&mut self, _: ScopeKind, _: Option<&str>) {}
        fn exit(&mut self) {}
        fn bind(&mut self, name: &mut String, kind: BindKind) {
            if kind == BindKind::FunctionDecl {
                self.0.push(name.clone());
            }
        }
        fn reference(&mut self, _: &mut String) {}
    }
    let mut names = Names::default();
    walk::walk_program(&mut got, &mut names);
    assert_eq!(names.0.iter().filter(|n| n.starts_with('f')).count(), 2);
    assert_ne!(names.0[0], names.0[1], "{:?}", names.0);
    for name in names.0.iter().filter(|n| n.starts_with('f')) {
        seen.extend(call_args(&mut got, name).into_iter().flatten());
    }
    assert_eq!(seen, vec![number(1.0), number(2.0)]);
}

#[test]
fn an_unrelated_binding_does_not_block_the_fill() {
    let (refusals, mut got) = applied("function f(a = 1) { return a; } const g = 2; f();");
    assert!(refusals.is_empty(), "{refusals:?}");
    assert_eq!(call_args(&mut got, "f"), vec![vec![number(1.0)]]);
}
