//! The JavaScript NumericLiteral grammar: one function turns a literal's source
//! text into the value JavaScript gives it.
//!
//! The lexer calls it to decide whether a token is well formed, and the parser
//! calls it to read the token's value, so the two cannot disagree. Before this
//! module the parser read the text with Rust's `str::parse::<f64>`, which has
//! no hex, binary, octal, separator or legacy-octal grammar. `042` read as
//! `42` where JavaScript says `34` (register entry R-58), and the lexer did not
//! tokenize `0x`/`0b`/`0o` or `_` at all.
//!
//! Legacy octal (`042`) and non-octal decimal (`08`) follow sloppy-mode
//! JavaScript, which is what a `.js` entry file runs in. Strict mode and
//! modules refuse both. Kali has no strict-mode notion on this path, so that
//! refusal is not implemented here.

/// The value a well-formed numeric literal denotes.
#[derive(Debug, Clone, PartialEq)]
pub enum NumericLiteral {
    Number(f64),
    /// A decimal BigInt, as its digits with separators and the `n` removed.
    BigInt(String),
    /// A well-formed `0x`/`0b`/`0o` BigInt. Its value is not computed here.
    NonDecimalBigInt,
}

/// Reads `text` as a JavaScript NumericLiteral, or `None` when JavaScript
/// refuses the spelling with a SyntaxError.
pub fn parse_js_numeric_literal(text: &str) -> Option<NumericLiteral> {
    if let Some((radix, digits)) = radix_prefix(text) {
        return match digits.strip_suffix('n') {
            Some(digits) => {
                radix_digits_value(digits, radix)?;
                Some(NumericLiteral::NonDecimalBigInt)
            }
            None => radix_digits_value(digits, radix).map(NumericLiteral::Number),
        };
    }
    if let Some(digits) = text.strip_suffix('n') {
        let plain = strip_separators(digits, |byte| byte.is_ascii_digit())?;
        let well_formed = !plain.is_empty()
            && plain.bytes().all(|byte| byte.is_ascii_digit())
            && !(plain.len() > 1 && plain.starts_with('0'));
        return well_formed.then_some(NumericLiteral::BigInt(plain));
    }
    if is_legacy_leading_zero(text) {
        return legacy_leading_zero_value(text).map(NumericLiteral::Number);
    }
    decimal_value(text).map(NumericLiteral::Number)
}

fn radix_prefix(text: &str) -> Option<(u32, &str)> {
    let rest = text.strip_prefix('0')?;
    let mut chars = rest.chars();
    let radix = match chars.next()? {
        'x' | 'X' => 16,
        'b' | 'B' => 2,
        'o' | 'O' => 8,
        _ => return None,
    };
    Some((radix, chars.as_str()))
}

/// A leading `0` followed by another digit: legacy octal or non-octal decimal.
fn is_legacy_leading_zero(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() > 1 && bytes[0] == b'0' && bytes[1].is_ascii_digit()
}

fn legacy_leading_zero_value(text: &str) -> Option<f64> {
    if text.contains('_') {
        return None;
    }
    let integer_end = text
        .bytes()
        .position(|byte| !byte.is_ascii_digit())
        .unwrap_or(text.len());
    let integer = &text[..integer_end];
    if integer.bytes().all(|byte| (b'0'..=b'7').contains(&byte)) {
        // LegacyOctalIntegerLiteral takes no fraction and no exponent.
        if integer_end != text.len() {
            return None;
        }
        return radix_digits_value(integer, 8);
    }
    // NonOctalDecimalIntegerLiteral: decimal, and it may take a fraction or
    // an exponent (`08.5`, `09e1`).
    decimal_value(text)
}

/// The value of `digits` in `radix`, rounded to the nearest `f64` with ties to
/// even, as JavaScript requires for every length of literal.
fn radix_digits_value(digits: &str, radix: u32) -> Option<f64> {
    let digits = strip_separators(digits, |byte| char::from(byte).is_digit(radix))?;
    if digits.is_empty() {
        return None;
    }
    let bits_per_digit = radix.trailing_zeros();
    let mut mantissa: u128 = 0;
    let mut dropped_bits: i32 = 0;
    let mut sticky = false;
    for ch in digits.chars() {
        let digit = ch.to_digit(radix)? as u128;
        if mantissa.leading_zeros() >= bits_per_digit {
            mantissa = (mantissa << bits_per_digit) | digit;
        } else {
            // The window is full. The digit is below every bit an f64 can
            // keep, so only whether it is nonzero matters for rounding.
            dropped_bits += bits_per_digit as i32;
            sticky |= digit != 0;
        }
    }
    // A u128 holds far more than the 54 bits rounding looks at, so folding
    // the sticky bit into its lowest bit cannot move a tie the wrong way.
    // `u128 as f64` rounds to nearest, ties to even.
    let value = (mantissa | u128::from(sticky)) as f64;
    Some(value * 2f64.powi(dropped_bits))
}

/// Decimal digits, an optional fraction and an optional exponent, with
/// separators allowed only between two digits.
fn decimal_value(text: &str) -> Option<f64> {
    let plain = strip_separators(text, |byte| byte.is_ascii_digit())?;
    if text.starts_with('0') && text.as_bytes().get(1) == Some(&b'_') {
        return None;
    }
    let valid = plain
        .bytes()
        .all(|byte| byte.is_ascii_digit() || matches!(byte, b'.' | b'e' | b'E' | b'+' | b'-'));
    if !valid || !plain.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    plain.parse::<f64>().ok()
}

/// Removes `_` separators, or `None` when one is not between two digits as
/// `is_digit` defines them for the literal's radix.
fn strip_separators(text: &str, is_digit: impl Fn(u8) -> bool) -> Option<String> {
    let bytes = text.as_bytes();
    for (index, &byte) in bytes.iter().enumerate() {
        if byte != b'_' {
            continue;
        }
        let before = index.checked_sub(1).map(|i| bytes[i]);
        let after = bytes.get(index + 1).copied();
        if !before.is_some_and(&is_digit) || !after.is_some_and(&is_digit) {
            return None;
        }
    }
    Some(text.replace('_', ""))
}

#[cfg(test)]
#[path = "numeric_literal_tests.rs"]
mod numeric_literal_tests;
