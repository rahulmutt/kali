//! Tokenizer/lexer for TypeScript and JavaScript.

use kali_error::diagnostic::Diagnostic;
use kali_error::_error_codes::e1;
use kali_common::{FileId, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Plus, Minus, Star, Slash, Caret, Percent,
    And, Or, Not, EqEquals, EqEqEq, Neq, NeqNeq,
    Lt, Gt, LtEq, GtEq, LtLt, GtGt,
    PlusEq, MinusEq, StarEq, SlashEq, 
    AndAnd, OrOr, QuestionDot, Arrow,
    Colon, Eq, Ampersand, Pipe, Tilde, At,
    Bang, Dot, DotDotDot, Hash, LeftParen,
    RightParen, LeftBrace, RightBrace, LeftBracket,
    RightBracket, Semicolon, Comma, Backtick,
    Question, NullCoalesce, Eof, Comment, Identifier,
    NumericLiteral, StringLiteral, Template, Unknown,
    If, Else, For, While, Do, Switch, Case, Default,
    Break, Continue, Return, Throw, Try, Catch, Finally,
    New, Function, Var, Let, Const, Class, Interface,
    Type, Enum, Import, Export, From, As, This, Super,
    Extends, Implements, Async, Await, Yield, InstanceOf,
    In, Of, True, False, Null, Undefined, Void, Delete, Typeof,
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
        LexerResult { tokens, diagnostics: std::mem::take(&mut self.diagnostics) }
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
            "if" => TokenType::If, "else" => TokenType::Else,
            "for" => TokenType::For, "while" => TokenType::While,
            "do" => TokenType::Do, "switch" => TokenType::Switch,
            "case" => TokenType::Case, "default" => TokenType::Default,
            "break" => TokenType::Break, "continue" => TokenType::Continue,
            "return" => TokenType::Return, "throw" => TokenType::Throw,
            "try" => TokenType::Try, "catch" => TokenType::Catch,
            "new" => TokenType::New, "function" => TokenType::Function,
            "var" => TokenType::Var, "let" => TokenType::Let,
            "const" => TokenType::Const, "class" => TokenType::Class,
            "interface" => TokenType::Interface, "type" => TokenType::Type,
            "enum" => TokenType::Enum, "import" => TokenType::Import,
            "export" => TokenType::Export, "from" => TokenType::From,
            "as" => TokenType::As, "this" => TokenType::This,
            "super" => TokenType::Super, "extends" => TokenType::Extends,
            "implements" => TokenType::Implements, "async" => TokenType::Async,
            "await" => TokenType::Await, "yield" => TokenType::Yield,
            "instanceof" => TokenType::InstanceOf, "in" => TokenType::In,
            "of" => TokenType::Of, "true" => TokenType::True,
            "false" => TokenType::False, "null" => TokenType::Null,
            "undefined" => TokenType::Undefined, "void" => TokenType::Void,
            "delete" => TokenType::Delete, "typeof" => TokenType::Typeof,
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
                        self.position += 1;
                    }
                }
                Some(&'\n') => {
                    self.emit_error(e1::UNTERMINATED_TEMPLATE, "Unterminated template");
                    return Token::new(TokenType::Template, value, self.span());
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
            _ => {
                self.position -= 1;
                Token::new(TokenType::Slash, "/".into(), self.span())
            }
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
        self.position += 1;
        let mut value = initial.to_string();
        
        let kind = match initial {
            '&' if self.next_is('&') => {
                self.position += 1;
                return Some(Token::new(TokenType::AndAnd, "&&".into(), self.span()));
            }
            '|' if self.next_is('|') => {
                self.position += 1;
                return Some(Token::new(TokenType::OrOr, "||".into(), self.span()));
            }
            '=' if self.next_is('=') => {
                self.position += 1;
                if self.next_is('=') {
                    self.position += 1;
                    return Some(Token::new(TokenType::EqEqEq, "===".into(), self.span()));
                }
                return Some(Token::new(TokenType::EqEquals, "==".into(), self.span()));
            }
            '!' if self.next_is('=') => {
                self.position += 1;
                if self.next_is('=') {
                    self.position += 1;
                    return Some(Token::new(TokenType::NeqNeq, "!==".into(), self.span()));
                }
                return Some(Token::new(TokenType::Neq, "!=".into(), self.span()));
            }
            '<' if self.next_is('=') => {
                self.position += 1;
                return Some(Token::new(TokenType::LtEq, "<=".into(), self.span()));
            }
            '<' if self.next_is('<') => {
                self.position += 1;
                return Some(Token::new(TokenType::LtLt, "<<".into(), self.span()));
            }
            '>' if self.next_is('=') => {
                self.position += 1;
                return Some(Token::new(TokenType::GtEq, ">=".into(), self.span()));
            }
            '>' if self.next_is('>') => {
                self.position += 1;
                if self.next_is('>') {
                    self.position += 1;
                    return Some(Token::new(TokenType::GtGt, ">>>".into(), self.span()));
                }
                return Some(Token::new(TokenType::GtGt, ">>".into(), self.span()));
            }
            '?' if self.next_is('=') => {
                self.position += 1;
                return Some(Token::new(TokenType::NullCoalesce, "??".into(), self.span()));
            }
            '?' if self.next_is('?') => {
                self.position += 1;
                return Some(Token::new(TokenType::NullCoalesce, "??".into(), self.span()));
            }
            '.' if self.next_is('.') && self.nth_is(2, '.') => {
                self.position += 1;
                self.position += 1;
                return Some(Token::new(TokenType::DotDotDot, "...".into(), self.span()));
            }
            '=' if self.next_is('>') => {
                self.position += 1;
                return Some(Token::new(TokenType::Arrow, "=>".into(), self.span()));
            }
            '+' if self.next_is('+') => {
                self.position += 1;
                value.push('+');
                TokenType::Plus
            }
            '-' if self.next_is('-') => {
                self.position += 1;
                value.push('-');
                TokenType::Minus
            }
            _ => match initial {
                '+' => TokenType::Plus,
                '-' => TokenType::Minus,
                '*' => TokenType::Star,
                '/' => TokenType::Slash,
                '&' => TokenType::Ampersand,
                '|' => TokenType::Pipe,
                '!' => TokenType::Not,
                '<' => TokenType::Lt,
                '>' => TokenType::Gt,
                '?' => TokenType::Question,
                '=' => TokenType::Eq,
                ':' => TokenType::Colon,
                '.' => TokenType::Dot,
                '#' => TokenType::Hash,
                '@' => TokenType::At,
                '~' => TokenType::Tilde,
                '(' => TokenType::LeftParen,
                ')' => TokenType::RightParen,
                '{' => TokenType::LeftBrace,
                '}' => TokenType::RightBrace,
                '[' => TokenType::LeftBracket,
                ']' => TokenType::RightBracket,
                ';' => TokenType::Semicolon,
                ',' => TokenType::Comma,
                '`' => TokenType::Backtick,
                _ => TokenType::Unknown,
            }
        };
        Some(Token::new(kind, value, self.span()))
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.position).copied()
    }

    fn nth(&self, n: usize) -> Option<char> {
        self.source.get(self.position + n).copied()
    }

    fn next_is(&self, c: char) -> bool {
        self.nth(1) == Some(c)
    }

    fn nth_is(&self, n: usize, c: char) -> bool {
        self.nth(n) == Some(c)
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
mod tests {
    use super::*;

    #[test]
    fn test_lexer_eof() {
        let lexer = Lexer::new(FileId::new(0), String::new());
        let result = lexer.lex_all();
        assert_eq!(result.tokens.len(), 1);
        assert_eq!(result.tokens[0].kind, TokenType::Eof);
    }

    #[test]
    fn test_lexer_function() {
        let mut lexer = Lexer::new(FileId::new(0), "function".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenType::Function);
    }

    #[test]
    fn test_lexer_identifier() {
        let mut lexer = Lexer::new(FileId::new(0), "x".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenType::Identifier);
    }

    #[test]
    fn test_lexer_number() {
        let mut lexer = Lexer::new(FileId::new(0), "42".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenType::NumericLiteral);
    }

    #[test]
    fn test_lexer_string() {
        let mut lexer = Lexer::new(FileId::new(0), "\"hello\"".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenType::StringLiteral);
    }

    #[test]
    fn test_lexer_plus() {
        let mut lexer = Lexer::new(FileId::new(0), "+".to_string());
        let token = lexer.next_token().unwrap();
        assert_eq!(token.kind, TokenType::Plus);
    }

    #[test]
    fn test_lexer_unterminated_string() {
        let mut lexer = Lexer::new(FileId::new(0), "\"hello".to_string());
        let result = lexer.lex_all();
        assert!(result.diagnostics.iter().any(|d| d.code == Some(e1::UNTERMINATED_STRING as u32)));
    }
}
