//! Declaration parsing: functions, classes, parameters, arrow functions.

use crate::Parser;
use kali_ast::{
    ArrowFunctionExpression, BlockStatement, ClassBody, ClassDeclaration, ClassExpression,
    ClassField, Expression, FunctionDeclaration, FunctionExpression, FunctionParam,
    MethodDefinition, MethodKind, SequenceExpression, Statement,
};
use kali_lexer::{Token, TokenType};
use std::boxed::Box;

/// One scanned parameter: its name and, when it has a default, the absolute
/// token range of the default's expression (the tokens after `=`).
struct ScannedParam {
    name: String,
    default: Option<std::ops::Range<usize>>,
}

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
    Simple {
        after: usize,
        params: Vec<ScannedParam>,
    },
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
        /// A parameter had a default. The refusal is deferred to the caller,
        /// which reports it only once the arrow's `=>` is confirmed (past any
        /// return-type annotation); `(b = 6)` is a parenthesized assignment.
        had_default: bool,
    },
    /// Provably an arrow parameter list (a `=>` follows the `)`) that kali
    /// cannot lower. The E5506 has already been reported; `after` indexes the
    /// `=>` so the caller can consume the whole arrow.
    Rejected { after: usize },
    /// Not an arrow parameter list — the caller must fall back to its other
    /// interpretations (typically a parenthesized expression) unchanged.
    No,
}

/// Words that may precede a class member's key.
const CLASS_MEMBER_MODIFIERS: &[&str] = &[
    "static",
    "get",
    "set",
    "readonly",
    "public",
    "private",
    "protected",
    "abstract",
    "override",
    "declare",
    "accessor",
];

impl Parser {
    /// Classifies one comma-separated parameter-list segment.
    ///
    /// `Ok((name, None))` for `ident` and `ident: Type` (the annotation is
    /// erased). `Ok((name, Some(offset)))` for `ident = expr` and
    /// `ident: Type = expr`, where `offset` indexes the `=` within the
    /// segment (default-parameters spec 3.1). `Err(construct)` for
    /// everything else.
    fn classify_param_segment(segment: &[Token]) -> Result<(String, Option<usize>), &'static str> {
        let Some(first) = segment.first() else {
            return Err("an empty parameter");
        };
        match first.kind {
            TokenType::DotDotDot => Err("a rest parameter"),
            TokenType::LeftBrace | TokenType::LeftBracket => Err("a destructured parameter"),
            TokenType::Identifier => match segment.get(1).map(|token| &token.kind) {
                None => Ok((first.value.clone(), None)),
                Some(TokenType::Eq) => Ok((first.value.clone(), Some(1))),
                Some(TokenType::Colon) => Ok((first.value.clone(), Self::top_level_eq(segment, 2))),
                // `ident?: Type` has no default to fill in, so the call-site
                // rewrite cannot adapt its arity.
                Some(TokenType::Question) => Err("an optional parameter"),
                _ => Err("this parameter form"),
            },
            _ => Err("this parameter form"),
        }
    }

    /// The index of the first `=` at bracket depth 0 in `segment[from..]`.
    fn top_level_eq(segment: &[Token], from: usize) -> Option<usize> {
        let mut depth = 0usize;
        for (index, token) in segment.iter().enumerate().skip(from) {
            match token.kind {
                TokenType::LeftParen | TokenType::LeftBrace | TokenType::LeftBracket => depth += 1,
                TokenType::RightParen | TokenType::RightBrace | TokenType::RightBracket => {
                    depth = depth.saturating_sub(1)
                }
                TokenType::Eq if depth == 0 => return Some(index),
                _ => {}
            }
        }
        None
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

        // Split on top-level commas, keeping each segment's absolute start.
        let mut segments: Vec<(usize, &[Token])> = Vec::new();
        let mut depth = 0usize;
        let mut segment_start = 0usize;
        for (offset, token) in body.iter().enumerate() {
            match token.kind {
                TokenType::LeftParen | TokenType::LeftBrace | TokenType::LeftBracket => depth += 1,
                TokenType::RightParen | TokenType::RightBrace | TokenType::RightBracket => {
                    depth = depth.saturating_sub(1)
                }
                TokenType::Comma if depth == 0 => {
                    segments.push((start + 1 + segment_start, &body[segment_start..offset]));
                    segment_start = offset + 1;
                }
                _ => {}
            }
        }
        segments.push((start + 1 + segment_start, &body[segment_start..]));

        // A single trailing comma is legal and produces one empty final
        // segment; drop it. An empty segment anywhere else is a syntax error
        // and falls through to `classify_param_segment`'s rejection.
        if segments.len() > 1 && segments.last().is_some_and(|(_, last)| last.is_empty()) {
            segments.pop();
        }

        let mut params = Vec::with_capacity(segments.len());
        for (absolute_start, segment) in segments {
            match Self::classify_param_segment(segment) {
                Ok((name, eq)) => {
                    let default = match eq {
                        Some(eq) if eq + 1 < segment.len() => {
                            Some(absolute_start + eq + 1..absolute_start + segment.len())
                        }
                        // `ident =` with nothing after it.
                        Some(_) => {
                            return ParamListScan::Unsupported {
                                after,
                                construct: "an empty default parameter",
                            }
                        }
                        None => None,
                    };
                    params.push(ScannedParam { name, default });
                }
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

    /// Parses a parameter list with the stream positioned AT the opening `(`,
    /// for a function form that cannot take defaults (methods and function
    /// expressions; default-parameters spec A-2). A default is refused with
    /// E5506; the names are still returned so the body parses normally.
    ///
    /// Always leaves the cursor just past the matching `)` when one exists, so
    /// the caller can parse the body without risk of absorbing the rest of the
    /// module.
    pub(crate) fn parse_parameter_list(&mut self) -> Vec<String> {
        let (params, defaults) = self.parse_declaration_parameter_list();
        if defaults.iter().any(Option::is_some) {
            self.push_feature_unavailable(kali_common::default_param_non_declaration_message());
        }
        params
    }

    /// Parses a function declaration's parameter list, keeping each default
    /// (default-parameters spec 3.1). Returns the names and the defaults,
    /// index-aligned; the defaults vector is empty when no parameter has one.
    pub(crate) fn parse_declaration_parameter_list(
        &mut self,
    ) -> (Vec<String>, Vec<Option<Box<Expression>>>) {
        match self.scan_param_list(self.stream.position) {
            ParamListScan::Simple { after, params } => {
                let mut names = Vec::with_capacity(params.len());
                let mut defaults = Vec::with_capacity(params.len());
                for param in params {
                    names.push(param.name);
                    defaults.push(
                        param
                            .default
                            .map(|range| Box::new(self.parse_default(range))),
                    );
                }
                if defaults.iter().all(Option::is_none) {
                    defaults.clear();
                }
                self.stream.position = after;
                (names, defaults)
            }
            ParamListScan::Unsupported { after, construct } => {
                self.reject_unsupported_param(construct);
                self.stream.position = after;
                (Vec::new(), Vec::new())
            }
            ParamListScan::NotAParamList => {
                self.push_feature_unavailable(
                    "unterminated parameter list — expected a closing `)`".to_string(),
                );
                self.stream.position = self.stream.tokens.len();
                (Vec::new(), Vec::new())
            }
        }
    }

    /// Parses one default's tokens as an assignment expression with a
    /// sub-parser, so the main stream never moves into the parameter list.
    /// Tokens left over after the expression are refused.
    fn parse_default(&mut self, range: std::ops::Range<usize>) -> Expression {
        let mut tokens: Vec<Token> = self.stream.tokens[range.clone()].to_vec();
        let eof_span = tokens
            .last()
            .map(|token| token.span)
            .unwrap_or(self.stream.tokens[range.start].span);
        tokens.push(Token::new(TokenType::Eof, String::new(), eof_span));
        let mut sub = Parser::new(self.file_id, tokens);
        let expression = sub.parse_assignment_expression();
        if !matches!(sub.stream.current_kind(), Some(TokenType::Eof) | None) {
            sub.push_feature_unavailable(
                "this default parameter value is unavailable in the current phase",
            );
        }
        self.diagnostics.extend(sub.diagnostics);
        expression
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
        let (params, defaults) = self.parse_declaration_parameter_list();
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
            defaults,
            body: Box::new(body_block),
            is_async,
            generator,
        }))
    }

    /// Consume an optional heritage clause up to (not including) the class
    /// body's `{`, returning the class's base. Only an `extends` at
    /// angle-bracket depth 0 is the class's own (`class B<T extends Foo>`
    /// constrains a type parameter). The base is the identifier after it when
    /// the next token is `{`, `<` or `implements`; any other base expression
    /// (`ns.A`, `mixin(A)`, `(A)`) gives `Some("")`, a base that leaves the
    /// program. Type parameters and `implements` lists are skipped, as before.
    fn parse_class_heritage(&mut self) -> Option<String> {
        let mut super_class = None;
        let mut angle_depth: usize = 0;
        while !matches!(
            self.stream.current_kind(),
            Some(TokenType::LeftBrace) | None
        ) {
            match self.stream.current_kind() {
                Some(TokenType::Lt) => angle_depth += 1,
                Some(TokenType::Gt) => angle_depth = angle_depth.saturating_sub(1),
                // `>>` and `>>>` both lex as `GtGt`; the text gives the count.
                Some(TokenType::GtGt) => {
                    let closes = self.stream.current().map_or(2, |token| token.value.len());
                    angle_depth = angle_depth.saturating_sub(closes);
                }
                Some(TokenType::Extends) if angle_depth == 0 => {
                    let _ = self.stream.advance();
                    let simple_base = self.stream.current_kind() == Some(&TokenType::Identifier)
                        && matches!(
                            self.stream.peek_next_kind(),
                            Some(TokenType::LeftBrace | TokenType::Lt | TokenType::Implements)
                        );
                    super_class = Some(if simple_base {
                        self.stream
                            .current()
                            .map(|token| token.value.clone())
                            .unwrap_or_default()
                    } else {
                        String::new()
                    });
                    continue;
                }
                _ => {}
            }
            let _ = self.stream.advance();
        }
        super_class
    }

    pub(crate) fn parse_class_body(&mut self) -> ClassBody {
        let _ = self.stream.accept(TokenType::LeftBrace);
        let mut body = ClassBody::default();
        loop {
            match self.stream.current_kind() {
                None | Some(TokenType::Eof) => break,
                Some(TokenType::RightBrace) => {
                    let _ = self.stream.advance();
                    break;
                }
                Some(TokenType::Semicolon) => {
                    let _ = self.stream.advance();
                    continue;
                }
                _ => {}
            }
            self.parse_class_member(&mut body);
        }
        body
    }

    /// Parses one class member into `body`. A member the AST does not model
    /// (`#private`, a computed key, a `static {}` block) is skipped and
    /// flagged.
    fn parse_class_member(&mut self, body: &mut ClassBody) {
        let mut is_static = false;
        let mut kind = MethodKind::Method;
        // A modifier word is a modifier only when another key follows it:
        // `get(){}` is a method named `get`, `get v(){}` is a getter.
        while let Some(token) = self.stream.current() {
            if token.kind != TokenType::Identifier
                || !CLASS_MEMBER_MODIFIERS.contains(&token.value.as_str())
                || matches!(
                    self.stream.peek_next_kind(),
                    Some(
                        TokenType::LeftParen
                            | TokenType::Eq
                            | TokenType::Semicolon
                            | TokenType::Colon
                            | TokenType::Question
                            | TokenType::Not
                            | TokenType::RightBrace
                    )
                )
            {
                break;
            }
            match token.value.as_str() {
                "static" => is_static = true,
                "get" => kind = MethodKind::Get,
                "set" => kind = MethodKind::Set,
                _ => {}
            }
            let _ = self.stream.advance();
        }
        if is_static && self.stream.current_kind() == Some(&TokenType::LeftBrace) {
            body.has_static_block = true;
            self.skip_class_member();
            return;
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
        let generator = self.stream.accept(TokenType::Star);
        match self.stream.current_kind() {
            Some(TokenType::Hash) => {
                body.has_private_members = true;
                self.skip_class_member();
                return;
            }
            Some(TokenType::LeftBracket) => {
                body.has_computed_members = true;
                self.skip_class_member();
                return;
            }
            Some(TokenType::Identifier) | Some(TokenType::Async) => {}
            _ => {
                // A string or number key: not modelled; skip it.
                self.skip_class_member();
                return;
            }
        }
        let name = self.stream.advance().map(|t| t.value).unwrap_or_default();
        if self.stream.current_kind() == Some(&TokenType::LeftParen) {
            let params = self.parse_parameter_list();
            self.skip_return_type_annotation();
            let previous_async = self.in_async_function;
            let previous_generator = self.in_generator_function;
            self.in_async_function = is_async;
            self.in_generator_function = generator;
            let block = match self.parse_block_statement() {
                Some(Statement::BlockStatement(bs)) => bs,
                _ => BlockStatement { body: Vec::new() },
            };
            self.in_generator_function = previous_generator;
            self.in_async_function = previous_async;
            body.methods.push(MethodDefinition {
                name,
                params,
                body: Some(Box::new(block)),
                is_async,
                generator,
                kind,
                is_static,
            });
            return;
        }
        // A field: `name`, `name?`, `name!`, `name: Type`, each with an
        // optional `= initializer`.
        let _ = self.stream.accept(TokenType::Question) || self.stream.accept(TokenType::Not);
        let mut initializer_follows = false;
        if self.stream.current_kind() == Some(&TokenType::Colon) {
            initializer_follows = self.skip_field_type_annotation();
        }
        let value = if initializer_follows || self.stream.accept(TokenType::Eq) {
            Some(self.parse_assignment_expression())
        } else {
            None
        };
        let _ = self.stream.accept(TokenType::Semicolon);
        body.field_names.push(name.clone());
        body.fields.push(ClassField {
            name,
            value,
            is_static,
        });
    }

    /// Skips a field's `: Type`, leaving the cursor on `=`, `;`, `}` or the
    /// next member's key. Tokens carry no line breaks, so an annotation
    /// without `;` ends where two words meet that no type spells together
    /// (`string m`).
    ///
    /// The lexer fuses closing angles with what follows (`>>`, `>>>`, `>=`,
    /// `>>=`, `>>>=`), so each closes as many `<` as it has `>` (R-22). When
    /// such a token closes the last one and ends in `=`, that `=` starts the
    /// initializer: it is consumed here and `true` is returned.
    fn skip_field_type_annotation(&mut self) -> bool {
        const TYPE_WORDS: &[&str] = &[
            "keyof", "typeof", "readonly", "infer", "unique", "asserts", "is", "extends", "new",
        ];
        let _ = self.stream.advance();
        let mut depth = 0usize;
        let mut previous_word: Option<String> = None;
        while let Some(token) = self.stream.current().cloned() {
            match token.kind {
                TokenType::Eq | TokenType::Semicolon | TokenType::RightBrace if depth == 0 => break,
                TokenType::Eof => break,
                TokenType::LeftParen
                | TokenType::LeftBracket
                | TokenType::LeftBrace
                | TokenType::Lt => depth += 1,
                TokenType::RightParen
                | TokenType::RightBracket
                | TokenType::RightBrace
                | TokenType::Gt => depth = depth.saturating_sub(1),
                // `>>` and `>>>` share `GtGt`; the lexeme tells them apart.
                TokenType::GtGt => depth = depth.saturating_sub(token.value.len()),
                TokenType::GtEq | TokenType::GtGtEq | TokenType::GtGtGtEq if depth > 0 => {
                    let closes = token.value.len() - 1;
                    depth = depth.saturating_sub(closes);
                    if depth == 0 {
                        let _ = self.stream.advance();
                        return true;
                    }
                }
                TokenType::Identifier if depth == 0 => {
                    if let Some(previous) = &previous_word {
                        if !TYPE_WORDS.contains(&previous.as_str()) {
                            break;
                        }
                    }
                }
                _ => {}
            }
            previous_word = (token.kind == TokenType::Identifier).then(|| token.value.clone());
            let _ = self.stream.advance();
        }
        false
    }

    /// Skips one member the AST does not model: through a `;` at depth 0,
    /// through the `}` that closes a body opened at depth 0, or up to the
    /// class's own `}`.
    fn skip_class_member(&mut self) {
        let mut depth = 0usize;
        while let Some(kind) = self.stream.current_kind().copied() {
            match kind {
                TokenType::Eof => return,
                TokenType::Semicolon if depth == 0 => {
                    let _ = self.stream.advance();
                    return;
                }
                TokenType::RightBrace if depth == 0 => return,
                TokenType::LeftParen | TokenType::LeftBracket | TokenType::LeftBrace => depth += 1,
                TokenType::RightParen | TokenType::RightBracket => depth = depth.saturating_sub(1),
                TokenType::RightBrace => {
                    depth -= 1;
                    if depth == 0 {
                        let _ = self.stream.advance();
                        return;
                    }
                }
                _ => {}
            }
            let _ = self.stream.advance();
        }
    }

    pub(crate) fn parse_class_declaration(&mut self) -> Option<Statement> {
        let _ = self.stream.advance();
        let name_token = self.stream.advance()?;
        let name = name_token.value;
        let super_class = self.parse_class_heritage();
        let body = self.parse_class_body();

        Some(Statement::ClassDeclaration(ClassDeclaration {
            name,
            super_class,
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
        let super_class = self.parse_class_heritage();
        let body = self.parse_class_body();

        Expression::ClassExpression(Box::new(ClassExpression {
            id,
            super_class,
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
            ParamListScan::Simple { after, params } => {
                let had_default = params.iter().any(|param| param.default.is_some());
                ArrowParams::Ok {
                    after,
                    had_default,
                    params: params.into_iter().map(|param| param.name).collect(),
                }
            }
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
        let mut had_default = false;
        let mut allow_return_type = false;
        match self.stream.tokens.get(scan).map(|token| &token.kind) {
            Some(TokenType::LeftParen) => {
                allow_return_type = true;
                match self.scan_arrow_param_list(scan) {
                    ArrowParams::Ok {
                        after,
                        params: p,
                        had_default: d,
                    } => {
                        had_default = d;
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

        if had_default {
            self.push_feature_unavailable(kali_common::default_param_non_declaration_message());
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
        let (scan, params, had_default) = match self.scan_arrow_param_list(start) {
            ArrowParams::Ok {
                after,
                params,
                had_default,
            } => (after, params, had_default),
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

        if had_default {
            self.push_feature_unavailable(kali_common::default_param_non_declaration_message());
        }
        self.stream.position = scan + 1;
        let Some(Statement::BlockStatement(block)) = self.parse_block_statement() else {
            self.stream.position = start;
            return None;
        };
        Some(Expression::FunctionExpression(Box::new(
            FunctionExpression {
                returnType: None,
                id: None,
                params: params
                    .into_iter()
                    .map(|name| FunctionParam { name })
                    .collect(),
                body: Some(Box::new(block)),
                is_async: false,
                generator: false,
                is_arrow: true,
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
            returnType: None,
            id,
            params,
            body: func_body,
            is_async,
            generator,
            is_arrow: false,
        }))
    }
}

#[cfg(test)]
#[path = "declaration_tests.rs"]
mod declaration_tests;
