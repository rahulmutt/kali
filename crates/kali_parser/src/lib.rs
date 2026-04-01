//! Parser for TypeScript/JavaScript.
//!
//! This crate provides a recursive-descent parser that covers ECMA-262 grammar,
//! producing a typed AST with source spans and error recovery.

use kali_common::FileId;
use kali_error::diagnostic::Diagnostic;
use kali_lexer::{Lexer, Token, TokenType};

/// Result type for parsing operations.
pub type ParseResult<T> = Result<T, Vec<Diagnostic>>;

/// Token stream wrapper for parsing.
/// Provides a clean interface for lookahead and position tracking.
pub struct TokenStream {
    /// All tokens to parse.
    tokens: Vec<Token>,
    /// Current position in the token stream.
    position: usize,
}

impl TokenStream {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, position: 0 }
    }

    fn current(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn current_kind(&self) -> Option<&TokenType> {
        self.tokens.get(self.position).map(|t| &t.kind)
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.position).cloned()
    }

    fn eof(&self) -> bool {
        self.position >= self.tokens.len()
    }

    fn peek_at(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.position + offset)
    }

    fn advance(&mut self) -> Option<Token> {
        self.tokens.get(self.position).cloned().inspect(|_| {
            if self.position < self.tokens.len() {
                self.position += 1;
            }
        })
    }

    fn advance_if(&mut self, expected: TokenType) -> Option<Token> {
        if let Some(token) = self.current() {
            if token.kind == expected {
                self.advance()
            } else {
                None
            }
        } else {
            None
        }
    }

    fn skip(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }
}

/// Parser result containing parsed statements and diagnostics.
pub struct ParseResultOutput {
    /// Parsed statements.
    pub statements: Vec<kali_ast::Statement>,
    /// Diagnostics collected during parsing.
    pub diagnostics: Vec<Diagnostic>,
}

/// Parser skeleton for TypeScript/JavaScript grammar.
pub struct Parser {
    file_id: FileId,
    stream: TokenStream,
    diagnostics: Vec<Diagnostic>,
    /// Track if we're in JSX mode (for .jsx/.tsx files)
    jsx_mode: bool,
}

impl Parser {
    /// Create a new parser for the given tokens.
    pub fn new(file_id: FileId, tokens: Vec<Token>) -> Self {
        let stream = TokenStream::new(tokens);
        Self {
            file_id,
            stream,
            diagnostics: Vec::new(),
            jsx_mode: false,
        }
    }

    /// Parse the token stream into an AST.
    /// 
    /// TODO: Implement actual parsing logic for each ECMA-262 construct.
    pub fn parse(
        &mut self,
        _file_path: Option<String>,
    ) -> ParserOutput {
        let mut statements = Vec::new();
        
        // Parse module-level statements - stub implementation for Stage 1.3
        // This will be implemented per spec in full parsing:
        // Expression statements, declarations, import/export, etc.
        
        // For now, return empty parse result
        ParserOutput {
            root: kali_ast::AST::empty(),
            statements,
            diagnostics: self.diagnostics.clone(),
        }
    }

    /// Enable JSX mode for .jsx/.tsx files
    pub fn with_jsx(mut self, enable: bool) -> Self {
        self.jsx_mode = enable;
        self
    }

    /// Get diagnostics collected during parsing
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Add a diagnostic
    fn add_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }
}

pub struct ParserOutput {
    /// Parsed AST root node
    pub root: kali_ast::AST,
    /// Parsed statements
    pub statements: Vec<kali_ast::Statement>,
    /// Diagnostics collected during parsing
    pub diagnostics: Vec<Diagnostic>,
}
