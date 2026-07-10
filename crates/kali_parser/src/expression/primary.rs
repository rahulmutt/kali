//! Primary expression parsing.

use crate::Parser;
use kali_ast::{
    ArrayExpression, ArrowFunctionExpression, Expression, ExpressionOrSpread, FunctionParam,
    ImportExpression, ParenthesizedExpression, SpreadElement,
};
use kali_common::template::split_template_literal;
use kali_error::{_error_codes::e2, diagnostic::Diagnostic};
use kali_lexer::TokenType;
use std::boxed::Box;

impl Parser {
    pub(crate) fn parse_primary_expression(&mut self) -> Expression {
        let kind = self
            .stream
            .current_kind()
            .copied()
            .unwrap_or(TokenType::Unknown);
        match kind {
            TokenType::Identifier => {
                let token = self.stream.advance();
                let name = token
                    .map(|t| t.value)
                    .unwrap_or_else(|| "unknown".to_string());
                if self.stream.current_kind() == Some(&TokenType::Arrow)
                    && self.stream.peek_next_kind() != Some(&TokenType::LeftBrace)
                {
                    let _ = self.stream.advance();
                    let body = self.parse_arrow_function_body_expression();
                    return Expression::ArrowFunctionExpression(Box::new(
                        ArrowFunctionExpression {
                            params: vec![FunctionParam { name }],
                            body,
                            is_async: false,
                            returnType: None,
                        },
                    ));
                }
                Expression::Identifier(name)
            }
            TokenType::This => {
                let _ = self.stream.advance();
                Expression::ThisExpression
            }
            TokenType::True | TokenType::False => {
                let is_true = self.stream.current_kind().copied() == Some(TokenType::True);
                let _ = self.stream.advance();
                Expression::Literal(kali_ast::LiteralValue::Boolean(is_true))
            }
            TokenType::Null => {
                let _ = self.stream.advance();
                Expression::Literal(kali_ast::LiteralValue::Null)
            }
            TokenType::Undefined => {
                let _ = self.stream.advance();
                Expression::Identifier("undefined".to_string())
            }
            TokenType::NumericLiteral => {
                let token = self.stream.advance();
                let value = token.map(|t| t.value).unwrap_or_default();
                if value.ends_with('n') {
                    Expression::BigIntLiteral(value)
                } else {
                    let parsed = value.parse::<f64>().unwrap_or(0.0);
                    Expression::Literal(kali_ast::LiteralValue::Number(parsed))
                }
            }
            TokenType::StringLiteral | TokenType::Template | TokenType::Backtick => {
                let token = self.stream.advance();
                let value = token.map(|t| t.value).unwrap_or_default();
                if matches!(kind, TokenType::Template | TokenType::Backtick) && value.contains("${")
                {
                    return self.desugar_template_literal(&value);
                }
                Expression::Literal(kali_ast::LiteralValue::String(value))
            }
            TokenType::LeftParen => {
                if let Some(expr) = self.try_parse_arrow_function_expression() {
                    return expr;
                }

                let _ = self.stream.advance();
                let first = self.parse_expression();
                let mut expressions = vec![first];
                while self.stream.accept(TokenType::Comma) {
                    expressions.push(self.parse_expression());
                }
                let _ = self.stream.accept(TokenType::RightParen);

                match expressions.len() {
                    0 => Expression::Literal(kali_ast::LiteralValue::Null),
                    1 => Expression::ParenthesizedExpression(Box::new(ParenthesizedExpression {
                        expression: Box::new(expressions.pop().unwrap()),
                    })),
                    _ => Expression::SequenceExpression(Box::new(kali_ast::SequenceExpression {
                        expressions,
                    })),
                }
            }
            TokenType::LeftBracket => {
                let _ = self.stream.advance();
                let mut elements = Vec::new();
                if !self.stream.accept(TokenType::RightBracket) {
                    loop {
                        if self.stream.current_kind() == Some(&TokenType::DotDotDot) {
                            let _ = self.stream.advance();
                            let argument = self.parse_expression();
                            elements
                                .push(Some(ExpressionOrSpread::Spread(SpreadElement { argument })));
                        } else {
                            let element = self.parse_expression();
                            elements.push(Some(ExpressionOrSpread::Expression(element)));
                        }
                        if self.stream.accept(TokenType::Comma) {
                            if self.stream.current_kind() == Some(&TokenType::RightBracket) {
                                let _ = self.stream.accept(TokenType::RightBracket);
                                break;
                            }
                            continue;
                        }
                        let _ = self.stream.accept(TokenType::RightBracket);
                        break;
                    }
                }
                Expression::ArrayExpression(ArrayExpression { elements })
            }
            TokenType::LeftBrace => self.parse_object_expression(),
            TokenType::Yield => {
                if self.in_generator_function {
                    self.parse_yield_expression()
                } else {
                    let token = self.stream.advance();
                    let name = token
                        .map(|t| t.value)
                        .unwrap_or_else(|| "yield".to_string());
                    Expression::Identifier(name)
                }
            }
            TokenType::Await => {
                if self.in_async_function {
                    self.parse_await_expression()
                } else {
                    let token = self.stream.advance();
                    let name = token
                        .map(|t| t.value)
                        .unwrap_or_else(|| "await".to_string());
                    Expression::Identifier(name)
                }
            }
            TokenType::Async => {
                if self.stream.peek_next_kind() == Some(&TokenType::Function) {
                    self.parse_function_expression_with_async(true)
                } else if let Some(expr) =
                    self.try_parse_arrow_function_expression_from(self.stream.position + 1, true)
                {
                    expr
                } else {
                    let token = self.stream.advance();
                    let name = token
                        .map(|t| t.value)
                        .unwrap_or_else(|| "async".to_string());
                    Expression::Identifier(name)
                }
            }
            // Contextual keywords that are legal JS binding identifiers
            // (see `is_binding_name_token`) must also read back as plain
            // identifier expressions — `const type = 1; console.log(type)`
            // is valid JS. `Async`/`Yield`/`Await` already have their own
            // arms above; this covers the rest of the allowlist.
            TokenType::Type
            | TokenType::Interface
            | TokenType::Enum
            | TokenType::From
            | TokenType::As
            | TokenType::Of => {
                let token = self.stream.advance();
                let name = token
                    .map(|t| t.value)
                    .unwrap_or_else(|| "unknown".to_string());
                Expression::Identifier(name)
            }
            TokenType::Function => self.parse_function_expression_with_async(false),
            TokenType::Class => self.parse_class_expression(),
            TokenType::Import => {
                let _ = self.stream.advance();
                if self.stream.accept(TokenType::LeftParen) {
                    let source = self.parse_expression();
                    let _ = self.stream.accept(TokenType::RightParen);
                    Expression::ImportExpression(Box::new(ImportExpression { source }))
                } else {
                    Expression::Identifier("import".to_string())
                }
            }
            TokenType::New => {
                let _ = self.stream.advance();
                // `new [1,2,3]` — `new` on an array literal is not
                // constructible (node throws TypeError). Reject fail-closed
                // BEFORE parsing the callee, so the ArrayExpression unwrap
                // below can only ever be reached via the call-path desugar
                // (`Array(a, b, …)` → array literal), never a bracket
                // literal. Still parse the expression to keep the stream
                // consistent; the diagnostic rejects the program.
                if self.stream.current_kind() == Some(&TokenType::LeftBracket) {
                    self.push_feature_unavailable("`new` on an array literal is not constructible");
                    return self.parse_call_expression();
                }
                let callee = self.parse_call_expression();
                let mut args = Vec::new();
                if self.stream.accept(TokenType::LeftParen)
                    && !self.stream.accept(TokenType::RightParen)
                {
                    args.push(self.parse_expression());
                    while self.stream.accept(TokenType::Comma) {
                        args.push(self.parse_expression());
                    }
                    let _ = self.stream.accept(TokenType::RightParen);
                }
                // `new Array(a, b, …)` desugars identically to
                // `Array(a, b, …)` — the call-path desugar already turned
                // the callee into the array literal; `new` adds nothing for
                // the Array constructor. INVARIANT: an ArrayExpression callee
                // here can only be desugar output — a bracket-literal callee
                // (`new [...]`) was rejected fail-closed above, before
                // callee parsing.
                if matches!(&callee, Expression::ArrayExpression(_)) {
                    return callee;
                }
                Expression::NewExpression(Box::new(kali_ast::NewExpression { callee, args }))
            }
            _ => {
                let _ = self.stream.advance();
                Expression::Identifier("unknown".to_string())
            }
        }
    }

    /// Desugars an interpolated template literal token into a left-associated
    /// string `+` chain: `` `v: ${7 / 2}` `` becomes `"v: " + (7 / 2)`.
    /// Quasis are carried as backtick-delimited string literals — a quasi can
    /// never contain a backtick, so the delimiter is unambiguous and the
    /// literal takes the plain-template path everywhere downstream. The
    /// leading quasi is always emitted (even empty) so the chain is
    /// string-valued from its first operand; later empty quasis are skipped.
    fn desugar_template_literal(&mut self, raw: &str) -> Expression {
        if raw.contains("\\${") {
            self.diagnostics.push(Diagnostic::error(
                e2::MALFORMED_TEMPLATE_INTERPOLATION as u32,
                "Escaped `\\${` in a template literal is not supported: template escape \
                 sequences are not processed in the current runtime path",
            ));
            return Expression::Literal(kali_ast::LiteralValue::String(raw.to_string()));
        }
        let Some(segments) = split_template_literal(raw) else {
            self.diagnostics.push(Diagnostic::error(
                e2::MALFORMED_TEMPLATE_INTERPOLATION as u32,
                "Unterminated `${` interpolation in template literal",
            ));
            return Expression::Literal(kali_ast::LiteralValue::String(raw.to_string()));
        };

        fn quasi_literal(quasi: &str) -> Expression {
            Expression::Literal(kali_ast::LiteralValue::String(format!("`{quasi}`")))
        }
        fn concat(left: Expression, right: Expression) -> Expression {
            Expression::BinaryExpression(Box::new(kali_ast::BinaryExpression {
                operator: "+".to_string(),
                left,
                right,
            }))
        }

        let mut chain = quasi_literal(&segments.quasis[0]);
        for (expression_source, quasi) in segments.expressions.iter().zip(&segments.quasis[1..]) {
            let expression = self.parse_template_expression_segment(expression_source);
            chain = concat(chain, expression);
            if !quasi.is_empty() {
                chain = concat(chain, quasi_literal(quasi));
            }
        }
        chain
    }

    /// Parses the raw source of one `${...}` segment with a sub-lexer and
    /// sub-parser, merging their diagnostics into this parser. Spans inside
    /// the segment are relative to the segment text, matching the precedent
    /// set by `resolve_static_string_from_source` in kali_types.
    fn parse_template_expression_segment(&mut self, source: &str) -> Expression {
        let trimmed = source.trim();
        if trimmed.is_empty() {
            self.diagnostics.push(Diagnostic::error(
                e2::MALFORMED_TEMPLATE_INTERPOLATION as u32,
                "Empty `${}` interpolation in template literal",
            ));
            return Expression::Literal(kali_ast::LiteralValue::String("``".to_string()));
        }
        let lexed = kali_lexer::Lexer::new(self.file_id, trimmed.to_string()).lex_all();
        self.diagnostics.extend(lexed.diagnostics);
        let mut sub_parser = Parser::new(self.file_id, lexed.tokens);
        let expression = sub_parser.parse_expression();
        self.diagnostics.extend(sub_parser.diagnostics);
        // The lexer always appends a trailing `Eof` token, so a fully
        // consumed sub-expression leaves the sub-parser's stream sitting on
        // that `Eof` token. Anything else left over is unconsumed input,
        // e.g. `${1 2}`, which would otherwise be silently dropped.
        if sub_parser.stream.current_kind() != Some(&TokenType::Eof) {
            self.diagnostics.push(Diagnostic::error(
                e2::MALFORMED_TEMPLATE_INTERPOLATION as u32,
                "Unexpected tokens after expression in `${...}` interpolation",
            ));
        }
        expression
    }
}

#[cfg(test)]
#[path = "primary_tests.rs"]
mod primary_tests;
