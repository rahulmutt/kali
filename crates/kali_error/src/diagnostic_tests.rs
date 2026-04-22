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

#[test]
fn test_diagnostic_context_builder_retains_effective_values() {
    let context = DiagnosticContext::new(DiagnosticContextOrigin::Config)
        .with_config_path("compilerOptions.apiSurface")
        .with_requested_value("browser")
        .with_effective_value("browser");

    assert_eq!(context.origin, DiagnosticContextOrigin::Config);
    assert_eq!(
        context.config_path.as_deref(),
        Some("compilerOptions.apiSurface")
    );
    assert_eq!(context.requested_value.as_deref(), Some("browser"));
    assert_eq!(context.effective_value.as_deref(), Some("browser"));
}
