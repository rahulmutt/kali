//! Token cursor over the lexer output.

use kali_lexer::{Token, TokenType};

pub struct TokenStream {
    pub(crate) tokens: Vec<Token>,
    pub(crate) position: usize,
}

impl TokenStream {
    pub(crate) fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    pub(crate) fn current(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    pub(crate) fn current_kind(&self) -> Option<&TokenType> {
        self.tokens.get(self.position).map(|t| &t.kind)
    }

    pub(crate) fn peek_next_kind(&self) -> Option<&TokenType> {
        self.tokens.get(self.position + 1).map(|t| &t.kind)
    }

    pub(crate) fn eof(&self) -> bool {
        self.tokens.is_empty() || self.position >= self.tokens.len()
    }

    pub(crate) fn advance(&mut self) -> Option<Token> {
        if self.position < self.tokens.len() {
            let tok = self.tokens[self.position].clone();
            self.position += 1;
            Some(tok)
        } else {
            None
        }
    }

    pub(crate) fn advance_if(&mut self, expected: TokenType) -> bool {
        if self.current_kind() == Some(&expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }

    pub(crate) fn accept(&mut self, k: TokenType) -> bool {
        self.advance_if(k)
    }

    pub(crate) fn skip(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }
}
