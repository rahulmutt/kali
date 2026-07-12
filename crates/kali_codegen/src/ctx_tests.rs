use super::*;

#[test]
fn decode_escapes_translates_recognized_and_passes_unknown() {
    assert_eq!(decode_string_escapes(r"a\tb"), "a\tb");
    assert_eq!(decode_string_escapes(r"c\nd"), "c\nd");
    assert_eq!(decode_string_escapes(r"e\\f"), r"e\f");
    assert_eq!(decode_string_escapes(r"\q"), r"\q"); // unknown passed through (lexer already rejected)
}

#[test]
fn delete_statement_survives_to_lir_as_a_delete_unary() {
    // Stage 2 (throw-fallout): the parser previously had NO
    // `TokenType::Delete` arm — the token was swallowed and `delete r.b;`
    // reached LIR as a bare member-read statement, making every downstream
    // "delete" arm dead code (the same historical bug the `typeof` comment
    // in kali_parser::expression::parse_unary_expression documents).
    let program = crate::test_support::parse_and_lower_lir("const r = { a: 1 };\ndelete r.a;");
    let found = program.nodes.iter().any(|n| {
        n.kind == LirNodeKind::Value && n.text.as_deref() == Some("delete") && n.children.len() == 1
    });
    assert!(
        found,
        "no Value(\"delete\") node in LIR: {:#?}",
        program.nodes
    );
}
