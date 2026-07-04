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
                Some(&'\\') => {
                    value.push('\\');
                    self.position += 1;
                    if let Some(next) = self.source.get(self.position).copied() {
                        // Keep the raw sequence in `value` (kali_fmt re-emits templates
                        // verbatim); only validate, matching the recognized set in
                        // string.rs. Consuming both chars here also means an escaped
                        // backtick/`$` doesn't terminate the template or start an
                        // interpolation early.
                        if !matches!(
                            next,
                            'n' | 't' | 'r' | '\\' | '"' | '\'' | '`' | '0' | 'b' | 'f' | 'v'
                        ) {
                            self.emit_error(
                                e1::UNSUPPORTED_ESCAPE,
                                "unsupported string escape sequence",
                            );
                        }
                        value.push(next);
                        self.position += 1;
                    }
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
