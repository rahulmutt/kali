use crate::token::{LexerResult, Token, TokenType};
use crate::Lexer;

impl Lexer {
    pub fn lex_all(mut self) -> LexerResult {
        let mut tokens: Vec<Token> = Vec::new();
        while let Some(token) = self.next_token() {
            if token.kind == TokenType::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        LexerResult {
            tokens,
            diagnostics: std::mem::take(&mut self.diagnostics),
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        if self.is_eof() {
            return Some(Token::new(TokenType::Eof, String::new(), self.span()));
        }
        self.collect_token()
    }

    fn collect_token(&mut self) -> Option<Token> {
        let c = self.peek().unwrap();

        if c.is_ascii_alphabetic() || c == '_' || c == '$' {
            return Some(self.lex_identifier());
        }
        if c.is_ascii_digit() {
            return Some(self.lex_number());
        }
        if c == '"' || c == '\'' {
            return Some(self.lex_string(c));
        }
        if c == '`' {
            return Some(self.lex_template());
        }
        if c == '/' {
            return Some(self.lex_division_or_comment());
        }
        self.lex_punct(c)
    }
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod engine_tests;
