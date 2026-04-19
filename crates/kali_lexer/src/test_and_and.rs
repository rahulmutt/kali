use super::*;
#[test]
fn test_peek_and_and() {
    let lexer = Lexer::new(FileId::new(0), "x && y;".to_string());
    let result = lexer.lex_all();
    let tokens: Vec<_> = result
        .tokens
        .into_iter()
        .filter(|t| t.kind != TokenType::Eof)
        .collect();
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[1].kind, TokenType::AndAnd);
}
