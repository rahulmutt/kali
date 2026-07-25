use crate::token::{Token, TokenType};
use crate::Lexer;

impl Lexer {
    pub(crate) fn lex_punct(&mut self, initial: char) -> Option<Token> {
        let (kind, lexeme, len) = match initial {
            '&' if self.nth(1) == Some('&') && self.nth(2) == Some('=') => {
                (TokenType::AndAndEq, "&&=".to_string(), 3)
            }
            '&' if self.nth(1) == Some('&') => (TokenType::AndAnd, "&&".to_string(), 2),
            '|' if self.nth(1) == Some('|') && self.nth(2) == Some('=') => {
                (TokenType::OrOrEq, "||=".to_string(), 3)
            }
            '|' if self.nth(1) == Some('|') => (TokenType::OrOr, "||".to_string(), 2),
            '=' if self.nth(1) == Some('=') && self.nth(2) == Some('=') => {
                (TokenType::EqEqEq, "===".to_string(), 3)
            }
            '=' if self.nth(1) == Some('=') => (TokenType::EqEquals, "==".to_string(), 2),
            '!' if self.nth(1) == Some('=') && self.nth(2) == Some('=') => {
                (TokenType::NeqNeq, "!==".to_string(), 3)
            }
            '!' if self.nth(1) == Some('=') => (TokenType::Neq, "!=".to_string(), 2),
            '<' if self.nth(1) == Some('=') => (TokenType::LtEq, "<=".to_string(), 2),
            // `<<=` must be checked before the bare `<<` arm below — its guard
            // (nth(2) == '=') is a strict superset match that the bare arm's
            // guard (nth(1) == '<' alone) would otherwise swallow first.
            '<' if self.nth(1) == Some('<') && self.nth(2) == Some('=') => {
                (TokenType::LtLtEq, "<<=".to_string(), 3)
            }
            '<' if self.nth(1) == Some('<') => (TokenType::LtLt, "<<".to_string(), 2),
            '>' if self.nth(1) == Some('=') => (TokenType::GtEq, ">=".to_string(), 2),
            // `>>>=` must be checked before the existing `>>>` arm: that arm's
            // guard only inspects nth(1)/nth(2) and does not look at nth(3), so
            // it would otherwise consume `>>>=` as `>>>` followed by a stray
            // `=` token. Distinct TokenType from plain `>>>` (unlike the
            // pre-existing `>>`/`>>>` pair below, which share `GtGt` and are
            // told apart only by lexeme — do not replicate that here).
            '>' if self.nth(1) == Some('>')
                && self.nth(2) == Some('>')
                && self.nth(3) == Some('=') =>
            {
                (TokenType::GtGtGtEq, ">>>=".to_string(), 4)
            }
            '>' if self.nth(1) == Some('>') && self.nth(2) == Some('>') => {
                (TokenType::GtGt, ">>>".to_string(), 3)
            }
            // `>>=` must be checked before the bare `>>` arm below, for the
            // same reason as `<<=` above. Its guard (nth(2) == '=') is disjoint
            // from the `>>>`/`>>>=` arms' guard (nth(2) == '>'), so ordering
            // relative to those two does not matter.
            '>' if self.nth(1) == Some('>') && self.nth(2) == Some('=') => {
                (TokenType::GtGtEq, ">>=".to_string(), 3)
            }
            '>' if self.nth(1) == Some('>') => (TokenType::GtGt, ">>".to_string(), 2),
            '?' if self.nth(1) == Some('?') && self.nth(2) == Some('=') => {
                (TokenType::NullCoalesceEq, "??=".to_string(), 3)
            }
            '?' if self.nth(1) == Some('?') => (TokenType::NullCoalesce, "??".to_string(), 2),
            '?' if self.nth(1) == Some('.') => (TokenType::QuestionDot, "?.".to_string(), 2),
            '+' if self.nth(1) == Some('+') => (TokenType::Plus, "++".to_string(), 2),
            '+' if self.nth(1) == Some('=') => (TokenType::PlusEq, "+=".to_string(), 2),
            '-' if self.nth(1) == Some('-') => (TokenType::Minus, "--".to_string(), 2),
            '-' if self.nth(1) == Some('=') => (TokenType::MinusEq, "-=".to_string(), 2),
            '*' if self.nth(1) == Some('*') && self.nth(2) == Some('=') => {
                (TokenType::StarStarEq, "**=".to_string(), 3)
            }
            '*' if self.nth(1) == Some('*') => (TokenType::StarStar, "**".to_string(), 2),
            '*' if self.nth(1) == Some('=') => (TokenType::StarEq, "*=".to_string(), 2),
            '/' if self.nth(1) == Some('=') => (TokenType::SlashEq, "/=".to_string(), 2),
            '%' if self.nth(1) == Some('=') => (TokenType::PercentEq, "%=".to_string(), 2),
            '%' => (TokenType::Percent, "%".to_string(), 1),
            '=' if self.nth(1) == Some('>') => (TokenType::Arrow, "=>".to_string(), 2),
            '+' => (TokenType::Plus, "+".to_string(), 1),
            '-' => (TokenType::Minus, "-".to_string(), 1),
            '*' => (TokenType::Star, "*".to_string(), 1),
            '/' => (TokenType::Slash, "/".to_string(), 1),
            // `&=`/`|=`/`^=` must be checked before the bare single-char arms
            // below, for the same reason as the shift compound-assigns above.
            // They do not collide with the `&&=`/`&&`/`||=`/`||` arms earlier
            // in this match: those guard on nth(1) being `&`/`|` (a second
            // copy of the same character), which is mutually exclusive with
            // nth(1) == '='.
            '&' if self.nth(1) == Some('=') => (TokenType::AmpEq, "&=".to_string(), 2),
            '|' if self.nth(1) == Some('=') => (TokenType::PipeEq, "|=".to_string(), 2),
            '^' if self.nth(1) == Some('=') => (TokenType::CaretEq, "^=".to_string(), 2),
            '&' => (TokenType::Ampersand, "&".to_string(), 1),
            '|' => (TokenType::Pipe, "|".to_string(), 1),
            '^' => (TokenType::Caret, "^".to_string(), 1),
            '!' => (TokenType::Not, "!".to_string(), 1),
            '<' => (TokenType::Lt, "<".to_string(), 1),
            '>' => (TokenType::Gt, ">".to_string(), 1),
            '?' => (TokenType::Question, "?".to_string(), 1),
            '=' => (TokenType::Eq, "=".to_string(), 1),
            ':' => (TokenType::Colon, ":".to_string(), 1),
            '.' if self.nth(1) == Some('.') && self.nth(2) == Some('.') => {
                (TokenType::DotDotDot, "...".to_string(), 3)
            }
            '.' => (TokenType::Dot, ".".to_string(), 1),
            '#' => (TokenType::Hash, "#".to_string(), 1),
            '@' => (TokenType::At, "@".to_string(), 1),
            '~' => (TokenType::Tilde, "~".to_string(), 1),
            '(' => (TokenType::LeftParen, "(".to_string(), 1),
            ')' => (TokenType::RightParen, ")".to_string(), 1),
            '{' => (TokenType::LeftBrace, "{".to_string(), 1),
            '}' => (TokenType::RightBrace, "}".to_string(), 1),
            '[' => (TokenType::LeftBracket, "[".to_string(), 1),
            ']' => (TokenType::RightBracket, "]".to_string(), 1),
            ';' => (TokenType::Semicolon, ";".to_string(), 1),
            ',' => (TokenType::Comma, ",".to_string(), 1),
            '`' => (TokenType::Backtick, "`".to_string(), 1),
            _ => (TokenType::Unknown, initial.to_string(), 1),
        };

        self.position += len;
        Some(Token::new(kind, lexeme, self.span()))
    }
}

#[cfg(test)]
#[path = "punctuation_tests.rs"]
mod punctuation_tests;
