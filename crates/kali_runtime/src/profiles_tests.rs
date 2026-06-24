use crate::*;
use crate::test_support::*;


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


#[test]
fn runtime_accepts_threaded_runtime_profile_requests() {
    let runtime = RuntimeCtx::with_api_surface(None, "deno")
        .with_runtime_profiles(vec!["wasm-threads".to_string()]);
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("threaded runtime profile");
    assert_eq!(outcome.runtime_profiles, vec!["wasm-threads".to_string()]);
}


#[test]
fn runtime_rejects_positive_thread_budget_requests_without_threaded_profile() {
    let runtime = RuntimeCtx::with_api_surface(None, "deno").with_max_threads(Some(1));
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
            "#,
    );

    let diagnostics = runtime
        .execute_tests(&wasm)
        .expect_err("positive thread budgets should remain gated without the threaded profile");
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(
        diagnostic.code,
        Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
    );
    assert!(
        diagnostic.message.contains("resources.maxThreads"),
        "diagnostic: {diagnostic:?}"
    );
}


#[test]
fn runtime_accepts_positive_thread_budget_requests_with_threaded_profile() {
    let runtime = RuntimeCtx::with_api_surface(None, "deno")
        .with_runtime_profiles(vec!["wasm-threads".to_string()])
        .with_max_threads(Some(1));
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
            "#,
    );

    let outcome = runtime
        .execute_tests(&wasm)
        .expect("positive thread budgets should be accepted when the threaded profile is active");
    assert_eq!(outcome.runtime_profiles, vec!["wasm-threads".to_string()]);
}


#[test]
fn runtime_outcome_carries_runtime_profiles() {
    let runtime = RuntimeCtx::with_api_surface(None, "deno").with_runtime_profiles(vec![
        "beta".to_string(),
        "beta".to_string(),
        "alpha".to_string(),
    ]);

    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(
        outcome.runtime_profiles,
        vec!["alpha".to_string(), "beta".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::KaliHosted);
    assert_eq!(outcome.runtime_backend, RuntimeBackend::Wasmtime);
}


#[test]
fn runtime_execute_normalizes_profiles_from_public_field_mutation() {
    let mut runtime = RuntimeCtx::with_api_surface(None, "deno");
    runtime.runtime_profiles = vec![
        " beta ".to_string(),
        "alpha".to_string(),
        "beta".to_string(),
        "alpha".to_string(),
    ];

    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
            "#,
    );

    let outcome = runtime.execute(&wasm).expect("runtime outcome");
    assert_eq!(
        outcome.runtime_profiles,
        vec!["alpha".to_string(), "beta".to_string()]
    );
}


#[test]
fn runtime_execute_tests_normalizes_profiles_from_public_field_mutation() {
    let mut runtime = RuntimeCtx::with_api_surface(None, "deno");
    runtime.runtime_profiles = vec![
        " beta ".to_string(),
        "alpha".to_string(),
        "beta".to_string(),
        "alpha".to_string(),
    ];

    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
            "#,
    );

    let outcome = runtime.execute_tests(&wasm).expect("runtime test outcome");
    assert_eq!(outcome.tests_run, 1);
    assert_eq!(
        outcome.runtime_profiles,
        vec!["alpha".to_string(), "beta".to_string()]
    );
}


#[test]
fn runtime_test_outcome_carries_runtime_profiles() {
    let runtime = RuntimeCtx::with_api_surface(None, "deno").with_runtime_profiles(vec![
        "beta".to_string(),
        "beta".to_string(),
        "alpha".to_string(),
    ]);

    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
            "#,
    );

    let outcome = runtime.execute_tests(&wasm).expect("runtime test outcome");
    assert_eq!(outcome.tests_run, 1);
    assert_eq!(
        outcome.runtime_profiles,
        vec!["alpha".to_string(), "beta".to_string()]
    );
    assert_eq!(outcome.host_contract, RuntimeHostContract::KaliHosted);
}


#[test]
fn runtime_exposes_canonical_runtime_profiles_from_public_field_mutation() {
    let mut runtime = RuntimeCtx::with_api_surface(None, "deno");
    runtime.runtime_profiles = vec![
        " wasm-threads ".to_string(),
        "alpha".to_string(),
        "wasm-threads".to_string(),
        "alpha".to_string(),
    ];

    assert_eq!(
        runtime.canonical_runtime_profiles(),
        vec!["alpha".to_string(), "wasm-threads".to_string()]
    );
}


#[test]
fn normalize_runtime_profiles_is_shared_between_callers() {
    assert_eq!(
        normalize_runtime_profiles(vec![
            " wasm-threads ".to_string(),
            "alpha".to_string(),
            "wasm-threads".to_string(),
            "alpha".to_string(),
        ]),
        vec!["alpha".to_string(), "wasm-threads".to_string()]
    );
}
