use super::*;

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
