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

        // Scientific-notation exponent: `e`/`E`, optional sign, then at least
        // one digit (`1e5`, `4.84e+00`, `2E-3`). Without a digit the suffix is
        // not part of the number (`1e` lexes as `1` then identifier `e`), and
        // an exponent never takes a bigint `n` suffix (`1e5n` leaves `n` to
        // the identifier lexer; the parser rejects it).
        if matches!(self.source.get(self.position), Some(&'e') | Some(&'E')) {
            let mut probe = self.position + 1;
            if matches!(self.source.get(probe), Some(&'+') | Some(&'-')) {
                probe += 1;
            }
            if self.source.get(probe).is_some_and(|c| c.is_ascii_digit()) {
                self.position = probe;
                while let Some(&c) = self.source.get(self.position) {
                    if c.is_ascii_digit() {
                        self.position += 1;
                    } else {
                        break;
                    }
                }
                return Token::new(TokenType::NumericLiteral, self.slice(_start), self.span());
            }
        }

        if self.source.get(self.position) == Some(&'n') {
            self.position += 1;
        }

        Token::new(TokenType::NumericLiteral, self.slice(_start), self.span())
    }
}
