use super::*;

#[test]
fn test_diagnostic_creation() {
    let diag = Diagnostic::new(Severity::Error, 1000, "test error".to_string());

    assert_eq!(diag.severity, Severity::Error);
    assert_eq!(diag.code, Some(1000));
}

#[test]
fn test_terminal_format_uses_canonical_prefixes() {
    let diag = Diagnostic::error(3021, "type 'string' is not assignable to type 'number'")
        .with_suggestion("change the value or the type annotation")
        .note("strict null checks are enabled");

    let formatted = diag.format_terminal();
    assert!(formatted.starts_with("error[E3021]: type 'string' is not assignable to type 'number'"));
    assert!(formatted.contains("= help: change the value or the type annotation"));
    assert!(formatted.contains("= note: strict null checks are enabled"));
}
