use super::*;

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
