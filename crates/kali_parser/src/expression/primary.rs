//! Primary expression parsing.

use crate::Parser;
use kali_ast::{
    ArrowFunctionExpression, ArrayExpression, Expression, ExpressionOrSpread, FunctionParam,
    ImportExpression, ParenthesizedExpression, SpreadElement,
};
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
                Expression::NewExpression(Box::new(kali_ast::NewExpression { callee, args }))
            }
            _ => {
                let _ = self.stream.advance();
                Expression::Identifier("unknown".to_string())
            }
        }
    }
}

#[cfg(test)]
#[path = "primary_tests.rs"]
mod primary_tests;
