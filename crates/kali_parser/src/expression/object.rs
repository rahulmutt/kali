//! Object expression parsing and property-name resolution helpers.

use crate::literal::unquote_string_literal;
use crate::Parser;
use kali_ast::{
    Expression, LiteralValue, ObjectExpression, ObjectProperty, ObjectPropertyKind, PropertyName,
};
use kali_lexer::TokenType;

impl Parser {
    pub(crate) fn parse_object_expression(&mut self) -> Expression {
        let _ = self.stream.advance();
        let mut properties = Vec::new();

        while !matches!(
            self.stream.current_kind(),
            Some(TokenType::RightBrace) | None
        ) {
            if self.stream.accept(TokenType::Comma) {
                continue;
            }

            let (key, value) = match self.stream.current_kind().copied() {
                Some(TokenType::Identifier) => {
                    let name = self
                        .stream
                        .advance()
                        .map(|token| token.value)
                        .unwrap_or_default();

                    if self.stream.accept(TokenType::Colon) {
                        (PropertyName::Identifier(name), self.parse_expression())
                    } else {
                        let expr = Expression::Identifier(name.clone());
                        (PropertyName::Identifier(name), expr)
                    }
                }
                Some(TokenType::StringLiteral) => {
                    let token = self.stream.advance();
                    let name = token
                        .map(|token| unquote_string_literal(&token.value))
                        .unwrap_or_default();
                    let _ = self.stream.accept(TokenType::Colon);
                    (PropertyName::String(name), self.parse_expression())
                }
                Some(TokenType::NumericLiteral) => {
                    let token = self.stream.advance();
                    let name = token
                        .and_then(|token| token.value.parse::<f64>().ok())
                        .unwrap_or(0.0);
                    let _ = self.stream.accept(TokenType::Colon);
                    (PropertyName::Number(name), self.parse_expression())
                }
                Some(TokenType::LeftBracket) => {
                    let _ = self.stream.advance();
                    let key_expr = self.parse_expression();
                    let _ = self.stream.accept(TokenType::RightBracket);
                    let _ = self.stream.accept(TokenType::Colon);
                    if let Some(key) = self.computed_object_property_name(key_expr) {
                        (key, self.parse_expression())
                    } else {
                        self.push_feature_unavailable(
                            "computed object property names are unavailable in the current phase; use a string or numeric literal key",
                        );
                        let _ = self.parse_expression();
                        continue;
                    }
                }
                _ => {
                    let _ = self.stream.advance();
                    continue;
                }
            };

            properties.push(ObjectProperty {
                key,
                value,
                kind: ObjectPropertyKind::Init,
            });

            if self.stream.accept(TokenType::Comma) {
                continue;
            }
            let _ = self.stream.accept(TokenType::RightBrace);
            break;
        }

        let _ = self.stream.accept(TokenType::RightBrace);
        Expression::ObjectExpression(ObjectExpression { properties })
    }

    pub(crate) fn unwrap_await_literal_array_expression(&self, expression: Expression) -> Option<Expression> {
        match expression {
            Expression::AwaitExpression(await_expr) => {
                self.unwrap_await_literal_array_expression(await_expr.argument)
            }
            Expression::ParenthesizedExpression(parenthesized) => {
                self.unwrap_await_literal_array_expression(*parenthesized.expression)
            }
            Expression::TypeAssertion(assertion) => {
                self.unwrap_await_literal_array_expression(*assertion.expression)
            }
            Expression::SatisfiesExpression(satisfies) => {
                self.unwrap_await_literal_array_expression(*satisfies.expression)
            }
            Expression::DecoratedExpression(decorated) => {
                self.unwrap_await_literal_array_expression(*decorated.expression)
            }
            Expression::ChainExpression(chain) => {
                self.unwrap_await_literal_array_expression(*chain.expression)
            }
            Expression::SequenceExpression(sequence) => sequence
                .expressions
                .last()
                .cloned()
                .and_then(|expression| self.unwrap_await_literal_array_expression(expression)),
            Expression::ArrayExpression(_) => Some(expression),
            _ => None,
        }
    }

    pub(crate) fn computed_object_property_name(&self, expression: Expression) -> Option<PropertyName> {
        match expression {
            Expression::ParenthesizedExpression(parenthesized) => {
                self.computed_object_property_name(*parenthesized.expression)
            }
            Expression::TypeAssertion(assertion) => {
                self.computed_object_property_name(*assertion.expression)
            }
            Expression::SatisfiesExpression(satisfies) => {
                self.computed_object_property_name(*satisfies.expression)
            }
            Expression::DecoratedExpression(decorated) => {
                self.computed_object_property_name(*decorated.expression)
            }
            Expression::ChainExpression(chain) => {
                self.computed_object_property_name(*chain.expression)
            }
            Expression::AwaitExpression(await_expr) => {
                self.computed_object_property_name(await_expr.argument)
            }
            Expression::SequenceExpression(sequence) => sequence
                .expressions
                .last()
                .cloned()
                .and_then(|expression| self.computed_object_property_name(expression)),
            Expression::CallExpression(call)
                if Self::is_object_freeze_call(&call) && call.args.len() == 1 =>
            {
                call.args
                    .first()
                    .cloned()
                    .and_then(|expression| self.computed_object_property_name(expression))
            }
            Expression::UnaryExpression(unary)
                if unary.operator == "+" || unary.operator == "-" =>
            {
                let value = self.computed_object_property_name(unary.argument.clone())?;
                match (unary.operator.as_str(), value) {
                    ("+", PropertyName::Number(number)) => Some(PropertyName::Number(number)),
                    ("-", PropertyName::Number(number)) => {
                        Some(PropertyName::Number(if number == 0.0 {
                            if number.is_sign_negative() {
                                0.0
                            } else {
                                -0.0
                            }
                        } else {
                            -number
                        }))
                    }
                    _ => None,
                }
            }
            Expression::Literal(LiteralValue::String(value)) => {
                Some(PropertyName::String(unquote_string_literal(&value)))
            }
            Expression::Literal(LiteralValue::Number(value)) => Some(PropertyName::Number(value)),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "object_tests.rs"]
mod object_tests;
