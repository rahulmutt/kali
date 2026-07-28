//! Parser entry point: struct, constructor, top-level `parse`, shared helpers.

use crate::TokenStream;
use kali_ast::{ASTBuilder, BlockStatement, Statement, AST};
use kali_common::FileId;
use kali_error::{
    _error_codes::{e2, e5},
    diagnostic::Diagnostic,
};
use kali_lexer::{Token, TokenType};

pub struct Parser {
    pub(crate) file_id: FileId,
    pub(crate) stream: TokenStream,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) jsx_mode: bool,
    pub(crate) in_generator_function: bool,
    pub(crate) in_async_function: bool,
    /// True while parsing the expression form of a `for (` head, where a
    /// trailing `in` belongs to the for-in statement and must terminate the
    /// expression instead of being treated as (rejected) binary `in`.
    pub(crate) no_in: bool,
}

impl Parser {
    pub fn new(file_id: FileId, tokens: Vec<Token>) -> Self {
        Self {
            file_id,
            stream: TokenStream::new(tokens),
            diagnostics: Vec::new(),
            jsx_mode: false,
            in_generator_function: false,
            in_async_function: false,
            no_in: false,
        }
    }

    pub(crate) fn wrap_statement_as_block(stmt: Statement) -> BlockStatement {
        match stmt {
            Statement::BlockStatement(block) => block,
            other => BlockStatement { body: vec![other] },
        }
    }

    pub(crate) fn push_feature_unavailable(&mut self, message: impl Into<String>) {
        self.diagnostics.push(Diagnostic::error(
            e5::FEATURE_UNAVAILABLE as u32,
            message.into(),
        ));
    }

    /// Consume `kind` if present; otherwise report `E2000` and consume nothing.
    ///
    /// The parser had only `accept -> bool`, so every REQUIRED-token position
    /// was a blind `advance()` or a discarded bool — each silently accepting
    /// whatever token happened to be there. `e2::EXPECTED_TOKEN` was declared
    /// in `kali_error` and emitted nowhere in the compiler.
    ///
    /// Returns whether the token was consumed, so a caller can decide between
    /// continuing and bailing. It deliberately does NOT skip the offending
    /// token: recovery stays the caller's decision.
    pub(crate) fn expect(&mut self, kind: TokenType) -> bool {
        if self.stream.accept(kind) {
            return true;
        }
        let found = match self.stream.current_kind() {
            Some(k) => format!("{k:?}"),
            None => "end of input".to_string(),
        };
        self.diagnostics.push(Diagnostic::error(
            e2::EXPECTED_TOKEN as u32,
            format!("expected {kind:?} but found {found}"),
        ));
        false
    }

    pub(crate) fn skip_class_body(&mut self) {
        let mut brace_depth = 0usize;

        while !self.stream.eof() {
            match self.stream.current_kind() {
                Some(TokenType::LeftBrace) => {
                    brace_depth += 1;
                    let _ = self.stream.advance();
                }
                Some(TokenType::RightBrace) => {
                    let _ = self.stream.advance();
                    if brace_depth == 0 {
                        break;
                    }
                    brace_depth -= 1;
                }
                _ => {
                    let _ = self.stream.advance();
                }
            }
        }
    }

    pub fn parse(&mut self, _path: Option<String>) -> ParserOutput {
        let mut statements = Vec::new();
        while !self.stream.eof() {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            } else {
                // Statement parsing failed, advance to avoid infinite loop
                let _ = self.stream.advance();
            }
        }

        let root = ASTBuilder::new().into_ast();
        ParserOutput {
            root,
            statements,
            diagnostics: self.diagnostics.clone(),
        }
    }

    pub(crate) fn current_token_value_is(&self, value: &str) -> bool {
        self.stream
            .current()
            .is_some_and(|token| token.value == value)
    }
}

pub struct ParserOutput {
    pub root: AST,
    pub statements: Vec<Statement>,
    pub diagnostics: Vec<Diagnostic>,
}
