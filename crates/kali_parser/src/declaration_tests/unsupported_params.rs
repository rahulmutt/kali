//! Unsupported parameter shapes must fail CLOSED, never truncate the module.
//!
//! Before this suite existed, a parameter shape the parameter parsers did not
//! understand (`b = 5`, `...rest`, `{x}`, `[x]`) left the token stream parked on
//! the offending token instead of past the closing `)`. `parse_block_statement`
//! then `advance()`d unconditionally over it and absorbed EVERY REMAINING TOKEN
//! IN THE MODULE into the function body — silently, with no diagnostic and a
//! zero exit code. Every statement after such a declaration stopped running.
//!
//! The invariant these tests pin: an unsupported parameter shape produces an
//! E5506 diagnostic AND leaves the statements that follow it at module scope.

use super::*;
use kali_error::_error_codes::e5;

/// Parses `source` and returns the top-level statements plus diagnostics.
fn parse(source: &str) -> crate::ParserOutput {
    let tokens = lex(source);
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    parser.parse(None)
}

fn assert_has_feature_unavailable(output: &crate::ParserOutput, construct: &str) {
    let found = output
        .diagnostics
        .iter()
        .any(|d| d.code == Some(e5::FEATURE_UNAVAILABLE as u32));
    assert!(
        found,
        "expected an E5506 diagnostic naming `{construct}`, got {:?}",
        output.diagnostics
    );
    let names_construct = output
        .diagnostics
        .iter()
        .any(|d| d.message.contains(construct));
    assert!(
        names_construct,
        "expected a diagnostic message naming `{construct}`, got {:?}",
        output.diagnostics
    );
}

/// The trailing `console.log(...)` is the positive control: it exists ONLY to be
/// counted. If the parameter list desyncs the stream it gets absorbed into the
/// function body and the top-level statement count drops from 2 to 1.
fn assert_module_not_truncated(output: &crate::ParserOutput, source: &str) {
    assert_eq!(
        output.statements.len(),
        2,
        "module truncated: the statement after the declaration was absorbed \
         into the function body.\nsource: {source}\nstatements: {:?}",
        output.statements
    );
    assert!(
        matches!(output.statements[1], Statement::ExpressionStatement(_)),
        "expected the trailing call to survive at module scope, got {:?}",
        output.statements[1]
    );
}

fn assert_fails_closed_without_truncating(source: &str, construct: &str) {
    let output = parse(source);
    assert_has_feature_unavailable(&output, construct);
    assert_module_not_truncated(&output, source);
}

// --- function declarations -------------------------------------------------

#[test]
fn default_param_in_function_declaration_fails_closed() {
    assert_fails_closed_without_truncating(
        "function g(b = 5) { return b; }\nconsole.log(\"after\");",
        "default parameter",
    );
}

#[test]
fn default_param_after_plain_param_fails_closed() {
    assert_fails_closed_without_truncating(
        "function g(a, b = 5) { return a; }\nconsole.log(\"after\");",
        "default parameter",
    );
}

#[test]
fn multiple_default_params_fail_closed() {
    assert_fails_closed_without_truncating(
        "function g(a = 1, b = 2) { return a; }\nconsole.log(\"after\");",
        "default parameter",
    );
}

#[test]
fn rest_param_in_function_declaration_fails_closed() {
    assert_fails_closed_without_truncating(
        "function g(...r) { return 1; }\nconsole.log(\"after\");",
        "rest parameter",
    );
}

#[test]
fn object_destructured_param_fails_closed() {
    assert_fails_closed_without_truncating(
        "function g({ x }) { return 1; }\nconsole.log(\"after\");",
        "destructured parameter",
    );
}

#[test]
fn array_destructured_param_fails_closed() {
    assert_fails_closed_without_truncating(
        "function g([x]) { return 1; }\nconsole.log(\"after\");",
        "destructured parameter",
    );
}

// --- function expressions --------------------------------------------------

#[test]
fn default_param_in_function_expression_fails_closed() {
    assert_fails_closed_without_truncating(
        "const g = function (b = 5) { return b; };\nconsole.log(\"after\");",
        "default parameter",
    );
}

#[test]
fn rest_param_in_function_expression_fails_closed() {
    assert_fails_closed_without_truncating(
        "const g = function (...r) { return 1; };\nconsole.log(\"after\");",
        "rest parameter",
    );
}

// --- class methods ---------------------------------------------------------

#[test]
fn default_param_in_class_method_fails_closed() {
    assert_fails_closed_without_truncating(
        "class C { m(b = 5) { return b; } }\nconsole.log(\"after\");",
        "default parameter",
    );
}

#[test]
fn rest_param_in_class_method_fails_closed() {
    assert_fails_closed_without_truncating(
        "class C { m(...r) { return 1; } }\nconsole.log(\"after\");",
        "rest parameter",
    );
}

// --- arrows ----------------------------------------------------------------

#[test]
fn default_param_in_arrow_fails_closed() {
    let source = "const g = (b = 5) => b;\nconsole.log(\"after\");";
    let output = parse(source);
    assert_has_feature_unavailable(&output, "default parameter");
    assert_module_not_truncated(&output, source);
}

#[test]
fn rest_param_in_block_arrow_fails_closed() {
    let source = "const g = (...r) => { return 1; };\nconsole.log(\"after\");";
    let output = parse(source);
    assert_has_feature_unavailable(&output, "rest parameter");
    assert_module_not_truncated(&output, source);
}

// --- shapes that must KEEP working (no false rejection) --------------------

#[test]
fn trailing_comma_param_list_still_parses() {
    let source = "function g(a,) { return a; }\nconsole.log(\"after\");";
    let output = parse(source);
    assert!(
        output.diagnostics.is_empty(),
        "trailing commas are supported and must not be rejected: {:?}",
        output.diagnostics
    );
    assert_module_not_truncated(&output, source);
    let Statement::FunctionDeclaration(decl) = &output.statements[0] else {
        panic!(
            "expected FunctionDeclaration, got {:?}",
            output.statements[0]
        );
    };
    assert_eq!(decl.params, vec!["a".to_string()]);
}

#[test]
fn plain_params_still_parse() {
    let source = "function g(a, b) { return a + b; }\nconsole.log(\"after\");";
    let output = parse(source);
    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_module_not_truncated(&output, source);
    let Statement::FunctionDeclaration(decl) = &output.statements[0] else {
        panic!(
            "expected FunctionDeclaration, got {:?}",
            output.statements[0]
        );
    };
    assert_eq!(decl.params, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn empty_param_list_still_parses() {
    let source = "function g() { return 1; }\nconsole.log(\"after\");";
    let output = parse(source);
    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        output.diagnostics
    );
    assert_module_not_truncated(&output, source);
}
