use super::*;
use kali_common::{
    array_from_alias_inventory_source, array_from_loop_lines, late_threaded_runtime_source,
    map_constructor_frozen_callable_source, map_constructor_iteration_source,
    math_pow_browser_alias_inventory_aliases, math_pow_invocation_lines_for_aliases,
    promise_any_browser_body_source, set_constructor_frozen_callable_source,
    set_constructor_iteration_source,
};
use kali_optimize::{ProfileData, ProfileSample, ProfileSampleKind};
use sha2::{Digest, Sha256};
use std::fs;
use tempfile::tempdir;
use wasmparser::Validator;

fn assert_build_source_file_supports_deno_env_set_and_delete_in_input(extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        r#"Deno.env.set('KALI_ENV_SET_DELETE_SMOKE', 'hello-environment'); Deno.env.delete('KALI_ENV_SET_DELETE_SMOKE'); Deno["env"]["set"]('KALI_ENV_SET_DELETE_SMOKE', 'hello-environment'); Deno["env"]["delete"]('KALI_ENV_SET_DELETE_SMOKE'); globalThis["Deno"]["env"]["set"]('KALI_ENV_SET_DELETE_SMOKE', 'hello-environment'); globalThis["Deno"]["env"]["delete"]('KALI_ENV_SET_DELETE_SMOKE'); globalThis.Deno["env"]["set"]('KALI_ENV_SET_DELETE_SMOKE', 'hello-environment'); globalThis.Deno["env"]["delete"]('KALI_ENV_SET_DELETE_SMOKE'); globalThis["Deno"].env["set"]('KALI_ENV_SET_DELETE_SMOKE', 'hello-environment'); globalThis["Deno"].env["delete"]('KALI_ENV_SET_DELETE_SMOKE'); globalThis.Deno.env["set"]('KALI_ENV_SET_DELETE_SMOKE', 'hello-environment'); globalThis.Deno.env["delete"]('KALI_ENV_SET_DELETE_SMOKE');"#,
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
    .expect("build should succeed");

    assert!(output.output_path.exists(), "extension: {extension}");
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

fn assert_build_source_file_rejects_unsupported_permission_query_descriptors_in_input(
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        r#"Deno.permissions.query({ name: "ffi" });
Deno.permissions.query({ name: "sys" });
Deno.permissions["query"]({ name: "ffi" });
Deno.permissions["query"]({ name: "sys" });
Deno["permissions"].query({ name: "ffi" });
Deno["permissions"].query({ name: "sys" });
Deno["permissions"]["query"]({ name: "ffi" });
Deno["permissions"]["query"]({ name: "sys" });
globalThis.Deno.permissions.query({ name: "ffi" });
globalThis.Deno.permissions.query({ name: "sys" });
globalThis.Deno.permissions["query"]({ name: "ffi" });
globalThis.Deno.permissions["query"]({ name: "sys" });
globalThis.Deno["permissions"].query({ name: "ffi" });
globalThis.Deno["permissions"].query({ name: "sys" });
globalThis.Deno["permissions"]["query"]({ name: "ffi" });
globalThis.Deno["permissions"]["query"]({ name: "sys" });
globalThis["Deno"]["permissions"].query({ name: "ffi" });
globalThis["Deno"]["permissions"].query({ name: "sys" });
globalThis["Deno"]["permissions"]["query"]({ name: "ffi" });
globalThis["Deno"]["permissions"]["query"]({ name: "sys" });
"#,
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
    .expect_err("unsupported permission query descriptors should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| diagnostic
            .message
            .contains("permission query descriptor 'ffi'")),
        "unexpected diagnostics: {error:?}"
    );
    assert!(
        error.iter().any(|diagnostic| diagnostic
            .message
            .contains("permission query descriptor 'sys'")),
        "unexpected diagnostics: {error:?}"
    );
}

fn assert_build_source_file_rejects_late_subprocess_global_in_input(extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        r#"Deno.connect;
Deno.listen;
Deno.serve;
new Deno.Command("sh");
"#,
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
    .expect_err("late subprocess and network APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    for expected in ["Deno.connect", "Deno.listen", "Deno.serve", "Deno.Command"] {
        assert!(
            error
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "missing {expected} in {error:?}"
        );
    }
}

fn assert_build_source_file_supports_object_has_own_call_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        r#"const object = Object.fromEntries([["a", 1], ["b", 2]]); const hasOwn = Object.hasOwn; const hasOwnPropertyCall = Object.prototype.hasOwnProperty.call; Object.hasOwn(object, "a"); hasOwn(object, "a"); globalThis.Object.hasOwn(object, "a"); globalThis["Object"]["hasOwn"](object, "a"); Object.freeze(globalThis.Object.hasOwn)(object, "a"); Object.freeze(globalThis["Object"].hasOwn)(object, "a"); Object.freeze((globalThis["Object"].hasOwn))(object, "a"); Object.freeze((globalThis["Object"]["hasOwn"]))(object, "a"); Object.freeze(globalThis?.Object.hasOwn)(object, "a"); Object.freeze((globalThis?.Object.hasOwn))(object, "a"); Object.freeze(globalThis?.Object["hasOwn"])(object, "a"); Object.freeze((globalThis?.Object["hasOwn"]))(object, "a"); Object.prototype.hasOwnProperty.call(object, "a"); hasOwnPropertyCall(object, "a"); globalThis.Object.prototype.hasOwnProperty.call(object, "a"); globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](object, "a"); Object.freeze(globalThis?.Object.prototype.hasOwnProperty.call)(object, "a"); Object.freeze((globalThis?.Object.prototype.hasOwnProperty.call))(object, "a"); Object.freeze(globalThis?.Object.prototype.hasOwnProperty["call"])(object, "a"); Object.freeze((globalThis?.Object.prototype.hasOwnProperty["call"]))(object, "a");"#,
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("browser object-model helpers should succeed");

    assert!(!output.wasm_bytes.is_empty());
}

fn assert_build_source_file_supports_object_is_primitive_literals_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        r#"const zero = 0; const zeroAlias = zero; console.log(Object.is(zeroAlias, -0)); console.log(Object.is(+1, 1)); console.log(Object.is(true, true)); console.log(Object.is("hello", "hello")); console.log(Object.is(1n, 1n)); console.log(Object.is(-1n, -1n)); console.log(Object.is(null, null)); console.log(Object.is(Infinity, Infinity)); console.log(Object.is(NaN, NaN)); console.log(Object.is(-Infinity, -Infinity)); console.log(globalThis["Object"]["is"](+1, 1)); console.log(globalThis.Object["is"](+1, 1)); console.log(globalThis["Object"].is(+1, 1)); console.log(globalThis.Object.is(+1, 1)); console.log(Object["is"](+1, 1));"#,
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Object.is primitive-literal build should succeed");

    assert!(!output.wasm_bytes.is_empty());
}

fn assert_build_source_file_supports_object_is_same_reference_alias_chain_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        r#"const object = { a: 1 }; const alias = object; const frozen = Object.freeze(object); console.log(globalThis["Object"]["is"](alias, object)); console.log(globalThis.Object["is"](frozen, object)); console.log(globalThis["Object"].is(alias, object)); console.log(globalThis.Object.is(frozen, object)); console.log(globalThis["Object"]["is"](frozen, object)); console.log(globalThis["Object"].is(frozen, object)); console.log(globalThis.Object["is"](frozen, object)); console.log(Object["is"](frozen, object));"#,
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Object.is same-reference alias-chain build should succeed");

    assert!(!output.wasm_bytes.is_empty());
}

fn assert_check_source_file_supports_number_predicates_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        r#"const alias = 1; if (!Number.isFinite(alias) || !Number.isInteger(alias) || !Number.isSafeInteger(alias)) { throw new Error('expected positive integer predicates'); } if (Number.isInteger(1.5) || Number.isFinite('hello') || Number.isSafeInteger(1.5)) { throw new Error('expected negative primitive predicate cases'); } if (!globalThis["Number"]["isNaN"](NaN) || globalThis.Number.isNaN(1) || !globalThis["Number"]["isFinite"](alias) || !globalThis["Number"]["isInteger"](alias) || !globalThis["Number"]["isSafeInteger"](alias) || globalThis.Number["isNaN"](1) || !globalThis["Number"].isFinite(alias) || !globalThis.Number["isInteger"](alias) || !globalThis["Number"].isSafeInteger(alias)) { throw new Error('expected bracketed Number predicate aliases'); }"#,
    )
    .expect("write source");

    check_source_file(&source_path, api_surface, &[], false, false)
        .expect("Number predicate sources should succeed");
}

fn assert_build_source_file_supports_number_predicates_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        r#"const alias = 1; if (!Number.isFinite(alias) || !Number.isInteger(alias) || !Number.isSafeInteger(alias)) { throw new Error('expected positive integer predicates'); } if (Number.isInteger(1.5) || Number.isFinite('hello') || Number.isSafeInteger(1.5)) { throw new Error('expected negative primitive predicate cases'); } if (!globalThis["Number"]["isNaN"](NaN) || globalThis.Number.isNaN(1) || !globalThis["Number"]["isFinite"](alias) || !globalThis["Number"]["isInteger"](alias) || !globalThis["Number"]["isSafeInteger"](alias) || globalThis.Number["isNaN"](1) || !globalThis["Number"].isFinite(alias) || !globalThis.Number["isInteger"](alias) || !globalThis["Number"].isSafeInteger(alias)) { throw new Error('expected bracketed Number predicate aliases'); }"#,
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Number predicate build should succeed");

    assert!(!output.wasm_bytes.is_empty());
}

fn promise_all_settled_source_variants() -> [&'static str; 28] {
    [
        "console.log(Promise.allSettled([1, 2]));\n",
        "console.log(Promise[\"allSettled\"]([1, 2]));\n",
        "console.log(Promise['allSettled']([1, 2]));\n",
        "console.log(globalThis.Promise.allSettled([1, 2]));\n",
        "console.log(globalThis.Promise[\"allSettled\"]([1, 2]));\n",
        "console.log(globalThis.Promise['allSettled']([1, 2]));\n",
        "console.log(globalThis[\"Promise\"].allSettled([1, 2]));\n",
        "console.log(globalThis['Promise'].allSettled([1, 2]));\n",
        "console.log(globalThis[\"Promise\"][\"allSettled\"]([1, 2]));\n",
        "console.log(globalThis['Promise']['allSettled']([1, 2]));\n",
        "console.log(Object.freeze((globalThis.Promise)[\"allSettled\"])([1, 2]));\n",
        "console.log(Object.freeze((globalThis[\"Promise\"])[\"allSettled\"])([1, 2]));\n",
        "console.log(Object.freeze((globalThis[\"Promise\"])['allSettled'])([1, 2]));\n",
        "console.log(Object.freeze((globalThis['Promise'])['allSettled'])([1, 2]));\n",
        "console.log(Object.freeze(Promise[\"allSettled\"])([1, 2]));\n",
        "console.log(Object.freeze((Promise[\"allSettled\"]))([1, 2]));\n",
        "console.log(Object.freeze(Promise.allSettled)([1, 2]));\n",
        "console.log(Object.freeze((Promise.allSettled))([1, 2]));\n",
        "console.log(Object.freeze(globalThis.Promise.allSettled)([1, 2]));\n",
        "console.log(Object.freeze((globalThis.Promise.allSettled))([1, 2]));\n",
        "console.log(Object.freeze(globalThis.Promise[\"allSettled\"])([1, 2]));\n",
        "console.log(Object.freeze((globalThis.Promise[\"allSettled\"]))([1, 2]));\n",
        "console.log(Object.freeze(globalThis[\"Promise\"].allSettled)([1, 2]));\n",
        "console.log(Object.freeze((globalThis[\"Promise\"].allSettled))([1, 2]));\n",
        "console.log(Object.freeze(globalThis['Promise'].allSettled)([1, 2]));\n",
        "console.log(Object.freeze((globalThis['Promise'].allSettled))([1, 2]));\n",
        "console.log(Object.freeze(globalThis['Promise']['allSettled'])([1, 2]));\n",
        "console.log(Object.freeze((globalThis['Promise']['allSettled']))([1, 2]));\n",
    ]
}

fn assert_build_source_file_supports_promise_all_settled_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    for source in promise_all_settled_source_variants() {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, source).expect("write source");

        let output = build_source_file(
            &source_path,
            BuildMode::Fast,
            api_surface,
            false,
            &[],
            16,
            None,
            None,
        )
        .expect("Promise.allSettled should succeed");

        Validator::new()
            .validate_all(&output.wasm_bytes)
            .expect("generated wasm should validate");
    }
}

fn promise_any_source() -> String {
    format!(
        "async function promiseAnySmoke() {{\n{}\n}}\npromiseAnySmoke();\n",
        promise_any_browser_body_source()
    )
}

fn assert_build_source_file_supports_promise_any_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, promise_any_source()).expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Promise.any should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn unsupported_math_member_call_source_variants(method: &str) -> Vec<String> {
    vec![
        format!("console.log(Math.{method}(1.6));\n"),
        format!("console.log(Math[\"{method}\"](1.6));\n"),
        format!("console.log(Math['{method}'](1.6));\n"),
        format!("console.log(globalThis.Math.{method}(1.6));\n"),
        format!("console.log(globalThis.Math[\"{method}\"](1.6));\n"),
        format!("console.log(globalThis.Math['{method}'](1.6));\n"),
        format!("console.log(globalThis[\"Math\"].{method}(1.6));\n"),
        format!("console.log(globalThis[\"Math\"][\"{method}\"](1.6));\n"),
        format!("console.log(globalThis[\"Math\"]['{method}'](1.6));\n"),
        format!("console.log(globalThis['Math'].{method}(1.6));\n"),
        format!("console.log(globalThis['Math'][\"{method}\"](1.6));\n"),
        format!("console.log(globalThis['Math']['{method}'](1.6));\n"),
    ]
}

fn assert_build_source_file_rejects_unsupported_math_member_calls_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    // NOTE: `sqrt` is no longer listed here — it gained runtime support via F64Sqrt
    // (codegen commit e5d776d93). `exp`/`log` remain genuinely unavailable.
    for method in ["exp", "log"] {
        for source in unsupported_math_member_call_source_variants(method) {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join(format!("main.{extension}"));
            fs::write(&source_path, source).expect("write source");

            let error = build_source_file(
                &source_path,
                BuildMode::Fast,
                api_surface,
                false,
                &[],
                16,
                None,
                None,
            )
            .expect_err("unsupported Math member call should fail");

            assert!(error.iter().any(|diagnostic| diagnostic.code
                == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
            assert!(
                error
                    .iter()
                    .any(|diagnostic| diagnostic.message.contains(&format!("Math.{method}"))),
                "unexpected diagnostics: {error:?}"
            );
        }
    }
}

fn optional_chain_math_pow_source_variants() -> [&'static str; 6] {
    [
        "console.log(Math?.pow(2, 3));\n",
        "console.log(Math?.[\"pow\"](2, 3));\n",
        "console.log(globalThis.Math?.pow(2, 3));\n",
        "console.log(globalThis[\"Math\"]?.pow(2, 3));\n",
        "console.log(globalThis['Math']?.pow(2, 3));\n",
        "console.log(globalThis?.Math?.pow(2, 3));\n",
    ]
}

fn assert_build_source_file_rejects_optional_chain_wrapped_math_pow_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    for source in optional_chain_math_pow_source_variants() {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, source).expect("write source");

        let error = build_source_file(
            &source_path,
            BuildMode::Fast,
            api_surface,
            false,
            &[],
            16,
            None,
            None,
        )
        .expect_err("optional-chain wrapped Math.pow should fail");

        assert!(error.iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
        assert!(error
            .iter()
            .any(|diagnostic| diagnostic.message.contains("optional-chain wrappers")));
    }
}

fn assert_build_source_file_rejects_negative_math_pow_exponents_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, "console.log(Math.pow(2, -1));\n").expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("Math.pow negative exponents should fail");

    assert!(
        error.iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "unexpected diagnostics: {error:?}"
    );
    assert!(
        error.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("Math.pow is unavailable for negative numeric literals")
        }),
        "unexpected diagnostics: {error:?}"
    );
}

fn assert_build_source_file_supports_math_floor_const_numeric_alias_chain_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const value = 1.6; const alias = value; console.log(Math.floor(alias)); console.log(Math.trunc(alias)); console.log(Math.ceil(alias)); console.log(Object.freeze(globalThis.Math[\"floor\"])(alias)); console.log(Object.freeze((globalThis.Math[\"floor\"]))(alias)); console.log(Object.freeze(globalThis[\"Math\"][\"floor\"])(alias)); console.log(Object.freeze((globalThis[\"Math\"][\"floor\"]))(alias)); console.log(Object.freeze(globalThis.Math[\"trunc\"])(alias)); console.log(Object.freeze((globalThis.Math[\"trunc\"]))(alias)); console.log(Object.freeze(globalThis[\"Math\"][\"trunc\"])(alias)); console.log(Object.freeze((globalThis[\"Math\"][\"trunc\"]))(alias)); console.log(Object.freeze(globalThis.Math[\"ceil\"])(alias)); console.log(Object.freeze((globalThis.Math[\"ceil\"]))(alias)); console.log(Object.freeze(globalThis[\"Math\"][\"ceil\"])(alias)); console.log(Object.freeze((globalThis[\"Math\"][\"ceil\"]))(alias));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Math.floor const alias chain should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_math_ceil_and_trunc_const_numeric_alias_chain_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const value = 1.6; const alias = value; console.log(Math.ceil(alias)); console.log(Math.trunc(alias));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Math.ceil/trunc const alias chain should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_bracketed_global_this_math_floor_trunc_and_ceil_numeric_literals_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        r#"console.log(globalThis["Math"]["floor"](1.6)); console.log(globalThis["Math"]["trunc"](1.6)); console.log(globalThis["Math"]["ceil"](1.6));
"#,
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("bracketed Math floor/trunc/ceil literal slices should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_math_round_const_numeric_alias_chain_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const value = 1.6; const alias = value; console.log(Math.round(alias));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Math.round const alias chain should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_global_this_math_round_identity_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const value = 1.6; const flag = value > 0; console.log(globalThis.Math.round(value)); console.log(globalThis.Math[\"round\"](value)); console.log(globalThis.Math['round'](value)); console.log(globalThis[\"Math\"].round(value)); console.log(globalThis[\"Math\"][\"round\"](value)); console.log(globalThis[\"Math\"]['round'](value)); console.log(globalThis['Math']['round'](value)); console.log(globalThis['Math'].round(value)); console.log(Object.freeze(globalThis.Math.round)(value)); console.log(Object.freeze(globalThis.Math[\"round\"])(value)); console.log(Object.freeze(globalThis.Math['round'])(value)); console.log(Object.freeze(globalThis[\"Math\"][\"round\"])(value)); console.log(Object.freeze(globalThis[\"Math\"]['round'])(value)); console.log(Object.freeze(globalThis['Math']['round'])(value)); console.log(Object.freeze(globalThis['Math'].round)(value)); console.log(Object.freeze((globalThis[\"Math\"].round))(value)); console.log(Object.freeze(Math.round)(value)); console.log(Object.freeze((globalThis[\"Math\"][\"round\"]))(value)); console.log(\"frozen-parenthesized-mixed-bracket\", Object.freeze((globalThis.Math[\"round\"]))(value)); console.log(Object.freeze(globalThis['Math']['round'])(value)); console.log(Object.freeze((globalThis.Math.round))(value)); console.log(Object.freeze((globalThis[\"Math\"].round))(value)); console.log(Object.freeze((Math.round))(value)); console.log(Object.freeze((globalThis['Math'])[\"round\"])(value)); console.log(Object.freeze((globalThis.Math)[\"round\"])(value)); console.log((flag ? globalThis.Math.round : Object.freeze(globalThis.Math.round))(value));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("globalThis.Math.round identity should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_math_log2_and_log10_const_numeric_alias_chain_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const log2Value = 8; const log2Alias = log2Value; console.log(Math.log2(log2Alias));\nconst log10Value = 1000; const log10Alias = log10Value; console.log(Math.log10(log10Alias));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Math.log2/log10 const alias chain should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_global_this_math_mixed_bracket_log2_and_log10_identity_literals_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const log2Value = 8; const log10Value = 1000; console.log(globalThis.Math[\"log2\"](log2Value)); console.log(globalThis.Math[\"log10\"](log10Value));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("globalThis.Math[\"log2\"]/globalThis.Math[\"log10\"] identity build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_math_hypot_perfect_square_literal_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, "console.log(Math.hypot(3, 4));\n").expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Math.hypot perfect-square build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_math_hypot_zero_arguments_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, "console.log(Math.hypot());\n").expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Math.hypot zero-argument build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_math_clz32_zero_arguments_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, "console.log(Math.clz32());\n").expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Math.clz32 zero-argument build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_math_sqrt_perfect_square_literal_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, "console.log(Math.sqrt(4));\n").expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Math.sqrt perfect-square build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_math_sqrt_perfect_square_literal_through_object_freeze_callable_wrappers_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const value = 4; console.log(Object.freeze(globalThis.Math.sqrt)(value)); console.log(Object.freeze(globalThis[\"Math\"][\"sqrt\"])(value));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Math.sqrt frozen callable wrapper build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_math_cbrt_negative_perfect_cube_literal_through_object_freeze_callable_wrappers_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const value = -27; console.log(Object.freeze(globalThis.Math.cbrt)(value)); console.log(Object.freeze(globalThis[\"Math\"][\"cbrt\"])(value));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Math.cbrt frozen callable wrapper build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, "console.log(Math.cbrt(-27));\n").expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Math.cbrt perfect-cube build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_math_inverse_hyperbolic_identity_literals_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(Math.asinh(zero)); console.log(Math.acosh(one)); console.log(Math.atanh(zero));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Math.asinh/acosh/atanh identity build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_rejects_unsupported_math_inverse_hyperbolic_member_calls_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    for (method, source) in [
        ("asinh", "console.log(Math.asinh(1.6));\n"),
        ("acosh", "console.log(Math.acosh(0));\n"),
        ("atanh", "console.log(Math.atanh(1));\n"),
    ] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, source).expect("write source");

        let error = build_source_file(
            &source_path,
            BuildMode::Fast,
            api_surface,
            false,
            &[],
            16,
            None,
            None,
        )
        .expect_err("unsupported Math inverse hyperbolic member call should fail");

        assert!(error.iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
        assert!(
            error
                .iter()
                .any(|diagnostic| diagnostic.message.contains(&format!("Math.{method}"))),
            "unexpected diagnostics: {error:?}"
        );
    }
}

fn assert_build_source_file_supports_math_inverse_trig_identity_literals_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "console.log(Math.asin(0)); console.log(Math.acos(1)); console.log(Math.atan(0));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Math.asin/acos/atan identity build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_math_expm1_log1p_and_fround_identity_literals_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "console.log(Math.expm1(0)); console.log(Math.log1p(0)); console.log(Math.fround(0));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Math.expm1/log1p/fround identity build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_math_exp2_non_negative_integer_literals_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const exponent = 2; const alias = exponent; console.log(Math.exp2(alias)); console.log(Math.exp2(0)); console.log(Math.exp2(1)); console.log(Math.exp2(3));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Math.exp2 non-negative integer build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_global_this_math_exp2_non_negative_integer_literals_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const exponent = 2; const alias = exponent; console.log(globalThis.Math.exp2(alias)); console.log(globalThis.Math[\"exp2\"](alias)); console.log(globalThis[\"Math\"].exp2(alias)); console.log(globalThis[\"Math\"][\"exp2\"](alias));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("globalThis.Math exp2 non-negative integer build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_math_exp2_zero_identity_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    assert_build_source_file_supports_math_exp2_non_negative_integer_literals_in_input(
        api_surface,
        extension,
    );
}

fn assert_build_source_file_supports_global_this_math_exp2_zero_identity_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    assert_build_source_file_supports_global_this_math_exp2_non_negative_integer_literals_in_input(
        api_surface,
        extension,
    );
}

fn assert_build_source_file_supports_math_exp_and_log_const_alias_chain_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const zero = 0; const zeroAlias = zero; const one = 1; const oneAlias = one; console.log(Math.exp(zeroAlias)); console.log(Math.log(oneAlias));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Math.exp/log const alias chain build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_math_pow_negative_integer_exponents_for_unit_bases_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const exponent = -3; const alias = exponent; console.log(Math.pow(1, alias)); console.log(Math.pow(-1, alias));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Math.pow negative unit-base build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_fully_bracketed_global_this_math_pow_positive_integer_exponent_alias_chain_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const exponent = 3; const alias = exponent; console.log(globalThis[\"Math\"][\"pow\"](2, alias));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("globalThis[\"Math\"][\"pow\"] alias chain build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn math_pow_browser_alias_inventory_build_source() -> String {
    format!(
        "const exponent = 3; const alias = exponent;\n{}\n",
        math_pow_invocation_lines_for_aliases(
            math_pow_browser_alias_inventory_aliases().as_slice(),
            "2",
            "alias",
            "",
        )
    )
}

fn assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(Math.atan2(zero, one));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Math.atan2 literal build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_fully_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(globalThis[\"Math\"][\"atan2\"](zero, one));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("globalThis[\"Math\"][\"atan2\"] literal build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_const_alias_chain_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const zero = 0; const zeroAlias = zero; const one = 1; const oneAlias = one; console.log(Math.atan2(zeroAlias, oneAlias));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Math.atan2 const alias chain build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_wrapper_literals_in_input(
    api_surface: ApiSurface,
    extension: &str,
    source: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, source).expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Math.atan2 wrapper literal build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_single_quoted_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(globalThis['Math']['atan2'](zero, one));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("globalThis['Math']['atan2'] literal build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_global_this_math_atan2_zero_numerator_and_non_negative_denominator_root_variants_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(globalThis.Math.atan2(zero, one)); console.log(globalThis.Math[\"atan2\"](zero, one)); console.log(globalThis[\"Math\"].atan2(zero, one)); console.log(globalThis[\"Math\"][\"atan2\"](zero, one));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("globalThis.Math atan2 root variants should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_math_expm1_log1p_and_fround_const_alias_chain_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const zero = 0; const alias = zero; console.log(Math.expm1(alias)); console.log(Math.log1p(alias)); console.log(Math.fround(alias));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Math.expm1/log1p/fround const alias chain build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_of_identifier_binding_in_input(extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "let value = 0; for (value of [1, 2]) { console.log(value); }\n",
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
    .expect("identifier-binding for-of lowering should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_array_from_iteration_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    let alias_inventory = array_from_alias_inventory_source();
    let source = format!(
        "const values = [1, 2];\n{}\n{}\n",
        array_from_loop_lines(&alias_inventory, "for (const value of ", "  "),
        array_from_loop_lines(&alias_inventory, "for await (const value of ", "  "),
    );
    fs::write(&source_path, source).expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Array.from iteration should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

// The break/continue probes used to be wrapped in try{...}finally{...};
// try/catch/finally is now rejected fail-closed (E5506, soundness batch 1),
// so the wrappers are gone and the probes' self-check throws — which really
// abort now that `throw` is print-then-trap — verify the counts directly.
fn assert_build_source_file_supports_array_from_new_set_and_new_map_iteration_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "for (const value of Array.from(new Set([1, 2, 1]))) { console.log(value); }\nfor await (const entry of Array.from(new Map([[1, 2], [1, 3], [4, 5]]))) { console.log(entry[0], entry[1]); }\nfor (const frozenValue of Array.from(Object.freeze(new Set([1, 2, 1])))) { console.log(frozenValue); }\nfor await (const frozenEntry of Array.from(Object.freeze(new Map([[1, 2], [1, 3], [4, 5]])))) { console.log(frozenEntry[0], frozenEntry[1]); }\nfor (const nullishValue of Array.from(Object.freeze((null ?? new Set([1, 2, 1]))))) { console.log(nullishValue); }\nfor await (const nullishEntry of Array.from(Object.freeze((null ?? new Map([[1, 2], [1, 3], [4, 5]]))))) { console.log(nullishEntry[0], nullishEntry[1]); }\nfor (const logicalAndValue of Array.from(Object.freeze((true && new Set([1, 2, 1]))))) { console.log(logicalAndValue); }\nfor await (const logicalAndEntry of Array.from(Object.freeze((true && new Map([[1, 2], [1, 3], [4, 5]]))))) { console.log(logicalAndEntry[0], logicalAndEntry[1]); }\nfor (const logicalOrValue of Array.from(Object.freeze((false || new Set([1, 2, 1]))))) { console.log(logicalOrValue); }\nfor await (const logicalOrEntry of Array.from(Object.freeze((false || new Map([[1, 2], [1, 3], [4, 5]]))))) { console.log(logicalOrEntry[0], logicalOrEntry[1]); }\nfor (const aliasedValue of Array.from(new globalThis[\"Set\"]([1, 2, 1]))) { console.log(aliasedValue); }\nfor (const parenthesizedAliasedValue of Array.from(new (globalThis[\"Set\"])([1, 2, 1]))) { console.log(parenthesizedAliasedValue); }\nfor await (const aliasedEntry of Array.from(new globalThis['Map']([[1, 2], [1, 3], [4, 5]]))) { console.log(aliasedEntry[0], aliasedEntry[1]); }\nfor await (const parenthesizedAliasedEntry of Array.from(new (globalThis['Map'])([[1, 2], [1, 3], [4, 5]]))) { console.log(parenthesizedAliasedEntry[0], parenthesizedAliasedEntry[1]); }\nfor (const frozenAliasedValue of Array.from(Object.freeze(new globalThis[\"Set\"]([1, 2, 1])))) { console.log(frozenAliasedValue); }\nfor await (const frozenAliasedEntry of Array.from(Object.freeze(new globalThis['Map']([[1, 2], [1, 3], [4, 5]])))) { console.log(frozenAliasedEntry[0], frozenAliasedEntry[1]); }\nconst setBreakContinueValues = [1, 2, 1];\nlet setBreakContinueCount = 0;\nfor (const value of new Set(setBreakContinueValues)) {\n  if (value === 1) {\n    continue;\n  }\n  setBreakContinueCount += 1;\n  break;\n}\nif (setBreakContinueCount !== 1) {\n  throw new Error(\"unexpected Set constructor break/continue semantics\");\n}\nconst mapBreakContinueValues = [[1, 2], [1, 3], [4, 5]];\nlet mapBreakContinueCount = 0;\nfor (const entry of new Map(mapBreakContinueValues)) {\n  if (entry[0] === 1) {\n    continue;\n  }\n  mapBreakContinueCount += 1;\n  break;\n}\nif (mapBreakContinueCount !== 1) {\n  throw new Error(\"unexpected Map constructor break/continue semantics\");\n}\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("Array.from(new Set/new Map) iteration should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "let value = 0; for ((value) of [1, 2]) { console.log(value); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for-of array iteration with parenthesized binding should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_of_array_iteration_with_sequence_wrappers_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "let value = 0; for ((0, value) of (0, [(0, 1), (0, 2)])) { console.log(value); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for-of array iteration with sequence wrappers should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_check_source_file_supports_set_constructor_iteration_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, set_constructor_iteration_source()).expect("write source");

    check_source_file(&source_path, api_surface, &[], false, false)
        .expect("set constructor iteration should type-check");
}

fn assert_build_source_file_supports_set_constructor_iteration_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, set_constructor_iteration_source()).expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("set constructor iteration should build");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_check_source_file_supports_map_constructor_iteration_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, map_constructor_iteration_source()).expect("write source");

    check_source_file(&source_path, api_surface, &[], false, false)
        .expect("map constructor iteration should type-check");
}

fn assert_build_source_file_supports_map_constructor_iteration_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, map_constructor_iteration_source()).expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("map constructor iteration should build");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_set_and_map_frozen_callable_inventory_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        format!(
            "{} {}",
            set_constructor_frozen_callable_source(),
            map_constructor_frozen_callable_source()
        ),
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("set and map frozen callable inventory should build");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_rejects_for_of_non_literal_iterable_in_input(extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "let values = [1, 2]; values = [3, 4]; for (const item of values) { console.log(item); }\n",
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
    .expect_err("non-literal iterator sources should remain gated");

    assert!(
        error.iter().any(|diagnostic| {
            diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic.message.contains("literal array")
        }),
        "unexpected diagnostics: {:?}",
        error
    );
}

fn array_callback_iteration_sources() -> [&'static str; 8] {
    [
        "const values = [1, 2]; for (const item of values.find((value) => value > 1)) { console.log(item); }\n",
        "const values = [1, 2]; for (const item of values.findIndex((value) => value > 1)) { console.log(item); }\n",
        "const values = [1, 2]; for (const item of values.findLast((value) => value > 1)) { console.log(item); }\n",
        "const values = [1, 2]; for (const item of values.findLastIndex((value) => value > 1)) { console.log(item); }\n",
        "const values = [1, 2]; for (const item of values.some((value) => value > 1)) { console.log(item); }\n",
        "const values = [1, 2]; for (const item of values.every((value) => value > 1)) { console.log(item); }\n",
        "const values = [1, 2]; for (const item of values.reduce((acc, value) => acc + value, 0)) { console.log(item); }\n",
        "const values = [1, 2]; for (const item of values.reduceRight((acc, value) => acc + value, 0)) { console.log(item); }\n",
    ]
}

fn assert_build_source_file_rejects_array_callback_iteration_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    for source in array_callback_iteration_sources() {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, source).expect("write source");

        let error = build_source_file(
            &source_path,
            BuildMode::Fast,
            api_surface,
            false,
            &[],
            16,
            None,
            None,
        )
        .expect_err("array callback-produced iterables should remain gated");

        assert!(
            error.iter().any(|diagnostic| {
                diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                    && (diagnostic
                        .message
                        .contains("array callback-produced iterables")
                        || diagnostic.message.contains("literal array"))
            }),
            "unexpected diagnostics: {:?}",
            error
        );
    }
}

fn assert_check_source_file_rejects_array_callback_iteration_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    for source in array_callback_iteration_sources() {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(&source_path, source).expect("write source");

        let error = check_source_file(&source_path, api_surface, &[], false, false)
            .expect_err("array callback-produced iterables should remain gated");

        assert!(
            error.iter().any(|diagnostic| {
                diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                    && (diagnostic
                        .message
                        .contains("array callback-produced iterables")
                        || diagnostic.message.contains("literal array"))
            }),
            "unexpected diagnostics: {:?}",
            error
        );
    }
}

fn assert_check_source_file_rejects_for_of_object_keys_non_literal_iterable_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "let values = { a: 1 }; values = { a: 2 }; for (const key of Object.keys(values)) { console.log(key); }\n",
    )
    .expect("write source");

    let error = check_source_file(&source_path, api_surface, &[], false, false)
        .expect_err("non-literal Object.keys iterator sources should remain gated");

    assert!(
        error.iter().any(|diagnostic| {
            diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic.message.contains("literal array")
        }),
        "unexpected diagnostics: {:?}",
        error
    );
}

fn assert_check_source_file_supports_for_of_object_keys_const_bound_iterable_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const values = { a: 1 }; for (const key of Object.keys(values)) { console.log(key); }\n",
    )
    .expect("write source");

    check_source_file(&source_path, api_surface, &[], false, false)
        .expect("const-bound Object.keys iterator sources should succeed");
}

fn assert_build_source_file_supports_for_of_object_keys_const_bound_iterable_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const values = { a: 1 }; for (const key of Object.keys(values)) { console.log(key); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("const-bound Object.keys iterator sources should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn object_values_spread_iteration_source() -> &'static str {
    r##"const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);
const frozenFromEntries = Object.freeze(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]));
const bracketedFromEntries = globalThis["Object"]["fromEntries"]([["b", 1], ["a", 2], ["b", 3]]);
const values = Object.values(fromEntries);
const frozenValues = Object.values(frozenFromEntries);
const bracketedFromEntriesValues = Object.values(bracketedFromEntries);
const globalValues = globalThis.Object.values(fromEntries);
const frozenGlobalValues = globalThis.Object.values(frozenFromEntries);
const bracketedGlobalValues = globalThis.Object.values(bracketedFromEntries);
const mixedValues = globalThis.Object["values"](fromEntries);
const frozenMixedValues = globalThis.Object["values"](frozenFromEntries);
const bracketedMixedValues = globalThis.Object["values"](bracketedFromEntries);
const parenthesizedBracketedValues = Object.freeze((globalThis["Object"]).values)(fromEntries);
const parenthesizedSingleQuotedBracketedValues = Object.freeze((globalThis['Object'])["values"])(fromEntries);
const parenthesizedDoubleQuotedBracketedValues = Object.freeze((globalThis["Object"])["values"])(fromEntries);
const frozenParenthesizedDoubleQuotedBracketedValues = Object.freeze((globalThis["Object"])["values"])(frozenFromEntries);
const parenthesizedFrozenBracketedValues = Object.freeze((globalThis["Object"]).values)(frozenFromEntries);
const parenthesizedBracketedBracketedValues = Object.freeze((globalThis["Object"])["values"])(bracketedFromEntries);
const singleQuotedValues = globalThis['Object'].values(fromEntries);
const singleQuotedFrozenValues = globalThis['Object'].values(frozenFromEntries);
const singleQuotedBracketedValues = globalThis['Object']['values'](bracketedFromEntries);
const mixedBracketedValues = globalThis["Object"].values(fromEntries);
const frozenMixedBracketedValues = globalThis["Object"].values(frozenFromEntries);
const bracketedMixedBracketedValues = globalThis["Object"].values(bracketedFromEntries);
const mixedSingleQuotedValues = globalThis["Object"]['values'](fromEntries);
const frozenMixedSingleQuotedValues = globalThis["Object"]['values'](frozenFromEntries);
const bracketedMixedSingleQuotedValues = globalThis["Object"]['values'](bracketedFromEntries);
const parenthesizedMixedSingleQuotedBracketedValues = Object.freeze((globalThis["Object"]['values']))(fromEntries);
const frozenParenthesizedMixedSingleQuotedBracketedValues = Object.freeze((globalThis["Object"]['values']))(frozenFromEntries);
const bracketedValues = globalThis["Object"]["values"](fromEntries);
const frozenBracketedValues = globalThis["Object"]["values"](frozenFromEntries);
const bracketedBracketedValues = globalThis["Object"]["values"](bracketedFromEntries);
const frozenCallableValues = Object.freeze(Object.values)(fromEntries);
const frozenCallableGlobalValues = Object.freeze(globalThis.Object.values)(fromEntries);
const frozenCallableBracketedValues = Object.freeze(globalThis["Object"]["values"])(fromEntries);
for (const item of [...values]) { console.log(item); }
for (const item of [...frozenValues]) { console.log(item); }
for (const item of [...bracketedFromEntriesValues]) { console.log(item); }
for (const item of [...globalValues]) { console.log(item); }
for (const item of [...frozenGlobalValues]) { console.log(item); }
for (const item of [...bracketedGlobalValues]) { console.log(item); }
for (const item of [...mixedValues]) { console.log(item); }
for (const item of [...frozenMixedValues]) { console.log(item); }
for (const item of [...bracketedMixedValues]) { console.log(item); }
for (const item of [...singleQuotedValues]) { console.log(item); }
for (const item of [...singleQuotedFrozenValues]) { console.log(item); }
for (const item of [...singleQuotedBracketedValues]) { console.log(item); }
for (const item of [...mixedBracketedValues]) { console.log(item); }
for (const item of [...frozenMixedBracketedValues]) { console.log(item); }
for (const item of [...bracketedMixedBracketedValues]) { console.log(item); }
for (const item of [...mixedSingleQuotedValues]) { console.log(item); }
for (const item of [...frozenMixedSingleQuotedValues]) { console.log(item); }
for (const item of [...bracketedMixedSingleQuotedValues]) { console.log(item); }
for (const item of [...parenthesizedMixedSingleQuotedBracketedValues]) { console.log(item); }
for (const item of [...frozenParenthesizedMixedSingleQuotedBracketedValues]) { console.log(item); }
for (const item of [...parenthesizedBracketedValues]) { console.log(item); }
for (const item of [...parenthesizedSingleQuotedBracketedValues]) { console.log(item); }
for (const item of [...parenthesizedDoubleQuotedBracketedValues]) { console.log(item); }
for (const item of [...frozenParenthesizedDoubleQuotedBracketedValues]) { console.log(item); }
for (const item of [...parenthesizedFrozenBracketedValues]) { console.log(item); }
for (const item of [...parenthesizedBracketedBracketedValues]) { console.log(item); }
for (const item of [...bracketedValues]) { console.log(item); }
for (const item of [...frozenBracketedValues]) { console.log(item); }
for (const item of [...bracketedBracketedValues]) { console.log(item); }
for (const item of [...frozenCallableValues]) { console.log(item); }
for (const item of [...frozenCallableGlobalValues]) { console.log(item); }
for (const item of [...frozenCallableBracketedValues]) { console.log(item); }
const asyncFromEntries = Object.fromEntries([["c", 4], ["d", 5], ["c", 6]]);
const frozenAsyncFromEntries = Object.freeze(Object.fromEntries([["c", 4], ["d", 5], ["c", 6]]));
const asyncBracketedFromEntries = globalThis["Object"]["fromEntries"]([["c", 4], ["d", 5], ["c", 6]]);
const asyncValues = Object.values(asyncFromEntries);
const frozenAsyncValues = Object.values(frozenAsyncFromEntries);
const asyncBracketedFromEntriesValues = Object.values(asyncBracketedFromEntries);
const asyncGlobalValues = globalThis.Object.values(asyncFromEntries);
const frozenAsyncGlobalValues = globalThis.Object.values(frozenAsyncFromEntries);
const asyncBracketedGlobalValues = globalThis.Object.values(asyncBracketedFromEntries);
const asyncMixedValues = globalThis.Object["values"](asyncFromEntries);
const frozenAsyncMixedValues = globalThis.Object["values"](frozenAsyncFromEntries);
const asyncBracketedMixedValues = globalThis.Object["values"](asyncBracketedFromEntries);
const asyncMixedSingleQuotedValues = globalThis["Object"]['values'](asyncFromEntries);
const frozenAsyncMixedSingleQuotedValues = globalThis["Object"]['values'](frozenAsyncFromEntries);
const asyncBracketedMixedSingleQuotedValues = globalThis["Object"]['values'](asyncBracketedFromEntries);
const asyncParenthesizedMixedSingleQuotedBracketedValues = Object.freeze((globalThis["Object"]['values']))(asyncFromEntries);
const frozenAsyncParenthesizedMixedSingleQuotedBracketedValues = Object.freeze((globalThis["Object"]['values']))(frozenAsyncFromEntries);
const asyncParenthesizedBracketedValues = Object.freeze((globalThis["Object"]).values)(asyncFromEntries);
const asyncParenthesizedSingleQuotedBracketedValues = Object.freeze((globalThis['Object'])["values"])(asyncFromEntries);
const asyncParenthesizedDoubleQuotedBracketedValues = Object.freeze((globalThis["Object"])["values"])(asyncFromEntries);
const frozenAsyncParenthesizedDoubleQuotedBracketedValues = Object.freeze((globalThis["Object"])["values"])(frozenAsyncFromEntries);
const frozenAsyncParenthesizedBracketedValues = Object.freeze((globalThis["Object"]).values)(frozenAsyncFromEntries);
const asyncParenthesizedBracketedBracketedValues = Object.freeze((globalThis["Object"])["values"])(asyncBracketedFromEntries);
const asyncSingleQuotedValues = globalThis['Object'].values(asyncFromEntries);
const frozenAsyncSingleQuotedValues = globalThis['Object'].values(frozenAsyncFromEntries);
const asyncSingleQuotedBracketedValues = globalThis['Object']['values'](asyncBracketedFromEntries);
const asyncMixedBracketedValues = globalThis["Object"].values(asyncFromEntries);
const frozenAsyncMixedBracketedValues = globalThis["Object"].values(frozenAsyncFromEntries);
const asyncBracketedMixedBracketedValues = globalThis["Object"].values(asyncBracketedFromEntries);
const asyncBracketedValues = globalThis["Object"]["values"](asyncFromEntries);
const frozenAsyncBracketedValues = globalThis["Object"]["values"](frozenAsyncFromEntries);
const asyncBracketedBracketedValues = globalThis["Object"]["values"](asyncBracketedFromEntries);
for await (const item of [...asyncValues]) { console.log(item); }
for await (const item of [...frozenAsyncValues]) { console.log(item); }
for await (const item of [...asyncBracketedFromEntriesValues]) { console.log(item); }
for await (const item of [...asyncGlobalValues]) { console.log(item); }
for await (const item of [...frozenAsyncGlobalValues]) { console.log(item); }
for await (const item of [...asyncBracketedGlobalValues]) { console.log(item); }
for await (const item of [...asyncMixedValues]) { console.log(item); }
for await (const item of [...frozenAsyncMixedValues]) { console.log(item); }
for await (const item of [...asyncBracketedMixedValues]) { console.log(item); }
for await (const item of [...asyncMixedSingleQuotedValues]) { console.log(item); }
for await (const item of [...frozenAsyncMixedSingleQuotedValues]) { console.log(item); }
for await (const item of [...asyncBracketedMixedSingleQuotedValues]) { console.log(item); }
for await (const item of [...asyncParenthesizedMixedSingleQuotedBracketedValues]) { console.log(item); }
for await (const item of [...frozenAsyncParenthesizedMixedSingleQuotedBracketedValues]) { console.log(item); }
for await (const item of [...asyncSingleQuotedValues]) { console.log(item); }
for await (const item of [...frozenAsyncSingleQuotedValues]) { console.log(item); }
for await (const item of [...asyncSingleQuotedBracketedValues]) { console.log(item); }
for await (const item of [...asyncMixedBracketedValues]) { console.log(item); }
for await (const item of [...frozenAsyncMixedBracketedValues]) { console.log(item); }
for await (const item of [...asyncBracketedMixedBracketedValues]) { console.log(item); }
for await (const item of [...asyncParenthesizedBracketedValues]) { console.log(item); }
for await (const item of [...asyncParenthesizedSingleQuotedBracketedValues]) { console.log(item); }
for await (const item of [...asyncParenthesizedDoubleQuotedBracketedValues]) { console.log(item); }
for await (const item of [...frozenAsyncParenthesizedDoubleQuotedBracketedValues]) { console.log(item); }
for await (const item of [...frozenAsyncParenthesizedBracketedValues]) { console.log(item); }
for await (const item of [...asyncParenthesizedBracketedBracketedValues]) { console.log(item); }
for await (const item of [...asyncBracketedValues]) { console.log(item); }
for await (const item of [...frozenAsyncBracketedValues]) { console.log(item); }
for await (const item of [...asyncBracketedBracketedValues]) { console.log(item); }
"##
}

fn object_helper_nullish_logical_iteration_source() -> &'static str {
    r##"const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);
const nullishKeys = Object.freeze((null ?? Object.keys))(fromEntries);
const logicalKeys = Object.freeze((true && Object.keys))(Object.freeze(fromEntries));
const logicalOrKeys = Object.freeze((false || Object.keys))(fromEntries);
const parenthesizedSingleQuotedBracketedKeys = Object.freeze((globalThis['Object'])['keys'])(fromEntries);
const parenthesizedDoubleQuotedBracketedKeys = Object.freeze((globalThis["Object"])["keys"])(fromEntries);
const logicalValues = Object.freeze((true && Object.values))(Object.freeze(fromEntries));
const logicalEntries = Object.freeze((false || Object.entries))(fromEntries);
const parenthesizedSingleQuotedBracketedEntries = Object.freeze((globalThis['Object'])["entries"])(fromEntries);
const parenthesizedDoubleQuotedBracketedEntries = Object.freeze((globalThis["Object"])["entries"])(fromEntries);
const frozenParenthesizedDoubleQuotedBracketedEntries = Object.freeze((globalThis["Object"])["entries"])(Object.freeze(fromEntries));
const frozenNullishKeys = Object.freeze((null ?? Object.keys))(Object.freeze(fromEntries));
const frozenLogicalValues = Object.freeze((true && Object.values))(fromEntries);
const frozenLogicalEntries = Object.freeze((false || Object.entries))(Object.freeze(fromEntries));
const frozenParenthesizedSingleQuotedBracketedEntries = Object.freeze((globalThis['Object'])["entries"])(Object.freeze(fromEntries));
for (const key of [...nullishKeys]) { console.log(key); }
for (const key of [...logicalKeys]) { console.log(key); }
for (const key of [...logicalOrKeys]) { console.log(key); }
for (const key of [...parenthesizedSingleQuotedBracketedKeys]) { console.log(key); }
for (const key of [...parenthesizedDoubleQuotedBracketedKeys]) { console.log(key); }
for await (const value of [...logicalValues]) { console.log(value); }
for (const entry of [...logicalEntries]) { console.log(entry[0]); console.log(entry[1]); }
for (const entry of [...parenthesizedSingleQuotedBracketedEntries]) { console.log(entry[0]); console.log(entry[1]); }
for (const entry of [...parenthesizedDoubleQuotedBracketedEntries]) { console.log(entry[0]); console.log(entry[1]); }
for (const entry of [...frozenParenthesizedDoubleQuotedBracketedEntries]) { console.log(entry[0]); console.log(entry[1]); }
for (const key of [...frozenNullishKeys]) { console.log(key); }
for await (const value of [...frozenLogicalValues]) { console.log(value); }
for (const entry of [...frozenLogicalEntries]) { console.log(entry[0]); console.log(entry[1]); }
for (const entry of [...frozenParenthesizedSingleQuotedBracketedEntries]) { console.log(entry[0]); console.log(entry[1]); }
"##
}

fn reflect_own_keys_spread_iteration_source() -> &'static str {
    r##"const object = { "b": 1, "2": 2, "a": 3, "1": 4 };
const alias = object;
const frozenObject = Object.freeze(object);
const frozenAlias = Object.freeze(alias);
const keys = Reflect.ownKeys(object);
const aliasKeys = Reflect.ownKeys(alias);
const frozenKeys = Reflect.ownKeys(frozenObject);
const frozenAliasKeys = Reflect.ownKeys(frozenAlias);
const globalKeys = globalThis.Reflect.ownKeys(object);
const bracketedRootKeys = globalThis["Reflect"].ownKeys(object);
const mixedKeys = globalThis.Reflect["ownKeys"](alias);
const bracketedKeys = globalThis["Reflect"]["ownKeys"](object);
const frozenBracketedKeys = globalThis['Reflect']['ownKeys'](frozenObject);
const mixedBracketedRootKeys = Object.freeze(globalThis["Reflect"]['ownKeys'])(object);
const mixedSingleQuotedRootKeys = Object.freeze(globalThis['Reflect']["ownKeys"])(object);
const parenthesizedMixedBracketedRootKeys = Object.freeze((globalThis["Reflect"]['ownKeys']))(object);
const parenthesizedMixedSingleQuotedRootKeys = Object.freeze((globalThis['Reflect']["ownKeys"]))(object);
const frozenParenthesizedMixedRootKeys = Object.freeze((globalThis["Reflect"]).ownKeys)(object);
const frozenParenthesizedBracketedRootKeys = Object.freeze((globalThis["Reflect"])["ownKeys"])(object);
const frozenParenthesizedSingleQuotedRootKeys = Object.freeze((globalThis['Reflect']).ownKeys)(object);
const frozenParenthesizedSingleQuotedBracketedKeys = Object.freeze((globalThis['Reflect'])['ownKeys'])(object);
const frozenCallableKeys = Object.freeze(Reflect.ownKeys)(object);
const frozenCallableParenKeys = Object.freeze((Reflect.ownKeys))(object);
const frozenCallableGlobalKeys = Object.freeze(globalThis.Reflect.ownKeys)(object);
const frozenCallableBracketedKeys = Object.freeze(globalThis['Reflect']['ownKeys'])(object);
const nullishKeys = Object.freeze((null ?? Reflect.ownKeys))(object);
const logicalAndKeys = Object.freeze((true && Reflect.ownKeys))(frozenObject);
const logicalOrKeys = Object.freeze((false || Reflect.ownKeys))(alias);
for (const item of [...keys]) { console.log(item); }
for (const item of [...aliasKeys]) { console.log(item); }
for (const item of [...frozenKeys]) { console.log(item); }
for (const item of [...frozenAliasKeys]) { console.log(item); }
for (const item of [...globalKeys]) { console.log(item); }
for (const item of [...bracketedRootKeys]) { console.log(item); }
for (const item of [...mixedKeys]) { console.log(item); }
for (const item of [...bracketedKeys]) { console.log(item); }
for (const item of [...frozenBracketedKeys]) { console.log(item); }
for (const item of [...frozenCallableKeys]) { console.log(item); }
for (const item of [...frozenCallableParenKeys]) { console.log(item); }
for (const item of [...frozenCallableGlobalKeys]) { console.log(item); }
for (const item of [...frozenCallableBracketedKeys]) { console.log(item); }
for (const item of [...nullishKeys]) { console.log(item); }
for (const item of [...logicalAndKeys]) { console.log(item); }
for (const item of [...logicalOrKeys]) { console.log(item); }
for await (const item of keys) { console.log(item); }
for await (const item of aliasKeys) { console.log(item); }
for await (const item of frozenKeys) { console.log(item); }
for await (const item of frozenAliasKeys) { console.log(item); }
for await (const item of globalKeys) { console.log(item); }
for await (const item of mixedKeys) { console.log(item); }
for await (const item of bracketedKeys) { console.log(item); }
for await (const item of frozenBracketedKeys) { console.log(item); }
"##
}

fn reflect_own_keys_direct_iteration_source() -> &'static str {
    r##"const object = { "b": 1, "2": 2, "a": 3, "1": 4 };
const alias = object;
const frozenObject = Object.freeze(object);
const frozenAlias = Object.freeze(alias);
const keys = Reflect.ownKeys(object);
const aliasKeys = Reflect.ownKeys(alias);
const frozenKeys = Reflect.ownKeys(frozenObject);
const frozenAliasKeys = Reflect.ownKeys(frozenAlias);
const globalKeys = globalThis.Reflect.ownKeys(object);
const bracketedRootKeys = globalThis["Reflect"].ownKeys(object);
const mixedKeys = globalThis.Reflect["ownKeys"](alias);
const bracketedKeys = globalThis["Reflect"]["ownKeys"](object);
const frozenBracketedKeys = globalThis['Reflect']['ownKeys'](frozenObject);
const mixedBracketedRootKeys = Object.freeze(globalThis["Reflect"]['ownKeys'])(object);
const mixedSingleQuotedRootKeys = Object.freeze(globalThis['Reflect']["ownKeys"])(object);
const parenthesizedMixedBracketedRootKeys = Object.freeze((globalThis["Reflect"]['ownKeys']))(object);
const parenthesizedMixedSingleQuotedRootKeys = Object.freeze((globalThis['Reflect']["ownKeys"]))(object);
const frozenParenthesizedMixedRootKeys = Object.freeze((globalThis["Reflect"]).ownKeys)(object);
const frozenParenthesizedBracketedRootKeys = Object.freeze((globalThis["Reflect"])["ownKeys"])(object);
const frozenParenthesizedSingleQuotedRootKeys = Object.freeze((globalThis['Reflect']).ownKeys)(object);
const frozenParenthesizedSingleQuotedBracketedKeys = Object.freeze((globalThis['Reflect'])['ownKeys'])(object);
const frozenCallableKeys = Object.freeze(Reflect.ownKeys)(object);
const frozenCallableParenKeys = Object.freeze((Reflect.ownKeys))(object);
const frozenCallableGlobalKeys = Object.freeze(globalThis.Reflect.ownKeys)(object);
const frozenCallableBracketedKeys = Object.freeze(globalThis['Reflect']['ownKeys'])(object);
const nullishKeys = Object.freeze((null ?? Reflect.ownKeys))(object);
const logicalAndKeys = Object.freeze((true && Reflect.ownKeys))(frozenObject);
const logicalOrKeys = Object.freeze((false || Reflect.ownKeys))(alias);
for (const item of keys) { console.log(item); }
for (const item of aliasKeys) { console.log(item); }
for (const item of frozenKeys) { console.log(item); }
for (const item of frozenAliasKeys) { console.log(item); }
for (const item of globalKeys) { console.log(item); }
for (const item of bracketedRootKeys) { console.log(item); }
for (const item of mixedKeys) { console.log(item); }
for (const item of bracketedKeys) { console.log(item); }
for (const item of frozenBracketedKeys) { console.log(item); }
for (const item of frozenCallableKeys) { console.log(item); }
for (const item of frozenCallableParenKeys) { console.log(item); }
for (const item of frozenCallableGlobalKeys) { console.log(item); }
for (const item of frozenCallableBracketedKeys) { console.log(item); }
for (const item of nullishKeys) { console.log(item); }
for (const item of logicalAndKeys) { console.log(item); }
for (const item of logicalOrKeys) { console.log(item); }
for await (const item of keys) { console.log(item); }
for await (const item of aliasKeys) { console.log(item); }
for await (const item of frozenKeys) { console.log(item); }
for await (const item of frozenAliasKeys) { console.log(item); }
for await (const item of globalKeys) { console.log(item); }
for await (const item of mixedKeys) { console.log(item); }
for await (const item of bracketedKeys) { console.log(item); }
for await (const item of frozenBracketedKeys) { console.log(item); }
for await (const item of frozenCallableKeys) { console.log(item); }
for await (const item of frozenCallableParenKeys) { console.log(item); }
for await (const item of frozenCallableGlobalKeys) { console.log(item); }
for await (const item of frozenCallableBracketedKeys) { console.log(item); }
"##
}

fn assert_check_source_file_supports_spread_of_object_values_iterator_slices_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, object_values_spread_iteration_source()).expect("write source");

    check_source_file(&source_path, api_surface, &[], false, false)
        .expect("spread of object.values iterator slices should succeed");
}

fn assert_build_source_file_supports_spread_of_object_values_iterator_slices_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, object_values_spread_iteration_source()).expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("spread of object.values iterator slices should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_check_source_file_supports_object_helper_nullish_logical_iterator_slices_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        object_helper_nullish_logical_iteration_source(),
    )
    .expect("write source");

    check_source_file(&source_path, api_surface, &[], false, false)
        .expect("spread of nullish/logical object helper iterator slices should succeed");
}

fn assert_build_source_file_supports_object_helper_nullish_logical_iterator_slices_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        object_helper_nullish_logical_iteration_source(),
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("spread of nullish/logical object helper iterator slices should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_check_source_file_supports_spread_of_object_keys_and_entries_iterator_slices_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const frozenFromEntries = Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])); const frozenAsyncFromEntries = Object.freeze(Object.fromEntries([[\"c\", 4], [\"d\", 5], [\"c\", 6]])); const bracketedKeys = Object[\"keys\"](Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])); const bracketedEntries = globalThis[\"Object\"][\"entries\"](Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])); for (const key of [...globalThis.Object[\"keys\"](Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))]) { console.log(key); } for (const key of [...globalThis[\"Object\"].keys(frozenFromEntries)]) { console.log(key); } for (const key of [...bracketedKeys]) { console.log(key); } for (const entry of [...globalThis.Object[\"entries\"](Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))]) { console.log(entry[0]); console.log(entry[1]); } for (const entry of [...globalThis[\"Object\"].entries(frozenFromEntries)]) { console.log(entry[0]); console.log(entry[1]); } for (const entry of [...bracketedEntries]) { console.log(entry[0]); console.log(entry[1]); } for await (const key of [...globalThis.Object[\"keys\"](Object.fromEntries([[\"c\", 4], [\"d\", 5], [\"c\", 6]]))]) { console.log(key); } for await (const key of [...globalThis[\"Object\"].keys(frozenAsyncFromEntries)]) { console.log(key); } for await (const key of [...bracketedKeys]) { console.log(key); } for await (const entry of [...globalThis.Object[\"entries\"](Object.fromEntries([[\"c\", 4], [\"d\", 5], [\"c\", 6]]))]) { console.log(entry[0]); console.log(entry[1]); } for await (const entry of [...globalThis[\"Object\"].entries(frozenAsyncFromEntries)]) { console.log(entry[0]); console.log(entry[1]); } for await (const entry of [...bracketedEntries]) { console.log(entry[0]); console.log(entry[1]); }\n",
    )
    .expect("write source");

    check_source_file(&source_path, api_surface, &[], false, false)
        .expect("spread of object.keys/object.entries iterator slices should succeed");
}

fn assert_check_source_file_supports_spread_of_reflect_own_keys_iterator_slices_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, reflect_own_keys_spread_iteration_source()).expect("write source");

    check_source_file(&source_path, api_surface, &[], false, false)
        .expect("spread of Reflect.ownKeys iterator slices should succeed");
}

fn assert_build_source_file_supports_spread_of_reflect_own_keys_iterator_slices_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, reflect_own_keys_spread_iteration_source()).expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("spread of Reflect.ownKeys iterator slices should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_check_source_file_supports_for_of_reflect_own_keys_const_bound_iterable_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, reflect_own_keys_direct_iteration_source()).expect("write source");

    check_source_file(&source_path, api_surface, &[], false, false)
        .expect("const-bound Reflect.ownKeys iterator sources should succeed");
}

fn assert_build_source_file_supports_for_of_reflect_own_keys_const_bound_iterable_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, reflect_own_keys_direct_iteration_source()).expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("const-bound Reflect.ownKeys iterator sources should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_spread_of_object_keys_and_entries_iterator_slices_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const frozenFromEntries = Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])); const frozenAsyncFromEntries = Object.freeze(Object.fromEntries([[\"c\", 4], [\"d\", 5], [\"c\", 6]])); const bracketedKeys = Object[\"keys\"](Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])); const bracketedEntries = globalThis[\"Object\"][\"entries\"](Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]])); for (const key of [...globalThis.Object[\"keys\"](Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))]) { console.log(key); } for (const key of [...globalThis[\"Object\"].keys(frozenFromEntries)]) { console.log(key); } for (const key of [...bracketedKeys]) { console.log(key); } for (const entry of [...globalThis.Object[\"entries\"](Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))]) { console.log(entry[0]); console.log(entry[1]); } for (const entry of [...globalThis[\"Object\"].entries(frozenFromEntries)]) { console.log(entry[0]); console.log(entry[1]); } for (const entry of [...bracketedEntries]) { console.log(entry[0]); console.log(entry[1]); } for await (const key of [...globalThis.Object[\"keys\"](Object.fromEntries([[\"c\", 4], [\"d\", 5], [\"c\", 6]]))]) { console.log(key); } for await (const key of [...globalThis[\"Object\"].keys(frozenAsyncFromEntries)]) { console.log(key); } for await (const key of [...bracketedKeys]) { console.log(key); } for await (const entry of [...globalThis.Object[\"entries\"](Object.fromEntries([[\"c\", 4], [\"d\", 5], [\"c\", 6]]))]) { console.log(entry[0]); console.log(entry[1]); } for await (const entry of [...globalThis[\"Object\"].entries(frozenAsyncFromEntries)]) { console.log(entry[0]); console.log(entry[1]); } for await (const entry of [...bracketedEntries]) { console.log(entry[0]); console.log(entry[1]); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("spread of object.keys/object.entries iterator slices should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_await_array_iteration_with_sequence_wrappers_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "let value = 0; for await ((0, value) of (0, [(0, 1), (0, 2)])) { console.log(value); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for await array iteration with sequence wrappers should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_await_array_iteration_with_await_wrapper_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "for await (const value of await [1, 2]) { console.log(value); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for await array iteration with await wrapper should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_rejects_for_await_non_literal_iterable_in_input(extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "let values = [1, 2]; values = [3, 4]; for await (const item of values) { console.log(item); }\n",
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
    .expect_err("non-literal async iterator sources should remain gated");

    assert!(
        error.iter().any(|diagnostic| {
            diagnostic.code == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)
                && diagnostic.message.contains("literal array")
        }),
        "unexpected diagnostics: {:?}",
        error
    );
}

fn assert_build_source_file_supports_for_of_array_iteration_with_const_alias_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const value = 1; const alias = value; for (const item of [alias]) { console.log(item); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for-of array iteration with const alias should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_of_array_iteration_with_const_alias_chain_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const values = [1, 2]; const alias = values; for (const item of alias) { console.log(item); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for-of array iteration with const alias chain should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_of_array_iteration_with_let_binding_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "let values = [1, 2]; for (const item of values) { console.log(item); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for-of array iteration with let binding should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_of_array_iteration_with_const_string_alias_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const value = \"hello\"; const alias = value; for (const item of [alias]) { console.log(item); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for-of array iteration with const string alias should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_of_string_concatenation_iteration_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const prefix = \"he\"; const suffix = \"llo\"; for (const ch of prefix + suffix) { console.log(ch); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for-of string concatenation iteration should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_of_template_literal_string_iteration_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "for (const ch of `hello`) { console.log(ch); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for-of template literal string iteration should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_of_array_iteration_with_const_boolean_alias_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const value = true; const alias = value; for (const item of [alias]) { console.log(item); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for-of array iteration with const boolean alias should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const values = [1, 2]; for (const item of [...(values)]) { console.log(item); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for-of array iteration with spread of const-bound literal arrays should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const value = 2; const values = ([1, (value)]); for (const item of (values)) { console.log(item); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for-of array iteration with parenthesized const alias should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_of_array_iteration_with_satisfies_wrapper_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const value = 2; for (const item of ([1, (value)] satisfies readonly [1, 2])) { console.log(item); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for-of array iteration with satisfies wrapper should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_of_array_iteration_with_as_const_wrapper_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const value = 2; for (const item of ([1, (value)] as const)) { console.log(item); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for-of array iteration with as const wrapper should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_const_alias_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const value = 2; const values = ([1, (value)]); for await (const item of (values)) { console.log(item); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for await array iteration with parenthesized const alias should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_await_string_concatenation_iteration_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const prefix = \"he\"; const suffix = \"llo\"; for await (const ch of prefix + suffix) { console.log(ch); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for await string concatenation iteration should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_await_array_iteration_with_spread_of_const_bound_literal_arrays_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const values = [1, 2]; for await (const item of [...(values)]) { console.log(item); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for await array iteration with spread of const-bound literal arrays should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "let value = 0; for await ((value) of [1, 2]) { console.log(value); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for await array iteration with parenthesized binding should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_await_array_iteration_with_satisfies_wrapper_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const value = 2; for await (const item of ([1, (value)] satisfies readonly [1, 2])) { console.log(item); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for await array iteration with satisfies wrapper should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_await_array_iteration_with_as_const_wrapper_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const value = 2; for await (const item of ([1, (value)] as const)) { console.log(item); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for await array iteration with as const wrapper should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_rejects_generator_lowering_in_input(extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, "function* main() { yield* []; }\nmain();\n").expect("write source");

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
    .expect_err("generator lowering should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(
            |diagnostic| diagnostic.message.contains("generator function lowering")
                || diagnostic.message.contains("yield expressions")
        ),
        "unexpected diagnostics: {error:?}"
    );
}

fn assert_build_source_file_rejects_async_generator_lowering_in_input(extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "async function* main() { yield 1; }\nmain();\n",
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
    .expect_err("async generator lowering should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| diagnostic
            .message
            .contains("async-generator function lowering")
            || diagnostic.message.contains("yield expressions")),
        "unexpected diagnostics: {error:?}"
    );
}

fn assert_build_source_file_rejects_generator_lowering_for_source_in_input(
    extension: &str,
    source: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, source).expect("write source");

    let expected_messages: &[&str] = if source.contains("yield*") {
        &["yield* delegation"]
    } else if source.contains("async function*") {
        &[
            "async-generator function lowering is unavailable in the current phase",
            "generator function lowering is unavailable in the current phase",
        ]
    } else {
        &["generator function lowering is unavailable in the current phase"]
    };

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
    .expect_err("generator lowering should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| expected_messages
            .iter()
            .any(|expected| diagnostic.message.contains(expected))),
        "unexpected diagnostics: {error:?}"
    );
}

fn assert_build_source_file_supports_async_class_method_in_input(
    api_surface: ApiSurface,
    bundle: bool,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "class Example { async main() { return 1; } }\nnew Example().main();\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        bundle,
        &[],
        16,
        None,
        None,
    )
    .expect("async class method lowering should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_runtime_entrypoint_rejects_async_class_expression_in_input(
    extension: &str,
    source: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, source).expect("write source");

    let error = reject_async_and_generator_class_methods_in_runtime_entrypoint(&source_path)
        .expect_err("async class method lowering should fail in the direct runtime path");
    assert!(
        error.iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected an E5506 diagnostic: {error:?}"
    );
    assert!(
        error.iter().any(|diagnostic| diagnostic
            .message
            .contains("async class method lowering is unavailable in the direct runtime path")),
        "unexpected diagnostics: {error:?}"
    );
}

fn assert_runtime_entrypoint_rejects_generator_class_expression_in_input(
    extension: &str,
    source: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, source).expect("write source");

    let expected_message = if source.contains("async *") {
        "async-generator class method lowering is unavailable in the direct runtime path"
    } else {
        "generator class method lowering is unavailable in the direct runtime path"
    };

    let error = reject_async_and_generator_class_methods_in_runtime_entrypoint(&source_path)
        .expect_err("generator class method lowering should fail in the direct runtime path");
    assert!(
        error.iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected an E5506 diagnostic: {error:?}"
    );
    assert!(
        error
            .iter()
            .any(|diagnostic| diagnostic.message.contains(expected_message)),
        "unexpected diagnostics: {error:?}"
    );
}

fn assert_runtime_entrypoint_rejects_mixed_generator_class_expression_in_input(
    extension: &str,
    source: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, source).expect("write source");

    let error = reject_async_and_generator_class_methods_in_runtime_entrypoint(&source_path)
        .expect_err("generator class method lowering should fail in the direct runtime path");
    assert!(
        error.iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected an E5506 diagnostic: {error:?}"
    );
    assert!(
        error.iter().any(|diagnostic| diagnostic.message.contains(
            "generator and async-generator class method lowering is unavailable in the direct runtime path for yield* delegation"
        )),
        "unexpected diagnostics: {error:?}"
    );
}

fn assert_runtime_entrypoint_rejects_generator_function_expression_in_input(
    extension: &str,
    source: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, source).expect("write source");

    let expected_messages: &[&str] = if source.contains("yield*") {
        &["yield* delegation"]
    } else if source.contains("async function*") {
        &[
            "async-generator function lowering is unavailable in the current phase",
            "generator function lowering is unavailable in the current phase",
        ]
    } else {
        &["generator function lowering is unavailable in the current phase"]
    };

    let error = reject_async_and_generator_class_methods_in_runtime_entrypoint(&source_path)
        .expect_err("generator function lowering should fail in the direct runtime path");
    assert!(
        error.iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected an E5506 diagnostic: {error:?}"
    );
    assert!(
        error.iter().any(|diagnostic| expected_messages
            .iter()
            .any(|expected| diagnostic.message.contains(expected))),
        "unexpected diagnostics: {error:?}"
    );
}

fn assert_runtime_entrypoint_rejects_generator_function_declaration_in_input(
    extension: &str,
    source: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, source).expect("write source");

    let expected_messages: &[&str] = if source.contains("yield*") {
        &["yield* delegation"]
    } else if source.contains("async function*") {
        &[
            "async-generator function lowering is unavailable in the current phase",
            "generator function lowering is unavailable in the current phase",
        ]
    } else {
        &["generator function lowering is unavailable in the current phase"]
    };

    let error = reject_async_and_generator_class_methods_in_runtime_entrypoint(&source_path)
        .expect_err("generator function lowering should fail in the direct runtime path");
    assert!(
        error.iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "expected an E5506 diagnostic: {error:?}"
    );
    assert!(
        error.iter().any(|diagnostic| expected_messages
            .iter()
            .any(|expected| diagnostic.message.contains(expected))),
        "unexpected diagnostics: {error:?}"
    );
}

fn assert_check_source_file_rejects_generator_lowering_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, "function* main() { yield* []; }\nmain();\n").expect("write source");

    let error = check_source_file(&source_path, api_surface, &[], false, false)
        .expect_err("generator lowering should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(
            |diagnostic| diagnostic.message.contains("generator function lowering")
                || diagnostic.message.contains("yield expressions")
        ),
        "unexpected diagnostics: {error:?}"
    );
}

fn assert_check_source_file_rejects_async_generator_lowering_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "async function* main() { yield 1; }\nmain();\n",
    )
    .expect("write source");

    let error = check_source_file(&source_path, api_surface, &[], false, false)
        .expect_err("async generator lowering should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(
            |diagnostic| diagnostic.message.contains("generator function lowering")
                || diagnostic.message.contains("yield expressions")
        ),
        "unexpected diagnostics: {error:?}"
    );
}

fn assert_check_source_file_rejects_class_generator_methods_in_input(api_surface: ApiSurface) {
    for extension in ["ts", "js", "jsx", "tsx"] {
        for source in [
            "class Example { *main() { yield 1; } }\nnew Example();\n",
            "class Example { async *main() { yield 1; } }\nnew Example();\n",
            "const Example = class NamedExample { *main() { yield 1; } };\n",
            "const Example = class NamedExample { async *main() { yield 1; } };\nnew Example();\n",
            "export default (class NamedExample { *main() { yield 1; } });\n",
            "export default (class NamedExample { async *main() { yield 1; } });\n",
            "const Example = ((0, class NamedExample { *main() { yield 1; } }));\nnew Example();\n",
            "const Example = ((0, class NamedExample { async *main() { yield 1; } }));\nnew Example();\n",
            "class Example { *main() { yield* []; } }\nnew Example();\n",
            "class Example { async *main() { yield* []; } }\nnew Example();\n",
        ] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join(format!("main.{extension}"));
            fs::write(&source_path, source).expect("write source");

            let expected_message = match (source.contains("async *"), source.contains("yield*")) {
                (true, true) => {
                    "async-generator class method lowering is unavailable in the direct runtime path for yield* delegation"
                }
                (true, false) => {
                    "async-generator class method lowering is unavailable in the direct runtime path"
                }
                (false, true) => {
                    "generator class method lowering is unavailable in the direct runtime path for yield* delegation"
                }
                (false, false) => {
                    "generator class method lowering is unavailable in the direct runtime path"
                }
            };

            let error = check_source_file(&source_path, api_surface, &[], false, false)
                .expect_err("class generator method lowering should fail");

            assert!(error.iter().any(|diagnostic| diagnostic.code
                == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
            assert!(error
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected_message)));
        }
    }
}

fn assert_build_source_file_rejects_generator_lowering_in_browser_input(extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, "function* main() { yield* []; }\nmain();\n").expect("write source");

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
    .expect_err("generator lowering should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error
            .iter()
            .any(|diagnostic| diagnostic.message.contains("yield* delegation")),
        "unexpected diagnostics: {error:?}"
    );
}

fn assert_build_source_file_rejects_async_generator_lowering_in_browser_input(extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "async function* main() { yield 1; }\nmain();\n",
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
    .expect_err("async generator lowering should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(
            |diagnostic| diagnostic.message.contains("generator function lowering")
                || diagnostic.message.contains("yield expressions")
        ),
        "unexpected diagnostics: {error:?}"
    );
}

fn assert_build_source_file_rejects_class_generator_methods_in_input(api_surface: ApiSurface) {
    for extension in ["ts", "js", "jsx", "tsx"] {
        for source in [
            "class Example { *main() { yield 1; } }\nnew Example();\n",
            "class Example { async *main() { yield 1; } }\nnew Example();\n",
            "const Example = class NamedExample { *main() { yield 1; } };\n",
            "const Example = class NamedExample { async *main() { yield 1; } };\nnew Example();\n",
            "export default (class NamedExample { *main() { yield 1; } });\n",
            "export default (class NamedExample { async *main() { yield 1; } });\n",
            "const Example = ((0, class NamedExample { *main() { yield 1; } }));\nnew Example();\n",
            "const Example = ((0, class NamedExample { async *main() { yield 1; } }));\nnew Example();\n",
            "class Example { *main() { yield* []; } }\nnew Example();\n",
            "class Example { async *main() { yield* []; } }\nnew Example();\n",
        ] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join(format!("main.{extension}"));
            fs::write(&source_path, source).expect("write source");

            let expected_message = match (source.contains("async *"), source.contains("yield*")) {
                (true, true) => {
                    "async-generator class method lowering is unavailable in the direct runtime path for yield* delegation"
                }
                (true, false) => {
                    "async-generator class method lowering is unavailable in the direct runtime path"
                }
                (false, true) => {
                    "generator class method lowering is unavailable in the direct runtime path for yield* delegation"
                }
                (false, false) => {
                    "generator class method lowering is unavailable in the direct runtime path"
                }
            };

            let error = build_source_file(
                &source_path,
                BuildMode::Fast,
                api_surface,
                false,
                &[],
                16,
                None,
                None,
            )
            .expect_err("class generator method lowering should fail");

            assert!(error.iter().any(|diagnostic| diagnostic.code
                == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
            assert!(error
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected_message)));
        }
    }
}

fn assert_build_source_file_supports_for_await_array_iteration_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "for await (const value of [1, 2]) { console.log(value); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for await array iteration should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_await_array_iteration_with_const_alias_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const value = 1; const alias = value; for await (const item of [alias]) { console.log(item); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for await array iteration with const alias should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_await_array_iteration_with_const_alias_chain_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const values = [1, 2]; const alias = values; for await (const item of alias) { console.log(item); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for await array iteration with const alias chain should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_await_array_iteration_with_const_string_alias_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const value = \"hello\"; const alias = value; for await (const item of [alias]) { console.log(item); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for await array iteration with const string alias should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_supports_for_await_array_iteration_with_const_boolean_alias_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const value = true; const alias = value; for await (const item of [alias]) { console.log(item); }\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("for await array iteration with const boolean alias should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

fn assert_build_source_file_rejects_process_env_mutation_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        kali_common::late_process_env_mutation_source(),
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("process env mutation should fail");

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
                        .contains(r#"globalThis["process"]["env"]["KALI_BROWSER_ENV_MUTATION"]"#))
        }),
        "unexpected diagnostics: {error:?}"
    );
}

fn assert_build_source_file_rejects_bracketed_process_control_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        r#"globalThis["process"]["pid"]; globalThis["process"]["cwd"]; globalThis["process"]["chdir"]; globalThis["process"]["exit"];"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("bracketed process control should fail");

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

fn assert_build_source_file_rejects_process_kill_zero_probe_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, kali_common::process_kill_zero_probe_source()).expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("process kill zero probe should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    for expected in [
        "process.kill",
        "process.kill((0))",
        r#"process["kill"]"#,
        r#"process["kill"]((0))"#,
        "globalThis.process.kill",
        "globalThis.process.kill((0))",
        r#"globalThis.process["kill"]"#,
        r#"globalThis.process["kill"]((0))"#,
        r#"globalThis["process"].kill"#,
        r#"globalThis["process"].kill((0))"#,
        r#"globalThis["process"]["kill"]"#,
        r#"globalThis["process"]["kill"]((0))"#,
    ] {
        assert!(
            error
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "missing {expected} in {error:?}"
        );
    }
}

fn assert_build_source_file_rejects_bracketed_deno_network_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        r#"globalThis["Deno"]["connect"]; globalThis["Deno"]["listen"]; globalThis["Deno"]["serve"];"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("bracketed Deno network APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    for expected in [
        r#"globalThis["Deno"]["connect"]"#,
        r#"globalThis["Deno"]["listen"]"#,
        r#"globalThis["Deno"]["serve"]"#,
    ] {
        assert!(
            error
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "missing {expected} in {error:?}"
        );
    }
}

fn assert_build_source_file_rejects_broader_intl_apis_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        r#"globalThis['Intl']['DateTimeFormat']; globalThis["Intl"]["DateTimeFormat"]; globalThis['Intl']['RelativeTimeFormat']; globalThis["Intl"]["RelativeTimeFormat"]; globalThis['Intl']['PluralRules']; globalThis["Intl"]["PluralRules"]; globalThis['Intl']['Collator']; globalThis["Intl"]["Collator"]; globalThis['Intl']['DisplayNames']; globalThis["Intl"]["DisplayNames"]; globalThis['Intl']['Segmenter']; globalThis["Intl"]["Segmenter"]; globalThis['Intl']['Locale']; globalThis["Intl"]["Locale"]; globalThis['Intl']['NumberFormat']; globalThis["Intl"]["NumberFormat"]; Intl.NumberFormat; Intl.RelativeTimeFormat; Intl.Collator; Intl.DisplayNames; Intl.Segmenter; Intl.Locale;"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
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
                        .contains(r#"globalThis[\"Intl\"][\"NumberFormat\"]"#)
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

fn assert_build_source_file_rejects_late_weak_reference_apis_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, late_threaded_runtime_source()).expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
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

fn assert_build_source_file_rejects_threaded_runtime_globals_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        r#"globalThis.SharedArrayBuffer; globalThis["SharedArrayBuffer"]; globalThis.Atomics; globalThis["Atomics"];"#,
    )
    .expect("write source");

    let error = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect_err("threaded runtime globals should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| {
            diagnostic.message.contains("SharedArrayBuffer")
                || diagnostic.message.contains("Atomics")
                || diagnostic.message.contains("threaded runtime globals")
        }),
        "unexpected diagnostics: {error:?}"
    );
}

fn assert_build_source_file_supports_exponent_assignment_on_mutable_binding_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "let value = 2; ((value)) **= 3; console.log(value);",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        api_surface,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("build should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[path = "build_tests/supports_math.rs"]
mod supports_math;

#[path = "build_tests/supports_for.rs"]
mod supports_for;

#[path = "build_tests/supports_misc.rs"]
mod supports_misc;

#[path = "build_tests/rejects.rs"]
mod rejects;

#[path = "build_tests/check.rs"]
mod check;

#[path = "build_tests/collect.rs"]
mod collect;

#[path = "build_tests/validate.rs"]
mod validate;

#[path = "build_tests/runtime.rs"]
mod runtime;

#[path = "build_tests/discover.rs"]
mod discover;

#[path = "build_tests/misc.rs"]
mod misc;
