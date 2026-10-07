//! The JavaScript NumericLiteral grammar, pinned against what node evaluates.
//!
//! Every `Some` value below was checked with `node -p` (v26.10.0); every
//! `None` is a spelling node refuses with a SyntaxError.
use super::*;

fn number(text: &str) -> f64 {
    match parse_js_numeric_literal(text) {
        Some(NumericLiteral::Number(value)) => value,
        other => panic!("{text}: expected a Number, got {other:?}"),
    }
}

#[test]
fn decimal_spellings_keep_their_values() {
    assert_eq!(number("0"), 0.0);
    assert_eq!(number("42"), 42.0);
    assert_eq!(number("42.5"), 42.5);
    assert_eq!(number("1e5"), 1e5);
    assert_eq!(number("4.84e+00"), 4.84);
    assert_eq!(number("2E-3"), 2e-3);
    assert_eq!(number("0.1"), 0.1);
    assert_eq!(number("1e400"), f64::INFINITY);
}

#[test]
fn prefixed_integers_read_in_their_radix() {
    assert_eq!(number("0xff"), 255.0);
    assert_eq!(number("0XFF"), 255.0);
    assert_eq!(number("0xedb88320"), 3_988_292_384.0);
    assert_eq!(number("0b101"), 5.0);
    assert_eq!(number("0B11"), 3.0);
    assert_eq!(number("0o17"), 15.0);
    assert_eq!(number("0O7"), 7.0);
    assert_eq!(number("0x0"), 0.0);
}

#[test]
fn prefixed_integers_past_2_pow_53_round_to_nearest_even() {
    // node: 0x20000000000001 === 9007199254740992 (ties to even, down)
    assert_eq!(number("0x20000000000001"), 9_007_199_254_740_992.0);
    // node: 0x20000000000003 === 9007199254740996 (ties to even, up)
    assert_eq!(number("0x20000000000003"), 9_007_199_254_740_996.0);
    // 40 hex digits overflow a u128: node prints 1.461501637330903e+48
    assert_eq!(
        number("0xffffffffffffffffffffffffffffffffffffffff"),
        1.461_501_637_330_903e48
    );
    // 0x1000...0080...0001: the low `1` sits far below the u128 window. It
    // still decides the round-off. node prints 9.134385233318145e+46.
    assert_eq!(
        number("0x1000000000000080000000000000000000000001"),
        9.134_385_233_318_145e46
    );
}

#[test]
fn separators_sit_between_digits() {
    assert_eq!(number("1_000"), 1000.0);
    assert_eq!(number("1_000.5_5"), 1000.55);
    assert_eq!(number("1e1_0"), 1e10);
    assert_eq!(number("0xff_ff"), 65535.0);
    assert_eq!(number("0b1_0"), 2.0);
}

#[test]
fn legacy_octal_and_non_octal_decimal_follow_sloppy_mode() {
    // Register entry R-58: `042` is 34, not 42.
    assert_eq!(number("042"), 34.0);
    assert_eq!(number("00"), 0.0);
    assert_eq!(number("0777"), 511.0);
    // A leading zero followed by an 8 or 9 is decimal (NonOctalDecimalIntegerLiteral).
    assert_eq!(number("08"), 8.0);
    assert_eq!(number("019"), 19.0);
    assert_eq!(number("08.5"), 8.5);
    assert_eq!(number("09e1"), 90.0);
}

#[test]
fn bigints_report_decimal_digits_or_their_prefix() {
    assert_eq!(
        parse_js_numeric_literal("42n"),
        Some(NumericLiteral::BigInt("42".to_string()))
    );
    assert_eq!(
        parse_js_numeric_literal("0n"),
        Some(NumericLiteral::BigInt("0".to_string()))
    );
    assert_eq!(
        parse_js_numeric_literal("1_000n"),
        Some(NumericLiteral::BigInt("1000".to_string()))
    );
    assert_eq!(
        parse_js_numeric_literal("0xffn"),
        Some(NumericLiteral::NonDecimalBigInt)
    );
    assert_eq!(
        parse_js_numeric_literal("0b1n"),
        Some(NumericLiteral::NonDecimalBigInt)
    );
}

#[test]
fn malformed_spellings_are_refused() {
    for text in [
        "", "0x", "0b", "0o", "0xg", "0b2", "0o8", "0x_1", "0x1_", "1_", "1__0", "_1", "0_1",
        "1._5", "1_.5", "1e_1", "1_e1", "1e+_1", "042_1", "08_1", "042n", "00n", "08n", "1.5n",
        "1e5n", "042.5", "042e1", "0xffg", "3in", "1e",
    ] {
        assert_eq!(parse_js_numeric_literal(text), None, "text: {text:?}");
    }
}
