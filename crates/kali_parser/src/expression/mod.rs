//! Expression parsing: operator precedence + sub-parsers.

mod call;
mod object;
mod primary;

use crate::Parser;
use kali_ast::{
    AssignmentExpression, AssignmentOperator, BinaryExpression, Expression, UnaryExpression,
    UpdateExpression, UpdateOperator, YieldExpression,
};
use kali_lexer::TokenType;
use std::boxed::Box;

impl Parser {
    pub(crate) fn parse_expression(&mut self) -> Expression {
        self.parse_assignment_expression()
    }

    pub(crate) fn parse_assignment_expression(&mut self) -> Expression {
        let left = self.parse_binary_expression(0);

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
                TokenType::And => Some(5),
                TokenType::EqEquals
                | TokenType::Neq
                | TokenType::NeqNeq
                | TokenType::Lt
                | TokenType::Gt
                | TokenType::LtEq
                | TokenType::GtEq => Some(6),
                TokenType::Plus | TokenType::Minus => Some(7),
                TokenType::Star | TokenType::Slash | TokenType::Percent => Some(8),
                TokenType::StarStar => Some(9),
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
                    TokenType::And => "&",
                    TokenType::EqEquals => "==",
                    TokenType::Neq => "!=",
                    TokenType::NeqNeq => "!==",
                    TokenType::Lt => "<",
                    TokenType::Gt => ">",
                    TokenType::LtEq => "<=",
                    TokenType::GtEq => ">=",
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
