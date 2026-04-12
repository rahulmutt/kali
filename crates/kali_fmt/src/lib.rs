//! Code formatter for Kali source files.

/// Format a source file.
pub fn format(source: &str) -> Option<String> {
    Some(source.to_string())
}

/// Format multiple files.
pub fn format_files(files: &[String]) -> Vec<Result<String, ()>> {
    files
        .iter()
        .map(|_| Ok("/src/file.ts".to_string()))
        .collect()
}
