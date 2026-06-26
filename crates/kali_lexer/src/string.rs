use crate::token::{Token, TokenType};
use crate::Lexer;
use kali_error::_error_codes::e1;

impl Lexer {
    pub(crate) fn lex_string(&mut self, quote: char) -> Token {
        let _start = self.position;
        self.position += 1; // skip quote
        let mut value = String::new();
        value.push(quote);

        loop {
            match self.source.get(self.position) {
                Some(&c) if c == quote => {
                    value.push(c);
                    self.position += 1;
                    break;
                }
                Some(&c) if c == '\\' => {
                    value.push(c);
                    self.position += 1;
                    if let Some(next) = self.source.get(self.position).copied() {
                        value.push(next);
                        self.position += 1;
                    }
                }
                Some(&'\n') => {
                    self.emit_error(e1::UNTERMINATED_STRING, "Unterminated string");
                    return Token::new(TokenType::StringLiteral, value, self.span());
                }
                Some(&c) => {
                    value.push(c);
                    self.position += 1;
                }
                None => {
                    self.emit_error(e1::UNTERMINATED_STRING, "Unterminated string");
                    return Token::new(TokenType::StringLiteral, value, self.span());
                }
            }
        }
        Token::new(TokenType::StringLiteral, value, self.span())
    }
}
