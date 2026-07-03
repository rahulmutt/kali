//! Declaration parsing: functions, classes, parameters, arrow functions.

use crate::Parser;
use kali_ast::{
    ArrowFunctionExpression, BlockStatement, ClassBody, ClassDeclaration, ClassExpression,
    Expression, FunctionDeclaration, FunctionExpression, FunctionParam, MethodDefinition,
    SequenceExpression, Statement,
};
use kali_lexer::TokenType;
use std::boxed::Box;

impl Parser {
    pub(crate) fn parse_parameter_list(&mut self) -> Vec<String> {
        let mut params = Vec::new();
        if self.stream.accept(TokenType::RightParen) {
            return params;
        }

        loop {
            if let Some(token) = self.stream.advance() {
                if token.kind == TokenType::Identifier {
                    params.push(token.value);
                }
            }
            if self.stream.accept(TokenType::RightParen) {
                break;
            }
            if !self.stream.accept(TokenType::Comma) {
                let _ = self.stream.accept(TokenType::RightParen);
                break;
            }
        }

        params
    }

    pub(crate) fn parse_function_declaration(&mut self) -> Option<Statement> {
        self.parse_function_declaration_with_async(false, false)
    }

    pub(crate) fn parse_function_declaration_with_async(
        &mut self,
        is_async: bool,
        allow_anonymous: bool,
    ) -> Option<Statement> {
        if is_async {
            let _ = self.stream.advance();
        }
        let previous_async = self.in_async_function;
        self.in_async_function = is_async;
        let _ = self.stream.advance();
        let generator = if self.stream.current_kind() == Some(&TokenType::Star) {
            let _ = self.stream.advance();
            true
        } else {
            false
        };
        let name = if allow_anonymous && self.stream.current_kind() == Some(&TokenType::LeftParen) {
            String::new()
        } else {
            let name_token = self.stream.advance()?;
            if name_token.kind != TokenType::Identifier {
                return None;
            }
            name_token.value
        };
        let _ = self.stream.accept(TokenType::LeftParen);

        let mut params = Vec::new();
        if !self.stream.accept(TokenType::RightParen) {
            if let Some(param) = self.stream.advance() {
                params.push(param.value);
            }
            while self.stream.accept(TokenType::Comma) {
                if let Some(param) = self.stream.advance() {
                    params.push(param.value);
                }
            }
            let _ = self.stream.accept(TokenType::RightParen);
        }

        let previous_generator = self.in_generator_function;
        self.in_generator_function = generator;
        let body_block = match self.parse_block_statement() {
            Some(Statement::BlockStatement(bs)) => bs,
            _ => BlockStatement { body: Vec::new() },
        };
        self.in_generator_function = previous_generator;
        self.in_async_function = previous_async;

        Some(Statement::FunctionDeclaration(FunctionDeclaration {
            name,
            params,
            body: Box::new(body_block),
            is_async,
            generator,
        }))
    }

    pub(crate) fn parse_class_body(&mut self) -> ClassBody {
        let _ = self.stream.accept(TokenType::LeftBrace);

        let mut methods = Vec::new();
        loop {
            if self.stream.eof() || self.stream.current_kind() == Some(&TokenType::RightBrace) {
                let _ = self.stream.accept(TokenType::RightBrace);
                break;
            }

            let is_async = if self.stream.current_kind() == Some(&TokenType::Async)
                && matches!(
                    self.stream.peek_next_kind(),
                    Some(TokenType::Star) | Some(TokenType::Identifier)
                ) {
                let _ = self.stream.advance();
                true
            } else {
                false
            };
            let generator = if self.stream.current_kind() == Some(&TokenType::Star) {
                let _ = self.stream.advance();
                true
            } else {
                false
            };

            let is_method = matches!(self.stream.current_kind(), Some(TokenType::Identifier))
                && matches!(self.stream.peek_next_kind(), Some(TokenType::LeftParen))
                || matches!(self.stream.current_kind(), Some(TokenType::Async))
                    && matches!(self.stream.peek_next_kind(), Some(TokenType::LeftParen));

            if is_method {
                let method_name = self.stream.advance().map(|t| t.value).unwrap_or_default();
                let _ = self.stream.accept(TokenType::LeftParen);
                let params = self.parse_parameter_list();
                let previous_async = self.in_async_function;
                let previous_generator = self.in_generator_function;
                self.in_async_function = is_async;
                self.in_generator_function = generator;
                let body = match self.parse_block_statement() {
                    Some(Statement::BlockStatement(bs)) => bs,
                    _ => BlockStatement { body: Vec::new() },
                };
                self.in_generator_function = previous_generator;
                self.in_async_function = previous_async;
                methods.push(MethodDefinition {
                    name: method_name,
                    params,
                    body: Some(Box::new(body)),
                    is_async,
                    generator,
                });
            } else {
                let _ = self.stream.advance();
            }
        }

        ClassBody { methods }
    }

    pub(crate) fn parse_class_declaration(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();
        let name_token = self.stream.advance()?;
        let name = name_token.value;
        let body = self.parse_class_body();

        Some(Statement::ClassDeclaration(ClassDeclaration {
            name,
            body: Box::new(body),
        }))
    }

    pub(crate) fn parse_class_expression(&mut self) -> Expression {
        let _ = self.stream.advance();
        let id = if self.stream.current_kind() == Some(&TokenType::Identifier) {
            self.stream.advance().map(|token| token.value)
        } else {
            None
        };
        let body = self.parse_class_body();

        Expression::ClassExpression(Box::new(ClassExpression {
            id,
            body: Box::new(body),
        }))
    }

    pub(crate) fn try_parse_arrow_function_expression(&mut self) -> Option<Expression> {
        self.try_parse_arrow_function_expression_from(self.stream.position, false)
    }

    /// Scans a parenthesized, identifier-only parameter list — `()` or
    /// `(a, b, c)` — starting at `start`, which must index a `LeftParen`
    /// token. Returns the parameter names in order and the token position
    /// immediately after the closing `RightParen`, or `None` if the tokens
    /// ahead are not a well-formed identifier-only parameter list. Shared by
    /// `try_parse_arrow_function_expression_from` (expression-bodied arrows,
    /// any position) and `try_parse_block_arrow_function_expression`
    /// (block-bodied arrows, declarator-init position only) — the two arrow
    /// shapes diverge after the parameter list (return-type annotation and
    /// bare-identifier-param arms are exclusive to the former).
    fn scan_paren_param_list(&self, start: usize) -> Option<(usize, Vec<String>)> {
        let mut scan = start + 1;
        let mut params = Vec::new();
        match self.stream.tokens.get(scan).map(|token| &token.kind) {
            Some(TokenType::RightParen) => {
                scan += 1;
            }
            Some(TokenType::Identifier) => loop {
                let token = self.stream.tokens.get(scan)?;
                params.push(token.value.clone());
                scan += 1;

                match self.stream.tokens.get(scan).map(|token| &token.kind) {
                    Some(TokenType::Comma) => {
                        scan += 1;
                    }
                    Some(TokenType::RightParen) => {
                        scan += 1;
                        break;
                    }
                    _ => return None,
                }
            },
            _ => return None,
        }
        Some((scan, params))
    }

    pub(crate) fn try_parse_arrow_function_expression_from(
        &mut self,
        start: usize,
        is_async: bool,
    ) -> Option<Expression> {
        let mut scan = start;
        let mut params = Vec::new();
        let mut allow_return_type = false;
        match self.stream.tokens.get(scan).map(|token| &token.kind) {
            Some(TokenType::LeftParen) => {
                allow_return_type = true;
                let (next_scan, parsed_params) = self.scan_paren_param_list(scan)?;
                scan = next_scan;
                params = parsed_params;
            }
            Some(TokenType::Identifier) => {
                let token = self.stream.tokens.get(scan)?;
                params.push(token.value.clone());
                scan += 1;
            }
            _ => return None,
        }

        let mut return_type = None;
        if allow_return_type
            && self.stream.tokens.get(scan).map(|token| &token.kind) == Some(&TokenType::Colon)
        {
            let saved_position = self.stream.position;
            self.stream.position = scan + 1;
            let parsed_return_type = self.parse_type_reference_text();
            scan = self.stream.position;
            self.stream.position = saved_position;
            if parsed_return_type.is_empty() {
                return None;
            }
            return_type = Some(parsed_return_type);
        }

        if self.stream.tokens.get(scan).map(|token| &token.kind) != Some(&TokenType::Arrow) {
            return None;
        }

        if self.stream.tokens.get(scan + 1).map(|token| &token.kind) == Some(&TokenType::LeftBrace)
        {
            return None;
        }

        self.stream.position = scan + 1;
        let body = self.parse_arrow_function_body_expression();
        Some(Expression::ArrowFunctionExpression(Box::new(
            ArrowFunctionExpression {
                params: params
                    .into_iter()
                    .map(|name| FunctionParam { name })
                    .collect(),
                body,
                is_async,
                returnType: return_type,
            },
        )))
    }

    /// Parses `(params) => { statements }` — a block-bodied arrow — into an
    /// unnamed `FunctionExpression`. Only invoked from variable-declarator init
    /// position (`parse_variable_declaration`); every other position keeps the
    /// legacy behavior so the `Kali.test('…', () => { … })` callback lane is
    /// untouched. Returns `None` (with the stream position unchanged) unless
    /// the tokens ahead are exactly a paren parameter list, `=>`, then `{`.
    pub(crate) fn try_parse_block_arrow_function_expression(&mut self) -> Option<Expression> {
        let start = self.stream.position;
        if self.stream.tokens.get(start).map(|token| &token.kind) != Some(&TokenType::LeftParen) {
            return None;
        }
        let (scan, params) = self.scan_paren_param_list(start)?;

        if self.stream.tokens.get(scan).map(|token| &token.kind) != Some(&TokenType::Arrow) {
            return None;
        }
        if self.stream.tokens.get(scan + 1).map(|token| &token.kind) != Some(&TokenType::LeftBrace)
        {
            return None;
        }

        self.stream.position = scan + 1;
        let Some(Statement::BlockStatement(block)) = self.parse_block_statement() else {
            self.stream.position = start;
            return None;
        };
        Some(Expression::FunctionExpression(Box::new(
            FunctionExpression {
                id: None,
                params: params
                    .into_iter()
                    .map(|name| FunctionParam { name })
                    .collect(),
                body: Some(Box::new(block)),
                is_async: false,
                generator: false,
            },
        )))
    }

    pub(crate) fn parse_arrow_function_body_expression(&mut self) -> Expression {
        if self.stream.current_kind() == Some(&TokenType::LeftBrace) {
            let _ = self.stream.advance();
            let mut expressions = Vec::new();

            while !self.stream.eof() && self.stream.current_kind() != Some(&TokenType::RightBrace) {
                if self.stream.current_kind() == Some(&TokenType::Semicolon) {
                    let _ = self.stream.advance();
                    continue;
                }

                expressions.push(self.parse_expression());
                let _ = self.stream.accept(TokenType::Semicolon);
            }

            let _ = self.stream.accept(TokenType::RightBrace);

            match expressions.len() {
                0 => Expression::Literal(kali_ast::LiteralValue::Null),
                1 => expressions.pop().unwrap(),
                _ => Expression::SequenceExpression(Box::new(SequenceExpression { expressions })),
            }
        } else {
            self.parse_expression()
        }
    }

    pub(crate) fn parse_function_expression(&mut self) -> Expression {
        self.parse_function_expression_with_async(false)
    }

    pub(crate) fn parse_function_expression_with_async(&mut self, is_async: bool) -> Expression {
        if is_async {
            let _ = self.stream.advance();
        }
        let previous_async = self.in_async_function;
        self.in_async_function = is_async;
        let _ = self.stream.advance();
        let generator = if self.stream.current_kind() == Some(&TokenType::Star) {
            let _ = self.stream.advance();
            true
        } else {
            false
        };

        let id = if self.stream.current_kind() == Some(&TokenType::Identifier)
            && self.stream.peek_next_kind() == Some(&TokenType::LeftParen)
        {
            self.stream.advance().map(|t| t.value)
        } else {
            None
        };

        let _ = self.stream.accept(TokenType::LeftParen);
        let params = self
            .parse_parameter_list()
            .into_iter()
            .map(|p| FunctionParam { name: p })
            .collect();

        let previous_generator = self.in_generator_function;
        self.in_generator_function = generator;
        let body = self
            .parse_block_statement()
            .unwrap_or(Statement::BlockStatement(BlockStatement {
                body: Vec::new(),
            }));
        self.in_generator_function = previous_generator;
        self.in_async_function = previous_async;
        let func_body = match body {
            Statement::BlockStatement(bs) => Some(Box::new(bs)),
            _ => Some(Box::new(BlockStatement { body: Vec::new() })),
        };

        Expression::FunctionExpression(Box::new(FunctionExpression {
            id,
            params,
            body: func_body,
            is_async,
            generator,
        }))
    }
}

#[cfg(test)]
#[path = "declaration_tests.rs"]
mod declaration_tests;
