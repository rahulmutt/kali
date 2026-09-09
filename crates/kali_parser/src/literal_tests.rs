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

/// A `+`/`-` unary over a STRING literal declines, even when Rust's float
/// parser would read the string. This is register entry R-59's exact shape:
/// the arm used to render its recursive result and re-parse that TEXT with
/// `str::parse::<f64>()`, which accepts `inf`, `infinity` and `nan`
/// case-insensitively while JavaScript's `ToNumber` does not, so `o[+"inf"]`
/// fabricated the name `Infinity` and read a real, wrong property at exit 0.
/// Measured at `ff8567e7f4` against node v26.8.1 on
/// `const o = {Infinity: 9, NaN: 7}`: `o[+"inf"]` and `o[+"infinity"]` printed
/// 9 where node prints 7. Spec section 4.1 scopes the unary arm to recursing
/// into a LITERAL, so the fix is to fold only a NUMBER-literal source; every
/// string spelling now declines and the E5506 gate refuses it.
#[test]
fn a_unary_over_a_string_literal_declines() {
    for source in [
        "o[+\"inf\"];",
        "o[+\"infinity\"];",
        "o[+\"Infinity\"];",
        "o[-\"inf\"];",
        "o[+\"nan\"];",
        "o[+\"NaN\"];",
        "o[+\"1\"];",
        "o[+(\"1\")];",
        "o[+(0, \"1\")];",
    ] {
        assert_eq!(
            computed_member_property(source),
            None,
            "{source} must decline: its unary source is a string, not a number"
        );
    }
}

/// The numeric spellings the unary arm reads are unchanged by that narrowing,
/// including the nested forms whose recursion passes through a parenthesized,
/// sequence-last or second unary layer, and `+1e400`, which IS `Infinity` in
/// JavaScript and must keep reading the property named "Infinity".
#[test]
fn a_unary_over_a_number_literal_still_reads_its_name() {
    for (source, name) in [
        ("o[+1];", "1"),
        ("o[-1];", "-1"),
        ("o[+1e21];", "1e+21"),
        ("o[+1e400];", "Infinity"),
        ("o[-1e400];", "-Infinity"),
        ("o[+(1)];", "1"),
        ("o[-(-1)];", "1"),
        ("o[+(0, 1)];", "1"),
        ("o[-0];", "0"),
    ] {
        assert_eq!(
            computed_member_property(source).as_deref(),
            Some(name),
            "{source}"
        );
    }
}
