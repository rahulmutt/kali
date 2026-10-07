//! Default parameters (default-parameters spec §3.1, A-2, A-3).

use super::*;
use kali_ast::{Expression, LiteralValue};

fn parse_ok(source: &str) -> Vec<Statement> {
    let tokens = lex(source);
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert!(
        output.diagnostics.is_empty(),
        "{source}: {:?}",
        output.diagnostics
    );
    output.statements
}

fn diagnostics(source: &str) -> Vec<String> {
    let tokens = lex(source);
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    parser
        .parse(None)
        .diagnostics
        .into_iter()
        .map(|d| d.message)
        .collect()
}

fn declaration(statements: &[Statement]) -> &kali_ast::FunctionDeclaration {
    match &statements[0] {
        Statement::FunctionDeclaration(f) => f,
        other => panic!("expected a FunctionDeclaration, got {other:?}"),
    }
}

#[test]
fn a_declaration_keeps_its_defaults_index_aligned() {
    let statements = parse_ok("function f(a, b = 2, c = \"x\") { return a; }");
    let f = declaration(&statements);
    assert_eq!(f.params, vec!["a", "b", "c"]);
    assert_eq!(f.defaults.len(), 3);
    assert!(f.defaults[0].is_none());
    assert_eq!(
        f.defaults[1].as_deref(),
        Some(&Expression::Literal(LiteralValue::Number(2.0)))
    );
    assert_eq!(
        f.defaults[2].as_deref(),
        Some(&Expression::Literal(LiteralValue::String(
            "\"x\"".to_string()
        )))
    );
}

#[test]
fn a_declaration_without_defaults_has_an_empty_vector() {
    let statements = parse_ok("function f(a, b) { return a; }");
    assert!(declaration(&statements).defaults.is_empty());
    // An all-`None` vector is cleared: the invariant is empty or params.len().
    let statements = parse_ok("function f(a, b = 1) { return a; }");
    let f = declaration(&statements);
    assert!(f.defaults.is_empty() || f.defaults.len() == f.params.len());
}

#[test]
fn a_typed_parameter_keeps_its_default() {
    let statements = parse_ok("function f(a: number, b: number = 3) { return a; }");
    let f = declaration(&statements);
    assert_eq!(f.params, vec!["a", "b"]);
    assert_eq!(
        f.defaults[1].as_deref(),
        Some(&Expression::Literal(LiteralValue::Number(3.0)))
    );
}

#[test]
fn a_plain_parameter_may_follow_a_defaulted_one() {
    let statements = parse_ok("function f(a = 1, b) { return a; }");
    let f = declaration(&statements);
    assert!(f.defaults[0].is_some());
    assert!(f.defaults[1].is_none());
}

#[test]
fn a_non_literal_default_is_parsed_and_left_to_the_pass() {
    let statements = parse_ok("function f(a, b = a * 2) { return b; }");
    assert!(matches!(
        declaration(&statements).defaults[1].as_deref(),
        Some(Expression::BinaryExpression(_))
    ));
}

#[test]
fn an_object_default_is_parsed() {
    let statements = parse_ok("function f(hooks = {}) { return 1; }");
    assert!(matches!(
        declaration(&statements).defaults[0].as_deref(),
        Some(Expression::ObjectExpression(_))
    ));
}

#[test]
fn defaults_on_other_function_forms_are_refused() {
    for source in [
        "const g = function (b = 5) { return b; };",
        "class C { m(b = 5) { return b; } }",
        "const g = (b = 5) => b;",
    ] {
        let messages = diagnostics(source);
        assert!(
            messages
                .iter()
                .any(|m| m.contains(kali_common::default_param_non_declaration_message())),
            "{source}: {messages:?}"
        );
    }
}

#[test]
fn an_exported_defaulted_declaration_is_refused() {
    for source in [
        "export function f(a = 1) { return a; }",
        "export async function f(a = 1) { return a; }",
        "export default function f(a = 1) { return a; }",
    ] {
        let messages = diagnostics(source);
        assert!(
            messages
                .iter()
                .any(|m| m == &kali_common::default_param_exported_message("f")),
            "{source}: {messages:?}"
        );
    }
}

#[test]
fn an_exported_declaration_without_defaults_is_still_accepted() {
    assert!(diagnostics("export function f(a) { return a; }").is_empty());
}

#[test]
fn a_parenthesized_assignment_is_not_an_arrow_default() {
    for source in [
        "let b = 5; const x = (b = b + 1);",
        "let b = 0; g((b = 7));",
    ] {
        let messages = diagnostics(source);
        assert!(messages.is_empty(), "{source}: {messages:?}");
    }
}

#[test]
fn an_annotated_arrow_with_a_default_is_refused() {
    for source in [
        "const f = (a = 1): number => a;",
        "const g = async (a = 1): Promise<number> => a;",
    ] {
        let messages = diagnostics(source);
        assert!(
            messages
                .iter()
                .any(|m| m.contains(kali_common::default_param_non_declaration_message())),
            "{source}: {messages:?}"
        );
    }
}

#[test]
fn an_annotated_arrow_without_a_default_is_accepted() {
    let messages = diagnostics("const h = (a): number => a;");
    assert!(messages.is_empty(), "{messages:?}");
}

#[test]
fn an_anonymous_default_export_is_named_default_in_the_refusal() {
    let messages = diagnostics("export default function (a = 1) { return a; }");
    assert!(
        messages
            .iter()
            .any(|m| m == &kali_common::default_param_exported_message("default")),
        "{messages:?}"
    );
    assert!(!messages.iter().any(|m| m.contains("``")), "{messages:?}");
}
