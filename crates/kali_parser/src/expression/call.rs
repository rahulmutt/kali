//! Call expression, optional chaining, and member-access name helpers.

use crate::Parser;
use kali_ast::{
    ArrayExpression, CallExpression, Expression, ExpressionOrSpread, MemberExpression,
    SatisfiesExpression, TypeAssertion, UpdateExpression, UpdateOperator,
};
use kali_lexer::TokenType;
use std::boxed::Box;

impl Parser {
    pub(crate) fn parse_call_expression(&mut self) -> Expression {
        let mut expr = self.parse_primary_expression();

        let mut iterations = 0;
        loop {
            match self.stream.current_kind() {
                Some(TokenType::LeftParen) => {
                    let _ = self.stream.advance();
                    let mut args = Vec::new();
                    if !self.stream.accept(TokenType::RightParen) {
                        let arg = self.parse_expression();
                        args.push(arg);
                        while self.stream.accept(TokenType::Comma) {
                            let arg = self.parse_expression();
                            args.push(arg);
                        }
                        let _ = self.stream.accept(TokenType::RightParen);
                    }
                    // `Array(e1, …, en)` with n >= 2 IS the array literal
                    // `[e1, …, en]` (JS semantics; single-arg `Array(n)` is a
                    // length). Desugar at parse time so BOTH twins (types on
                    // the AST, codegen downstream) see a plain
                    // `ArrayExpression` and every array-literal gate applies
                    // fail-closed by construction — no new recognizer
                    // surface.
                    let is_array_call = args.len() >= 2
                        && matches!(&expr, Expression::Identifier(name) if name == "Array");
                    expr = if is_array_call {
                        Expression::ArrayExpression(ArrayExpression {
                            elements: args
                                .drain(..)
                                .map(|arg| Some(ExpressionOrSpread::Expression(arg)))
                                .collect(),
                        })
                    } else {
                        Expression::CallExpression(Box::new(CallExpression { callee: expr, args }))
                    };
                }
                Some(TokenType::LeftBracket) => {
                    let _ = self.stream.advance();
                    let index = self.parse_expression();
                    let _ = self.stream.accept(TokenType::RightBracket);
                    let index_str = Self::expression_to_property_name(&index);
                    expr = Expression::MemberExpression(Box::new(MemberExpression {
                        object: expr,
                        property: index_str,
                        computed_index: Some(Box::new(index)),
                    }));
                }
                Some(TokenType::Dot) => {
                    let _ = self.stream.advance();
                    match self.stream.current_kind().copied() {
                        Some(kind) if Self::is_property_name_token(&kind) => {
                            let _ = self.stream.advance();
                            if let Some(token) = self.stream.tokens.get(self.stream.position - 1) {
                                let prop_name = token.value.clone();
                                expr = Expression::MemberExpression(Box::new(MemberExpression {
                                    object: expr,
                                    property: prop_name,
                                    computed_index: None,
                                }));
                            } else {
                                expr = Expression::MemberExpression(Box::new(MemberExpression {
                                    object: expr,
                                    property: "unknown".to_string(),
                                    computed_index: None,
                                }));
                            }
                        }
                        _ => {
                            // No identifier after dot, stop the chain
                            break;
                        }
                    }
                }
                Some(TokenType::QuestionDot) => {
                    let _ = self.stream.advance();
                    expr = self.parse_optional_chain_expression(expr);
                }
                Some(TokenType::Plus)
                    if self
                        .stream
                        .current()
                        .is_some_and(|token| token.value == "++") =>
                {
                    let _ = self.stream.advance();
                    expr = Expression::UpdateExpression(Box::new(UpdateExpression {
                        operator: UpdateOperator::Increment,
                        argument: expr,
                        prefix: false,
                    }));
                    break;
                }
                Some(TokenType::Minus)
                    if self
                        .stream
                        .current()
                        .is_some_and(|token| token.value == "--") =>
                {
                    let _ = self.stream.advance();
                    expr = Expression::UpdateExpression(Box::new(UpdateExpression {
                        operator: UpdateOperator::Decrement,
                        argument: expr,
                        prefix: false,
                    }));
                    break;
                }
                Some(TokenType::As) => {
                    let _ = self.stream.advance();
                    let type_name = self.parse_type_reference_text();
                    expr = Expression::TypeAssertion(Box::new(TypeAssertion {
                        type_name,
                        expression: Box::new(expr),
                    }));
                }
                Some(TokenType::Identifier)
                    if self
                        .stream
                        .current()
                        .is_some_and(|token| token.value == "satisfies") =>
                {
                    let _ = self.stream.advance();
                    let type_name = self.parse_type_reference_text();
                    expr = Expression::SatisfiesExpression(Box::new(SatisfiesExpression {
                        type_name,
                        expression: Box::new(expr),
                    }));
                }
                _ => break,
            }
            iterations += 1;
            if iterations > 100 {
                break;
            }
        }

        expr
    }

    /// True for every token `lex_identifier` can produce: a plain identifier
    /// or any reserved word. Reserved words are valid property names after `.`
    /// in JS/TS (`event.type`, `config.default`, `list.of`, ...); the token's
    /// `value` field always carries the word text, so the member-access parser
    /// can consume it like an identifier.
    pub(crate) fn is_property_name_token(kind: &TokenType) -> bool {
        matches!(
            kind,
            TokenType::Identifier
                | TokenType::If
                | TokenType::Else
                | TokenType::For
                | TokenType::While
                | TokenType::Do
                | TokenType::Switch
                | TokenType::Case
                | TokenType::Default
                | TokenType::Break
                | TokenType::Continue
                | TokenType::Return
                | TokenType::Throw
                | TokenType::Try
                | TokenType::Catch
                | TokenType::Finally
                | TokenType::Debugger
                | TokenType::New
                | TokenType::Function
                | TokenType::Var
                | TokenType::Let
                | TokenType::Const
                | TokenType::Class
                | TokenType::Interface
                | TokenType::Type
                | TokenType::Enum
                | TokenType::Import
                | TokenType::Export
                | TokenType::From
                | TokenType::As
                | TokenType::This
                | TokenType::Super
                | TokenType::Extends
                | TokenType::Implements
                | TokenType::Async
                | TokenType::Await
                | TokenType::Yield
                | TokenType::InstanceOf
                | TokenType::In
                | TokenType::Of
                | TokenType::True
                | TokenType::False
                | TokenType::Null
                | TokenType::Undefined
                | TokenType::Void
                | TokenType::Delete
                | TokenType::Typeof
        )
    }

    /// Tokens legal as a BINDING name. Deliberately a small default-deny
    /// allowlist (NOT is_property_name_token minus a denylist): property
    /// names admit every keyword, binding names admit only identifiers and
    /// the contextual keywords that are legal JS binding identifiers.
    pub(crate) fn is_binding_name_token(kind: &TokenType) -> bool {
        matches!(
            kind,
            TokenType::Identifier
                | TokenType::Type
                | TokenType::Interface
                | TokenType::Enum
                | TokenType::From
                | TokenType::As
                | TokenType::Of
                | TokenType::Async
        )
    }

    pub(crate) fn parse_optional_chain_expression(&mut self, object: Expression) -> Expression {
        // Wrap the receiver in the short-circuit marker so `a?.b` short-circuits
        // when `a` is nullish, then preserve the accessed property/index as a
        // real `MemberExpression`. The historical lowering DROPPED the property
        // (`a?.b` collapsed to `a`), which silently miscompiled optional member
        // access; keeping the property lets the downstream member recognizers
        // (host members such as `process.kill`, `Math.pow`, …) see `a?.b` with
        // the same shape as `a.b`.
        let optional_object =
            Expression::OptionalChainExpression(Box::new(kali_ast::OptionalChainExpression {
                inner: Box::new(kali_ast::OptionalChainInner::NonNull {
                    object: Box::new(object),
                    optional: true,
                }),
            }));
        match self.stream.current_kind().copied() {
            Some(TokenType::Identifier) => {
                let _ = self.stream.advance();
                let prop_name = self
                    .stream
                    .tokens
                    .get(self.stream.position - 1)
                    .map(|token| token.value.clone())
                    .unwrap_or_else(|| "unknown".to_string());
                return Expression::MemberExpression(Box::new(MemberExpression {
                    object: optional_object,
                    property: prop_name,
                    computed_index: None,
                }));
            }
            Some(TokenType::LeftBracket) => {
                let _ = self.stream.advance();
                let index = self.parse_expression();
                let _ = self.stream.accept(TokenType::RightBracket);
                let index_str = Self::expression_to_property_name(&index);
                return Expression::MemberExpression(Box::new(MemberExpression {
                    object: optional_object,
                    property: index_str,
                    computed_index: Some(Box::new(index)),
                }));
            }
            Some(TokenType::LeftParen) => {
                let _ = self.stream.advance();
                if !self.stream.accept(TokenType::RightParen) {
                    let _ = self.parse_expression();
                    while self.stream.accept(TokenType::Comma) {
                        let _ = self.parse_expression();
                    }
                    let _ = self.stream.accept(TokenType::RightParen);
                }
            }
            Some(TokenType::QuestionDot) => {
                // Support repeated optional chaining segments like `a?.b?.c`.
            }
            _ => {}
        }

        optional_object
    }

    pub(crate) fn is_object_freeze_call(call: &CallExpression) -> bool {
        matches!(
            Self::call_member_access_name(&call.callee).as_deref(),
            Some("Object.freeze")
                | Some("globalThis.Object.freeze")
                | Some(r#"globalThis["Object"].freeze"#)
                | Some(r#"globalThis["Object"]["freeze"]"#)
                | Some(r#"globalThis['Object'].freeze"#)
                | Some(r#"globalThis['Object']['freeze']"#)
                | Some(r#"Object["freeze"]"#)
                | Some(r#"Object['freeze']"#)
                | Some(r#"globalThis.Object["freeze"]"#)
                | Some(r#"globalThis.Object['freeze']"#)
        ) && call.args.len() == 1
    }

    pub(crate) fn call_member_access_name(expression: &Expression) -> Option<String> {
        match expression {
            Expression::MemberExpression(member) => Self::member_access_name(member),
            Expression::ParenthesizedExpression(expr) => {
                Self::call_member_access_name(&expr.expression)
            }
            Expression::TypeAssertion(expr) => Self::call_member_access_name(&expr.expression),
            Expression::SatisfiesExpression(expr) => {
                Self::call_member_access_name(&expr.expression)
            }
            Expression::DecoratedExpression(expr) => {
                Self::call_member_access_name(&expr.expression)
            }
            Expression::ChainExpression(expr) => Self::call_member_access_name(&expr.expression),
            Expression::SequenceExpression(expr) => expr
                .expressions
                .last()
                .and_then(Self::call_member_access_name),
            Expression::AwaitExpression(expr) => Self::call_member_access_name(&expr.argument),
            Expression::OptionalChainExpression(expr) => match expr.inner.as_ref() {
                kali_ast::OptionalChainInner::NonNull { object, .. } => {
                    Self::call_member_access_name(object)
                }
            },
            Expression::Identifier(name) => Some(name.clone()),
            _ => None,
        }
    }

    pub(crate) fn member_access_name(member: &MemberExpression) -> Option<String> {
        let object = Self::call_member_access_name(&member.object)?;
        Some(format!("{object}.{}", member.property))
    }
}

#[cfg(test)]
#[path = "call_tests.rs"]
mod call_tests;
