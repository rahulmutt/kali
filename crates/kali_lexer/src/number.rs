use crate::token::{Token, TokenType};
use crate::Lexer;
use kali_common::numeric_literal::parse_js_numeric_literal;
use kali_error::_error_codes::e1;

impl Lexer {
    /// Lexes one numeric literal and reports `E1100` when JavaScript would
    /// refuse its spelling.
    ///
    /// The token keeps the literal's raw text, so `kali_fmt` re-emits it
    /// verbatim. Its value is read by
    /// `kali_common::numeric_literal::parse_js_numeric_literal`, the same
    /// function this lexer validates with, so a token that lexes clean always
    /// has a value.
    pub(crate) fn lex_number(&mut self) -> Token {
        let start = self.position;
        if self.at_radix_prefix() {
            // `0x`/`0b`/`0o`: take every identifier character, so `0xfg` and
            // `0b12` are one malformed token rather than a number followed by
            // an identifier.
            self.position += 2;
            while self
                .source
                .get(self.position)
                .is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_')
            {
                self.position += 1;
            }
            return self.finish_number(start);
        }

        self.skip_digits_and_separators();

        // A fraction needs a digit after the dot, so `07.toString()` and
        // `1.toFixed` keep their member dot.
        if self.source.get(self.position) == Some(&'.')
            && self
                .source
                .get(self.position + 1)
                .is_some_and(|c| c.is_ascii_digit())
        {
            self.position += 1;
            self.skip_digits_and_separators();
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
                self.skip_digits_and_separators();
                return self.finish_number(start);
            }
        }

        // `042n` is still one token, so the refusal names it whole.
        if self.source.get(self.position) == Some(&'n') {
            self.position += 1;
        }

        self.finish_number(start)
    }

    fn at_radix_prefix(&self) -> bool {
        self.source.get(self.position) == Some(&'0')
            && matches!(
                self.source.get(self.position + 1),
                Some('x' | 'X' | 'b' | 'B' | 'o' | 'O')
            )
    }

    fn skip_digits_and_separators(&mut self) {
        while self
            .source
            .get(self.position)
            .is_some_and(|c| c.is_ascii_digit() || *c == '_')
        {
            self.position += 1;
        }
    }

    fn finish_number(&mut self, start: usize) -> Token {
        let text = self.slice(start);
        if parse_js_numeric_literal(&text).is_none() {
            self.emit_error(e1::INVALID_NUMBER, "invalid numeric literal");
        }
        Token::new(TokenType::NumericLiteral, text, self.span())
    }
}
