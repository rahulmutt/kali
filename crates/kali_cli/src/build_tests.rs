use super::*;
use kali_optimize::{ProfileData, ProfileSample, ProfileSampleKind};
use sha2::{Digest, Sha256};
use std::fs;
use tempfile::tempdir;
use wasmparser::Validator;

#[test]
fn build_source_file_writes_valid_wasm_artifact() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "function add(a, b) { return a + b; } add(1, 2);",
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

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_writes_valid_wasm_artifact_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "function add(a, b) { return a + b; } add(1, 2);",
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

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_deno_env_get_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log(Deno.env.get('KALI_ENV_GET_SMOKE'));",
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

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_deno_env_get_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Deno.env.get('KALI_ENV_GET_SMOKE'));",
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

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_bracketed_deno_env_get_in_ts_input_direct() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log(Deno[\"env\"][\"get\"]('KALI_ENV_GET_SMOKE'));",
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

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_bracketed_deno_env_get_in_js_input_direct() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Deno[\"env\"][\"get\"]('KALI_ENV_GET_SMOKE'));",
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

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_bracketed_deno_env_get_in_js_input_global_this() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(globalThis[\"Deno\"][\"env\"][\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis.Deno[\"env\"][\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis.Deno.env[\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis[\"Deno\"].env[\"get\"]('KALI_ENV_GET_SMOKE'));",
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

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_bracketed_deno_env_get_in_ts_input_global_this() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log(globalThis[\"Deno\"][\"env\"][\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis.Deno[\"env\"][\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis.Deno.env[\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis[\"Deno\"].env[\"get\"]('KALI_ENV_GET_SMOKE'));",
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

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_permission_query_const_bindings_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"const read_descriptor = "read";
const write_descriptor = "write";
const env_descriptor = "env";
const net_descriptor = "net";
Deno.permissions.query({ name: read_descriptor });
Deno.permissions["query"]({ name: read_descriptor });
Deno["permissions"]["query"]({ name: read_descriptor });
globalThis.Deno.permissions.query({ name: read_descriptor });
globalThis.Deno.permissions["query"]({ name: read_descriptor });
globalThis["Deno"]["permissions"]["query"]({ name: read_descriptor });
globalThis["Deno"]["permissions"].query({ name: write_descriptor });
globalThis["Deno"]["permissions"]["query"]({ name: write_descriptor });
globalThis.Deno.permissions.query({ name: write_descriptor });
globalThis.Deno.permissions["query"]({ name: write_descriptor });
Deno.permissions.query({ name: env_descriptor });
Deno.permissions["query"]({ name: env_descriptor });
Deno["permissions"]["query"]({ name: env_descriptor });
globalThis["Deno"]["permissions"].query({ name: net_descriptor });
globalThis["Deno"]["permissions"]["query"]({ name: net_descriptor });
globalThis.Deno.permissions.query({ name: net_descriptor });
globalThis.Deno.permissions["query"]({ name: net_descriptor });
"#,
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

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_permission_query_const_bindings_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"const read_descriptor = "read";
const write_descriptor = "write";
const env_descriptor = "env";
const net_descriptor = "net";
Deno.permissions.query({ name: read_descriptor });
Deno.permissions["query"]({ name: read_descriptor });
Deno["permissions"]["query"]({ name: read_descriptor });
globalThis.Deno.permissions.query({ name: read_descriptor });
globalThis.Deno.permissions["query"]({ name: read_descriptor });
globalThis["Deno"]["permissions"]["query"]({ name: read_descriptor });
globalThis["Deno"]["permissions"].query({ name: write_descriptor });
globalThis["Deno"]["permissions"]["query"]({ name: write_descriptor });
globalThis.Deno.permissions.query({ name: write_descriptor });
globalThis.Deno.permissions["query"]({ name: write_descriptor });
Deno.permissions.query({ name: env_descriptor });
Deno.permissions["query"]({ name: env_descriptor });
Deno["permissions"]["query"]({ name: env_descriptor });
globalThis["Deno"]["permissions"].query({ name: net_descriptor });
globalThis["Deno"]["permissions"]["query"]({ name: net_descriptor });
globalThis.Deno.permissions.query({ name: net_descriptor });
globalThis.Deno.permissions["query"]({ name: net_descriptor });
"#,
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

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_rejects_unsupported_permission_query_descriptors_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
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

#[test]
fn build_source_file_rejects_unsupported_permission_query_descriptors_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
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

#[test]
fn build_source_file_rejects_bracketed_proxy_revocable_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, r#"globalThis["Proxy"]["revocable"]({}, {});"#).expect("write source");

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
            || diagnostic.message.contains("Proxy.revocable")),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_bracketed_object_has_own_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, r#"globalThis["Object"]["hasOwn"]({}, "a");"#).expect("write source");

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
            .contains(r#"globalThis["Object"]["hasOwn"]"#)
            || diagnostic.message.contains("Object.hasOwn")),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_bracketed_object_has_own_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, r#"globalThis["Object"]["hasOwn"]({}, "a");"#).expect("write source");

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
            .contains(r#"globalThis["Object"]["hasOwn"]"#)
            || diagnostic.message.contains("Object.hasOwn")),
        "unexpected diagnostics: {error:?}"
    );
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
            .contains(r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]"#)
            || diagnostic
                .message
                .contains("Object.prototype.hasOwnProperty.call")),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_promise_all_settled_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, "console.log(Promise.allSettled([1, 2]));\n").expect("write source");

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
    .expect_err("Promise.allSettled should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Promise.allSettled")),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_generator_functions_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
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

#[test]
fn build_source_file_rejects_generator_functions_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
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
fn build_source_file_rejects_process_env_mutation_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"process.env = {}; globalThis.process.env = {}; process["env"] = {}; globalThis.process["env"] = {}; globalThis["process"].env = {}; globalThis["process"]["env"] = {};"#,
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
    .expect_err("process env mutation should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("environment mutation API 'process.env'")
                && (diagnostic.message.contains("process.env")
                    || diagnostic.message.contains(r#"process["env"]"#))
        }),
        "unexpected diagnostics: {error:?}"
    );
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
fn build_source_file_rejects_mixed_bracket_dot_permission_escalation_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
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
fn build_source_file_rejects_deno_env_to_object_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"Deno.env.toObject; globalThis.Deno.env.toObject; globalThis.Deno.env["toObject"]; Deno.env["toObject"]; Deno["env"]["toObject"]; globalThis.Deno["env"]["toObject"]; globalThis.Deno["env"]["toObject"]; globalThis["Deno"].env["toObject"]; globalThis["Deno"]["env"]["toObject"];"#,
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
    .expect_err("env materialization APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| diagnostic
            .message
            .contains("environment snapshot materialization API")
            && (diagnostic.message.contains("Deno.env.toObject")
                || diagnostic.message.contains("globalThis.Deno.env.toObject")
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
fn build_source_file_rejects_deno_env_to_object_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"Deno.env.toObject; globalThis.Deno.env.toObject; globalThis.Deno.env["toObject"]; Deno.env["toObject"]; Deno["env"]["toObject"]; globalThis.Deno["env"]["toObject"]; globalThis.Deno["env"]["toObject"]; globalThis["Deno"].env["toObject"]; globalThis["Deno"]["env"]["toObject"];"#,
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
    .expect_err("env materialization APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| diagnostic
            .message
            .contains("environment snapshot materialization API")
            && (diagnostic.message.contains("Deno.env.toObject")
                || diagnostic.message.contains("globalThis.Deno.env.toObject")
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
        r#"globalThis["Intl"]["DateTimeFormat"]; globalThis["Intl"]["RelativeTimeFormat"]; globalThis["Intl"]["PluralRules"]; globalThis["Intl"]["Collator"]; globalThis["Intl"]["DisplayNames"]; globalThis["Intl"]["Locale"]; Intl.RelativeTimeFormat; Intl.Collator; Intl.DisplayNames; Intl.Locale;"#,
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
        r#"globalThis["Intl"]["DateTimeFormat"]; globalThis["Intl"]["RelativeTimeFormat"]; globalThis["Intl"]["PluralRules"]; globalThis["Intl"]["Collator"]; globalThis["Intl"]["DisplayNames"]; globalThis["Intl"]["Locale"]; Intl.RelativeTimeFormat; Intl.Collator; Intl.DisplayNames; Intl.Locale;"#,
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
                        .contains(r#"globalThis["Intl"]["Locale"]"#))
        }),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_late_weak_reference_apis_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"new WeakMap(); globalThis.WeakMap; globalThis["WeakMap"]; new WeakSet(); globalThis.WeakSet; globalThis["WeakSet"]; globalThis.WeakRef; globalThis["WeakRef"]; new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry; globalThis["FinalizationRegistry"];"#,
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
    .expect_err("late weak-reference APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| {
            diagnostic.message.contains("WeakMap")
                || diagnostic.message.contains("WeakSet")
                || diagnostic.message.contains("WeakRef")
                || diagnostic.message.contains("FinalizationRegistry")
        }),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_late_weak_reference_apis_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"new WeakMap(); globalThis.WeakMap; globalThis["WeakMap"]; new WeakSet(); globalThis.WeakSet; globalThis["WeakSet"]; globalThis.WeakRef; globalThis["WeakRef"]; new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry; globalThis["FinalizationRegistry"];"#,
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
    .expect_err("late weak-reference APIs should fail");

    assert!(error.iter().any(|diagnostic| diagnostic.code
        == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)));
    assert!(
        error.iter().any(|diagnostic| {
            diagnostic.message.contains("WeakMap")
                || diagnostic.message.contains("WeakSet")
                || diagnostic.message.contains("WeakRef")
                || diagnostic.message.contains("FinalizationRegistry")
        }),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn build_source_file_rejects_threaded_runtime_globals_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"globalThis.SharedArrayBuffer; globalThis["SharedArrayBuffer"]; globalThis.Atomics; globalThis["Atomics"];"#,
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

#[test]
fn build_source_file_rejects_threaded_runtime_globals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"globalThis.SharedArrayBuffer; globalThis["SharedArrayBuffer"]; globalThis.Atomics; globalThis["Atomics"];"#,
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

#[test]
fn compile_source_file_uses_incremental_cache_on_repeat_builds() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).expect("write manifest");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");

    let first = compile_source_file_with_cache_state(
        &source_path,
        BuildMode::Release,
        16,
        ApiSurface::Deno,
        &[],
        false,
        false,
    )
    .expect("first compile");
    assert!(!first.cache_hit);
    let first_cache_path = first
        .cache_path
        .as_ref()
        .expect("cache path should be recorded for project-root builds");
    assert!(
        first_cache_path.exists(),
        "cache path should be written on first build"
    );

    let second = compile_source_file_with_cache_state(
        &source_path,
        BuildMode::Release,
        16,
        ApiSurface::Deno,
        &[],
        false,
        false,
    )
    .expect("second compile");
    assert!(second.cache_hit);
    assert_eq!(first.wasm_bytes, second.wasm_bytes);
    assert_eq!(first.cache_path, second.cache_path);
}

#[test]
fn compile_source_file_invalidates_incremental_cache_when_source_changes() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).expect("write manifest");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write initial source");

    let first = compile_source_file_with_cache_state(
        &source_path,
        BuildMode::Release,
        16,
        ApiSurface::Deno,
        &[],
        false,
        false,
    )
    .expect("first compile");
    let first_cache_path = first
        .cache_path
        .clone()
        .expect("cache path should be recorded for project-root builds");

    fs::write(&source_path, "console.log(2);").expect("rewrite source");

    let second = compile_source_file_with_cache_state(
        &source_path,
        BuildMode::Release,
        16,
        ApiSurface::Deno,
        &[],
        false,
        false,
    )
    .expect("second compile after source change");

    assert!(
        !second.cache_hit,
        "source edits must invalidate the incremental cache"
    );
    assert_ne!(
        first_cache_path.as_path(),
        second
            .cache_path
            .as_ref()
            .expect("cache path should still be recorded after source changes")
            .as_path(),
        "source hash should be part of the cache key"
    );
    assert_ne!(
        first.wasm_bytes, second.wasm_bytes,
        "changing the source should produce a distinct artifact"
    );
}

#[test]
fn compile_source_file_with_cache_state_rejects_invalid_runtime_profiles() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).expect("write manifest");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");

    let error = compile_source_file_with_cache_state(
        &source_path,
        BuildMode::Release,
        16,
        ApiSurface::Deno,
        &["wasm-threads".to_string(), "wasm-threads".to_string()],
        false,
        false,
    )
    .expect_err("invalid runtime profiles should fail");

    assert!(error
        .iter()
        .any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::INVALID_CONFIG as u32)));
}

#[test]
fn incremental_cache_path_includes_runtime_profiles() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).expect("write manifest");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");

    let base = incremental_cache_path(
        &source_path,
        BuildMode::Release,
        16,
        ApiSurface::Deno,
        &[],
        None,
        false,
        false,
    )
    .expect("base cache path")
    .expect("base cache path should exist");
    let normalized = incremental_cache_path(
        &source_path,
        BuildMode::Release,
        16,
        ApiSurface::Deno,
        &[" wasm-threads ".to_string(), "wasm-threads".to_string()],
        None,
        false,
        false,
    )
    .expect("normalized cache path")
    .expect("normalized cache path should exist");
    let canonical = incremental_cache_path(
        &source_path,
        BuildMode::Release,
        16,
        ApiSurface::Deno,
        &["wasm-threads".to_string()],
        None,
        false,
        false,
    )
    .expect("canonical cache path")
    .expect("canonical cache path should exist");

    assert_ne!(base, normalized);
    assert_eq!(normalized, canonical);
}

#[test]
fn incremental_cache_path_separates_build_modes() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).expect("write manifest");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");

    let fast = incremental_cache_path(
        &source_path,
        BuildMode::Fast,
        16,
        ApiSurface::Deno,
        &[],
        None,
        false,
        false,
    )
    .expect("fast cache path")
    .expect("fast cache path should exist");
    let release = incremental_cache_path(
        &source_path,
        BuildMode::Release,
        16,
        ApiSurface::Deno,
        &[],
        None,
        false,
        false,
    )
    .expect("release cache path")
    .expect("release cache path should exist");
    let advanced = incremental_cache_path(
        &source_path,
        BuildMode::ReleaseAdvanced,
        16,
        ApiSurface::Deno,
        &[],
        None,
        false,
        false,
    )
    .expect("release-advanced cache path")
    .expect("release-advanced cache path should exist");

    assert_ne!(fast, release);
    assert_ne!(fast, advanced);
    assert_ne!(release, advanced);
}

#[test]
fn incremental_cache_path_separates_specialization_budgets() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).expect("write manifest");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "console.log(1);").expect("write source");

    let narrow = incremental_cache_path(
        &source_path,
        BuildMode::Release,
        8,
        ApiSurface::Deno,
        &[],
        None,
        false,
        false,
    )
    .expect("narrow cache path")
    .expect("narrow cache path should exist");
    let wide = incremental_cache_path(
        &source_path,
        BuildMode::Release,
        32,
        ApiSurface::Deno,
        &[],
        None,
        false,
        false,
    )
    .expect("wide cache path")
    .expect("wide cache path should exist");

    assert_ne!(narrow, wide);
}

#[test]
fn load_profile_data_file_validates_version_and_normalizes_samples() {
    let dir = tempdir().expect("tempdir");
    let profile_path = dir.path().join("profile.json");
    fs::write(
        &profile_path,
        r#"{"version":1,"samples":[{"kind":"function","key":" hot-path ","weight":2},{"kind":"function","key":"hot-path","weight":3}]}"#,
    )
    .expect("write profile");

    let profile = load_profile_data_file(&profile_path).expect("profile data");
    assert!(profile.is_current_version());
    assert_eq!(
        profile.samples,
        vec![ProfileSample::new(
            ProfileSampleKind::Function,
            "hot-path",
            5
        )]
    );

    fs::write(&profile_path, r#"{"version":2,"samples":[]}"#).expect("rewrite profile");
    let error = load_profile_data_file(&profile_path).expect_err("version mismatch should fail");
    assert!(error
        .iter()
        .any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::INVALID_CONFIG as u32)));

    fs::write(
        &profile_path,
        r#"{"version":1,"samples":[],"unexpected":true}"#,
    )
    .expect("rewrite profile with unknown field");
    let error = load_profile_data_file(&profile_path).expect_err("unknown fields should fail");
    assert!(error
        .iter()
        .any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::INVALID_CONFIG as u32)));
}

#[test]
fn compile_source_file_with_profile_data_uses_profile_specific_cache_key() {
    let dir = tempdir().expect("tempdir");
    fs::write(dir.path().join("kali.json"), r#"{"schemaVersion":1}"#).expect("write manifest");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "function hot_add(a, b) { return a + b; } hot_add(1, 2);",
    )
    .expect("write source");

    let hot_profile = ProfileData::new(vec![ProfileSample::new(
        ProfileSampleKind::Function,
        "hot_add",
        8,
    )]);

    let cold = compile_source_file_with_cache_state_and_profile_data(
        &source_path,
        BuildMode::Release,
        16,
        ApiSurface::Deno,
        None,
        &[],
        false,
        false,
    )
    .expect("cold compile");
    let hot = compile_source_file_with_cache_state_and_profile_data(
        &source_path,
        BuildMode::Release,
        16,
        ApiSurface::Deno,
        Some(&hot_profile),
        &[],
        false,
        false,
    )
    .expect("hot compile");

    assert_ne!(cold.cache_path, hot.cache_path);
    assert_eq!(cold.wasm_bytes, hot.wasm_bytes);
}

#[test]
fn build_artifact_metadata_preserves_runtime_profiles() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const main = 1;").expect("write source");

    let runtime_profiles = vec!["wasm-threads".to_string()];
    let metadata = build_artifact_metadata(
        &source_path,
        "executable",
        BuildMode::Fast,
        "deno",
        &runtime_profiles,
        16,
        None,
        None,
    )
    .expect("build metadata");

    assert_eq!(metadata.runtime_profiles, runtime_profiles);
    assert_eq!(metadata.max_specializations, 16);
}

#[test]
fn build_artifact_metadata_serializes_runtime_provenance_fields() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const main = 1;").expect("write source");

    let metadata = build_artifact_metadata(
        &source_path,
        "component",
        BuildMode::ReleaseAdvanced,
        "browser",
        &["wasm-threads".to_string()],
        24,
        None,
        None,
    )
    .expect("build metadata");

    let json: serde_json::Value = serde_json::from_slice(&serialize_artifact_metadata(&metadata))
        .expect("serialize metadata");

    assert_eq!(json["runtimeProfiles"], serde_json::json!(["wasm-threads"]));
    assert_eq!(json["maxSpecializations"], 24);
    assert_eq!(json["hostContract"], "kali-hosted");
    assert_eq!(json["runtimeBackend"], "wasmtime");
    assert!(json.get("profileDataHash").is_none());
}

#[test]
fn build_artifact_metadata_round_trips_through_schema_validation() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const main = 1;").expect("write source");

    let metadata = build_artifact_metadata(
        &source_path,
        "component",
        BuildMode::ReleaseAdvanced,
        "browser",
        &["wasm-threads".to_string()],
        24,
        None,
        Some(vec![LibraryExport {
            name: "main".to_string(),
            signature: "(input) => number".to_string(),
        }]),
    )
    .expect("build metadata");

    let value = serde_json::to_value(&metadata).expect("serialize metadata");
    validate_artifact_metadata_value(&value).expect("metadata should satisfy schema validation");
}

#[test]
fn build_browser_bundle_result_round_trips_through_schema_validation() {
    let value = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "artifacts": [
            { "kind": "wasm-module", "path": "browser.wasm" },
            { "kind": "js-glue", "path": "browser.js" },
            { "kind": "source-map", "path": "browser.js.map" }
        ],
        "exports": [],
        "bundleFormat": "esm"
    });

    validate_build_result_value(&value).expect("browser bundle result should validate");
}

#[test]
fn validate_artifact_metadata_value_rejects_invalid_export_shape() {
    let invalid_metadata = serde_json::json!({
        "schemaVersion": 1,
        "artifactKind": "component",
        "entrypoint": "src/main.ts",
        "buildMode": "release",
        "apiSurface": "browser",
        "runtimeProfiles": ["wasm-threads"],
        "maxSpecializations": 24,
        "hostContract": "kali-hosted",
        "runtimeBackend": "wasmtime",
        "kaliVersion": "1.2.3",
        "sourceHash": "sha256-deadbeef",
        "exports": [
            {"name": "main", "signature": "(input) => number", "extra": true}
        ]
    });

    let err = validate_artifact_metadata_value(&invalid_metadata)
        .expect_err("extra export keys should fail validation");
    assert!(err.contains("exports[0]"), "unexpected error: {err}");
}

#[test]
fn validate_build_result_value_rejects_invalid_bundle_format() {
    let invalid_bundle = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "artifacts": [
            { "kind": "wasm-module", "path": "browser.wasm" },
            { "kind": "js-glue", "path": "browser.js" }
        ],
        "exports": [],
        "bundleFormat": "umd"
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("unsupported bundleFormat should fail validation");
    assert!(err.contains("bundleFormat"), "unexpected error: {err}");
}

#[test]
fn build_artifact_metadata_records_profile_data_hash() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const main = 1;").expect("write source");

    let profile_data = ProfileData::new(vec![
        ProfileSample::new(ProfileSampleKind::Function, "hot", 4),
        ProfileSample::new(ProfileSampleKind::Branch, "branch:hot", 3),
    ]);
    let expected_hash = {
        let normalized = profile_data.clone().normalized();
        let profile_json = serde_json::to_vec(&normalized).expect("serialize profile data");
        format!("sha256-{:x}", Sha256::digest(profile_json))
    };

    let metadata = build_artifact_metadata(
        &source_path,
        "component",
        BuildMode::Release,
        "deno",
        &[],
        16,
        Some(&profile_data),
        None,
    )
    .expect("build metadata");

    assert_eq!(
        metadata.profile_data_hash.as_deref(),
        Some(expected_hash.as_str())
    );

    let json: serde_json::Value = serde_json::from_slice(&serialize_artifact_metadata(&metadata))
        .expect("serialize metadata");
    assert_eq!(json["profileDataHash"], expected_hash);
}

#[test]
fn build_artifact_metadata_normalizes_equivalent_profile_data_hashes() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const main = 1;").expect("write source");

    let equivalent_profiles = [
        ProfileData::new(vec![
            ProfileSample::new(ProfileSampleKind::Function, " hot-path ", 2),
            ProfileSample::new(ProfileSampleKind::Branch, "branch:hot", 3),
            ProfileSample::new(ProfileSampleKind::Function, "hot-path", 4),
        ]),
        ProfileData::new(vec![
            ProfileSample::new(ProfileSampleKind::Branch, "branch:hot", 3),
            ProfileSample::new(ProfileSampleKind::Function, "hot-path", 6),
        ]),
    ];

    let expected_hash = {
        let normalized = equivalent_profiles[0].clone().normalized();
        let profile_json = serde_json::to_vec(&normalized).expect("serialize profile data");
        format!("sha256-{:x}", Sha256::digest(profile_json))
    };

    let hashes: Vec<_> = equivalent_profiles
        .iter()
        .map(|profile_data| {
            let metadata = build_artifact_metadata(
                &source_path,
                "component",
                BuildMode::Release,
                "deno",
                &[],
                16,
                Some(profile_data),
                None,
            )
            .expect("build metadata");

            metadata.profile_data_hash.expect("profile data hash")
        })
        .collect();

    assert_eq!(hashes, vec![expected_hash.clone(), expected_hash.clone()]);
}

#[test]
fn build_artifact_metadata_rejects_duplicate_runtime_profiles() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const main = 1;").expect("write source");

    let runtime_profiles = vec!["wasm-threads".to_string(), "wasm-threads".to_string()];
    let error = build_artifact_metadata(
        &source_path,
        "executable",
        BuildMode::Fast,
        "deno",
        &runtime_profiles,
        16,
        None,
        None,
    )
    .expect_err("duplicate runtime profiles should fail");

    assert!(error
        .iter()
        .any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::INVALID_CONFIG as u32)));
}

#[test]
fn build_artifact_metadata_rejects_unknown_runtime_profiles() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const main = 1;").expect("write source");

    let runtime_profiles = vec!["fiber-threads".to_string()];
    let error = build_artifact_metadata(
        &source_path,
        "executable",
        BuildMode::Fast,
        "deno",
        &runtime_profiles,
        16,
        None,
        None,
    )
    .expect_err("unknown runtime profiles should fail");

    assert!(error
        .iter()
        .any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::INVALID_CONFIG as u32)));
}

#[test]
fn output_path_uses_source_stem() {
    let source = PathBuf::from("/tmp/demo/main.ts");
    let output = executable_output_path_for(&source, Some(Path::new("dist")));
    assert_eq!(output, PathBuf::from("dist/main.wasm"));
}

#[test]
fn capi_binding_package_manifest_path_uses_source_stem() {
    let source = PathBuf::from("/tmp/demo/main.ts");
    let output = binding_package_manifest_output_path_for(&source, Some(Path::new("dist")));
    assert_eq!(output, PathBuf::from("dist/main.binding-package.json"));
}

#[test]
fn component_output_paths_use_source_stem_and_binding_manifest() {
    let source = PathBuf::from("/tmp/demo/main.ts");
    let (wasm, wit, meta, binding_package) =
        component_output_paths_for(&source, Some(Path::new("dist")));
    assert_eq!(wasm, PathBuf::from("dist/main.component.wasm"));
    assert_eq!(wit, PathBuf::from("dist/main.wit"));
    assert_eq!(meta, PathBuf::from("dist/main.component.meta.json"));
    assert_eq!(
        binding_package,
        PathBuf::from("dist/main.binding-package.json")
    );
}

#[test]
fn discover_dynamic_import_targets_ignores_comment_and_string_substrings() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    let ghost_path = dir.path().join("ghost.ts");
    let lazy_path = dir.path().join("lazy.ts");
    fs::write(&ghost_path, "export const ghost = true;").expect("write ghost chunk");
    fs::write(&lazy_path, "export const lazy = true;").expect("write lazy chunk");
    fs::write(
        &source_path,
        "const comment = \"import('./ghost.ts')\";\n/* import('./ghost.ts') */\nconst lazy = import('./lazy.ts');\n",
    )
    .expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy.ts");
    assert_eq!(
        targets[0].target,
        lazy_path.canonicalize().expect("canonical lazy path")
    );
}

#[test]
fn discover_dynamic_import_targets_resolves_directory_index_chunks() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).expect("create lazy dir");
    fs::write(lazy_dir.join("index.ts"), "export const lazy = true;").expect("write lazy index");
    fs::write(&source_path, "const lazy = import('./lazy');").expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy");
    assert_eq!(
        targets[0].target,
        lazy_dir
            .join("index.ts")
            .canonicalize()
            .expect("canonical lazy index path")
    );
}

#[test]
fn discover_dynamic_import_targets_resolves_literal_dynamic_import_chunks_in_js_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    let lazy_path = dir.path().join("lazy.js");
    fs::write(&lazy_path, "export const lazy = true;").expect("write lazy chunk");
    fs::write(&source_path, "const lazy = import('./lazy.js');").expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy.js");
    assert_eq!(
        targets[0].target,
        lazy_path.canonicalize().expect("canonical lazy path")
    );
}

#[test]
fn discover_dynamic_import_targets_resolves_directory_index_chunks_in_js_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).expect("create lazy dir");
    fs::write(lazy_dir.join("index.js"), "export const lazy = true;").expect("write lazy index");
    fs::write(&source_path, "const lazy = import('./lazy');").expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy");
    assert_eq!(
        targets[0].target,
        lazy_dir
            .join("index.js")
            .canonicalize()
            .expect("canonical lazy index path")
    );
}

#[test]
fn discover_dynamic_import_targets_resolves_directory_index_chunks_in_jsx_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.jsx");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).expect("create lazy dir");
    fs::write(lazy_dir.join("index.jsx"), "export const lazy = true;").expect("write lazy index");
    fs::write(&source_path, "const lazy = import('./lazy');").expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy");
    assert_eq!(
        targets[0].target,
        lazy_dir
            .join("index.jsx")
            .canonicalize()
            .expect("canonical lazy index path")
    );
}

#[test]
fn discover_dynamic_import_targets_resolves_directory_index_chunks_in_tsx_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.tsx");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).expect("create lazy dir");
    fs::write(lazy_dir.join("index.tsx"), "export const lazy = true;").expect("write lazy index");
    fs::write(&source_path, "const lazy = import('./lazy');").expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy");
    assert_eq!(
        targets[0].target,
        lazy_dir
            .join("index.tsx")
            .canonicalize()
            .expect("canonical lazy index path")
    );
}

#[test]
fn discover_dynamic_import_targets_resolves_parenthesized_dynamic_import_targets_in_js_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).expect("create lazy dir");
    fs::write(lazy_dir.join("index.js"), "export const lazy = true;").expect("write lazy index");
    fs::write(
        &source_path,
        "const name = 'lazy'; const root = './'; const lazy = import((root + name));",
    )
    .expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy");
    assert_eq!(
        targets[0].target,
        lazy_dir
            .join("index.js")
            .canonicalize()
            .expect("canonical lazy index path")
    );
}

#[test]
fn discover_dynamic_import_targets_resolves_parenthesized_dynamic_import_targets_in_jsx_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.jsx");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).expect("create lazy dir");
    fs::write(lazy_dir.join("index.jsx"), "export const lazy = true;").expect("write lazy index");
    fs::write(
        &source_path,
        "const name = 'lazy'; const root = './'; const lazy = import((root + name));",
    )
    .expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy");
    assert_eq!(
        targets[0].target,
        lazy_dir
            .join("index.jsx")
            .canonicalize()
            .expect("canonical lazy index path")
    );
}

#[test]
fn discover_dynamic_import_targets_resolves_parenthesized_dynamic_import_targets_in_tsx_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.tsx");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).expect("create lazy dir");
    fs::write(lazy_dir.join("index.tsx"), "export const lazy = true;").expect("write lazy index");
    fs::write(
        &source_path,
        "const name = 'lazy'; const root = './'; const lazy = import((root + name));",
    )
    .expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy");
    assert_eq!(
        targets[0].target,
        lazy_dir
            .join("index.tsx")
            .canonicalize()
            .expect("canonical lazy index path")
    );
}

#[test]
fn discover_dynamic_import_targets_ignores_comment_and_string_substrings_in_js_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.js");
    let ghost_path = dir.path().join("ghost.js");
    let lazy_path = dir.path().join("lazy.js");
    fs::write(&ghost_path, "export const ghost = true;").expect("write ghost chunk");
    fs::write(&lazy_path, "export const lazy = true;").expect("write lazy chunk");
    fs::write(
        &source_path,
        "const comment = \"import('./ghost.js')\";\n/* import('./ghost.js') */\nconst lazy = import('./lazy.js');\n",
    )
    .expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy.js");
    assert_eq!(
        targets[0].target,
        lazy_path.canonicalize().expect("canonical lazy path")
    );
}

#[test]
fn discover_dynamic_import_targets_ignores_comment_and_string_substrings_in_jsx_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.jsx");
    let ghost_path = dir.path().join("ghost.jsx");
    let lazy_path = dir.path().join("lazy.jsx");
    fs::write(&ghost_path, "export const ghost = true;").expect("write ghost chunk");
    fs::write(&lazy_path, "export const lazy = true;").expect("write lazy chunk");
    fs::write(
        &source_path,
        "const comment = \"import('./ghost.jsx')\";\n/* import('./ghost.jsx') */\nconst lazy = import('./lazy.jsx');\n",
    )
    .expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy.jsx");
    assert_eq!(
        targets[0].target,
        lazy_path.canonicalize().expect("canonical lazy path")
    );
}

#[test]
fn discover_dynamic_import_targets_ignores_comment_and_string_substrings_in_tsx_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.tsx");
    let ghost_path = dir.path().join("ghost.tsx");
    let lazy_path = dir.path().join("lazy.tsx");
    fs::write(&ghost_path, "export const ghost = true;").expect("write ghost chunk");
    fs::write(&lazy_path, "export const lazy = true;").expect("write lazy chunk");
    fs::write(
        &source_path,
        "const comment = \"import('./ghost.tsx')\";\n/* import('./ghost.tsx') */\nconst lazy = import('./lazy.tsx');\n",
    )
    .expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy.tsx");
    assert_eq!(
        targets[0].target,
        lazy_path.canonicalize().expect("canonical lazy path")
    );
}

#[test]
fn discover_dynamic_import_targets_resolves_parenthesized_dynamic_import_targets_in_ts_files() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    let lazy_dir = dir.path().join("lazy");
    fs::create_dir(&lazy_dir).expect("create lazy dir");
    fs::write(lazy_dir.join("index.ts"), "export const lazy = true;").expect("write lazy index");
    fs::write(
        &source_path,
        "const name = 'lazy'; const root = './'; const lazy = import((root + name));",
    )
    .expect("write source");

    let targets = discover_dynamic_import_targets(
        &source_path,
        &fs::read_to_string(&source_path).expect("read source"),
    )
    .expect("discover dynamic import targets");

    assert_eq!(targets.len(), 1, "targets: {targets:?}");
    assert_eq!(targets[0].specifier, "./lazy");
    assert_eq!(
        targets[0].target,
        lazy_dir
            .join("index.ts")
            .canonicalize()
            .expect("canonical lazy index path")
    );
}
