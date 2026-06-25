//! Parser entry point: struct, constructor, top-level `parse`, shared helpers.

use crate::TokenStream;
use kali_ast::{ASTBuilder, BlockStatement, Statement, AST};
use kali_common::FileId;
use kali_error::{_error_codes::e5, diagnostic::Diagnostic};
use kali_lexer::{Token, TokenType};

pub struct Parser {
    pub(crate) file_id: FileId,
    pub(crate) stream: TokenStream,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) jsx_mode: bool,
    pub(crate) in_generator_function: bool,
    pub(crate) in_async_function: bool,
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
