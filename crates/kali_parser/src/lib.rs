#![allow(dead_code)]
//! Basic parser for T/JS.

use kali_common::FileId;
use kali_error::diagnostic::Diagnostic;
use kali_lexer::{Token, TokenType};

use kali_ast::{AST, ASTBuilder, Statement, VariableDeclarator};

pub struct TokenStream { tokens: Vec<Token>, position: usize }

impl TokenStream {
    fn new(tokens: Vec<Token>) -> Self { Self { tokens, position: 0 } }
    fn current(&self) -> Option<&Token> { self.tokens.get(self.position) }
    fn current_kind(&self) -> Option<&TokenType> { self.tokens.get(self.position).map(|t| &t.kind) }
    fn eof(&self) -> bool { self.position >= self.tokens.len() }
    fn advance(&mut self) -> Option<Token> { if self.position < self.tokens.len() { let tok = self.tokens[self.position].clone(); self.position += 1; Some(tok) } else { None } }
    fn advance_if(&mut self, expected: TokenType) -> bool { if self.current_kind() == Some(&expected) { self.position += 1; true } else { false } }
    fn accept(&mut self, k: TokenType) -> bool { self.advance_if(k) }
}

pub struct Parser { file_id: FileId, stream: TokenStream, diagnostics: Vec<Diagnostic>, jsx_mode: bool }

impl Parser {
    pub fn new(file_id: FileId, tokens: Vec<Token>) -> Self {
        Self { file_id, stream: TokenStream::new(tokens), diagnostics: Vec::new(), jsx_mode: false }
      }
    pub fn parse(&mut self, _path: Option<String>) -> ParserOutput {
        let mut statements = Vec::new();
        while !self.stream.eof() {
            if let Some(stmt) = self.parse_statement() { statements.push(stmt); } else { self.stream.position += 1; }
           }
          ParserOutput { root: ASTBuilder::new().into_ast(), statements, diagnostics: self.diagnostics.clone() }
        }
    fn parse_statement(&mut self) -> Option<Statement> {
        let kind = self.stream.current_kind()?;
        match kind {
            TokenType::Var | TokenType::Let | TokenType::Const => {
                self.stream.advance();
                let id = self.stream.advance().map(|t| t.value).unwrap_or_default();
                self.stream.accept(TokenType::Semicolon);
                let decl = kali_ast::VariableDeclaration { kind: "const".to_string(), declarations: vec![VariableDeclarator { id, init: None }] };
                Some(Statement::VariableDeclaration(decl))
              }
              _ => None,
              }
              }
}

pub struct ParserOutput { pub root: AST, pub statements: Vec<Statement>, pub diagnostics: Vec<Diagnostic> }
