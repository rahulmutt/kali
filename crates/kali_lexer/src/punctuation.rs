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
            '<' if self.nth(1) == Some('<') => (TokenType::LtLt, "<<".to_string(), 2),
            '>' if self.nth(1) == Some('=') => (TokenType::GtEq, ">=".to_string(), 2),
            '>' if self.nth(1) == Some('>') && self.nth(2) == Some('>') => {
                (TokenType::GtGt, ">>>".to_string(), 3)
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
            '&' => (TokenType::Ampersand, "&".to_string(), 1),
            '|' => (TokenType::Pipe, "|".to_string(), 1),
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
