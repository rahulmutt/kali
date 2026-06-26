use crate::token::{Token, TokenType};
use crate::Lexer;
use kali_error::_error_codes::e1;

impl Lexer {
    pub(crate) fn lex_template(&mut self) -> Token {
        let _start = self.position;
        self.position += 1; // skip backtick
        let mut value = String::new();
        value.push('`');

        loop {
            match self.source.get(self.position) {
                Some(&'`') => {
                    value.push('`');
                    self.position += 1;
                    return Token::new(TokenType::Template, value, self.span());
                }
                Some(&'$') => {
                    value.push('$');
                    self.position += 1;
                    if let Some(&'{') = self.source.get(self.position) {
                        value.push('{');
                        self.position += 1;
                    }
                }
                Some(&'\n') => {
                    value.push('\n');
                    self.position += 1;
                }
                Some(&c) => {
                    value.push(c);
                    self.position += 1;
                }
                None => {
                    self.emit_error(e1::UNTERMINATED_TEMPLATE, "Unterminated template");
                    return Token::new(TokenType::Template, value, self.span());
                }
            }
        }
    }
}
