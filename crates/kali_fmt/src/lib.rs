//! Code formatter for Kali source files.

use kali_common::FileId;
use kali_lexer::{Lexer, Token, TokenType};

/// Format a source file.
pub fn format(source: &str) -> Option<String> {
    Some(format_source(source))
}

/// Format multiple source snippets.
///
/// This helper is primarily used by higher-level tooling; each input string is
/// treated as source text and formatted independently.
pub fn format_files(files: &[String]) -> Vec<Result<String, ()>> {
    files.iter().map(|source| Ok(format_source(source))).collect()
}

/// Format a Kali source snippet into the canonical Phase-1 style.
pub fn format_source(source: &str) -> String {
    let lexer = Lexer::new(FileId::new(0), source.to_string());
    let mut tokens = lexer.lex_all().tokens;
    tokens.retain(|token| token.kind != TokenType::Eof);

    let mut formatter = Formatter::new(tokens);
    formatter.run();
    formatter.finish()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BraceKind {
    Block,
    Object,
}

struct Formatter {
    tokens: Vec<Token>,
    output: String,
    indent: usize,
    line_start: bool,
    paren_depth: usize,
    brace_stack: Vec<BraceKind>,
    block_candidate: bool,
    prev_kind: Option<TokenType>,
}

impl Formatter {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            output: String::new(),
            indent: 0,
            line_start: true,
            paren_depth: 0,
            brace_stack: Vec::new(),
            block_candidate: false,
            prev_kind: None,
        }
    }

    fn run(&mut self) {
        for index in 0..self.tokens.len() {
            let token = self.tokens[index].clone();
            let next_kind = self.tokens.get(index + 1).map(|token| token.kind);
            self.emit_token(token, next_kind);
        }

        if !self.output.ends_with('\n') {
            self.output.push('\n');
        }
    }

    fn finish(self) -> String {
        self.output
    }

    fn emit_token(&mut self, token: Token, next_kind: Option<TokenType>) {
        match token.kind {
            TokenType::Comment => self.emit_comment(token.value),
            TokenType::LeftBrace => self.emit_left_brace(next_kind),
            TokenType::RightBrace => self.emit_right_brace(next_kind),
            TokenType::Semicolon => self.emit_semicolon(),
            TokenType::Comma => self.emit_comma(next_kind),
            TokenType::Colon => self.emit_symbol(":", Some(' ')),
            TokenType::Dot => self.emit_symbol(".", None),
            TokenType::LeftParen => self.emit_left_paren(),
            TokenType::RightParen => self.emit_right_paren(),
            TokenType::LeftBracket => self.emit_left_bracket(),
            TokenType::RightBracket => self.emit_right_bracket(),
            TokenType::Arrow => self.emit_operator("=>"),
            TokenType::Plus
            | TokenType::Minus
            | TokenType::Star
            | TokenType::Slash
            | TokenType::Percent
            | TokenType::AndAnd
            | TokenType::OrOr
            | TokenType::Pipe
            | TokenType::Caret
            | TokenType::And
            | TokenType::EqEquals
            | TokenType::EqEqEq
            | TokenType::Neq
            | TokenType::NeqNeq
            | TokenType::Lt
            | TokenType::Gt
            | TokenType::LtEq
            | TokenType::GtEq
            | TokenType::NullCoalesce
            | TokenType::Eq => self.emit_operator(token.value.as_str()),
            TokenType::Bang | TokenType::Not | TokenType::Tilde | TokenType::At | TokenType::Hash => {
                self.emit_prefix_operator(token.value.as_str())
            }
            TokenType::StringLiteral => self.emit_string_literal(token.value.as_str()),
            TokenType::Template | TokenType::Backtick => self.emit_raw(token.value.as_str()),
            TokenType::Identifier
            | TokenType::NumericLiteral
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
            | TokenType::Unknown => self.emit_word(token.kind, token.value.as_str(), next_kind),
            TokenType::Eof => {}
            _ => self.emit_raw(token.value.as_str()),
        }
    }

    fn emit_comment(&mut self, comment: String) {
        if self.line_start {
            self.write_indent();
        } else if !self.output.ends_with(' ') && !self.output.ends_with('\n') {
            self.output.push(' ');
        }

        self.output.push_str(&comment);
        self.prev_kind = Some(TokenType::Comment);

        if comment.starts_with("//") || comment.contains('\n') {
            self.output.push('\n');
            self.line_start = true;
        } else {
            self.line_start = false;
        }

        self.block_candidate = false;
    }

    fn emit_left_brace(&mut self, next_kind: Option<TokenType>) {
        let is_block = self.block_candidate;
        self.block_candidate = false;

        self.space_if_needed();
        self.output.push('{');
        self.prev_kind = Some(TokenType::LeftBrace);

        if is_block {
            self.brace_stack.push(BraceKind::Block);
            self.indent = self.indent.saturating_add(1);
            self.output.push('\n');
            self.line_start = true;
        } else {
            self.brace_stack.push(BraceKind::Object);
            if !matches!(next_kind, Some(TokenType::RightBrace)) {
                self.output.push(' ');
            }
            self.line_start = false;
        }
    }

    fn emit_right_brace(&mut self, next_kind: Option<TokenType>) {
        let context = self.brace_stack.pop().unwrap_or(BraceKind::Block);
        match context {
            BraceKind::Block => {
                if !self.line_start {
                    self.output.push('\n');
                }
                self.indent = self.indent.saturating_sub(1);
                self.write_indent();
                self.output.push('}');
                self.prev_kind = Some(TokenType::RightBrace);

                if matches!(next_kind, Some(TokenType::Else | TokenType::Catch | TokenType::Finally | TokenType::While)) {
                    self.output.push(' ');
                    self.line_start = false;
                } else {
                    self.output.push('\n');
                    self.line_start = true;
                }
            }
            BraceKind::Object => {
                if !self.line_start && !self.output.ends_with(' ') && !self.output.ends_with('{') {
                    self.output.push(' ');
                }
                self.output.push('}');
                self.line_start = false;
                self.prev_kind = Some(TokenType::RightBrace);
            }
        }
    }

    fn emit_semicolon(&mut self) {
        self.output.push(';');
        self.prev_kind = Some(TokenType::Semicolon);
        if self.paren_depth == 0 {
            self.output.push('\n');
            self.line_start = true;
        } else {
            self.line_start = false;
        }
        self.block_candidate = false;
    }

    fn emit_comma(&mut self, next_kind: Option<TokenType>) {
        self.output.push(',');
        if !matches!(next_kind, Some(TokenType::RightParen | TokenType::RightBracket | TokenType::RightBrace)) {
            self.output.push(' ');
        }
        self.prev_kind = Some(TokenType::Comma);
        self.line_start = false;
    }

    fn emit_symbol(&mut self, symbol: &str, trailing_space: Option<char>) {
        self.output.push_str(symbol);
        if let Some(space) = trailing_space {
            self.output.push(space);
        }
        self.prev_kind = None;
        self.line_start = false;
    }

    fn emit_left_paren(&mut self) {
        if matches!(
            self.prev_kind,
            Some(
                TokenType::If
                    | TokenType::Else
                    | TokenType::For
                    | TokenType::While
                    | TokenType::Do
                    | TokenType::Switch
                    | TokenType::Catch
                    | TokenType::Try
                    | TokenType::Function
                    | TokenType::New
                    | TokenType::Return
                    | TokenType::Throw
            )
        ) {
            self.output.push(' ');
        }
        self.output.push('(');
        self.paren_depth = self.paren_depth.saturating_add(1);
        self.prev_kind = Some(TokenType::LeftParen);
        self.line_start = false;
    }

    fn emit_right_paren(&mut self) {
        self.paren_depth = self.paren_depth.saturating_sub(1);
        self.output.push(')');
        self.prev_kind = Some(TokenType::RightParen);
        self.line_start = false;
    }

    fn emit_left_bracket(&mut self) {
        self.output.push('[');
        self.prev_kind = Some(TokenType::LeftBracket);
        self.line_start = false;
    }

    fn emit_right_bracket(&mut self) {
        self.output.push(']');
        self.prev_kind = Some(TokenType::RightBracket);
        self.line_start = false;
    }

    fn emit_operator(&mut self, op: &str) {
        self.space_if_needed();
        self.output.push_str(op);
        self.output.push(' ');
        self.prev_kind = None;
        self.line_start = false;
        self.block_candidate |= op == "=>";
    }

    fn emit_prefix_operator(&mut self, op: &str) {
        if self.line_start {
            self.write_indent();
        } else if !self.output.ends_with(' ') && !self.output.ends_with('\n') && !self.output.ends_with('(') {
            self.output.push(' ');
        }
        self.output.push_str(op);
        self.prev_kind = None;
        self.line_start = false;
    }

    fn emit_word(&mut self, kind: TokenType, value: &str, next_kind: Option<TokenType>) {
        if self.line_start {
            self.write_indent();
        } else if self.needs_space_before_word() {
            self.output.push(' ');
        }

        self.output.push_str(value);
        self.prev_kind = Some(kind);
        self.line_start = false;

        self.block_candidate |= matches!(
            kind,
            TokenType::If
                | TokenType::Else
                | TokenType::For
                | TokenType::While
                | TokenType::Do
                | TokenType::Switch
                | TokenType::Try
                | TokenType::Catch
                | TokenType::Function
                | TokenType::Class
                | TokenType::Async
        );

        if matches!(kind, TokenType::Case | TokenType::Default) {
            self.output.push(' ');
        }

        if matches!(kind, TokenType::Else) && matches!(next_kind, Some(TokenType::If)) {
            self.output.push(' ');
        }
    }

    fn emit_string_literal(&mut self, value: &str) {
        if self.line_start {
            self.write_indent();
        } else if self.needs_space_before_word() {
            self.output.push(' ');
        }
        self.output.push_str(&normalize_string(value));
        self.prev_kind = Some(TokenType::StringLiteral);
        self.line_start = false;
        self.block_candidate = false;
    }

    fn emit_raw(&mut self, value: &str) {
        if self.line_start {
            self.write_indent();
        } else if !self.output.ends_with(' ') && !self.output.ends_with('\n') {
            self.output.push(' ');
        }
        self.output.push_str(value);
        self.prev_kind = None;
        self.line_start = false;
    }

    fn needs_space_before_word(&self) -> bool {
        !self.output.is_empty()
            && !self.output.ends_with('\n')
            && !self.output.ends_with(' ')
            && !self.output.ends_with('{')
            && !self.output.ends_with('(')
            && !self.output.ends_with('[')
            && !matches!(
                self.prev_kind,
                None
                    | Some(TokenType::LeftParen)
                    | Some(TokenType::LeftBracket)
                    | Some(TokenType::LeftBrace)
                    | Some(TokenType::Dot)
                    | Some(TokenType::Comma)
                    | Some(TokenType::Colon)
                    | Some(TokenType::Semicolon)
                    | Some(TokenType::Arrow)
            )
    }

    fn space_if_needed(&mut self) {
        if !self.line_start && !self.output.ends_with(' ') && !self.output.ends_with('\n') {
            self.output.push(' ');
        }
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("  ");
        }
        self.line_start = false;
    }
}

fn normalize_string(raw: &str) -> String {
    if raw.starts_with('"') {
        return raw.to_string();
    }

    let inner = raw.trim_matches('"').trim_matches('\'');
    let mut out = String::with_capacity(inner.len() + 2);
    out.push('"');
    for ch in inner.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_simple_function() {
        let formatted = format_source("function add(a,b){return a+b;}");
        assert!(formatted.contains("function add(a, b) {\n"));
        assert!(formatted.contains("return a + b;\n"));
        assert!(formatted.ends_with('\n'));
    }

    #[test]
    fn formatting_is_idempotent() {
        let source = "function add(a,b){return a+b;}";
        let once = format_source(source);
        let twice = format_source(&once);
        assert_eq!(once, twice);
    }
}
