use super::*;

#[test]
fn runtime_reports_browser_host_contract_for_browser_api_surface() {
    let runtime = RuntimeCtx::with_api_surface(None, "browser");

    assert_eq!(
        runtime.host_contract(),
        RuntimeHostContract::BrowserRequested
    );
    assert!(runtime
        .host_contract()
        .canonical_label()
        .contains("browser"));
}

#[test]
fn runtime_rejects_browser_api_surface() {
    let runtime = RuntimeCtx::with_host_context_with_api_surface(
        None,
        Vec::new(),
        BTreeMap::new(),
        PathBuf::from("."),
        "browser",
    );
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
            "#,
    );

    let diagnostics = runtime
        .execute(&wasm)
        .expect_err("browser runtime should be gated");
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(
        diagnostic.code,
        Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(
        diagnostic
            .message
            .contains("standalone browser runtime contract"),
        "diagnostic: {diagnostic:?}"
    );
    assert!(
        diagnostic
            .notes
            .iter()
            .any(|note| note == "current runtime backend: wasmtime"),
        "diagnostic: {diagnostic:?}"
    );
    assert!(
        diagnostic
            .notes
            .iter()
            .any(|note| note == BrowserRuntimeContract::summary_note()),
        "diagnostic: {diagnostic:?}"
    );
    assert_eq!(
        diagnostic.context.as_deref(),
        Some(
            &DiagnosticContext::new(DiagnosticContextOrigin::Default)
                .with_requested_value("browser")
                .with_effective_value("browser")
        ),
        "diagnostic: {diagnostic:?}"
    );
}

#[test]
fn runtime_test_execution_rejects_browser_api_surface() {
    let runtime = RuntimeCtx::with_host_context_with_api_surface(
        None,
        Vec::new(),
        BTreeMap::new(),
        PathBuf::from("."),
        "browser",
    );
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
            "#,
    );

    let diagnostics = runtime
        .execute_tests(&wasm)
        .expect_err("browser runtime test path should be gated");
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(
        diagnostic.code,
        Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(
        diagnostic
            .message
            .contains("standalone browser runtime contract"),
        "diagnostic: {diagnostic:?}"
    );
    assert!(
        diagnostic
            .notes
            .iter()
            .any(|note| note == "current runtime backend: wasmtime"),
        "diagnostic: {diagnostic:?}"
    );
    assert!(
        diagnostic
            .notes
            .iter()
            .any(|note| note == BrowserRuntimeContract::summary_note()),
        "diagnostic: {diagnostic:?}"
    );
    assert_eq!(
        diagnostic.context.as_deref(),
        Some(
            &DiagnosticContext::new(DiagnosticContextOrigin::Default)
                .with_requested_value("browser")
                .with_effective_value("browser")
        ),
        "diagnostic: {diagnostic:?}"
    );
}
