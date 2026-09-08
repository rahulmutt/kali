use crate::test_support::lex;
use crate::Parser;
use kali_ast::{Expression, Statement};

/// Parses `<source>` (one expression statement that is a computed member
/// access) and returns the member's static property name.
fn computed_member_property(source: &str) -> Option<String> {
    let tokens = lex(source);
    let mut parser = Parser::new(kali_common::FileId::new(0), tokens);
    let output = parser.parse(None);
    assert!(
        output.diagnostics.is_empty(),
        "unexpected diagnostics for {source}: {:?}",
        output.diagnostics
    );
    let Statement::ExpressionStatement(stmt) = &output.statements[0] else {
        panic!(
            "expected an expression statement for {source}, got {:?}",
            output.statements[0]
        );
    };
    let Expression::MemberExpression(member) = stmt.expression.as_ref() else {
        panic!(
            "expected a member expression for {source}, got {:?}",
            stmt.expression
        );
    };
    assert!(
        member.computed_index.is_some(),
        "{source} must parse as a computed access"
    );
    member.property.clone()
}

#[test]
fn a_readable_index_keeps_its_name() {
    for (source, name) in [
        ("o[\"b\"];", "b"),
        ("o['b'];", "b"),
        ("o[1];", "1"),
        ("o[1.5];", "1.5"),
        ("o[(1)];", "1"),
        ("o[(0, 1)];", "1"),
        ("o[+1];", "1"),
        ("o[-1];", "-1"),
        ("o[\"\"];", ""),
    ] {
        assert_eq!(
            computed_member_property(source).as_deref(),
            Some(name),
            "{source}"
        );
    }
}

#[test]
fn an_unreadable_index_has_no_name_and_no_fallback() {
    for source in [
        "o[i];",
        "o[i + 0];",
        "o[(i)];",
        "o[(0, i)];",
        "o[+i];",
        "o[-i];",
        "o[true];",
        "o[null];",
        "o[1n];",
        "o[f()];",
        "o[\"a\" + \"b\"];",
    ] {
        assert_eq!(
            computed_member_property(source),
            None,
            "{source} must decline, not fabricate"
        );
    }
}
