use crate::token::{Token, TokenType};
use crate::Lexer;

impl Lexer {
    pub(crate) fn lex_number(&mut self) -> Token {
        let _start = self.position;
        while let Some(&c) = self.source.get(self.position) {
            if c.is_ascii_digit() {
                self.position += 1;
            } else {
                break;
            }
        }

        if self.source.get(self.position) == Some(&'.')
            && self
                .source
                .get(self.position + 1)
                .is_some_and(|c| c.is_ascii_digit())
        {
            self.position += 1;
            while let Some(&c) = self.source.get(self.position) {
                if c.is_ascii_digit() {
                    self.position += 1;
                } else {
                    break;
                }
            }
        }

        if self.source.get(self.position) == Some(&'n') {
            self.position += 1;
        }

        Token::new(TokenType::NumericLiteral, self.slice(_start), self.span())
    }
}
