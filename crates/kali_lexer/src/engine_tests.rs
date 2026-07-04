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
