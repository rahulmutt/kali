use super::*;

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
fn test_lexer_unterminated_string() {
    let lexer = Lexer::new(FileId::new(0), "\"hello".to_string());
    let result = lexer.lex_all();
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == Some(e1::UNTERMINATED_STRING as u32)));
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
