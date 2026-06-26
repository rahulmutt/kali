use crate::token::{Token, TokenType};
use crate::Lexer;
use kali_error::_error_codes::e1;

impl Lexer {
    pub(crate) fn lex_division_or_comment(&mut self) -> Token {
        self.position += 1;
        match self.source.get(self.position) {
            Some(&'=') => {
                self.position += 1;
                Token::new(TokenType::SlashEq, "/=".into(), self.span())
            }
            Some(&'*') => self.lex_block_comment(),
            Some(&'/') => self.lex_line_comment(),
            _ => Token::new(TokenType::Slash, "/".into(), self.span()),
        }
    }

    fn lex_block_comment(&mut self) -> Token {
        let _start = self.position;
        self.position += 1; // skip *
        let mut value = String::new();
        value.push('*');

        loop {
            match self.source.get(self.position) {
                Some(&'*') => {
                    value.push('*');
                    self.position += 1;
                    if self.source.get(self.position) == Some(&'/') {
                        value.push('/');
                        self.position += 1;
                        return Token::new(TokenType::Comment, format!("/*{}", value), self.span());
                    }
                }
                Some(&'\n') | None => {
                    self.emit_error(e1::ILLEGAL_SYMBOL, "Unterminated block comment");
                    return Token::new(TokenType::Comment, format!("/*{}", value), self.span());
                }
                Some(&c) => {
                    value.push(c);
                    self.position += 1;
                }
            }
        }
    }

    fn lex_line_comment(&mut self) -> Token {
        let _start = self.position;
        let mut value = String::new();
        value.push('/');

        loop {
            match self.source.get(self.position) {
                Some(&'\n') => {
                    value.push('\n');
                    self.position += 1;
                    return Token::new(TokenType::Comment, format!("//{}", value), self.span());
                }
                None => return Token::new(TokenType::Comment, format!("//{}", value), self.span()),
                Some(&c) => {
                    value.push(c);
                    self.position += 1;
                }
            }
        }
    }
}
