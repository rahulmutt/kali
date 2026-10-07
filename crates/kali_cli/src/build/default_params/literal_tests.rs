use super::*;
use crate::build::block_scope_rename::test_support::parse;
use kali_ast::Statement;

/// The default of `b` in `function f(b = <default>) {}`.
fn default_of(default: &str) -> Expression {
    let statements = parse(&format!("function f(b = {default}) {{ return 1; }}"));
    let Statement::FunctionDeclaration(f) = &statements[0] else {
        panic!("{statements:?}")
    };
    *f.defaults[0].clone().expect("a default")
}

#[test]
fn scalar_literal_defaults_are_admitted() {
    for default in [
        "0", "2.5", "-1", "+3", "\"x\"", "'y'", "`z`", "true", "false", "null", "0n",
    ] {
        assert_eq!(
            classify_default(&default_of(default)),
            DefaultKind::Scalar,
            "{default}"
        );
    }
}

#[test]
fn object_and_array_defaults_are_composite() {
    for default in ["[]", "[1, 2]", "{}", "{ a: 1 }", "[[1], { c: {} }]"] {
        assert_eq!(
            classify_default(&default_of(default)),
            DefaultKind::Composite,
            "{default}"
        );
    }
}

#[test]
fn every_other_default_is_not_a_literal() {
    for default in [
        "x",
        "1 + 2",
        "-x",
        "!true",
        "f()",
        "`a${1}`",
        "() => 1",
        "undefined",
        "void 0",
    ] {
        assert_eq!(
            classify_default(&default_of(default)),
            DefaultKind::Other,
            "{default}"
        );
    }
}
