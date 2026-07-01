//! Cross-cutting build helpers (crate-internal).

use std::path::Path;

use super::read_compiler_source_file;
use kali_ast::Statement;
use kali_common::FileId;
use kali_error::{_error_codes::e5, Diagnostic, DiagnosticContext, DiagnosticContextOrigin};
use kali_lexer::Lexer;
use kali_parser::Parser;

pub(crate) fn parse_source_file(source_path: &Path) -> Result<Vec<Statement>, Vec<Diagnostic>> {
    let source = read_compiler_source_file(source_path)?;

    let lexer = Lexer::new(FileId::new(0), source);
    let tokens = lexer.lex_all().tokens;
    let mut parser = Parser::new(FileId::new(0), tokens);
    let parsed = parser.parse(Some(source_path.to_string_lossy().to_string()));
    let diagnostics = parsed.diagnostics;

    if has_errors(&diagnostics) {
        return Err(diagnostics);
    }

    Ok(parsed.statements)
}

pub(crate) fn signature_from_export_specifier(local: &str) -> String {
    format!("({}) => unknown", local)
}

pub(crate) fn invalid_export_surface(source_path: &Path, message: &str) -> Diagnostic {
    Diagnostic::error(
        e5::INVALID_EXPORT_SURFACE as u32,
        format!(
            "cannot build library artifact from '{}': {}",
            source_path.display(),
            message
        ),
    )
    .with_context(DiagnosticContext::new(DiagnosticContextOrigin::Source))
}

pub(crate) fn source_stem(source_path: &Path) -> String {
    source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("main")
        .to_string()
}

pub(crate) fn has_errors(diagnostics: &[Diagnostic]) -> bool {
    diagnostics.iter().any(|diagnostic| diagnostic.is_error())
}
