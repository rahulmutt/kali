use super::*;

#[test]
fn build_source_file_rejects_unsupported_permission_query_descriptors_in_js_input() {
    assert_build_source_file_rejects_unsupported_permission_query_descriptors_in_input("js");
}

#[test]
fn build_source_file_rejects_unsupported_permission_query_descriptors_in_ts_input() {
    assert_build_source_file_rejects_unsupported_permission_query_descriptors_in_input("ts");
}

#[test]
fn build_source_file_rejects_unsupported_permission_query_descriptors_in_jsx_input() {
    assert_build_source_file_rejects_unsupported_permission_query_descriptors_in_input("jsx");
}

#[test]
fn build_source_file_rejects_unsupported_permission_query_descriptors_in_tsx_input() {
    assert_build_source_file_rejects_unsupported_permission_query_descriptors_in_input("tsx");
}

#[test]
fn build_source_file_rejects_late_subprocess_and_network_globals_in_js_input() {
    assert_build_source_file_rejects_late_subprocess_global_in_input("js");
}

#[test]
fn build_source_file_rejects_late_subprocess_and_network_globals_in_ts_input() {
    assert_build_source_file_rejects_late_subprocess_global_in_input("ts");
}

#[test]
fn build_source_file_rejects_late_subprocess_and_network_globals_in_jsx_input() {
    assert_build_source_file_rejects_late_subprocess_global_in_input("jsx");
}

#[test]
fn build_source_file_rejects_late_subprocess_and_network_globals_in_tsx_input() {
    assert_build_source_file_rejects_late_subprocess_global_in_input("tsx");
}

#[test]
fn build_source_file_rejects_bracketed_proxy_revocable_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"globalThis["Proxy"]["revocable"]({}, {}); globalThis["Proxy"].revocable({}, {}); Object.freeze(globalThis["Proxy"]["revocable"])({}, {}); Object.freeze(globalThis["Proxy"].revocable)({}, {});"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("late object-model APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| diagnostic
            .message
            .contains(r#"globalThis["Proxy"]["revocable"]"#)
            || diagnostic
                .message
                .contains(r#"globalThis["Proxy"].revocable"#)
            || diagnostic.message.contains("Proxy.revocable")),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_bracketed_proxy_revocable_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"globalThis["Proxy"]["revocable"]({}, {}); globalThis["Proxy"].revocable({}, {}); Object.freeze(globalThis["Proxy"]["revocable"])({}, {}); Object.freeze(globalThis["Proxy"].revocable)({}, {});"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("late object-model APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| diagnostic
            .message
            .contains(r#"globalThis["Proxy"]["revocable"]"#)
            || diagnostic
                .message
                .contains(r#"globalThis["Proxy"].revocable"#)
            || diagnostic.message.contains("Proxy.revocable")),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_bracketed_proxy_revocable_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.jsx");
    fs::write(
        &source_path,
        r#"globalThis["Proxy"]["revocable"]({}, {}); globalThis["Proxy"].revocable({}, {}); Object.freeze(globalThis["Proxy"]["revocable"])({}, {}); Object.freeze(globalThis["Proxy"].revocable)({}, {});"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("late object-model APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| diagnostic
            .message
            .contains(r#"globalThis["Proxy"]["revocable"]"#)
            || diagnostic
                .message
                .contains(r#"globalThis["Proxy"].revocable"#)
            || diagnostic.message.contains("Proxy.revocable")),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_bracketed_proxy_revocable_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(
        &source_path,
        r#"globalThis["Proxy"]["revocable"]({}, {}); globalThis["Proxy"].revocable({}, {}); Object.freeze(globalThis["Proxy"]["revocable"])({}, {}); Object.freeze(globalThis["Proxy"].revocable)({}, {});"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("late object-model APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| diagnostic
            .message
            .contains(r#"globalThis["Proxy"]["revocable"]"#)
            || diagnostic
                .message
                .contains(r#"globalThis["Proxy"].revocable"#)
            || diagnostic.message.contains("Proxy.revocable")),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_bracketed_proxy_revocable_in_browser_api_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"globalThis["Proxy"]["revocable"]({}, {}); globalThis["Proxy"].revocable({}, {}); Object.freeze(globalThis["Proxy"]["revocable"])({}, {}); Object.freeze(globalThis["Proxy"].revocable)({}, {});"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Browser,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("late object-model APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| diagnostic
            .message
            .contains(r#"globalThis["Proxy"]["revocable"]"#)
            || diagnostic
                .message
                .contains(r#"globalThis["Proxy"].revocable"#)
            || diagnostic.message.contains("Proxy.revocable")),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_bracketed_proxy_revocable_in_browser_api_surface_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"globalThis["Proxy"]["revocable"]({}, {}); globalThis["Proxy"].revocable({}, {}); Object.freeze(globalThis["Proxy"]["revocable"])({}, {}); Object.freeze(globalThis["Proxy"].revocable)({}, {});"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Browser,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("late object-model APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| diagnostic
            .message
            .contains(r#"globalThis["Proxy"]["revocable"]"#)
            || diagnostic
                .message
                .contains(r#"globalThis["Proxy"].revocable"#)
            || diagnostic.message.contains("Proxy.revocable")),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_bracketed_proxy_revocable_in_browser_api_surface_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.jsx");
    fs::write(
        &source_path,
        r#"globalThis["Proxy"]["revocable"]({}, {}); globalThis["Proxy"].revocable({}, {}); Object.freeze(globalThis["Proxy"]["revocable"])({}, {}); Object.freeze(globalThis["Proxy"].revocable)({}, {});"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Browser,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("late object-model APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| diagnostic
            .message
            .contains(r#"globalThis["Proxy"]["revocable"]"#)
            || diagnostic
                .message
                .contains(r#"globalThis["Proxy"].revocable"#)
            || diagnostic.message.contains("Proxy.revocable")),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_bracketed_proxy_revocable_in_browser_api_surface_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(
        &source_path,
        r#"globalThis["Proxy"]["revocable"]({}, {}); globalThis["Proxy"].revocable({}, {}); Object.freeze(globalThis["Proxy"]["revocable"])({}, {}); Object.freeze(globalThis["Proxy"].revocable)({}, {});"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Browser,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("late object-model APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| diagnostic
            .message
            .contains(r#"globalThis["Proxy"]["revocable"]"#)
            || diagnostic
                .message
                .contains(r#"globalThis["Proxy"].revocable"#)
            || diagnostic.message.contains("Proxy.revocable")),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_bracketed_object_has_own_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, r#"globalThis["Object"]["hasOwn"]({}, "a");"#).expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("late object-model APIs should succeed");

    assert!(!output.wasm_bytes.is_empty());
}

#[test]
fn build_source_file_rejects_mixed_object_has_own_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"globalThis.Object.hasOwn({}, "a"); globalThis.Object.prototype.hasOwnProperty.call({}, "a"); globalThis["Object"]["prototype"].hasOwnProperty.call({}, "a"); globalThis.Object["hasOwn"]({}, "a"); globalThis["Object"].hasOwn({}, "a"); globalThis.Object["prototype"].hasOwnProperty.call({}, "a"); globalThis["Object"].prototype.hasOwnProperty.call({}, "a");"#,
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("late object-model APIs should succeed");

    assert!(!output.wasm_bytes.is_empty());
}

#[test]
fn build_source_file_rejects_mixed_object_has_own_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.jsx");
    fs::write(
        &source_path,
        r#"globalThis.Object.hasOwn({}, "a"); globalThis.Object.prototype.hasOwnProperty.call({}, "a"); globalThis["Object"]["prototype"].hasOwnProperty.call({}, "a"); globalThis.Object["hasOwn"]({}, "a"); globalThis["Object"].hasOwn({}, "a"); globalThis.Object["prototype"].hasOwnProperty.call({}, "a"); globalThis["Object"].prototype.hasOwnProperty.call({}, "a");"#,
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("late object-model APIs should succeed");

    assert!(!output.wasm_bytes.is_empty());
}

#[test]
fn build_source_file_rejects_mixed_object_has_own_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(
        &source_path,
        r#"globalThis.Object.hasOwn({}, "a"); globalThis.Object.prototype.hasOwnProperty.call({}, "a"); globalThis["Object"]["prototype"].hasOwnProperty.call({}, "a"); globalThis.Object["hasOwn"]({}, "a"); globalThis["Object"].hasOwn({}, "a"); globalThis.Object["prototype"].hasOwnProperty.call({}, "a"); globalThis["Object"].prototype.hasOwnProperty.call({}, "a");"#,
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("late object-model APIs should succeed");

    assert!(!output.wasm_bytes.is_empty());
}

#[test]
fn build_source_file_rejects_mixed_object_has_own_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"globalThis.Object.hasOwn({}, "a"); globalThis.Object.prototype.hasOwnProperty.call({}, "a"); globalThis["Object"]["prototype"].hasOwnProperty.call({}, "a"); globalThis.Object["hasOwn"]({}, "a"); globalThis["Object"].hasOwn({}, "a"); globalThis.Object["prototype"].hasOwnProperty.call({}, "a"); globalThis["Object"].prototype.hasOwnProperty.call({}, "a");"#,
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("late object-model APIs should succeed");

    assert!(!output.wasm_bytes.is_empty());
}

#[test]
fn build_source_file_rejects_bracketed_object_has_own_property_call_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]({}, "a");"#,
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("late object-model APIs should succeed");

    assert!(!output.wasm_bytes.is_empty());
}

#[test]
fn build_source_file_rejects_bracketed_object_has_own_property_call_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]({}, "a");"#,
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("late object-model APIs should succeed");

    assert!(!output.wasm_bytes.is_empty());
}

#[test]
fn build_source_file_rejects_unsupported_math_member_calls_in_js_input() {
    assert_build_source_file_rejects_unsupported_math_member_calls_in_input(ApiSurface::Deno, "js");
}

#[test]
fn build_source_file_rejects_unsupported_math_member_calls_in_ts_input() {
    assert_build_source_file_rejects_unsupported_math_member_calls_in_input(ApiSurface::Deno, "ts");
}

#[test]
fn build_source_file_rejects_unsupported_math_member_calls_in_browser_api_surface_in_js_input() {
    assert_build_source_file_rejects_unsupported_math_member_calls_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_rejects_unsupported_math_member_calls_in_browser_api_surface_in_ts_input() {
    assert_build_source_file_rejects_unsupported_math_member_calls_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_rejects_unsupported_math_member_calls_in_jsx_input() {
    assert_build_source_file_rejects_unsupported_math_member_calls_in_input(
        ApiSurface::Deno,
        "jsx",
    );
}

#[test]
fn build_source_file_rejects_unsupported_math_member_calls_in_tsx_input() {
    assert_build_source_file_rejects_unsupported_math_member_calls_in_input(
        ApiSurface::Deno,
        "tsx",
    );
}

#[test]
fn build_source_file_rejects_unsupported_math_member_calls_in_browser_api_surface_in_jsx_input() {
    assert_build_source_file_rejects_unsupported_math_member_calls_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_rejects_unsupported_math_member_calls_in_browser_api_surface_in_tsx_input() {
    assert_build_source_file_rejects_unsupported_math_member_calls_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_rejects_optional_chain_wrapped_math_pow_in_js_input() {
    assert_build_source_file_rejects_optional_chain_wrapped_math_pow_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_rejects_optional_chain_wrapped_math_pow_in_ts_input() {
    assert_build_source_file_rejects_optional_chain_wrapped_math_pow_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_rejects_optional_chain_wrapped_math_pow_in_jsx_input() {
    assert_build_source_file_rejects_optional_chain_wrapped_math_pow_in_input(
        ApiSurface::Deno,
        "jsx",
    );
}

#[test]
fn build_source_file_rejects_optional_chain_wrapped_math_pow_in_tsx_input() {
    assert_build_source_file_rejects_optional_chain_wrapped_math_pow_in_input(
        ApiSurface::Deno,
        "tsx",
    );
}

#[test]
fn build_source_file_rejects_optional_chain_wrapped_math_pow_in_browser_api_surface_js_input() {
    assert_build_source_file_rejects_optional_chain_wrapped_math_pow_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_rejects_optional_chain_wrapped_math_pow_in_browser_api_surface_ts_input() {
    assert_build_source_file_rejects_optional_chain_wrapped_math_pow_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_rejects_optional_chain_wrapped_math_pow_in_browser_api_surface_jsx_input() {
    assert_build_source_file_rejects_optional_chain_wrapped_math_pow_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_rejects_optional_chain_wrapped_math_pow_in_browser_api_surface_tsx_input() {
    assert_build_source_file_rejects_optional_chain_wrapped_math_pow_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_rejects_negative_math_pow_exponents_in_js_input() {
    assert_build_source_file_rejects_negative_math_pow_exponents_in_input(ApiSurface::Deno, "js");
}

#[test]
fn build_source_file_rejects_negative_math_pow_exponents_in_ts_input() {
    assert_build_source_file_rejects_negative_math_pow_exponents_in_input(ApiSurface::Deno, "ts");
}

#[test]
fn build_source_file_rejects_negative_math_pow_exponents_in_jsx_input() {
    assert_build_source_file_rejects_negative_math_pow_exponents_in_input(ApiSurface::Deno, "jsx");
}

#[test]
fn build_source_file_rejects_negative_math_pow_exponents_in_tsx_input() {
    assert_build_source_file_rejects_negative_math_pow_exponents_in_input(ApiSurface::Deno, "tsx");
}

#[test]
fn build_source_file_rejects_negative_math_pow_exponents_in_browser_api_surface_in_js_input() {
    assert_build_source_file_rejects_negative_math_pow_exponents_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_rejects_negative_math_pow_exponents_in_browser_api_surface_in_ts_input() {
    assert_build_source_file_rejects_negative_math_pow_exponents_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_rejects_negative_math_pow_exponents_in_browser_api_surface_in_jsx_input() {
    assert_build_source_file_rejects_negative_math_pow_exponents_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_rejects_negative_math_pow_exponents_in_browser_api_surface_in_tsx_input() {
    assert_build_source_file_rejects_negative_math_pow_exponents_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_rejects_unsupported_math_inverse_hyperbolic_member_calls_in_js_input() {
    assert_build_source_file_rejects_unsupported_math_inverse_hyperbolic_member_calls_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_rejects_unsupported_math_inverse_hyperbolic_member_calls_in_ts_input() {
    assert_build_source_file_rejects_unsupported_math_inverse_hyperbolic_member_calls_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_rejects_unsupported_math_inverse_hyperbolic_member_calls_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_rejects_unsupported_math_inverse_hyperbolic_member_calls_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_rejects_unsupported_math_inverse_hyperbolic_member_calls_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_rejects_unsupported_math_inverse_hyperbolic_member_calls_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_rejects_for_of_non_literal_iterable_in_ts_input() {
    assert_build_source_file_rejects_for_of_non_literal_iterable_in_input("ts");
}

#[test]
fn build_source_file_rejects_for_of_non_literal_iterable_in_js_input() {
    assert_build_source_file_rejects_for_of_non_literal_iterable_in_input("js");
}

#[test]
fn build_source_file_rejects_array_callback_iteration_in_ts_input() {
    assert_build_source_file_rejects_array_callback_iteration_in_input(ApiSurface::Deno, "ts");
}

#[test]
fn build_source_file_rejects_array_callback_iteration_in_js_input() {
    assert_build_source_file_rejects_array_callback_iteration_in_input(ApiSurface::Deno, "js");
}

#[test]
fn build_source_file_rejects_array_callback_iteration_in_browser_ts_input() {
    assert_build_source_file_rejects_array_callback_iteration_in_input(ApiSurface::Browser, "ts");
}

#[test]
fn build_source_file_rejects_array_callback_iteration_in_browser_js_input() {
    assert_build_source_file_rejects_array_callback_iteration_in_input(ApiSurface::Browser, "js");
}

#[test]
fn build_source_file_rejects_array_callback_iteration_in_browser_jsx_input() {
    assert_build_source_file_rejects_array_callback_iteration_in_input(ApiSurface::Browser, "jsx");
}

#[test]
fn build_source_file_rejects_array_callback_iteration_in_browser_tsx_input() {
    assert_build_source_file_rejects_array_callback_iteration_in_input(ApiSurface::Browser, "tsx");
}

#[test]
fn build_source_file_rejects_for_await_non_literal_iterable_in_ts_input() {
    assert_build_source_file_rejects_for_await_non_literal_iterable_in_input("ts");
}

#[test]
fn build_source_file_rejects_for_await_non_literal_iterable_in_js_input() {
    assert_build_source_file_rejects_for_await_non_literal_iterable_in_input("js");
}

#[test]
fn build_source_file_rejects_generator_functions_in_ts_input() {
    assert_build_source_file_rejects_generator_lowering_in_input("ts");
}

#[test]
fn build_source_file_rejects_generator_functions_in_js_input() {
    assert_build_source_file_rejects_generator_lowering_in_input("js");
}

#[test]
fn build_source_file_rejects_generator_functions_in_jsx_input() {
    assert_build_source_file_rejects_generator_lowering_in_input("jsx");
}

#[test]
fn build_source_file_rejects_generator_functions_in_tsx_input() {
    assert_build_source_file_rejects_generator_lowering_in_input("tsx");
}

#[test]
fn build_source_file_rejects_async_generator_functions_in_ts_input() {
    assert_build_source_file_rejects_async_generator_lowering_in_input("ts");
}

#[test]
fn build_source_file_rejects_async_generator_functions_in_js_input() {
    assert_build_source_file_rejects_async_generator_lowering_in_input("js");
}

#[test]
fn build_source_file_rejects_async_generator_functions_in_jsx_input() {
    assert_build_source_file_rejects_async_generator_lowering_in_input("jsx");
}

#[test]
fn build_source_file_rejects_async_generator_functions_in_tsx_input() {
    assert_build_source_file_rejects_async_generator_lowering_in_input("tsx");
}

#[test]
fn build_source_file_rejects_anonymous_default_export_generator_function_declarations_in_supported_input_matrix(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_build_source_file_rejects_generator_lowering_for_source_in_input(
            extension,
            "export default function*() { yield* []; }\n",
        );
    }
}

#[test]
fn build_source_file_rejects_anonymous_default_export_async_generator_function_declarations_in_supported_input_matrix(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_build_source_file_rejects_generator_lowering_for_source_in_input(
            extension,
            "export default async function*() { yield 1; }\n",
        );
    }
}

#[test]
fn build_source_file_rejects_anonymous_default_export_async_generator_function_declarations_with_yield_delegation_in_supported_input_matrix(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_build_source_file_rejects_generator_lowering_for_source_in_input(
            extension,
            "export default async function*() { yield* []; }\n",
        );
    }
}

#[test]
fn build_source_file_rejects_generator_functions_in_browser_ts_input() {
    assert_build_source_file_rejects_generator_lowering_in_browser_input("ts");
}

#[test]
fn build_source_file_rejects_generator_functions_in_browser_js_input() {
    assert_build_source_file_rejects_generator_lowering_in_browser_input("js");
}

#[test]
fn build_source_file_rejects_generator_functions_in_browser_jsx_input() {
    assert_build_source_file_rejects_generator_lowering_in_browser_input("jsx");
}

#[test]
fn build_source_file_rejects_generator_functions_in_browser_tsx_input() {
    assert_build_source_file_rejects_generator_lowering_in_browser_input("tsx");
}

#[test]
fn build_source_file_rejects_async_generator_functions_in_browser_ts_input() {
    assert_build_source_file_rejects_async_generator_lowering_in_browser_input("ts");
}

#[test]
fn build_source_file_rejects_async_generator_functions_in_browser_js_input() {
    assert_build_source_file_rejects_async_generator_lowering_in_browser_input("js");
}

#[test]
fn build_source_file_rejects_async_generator_functions_in_browser_jsx_input() {
    assert_build_source_file_rejects_async_generator_lowering_in_browser_input("jsx");
}

#[test]
fn build_source_file_rejects_async_generator_functions_in_browser_tsx_input() {
    assert_build_source_file_rejects_async_generator_lowering_in_browser_input("tsx");
}

#[test]
fn build_source_file_rejects_class_generator_methods_in_deno_and_browser_input() {
    assert_build_source_file_rejects_class_generator_methods_in_input(ApiSurface::Deno);
    assert_build_source_file_rejects_class_generator_methods_in_input(ApiSurface::Browser);
}

#[test]
fn build_source_file_rejects_permission_escalation_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"Deno.permissions["request"](); Deno.permissions["revoke"](); globalThis.Deno.permissions["request"](); globalThis.Deno.permissions["revoke"](); globalThis["Deno"]["permissions"]["request"](); globalThis["Deno"]["permissions"]["revoke"]();"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("permission escalation APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(
            |diagnostic| diagnostic.message.contains("permission escalation API")
                && (diagnostic.message.contains("Deno.permissions.request")
                    || diagnostic
                        .message
                        .contains("globalThis.Deno.permissions.request")
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Deno"]["permissions"]["request"]"#))
        ),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_permission_escalation_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"Deno.permissions["request"](); Deno.permissions["revoke"](); globalThis.Deno.permissions["request"](); globalThis.Deno.permissions["revoke"](); globalThis["Deno"]["permissions"]["request"](); globalThis["Deno"]["permissions"]["revoke"]();"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("permission escalation APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(
            |diagnostic| diagnostic.message.contains("permission escalation API")
                && (diagnostic.message.contains("Deno.permissions.request")
                    || diagnostic
                        .message
                        .contains("globalThis.Deno.permissions.request")
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Deno"]["permissions"]["request"]"#))
        ),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_permission_escalation_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.jsx");
    fs::write(
        &source_path,
        r#"Deno.permissions["request"](); Deno.permissions["revoke"](); globalThis.Deno.permissions["request"](); globalThis.Deno.permissions["revoke"](); globalThis["Deno"]["permissions"]["request"](); globalThis["Deno"]["permissions"]["revoke"]();"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("permission escalation APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(
            |diagnostic| diagnostic.message.contains("permission escalation API")
                && (diagnostic.message.contains("Deno.permissions.request")
                    || diagnostic
                        .message
                        .contains("globalThis.Deno.permissions.request")
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Deno"]["permissions"]["request"]"#))
        ),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_permission_escalation_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(
        &source_path,
        r#"Deno.permissions["request"](); Deno.permissions["revoke"](); globalThis.Deno.permissions["request"](); globalThis.Deno.permissions["revoke"](); globalThis["Deno"]["permissions"]["request"](); globalThis["Deno"]["permissions"]["revoke"]();"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("permission escalation APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(
            |diagnostic| diagnostic.message.contains("permission escalation API")
                && (diagnostic.message.contains("Deno.permissions.request")
                    || diagnostic
                        .message
                        .contains("globalThis.Deno.permissions.request")
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Deno"]["permissions"]["request"]"#))
        ),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_mixed_bracket_dot_permission_escalation_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"globalThis["Deno"].permissions["request"](); globalThis["Deno"].permissions.request(); globalThis["Deno"].permissions["revoke"](); globalThis["Deno"].permissions.revoke();"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("mixed-bracket permission escalation APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| {
            diagnostic.message.contains("permission escalation API")
                && (diagnostic.message.contains("Deno.permissions.request")
                    || diagnostic
                        .message
                        .contains("globalThis.Deno.permissions.request")
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Deno"].permissions.request"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Deno"].permissions.revoke"#))
        }),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_bracketed_permission_object_escalation_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"globalThis["Deno"]["permissions"].request(); globalThis["Deno"]["permissions"].revoke();"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("bracketed permission object escalation APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| {
            diagnostic.message.contains("permission escalation API")
                && (diagnostic
                    .message
                    .contains(r#"globalThis["Deno"]["permissions"]["request"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Deno"]["permissions"]["revoke"]"#))
        }),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_bracketed_permission_object_escalation_in_browser_api_surface_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"globalThis["Deno"]["permissions"].request(); globalThis["Deno"]["permissions"].revoke();"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Browser,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("browser permission object escalation APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| {
            diagnostic.message.contains("permission escalation API")
                && (diagnostic
                    .message
                    .contains(r#"globalThis["Deno"]["permissions"]["request"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Deno"]["permissions"]["revoke"]"#))
        }),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_mixed_bracket_dot_permission_escalation_in_browser_api_surface_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"globalThis["Deno"].permissions["request"](); globalThis["Deno"].permissions.request(); globalThis["Deno"].permissions["revoke"](); globalThis["Deno"].permissions.revoke();"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Browser,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("browser mixed-bracket permission escalation APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| {
            diagnostic.message.contains("permission escalation API")
                && (diagnostic.message.contains("Deno.permissions.request")
                    || diagnostic
                        .message
                        .contains("globalThis.Deno.permissions.request")
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Deno"].permissions.request"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Deno"].permissions.revoke"#))
        }),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_mixed_bracket_dot_permission_escalation_in_browser_api_surface_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"globalThis["Deno"].permissions["request"](); globalThis["Deno"].permissions.request(); globalThis["Deno"].permissions["revoke"](); globalThis["Deno"].permissions.revoke();"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Browser,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("browser mixed-bracket permission escalation APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| {
            diagnostic.message.contains("permission escalation API")
                && (diagnostic.message.contains("Deno.permissions.request")
                    || diagnostic
                        .message
                        .contains("globalThis.Deno.permissions.request")
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Deno"].permissions.request"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Deno"].permissions.revoke"#))
        }),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_permission_escalation_in_browser_api_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"Deno.permissions.request(); Deno.permissions.revoke(); globalThis.Deno.permissions.request(); globalThis.Deno.permissions.revoke();"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Browser,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("browser permission escalation APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| {
            diagnostic.message.contains("permission escalation API")
                && (diagnostic.message.contains("Deno.permissions.request")
                    || diagnostic
                        .message
                        .contains("globalThis.Deno.permissions.request"))
        }),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_mixed_bracket_dot_permission_escalation_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(
        &source_path,
        r#"globalThis["Deno"].permissions["request"](); globalThis["Deno"].permissions.request(); globalThis["Deno"].permissions["revoke"](); globalThis["Deno"].permissions.revoke();"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("mixed-bracket permission escalation APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| {
            diagnostic.message.contains("permission escalation API")
                && (diagnostic.message.contains("Deno.permissions.request")
                    || diagnostic
                        .message
                        .contains("globalThis.Deno.permissions.request")
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Deno"].permissions.request"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Deno"].permissions.revoke"#))
        }),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_process_env_mutation_in_ts_input() {
    assert_build_source_file_rejects_process_env_mutation_in_input(ApiSurface::Deno, "ts");
}

#[test]
fn build_source_file_rejects_process_env_mutation_in_js_input() {
    assert_build_source_file_rejects_process_env_mutation_in_input(ApiSurface::Deno, "js");
}

#[test]
fn build_source_file_rejects_process_env_mutation_in_jsx_input() {
    assert_build_source_file_rejects_process_env_mutation_in_input(ApiSurface::Deno, "jsx");
}

#[test]
fn build_source_file_rejects_process_env_mutation_in_tsx_input() {
    assert_build_source_file_rejects_process_env_mutation_in_input(ApiSurface::Deno, "tsx");
}

#[test]
fn build_source_file_rejects_process_env_mutation_in_browser_api_surface_in_ts_input() {
    assert_build_source_file_rejects_process_env_mutation_in_input(ApiSurface::Browser, "ts");
}

#[test]
fn build_source_file_rejects_process_env_mutation_in_browser_api_surface_in_js_input() {
    assert_build_source_file_rejects_process_env_mutation_in_input(ApiSurface::Browser, "js");
}

#[test]
fn build_source_file_rejects_process_env_mutation_in_browser_api_surface_in_jsx_input() {
    assert_build_source_file_rejects_process_env_mutation_in_input(ApiSurface::Browser, "jsx");
}

#[test]
fn build_source_file_rejects_process_env_mutation_in_browser_api_surface_in_tsx_input() {
    assert_build_source_file_rejects_process_env_mutation_in_input(ApiSurface::Browser, "tsx");
}

#[test]
fn build_source_file_rejects_bracketed_process_env_mutation_in_browser_api_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"] = {};"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Browser,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("bracketed process env mutation should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("environment mutation API 'process.env'")
                && (diagnostic.message.contains("process.env")
                    || diagnostic.message.contains(r#"process["env"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"]"#))
        }),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_deleted_bracketed_process_env_mutation_in_browser_api_surface_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"delete globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"]"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Browser,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("deleted bracketed process env mutation should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("environment mutation API 'process.env'")
                && (diagnostic.message.contains("process.env")
                    || diagnostic.message.contains(r#"process["env"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis["process"].env["KALI_BROWSER_ENV_MUTATION"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"]"#))
        }),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_deleted_mixed_bracketed_process_env_mutation_in_browser_api_surface_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"delete globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"]"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Browser,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("deleted mixed bracketed process env mutation should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("environment mutation API 'process.env'")
                && (diagnostic.message.contains("process.env")
                    || diagnostic.message.contains(r#"process["env"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis.process["env"]["KALI_BROWSER_ENV_MUTATION"]"#))
        }),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_mixed_bracket_dot_process_control_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"globalThis["process"].pid; globalThis["process"].cwd; globalThis["process"].chdir; globalThis["process"].exit;"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("mixed bracket/dot process control should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    for expected in [
        "globalThis.process.pid",
        "globalThis.process.cwd",
        "globalThis.process.chdir",
        "globalThis.process.exit",
    ] {
        assert!(
            error
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "missing {expected} in {error:?}"
        );
    }
}

#[test]
fn build_source_file_rejects_mixed_bracket_dot_process_control_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"globalThis["process"].pid; globalThis["process"].cwd; globalThis["process"].chdir; globalThis["process"].exit;"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("mixed bracket/dot process control should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    for expected in [
        "globalThis.process.pid",
        "globalThis.process.cwd",
        "globalThis.process.chdir",
        "globalThis.process.exit",
    ] {
        assert!(
            error
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "missing {expected} in {error:?}"
        );
    }
}

#[test]
fn build_source_file_rejects_bracketed_process_control_in_js_input() {
    assert_build_source_file_rejects_bracketed_process_control_in_input(ApiSurface::Deno, "js");
}

#[test]
fn build_source_file_rejects_bracketed_process_control_in_ts_input() {
    assert_build_source_file_rejects_bracketed_process_control_in_input(ApiSurface::Deno, "ts");
}

#[test]
fn build_source_file_rejects_bracketed_process_control_in_browser_api_surface_in_js_input() {
    assert_build_source_file_rejects_bracketed_process_control_in_input(ApiSurface::Browser, "js");
}

#[test]
fn build_source_file_rejects_bracketed_process_control_in_browser_api_surface_in_ts_input() {
    assert_build_source_file_rejects_bracketed_process_control_in_input(ApiSurface::Browser, "ts");
}

#[test]
fn build_source_file_rejects_bracketed_process_control_in_browser_api_surface_in_jsx_input() {
    assert_build_source_file_rejects_bracketed_process_control_in_input(ApiSurface::Browser, "jsx");
}

#[test]
fn build_source_file_rejects_bracketed_process_control_in_browser_api_surface_in_tsx_input() {
    assert_build_source_file_rejects_bracketed_process_control_in_input(ApiSurface::Browser, "tsx");
}

#[test]
fn build_source_file_rejects_process_kill_zero_probe_in_browser_api_surface_in_js_input() {
    assert_build_source_file_rejects_process_kill_zero_probe_in_input(ApiSurface::Browser, "js");
}

#[test]
fn build_source_file_rejects_process_kill_zero_probe_in_browser_api_surface_in_ts_input() {
    assert_build_source_file_rejects_process_kill_zero_probe_in_input(ApiSurface::Browser, "ts");
}

#[test]
fn build_source_file_rejects_process_kill_zero_probe_in_browser_api_surface_in_jsx_input() {
    assert_build_source_file_rejects_process_kill_zero_probe_in_input(ApiSurface::Browser, "jsx");
}

#[test]
fn build_source_file_rejects_process_kill_zero_probe_in_browser_api_surface_in_tsx_input() {
    assert_build_source_file_rejects_process_kill_zero_probe_in_input(ApiSurface::Browser, "tsx");
}

#[test]
fn build_source_file_rejects_bracketed_deno_network_in_browser_api_surface_in_js_input() {
    assert_build_source_file_rejects_bracketed_deno_network_in_input(ApiSurface::Browser, "js");
}

#[test]
fn build_source_file_rejects_bracketed_deno_network_in_browser_api_surface_in_ts_input() {
    assert_build_source_file_rejects_bracketed_deno_network_in_input(ApiSurface::Browser, "ts");
}

#[test]
fn build_source_file_rejects_mixed_bracket_dot_permission_escalation_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"globalThis["Deno"].permissions["request"](); globalThis["Deno"].permissions["revoke"]();"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("mixed-bracket permission escalation APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| {
            diagnostic.message.contains("permission escalation API")
                && (diagnostic.message.contains("Deno.permissions.request")
                    || diagnostic
                        .message
                        .contains("globalThis.Deno.permissions.request"))
        }),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_mixed_bracket_dot_permission_escalation_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.jsx");
    fs::write(
        &source_path,
        r#"globalThis["Deno"].permissions["request"](); globalThis["Deno"].permissions["revoke"]();"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("mixed-bracket permission escalation APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| {
            diagnostic.message.contains("permission escalation API")
                && (diagnostic.message.contains("Deno.permissions.request")
                    || diagnostic
                        .message
                        .contains("globalThis.Deno.permissions.request"))
        }),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_deno_env_to_object_in_browser_api_surface_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"Deno.env.toObject; globalThis.Deno.env.toObject; globalThis.Deno.env["toObject"]; Deno.env["toObject"]; Deno["env"]["toObject"]; Deno["env"].toObject; globalThis.Deno["env"]["toObject"]; globalThis.Deno["env"].toObject; globalThis.Deno["env"]["toObject"]; globalThis["Deno"].env.toObject; globalThis["Deno"].env["toObject"]; globalThis["Deno"]["env"].toObject; globalThis["Deno"]["env"]["toObject"]; globalThis.Deno["env"]["toObject"];"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Browser,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("browser env materialization APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| diagnostic
            .message
            .contains("environment snapshot materialization API")
            && diagnostic.message.contains("object-aggregate lowering")
            && (diagnostic.message.contains("Deno.env.toObject")
                || diagnostic.message.contains("globalThis.Deno.env.toObject")
                || diagnostic
                    .message
                    .contains(r#"globalThis["Deno"].env.toObject"#)
                || diagnostic
                    .message
                    .contains(r#"globalThis.Deno.env["toObject"]"#)
                || diagnostic
                    .message
                    .contains(r#"globalThis["Deno"].env["toObject"]"#)
                || diagnostic
                    .message
                    .contains(r#"globalThis["Deno"]["env"]["toObject"]"#))),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_broader_intl_apis_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"globalThis['Intl']['DateTimeFormat']; globalThis["Intl"]["DateTimeFormat"]; globalThis['Intl']['RelativeTimeFormat']; globalThis["Intl"]["RelativeTimeFormat"]; globalThis['Intl']['PluralRules']; globalThis["Intl"]["PluralRules"]; globalThis['Intl']['Collator']; globalThis["Intl"]["Collator"]; globalThis['Intl']['DisplayNames']; globalThis["Intl"]["DisplayNames"]; globalThis['Intl']['Segmenter']; globalThis["Intl"]["Segmenter"]; globalThis['Intl']['Locale']; globalThis["Intl"]["Locale"]; globalThis['Intl']['NumberFormat']; globalThis["Intl"]["NumberFormat"]; Intl.NumberFormat; Intl.RelativeTimeFormat; Intl.Collator; Intl.DisplayNames; Intl.Segmenter; Intl.Locale;"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("broader Intl APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| {
            diagnostic.message.contains("broader Intl support")
                && (diagnostic.message.contains("Intl.DateTimeFormat")
                    || diagnostic.message.contains("Intl.RelativeTimeFormat")
                    || diagnostic.message.contains("Intl.PluralRules")
                    || diagnostic.message.contains("Intl.Collator")
                    || diagnostic.message.contains("Intl.DisplayNames")
                    || diagnostic.message.contains("Intl.Locale")
                    || diagnostic.message.contains("Intl.NumberFormat")
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Intl"]["NumberFormat"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Intl"]["DateTimeFormat"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Intl"]["RelativeTimeFormat"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Intl"]["PluralRules"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Intl"]["Collator"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Intl"]["DisplayNames"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Intl"]["Segmenter"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Intl"]["Locale"]"#))
        }),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_broader_intl_apis_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"globalThis['Intl']['DateTimeFormat']; globalThis["Intl"]["DateTimeFormat"]; globalThis['Intl']['RelativeTimeFormat']; globalThis["Intl"]["RelativeTimeFormat"]; globalThis['Intl']['PluralRules']; globalThis["Intl"]["PluralRules"]; globalThis['Intl']['Collator']; globalThis["Intl"]["Collator"]; globalThis['Intl']['DisplayNames']; globalThis["Intl"]["DisplayNames"]; globalThis['Intl']['Segmenter']; globalThis["Intl"]["Segmenter"]; globalThis['Intl']['Locale']; globalThis["Intl"]["Locale"]; globalThis['Intl']['NumberFormat']; globalThis["Intl"]["NumberFormat"]; Intl.NumberFormat; Intl.RelativeTimeFormat; Intl.Collator; Intl.DisplayNames; Intl.Segmenter; Intl.Locale;"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("broader Intl APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| {
            diagnostic.message.contains("broader Intl support")
                && (diagnostic.message.contains("Intl.DateTimeFormat")
                    || diagnostic.message.contains("Intl.RelativeTimeFormat")
                    || diagnostic.message.contains("Intl.PluralRules")
                    || diagnostic.message.contains("Intl.Collator")
                    || diagnostic.message.contains("Intl.DisplayNames")
                    || diagnostic.message.contains("Intl.Locale")
                    || diagnostic.message.contains("Intl.NumberFormat")
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Intl"]["NumberFormat"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Intl"]["DateTimeFormat"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Intl"]["RelativeTimeFormat"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Intl"]["PluralRules"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Intl"]["Collator"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Intl"]["DisplayNames"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Intl"]["Segmenter"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Intl"]["Locale"]"#))
        }),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_broader_intl_apis_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.jsx");
    fs::write(
        &source_path,
        r#"globalThis['Intl']['DateTimeFormat']; globalThis["Intl"]["DateTimeFormat"]; globalThis['Intl']['RelativeTimeFormat']; globalThis["Intl"]["RelativeTimeFormat"]; globalThis['Intl']['PluralRules']; globalThis["Intl"]["PluralRules"]; globalThis['Intl']['Collator']; globalThis["Intl"]["Collator"]; globalThis['Intl']['DisplayNames']; globalThis["Intl"]["DisplayNames"]; globalThis['Intl']['Segmenter']; globalThis["Intl"]["Segmenter"]; globalThis['Intl']['Locale']; globalThis["Intl"]["Locale"]; globalThis['Intl']['NumberFormat']; globalThis["Intl"]["NumberFormat"]; Intl.NumberFormat; Intl.RelativeTimeFormat; Intl.Collator; Intl.DisplayNames; Intl.Segmenter; Intl.Locale;"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("broader Intl APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| {
            diagnostic.message.contains("broader Intl support")
                && (diagnostic.message.contains("Intl.DateTimeFormat")
                    || diagnostic.message.contains("Intl.RelativeTimeFormat")
                    || diagnostic.message.contains("Intl.PluralRules")
                    || diagnostic.message.contains("Intl.Collator")
                    || diagnostic.message.contains("Intl.DisplayNames")
                    || diagnostic.message.contains("Intl.Locale")
                    || diagnostic.message.contains("Intl.NumberFormat")
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Intl"]["NumberFormat"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis[\"Intl\"][\"DateTimeFormat\"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis[\"Intl\"][\"RelativeTimeFormat\"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis[\"Intl\"][\"PluralRules\"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis[\"Intl\"][\"Collator\"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis[\"Intl\"][\"DisplayNames\"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis[\"Intl\"][\"Segmenter\"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis[\"Intl\"][\"Locale\"]"#))
        }),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_broader_intl_apis_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(
        &source_path,
        r#"globalThis['Intl']['DateTimeFormat']; globalThis["Intl"]["DateTimeFormat"]; globalThis['Intl']['RelativeTimeFormat']; globalThis["Intl"]["RelativeTimeFormat"]; globalThis['Intl']['PluralRules']; globalThis["Intl"]["PluralRules"]; globalThis['Intl']['Collator']; globalThis["Intl"]["Collator"]; globalThis['Intl']['DisplayNames']; globalThis["Intl"]["DisplayNames"]; globalThis['Intl']['Segmenter']; globalThis["Intl"]["Segmenter"]; globalThis['Intl']['Locale']; globalThis["Intl"]["Locale"]; globalThis['Intl']['NumberFormat']; globalThis["Intl"]["NumberFormat"]; Intl.NumberFormat; Intl.RelativeTimeFormat; Intl.Collator; Intl.DisplayNames; Intl.Segmenter; Intl.Locale;"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("broader Intl APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| {
            diagnostic.message.contains("broader Intl support")
                && (diagnostic.message.contains("Intl.DateTimeFormat")
                    || diagnostic.message.contains("Intl.RelativeTimeFormat")
                    || diagnostic.message.contains("Intl.PluralRules")
                    || diagnostic.message.contains("Intl.Collator")
                    || diagnostic.message.contains("Intl.DisplayNames")
                    || diagnostic.message.contains("Intl.Locale")
                    || diagnostic.message.contains("Intl.NumberFormat")
                    || diagnostic
                        .message
                        .contains(r#"globalThis["Intl"]["NumberFormat"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis[\"Intl\"][\"DateTimeFormat\"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis[\"Intl\"][\"RelativeTimeFormat\"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis[\"Intl\"][\"PluralRules\"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis[\"Intl\"][\"Collator\"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis[\"Intl\"][\"DisplayNames\"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis[\"Intl\"][\"Segmenter\"]"#)
                    || diagnostic
                        .message
                        .contains(r#"globalThis[\"Intl\"][\"Locale\"]"#))
        }),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_broader_intl_apis_in_browser_api_surface_in_js_input() {
    assert_build_source_file_rejects_broader_intl_apis_in_input(ApiSurface::Browser, "js");
}

#[test]
fn build_source_file_rejects_broader_intl_apis_in_browser_api_surface_in_ts_input() {
    assert_build_source_file_rejects_broader_intl_apis_in_input(ApiSurface::Browser, "ts");
}

#[test]
fn build_source_file_rejects_broader_intl_apis_in_browser_api_surface_in_jsx_input() {
    assert_build_source_file_rejects_broader_intl_apis_in_input(ApiSurface::Browser, "jsx");
}

#[test]
fn build_source_file_rejects_broader_intl_apis_in_browser_api_surface_in_tsx_input() {
    assert_build_source_file_rejects_broader_intl_apis_in_input(ApiSurface::Browser, "tsx");
}

#[test]
fn build_source_file_rejects_late_weak_reference_apis_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, late_threaded_runtime_source()).expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("late weak-reference APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| {
            diagnostic.message.contains("WeakMap")
                || diagnostic.message.contains("WeakSet")
                || diagnostic.message.contains("WeakRef")
                || diagnostic.message.contains("FinalizationRegistry")
                || diagnostic.message.contains("SharedArrayBuffer")
                || diagnostic.message.contains("Atomics")
                || diagnostic.message.contains("threaded runtime global")
        }),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_late_weak_reference_apis_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_threaded_runtime_source()).expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Deno,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("late weak-reference APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| {
            diagnostic.message.contains("WeakMap")
                || diagnostic.message.contains("WeakSet")
                || diagnostic.message.contains("WeakRef")
                || diagnostic.message.contains("FinalizationRegistry")
                || diagnostic.message.contains("SharedArrayBuffer")
                || diagnostic.message.contains("Atomics")
                || diagnostic.message.contains("threaded runtime global")
        }),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_late_weak_reference_apis_in_browser_api_surface_in_js_input() {
    assert_build_source_file_rejects_late_weak_reference_apis_in_input(ApiSurface::Browser, "js");
}

#[test]
fn build_source_file_rejects_late_weak_reference_apis_in_browser_api_surface_in_ts_input() {
    assert_build_source_file_rejects_late_weak_reference_apis_in_input(ApiSurface::Browser, "ts");
}

#[test]
fn build_source_file_rejects_late_weak_reference_apis_in_browser_api_surface_in_jsx_input() {
    assert_build_source_file_rejects_late_weak_reference_apis_in_input(ApiSurface::Browser, "jsx");
}

#[test]
fn build_source_file_rejects_late_weak_reference_apis_in_browser_api_surface_in_tsx_input() {
    assert_build_source_file_rejects_late_weak_reference_apis_in_input(ApiSurface::Browser, "tsx");
}

#[test]
fn build_source_file_rejects_threaded_runtime_globals_in_ts_input() {
    assert_build_source_file_rejects_threaded_runtime_globals_in_input(ApiSurface::Deno, "ts");
}

#[test]
fn build_source_file_rejects_threaded_runtime_globals_in_js_input() {
    assert_build_source_file_rejects_threaded_runtime_globals_in_input(ApiSurface::Deno, "js");
}

#[test]
fn build_source_file_rejects_threaded_runtime_globals_in_jsx_input() {
    assert_build_source_file_rejects_threaded_runtime_globals_in_input(ApiSurface::Deno, "jsx");
}

#[test]
fn build_source_file_rejects_threaded_runtime_globals_in_tsx_input() {
    assert_build_source_file_rejects_threaded_runtime_globals_in_input(ApiSurface::Deno, "tsx");
}

#[test]
fn build_source_file_rejects_threaded_runtime_globals_in_browser_api_surface_in_ts_input() {
    assert_build_source_file_rejects_threaded_runtime_globals_in_input(ApiSurface::Browser, "ts");
}

#[test]
fn build_source_file_rejects_threaded_runtime_globals_in_browser_api_surface_in_js_input() {
    assert_build_source_file_rejects_threaded_runtime_globals_in_input(ApiSurface::Browser, "js");
}

#[test]
fn build_source_file_rejects_threaded_runtime_globals_in_browser_api_surface_in_jsx_input() {
    assert_build_source_file_rejects_threaded_runtime_globals_in_input(ApiSurface::Browser, "jsx");
}

#[test]
fn build_source_file_rejects_threaded_runtime_globals_in_browser_api_surface_in_tsx_input() {
    assert_build_source_file_rejects_threaded_runtime_globals_in_input(ApiSurface::Browser, "tsx");
}
