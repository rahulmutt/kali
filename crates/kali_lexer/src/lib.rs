//! Tokenizer/lexer for TypeScript and JavaScript.

mod comment;
mod cursor;
mod identifier;
mod number;
mod string;
mod template;
mod token;

pub use token::{LexerResult, Token, TokenType};
pub use cursor::Lexer;

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
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "test_and_and.rs"]
mod test_and_and;
