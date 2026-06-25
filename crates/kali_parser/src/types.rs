//! Type-reference text parsing.

use crate::Parser;
use kali_lexer::TokenType;

impl Parser {
    pub(crate) fn parse_type_reference_text(&mut self) -> String {
        let mut rendered = String::new();
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut angle_depth = 0usize;

        while let Some(kind) = self.stream.current_kind().copied() {
            let top_level_terminator = paren_depth == 0
                && bracket_depth == 0
                && angle_depth == 0
                && matches!(
                    kind,
                    TokenType::Semicolon
                        | TokenType::Comma
                        | TokenType::RightParen
                        | TokenType::RightBracket
                        | TokenType::RightBrace
                        | TokenType::Eof
                        | TokenType::Plus
                        | TokenType::Minus
                        | TokenType::Star
                        | TokenType::Slash
                        | TokenType::Percent
                        | TokenType::AndAnd
                        | TokenType::OrOr
                        | TokenType::Eq
                        | TokenType::EqEquals
                        | TokenType::EqEqEq
                        | TokenType::Neq
                        | TokenType::NeqNeq
                        | TokenType::LtEq
                        | TokenType::GtEq
                        | TokenType::LtLt
                        | TokenType::GtGt
                        | TokenType::Question
                        | TokenType::QuestionDot
                        | TokenType::NullCoalesce
                        | TokenType::InstanceOf
                        | TokenType::In
                        | TokenType::Arrow
                );

            if top_level_terminator {
                break;
            }

            match kind {
                TokenType::LeftParen => paren_depth += 1,
                TokenType::RightParen if paren_depth > 0 => paren_depth -= 1,
                TokenType::LeftBracket => bracket_depth += 1,
                TokenType::RightBracket if bracket_depth > 0 => bracket_depth -= 1,
                TokenType::Lt => angle_depth += 1,
                TokenType::Gt if angle_depth > 0 => angle_depth -= 1,
                _ => {}
            }

            if let Some(token) = self.stream.advance() {
                rendered.push_str(&token.value);
            } else {
                break;
            }
        }

        rendered.trim().to_string()
    }
}
