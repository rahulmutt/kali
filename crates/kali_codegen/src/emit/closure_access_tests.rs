use crate::*;

#[test]
fn integer_literals_that_i64_parsing_rejects_are_handle_interned() {
    for text in ["1e20", "1e3", "1e300", "12345678901234567890"] {
        assert!(
            FunctionEmitter::numeric_literal_is_handle_interned(text),
            "{text}"
        );
    }
}

#[test]
fn plain_integers_floats_and_bigints_are_not_handle_interned() {
    for text in [
        "0",
        "3",
        "-3",
        "9007199254740993",
        "2.5",
        "1e-3",
        "7n",
        "true",
        "s",
    ] {
        assert!(
            !FunctionEmitter::numeric_literal_is_handle_interned(text),
            "{text}"
        );
    }
}
