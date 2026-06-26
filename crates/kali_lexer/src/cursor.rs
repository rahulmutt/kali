use kali_common::{FileId, Span};
use kali_error::diagnostic::Diagnostic;

pub struct Lexer {
    pub(crate) source: Vec<char>,
    pub(crate) file_id: FileId,
    pub(crate) position: usize,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

impl Lexer {
    pub fn new(file_id: FileId, source: String) -> Self {
        Self {
            source: source.chars().collect(),
            file_id,
            position: 0,
            diagnostics: Vec::new(),
        }
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub(crate) fn skip_whitespace(&mut self) {
        while let Some(&c) = self.source.get(self.position) {
            if c.is_whitespace() {
                self.position += 1;
            } else {
                break;
            }
        }
    }

    pub(crate) fn peek(&self) -> Option<char> {
        self.source.get(self.position).copied()
    }

    pub(crate) fn nth(&self, n: usize) -> Option<char> {
        self.source.get(self.position + n).copied()
    }

    pub(crate) fn is_eof(&self) -> bool {
        self.position >= self.source.len()
    }

    pub(crate) fn span(&self) -> Span {
        Span::new(self.file_id, self.position as u32, self.position as u32)
    }

    pub(crate) fn emit_error(&mut self, code: u16, message: &str) {
        self.diagnostics.push(Diagnostic::error(
            code as u32,
            format!("{} at position {}", message, self.position),
        ));
    }

    pub(crate) fn slice(&self, start: usize) -> String {
        self.source[start..self.position].iter().collect()
    }
}
