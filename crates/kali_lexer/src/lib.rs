//! Tokenizer/lexer for TypeScript and JavaScript.
//!
//! This crate tokenizes source code into a stream of tokens
//! for consumption by the parser.

use kali_error::diagnostic::Diagnostic;
use kali_common::{FileId, Span};

/// Token types produced by the lexer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    // Single-character tokens
    Plus,           // +
    Minus,          // -
    Star,           // *
    Slash,          // /
    Caret,          // ^
    Percent,        // %
    And,            // &&
    Or,             // ||
    Not,            // !
    EqEquals,       // ==
    EqEqEq,         // ===
    Neq,            // !=
    NeqNeq,         // !==
    Lt,             // <
    Gt,             // >
    LtEq,           // <=
    GtEq,           // >=
    LtLt,           // <<
    LtLtLt,         // <<<
    GtGt,           // >>
    GtGtGt,         // >>>
    PlusEq,         // +=
    MinusEq,        // -=
    StarEq,         // *=
    SlashEq,        // /=
    CaretEq,        // ^=
    PercentEq,      // %=
    AndAnd,         // &&
    OrOr,           // ||
    QuestionDot,    // ?.
    Arrow,          // =>
    Colon,          // :
    Equals,         // =
    EqualsEquals,   // ==
    Ampersand,      // &
    Pipe,           // |
    Tilde,          // ~
    At,             // @
    Bang,           // !
    Dot,            // .
    DotDotDot,      // ...
    EqGreater,      // =>
    Hash,           // #
    LeftParen,      // (
    RightParen,     // )
    LeftBrace,      // {
    RightBrace,     // }
    LeftBracket,    // [
    RightBracket,   // ]
    DoubleColon,    // ::
    Semicolon,      // ;
    Comma,          // ,
    Backtick,       // `
    Question,       // ?
    QuestionEquals, // ?=
    AndAndEquals,   // &&=
    OrOrEquals,     // ||=
    NullCoalesce,   // ??
    NullCoalesceEq, // ??=
    ShiftRight,     // >>
    
    // Keywords
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
    With,
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
    Inout,

    // Literals
    Identifier,
    StringLiteral,
    NumericLiteral,
    Template,
    TemplateTail,
    RegularExpression,
    BooleanLiteral,
    
    // Pseudo tokens
    IdentifierWithModifier(String),
    NumberWithModifier(String, f64),
    Modifier(String),
    
    // End and errors
    Eof,
    Unknown,
}

/// A single token produced by the lexer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// The type of this token.
    pub kind: TokenType,
    /// The lexical value of this token.
    pub value: String,
    /// The span where this token appears.
    pub span: Span,
}

impl Token {
    /// Create a new token.
    pub fn new(kind: TokenType, value: String, span: Span) -> Self {
        Self { kind, value, span }
    }
}

/// Result of lexing a source file.
pub struct LexerResult {
    /// The tokens produced by the lexer.
    pub tokens: Vec<Token>,
    /// Diagnostics encountered during lexing.
    pub diagnostics: Vec<Diagnostic>,
}

/// A tokenizer/streaming lexer for TypeScript/JavaScript.
pub struct Lexer {
    /// Source code being tokenized.
    source: String,
    /// File ID for this source.
    file_id: FileId,
    /// Current position in the source.
    position: usize,
    /// Current line number (1-indexed).
    line_number: usize,
    /// Column number (1-indexed, line-based).
    column_number: usize,
    /// Collected diagnostic messages.
    diagnostics: Vec<Diagnostic>,
}

impl Lexer {
    /// Create a new Lexer for the given source.
    pub fn new(file_id: FileId, source: String) -> Self {
        Self {
            source,
            file_id,
            position: 0,
            line_number: 1,
            column_number: 1,
            diagnostics: Vec::new(),
        }
    }

    /// Create a new Lexer for the given file path.
    pub fn for_file(file_id: FileId, source_path: impl Into<String>) -> Self {
        let _path = source_path.into();
        // In real implementation, read file contents here
        Self::new(file_id, String::new())
    }

    /// Get diagnostics encountered during lexing.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Get the next token in the sequence.
    /// Returns `None` when the end of the file is reached.
    pub fn next_token(&mut self) -> Option<Token> {
        // Skip whitespace
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }

        // Check EOF
        if self.is_eof() {
            return Some(Token::new(TokenType::Eof, String::new(), self.span()));
        }

        // Tokenize based on first character
        match self.peek() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$' => {
                self.lex_identifier()
            }
            Some(c) if c.is_ascii_digit() => {
                self.lex_number()
            }
            Some('"') | Some('\'') | Some('`') => {
                self.lex_string()
            }
            Some('/') if self.peek_next() == Some('*') => {
                self.lex_block_comment()
            }
            Some('/') if self.peek_next() == Some('/') => {
                self.lex_line_comment()
            }
            Some(c) => {
                self.lex_single_char(c)
            }
            None => {
                Some(Token::new(TokenType::Eof, String::new(), self.span()))
            }
        }
    }

    /// Lex all tokens in the source.
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

    /// Lex an identifier or keyword.
    fn lex_identifier(&mut self) -> Token {
        // Implementation placeholder
        // TODO: implement actual identifier lexing
        self.advance();
        Token::new(TokenType::Identifier, String::new(), self.span())
    }

    /// Lex a numeric literal.
    fn lex_number(&mut self) -> Token {
        // Implementation placeholder
        // TODO: implement actual number lexing
        self.advance();
        Token::new(TokenType::NumericLiteral, String::new(), self.span())
    }

    /// Lex a string or template literal.
    fn lex_string(&mut self) -> Token {
        // Implementation placeholder
        // TODO: implement actual string lexing
        self.advance();
        Token::new(TokenType::StringLiteral, String::new(), self.span())
    }

    /// Lex a block comment (/* ... */).
    fn lex_block_comment(&mut self) -> Token {
        // Implementation placeholder
        // TODO: implement actual block comment lexing
        self.advance();
        self.advance();
        Token::new(TokenType::Unknown, String::new(), self.span())
    }

    /// Lex a line comment (// ...).
    fn lex_line_comment(&mut self) -> Token {
        // Implementation placeholder
        // TODO: implement actual line comment lexing
        self.advance();
        self.advance();
        Token::new(TokenType::Unknown, String::new(), self.span())
    }

    /// Lex a single-character token.
    fn lex_single_char(&mut self, c: char) -> Token {
        self.advance();
        let value = c.to_string();
        Token::new(TokenType::Unknown, value, self.span())
    }

    /// Peek at the current character.
    fn peek(&self) -> Option<char> {
        self.source.chars().nth(self.position)
    }

    /// Peek at the next character.
    fn peek_next(&self) -> Option<char> {
        self.source.chars().nth(self.position + 1)
    }

    /// Advance the lexer position.
    fn advance(&mut self) {
        if let Some(c) = self.next() {
            self.position += c.len_utf8();
            if c == '\n' {
                self.line_number += 1;
                self.column_number = 1;
            } else {
                self.column_number += 1;
            }
        }
    }

    /// Get the current position in the source.
    fn position(&self) -> usize {
        self.position
    }

    /// Get the next character.
    fn next(&self) -> Option<char> {
        self.source.chars().nth(self.position)
    }

    /// Check if we're at the end of the file.
    fn is_eof(&self) -> bool {
        self.position >= self.source.len()
    }

    /// Get the current span.
    fn span(&self) -> Span {
        Span::new(
            self.file_id,
            self.position as u32,
            self.position as u32,
        )
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
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_lexer_simple_tokens() {
        let source = "x + 1;";
        let mut lexer = Lexer::new(FileId::new(0), source.to_string());
        let result = lexer.lex_all();
        
        assert!(!result.tokens.is_empty());
        assert!(result.tokens.iter().any(|t| t.kind == TokenType::Eof));
    }

    #[test]
    fn test_lexer_whitespace_handling() {
        let source = "   a
    b   ";
        let mut lexer = Lexer::new(FileId::new(0), source.to_string());
        
        let token = lexer.next_token();
        assert!(token.is_some());
        assert_eq!(token.unwrap().kind, TokenType::Identifier);
    }
}
