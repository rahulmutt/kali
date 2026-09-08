use crate::{Lexer, TokenType};
use kali_common::FileId;
use kali_error::_error_codes::e1;

#[test]
fn test_lexer_eof() {
    let lexer = Lexer::new(FileId::new(0), String::new());
    let result = lexer.lex_all();
    assert_eq!(result.tokens.len(), 1);
    assert_eq!(result.tokens[0].kind, TokenType::Eof);
}

#[test]
fn test_lexer_function() {
    let mut lexer = Lexer::new(FileId::new(0), "function".to_string());
    let token = lexer.next_token().unwrap();
    assert_eq!(token.kind, TokenType::Function);
}

#[test]
fn test_lexer_finally() {
    let mut lexer = Lexer::new(FileId::new(0), "finally".to_string());
    let token = lexer.next_token().unwrap();
    assert_eq!(token.kind, TokenType::Finally);
}

#[test]
fn test_lexer_identifier() {
    let mut lexer = Lexer::new(FileId::new(0), "x".to_string());
    let token = lexer.next_token().unwrap();
    assert_eq!(token.kind, TokenType::Identifier);
}

#[test]
fn test_lexer_number() {
    let mut lexer = Lexer::new(FileId::new(0), "42".to_string());
    let token = lexer.next_token().unwrap();
    assert_eq!(token.kind, TokenType::NumericLiteral);
}

#[test]
fn test_lexer_decimal_number() {
    let lexer = Lexer::new(FileId::new(0), "42.5".to_string());
    let result = lexer.lex_all();
    assert_eq!(result.tokens[0].kind, TokenType::NumericLiteral);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn test_lexer_string() {
    let mut lexer = Lexer::new(FileId::new(0), "\"hello\"".to_string());
    let token = lexer.next_token().unwrap();
    assert_eq!(token.kind, TokenType::StringLiteral);
}

#[test]
fn test_lexer_plus() {
    let mut lexer = Lexer::new(FileId::new(0), "+".to_string());
    let token = lexer.next_token().unwrap();
    assert_eq!(token.kind, TokenType::Plus);
}

#[test]
fn test_lexer_compound_assignment_tokens() {
    let lexer = Lexer::new(FileId::new(0), "+= -= *= /= %= **= &&= ||=".to_string());
    let result = lexer.lex_all();
    let kinds: Vec<_> = result.tokens.iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenType::PlusEq,
            TokenType::MinusEq,
            TokenType::StarEq,
            TokenType::SlashEq,
            TokenType::PercentEq,
            TokenType::StarStarEq,
            TokenType::AndAndEq,
            TokenType::OrOrEq,
            TokenType::Eof,
        ]
    );
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_lexer_unterminated_string() {
    let lexer = Lexer::new(FileId::new(0), "\"hello".to_string());
    let result = lexer.lex_all();
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == Some(e1::UNTERMINATED_STRING as u32)));
}

#[test]
fn test_lexer_rejects_unknown_escape() {
    let mut lexer = Lexer::new(FileId::new(0), r#""a\qb""#.to_string());
    let _ = lexer.next_token();
    assert!(
        lexer
            .diagnostics()
            .iter()
            .any(|d| d.message.contains("escape")),
        "expected an unsupported-escape diagnostic, got: {:?}",
        lexer.diagnostics()
    );
}

#[test]
fn test_lexer_accepts_known_escapes_and_keeps_raw_value() {
    let mut lexer = Lexer::new(FileId::new(0), r#""a\tb\n""#.to_string());
    let token = lexer.next_token().expect("token");
    // Value is kept RAW (with backslashes) so kali_fmt round-trips.
    assert_eq!(token.value, r#""a\tb\n""#);
    assert!(lexer.diagnostics().is_empty(), "{:?}", lexer.diagnostics());
}

/// PINS THE `__proto__` SECURITY INVARIANT this allowlist is load-bearing for.
///
/// `kali_optimize::object_fold` has two prototype-pollution guards
/// (`fold_object_enumeration_call`'s property-admission check, and
/// `collect_permitted_occurrences`'s timeline-eligibility disqualifier) that
/// compare a possibly-UNDECODED key/member text against the literal constant
/// `"__proto__"`. That comparison is sound only because (a) a key slot's text
/// is never decoded at all (`kali_parser::unquote_string_literal` strips
/// delimiters only), so any escaped spelling keeps a residual backslash and
/// can never equal the backslash-free `"__proto__"`, and (b) none of the
/// accepted escapes decode to a letter, digit, or underscore -- the only
/// characters `__proto__` is made of -- so an escaped spelling could never be
/// the real setter name even under full JavaScript decode semantics. See
/// docs/superpowers/followups/property-key-trim-site-classification.md
/// section 4.1, which this test enforces rather than merely asserts in prose.
///
/// If this test goes RED because an escape was ADDED to the lexer, STOP and
/// re-read that section before touching `kali_optimize::object_fold`'s two
/// `__proto__` guards: an escape that decodes to `_`, a letter, or a digit
/// (`\u`, `\x`, or similar) breaks precondition (b), and those guards must
/// move onto the DECODED property name, not the stored text, before the
/// change ships.
#[test]
fn test_lexer_string_escape_allowlist_is_pinned_for_the_proto_guard() {
    // The full accepted set, mirroring `crates/kali_lexer/src/string.rs`'s
    // `matches!` arm exactly. If this list and that `matches!` arm diverge,
    // this test's own coverage is wrong before its assertions even run.
    let allowed = ['n', 't', 'r', '\\', '"', '\'', '`', '0', 'b', 'f', 'v'];

    for c in allowed {
        let source = format!("\"a\\{c}b\"");
        let mut lexer = Lexer::new(FileId::new(0), source.clone());
        let _ = lexer.next_token();
        assert!(
            lexer.diagnostics().is_empty(),
            "escape `\\{c}` ({source:?}) regressed out of the lexer's \
             accepted set: {:?}. If an escape was intentionally REMOVED, \
             update this test's `allowed` list. If this is unexpected, an \
             escape being refused is a lexer bug independent of the \
             __proto__ guard this test exists to protect.",
            lexer.diagnostics()
        );
    }

    // None of the eleven decode to a letter, digit, or underscore. This is
    // precondition (b) above, checked against Rust's own escape semantics
    // (which agree with JavaScript's for this exact set) rather than assumed.
    let decoded: [(char, char); 11] = [
        ('n', '\n'),
        ('t', '\t'),
        ('r', '\r'),
        ('\\', '\\'),
        ('"', '"'),
        ('\'', '\''),
        ('`', '`'),
        ('0', '\0'),
        ('b', '\u{8}'),
        ('f', '\u{c}'),
        ('v', '\u{b}'),
    ];
    assert_eq!(
        decoded.map(|(escape, _)| escape),
        allowed,
        "the decoded-meaning table above drifted from `allowed`; keep both in sync"
    );
    for (escape, meaning) in decoded {
        assert!(
            !meaning.is_alphanumeric() && meaning != '_',
            "escape `\\{escape}` now decodes to `{meaning:?}`, a letter, digit, \
             or underscore -- one of the characters `__proto__` is made of. \
             This breaks precondition (b) of the __proto__ guard argument in \
             docs/superpowers/followups/property-key-trim-site-classification.md \
             section 4.1. Before merging, re-derive that section's Direction A \
             argument and, if it no longer holds, move \
             `kali_optimize::object_fold`'s two `__proto__` guards \
             (`fold_object_enumeration_call` and `collect_permitted_occurrences`) \
             onto the DECODED property name.",
        );
    }
}

#[test]
fn test_lexer_multiline_template() {
    let mut lexer = Lexer::new(FileId::new(0), "`hello\nworld`".to_string());
    let token = lexer.next_token().unwrap();
    assert_eq!(token.kind, TokenType::Template);
    assert_eq!(token.value, "`hello\nworld`");

    let result = lexer.lex_all();
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
}

#[test]
fn test_lexer_template_preserves_interpolation_delimiters() {
    let mut lexer = Lexer::new(FileId::new(0), "`hello ${world}`".to_string());
    let token = lexer.next_token().unwrap();
    assert_eq!(token.kind, TokenType::Template);
    assert_eq!(token.value, "`hello ${world}`");
}

#[test]
fn test_lexer_template_rejects_unknown_escape() {
    let mut lexer = Lexer::new(FileId::new(0), "`a\\qb`".to_string());
    let _ = lexer.next_token();
    assert!(
        lexer
            .diagnostics()
            .iter()
            .any(|d| d.message.contains("escape")),
        "expected an unsupported-escape diagnostic, got: {:?}",
        lexer.diagnostics()
    );
}

#[test]
fn test_lexer_template_accepts_known_escapes_and_keeps_raw_value() {
    let mut lexer = Lexer::new(FileId::new(0), "`a\\tb\\n`".to_string());
    let token = lexer.next_token().expect("token");
    // Value is kept RAW (with backslashes) so kali_fmt round-trips templates verbatim.
    assert_eq!(token.value, "`a\\tb\\n`");
    assert!(lexer.diagnostics().is_empty(), "{:?}", lexer.diagnostics());
}

#[test]
fn test_lexer_exponent_number() {
    for source in ["1e5", "4.84e+00", "2E-3", "1.5e1"] {
        let lexer = Lexer::new(FileId::new(0), source.to_string());
        let result = lexer.lex_all();
        assert_eq!(
            result.tokens[0].kind,
            TokenType::NumericLiteral,
            "source: {source}"
        );
        assert_eq!(result.tokens[0].value, source, "source: {source}");
        assert!(result.diagnostics.is_empty(), "source: {source}");
    }
}

#[test]
fn test_lexer_exponent_without_digits_is_not_consumed() {
    let lexer = Lexer::new(FileId::new(0), "1e".to_string());
    let result = lexer.lex_all();
    assert_eq!(result.tokens[0].kind, TokenType::NumericLiteral);
    assert_eq!(result.tokens[0].value, "1");
    assert_eq!(result.tokens[1].kind, TokenType::Identifier);
}
