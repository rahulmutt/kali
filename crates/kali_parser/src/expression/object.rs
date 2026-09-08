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
                Some(kind) if Self::is_property_name_token(&kind) => {
                    // Keyword property keys (`type`, `if`, …) are plain names in JS
                    // object literals — same key set as `.name` member access
                    // (is_property_name_token), so literal writes and member reads
                    // can never disagree again.
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
                    let text = self
                        .stream
                        .advance()
                        .map(|token| token.value)
                        .unwrap_or_default();
                    let _ = self.stream.accept(TokenType::Colon);
                    let Some(name) = numeric_property_name(&text) else {
                        // NOT `unwrap_or(0.0)`. Fabricating the key `0` for a literal this
                        // parser could not read is how `{42n: 1}` came to answer
                        // `Object.hasOwn(o, 0)` with `true` -- a program reading a value out
                        // of a key it never wrote. If it cannot be read, it is refused.
                        self.push_feature_unavailable(
                            "this numeric property key is unavailable in the current phase; use a decimal or string literal key",
                        );
                        let _ = self.parse_expression();
                        continue;
                    };
                    (name, self.parse_expression())
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
                    // Fail closed: the old arm advanced-and-continued, silently
                    // DISCARDING the whole property (keyword keys, spreads, methods —
                    // anything unrecognized). A property the parser cannot represent
                    // must reject, never vanish.
                    self.push_feature_unavailable(
                        "this object-literal property form is unavailable in the current phase; use `key: value` with an identifier, string, or numeric key",
                    );
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

    pub(crate) fn unwrap_await_literal_array_expression(
        &self,
        expression: Expression,
    ) -> Option<Expression> {
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

    pub(crate) fn computed_object_property_name(
        &self,
        expression: Expression,
    ) -> Option<PropertyName> {
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

/// The property name a numeric-literal key token denotes, or `None` when the
/// token is not one this phase can read.
///
/// The BigInt arm keeps DIGITS: `String(42n)` is `"42"`, exactly, for values
/// with no exact `f64`. A leading zero before another digit (`042n`) is
/// refused, not admitted as `"042"`: JavaScript makes that a SyntaxError (the
/// whole program fails to parse), so admitting it here would accept a program
/// node refuses -- fail-open in the one direction this arm must not take.
/// `0n` itself (a single `"0"`) is legal and stays admitted.
///
/// This phase also declines non-decimal BigInt literals (`0x2an`, `0b101n`,
/// `0o17n`) and non-decimal numeric keys generally: the lexer that hands this
/// function its `text` does not tokenize `0x`/`0b`/`0o` prefixes at all (a
/// pre-existing, unrelated gap -- `0x10` lexes as the numeric literal `0`
/// followed by the identifier `x10`, never reaching this function as one
/// token), so hex/binary/octal keys never arrive here to be refused by name;
/// they misparse upstream instead. Fixing that is out of this function's
/// scope.
fn numeric_property_name(text: &str) -> Option<PropertyName> {
    if let Some(digits) = text.strip_suffix('n') {
        let is_valid_bigint_digits = !digits.is_empty()
            && digits.bytes().all(|byte| byte.is_ascii_digit())
            && !(digits.len() > 1 && digits.starts_with('0'));
        return is_valid_bigint_digits.then(|| PropertyName::BigInt(digits.to_string()));
    }
    text.parse::<f64>().ok().map(PropertyName::Number)
}

#[cfg(test)]
#[path = "object_tests.rs"]
mod object_tests;
