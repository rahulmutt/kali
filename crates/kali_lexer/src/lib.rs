//! Tokenizer/lexer for TypeScript and JavaScript.

use kali_common::{FileId, Span};
use kali_error::_error_codes::e1;
use kali_error::diagnostic::Diagnostic;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Percent,
    And,
    Or,
    Not,
    EqEquals,
    EqEqEq,
    Neq,
    NeqNeq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    LtLt,
    GtGt,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    AndAnd,
    OrOr,
    QuestionDot,
    Arrow,
    Colon,
    Eq,
    Ampersand,
    Pipe,
    Tilde,
    At,
    Bang,
    Dot,
    DotDotDot,
    Hash,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Semicolon,
    Comma,
    Backtick,
    Question,
    NullCoalesce,
    Eof,
    Comment,
    Identifier,
    NumericLiteral,
    StringLiteral,
    Template,
    Unknown,
    If,
    Else,
    For,
    While,
    Do,
    Switch,
    Case,
    Default,
    Break,
    Continue,
    Return,
    Throw,
    Try,
    Catch,
    Finally,
    Debugger,
    New,
    Function,
    Var,
    Let,
    Const,
    Class,
    Interface,
    Type,
    Enum,
    Import,
    Export,
    From,
    As,
    This,
    Super,
    Extends,
    Implements,
    Async,
    Await,
    Yield,
    InstanceOf,
    In,
    Of,
    True,
    False,
    Null,
    Undefined,
    Void,
    Delete,
    Typeof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenType,
    pub value: String,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenType, value: String, span: Span) -> Self {
        Self { kind, value, span }
    }
}

pub struct LexerResult {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

pub struct Lexer {
    source: Vec<char>,
    file_id: FileId,
    position: usize,
    diagnostics: Vec<Diagnostic>,
}

impl Lexer {
    pub fn new(file_id: FileId, source: String) -> Self {
        Self {
            source: source.chars().collect(),
            file_id,
            position: 0,
            diagnostics: Vec::new(),
        }
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

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

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.source.get(self.position) {
            if c.is_whitespace() {
                self.position += 1;
            } else {
                break;
            }
        }
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

    fn lex_identifier(&mut self) -> Token {
        let _start = self.position;
        while let Some(&c) = self.source.get(self.position) {
            if c.is_ascii_alphanumeric() || c == '_' || c == '$' {
                self.position += 1;
            } else {
                break;
            }
        }
        let value: String = self.source[_start..self.position].iter().collect();
        let kind = match value.as_str() {
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "for" => TokenType::For,
            "while" => TokenType::While,
            "do" => TokenType::Do,
            "switch" => TokenType::Switch,
            "case" => TokenType::Case,
            "default" => TokenType::Default,
            "break" => TokenType::Break,
            "continue" => TokenType::Continue,
            "return" => TokenType::Return,
            "throw" => TokenType::Throw,
            "try" => TokenType::Try,
            "catch" => TokenType::Catch,
            "finally" => TokenType::Finally,
            "debugger" => TokenType::Debugger,
            "new" => TokenType::New,
            "function" => TokenType::Function,
            "var" => TokenType::Var,
            "let" => TokenType::Let,
            "const" => TokenType::Const,
            "class" => TokenType::Class,
            "interface" => TokenType::Interface,
            "type" => TokenType::Type,
            "enum" => TokenType::Enum,
            "import" => TokenType::Import,
            "export" => TokenType::Export,
            "from" => TokenType::From,
            "as" => TokenType::As,
            "this" => TokenType::This,
            "super" => TokenType::Super,
            "extends" => TokenType::Extends,
            "implements" => TokenType::Implements,
            "async" => TokenType::Async,
            "await" => TokenType::Await,
            "yield" => TokenType::Yield,
            "instanceof" => TokenType::InstanceOf,
            "in" => TokenType::In,
            "of" => TokenType::Of,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "null" => TokenType::Null,
            "undefined" => TokenType::Undefined,
            "void" => TokenType::Void,
            "delete" => TokenType::Delete,
            "typeof" => TokenType::Typeof,
            _ => TokenType::Identifier,
        };
        Token::new(kind, self.slice(_start), self.span())
    }

    fn lex_number(&mut self) -> Token {
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

    fn lex_string(&mut self, quote: char) -> Token {
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

    fn lex_template(&mut self) -> Token {
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

    fn lex_division_or_comment(&mut self) -> Token {
        self.position += 1;
        match self.source.get(self.position) {
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

    fn lex_punct(&mut self, initial: char) -> Option<Token> {
        let (kind, lexeme, len) = match initial {
            '&' if self.nth(1) == Some('&') => (TokenType::AndAnd, "&&".to_string(), 2),
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
            '?' if self.nth(1) == Some('?') => (TokenType::NullCoalesce, "??".to_string(), 2),
            '?' if self.nth(1) == Some('.') => (TokenType::QuestionDot, "?.".to_string(), 2),
            '.' if self.nth(1) == Some('.') && self.nth(2) == Some('.') => {
                (TokenType::DotDotDot, "...".to_string(), 3)
            }
            '=' if self.nth(1) == Some('>') => (TokenType::Arrow, "=>".to_string(), 2),
            '+' if self.nth(1) == Some('+') => (TokenType::Plus, "++".to_string(), 2),
            '-' if self.nth(1) == Some('-') => (TokenType::Minus, "--".to_string(), 2),
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

    fn peek(&self) -> Option<char> {
        self.source.get(self.position).copied()
    }

    fn nth(&self, n: usize) -> Option<char> {
        self.source.get(self.position + n).copied()
    }

    fn is_eof(&self) -> bool {
        self.position >= self.source.len()
    }

    fn span(&self) -> Span {
        Span::new(self.file_id, self.position as u32, self.position as u32)
    }

    fn emit_error(&mut self, code: u16, message: &str) {
        self.diagnostics.push(Diagnostic::error(
            code as u32,
            format!("{} at position {}", message, self.position),
        ));
    }

    fn slice(&self, start: usize) -> String {
        self.source[start..self.position].iter().collect()
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "test_and_and.rs"]
mod test_and_and;
