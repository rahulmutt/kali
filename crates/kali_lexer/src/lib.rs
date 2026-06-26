//! Tokenizer/lexer for TypeScript and JavaScript.

mod comment;
mod cursor;
mod engine;
mod identifier;
mod number;
mod punctuation;
mod string;
mod template;
mod token;

pub use cursor::Lexer;
pub use token::{LexerResult, Token, TokenType};
