use super::*;

#[test]
fn decode_escapes_translates_recognized_and_passes_unknown() {
    assert_eq!(decode_string_escapes(r"a\tb"), "a\tb");
    assert_eq!(decode_string_escapes(r"c\nd"), "c\nd");
    assert_eq!(decode_string_escapes(r"e\\f"), r"e\f");
    assert_eq!(decode_string_escapes(r"\q"), r"\q"); // unknown passed through (lexer already rejected)
}
