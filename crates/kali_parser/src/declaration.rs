//! Declaration parsing: functions, classes, parameters, arrow functions.

use crate::Parser;
use kali_ast::{
    ArrowFunctionExpression, BlockStatement, ClassBody, ClassDeclaration, ClassExpression,
    Expression, FunctionDeclaration, FunctionExpression, FunctionParam, MethodDefinition,
    SequenceExpression, Statement,
};
use kali_lexer::{Token, TokenType};
use std::boxed::Box;

/// Result of scanning a parenthesized parameter list.
///
/// The variants are exhaustive over "what the scanner found", and crucially
/// both `Simple` and `Unsupported` carry `after` — the token index just past
/// the matching `)`. That is what makes resynchronization unconditional: no
/// matter what the list contains, the caller knows where the list ends and can
/// continue parsing from there.
///
/// This replaced a loop that stopped consuming the moment it met a token it did
/// not recognize (`=`, `...`, `{`, `[`, `:`), leaving the stream parked
/// mid-list. `parse_block_statement` then `advance()`d over the stray token and
/// absorbed EVERY REMAINING TOKEN IN THE MODULE into the function body — no
/// diagnostic, exit code 0, every following statement silently skipped.
///
/// The classification is an ALLOWLIST: a segment yields a parameter only if it
/// matches a shape kali can actually lower. Everything else is `Unsupported` by
/// construction, so a newly-added parameter syntax fails closed instead of
/// silently desyncing the stream.
enum ParamListScan {
    /// Every segment was a plain named parameter (a type annotation is allowed
    /// and erased; a single trailing comma is allowed).
    Simple { after: usize, params: Vec<String> },
    /// The list is balanced but contains a construct kali cannot lower.
    /// `construct` is a noun phrase for the E5506 message.
    Unsupported {
        after: usize,
        construct: &'static str,
    },
    /// Not a parameter list at all: the start token is not `(`, or the parens
    /// never close before end-of-input.
    NotAParamList,
}

/// Result of scanning an arrow's parenthesized parameter list.
enum ArrowParams {
    Ok {
        after: usize,
        params: Vec<String>,
    },
    /// Provably an arrow parameter list (a `=>` follows the `)`) that kali
    /// cannot lower. The E5506 has already been reported; `after` indexes the
    /// `=>` so the caller can consume the whole arrow.
    Rejected {
        after: usize,
    },
    /// Not an arrow parameter list — the caller must fall back to its other
    /// interpretations (typically a parenthesized expression) unchanged.
    No,
}

impl Parser {
    /// Classifies one comma-separated parameter-list segment.
    ///
    /// `Ok(name)` for the two lowerable shapes — `ident` and `ident: Type` (the
    /// annotation is erased, which is what every consumer downstream of the
    /// parser expects). `Err(construct)` for everything else.
    fn classify_param_segment(segment: &[Token]) -> Result<String, &'static str> {
        let Some(first) = segment.first() else {
            return Err("an empty parameter");
        };
        match first.kind {
            TokenType::DotDotDot => Err("a rest parameter"),
            TokenType::LeftBrace | TokenType::LeftBracket => Err("a destructured parameter"),
            TokenType::Identifier => match segment.get(1).map(|token| &token.kind) {
                // `ident` — a plain parameter.
                None => Ok(first.value.clone()),
                // `ident: Type` — the annotation carries no runtime meaning.
                Some(TokenType::Colon) => Ok(first.value.clone()),
                // `ident = expr` — needs call-site arity adaptation, which
                // kali's codegen does not have (calls are emitted at exact
                // arity).
                Some(TokenType::Eq) => Err("a default parameter"),
                // `ident?: Type` — same arity problem as a default.
                Some(TokenType::Question) => Err("an optional parameter"),
                _ => Err("this parameter form"),
            },
            _ => Err("this parameter form"),
        }
    }

    /// Scans the parenthesized parameter list whose `(` is at `start`. Purely
    /// positional: it does not move the stream cursor.
    fn scan_param_list(&self, start: usize) -> ParamListScan {
        if self.stream.tokens.get(start).map(|token| &token.kind) != Some(&TokenType::LeftParen) {
            return ParamListScan::NotAParamList;
        }

        // Locate the matching `)`, tracking every bracket kind so that a
        // destructured or defaulted parameter containing punctuation cannot
        // fool the search.
        let mut depth = 0usize;
        let mut close = None;
        let mut index = start;
        while let Some(token) = self.stream.tokens.get(index) {
            match token.kind {
                TokenType::LeftParen | TokenType::LeftBrace | TokenType::LeftBracket => depth += 1,
                TokenType::RightBrace | TokenType::RightBracket => depth = depth.saturating_sub(1),
                TokenType::RightParen => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        close = Some(index);
                        break;
                    }
                }
                TokenType::Eof => break,
                _ => {}
            }
            index += 1;
        }
        let Some(close) = close else {
            return ParamListScan::NotAParamList;
        };
        let after = close + 1;

        let body = &self.stream.tokens[start + 1..close];
        if body.is_empty() {
            return ParamListScan::Simple {
                after,
                params: Vec::new(),
            };
        }

        // Split on top-level commas.
        let mut segments: Vec<&[Token]> = Vec::new();
        let mut depth = 0usize;
        let mut segment_start = 0usize;
        for (offset, token) in body.iter().enumerate() {
            match token.kind {
                TokenType::LeftParen | TokenType::LeftBrace | TokenType::LeftBracket => depth += 1,
                TokenType::RightParen | TokenType::RightBrace | TokenType::RightBracket => {
                    depth = depth.saturating_sub(1)
                }
                TokenType::Comma if depth == 0 => {
                    segments.push(&body[segment_start..offset]);
                    segment_start = offset + 1;
                }
                _ => {}
            }
        }
        segments.push(&body[segment_start..]);

        // A single trailing comma is legal and produces one empty final
        // segment; drop it. An empty segment anywhere else is a syntax error
        // and falls through to `classify_param_segment`'s rejection.
        if segments.len() > 1 && segments.last().is_some_and(|last| last.is_empty()) {
            segments.pop();
        }

        let mut params = Vec::with_capacity(segments.len());
        for segment in segments {
            match Self::classify_param_segment(segment) {
                Ok(name) => params.push(name),
                Err(construct) => return ParamListScan::Unsupported { after, construct },
            }
        }
        ParamListScan::Simple { after, params }
    }

    fn reject_unsupported_param(&mut self, construct: &'static str) {
        self.push_feature_unavailable(format!(
            "{construct} is not supported — kali functions take a fixed list of \
             plain named parameters"
        ));
    }

    /// Parses a parameter list with the stream positioned AT the opening `(`.
    ///
    /// Always leaves the cursor just past the matching `)` when one exists, so
    /// the caller can parse the body without risk of absorbing the rest of the
    /// module. An unsupported shape reports E5506 and yields no parameters; an
    /// unterminated list reports E5506 and consumes to end-of-input (there is
    /// nothing left to resynchronize to).
    pub(crate) fn parse_parameter_list(&mut self) -> Vec<String> {
        match self.scan_param_list(self.stream.position) {
            ParamListScan::Simple { after, params } => {
                self.stream.position = after;
                params
            }
            ParamListScan::Unsupported { after, construct } => {
                self.reject_unsupported_param(construct);
                self.stream.position = after;
                Vec::new()
            }
            ParamListScan::NotAParamList => {
                self.push_feature_unavailable(
                    "unterminated parameter list — expected a closing `)`".to_string(),
                );
                self.stream.position = self.stream.tokens.len();
                Vec::new()
            }
        }
    }

    /// Skips a return-type annotation (`): Type {`) if one is present, leaving
    /// the cursor on the body's `{`. Without this the `:` was another
    /// module-truncating desync — `parse_block_statement` would advance over it
    /// and swallow the file.
    pub(crate) fn skip_return_type_annotation(&mut self) {
        if self.stream.current_kind() != Some(&TokenType::Colon) {
            return;
        }
        let _ = self.stream.advance();
        let mut depth = 0usize;
        while let Some(kind) = self.stream.current_kind().copied() {
            match kind {
                TokenType::LeftBrace if depth == 0 => break,
                TokenType::LeftParen | TokenType::LeftBracket | TokenType::Lt => depth += 1,
                TokenType::RightParen | TokenType::RightBracket | TokenType::Gt => {
                    depth = depth.saturating_sub(1)
                }
                TokenType::Semicolon | TokenType::Eof => break,
                _ => {}
            }
            let _ = self.stream.advance();
        }
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
        let params = self.parse_parameter_list();
        self.skip_return_type_annotation();

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
                let params = self.parse_parameter_list();
                self.skip_return_type_annotation();
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

    /// Scans a parenthesized arrow parameter list starting at `start`, which
    /// must index a `LeftParen`. Shared by
    /// `try_parse_arrow_function_expression_from` (expression-bodied arrows,
    /// any position) and `try_parse_block_arrow_function_expression`
    /// (block-bodied arrows, declarator-init position only) — the two arrow
    /// shapes diverge after the parameter list.
    ///
    /// Unlike the function-declaration path this one must stay silent about
    /// lists it does not like, because at an arbitrary expression position
    /// `(a + b)` is a parenthesized expression, not a malformed parameter list.
    /// A diagnostic is therefore reported ONLY when a `=>` follows the closing
    /// `)`, which positively identifies the tokens as arrow parameters.
    fn scan_arrow_param_list(&mut self, start: usize) -> ArrowParams {
        match self.scan_param_list(start) {
            ParamListScan::Simple { after, params } => ArrowParams::Ok { after, params },
            ParamListScan::Unsupported { after, construct } => {
                if self.stream.tokens.get(after).map(|token| &token.kind) == Some(&TokenType::Arrow)
                {
                    self.reject_unsupported_param(construct);
                    ArrowParams::Rejected { after }
                } else {
                    ArrowParams::No
                }
            }
            ParamListScan::NotAParamList => ArrowParams::No,
        }
    }

    /// Consumes an arrow whose parameter list was already rejected, so the
    /// stream ends up past the arrow body instead of parked mid-expression.
    /// `after` indexes the `=>`.
    fn consume_rejected_arrow(&mut self, after: usize) -> Expression {
        self.stream.position = after + 1;
        if self.stream.current_kind() == Some(&TokenType::LeftBrace) {
            let _ = self.parse_block_statement();
        } else {
            let _ = self.parse_arrow_function_body_expression();
        }
        // The E5506 already reported makes compilation fail; this placeholder
        // only keeps the parser producing a well-formed tree for the remaining
        // diagnostics.
        Expression::Literal(kali_ast::LiteralValue::Null)
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
                match self.scan_arrow_param_list(scan) {
                    ArrowParams::Ok { after, params: p } => {
                        scan = after;
                        params = p;
                    }
                    ArrowParams::Rejected { after } => {
                        return Some(self.consume_rejected_arrow(after));
                    }
                    ArrowParams::No => return None,
                }
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
                id: None,
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
        let (scan, params) = match self.scan_arrow_param_list(start) {
            ArrowParams::Ok { after, params } => (after, params),
            ArrowParams::Rejected { after } => {
                return Some(self.consume_rejected_arrow(after));
            }
            ArrowParams::No => return None,
        };

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

        let params = self
            .parse_parameter_list()
            .into_iter()
            .map(|p| FunctionParam { name: p })
            .collect();
        self.skip_return_type_annotation();

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
