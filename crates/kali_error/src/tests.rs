use super::*;

#[test]
fn test_error_namespace_structure() {
    // Verify all namespace modules are accessible
    use _error_codes::*;

    // E1 namespace
    assert_eq!(e1::UNTERMINATED_STRING, 1000);

    // E2 namespace
    assert_eq!(e2::EXPECTED_TOKEN, 2000);

    // E3 namespace
    assert_eq!(e3::UNDEFINED_IDENTIFIER, 3100);

    // W2 namespace
    assert_eq!(w2::UNUSED_VARIABLE, 2000);

    // E4 namespace
    assert_eq!(e4::UNCAUGHT_ERROR, 4000);

    // E5 namespace
    assert_eq!(e5::UNKNOWN_COMMAND, 5002);

    // E6 namespace
    assert_eq!(e6::NOT_FOUND, 6001);

    // E7 namespace
    assert_eq!(e7::INVALID_WASM_MODULE, 7000);

    // E8 namespace
    assert_eq!(e8::UNIMPLEMENTED, 8001);

    // E9 namespace
    assert_eq!(e9::POLICY_VIOLATION, 9000);
}

#[test]
fn test_diagnostic_creation() {
    let diag = Diagnostic::new(Severity::Error, 1000, "test error".to_string());

    assert_eq!(diag.severity, Severity::Error);
    assert_eq!(diag.code, Some(1000));
}
