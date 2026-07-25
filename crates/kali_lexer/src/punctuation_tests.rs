use crate::{Lexer, TokenType};
use kali_common::FileId;

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

// --- Task 1.5: the six bitwise compound assignment operators ---

#[test]
fn test_lexer_bitwise_compound_assignment_tokens() {
    for (src, kind) in [
        ("&=", TokenType::AmpEq),
        ("|=", TokenType::PipeEq),
        ("^=", TokenType::CaretEq),
        ("<<=", TokenType::LtLtEq),
        (">>=", TokenType::GtGtEq),
        (">>>=", TokenType::GtGtGtEq),
    ] {
        let lexer = Lexer::new(FileId::new(0), src.to_string());
        let result = lexer.lex_all();
        let tokens: Vec<_> = result
            .tokens
            .iter()
            .filter(|t| t.kind != TokenType::Eof)
            .collect();
        assert_eq!(
            tokens.len(),
            1,
            "expected one token for {src:?}, got {tokens:?}"
        );
        assert_eq!(tokens[0].kind, kind, "wrong TokenType for {src:?}");
        assert_eq!(tokens[0].value, src, "wrong lexeme for {src:?}");
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics for {src:?}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn test_lexer_shift_operator_non_regression() {
    // The new `<<=`/`>>=`/`>>>=` arms must not disturb the existing shift and
    // relational operators they sit next to.
    let lexer = Lexer::new(FileId::new(0), ">>".to_string());
    let result = lexer.lex_all();
    let tokens: Vec<_> = result
        .tokens
        .into_iter()
        .filter(|t| t.kind != TokenType::Eof)
        .collect();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenType::GtGt);
    assert_eq!(tokens[0].value, ">>");

    let lexer = Lexer::new(FileId::new(0), ">>>".to_string());
    let result = lexer.lex_all();
    let tokens: Vec<_> = result
        .tokens
        .into_iter()
        .filter(|t| t.kind != TokenType::Eof)
        .collect();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenType::GtGt);
    assert_eq!(tokens[0].value, ">>>");

    let lexer = Lexer::new(FileId::new(0), "<<".to_string());
    let result = lexer.lex_all();
    let tokens: Vec<_> = result
        .tokens
        .into_iter()
        .filter(|t| t.kind != TokenType::Eof)
        .collect();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenType::LtLt);
    assert_eq!(tokens[0].value, "<<");

    let lexer = Lexer::new(FileId::new(0), ">=".to_string());
    let result = lexer.lex_all();
    let tokens: Vec<_> = result
        .tokens
        .into_iter()
        .filter(|t| t.kind != TokenType::Eof)
        .collect();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenType::GtEq);
    assert_eq!(tokens[0].value, ">=");

    let lexer = Lexer::new(FileId::new(0), "<=".to_string());
    let result = lexer.lex_all();
    let tokens: Vec<_> = result
        .tokens
        .into_iter()
        .filter(|t| t.kind != TokenType::Eof)
        .collect();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].kind, TokenType::LtEq);
    assert_eq!(tokens[0].value, "<=");
}

#[test]
fn test_lexer_shift_vs_shift_assign_streams_differ() {
    let lexer = Lexer::new(FileId::new(0), "a >> b".to_string());
    let result = lexer.lex_all();
    let kinds: Vec<_> = result
        .tokens
        .into_iter()
        .filter(|t| t.kind != TokenType::Eof)
        .map(|t| t.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![
            TokenType::Identifier,
            TokenType::GtGt,
            TokenType::Identifier
        ]
    );

    let lexer = Lexer::new(FileId::new(0), "a >>= b".to_string());
    let result = lexer.lex_all();
    let kinds: Vec<_> = result
        .tokens
        .into_iter()
        .filter(|t| t.kind != TokenType::Eof)
        .map(|t| t.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![
            TokenType::Identifier,
            TokenType::GtGtEq,
            TokenType::Identifier
        ]
    );
}
