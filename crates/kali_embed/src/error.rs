use kali_error::Diagnostic;

/// Compile error wrapper for embedding callers.
#[derive(Debug, Clone)]
pub struct CompileError {
    diagnostics: Vec<Diagnostic>,
}

impl CompileError {
    /// Access the underlying diagnostics.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
}

impl From<Vec<Diagnostic>> for CompileError {
    fn from(diagnostics: Vec<Diagnostic>) -> Self {
        Self { diagnostics }
    }
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.diagnostics.first() {
            Some(diagnostic) => write!(f, "{diagnostic}"),
            None => f.write_str("embedding compile error"),
        }
    }
}

impl std::error::Error for CompileError {}