//! Shared test helpers for the parser test modules.
use kali_common::FileId;
use kali_lexer::{Lexer, Token};

pub(crate) fn lex(source: &str) -> Vec<Token> {
    let lexer = Lexer::new(FileId::new(0), source.to_string());
    lexer.lex_all().tokens
}
