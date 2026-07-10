//! Expression parsing: operator precedence + sub-parsers.

mod call;
mod object;
mod primary;

use crate::Parser;
use kali_ast::{
    AssignmentExpression, AssignmentOperator, BinaryExpression, ConditionalExpression, Expression,
    UnaryExpression, UpdateExpression, UpdateOperator, YieldExpression,
};
use kali_lexer::TokenType;
use std::boxed::Box;

impl Parser {
    pub(crate) fn parse_expression(&mut self) -> Expression {
        self.parse_assignment_expression()
    }

    pub(crate) fn parse_assignment_expression(&mut self) -> Expression {
        let left = self.parse_conditional_expression();

        let Some(operator) = self.parse_assignment_operator() else {
            return left;
        };

        let _ = self.stream.advance();
        let right = self.parse_assignment_expression();
        Expression::AssignmentExpression(Box::new(AssignmentExpression {
            operator,
            left,
            right,
        }))
    }

    /// `ConditionalExpression : ShortCircuit ('?' AssignmentExpression ':' AssignmentExpression)?`
    /// Right-associative via the recursive `parse_assignment_expression` arms.
    /// `?.` never reaches here (it lexes as `QuestionDot`).
    fn parse_conditional_expression(&mut self) -> Expression {
        let test = self.parse_binary_expression(0);

        if self.stream.current_kind() != Some(&TokenType::Question) {
            return test;
        }
        let _ = self.stream.advance();
        let consequent = self.parse_assignment_expression();
        let _ = self.stream.accept(TokenType::Colon);
        let alternate = self.parse_assignment_expression();
        Expression::ConditionalExpression(Box::new(ConditionalExpression {
            test: Box::new(test),
            consequent: Box::new(consequent),
            alternate: Box::new(alternate),
        }))
    }

    pub(crate) fn parse_assignment_operator(&self) -> Option<AssignmentOperator> {
        match self.stream.current_kind().copied()? {
            TokenType::Eq => Some(AssignmentOperator::Assign),
            TokenType::PlusEq => Some(AssignmentOperator::AddAssign),
            TokenType::MinusEq => Some(AssignmentOperator::SubtractAssign),
            TokenType::StarEq => Some(AssignmentOperator::MultiplyAssign),
            TokenType::SlashEq => Some(AssignmentOperator::DivideAssign),
            TokenType::PercentEq => Some(AssignmentOperator::ModuloAssign),
            TokenType::StarStarEq => Some(AssignmentOperator::ExponentAssign),
            TokenType::NullCoalesceEq => Some(AssignmentOperator::NullishAssign),
            TokenType::AndAndEq => Some(AssignmentOperator::AndAssign),
            TokenType::OrOrEq => Some(AssignmentOperator::OrAssign),
            _ => None,
        }
    }

    pub(crate) fn parse_unary_expression(&mut self) -> Expression {
        match self.stream.current_kind() {
            Some(TokenType::Not) => {
                let _ = self.stream.advance();
                let argument = self.parse_unary_expression();
                Expression::UnaryExpression(Box::new(UnaryExpression {
                    operator: "!".to_string(),
                    argument,
                }))
            }
            Some(TokenType::Void) => {
                let _ = self.stream.advance();
                let argument = self.parse_unary_expression();
                Expression::UnaryExpression(Box::new(UnaryExpression {
                    operator: "void".to_string(),
                    argument,
                }))
            }
            Some(TokenType::Tilde) => {
                let _ = self.stream.advance();
                let argument = self.parse_unary_expression();
                Expression::UnaryExpression(Box::new(UnaryExpression {
                    operator: "~".to_string(),
                    argument,
                }))
            }
            // `typeof <expr>` was previously NOT parsed as a unary operator, so
            // the token fell through to the primary parser as a bare
            // identifier — `typeof value` read the undefined identifier
            // `typeof` (zero placeholder) and dropped `value`. Parse it as a
            // real unary expression; codegen's provable lane classifies it.
            Some(TokenType::Typeof) => {
                let _ = self.stream.advance();
                let argument = self.parse_unary_expression();
                Expression::UnaryExpression(Box::new(UnaryExpression {
                    operator: "typeof".to_string(),
                    argument,
                }))
            }
            Some(TokenType::Plus) => {
                if self
                    .stream
                    .current()
                    .is_some_and(|token| token.value == "++")
                {
                    let _ = self.stream.advance();
                    let argument = self.parse_unary_expression();
                    Expression::UpdateExpression(Box::new(UpdateExpression {
                        operator: UpdateOperator::Increment,
                        argument,
                        prefix: true,
                    }))
                } else {
                    let _ = self.stream.advance();
                    let argument = self.parse_unary_expression();
                    Expression::UnaryExpression(Box::new(UnaryExpression {
                        operator: "+".to_string(),
                        argument,
                    }))
                }
            }
            Some(TokenType::Minus) => {
                if self
                    .stream
                    .current()
                    .is_some_and(|token| token.value == "--")
                {
                    let _ = self.stream.advance();
                    let argument = self.parse_unary_expression();
                    Expression::UpdateExpression(Box::new(UpdateExpression {
                        operator: UpdateOperator::Decrement,
                        argument,
                        prefix: true,
                    }))
                } else {
                    let _ = self.stream.advance();
                    let argument = self.parse_unary_expression();
                    Expression::UnaryExpression(Box::new(UnaryExpression {
                        operator: "-".to_string(),
                        argument,
                    }))
                }
            }
            _ => self.parse_call_expression(),
        }
    }

    pub(crate) fn parse_binary_expression(&mut self, min_prec: usize) -> Expression {
        let mut left = self.parse_unary_expression();

        let mut iterations = 0;
        loop {
            let op_kind = self
                .stream
                .current_kind()
                .copied()
                .unwrap_or(TokenType::Unknown);

            // Get operator precedence (higher number = tighter binding)
            let op_prec: Option<usize> = match op_kind {
                TokenType::OrOr | TokenType::NullCoalesce => Some(1),
                TokenType::AndAnd => Some(2),
                TokenType::Pipe => Some(3),
                TokenType::Caret => Some(4),
                TokenType::Ampersand => Some(5),
                TokenType::EqEquals
                | TokenType::EqEqEq
                | TokenType::Neq
                | TokenType::NeqNeq
                | TokenType::Lt
                | TokenType::Gt
                | TokenType::LtEq
                | TokenType::GtEq => Some(6),
                // Shift operators bind tighter than relational/equality but looser
                // than additive, per JS operator precedence.
                TokenType::LtLt | TokenType::GtGt => Some(7),
                // Binary `in`/`instanceof` have no sound lowering (kali's
                // object model cannot decide runtime key presence after
                // `delete`, nor prototype chains). Previously these were not
                // binary operators here, so `'a' in obj` parsed as `'a'` and
                // `in obj` was silently dropped — the expression miscompiled
                // to its LEFT operand. Recognize them at relational precedence
                // and reject fail-closed in the op_str match below. A trailing
                // `in` inside a `for (expr in obj)` head must still terminate
                // the expression so the for-in statement parser can consume
                // it — that is the `no_in` guard.
                TokenType::In if !self.no_in => Some(6),
                TokenType::InstanceOf => Some(6),
                TokenType::Plus | TokenType::Minus => Some(8),
                TokenType::Star | TokenType::Slash | TokenType::Percent => Some(9),
                TokenType::StarStar => Some(10),
                _ => None,
            };

            // If operator has lower precedence than min_prec, we're done
            if let Some(prec) = op_prec {
                if prec < min_prec {
                    break;
                }

                let op_str = match op_kind {
                    TokenType::Plus => "+",
                    TokenType::Minus => "-",
                    TokenType::Star => "*",
                    TokenType::StarStar => "**",
                    TokenType::Slash => "/",
                    TokenType::Percent => "%",
                    TokenType::AndAnd => "&&",
                    TokenType::OrOr => "||",
                    TokenType::NullCoalesce => "??",
                    TokenType::Pipe => "|",
                    TokenType::Caret => "^",
                    TokenType::Ampersand => "&",
                    TokenType::LtLt => "<<",
                    // `>>` and `>>>` both lex to `GtGt`; disambiguate on the
                    // token's text so `>>>` (unsigned/zero-extend) is not lowered
                    // as `>>` (signed/sign-extend) in codegen.
                    TokenType::GtGt => {
                        if self.stream.current().map(|token| token.value.as_str()) == Some(">>>") {
                            ">>>"
                        } else {
                            ">>"
                        }
                    }
                    TokenType::EqEquals => "==",
                    TokenType::EqEqEq => "===",
                    TokenType::Neq => "!=",
                    TokenType::NeqNeq => "!==",
                    TokenType::Lt => "<",
                    TokenType::Gt => ">",
                    TokenType::LtEq => "<=",
                    TokenType::GtEq => ">=",
                    TokenType::In => {
                        self.push_feature_unavailable(
                            "the binary `in` operator is unavailable: runtime property presence is undecidable in kali's static object model",
                        );
                        "in"
                    }
                    TokenType::InstanceOf => {
                        self.push_feature_unavailable(
                            "the `instanceof` operator is unavailable: kali has no prototype-chain machinery",
                        );
                        "instanceof"
                    }
                    _ => {
                        // Not a binary operator we handle
                        break;
                    }
                };

                let _ = self.stream.advance();
                // Parse right side with higher precedence to get next operand
                // Using prec + 1 ensures left-associativity for same-precedence operators
                // Exponentiation is right-associative, so keep the same precedence on the right.
                let right_prec = if matches!(op_kind, TokenType::StarStar) {
                    prec
                } else {
                    prec + 1
                };
                left = Expression::BinaryExpression(Box::new(BinaryExpression {
                    left,
                    operator: op_str.to_string(),
                    right: self.parse_binary_expression(right_prec),
                }));
            } else {
                break;
            }
            iterations += 1;
            if iterations > 100 {
                break;
            }
        }

        left
    }

    pub(crate) fn parse_yield_expression(&mut self) -> Expression {
        let _ = self.stream.advance();
        let delegate = self.stream.accept(TokenType::Star);
        let argument = match self.stream.current_kind() {
            Some(TokenType::Semicolon)
            | Some(TokenType::RightParen)
            | Some(TokenType::RightBracket)
            | Some(TokenType::RightBrace)
            | Some(TokenType::Comma)
            | Some(TokenType::Eof) => None,
            _ => Some(self.parse_expression()),
        };

        Expression::YieldExpression(Box::new(YieldExpression { delegate, argument }))
    }

    pub(crate) fn parse_await_expression(&mut self) -> Expression {
        let _ = self.stream.advance();
        let argument = self.parse_call_expression();
        Expression::AwaitExpression(Box::new(kali_ast::AwaitExpression { argument }))
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod mod_tests;
