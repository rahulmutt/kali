//! Code formatter for Kali source files.

mod formatter;

pub use formatter::format_source;

/// Format a source file.
pub fn format(source: &str) -> Option<String> {
    Some(format_source(source))
}

/// Format multiple source snippets.
///
/// This helper is primarily used by higher-level tooling; each input string is
/// treated as source text and formatted independently.
pub fn format_files(files: &[String]) -> Vec<Result<String, ()>> {
    files
        .iter()
        .map(|source| Ok(format_source(source)))
        .collect()
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
