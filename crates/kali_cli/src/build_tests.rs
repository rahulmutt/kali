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
        "console.log(Deno.env.get('KALI_ENV_GET_SMOKE')); console.log(Deno[\"env\"][\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis[\"Deno\"][\"env\"][\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis.Deno[\"env\"][\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis.Deno.env[\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis[\"Deno\"].env[\"get\"]('KALI_ENV_GET_SMOKE'));",
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
fn build_source_file_supports_deno_env_get_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            "console.log(Deno.env.get('KALI_ENV_GET_SMOKE')); console.log(Deno[\"env\"][\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis[\"Deno\"][\"env\"][\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis.Deno[\"env\"][\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis.Deno.env[\"get\"]('KALI_ENV_GET_SMOKE')); console.log(globalThis[\"Deno\"].env[\"get\"]('KALI_ENV_GET_SMOKE'));",
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
}

#[test]
fn build_source_file_supports_deno_env_has_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log(Deno.env.has('KALI_ENV_HAS_SMOKE') && Deno[\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"][\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis.Deno[\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis.Deno.env[\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"].env[\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"][\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE'));",
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
fn build_source_file_supports_deno_env_has_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Deno.env.has('KALI_ENV_HAS_SMOKE') && Deno[\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"][\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis.Deno[\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis.Deno.env[\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"].env[\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"][\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE'));",
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
fn build_source_file_supports_deno_env_has_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
        fs::write(
            &source_path,
            "console.log(Deno.env.has('KALI_ENV_HAS_SMOKE') && Deno[\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"][\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis.Deno[\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis.Deno.env[\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"].env[\"has\"]('KALI_ENV_HAS_SMOKE') && globalThis[\"Deno\"][\"env\"][\"has\"]('KALI_ENV_HAS_SMOKE'));",
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
}

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

#[test]
fn build_source_file_supports_deno_env_set_and_delete_in_all_inputs() {
    for extension in ["ts", "js", "jsx", "tsx"] {
        assert_build_source_file_supports_deno_env_set_and_delete_in_input(extension);
    }
}

#[test]
fn build_source_file_supports_deno_pid_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log(Deno.pid); console.log(Deno[\"pid\"]); console.log(globalThis.Deno.pid); console.log(globalThis[\"Deno\"][\"pid\"]);",
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
fn build_source_file_supports_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Deno.pid); console.log(Deno[\"pid\"]); console.log(globalThis.Deno.pid); console.log(globalThis[\"Deno\"][\"pid\"]);",
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
fn build_source_file_supports_bracketed_deno_pid_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log(Deno[\"pid\"]); console.log(globalThis[\"Deno\"][\"pid\"]);",
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
fn build_source_file_supports_bracketed_deno_cwd_chdir_and_exit_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let nested_dir = dir.path().join("nested");
    fs::create_dir(&nested_dir).expect("create nested dir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "console.log(Deno[\"cwd\"]()); console.log(globalThis[\"Deno\"][\"cwd\"]()); Deno[\"chdir\"]('nested'); globalThis[\"Deno\"][\"chdir\"]('nested'); Deno[\"exit\"](7); globalThis[\"Deno\"][\"exit\"](7);",
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
fn build_source_file_supports_bracketed_deno_cwd_chdir_and_exit_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let nested_dir = dir.path().join("nested");
    fs::create_dir(&nested_dir).expect("create nested dir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Deno[\"cwd\"]()); console.log(globalThis[\"Deno\"][\"cwd\"]()); Deno[\"chdir\"]('nested'); globalThis[\"Deno\"][\"chdir\"]('nested'); Deno[\"exit\"](7); globalThis[\"Deno\"][\"exit\"](7);",
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
fn build_source_file_supports_bracketed_deno_pid_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "console.log(Deno[\"pid\"]); console.log(globalThis[\"Deno\"][\"pid\"]);",
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
fn build_source_file_supports_permission_query_const_bindings_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("main.{extension}"));
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

        assert!(output.output_path.exists(), "extension: {extension}");
        Validator::new()
            .validate_all(&output.wasm_bytes)
            .expect("artifact should validate");
    }
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
        r#"globalThis["Proxy"]["revocable"]({}, {}); globalThis["Proxy"].revocable({}, {});"#,
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
        r#"globalThis["Proxy"]["revocable"]({}, {}); globalThis["Proxy"].revocable({}, {});"#,
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
        r#"globalThis["Proxy"]["revocable"]({}, {}); globalThis["Proxy"].revocable({}, {});"#,
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
        r#"globalThis["Proxy"]["revocable"]({}, {}); globalThis["Proxy"].revocable({}, {});"#,
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
        r#"globalThis["Proxy"]["revocable"]({}, {}); globalThis["Proxy"].revocable({}, {});"#,
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
        r#"globalThis["Proxy"]["revocable"]({}, {}); globalThis["Proxy"].revocable({}, {});"#,
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
        r#"globalThis["Proxy"]["revocable"]({}, {}); globalThis["Proxy"].revocable({}, {});"#,
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
        r#"globalThis["Proxy"]["revocable"]({}, {}); globalThis["Proxy"].revocable({}, {});"#,
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

fn assert_build_source_file_supports_object_has_own_call_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        r#"const object = Object.fromEntries([["a", 1], ["b", 2]]); Object.hasOwn(object, "a"); globalThis.Object.hasOwn(object, "a"); globalThis["Object"]["hasOwn"](object, "a"); Object.prototype.hasOwnProperty.call(object, "a"); globalThis.Object.prototype.hasOwnProperty.call(object, "a"); globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](object, "a");"#,
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
        r#"const zero = 0; const zeroAlias = zero; console.log(Object.is(zeroAlias, -0)); console.log(Object.is(+1, 1)); console.log(Object.is(true, true)); console.log(Object.is("hello", "hello")); console.log(Object.is(null, null)); console.log(Object.is(Infinity, Infinity)); console.log(Object.is(NaN, NaN)); console.log(Object.is(-Infinity, -Infinity)); console.log(globalThis["Object"]["is"](+1, 1)); console.log(globalThis.Object["is"](+1, 1)); console.log(globalThis["Object"].is(+1, 1)); console.log(globalThis.Object.is(+1, 1));"#,
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

#[test]
fn build_source_file_supports_object_is_primitive_literals_in_deno_and_browser_ts_js_jsx_and_tsx_input(
) {
    for api_surface in [ApiSurface::Deno, ApiSurface::Browser] {
        for extension in ["ts", "js", "jsx", "tsx"] {
            assert_build_source_file_supports_object_is_primitive_literals_in_input(
                api_surface,
                extension,
            );
        }
    }
}

#[test]
fn build_source_file_supports_object_has_own_call_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.jsx");
    fs::write(
        &source_path,
        r#"Object.hasOwn({}, "a"); globalThis.Object.hasOwn({}, "a"); globalThis["Object"]["hasOwn"]({}, "a"); Object.prototype.hasOwnProperty.call({}, "a"); globalThis.Object.prototype.hasOwnProperty.call({}, "a"); globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]({}, "a");"#,
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
fn build_source_file_supports_object_has_own_call_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(
        &source_path,
        r#"Object.hasOwn({}, "a"); globalThis.Object.hasOwn({}, "a"); globalThis["Object"]["hasOwn"]({}, "a"); Object.prototype.hasOwnProperty.call({}, "a"); globalThis.Object.prototype.hasOwnProperty.call({}, "a"); globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]({}, "a");"#,
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
fn build_source_file_supports_object_has_own_call_in_browser_api_surface_in_js_input() {
    assert_build_source_file_supports_object_has_own_call_in_input(ApiSurface::Browser, "js");
}

#[test]
fn build_source_file_supports_object_has_own_call_in_browser_api_surface_in_ts_input() {
    assert_build_source_file_supports_object_has_own_call_in_input(ApiSurface::Browser, "ts");
}

#[test]
fn build_source_file_supports_object_has_own_call_in_browser_api_surface_in_jsx_input() {
    assert_build_source_file_supports_object_has_own_call_in_input(ApiSurface::Browser, "jsx");
}

#[test]
fn build_source_file_supports_object_has_own_call_in_browser_api_surface_in_tsx_input() {
    assert_build_source_file_supports_object_has_own_call_in_input(ApiSurface::Browser, "tsx");
}

fn promise_all_settled_source_variants() -> [&'static str; 10] {
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

#[test]
fn build_source_file_supports_promise_all_settled_across_input_classes() {
    for api_surface in [ApiSurface::Deno, ApiSurface::Browser] {
        for extension in ["ts", "js", "jsx", "tsx"] {
            assert_build_source_file_supports_promise_all_settled_in_input(api_surface, extension);
        }
    }
}

fn unsupported_math_member_call_source_variants(method: &str) -> [String; 6] {
    [
        format!("console.log(Math.{method}(1.6));\n"),
        format!("console.log(Math[\"{method}\"](1.6));\n"),
        format!("console.log(globalThis.Math.{method}(1.6));\n"),
        format!("console.log(globalThis.Math[\"{method}\"](1.6));\n"),
        format!("console.log(globalThis[\"Math\"][\"{method}\"](1.6));\n"),
        format!("console.log(globalThis[\"Math\"].{method}(1.6));\n"),
    ]
}

fn assert_build_source_file_rejects_unsupported_math_member_calls_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    for method in ["sqrt", "exp", "log"] {
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

fn assert_build_source_file_supports_math_floor_const_numeric_alias_chain_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const value = 1.6; const alias = value; console.log(Math.floor(alias));\n",
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

#[test]
fn build_source_file_supports_math_floor_const_numeric_alias_chain_in_js_input() {
    assert_build_source_file_supports_math_floor_const_numeric_alias_chain_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_floor_const_numeric_alias_chain_in_ts_input() {
    assert_build_source_file_supports_math_floor_const_numeric_alias_chain_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_floor_const_numeric_alias_chain_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_floor_const_numeric_alias_chain_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_floor_const_numeric_alias_chain_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_floor_const_numeric_alias_chain_in_input(
        ApiSurface::Browser,
        "ts",
    );
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

#[test]
fn build_source_file_supports_math_ceil_and_trunc_const_numeric_alias_chain_in_js_input() {
    assert_build_source_file_supports_math_ceil_and_trunc_const_numeric_alias_chain_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_ceil_and_trunc_const_numeric_alias_chain_in_ts_input() {
    assert_build_source_file_supports_math_ceil_and_trunc_const_numeric_alias_chain_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_ceil_and_trunc_const_numeric_alias_chain_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_ceil_and_trunc_const_numeric_alias_chain_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_ceil_and_trunc_const_numeric_alias_chain_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_ceil_and_trunc_const_numeric_alias_chain_in_input(
        ApiSurface::Browser,
        "ts",
    );
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

#[test]
fn build_source_file_supports_bracketed_global_this_math_floor_trunc_and_ceil_numeric_literals_in_js_input(
) {
    assert_build_source_file_supports_bracketed_global_this_math_floor_trunc_and_ceil_numeric_literals_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_bracketed_global_this_math_floor_trunc_and_ceil_numeric_literals_in_ts_input(
) {
    assert_build_source_file_supports_bracketed_global_this_math_floor_trunc_and_ceil_numeric_literals_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_bracketed_global_this_math_floor_trunc_and_ceil_numeric_literals_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_bracketed_global_this_math_floor_trunc_and_ceil_numeric_literals_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_bracketed_global_this_math_floor_trunc_and_ceil_numeric_literals_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_bracketed_global_this_math_floor_trunc_and_ceil_numeric_literals_in_input(
        ApiSurface::Browser,
        "ts",
    );
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
        "const value = 1.6; console.log(globalThis.Math.round(value)); console.log(globalThis.Math[\"round\"](value)); console.log(globalThis[\"Math\"].round(value)); console.log(globalThis[\"Math\"][\"round\"](value));\n",
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

#[test]
fn build_source_file_supports_math_round_const_numeric_alias_chain_in_js_input() {
    assert_build_source_file_supports_math_round_const_numeric_alias_chain_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_round_const_numeric_alias_chain_in_ts_input() {
    assert_build_source_file_supports_math_round_const_numeric_alias_chain_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_round_const_numeric_alias_chain_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_round_const_numeric_alias_chain_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_round_const_numeric_alias_chain_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_round_const_numeric_alias_chain_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_global_this_math_round_identity_in_js_input() {
    assert_build_source_file_supports_global_this_math_round_identity_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_global_this_math_round_identity_in_ts_input() {
    assert_build_source_file_supports_global_this_math_round_identity_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_global_this_math_round_identity_in_browser_api_surface_in_js_input() {
    assert_build_source_file_supports_global_this_math_round_identity_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_global_this_math_round_identity_in_browser_api_surface_in_ts_input() {
    assert_build_source_file_supports_global_this_math_round_identity_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_round_const_numeric_alias_chain_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_math_round_const_numeric_alias_chain_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_math_round_const_numeric_alias_chain_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_math_round_const_numeric_alias_chain_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_global_this_math_round_identity_in_browser_api_surface_in_jsx_input()
{
    assert_build_source_file_supports_global_this_math_round_identity_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_global_this_math_round_identity_in_browser_api_surface_in_tsx_input()
{
    assert_build_source_file_supports_global_this_math_round_identity_in_input(
        ApiSurface::Browser,
        "tsx",
    );
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

#[test]
fn build_source_file_supports_math_log2_and_log10_const_numeric_alias_chain_in_js_input() {
    assert_build_source_file_supports_math_log2_and_log10_const_numeric_alias_chain_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_log2_and_log10_const_numeric_alias_chain_in_ts_input() {
    assert_build_source_file_supports_math_log2_and_log10_const_numeric_alias_chain_in_input(
        ApiSurface::Deno,
        "ts",
    );
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

#[test]
fn build_source_file_supports_global_this_math_mixed_bracket_log2_and_log10_identity_literals_in_js_input(
) {
    assert_build_source_file_supports_global_this_math_mixed_bracket_log2_and_log10_identity_literals_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_global_this_math_mixed_bracket_log2_and_log10_identity_literals_in_ts_input(
) {
    assert_build_source_file_supports_global_this_math_mixed_bracket_log2_and_log10_identity_literals_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_global_this_math_mixed_bracket_log2_and_log10_identity_literals_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_global_this_math_mixed_bracket_log2_and_log10_identity_literals_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_global_this_math_mixed_bracket_log2_and_log10_identity_literals_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_global_this_math_mixed_bracket_log2_and_log10_identity_literals_in_input(
        ApiSurface::Browser,
        "ts",
    );
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

#[test]
fn build_source_file_supports_math_hypot_perfect_square_literal_in_js_input() {
    assert_build_source_file_supports_math_hypot_perfect_square_literal_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_hypot_perfect_square_literal_in_ts_input() {
    assert_build_source_file_supports_math_hypot_perfect_square_literal_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_hypot_perfect_square_literal_in_browser_api_surface_in_js_input()
{
    assert_build_source_file_supports_math_hypot_perfect_square_literal_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_hypot_perfect_square_literal_in_browser_api_surface_in_ts_input()
{
    assert_build_source_file_supports_math_hypot_perfect_square_literal_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_hypot_zero_arguments_in_js_input() {
    assert_build_source_file_supports_math_hypot_zero_arguments_in_input(ApiSurface::Deno, "js");
}

#[test]
fn build_source_file_supports_math_clz32_zero_arguments_in_js_input() {
    assert_build_source_file_supports_math_clz32_zero_arguments_in_input(ApiSurface::Deno, "js");
}

#[test]
fn build_source_file_supports_math_clz32_zero_arguments_in_ts_input() {
    assert_build_source_file_supports_math_clz32_zero_arguments_in_input(ApiSurface::Deno, "ts");
}

#[test]
fn build_source_file_supports_math_clz32_zero_arguments_in_jsx_input() {
    assert_build_source_file_supports_math_clz32_zero_arguments_in_input(ApiSurface::Deno, "jsx");
}

#[test]
fn build_source_file_supports_math_clz32_zero_arguments_in_tsx_input() {
    assert_build_source_file_supports_math_clz32_zero_arguments_in_input(ApiSurface::Deno, "tsx");
}

#[test]
fn build_source_file_supports_math_clz32_zero_arguments_in_browser_api_surface_in_js_input() {
    assert_build_source_file_supports_math_clz32_zero_arguments_in_input(ApiSurface::Browser, "js");
}

#[test]
fn build_source_file_supports_math_clz32_zero_arguments_in_browser_api_surface_in_ts_input() {
    assert_build_source_file_supports_math_clz32_zero_arguments_in_input(ApiSurface::Browser, "ts");
}

#[test]
fn build_source_file_supports_math_clz32_zero_arguments_in_browser_api_surface_in_jsx_input() {
    assert_build_source_file_supports_math_clz32_zero_arguments_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_math_clz32_zero_arguments_in_browser_api_surface_in_tsx_input() {
    assert_build_source_file_supports_math_clz32_zero_arguments_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_math_hypot_zero_arguments_in_ts_input() {
    assert_build_source_file_supports_math_hypot_zero_arguments_in_input(ApiSurface::Deno, "ts");
}

#[test]
fn build_source_file_supports_math_hypot_zero_arguments_in_jsx_input() {
    assert_build_source_file_supports_math_hypot_zero_arguments_in_input(ApiSurface::Deno, "jsx");
}

#[test]
fn build_source_file_supports_math_hypot_zero_arguments_in_tsx_input() {
    assert_build_source_file_supports_math_hypot_zero_arguments_in_input(ApiSurface::Deno, "tsx");
}

#[test]
fn build_source_file_supports_math_hypot_zero_arguments_in_browser_api_surface_in_js_input() {
    assert_build_source_file_supports_math_hypot_zero_arguments_in_input(ApiSurface::Browser, "js");
}

#[test]
fn build_source_file_supports_math_hypot_zero_arguments_in_browser_api_surface_in_ts_input() {
    assert_build_source_file_supports_math_hypot_zero_arguments_in_input(ApiSurface::Browser, "ts");
}

#[test]
fn build_source_file_supports_math_hypot_zero_arguments_in_browser_api_surface_in_jsx_input() {
    assert_build_source_file_supports_math_hypot_zero_arguments_in_input(
        ApiSurface::Browser,
        "jsx",
    );
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

#[test]
fn build_source_file_supports_math_sqrt_perfect_square_literal_in_js_input() {
    assert_build_source_file_supports_math_sqrt_perfect_square_literal_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_sqrt_perfect_square_literal_in_ts_input() {
    assert_build_source_file_supports_math_sqrt_perfect_square_literal_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_sqrt_perfect_square_literal_in_browser_api_surface_in_js_input()
{
    assert_build_source_file_supports_math_sqrt_perfect_square_literal_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_sqrt_perfect_square_literal_in_browser_api_surface_in_ts_input()
{
    assert_build_source_file_supports_math_sqrt_perfect_square_literal_in_input(
        ApiSurface::Browser,
        "ts",
    );
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

#[test]
fn build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_js_input() {
    assert_build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_ts_input() {
    assert_build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_cbrt_negative_perfect_cube_literal_in_input(
        ApiSurface::Browser,
        "ts",
    );
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

#[test]
fn build_source_file_supports_math_inverse_hyperbolic_identity_literals_in_js_input() {
    assert_build_source_file_supports_math_inverse_hyperbolic_identity_literals_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_inverse_hyperbolic_identity_literals_in_ts_input() {
    assert_build_source_file_supports_math_inverse_hyperbolic_identity_literals_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_inverse_hyperbolic_identity_literals_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_inverse_hyperbolic_identity_literals_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_inverse_hyperbolic_identity_literals_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_inverse_hyperbolic_identity_literals_in_input(
        ApiSurface::Browser,
        "ts",
    );
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

fn assert_build_source_file_supports_math_expm1_and_log1p_identity_literals_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "console.log(Math.expm1(0)); console.log(Math.log1p(0));\n",
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
    .expect("Math.expm1/log1p identity build should succeed");

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

#[test]
fn build_source_file_supports_math_inverse_trig_identity_literals_in_js_input() {
    assert_build_source_file_supports_math_inverse_trig_identity_literals_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_inverse_trig_identity_literals_in_ts_input() {
    assert_build_source_file_supports_math_inverse_trig_identity_literals_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_inverse_trig_identity_literals_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_inverse_trig_identity_literals_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_inverse_trig_identity_literals_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_inverse_trig_identity_literals_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_js_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_fully_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_js_input(
) {
    assert_build_source_file_supports_fully_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(globalThis[\"Math\"].atan2(zero, one));\n",
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
    .expect("globalThis[\"Math\"].atan2 literal build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn build_source_file_supports_single_quoted_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_js_input(
) {
    assert_build_source_file_supports_single_quoted_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_single_quoted_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_ts_input(
) {
    assert_build_source_file_supports_single_quoted_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_single_quoted_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_single_quoted_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_single_quoted_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_single_quoted_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Browser,
        "ts",
    );
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

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_as_const_wrappers_in_ts_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_wrapper_literals_in_input(
        ApiSurface::Deno,
        "ts",
        "const zero = (0 as const); const one = (1 as const); console.log(globalThis[\"Math\"].atan2(zero, one));\n",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_satisfies_wrappers_in_ts_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_wrapper_literals_in_input(
        ApiSurface::Deno,
        "ts",
        "const zero = (0 satisfies number); const one = (1 satisfies number); console.log(globalThis[\"Math\"].atan2(zero, one));\n",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_as_const_wrappers_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_wrapper_literals_in_input(
        ApiSurface::Browser,
        "ts",
        "const zero = (0 as const); const one = (1 as const); console.log(globalThis[\"Math\"].atan2(zero, one));\n",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_satisfies_wrappers_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_wrapper_literals_in_input(
        ApiSurface::Browser,
        "ts",
        "const zero = (0 satisfies number); const one = (1 satisfies number); console.log(globalThis[\"Math\"].atan2(zero, one));\n",
    );
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

#[test]
fn build_source_file_supports_global_this_math_atan2_zero_numerator_and_non_negative_denominator_root_variants_in_js_input(
) {
    assert_build_source_file_supports_global_this_math_atan2_zero_numerator_and_non_negative_denominator_root_variants_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_global_this_math_atan2_zero_numerator_and_non_negative_denominator_root_variants_in_ts_input(
) {
    assert_build_source_file_supports_global_this_math_atan2_zero_numerator_and_non_negative_denominator_root_variants_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_global_this_math_atan2_zero_numerator_and_non_negative_denominator_root_variants_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_global_this_math_atan2_zero_numerator_and_non_negative_denominator_root_variants_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_global_this_math_atan2_zero_numerator_and_non_negative_denominator_root_variants_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_global_this_math_atan2_zero_numerator_and_non_negative_denominator_root_variants_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_global_this_math_exp_and_log_exact_identity_literals_in_browser_api_surface_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(globalThis.Math.exp(zero)); console.log(globalThis.Math.log(one));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Browser,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("globalThis.Math.exp/log identity build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn build_source_file_supports_global_this_bracketed_math_exp_and_log_exact_identity_literals_in_browser_api_surface_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(globalThis[\"Math\"][\"exp\"](zero)); console.log(globalThis[\"Math\"][\"log\"](one));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Browser,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("globalThis[\"Math\"][\"exp\"]/[\"log\"] identity build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn build_source_file_supports_global_this_math_exp_and_log_exact_identity_literals_in_browser_api_surface_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(globalThis.Math.exp(zero)); console.log(globalThis.Math.log(one));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Browser,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("globalThis.Math.exp/log identity build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn build_source_file_supports_global_this_bracketed_math_exp_and_log_exact_identity_literals_in_browser_api_surface_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(globalThis[\"Math\"][\"exp\"](zero)); console.log(globalThis[\"Math\"][\"log\"](one));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Browser,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("globalThis[\"Math\"][\"exp\"]/[\"log\"] identity build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn build_source_file_supports_global_this_mixed_bracket_math_exp_and_log_exact_identity_literals_in_browser_api_surface_in_js_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(globalThis.Math[\"exp\"](zero)); console.log(globalThis.Math[\"log\"](one));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Browser,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("globalThis.Math[\"exp\"]/[\"log\"] identity build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn build_source_file_supports_global_this_mixed_bracket_math_exp_and_log_exact_identity_literals_in_browser_api_surface_in_ts_input(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const zero = 0; const one = 1; console.log(globalThis.Math[\"exp\"](zero)); console.log(globalThis.Math[\"log\"](one));\n",
    )
    .expect("write source");

    let output = build_source_file(
        &source_path,
        BuildMode::Fast,
        ApiSurface::Browser,
        false,
        &[],
        16,
        None,
        None,
    )
    .expect("globalThis.Math[\"exp\"]/[\"log\"] identity build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_ts_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_fully_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_fully_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_fully_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_fully_bracketed_global_this_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_jsx_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Deno,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_tsx_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Deno,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_literals_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_const_alias_chain_in_js_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_const_alias_chain_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_const_alias_chain_in_ts_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_const_alias_chain_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_const_alias_chain_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_const_alias_chain_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_const_alias_chain_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_atan2_zero_numerator_and_non_negative_denominator_const_alias_chain_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_expm1_and_log1p_identity_literals_in_js_input() {
    assert_build_source_file_supports_math_expm1_and_log1p_identity_literals_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_expm1_and_log1p_identity_literals_in_ts_input() {
    assert_build_source_file_supports_math_expm1_and_log1p_identity_literals_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_expm1_and_log1p_identity_literals_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_expm1_and_log1p_identity_literals_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_expm1_and_log1p_identity_literals_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_expm1_and_log1p_identity_literals_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_exp2_zero_identity_in_js_input() {
    assert_build_source_file_supports_math_exp2_zero_identity_in_input(ApiSurface::Deno, "js");
}

#[test]
fn build_source_file_supports_math_exp2_zero_identity_in_ts_input() {
    assert_build_source_file_supports_math_exp2_zero_identity_in_input(ApiSurface::Deno, "ts");
}

#[test]
fn build_source_file_supports_math_exp2_zero_identity_in_browser_api_surface_in_js_input() {
    assert_build_source_file_supports_math_exp2_zero_identity_in_input(ApiSurface::Browser, "js");
}

#[test]
fn build_source_file_supports_math_exp2_zero_identity_in_browser_api_surface_in_ts_input() {
    assert_build_source_file_supports_math_exp2_zero_identity_in_input(ApiSurface::Browser, "ts");
}

#[test]
fn build_source_file_supports_math_exp2_zero_identity_in_jsx_input() {
    assert_build_source_file_supports_math_exp2_zero_identity_in_input(ApiSurface::Deno, "jsx");
}

#[test]
fn build_source_file_supports_math_exp2_zero_identity_in_tsx_input() {
    assert_build_source_file_supports_math_exp2_zero_identity_in_input(ApiSurface::Deno, "tsx");
}

#[test]
fn build_source_file_supports_math_exp2_zero_identity_in_browser_api_surface_in_jsx_input() {
    assert_build_source_file_supports_math_exp2_zero_identity_in_input(ApiSurface::Browser, "jsx");
}

#[test]
fn build_source_file_supports_math_exp2_zero_identity_in_browser_api_surface_in_tsx_input() {
    assert_build_source_file_supports_math_exp2_zero_identity_in_input(ApiSurface::Browser, "tsx");
}

#[test]
fn build_source_file_supports_global_this_math_exp2_zero_identity_in_js_input() {
    assert_build_source_file_supports_global_this_math_exp2_zero_identity_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_global_this_math_exp2_zero_identity_in_ts_input() {
    assert_build_source_file_supports_global_this_math_exp2_zero_identity_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_global_this_math_exp2_zero_identity_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_global_this_math_exp2_zero_identity_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_global_this_math_exp2_zero_identity_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_global_this_math_exp2_zero_identity_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_global_this_math_exp2_zero_identity_in_jsx_input() {
    assert_build_source_file_supports_global_this_math_exp2_zero_identity_in_input(
        ApiSurface::Deno,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_global_this_math_exp2_zero_identity_in_tsx_input() {
    assert_build_source_file_supports_global_this_math_exp2_zero_identity_in_input(
        ApiSurface::Deno,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_global_this_math_exp2_zero_identity_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_global_this_math_exp2_zero_identity_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_global_this_math_exp2_zero_identity_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_global_this_math_exp2_zero_identity_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_math_exp_and_log_const_alias_chain_in_js_input() {
    assert_build_source_file_supports_math_exp_and_log_const_alias_chain_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_exp_and_log_const_alias_chain_in_ts_input() {
    assert_build_source_file_supports_math_exp_and_log_const_alias_chain_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_exp_and_log_const_alias_chain_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_exp_and_log_const_alias_chain_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_exp_and_log_const_alias_chain_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_exp_and_log_const_alias_chain_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_pow_negative_integer_exponents_for_unit_bases_in_js_input() {
    assert_build_source_file_supports_math_pow_negative_integer_exponents_for_unit_bases_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_pow_negative_integer_exponents_for_unit_bases_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_pow_negative_integer_exponents_for_unit_bases_in_input(
        ApiSurface::Browser,
        "js",
    );
}

fn assert_build_source_file_supports_math_expm1_and_log1p_const_alias_chain_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const zero = 0; const alias = zero; console.log(Math.expm1(alias)); console.log(Math.log1p(alias));\n",
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
    .expect("Math.expm1/log1p const alias chain build should succeed");

    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("generated wasm should validate");
}

#[test]
fn build_source_file_supports_math_expm1_and_log1p_const_alias_chain_in_js_input() {
    assert_build_source_file_supports_math_expm1_and_log1p_const_alias_chain_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_expm1_and_log1p_const_alias_chain_in_ts_input() {
    assert_build_source_file_supports_math_expm1_and_log1p_const_alias_chain_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_math_expm1_and_log1p_const_alias_chain_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_math_expm1_and_log1p_const_alias_chain_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_math_expm1_and_log1p_const_alias_chain_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_math_expm1_and_log1p_const_alias_chain_in_input(
        ApiSurface::Browser,
        "ts",
    );
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

#[test]
fn build_source_file_supports_for_of_identifier_binding_in_ts_input() {
    assert_build_source_file_supports_for_of_identifier_binding_in_input("ts");
}

#[test]
fn build_source_file_supports_for_of_identifier_binding_in_js_input() {
    assert_build_source_file_supports_for_of_identifier_binding_in_input("js");
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

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_ts_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_js_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_jsx_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Deno,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_tsx_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Deno,
        "tsx",
    );
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

#[test]
fn build_source_file_supports_for_of_array_iteration_with_sequence_wrappers_in_js_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_sequence_wrappers_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_sequence_wrappers_in_ts_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_sequence_wrappers_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_sequence_wrappers_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_sequence_wrappers_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_sequence_wrappers_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_sequence_wrappers_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_spread_of_object_values_iterator_slices_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_spread_of_object_values_iterator_slices_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_spread_of_object_values_iterator_slices_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_spread_of_object_values_iterator_slices_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_spread_of_object_values_iterator_slices_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_spread_of_object_values_iterator_slices_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_spread_of_object_values_iterator_slices_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_spread_of_object_values_iterator_slices_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn check_source_file_supports_spread_of_object_values_iterator_slices_in_browser_api_surface_in_js_input(
) {
    assert_check_source_file_supports_spread_of_object_values_iterator_slices_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn check_source_file_supports_spread_of_object_values_iterator_slices_in_browser_api_surface_in_ts_input(
) {
    assert_check_source_file_supports_spread_of_object_values_iterator_slices_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn check_source_file_supports_spread_of_object_values_iterator_slices_in_browser_api_surface_in_jsx_input(
) {
    assert_check_source_file_supports_spread_of_object_values_iterator_slices_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn check_source_file_supports_spread_of_object_values_iterator_slices_in_browser_api_surface_in_tsx_input(
) {
    assert_check_source_file_supports_spread_of_object_values_iterator_slices_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_spread_of_object_keys_and_entries_iterator_slices_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_spread_of_object_keys_and_entries_iterator_slices_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn check_source_file_supports_spread_of_object_keys_and_entries_iterator_slices_in_browser_api_surface_in_js_input(
) {
    assert_check_source_file_supports_spread_of_object_keys_and_entries_iterator_slices_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_object_keys_const_bound_iterable_in_browser_api_surface_in_js_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_build_source_file_supports_for_of_object_keys_const_bound_iterable_in_input(
            ApiSurface::Browser,
            extension,
        );
    }
}

#[test]
fn check_source_file_supports_for_of_object_keys_const_bound_iterable_in_browser_api_surface_in_js_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_check_source_file_supports_for_of_object_keys_const_bound_iterable_in_input(
            ApiSurface::Browser,
            extension,
        );
    }
}

fn assert_build_source_file_rejects_for_of_non_literal_iterable_in_input(extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "let values = [1, 2]; for (const item of values) { console.log(item); }\n",
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

fn assert_check_source_file_rejects_for_of_object_keys_non_literal_iterable_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "let values = { a: 1 }; for (const key of Object.keys(values)) { console.log(key); }\n",
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

fn assert_check_source_file_supports_spread_of_object_values_iterator_slices_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const values = Object.values({ a: 1, b: 2 }); for (const item of [...values]) { console.log(item); } for await (const item of [...Object.values({ a: 3, b: 4 })]) { console.log(item); }\n",
    )
    .expect("write source");

    check_source_file(&source_path, api_surface, &[], false, false)
        .expect("spread of object.values iterator slices should succeed");
}

fn assert_build_source_file_supports_spread_of_object_values_iterator_slices_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "const values = Object.values({ a: 1, b: 2 }); for (const item of [...values]) { console.log(item); } for await (const item of [...Object.values({ a: 3, b: 4 })]) { console.log(item); }\n",
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
    .expect("spread of object.values iterator slices should succeed");

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
        "for (const key of [...Object.keys(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))]) { console.log(key); } for (const entry of [...Object.entries(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))]) { console.log(entry[0]); console.log(entry[1]); } for await (const key of [...Object.keys(Object.fromEntries([[\"c\", 4], [\"d\", 5], [\"c\", 6]]))]) { console.log(key); } for await (const entry of [...Object.entries(Object.fromEntries([[\"c\", 4], [\"d\", 5], [\"c\", 6]]))]) { console.log(entry[0]); console.log(entry[1]); }\n",
    )
    .expect("write source");

    check_source_file(&source_path, api_surface, &[], false, false)
        .expect("spread of object.keys/object.entries iterator slices should succeed");
}

fn assert_build_source_file_supports_spread_of_object_keys_and_entries_iterator_slices_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "for (const key of [...Object.keys(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))]) { console.log(key); } for (const entry of [...Object.entries(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]))]) { console.log(entry[0]); console.log(entry[1]); } for await (const key of [...Object.keys(Object.fromEntries([[\"c\", 4], [\"d\", 5], [\"c\", 6]]))]) { console.log(key); } for await (const entry of [...Object.entries(Object.fromEntries([[\"c\", 4], [\"d\", 5], [\"c\", 6]]))]) { console.log(entry[0]); console.log(entry[1]); }\n",
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

#[test]
fn build_source_file_supports_for_await_array_iteration_with_sequence_wrappers_in_js_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_sequence_wrappers_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_sequence_wrappers_in_ts_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_sequence_wrappers_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_sequence_wrappers_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_sequence_wrappers_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_sequence_wrappers_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_sequence_wrappers_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

fn assert_build_source_file_rejects_for_await_non_literal_iterable_in_input(extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        "let values = [1, 2]; for await (const item of values) { console.log(item); }\n",
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

#[test]
fn build_source_file_rejects_for_of_non_literal_iterable_in_ts_input() {
    assert_build_source_file_rejects_for_of_non_literal_iterable_in_input("ts");
}

#[test]
fn build_source_file_rejects_for_of_non_literal_iterable_in_js_input() {
    assert_build_source_file_rejects_for_of_non_literal_iterable_in_input("js");
}

#[test]
fn check_source_file_rejects_for_of_object_keys_non_literal_iterable_in_browser_js_input() {
    assert_check_source_file_rejects_for_of_object_keys_non_literal_iterable_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn check_source_file_rejects_for_of_object_keys_non_literal_iterable_in_browser_ts_input() {
    assert_check_source_file_rejects_for_of_object_keys_non_literal_iterable_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn check_source_file_rejects_for_of_object_keys_non_literal_iterable_in_browser_jsx_input() {
    assert_check_source_file_rejects_for_of_object_keys_non_literal_iterable_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn check_source_file_rejects_for_of_object_keys_non_literal_iterable_in_browser_tsx_input() {
    assert_check_source_file_rejects_for_of_object_keys_non_literal_iterable_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_rejects_for_await_non_literal_iterable_in_ts_input() {
    assert_build_source_file_rejects_for_await_non_literal_iterable_in_input("ts");
}

#[test]
fn build_source_file_rejects_for_await_non_literal_iterable_in_js_input() {
    assert_build_source_file_rejects_for_await_non_literal_iterable_in_input("js");
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

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_alias_in_js_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_const_alias_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_alias_in_ts_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_const_alias_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_alias_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_const_alias_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_alias_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_const_alias_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_alias_chain_in_js_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_const_alias_chain_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_alias_chain_in_ts_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_const_alias_chain_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_alias_chain_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_const_alias_chain_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_alias_chain_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_const_alias_chain_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_string_alias_in_js_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_const_string_alias_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_boolean_alias_in_js_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_const_boolean_alias_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_string_alias_in_js_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_const_string_alias_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_string_alias_in_ts_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_const_string_alias_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_string_alias_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_const_string_alias_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_boolean_alias_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_const_boolean_alias_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_js_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_ts_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_spread_of_const_bound_literal_arrays_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_const_string_alias_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_const_string_alias_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_js_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_ts_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_parenthesized_const_alias_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_satisfies_wrapper_in_ts_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_satisfies_wrapper_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_satisfies_wrapper_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_satisfies_wrapper_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_satisfies_wrapper_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_satisfies_wrapper_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_as_const_wrapper_in_ts_input() {
    assert_build_source_file_supports_for_of_array_iteration_with_as_const_wrapper_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_as_const_wrapper_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_as_const_wrapper_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_of_array_iteration_with_as_const_wrapper_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_of_array_iteration_with_as_const_wrapper_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_const_alias_in_js_input()
{
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_const_alias_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_const_alias_in_ts_input()
{
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_const_alias_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_const_alias_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_const_alias_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_spread_of_const_bound_literal_arrays_in_js_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_spread_of_const_bound_literal_arrays_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_spread_of_const_bound_literal_arrays_in_ts_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_spread_of_const_bound_literal_arrays_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_spread_of_const_bound_literal_arrays_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_spread_of_const_bound_literal_arrays_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_spread_of_const_bound_literal_arrays_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_spread_of_const_bound_literal_arrays_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_const_alias_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_const_alias_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_string_alias_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_const_string_alias_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_boolean_alias_in_js_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_const_boolean_alias_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_boolean_alias_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_const_boolean_alias_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_string_alias_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_const_string_alias_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_satisfies_wrapper_in_ts_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_satisfies_wrapper_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_satisfies_wrapper_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_satisfies_wrapper_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_satisfies_wrapper_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_satisfies_wrapper_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_satisfies_wrapper_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_satisfies_wrapper_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_satisfies_wrapper_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_satisfies_wrapper_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_as_const_wrapper_in_ts_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_as_const_wrapper_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_as_const_wrapper_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_as_const_wrapper_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_as_const_wrapper_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_as_const_wrapper_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_as_const_wrapper_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_as_const_wrapper_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_as_const_wrapper_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_as_const_wrapper_in_input(
        ApiSurface::Browser,
        "tsx",
    );
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
        error.iter().any(
            |diagnostic| diagnostic.message.contains("generator function lowering")
                || diagnostic.message.contains("yield expressions")
        ),
        "unexpected diagnostics: {error:?}"
    );
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
fn collect_library_exports_rejects_generator_default_export_expression() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::FunctionExpression(Box::new(
            kali_ast::FunctionExpression {
                id: None,
                params: vec![],
                body: Some(Box::new(kali_ast::BlockStatement { body: vec![] })),
                is_async: false,
                generator: true,
            },
        ))),
    )];

    let error = collect_library_exports_from_statements(&statements, &source_path)
        .expect_err("generator default exports should fail");
    assert!(
        error.iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "unexpected diagnostics: {error:?}"
    );
    assert!(
        error
            .iter()
            .any(|diagnostic| diagnostic.message.contains("generator function lowering")),
        "unexpected diagnostics: {error:?}"
    );
}

#[test]
fn collect_library_exports_rejects_generator_exported_binding() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");

    let statements = vec![
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![kali_ast::VariableDeclarator {
                id: "exported".to_string(),
                init: Some(Expression::FunctionExpression(Box::new(
                    kali_ast::FunctionExpression {
                        id: None,
                        params: vec![],
                        body: Some(Box::new(kali_ast::BlockStatement { body: vec![] })),
                        is_async: false,
                        generator: true,
                    },
                ))),
            }],
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![kali_ast::ExportSpecifier {
                local: "exported".to_string(),
                exported: "exported".to_string(),
            }],
            source: None,
        }),
    ];

    let error = collect_library_exports_from_statements(&statements, &source_path)
        .expect_err("generator exported bindings should fail");
    assert!(
        error.iter().any(|diagnostic| diagnostic.code
            == Some(kali_error::_error_codes::e5::FEATURE_UNAVAILABLE as u32)),
        "unexpected diagnostics: {error:?}"
    );
    assert!(
        error
            .iter()
            .any(|diagnostic| diagnostic.message.contains("generator function lowering")),
        "unexpected diagnostics: {error:?}"
    );
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
        error.iter().any(
            |diagnostic| diagnostic.message.contains("generator function lowering")
                || diagnostic.message.contains("yield expressions")
        ),
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

#[test]
fn build_source_file_supports_for_await_array_iteration_in_js_input() {
    assert_build_source_file_supports_for_await_array_iteration_in_input(ApiSurface::Deno, "js");
}

#[test]
fn build_source_file_supports_for_await_array_iteration_in_ts_input() {
    assert_build_source_file_supports_for_await_array_iteration_in_input(ApiSurface::Deno, "ts");
}

#[test]
fn build_source_file_supports_for_await_array_iteration_in_jsx_input() {
    assert_build_source_file_supports_for_await_array_iteration_in_input(ApiSurface::Deno, "jsx");
}

#[test]
fn build_source_file_supports_for_await_array_iteration_in_tsx_input() {
    assert_build_source_file_supports_for_await_array_iteration_in_input(ApiSurface::Deno, "tsx");
}

#[test]
fn build_source_file_supports_for_await_array_iteration_in_browser_api_surface_in_js_input() {
    assert_build_source_file_supports_for_await_array_iteration_in_input(ApiSurface::Browser, "js");
}

#[test]
fn build_source_file_supports_for_await_array_iteration_in_browser_api_surface_in_ts_input() {
    assert_build_source_file_supports_for_await_array_iteration_in_input(ApiSurface::Browser, "ts");
}

#[test]
fn build_source_file_supports_for_await_array_iteration_in_browser_api_surface_in_jsx_input() {
    assert_build_source_file_supports_for_await_array_iteration_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_in_browser_api_surface_in_tsx_input() {
    assert_build_source_file_supports_for_await_array_iteration_in_input(
        ApiSurface::Browser,
        "tsx",
    );
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

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_alias_in_js_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_const_alias_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_alias_in_ts_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_const_alias_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_alias_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_const_alias_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_js_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_browser_api_surface_in_jsx_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Browser,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_browser_api_surface_in_tsx_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Browser,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_jsx_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Deno,
        "jsx",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_tsx_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Deno,
        "tsx",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_ts_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_parenthesized_binding_in_input(
        ApiSurface::Browser,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_alias_chain_in_js_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_const_alias_chain_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_alias_chain_in_ts_input() {
    assert_build_source_file_supports_for_await_array_iteration_with_const_alias_chain_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_alias_chain_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_const_alias_chain_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_for_await_array_iteration_with_const_alias_chain_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_for_await_array_iteration_with_const_alias_chain_in_input(
        ApiSurface::Browser,
        "ts",
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

fn assert_build_source_file_rejects_process_env_mutation_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        r#"process.env = {}; process.env.KALI_BROWSER_ENV_MUTATION = {}; globalThis.process.env = {}; globalThis.process.env.KALI_BROWSER_ENV_MUTATION = {}; process["env"] = {}; process["env"].KALI_BROWSER_ENV_MUTATION = {}; globalThis.process["env"] = {}; globalThis.process["env"].KALI_BROWSER_ENV_MUTATION = {}; globalThis["process"].env = {}; globalThis["process"].env.KALI_BROWSER_ENV_MUTATION = {}; globalThis["process"]["env"] = {}; globalThis["process"]["env"].KALI_BROWSER_ENV_MUTATION = {}; globalThis["process"]["env"]["KALI_BROWSER_ENV_MUTATION"] = {}; delete globalThis["process"]["env"]["KALI_BROWSER_ENV_MUTATION"];"#,
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
fn build_source_file_supports_deno_env_to_object_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        r#"Deno.env.toObject; globalThis.Deno.env.toObject; globalThis.Deno.env["toObject"]; Deno.env["toObject"]; Deno["env"]["toObject"]; Deno["env"].toObject; globalThis.Deno["env"]["toObject"]; globalThis.Deno["env"].toObject; globalThis.Deno["env"]["toObject"]; globalThis["Deno"].env.toObject; globalThis["Deno"].env["toObject"]; globalThis["Deno"]["env"].toObject; globalThis["Deno"]["env"]["toObject"]; globalThis.Deno["env"]["toObject"];"#,
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
    .expect("env materialization APIs should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_deno_env_to_object_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        r#"Deno.env.toObject; globalThis.Deno.env.toObject; globalThis.Deno.env["toObject"]; Deno.env["toObject"]; Deno["env"]["toObject"]; Deno["env"].toObject; globalThis.Deno["env"]["toObject"]; globalThis.Deno["env"].toObject; globalThis.Deno["env"]["toObject"]; globalThis["Deno"].env.toObject; globalThis["Deno"].env["toObject"]; globalThis["Deno"]["env"].toObject; globalThis["Deno"]["env"]["toObject"]; globalThis.Deno["env"]["toObject"];"#,
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
    .expect("env materialization APIs should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_deno_env_to_object_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.jsx");
    fs::write(
        &source_path,
        r#"Deno.env.toObject; globalThis.Deno.env.toObject; globalThis.Deno.env["toObject"]; Deno.env["toObject"]; Deno["env"]["toObject"]; Deno["env"].toObject; globalThis.Deno["env"]["toObject"]; globalThis.Deno["env"].toObject; globalThis.Deno["env"]["toObject"]; globalThis["Deno"].env.toObject; globalThis["Deno"].env["toObject"]; globalThis["Deno"]["env"].toObject; globalThis["Deno"]["env"]["toObject"]; globalThis.Deno["env"]["toObject"];"#,
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
    .expect("env materialization APIs should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
}

#[test]
fn build_source_file_supports_deno_env_to_object_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(
        &source_path,
        r#"Deno.env.toObject; globalThis.Deno.env.toObject; globalThis.Deno.env["toObject"]; Deno.env["toObject"]; Deno["env"]["toObject"]; Deno["env"].toObject; globalThis.Deno["env"]["toObject"]; globalThis.Deno["env"].toObject; globalThis.Deno["env"]["toObject"]; globalThis["Deno"].env.toObject; globalThis["Deno"].env["toObject"]; globalThis["Deno"]["env"].toObject; globalThis["Deno"]["env"]["toObject"]; globalThis.Deno["env"]["toObject"];"#,
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
    .expect("env materialization APIs should succeed");

    assert!(output.output_path.exists());
    Validator::new()
        .validate_all(&output.wasm_bytes)
        .expect("artifact should validate");
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
        r#"globalThis["Intl"]["DateTimeFormat"]; globalThis["Intl"]["RelativeTimeFormat"]; globalThis["Intl"]["PluralRules"]; globalThis["Intl"]["Collator"]; globalThis["Intl"]["DisplayNames"]; globalThis["Intl"]["Segmenter"]; globalThis["Intl"]["Locale"]; globalThis["Intl"]["NumberFormat"]; Intl.NumberFormat; Intl.RelativeTimeFormat; Intl.Collator; Intl.DisplayNames; Intl.Segmenter; Intl.Locale;"#,
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
        r#"globalThis["Intl"]["DateTimeFormat"]; globalThis["Intl"]["RelativeTimeFormat"]; globalThis["Intl"]["PluralRules"]; globalThis["Intl"]["Collator"]; globalThis["Intl"]["DisplayNames"]; globalThis["Intl"]["Segmenter"]; globalThis["Intl"]["Locale"]; globalThis["Intl"]["NumberFormat"]; Intl.NumberFormat; Intl.RelativeTimeFormat; Intl.Collator; Intl.DisplayNames; Intl.Segmenter; Intl.Locale;"#,
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
        r#"globalThis["Intl"]["DateTimeFormat"]; globalThis["Intl"]["RelativeTimeFormat"]; globalThis["Intl"]["PluralRules"]; globalThis["Intl"]["Collator"]; globalThis["Intl"]["DisplayNames"]; globalThis["Intl"]["Segmenter"]; globalThis["Intl"]["Locale"]; globalThis["Intl"]["NumberFormat"]; Intl.NumberFormat; Intl.RelativeTimeFormat; Intl.Collator; Intl.DisplayNames; Intl.Segmenter; Intl.Locale;"#,
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
        r#"globalThis["Intl"]["DateTimeFormat"]; globalThis["Intl"]["RelativeTimeFormat"]; globalThis["Intl"]["PluralRules"]; globalThis["Intl"]["Collator"]; globalThis["Intl"]["DisplayNames"]; globalThis["Intl"]["Segmenter"]; globalThis["Intl"]["Locale"]; globalThis["Intl"]["NumberFormat"]; Intl.NumberFormat; Intl.RelativeTimeFormat; Intl.Collator; Intl.DisplayNames; Intl.Segmenter; Intl.Locale;"#,
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

fn assert_build_source_file_rejects_broader_intl_apis_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        r#"globalThis["Intl"]["DateTimeFormat"]; globalThis["Intl"]["RelativeTimeFormat"]; globalThis["Intl"]["PluralRules"]; globalThis["Intl"]["Collator"]; globalThis["Intl"]["DisplayNames"]; globalThis["Intl"]["Segmenter"]; globalThis["Intl"]["Locale"]; globalThis["Intl"]["NumberFormat"]; Intl.NumberFormat; Intl.RelativeTimeFormat; Intl.Collator; Intl.DisplayNames; Intl.Segmenter; Intl.Locale;"#,
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

fn assert_build_source_file_rejects_late_weak_reference_apis_in_input(
    api_surface: ApiSurface,
    extension: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        r#"new WeakMap(); globalThis.WeakMap; globalThis["WeakMap"]; new WeakSet(); globalThis.WeakSet; globalThis["WeakSet"]; globalThis.WeakRef; globalThis["WeakRef"]; new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry; globalThis["FinalizationRegistry"];"#,
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
fn build_browser_bundle_result_accepts_cjs_format_through_schema_validation() {
    let value = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "artifacts": [
            { "kind": "wasm-module", "path": "browser.wasm" },
            { "kind": "js-glue", "path": "browser.cjs" },
            { "kind": "source-map", "path": "browser.cjs.map" }
        ],
        "exports": [],
        "bundleFormat": "cjs"
    });

    validate_build_result_value(&value).expect("browser bundle cjs result should validate");
}

#[test]
fn build_library_result_round_trips_through_schema_validation() {
    let value = serde_json::json!({
        "artifactKind": "lib",
        "outputPath": "/workspace/dist/lib",
        "sizeBytes": 42,
        "buildMode": "release",
        "sourceHash": "sha256-deadbeef",
        "profileDataHash": "sha256-feedface",
        "metadataPath": "/workspace/dist/lib/lib.meta.json",
        "witPath": "/workspace/dist/lib/lib.wit",
        "artifacts": [
            { "kind": "wasm-module", "path": "lib.wasm" },
            { "kind": "meta-json", "path": "lib.meta.json" }
        ],
        "exports": [
            { "name": "main", "signature": "(input) => number" }
        ]
    });

    validate_build_result_value(&value).expect("library result should validate");
}

#[test]
fn collect_library_exports_infers_literal_return_types_for_function_declarations_and_aliases() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "export function main(input) { return 1; } export { main as alias };",
    )
    .expect("write source");

    let statements = vec![
        Statement::FunctionDeclaration(kali_ast::FunctionDeclaration {
            name: "main".to_string(),
            params: vec!["input".to_string()],
            body: Box::new(kali_ast::BlockStatement {
                body: vec![Statement::ReturnStatement(kali_ast::ReturnStatement {
                    argument: Some(Expression::Literal(kali_ast::LiteralValue::Number(1.0))),
                })],
            }),
            is_async: false,
            generator: false,
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![kali_ast::ExportSpecifier {
                local: "main".to_string(),
                exported: "alias".to_string(),
            }],
            source: None,
        }),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 2, "exports: {exports:?}");
    assert!(exports
        .iter()
        .any(|export| { export.name == "main" && export.signature == "(input) => number" }));
    assert!(exports
        .iter()
        .any(|export| { export.name == "alias" && export.signature == "(input) => number" }));
}

#[test]
fn collect_library_exports_resolves_named_re_exports_across_source_graph() {
    let dir = tempdir().expect("tempdir");
    let helper_path = dir.path().join("helper.ts");
    let bridge_path = dir.path().join("bridge.ts");

    fs::write(
        &helper_path,
        "export function quadruple(value) { return value + value; }\n",
    )
    .expect("write helper source");
    fs::write(&bridge_path, "export { quadruple } from './helper.ts';\n")
        .expect("write bridge source");

    let exports = collect_library_exports(&bridge_path, ApiSurface::Deno, &[])
        .expect("library exports should resolve through re-exports");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert!(exports
        .iter()
        .any(|export| { export.name == "quadruple" && export.signature == "(value) => unknown" }));
}

#[test]
fn collect_library_exports_infers_const_function_expression_bindings_and_aliases() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const main = (input) => 1; const helper = function(input) { return 2; }; export { main, helper as alias };",
    )
    .expect("write source");

    let statements = vec![
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![
                kali_ast::VariableDeclarator {
                    id: "main".to_string(),
                    init: Some(Expression::ParenthesizedExpression(Box::new(
                        kali_ast::ParenthesizedExpression {
                            expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                                kali_ast::ArrowFunctionExpression {
                                    params: vec![kali_ast::FunctionParam {
                                        name: "input".to_string(),
                                    }],
                                    body: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                                    is_async: false,
                                    returnType: None,
                                },
                            ))),
                        },
                    ))),
                },
                kali_ast::VariableDeclarator {
                    id: "helper".to_string(),
                    init: Some(Expression::FunctionExpression(Box::new(
                        kali_ast::FunctionExpression {
                            id: None,
                            params: vec![kali_ast::FunctionParam {
                                name: "input".to_string(),
                            }],
                            body: Some(Box::new(kali_ast::BlockStatement {
                                body: vec![Statement::ReturnStatement(kali_ast::ReturnStatement {
                                    argument: Some(Expression::Literal(
                                        kali_ast::LiteralValue::Number(2.0),
                                    )),
                                })],
                            })),
                            is_async: false,
                            generator: false,
                        },
                    ))),
                },
            ],
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![
                kali_ast::ExportSpecifier {
                    local: "main".to_string(),
                    exported: "main".to_string(),
                },
                kali_ast::ExportSpecifier {
                    local: "helper".to_string(),
                    exported: "alias".to_string(),
                },
            ],
            source: None,
        }),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 2, "exports: {exports:?}");
    assert!(exports
        .iter()
        .any(|export| { export.name == "main" && export.signature == "(input) => number" }));
    assert!(exports
        .iter()
        .any(|export| { export.name == "alias" && export.signature == "(input) => number" }));
}

#[test]
fn collect_library_exports_infers_async_function_declarations_and_aliases() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "export async function main(input) { return await 1; } export { main as alias };",
    )
    .expect("write source");

    let statements = vec![
        Statement::FunctionDeclaration(kali_ast::FunctionDeclaration {
            name: "main".to_string(),
            params: vec!["input".to_string()],
            body: Box::new(kali_ast::BlockStatement {
                body: vec![Statement::ReturnStatement(kali_ast::ReturnStatement {
                    argument: Some(Expression::AwaitExpression(Box::new(
                        kali_ast::AwaitExpression {
                            argument: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                        },
                    ))),
                })],
            }),
            is_async: true,
            generator: false,
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![kali_ast::ExportSpecifier {
                local: "main".to_string(),
                exported: "alias".to_string(),
            }],
            source: None,
        }),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 2, "exports: {exports:?}");
    assert!(exports.iter().any(|export| {
        export.name == "main" && export.signature == "(input) => Promise<number>"
    }));
    assert!(exports.iter().any(|export| {
        export.name == "alias" && export.signature == "(input) => Promise<number>"
    }));
}

#[test]
fn collect_library_exports_infers_default_async_function_expression_exports_through_await() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default async (input) => await 1;").expect("write source");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::ArrowFunctionExpression(
            Box::new(kali_ast::ArrowFunctionExpression {
                params: vec![kali_ast::FunctionParam {
                    name: "input".to_string(),
                }],
                body: Expression::AwaitExpression(Box::new(kali_ast::AwaitExpression {
                    argument: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                })),
                is_async: true,
                returnType: None,
            }),
        )),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => Promise<number>");
}

#[test]
fn collect_library_exports_infers_default_async_function_expression_exports() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default async (input) => 1;").expect("write source");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::ArrowFunctionExpression(
            Box::new(kali_ast::ArrowFunctionExpression {
                params: vec![kali_ast::FunctionParam {
                    name: "input".to_string(),
                }],
                body: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                is_async: true,
                returnType: None,
            }),
        )),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => Promise<number>");
}

#[test]
fn collect_library_exports_infers_default_async_function_expression_exports_through_chain_wrapper()
{
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default async (input) => (1);").expect("write source");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::ArrowFunctionExpression(
            Box::new(kali_ast::ArrowFunctionExpression {
                params: vec![kali_ast::FunctionParam {
                    name: "input".to_string(),
                }],
                body: Expression::ChainExpression(Box::new(kali_ast::ChainExpression {
                    expression: Box::new(Expression::Literal(kali_ast::LiteralValue::Number(1.0))),
                })),
                is_async: true,
                returnType: None,
            }),
        )),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => Promise<number>");
}

#[test]
fn collect_library_exports_infers_default_async_function_expression_exports_through_decorated_wrapper(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default ((async (input) => 1));").expect("write source");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::DecoratedExpression(
            kali_ast::DecoratedExpression {
                expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                            kali_ast::ArrowFunctionExpression {
                                params: vec![kali_ast::FunctionParam {
                                    name: "input".to_string(),
                                }],
                                body: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                                is_async: true,
                                returnType: None,
                            },
                        ))),
                    },
                ))),
            },
        )),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => Promise<number>");
}

#[test]
fn collect_library_exports_infers_default_function_expression_exports_through_await_wrapper() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default await ((input) => 1);").expect("write source");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::AwaitExpression(Box::new(
            kali_ast::AwaitExpression {
                argument: Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                            kali_ast::ArrowFunctionExpression {
                                params: vec![kali_ast::FunctionParam {
                                    name: "input".to_string(),
                                }],
                                body: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                                is_async: false,
                                returnType: None,
                            },
                        ))),
                    },
                )),
            },
        ))),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_default_async_function_expression_exports_through_await_wrapper()
{
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default await ((async (input) => 1));").expect("write source");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::AwaitExpression(Box::new(
            kali_ast::AwaitExpression {
                argument: Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                            kali_ast::ArrowFunctionExpression {
                                params: vec![kali_ast::FunctionParam {
                                    name: "input".to_string(),
                                }],
                                body: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                                is_async: true,
                                returnType: None,
                            },
                        ))),
                    },
                )),
            },
        ))),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => Promise<number>");
}

#[test]
fn collect_library_exports_infers_default_function_expression_exports_through_decorated_wrapper() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default ((input) => 1);").expect("write source");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::DecoratedExpression(
            kali_ast::DecoratedExpression {
                expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                    kali_ast::ParenthesizedExpression {
                        expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                            kali_ast::ArrowFunctionExpression {
                                params: vec![kali_ast::FunctionParam {
                                    name: "input".to_string(),
                                }],
                                body: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                                is_async: false,
                                returnType: None,
                            },
                        ))),
                    },
                ))),
            },
        )),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_function_binding_signatures_through_await_wrapper() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const main = await ((input) => 1); export { main as alias };",
    )
    .expect("write source");

    let statements = vec![
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![kali_ast::VariableDeclarator {
                id: "main".to_string(),
                init: Some(Expression::AwaitExpression(Box::new(
                    kali_ast::AwaitExpression {
                        argument: Expression::ParenthesizedExpression(Box::new(
                            kali_ast::ParenthesizedExpression {
                                expression: Box::new(Expression::ArrowFunctionExpression(
                                    Box::new(kali_ast::ArrowFunctionExpression {
                                        params: vec![kali_ast::FunctionParam {
                                            name: "input".to_string(),
                                        }],
                                        body: Expression::Literal(kali_ast::LiteralValue::Number(
                                            1.0,
                                        )),
                                        is_async: false,
                                        returnType: None,
                                    }),
                                )),
                            },
                        )),
                    },
                ))),
            }],
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![kali_ast::ExportSpecifier {
                local: "main".to_string(),
                exported: "alias".to_string(),
            }],
            source: None,
        }),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "alias");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_async_function_expression_bindings_and_aliases() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const main = true ? async (input) => await 1 : async (input) => await 1; export { main as alias };",
    )
    .expect("write source");

    let async_function_expression = |value| {
        Expression::ParenthesizedExpression(Box::new(kali_ast::ParenthesizedExpression {
            expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                kali_ast::ArrowFunctionExpression {
                    params: vec![kali_ast::FunctionParam {
                        name: "input".to_string(),
                    }],
                    body: Expression::AwaitExpression(Box::new(kali_ast::AwaitExpression {
                        argument: Expression::Literal(kali_ast::LiteralValue::Number(value)),
                    })),
                    is_async: true,
                    returnType: None,
                },
            ))),
        }))
    };

    let statements = vec![
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![kali_ast::VariableDeclarator {
                id: "main".to_string(),
                init: Some(Expression::ConditionalExpression(Box::new(
                    kali_ast::ConditionalExpression {
                        test: Box::new(Expression::Literal(kali_ast::LiteralValue::Boolean(true))),
                        consequent: Box::new(async_function_expression(1.0)),
                        alternate: Box::new(async_function_expression(1.0)),
                    },
                ))),
            }],
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![
                kali_ast::ExportSpecifier {
                    local: "main".to_string(),
                    exported: "main".to_string(),
                },
                kali_ast::ExportSpecifier {
                    local: "main".to_string(),
                    exported: "alias".to_string(),
                },
            ],
            source: None,
        }),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 2, "exports: {exports:?}");
    assert!(exports.iter().any(|export| {
        export.name == "main" && export.signature == "(input) => Promise<number>"
    }));
    assert!(exports.iter().any(|export| {
        export.name == "alias" && export.signature == "(input) => Promise<number>"
    }));
}

#[test]
fn collect_library_exports_infers_default_function_expression_exports() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default (input) => 1;").expect("write source");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::ParenthesizedExpression(
            Box::new(kali_ast::ParenthesizedExpression {
                expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                    kali_ast::ArrowFunctionExpression {
                        params: vec![kali_ast::FunctionParam {
                            name: "input".to_string(),
                        }],
                        body: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                        is_async: false,
                        returnType: None,
                    },
                ))),
            }),
        )),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_default_function_expression_exports_through_chain_wrapper() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default (input) => 1;").expect("write source");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::ArrowFunctionExpression(
            Box::new(kali_ast::ArrowFunctionExpression {
                params: vec![kali_ast::FunctionParam {
                    name: "input".to_string(),
                }],
                body: Expression::ChainExpression(Box::new(kali_ast::ChainExpression {
                    expression: Box::new(Expression::Literal(kali_ast::LiteralValue::Number(1.0))),
                })),
                is_async: false,
                returnType: None,
            }),
        )),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_default_function_expression_exports_through_optional_chain_wrapper(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "export default (input) => 1;").expect("write source");

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::OptionalChainExpression(
            Box::new(kali_ast::OptionalChainExpression {
                inner: Box::new(kali_ast::OptionalChainInner::NonNull {
                    object: Box::new(Expression::ArrowFunctionExpression(Box::new(
                        kali_ast::ArrowFunctionExpression {
                            params: vec![kali_ast::FunctionParam {
                                name: "input".to_string(),
                            }],
                            body: Expression::OptionalChainExpression(Box::new(
                                kali_ast::OptionalChainExpression {
                                    inner: Box::new(kali_ast::OptionalChainInner::NonNull {
                                        object: Box::new(Expression::Literal(
                                            kali_ast::LiteralValue::Number(1.0),
                                        )),
                                        optional: true,
                                    }),
                                },
                            )),
                            is_async: false,
                            returnType: None,
                        },
                    ))),
                    optional: true,
                }),
            }),
        )),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_function_binding_signatures_through_satisfies_wrapper() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const main = (input) => 1; export { main as alias };",
    )
    .expect("write source");

    let statements = vec![
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![kali_ast::VariableDeclarator {
                id: "main".to_string(),
                init: Some(Expression::SatisfiesExpression(Box::new(
                    kali_ast::SatisfiesExpression {
                        type_name: "unknown".to_string(),
                        expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                            kali_ast::ArrowFunctionExpression {
                                params: vec![kali_ast::FunctionParam {
                                    name: "input".to_string(),
                                }],
                                body: Expression::Literal(kali_ast::LiteralValue::Number(1.0)),
                                is_async: false,
                                returnType: None,
                            },
                        ))),
                    },
                ))),
            }],
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![kali_ast::ExportSpecifier {
                local: "main".to_string(),
                exported: "alias".to_string(),
            }],
            source: None,
        }),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "alias");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_default_function_expression_exports_through_conditional_wrapper()
{
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "export default true ? ((input) => 1) : ((input) => 1);",
    )
    .expect("write source");

    let function_expression = |value| {
        Expression::ParenthesizedExpression(Box::new(kali_ast::ParenthesizedExpression {
            expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                kali_ast::ArrowFunctionExpression {
                    params: vec![kali_ast::FunctionParam {
                        name: "input".to_string(),
                    }],
                    body: Expression::Literal(kali_ast::LiteralValue::Number(value)),
                    is_async: false,
                    returnType: None,
                },
            ))),
        }))
    };

    let statements = vec![Statement::ExportDefault(
        kali_ast::ExportDefaultDeclaration::Expression(Expression::ConditionalExpression(
            Box::new(kali_ast::ConditionalExpression {
                test: Box::new(Expression::Literal(kali_ast::LiteralValue::Boolean(true))),
                consequent: Box::new(function_expression(1.0)),
                alternate: Box::new(function_expression(1.0)),
            }),
        )),
    )];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "default");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_function_binding_signatures_through_optional_chain_wrapper() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const main = (input) => 1;").expect("write source");

    let statements = vec![
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![kali_ast::VariableDeclarator {
                id: "main".to_string(),
                init: Some(Expression::OptionalChainExpression(Box::new(
                    kali_ast::OptionalChainExpression {
                        inner: Box::new(kali_ast::OptionalChainInner::NonNull {
                            object: Box::new(Expression::ArrowFunctionExpression(Box::new(
                                kali_ast::ArrowFunctionExpression {
                                    params: vec![kali_ast::FunctionParam {
                                        name: "input".to_string(),
                                    }],
                                    body: Expression::OptionalChainExpression(Box::new(
                                        kali_ast::OptionalChainExpression {
                                            inner: Box::new(
                                                kali_ast::OptionalChainInner::NonNull {
                                                    object: Box::new(Expression::Literal(
                                                        kali_ast::LiteralValue::Number(1.0),
                                                    )),
                                                    optional: true,
                                                },
                                            ),
                                        },
                                    )),
                                    is_async: false,
                                    returnType: None,
                                },
                            ))),
                            optional: true,
                        }),
                    },
                ))),
            }],
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![kali_ast::ExportSpecifier {
                local: "main".to_string(),
                exported: "alias".to_string(),
            }],
            source: None,
        }),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "alias");
    assert_eq!(exports[0].signature, "(input) => number");
}

#[test]
fn collect_library_exports_infers_function_binding_signatures_through_sequence_and_conditional_wrappers(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const main = 0; const helper = 1;").expect("write source");

    let statements = vec![
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![
                kali_ast::VariableDeclarator {
                    id: "main".to_string(),
                    init: Some(Expression::SequenceExpression(Box::new(
                        kali_ast::SequenceExpression {
                            expressions: vec![
                                Expression::Literal(kali_ast::LiteralValue::Number(0.0)),
                                Expression::ParenthesizedExpression(Box::new(
                                    kali_ast::ParenthesizedExpression {
                                        expression: Box::new(Expression::ArrowFunctionExpression(
                                            Box::new(kali_ast::ArrowFunctionExpression {
                                                params: vec![kali_ast::FunctionParam {
                                                    name: "input".to_string(),
                                                }],
                                                body: Expression::Literal(
                                                    kali_ast::LiteralValue::Number(1.0),
                                                ),
                                                is_async: false,
                                                returnType: None,
                                            }),
                                        )),
                                    },
                                )),
                            ],
                        },
                    ))),
                },
                kali_ast::VariableDeclarator {
                    id: "helper".to_string(),
                    init: Some(Expression::ConditionalExpression(Box::new(
                        kali_ast::ConditionalExpression {
                            test: Box::new(Expression::Literal(kali_ast::LiteralValue::Boolean(
                                true,
                            ))),
                            consequent: Box::new(Expression::ParenthesizedExpression(Box::new(
                                kali_ast::ParenthesizedExpression {
                                    expression: Box::new(Expression::FunctionExpression(Box::new(
                                        kali_ast::FunctionExpression {
                                            id: None,
                                            params: vec![kali_ast::FunctionParam {
                                                name: "input".to_string(),
                                            }],
                                            body: Some(Box::new(kali_ast::BlockStatement {
                                                body: vec![Statement::ReturnStatement(
                                                    kali_ast::ReturnStatement {
                                                        argument: Some(Expression::Literal(
                                                            kali_ast::LiteralValue::Number(2.0),
                                                        )),
                                                    },
                                                )],
                                            })),
                                            is_async: false,
                                            generator: false,
                                        },
                                    ))),
                                },
                            ))),
                            alternate: Box::new(Expression::ParenthesizedExpression(Box::new(
                                kali_ast::ParenthesizedExpression {
                                    expression: Box::new(Expression::ArrowFunctionExpression(
                                        Box::new(kali_ast::ArrowFunctionExpression {
                                            params: vec![kali_ast::FunctionParam {
                                                name: "input".to_string(),
                                            }],
                                            body: Expression::Literal(
                                                kali_ast::LiteralValue::Number(2.0),
                                            ),
                                            is_async: false,
                                            returnType: None,
                                        }),
                                    )),
                                },
                            ))),
                        },
                    ))),
                },
            ],
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![
                kali_ast::ExportSpecifier {
                    local: "main".to_string(),
                    exported: "main".to_string(),
                },
                kali_ast::ExportSpecifier {
                    local: "helper".to_string(),
                    exported: "alias".to_string(),
                },
            ],
            source: None,
        }),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 2, "exports: {exports:?}");
    assert!(exports
        .iter()
        .any(|export| { export.name == "main" && export.signature == "(input) => number" }));
    assert!(exports
        .iter()
        .any(|export| { export.name == "alias" && export.signature == "(input) => number" }));
}

#[test]
fn collect_library_exports_infers_function_binding_signatures_through_decorated_wrappers() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, "const main = 0; export { main as alias };").expect("write source");

    let statements = vec![
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![kali_ast::VariableDeclarator {
                id: "main".to_string(),
                init: Some(Expression::DecoratedExpression(
                    kali_ast::DecoratedExpression {
                        expression: Box::new(Expression::ParenthesizedExpression(Box::new(
                            kali_ast::ParenthesizedExpression {
                                expression: Box::new(Expression::ArrowFunctionExpression(
                                    Box::new(kali_ast::ArrowFunctionExpression {
                                        params: vec![kali_ast::FunctionParam {
                                            name: "input".to_string(),
                                        }],
                                        body: Expression::DecoratedExpression(
                                            kali_ast::DecoratedExpression {
                                                expression: Box::new(Expression::Literal(
                                                    kali_ast::LiteralValue::Number(1.0),
                                                )),
                                            },
                                        ),
                                        is_async: false,
                                        returnType: None,
                                    }),
                                )),
                            },
                        ))),
                    },
                )),
            }],
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![kali_ast::ExportSpecifier {
                local: "main".to_string(),
                exported: "alias".to_string(),
            }],
            source: None,
        }),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert!(exports
        .iter()
        .any(|export| { export.name == "alias" && export.signature == "(input) => number" }));
}

#[test]
fn collect_library_exports_preserves_unknown_signature_for_mixed_conditional_binding_exports() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(
        &source_path,
        "const main = true ? ((input) => 1) : ((input, extra) => 2); export { main as alias };",
    )
    .expect("write source");

    let function_expression = |params: Vec<&str>, value: f64| {
        Expression::ParenthesizedExpression(Box::new(kali_ast::ParenthesizedExpression {
            expression: Box::new(Expression::ArrowFunctionExpression(Box::new(
                kali_ast::ArrowFunctionExpression {
                    params: params
                        .into_iter()
                        .map(|name| kali_ast::FunctionParam {
                            name: name.to_string(),
                        })
                        .collect(),
                    body: Expression::Literal(kali_ast::LiteralValue::Number(value)),
                    is_async: false,
                    returnType: None,
                },
            ))),
        }))
    };

    let statements = vec![
        Statement::VariableDeclaration(kali_ast::VariableDeclaration {
            kind: "const".to_string(),
            declarations: vec![kali_ast::VariableDeclarator {
                id: "main".to_string(),
                init: Some(Expression::ConditionalExpression(Box::new(
                    kali_ast::ConditionalExpression {
                        test: Box::new(Expression::Literal(kali_ast::LiteralValue::Boolean(true))),
                        consequent: Box::new(function_expression(vec!["input"], 1.0)),
                        alternate: Box::new(function_expression(vec!["input", "extra"], 2.0)),
                    },
                ))),
            }],
        }),
        Statement::ExportNamed(kali_ast::ExportNamedDeclaration {
            specifiers: vec![kali_ast::ExportSpecifier {
                local: "main".to_string(),
                exported: "alias".to_string(),
            }],
            source: None,
        }),
    ];

    let exports = collect_library_exports_from_statements(&statements, &source_path)
        .expect("library exports should collect");

    assert_eq!(exports.len(), 1, "exports: {exports:?}");
    assert_eq!(exports[0].name, "alias");
    assert_eq!(exports[0].signature, "(main) => unknown");
}

#[test]
fn collect_direct_bundle_calls_from_statements_peels_transparent_call_wrappers() {
    let candidate_names = ["helper".to_string(), "sequence_helper".to_string()]
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();

    let statements = vec![
        Statement::ReturnStatement(kali_ast::ReturnStatement {
            argument: Some(Expression::CallExpression(Box::new(
                kali_ast::CallExpression {
                    callee: Expression::ParenthesizedExpression(Box::new(
                        kali_ast::ParenthesizedExpression {
                            expression: Box::new(Expression::Identifier("helper".to_string())),
                        },
                    )),
                    args: vec![],
                },
            ))),
        }),
        Statement::ExpressionStatement(kali_ast::ExpressionStatement {
            expression: Box::new(Expression::CallExpression(Box::new(
                kali_ast::CallExpression {
                    callee: Expression::SequenceExpression(Box::new(
                        kali_ast::SequenceExpression {
                            expressions: vec![
                                Expression::Identifier("ignored".to_string()),
                                Expression::Identifier("sequence_helper".to_string()),
                            ],
                        },
                    )),
                    args: vec![],
                },
            ))),
        }),
    ];

    let calls = collect_direct_bundle_calls_from_statements(&statements, &candidate_names);

    assert_eq!(
        calls,
        ["helper".to_string(), "sequence_helper".to_string()]
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>()
    );
}

#[test]
fn build_capi_result_round_trips_through_schema_validation() {
    let value = serde_json::json!({
        "artifactKind": "capi",
        "outputPath": "/workspace/dist/capi",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "profileDataHash": "sha256-feedface",
        "metadataPath": "/workspace/dist/capi/capi.meta.json",
        "witPath": "/workspace/dist/capi/capi.wit",
        "headerPath": "/workspace/dist/capi/capi.h",
        "artifacts": [
            { "kind": "wasm-module", "path": "capi.wasm" },
            { "kind": "meta-json", "path": "capi.meta.json" },
            { "kind": "header", "path": "capi.h" }
        ],
        "exports": []
    });

    validate_build_result_value(&value).expect("capi result should validate");
}

#[test]
fn build_component_result_accepts_artifact_roles_through_schema_validation() {
    let value = serde_json::json!({
        "artifactKind": "component",
        "outputPath": "/workspace/dist/component",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "metadataPath": "/workspace/dist/component/component.meta.json",
        "witPath": "/workspace/dist/component/component.wit",
        "bindingPackagePath": "/workspace/dist/component/component.binding-package.json",
        "artifacts": [
            { "kind": "wasm-component", "path": "component.wasm", "role": "primary-component" },
            { "kind": "wit", "path": "component.wit", "role": "interface-wit" },
            { "kind": "meta-json", "path": "component.meta.json", "role": "embedding-metadata" },
            { "kind": "binding-package", "path": "component.binding-package.json", "role": "binding-package-manifest" }
        ],
        "exports": []
    });

    validate_build_result_value(&value)
        .expect("component result with artifact roles should validate");
}

#[test]
fn build_result_variants_accept_artifact_roles_through_schema_validation() {
    let values = [
        serde_json::json!({
            "artifactKind": "lib",
            "outputPath": "/workspace/dist/lib",
            "sizeBytes": 42,
            "buildMode": "release",
            "sourceHash": "sha256-deadbeef",
            "profileDataHash": "sha256-feedface",
            "metadataPath": "/workspace/dist/lib/lib.meta.json",
            "witPath": "/workspace/dist/lib/lib.wit",
            "artifacts": [
                { "kind": "wasm-module", "path": "lib.wasm", "role": "primary-module" },
                { "kind": "meta-json", "path": "lib.meta.json", "role": "metadata" }
            ],
            "exports": [
                { "name": "main", "signature": "(input) => number" }
            ]
        }),
        serde_json::json!({
            "artifactKind": "bundle",
            "outputPath": "/workspace/dist/browser",
            "sizeBytes": 42,
            "buildMode": "release-advanced",
            "sourceHash": "sha256-deadbeef",
            "artifacts": [
                { "kind": "wasm-module", "path": "browser.wasm", "role": "bundle-module" },
                { "kind": "js-glue", "path": "browser.js", "role": "browser-glue" }
            ],
            "exports": [],
            "bundleFormat": "esm"
        }),
        serde_json::json!({
            "artifactKind": "capi",
            "outputPath": "/workspace/dist/capi",
            "sizeBytes": 42,
            "buildMode": "release-advanced",
            "sourceHash": "sha256-deadbeef",
            "profileDataHash": "sha256-feedface",
            "metadataPath": "/workspace/dist/capi/capi.meta.json",
            "witPath": "/workspace/dist/capi/capi.wit",
            "headerPath": "/workspace/dist/capi/capi.h",
            "artifacts": [
                { "kind": "wasm-module", "path": "capi.wasm", "role": "primary-module" },
                { "kind": "meta-json", "path": "capi.meta.json", "role": "metadata" },
                { "kind": "header", "path": "capi.h", "role": "header" }
            ],
            "exports": []
        }),
    ];

    for value in values {
        validate_build_result_value(&value)
            .expect("build result variant with artifact roles should validate");
    }
}

#[test]
fn validate_build_result_value_rejects_duplicate_primary_artifact_roles() {
    let invalid_bundle = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "artifacts": [
            { "kind": "wasm-module", "path": "browser.wasm", "role": "primary-executable" },
            { "kind": "wasm-module", "path": "browser-shadow.wasm", "role": "primary-executable" },
            { "kind": "js-glue", "path": "browser.js", "role": "browser-glue" }
        ],
        "exports": [],
        "bundleFormat": "esm"
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("duplicate primary artifact roles should fail validation");
    assert!(
        err.contains("primary-executable"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_build_result_value_rejects_duplicate_artifact_kind_path_pairs() {
    let invalid_bundle = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "artifacts": [
            { "kind": "wasm-module", "path": "browser.wasm" },
            { "kind": "wasm-module", "path": "browser.wasm" },
            { "kind": "js-glue", "path": "browser.js" }
        ],
        "exports": [],
        "bundleFormat": "esm"
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("duplicate artifact kind/path pairs should fail validation");
    assert!(
        err.contains("duplicates artifact `wasm-module` at `browser.wasm`"),
        "unexpected error: {err}"
    );
}

#[test]
fn validate_build_result_value_rejects_non_string_artifact_roles() {
    let invalid_component = serde_json::json!({
        "artifactKind": "component",
        "outputPath": "/workspace/dist/component",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "metadataPath": "/workspace/dist/component/component.meta.json",
        "witPath": "/workspace/dist/component/component.wit",
        "bindingPackagePath": "/workspace/dist/component/component.binding-package.json",
        "artifacts": [
            { "kind": "wasm-component", "path": "component.wasm", "role": 1 },
            { "kind": "wit", "path": "component.wit", "role": "interface-wit" }
        ],
        "exports": []
    });

    let err = validate_build_result_value(&invalid_component)
        .expect_err("non-string artifact roles should fail validation");
    assert!(err.contains("role"), "unexpected error: {err}");
}

#[test]
fn validate_build_result_value_rejects_fractional_size_bytes() {
    let invalid_bundle = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42.5,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "artifacts": [
            { "kind": "wasm-module", "path": "browser.wasm" },
            { "kind": "js-glue", "path": "browser.js" }
        ],
        "exports": [],
        "bundleFormat": "esm"
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("fractional build result sizeBytes should fail validation");
    assert!(err.contains("sizeBytes"), "unexpected error: {err}");
}

#[test]
fn validate_artifact_metadata_value_rejects_unexpected_top_level_keys() {
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
        "exports": [],
        "unexpected": true
    });

    let err = validate_artifact_metadata_value(&invalid_metadata)
        .expect_err("unexpected artifact metadata keys should fail validation");
    assert!(err.contains("unexpected key"), "unexpected error: {err}");
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
fn validate_artifact_metadata_value_rejects_duplicate_export_names() {
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
            {"name": "main", "signature": "(input) => number"},
            {"name": "main", "signature": "(input) => number"}
        ]
    });

    let err = validate_artifact_metadata_value(&invalid_metadata)
        .expect_err("duplicate export names should fail validation");
    assert!(err.contains("duplicates `main`"), "unexpected error: {err}");
}

#[test]
fn validate_artifact_metadata_value_rejects_invalid_optional_provenance_fields() {
    for (field, invalid_metadata) in [
        (
            "profileDataHash",
            serde_json::json!({
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
                "profileDataHash": 1
            }),
        ),
        (
            "runtimeProfiles[1]",
            serde_json::json!({
                "schemaVersion": 1,
                "artifactKind": "component",
                "entrypoint": "src/main.ts",
                "buildMode": "release",
                "apiSurface": "browser",
                "runtimeProfiles": ["wasm-threads", 1],
                "maxSpecializations": 24,
                "hostContract": "kali-hosted",
                "runtimeBackend": "wasmtime",
                "kaliVersion": "1.2.3",
                "sourceHash": "sha256-deadbeef"
            }),
        ),
        (
            "maxSpecializations",
            serde_json::json!({
                "schemaVersion": 1,
                "artifactKind": "component",
                "entrypoint": "src/main.ts",
                "buildMode": "release",
                "apiSurface": "browser",
                "runtimeProfiles": ["wasm-threads"],
                "maxSpecializations": 1.5,
                "hostContract": "kali-hosted",
                "runtimeBackend": "wasmtime",
                "kaliVersion": "1.2.3",
                "sourceHash": "sha256-deadbeef"
            }),
        ),
    ] {
        let err = validate_artifact_metadata_value(&invalid_metadata)
            .expect_err("invalid artifact metadata field should fail validation");
        assert!(err.contains(field), "unexpected error: {err}");
    }
}

#[test]
fn validate_artifact_metadata_value_rejects_duplicate_runtime_profiles() {
    let invalid_metadata = serde_json::json!({
        "schemaVersion": 1,
        "artifactKind": "component",
        "entrypoint": "src/main.ts",
        "buildMode": "release",
        "apiSurface": "browser",
        "runtimeProfiles": ["wasm-threads", "wasm-threads"],
        "maxSpecializations": 24,
        "hostContract": "kali-hosted",
        "runtimeBackend": "wasmtime",
        "kaliVersion": "1.2.3",
        "sourceHash": "sha256-deadbeef"
    });

    let err = validate_artifact_metadata_value(&invalid_metadata)
        .expect_err("duplicate runtime profiles should fail validation");
    assert!(err.contains("runtimeProfiles"), "unexpected error: {err}");
}

#[test]
fn validate_build_result_value_rejects_unexpected_top_level_keys() {
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
        "bundleFormat": "esm",
        "unexpected": true
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("unexpected build result keys should fail validation");
    assert!(err.contains("unexpected key"), "unexpected error: {err}");
}

#[test]
fn validate_build_result_value_rejects_unexpected_artifact_keys() {
    let invalid_bundle = serde_json::json!({
        "artifactKind": "bundle",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
        "artifacts": [
            { "kind": "wasm-module", "path": "browser.wasm", "extra": true },
            { "kind": "js-glue", "path": "browser.js" }
        ],
        "exports": [],
        "bundleFormat": "umd"
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("unexpected artifact keys should fail validation");
    assert!(err.contains("artifacts[0]"), "unexpected error: {err}");
}

#[test]
fn validate_build_result_value_rejects_duplicate_export_names() {
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
        "exports": [
            { "name": "main", "signature": "(input) => number" },
            { "name": "main", "signature": "(input) => number" }
        ],
        "bundleFormat": "esm"
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("duplicate export names should fail validation");
    assert!(err.contains("duplicates `main`"), "unexpected error: {err}");
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
fn validate_build_result_value_rejects_non_string_bundle_format() {
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
        "bundleFormat": 1
    });

    let err = validate_build_result_value(&invalid_bundle)
        .expect_err("non-string bundleFormat should fail validation");
    assert!(err.contains("bundleFormat"), "unexpected error: {err}");
}

#[test]
fn validate_build_result_value_rejects_unsupported_artifact_kind() {
    let invalid_result = serde_json::json!({
        "artifactKind": "meta-json",
        "outputPath": "/workspace/dist/browser",
        "sizeBytes": 42,
        "buildMode": "release-advanced",
        "sourceHash": "sha256-deadbeef",
    });

    let err = validate_build_result_value(&invalid_result)
        .expect_err("unsupported build result artifactKind should fail validation");
    assert!(err.contains("artifactKind"), "unexpected error: {err}");
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
fn discover_dynamic_import_targets_resolves_template_literal_dynamic_import_chunks() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    let lazy_path = dir.path().join("lazy.ts");
    fs::write(&lazy_path, "export const lazy = true;").expect("write lazy chunk");
    fs::write(
        &source_path,
        "const name = \"lazy.ts\"; const lazy = import(`./${name}`);",
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

#[test]
fn build_source_file_supports_exponent_assignment_on_mutable_binding_in_js_input() {
    assert_build_source_file_supports_exponent_assignment_on_mutable_binding_in_input(
        ApiSurface::Deno,
        "js",
    );
}

#[test]
fn build_source_file_supports_exponent_assignment_on_mutable_binding_in_ts_input() {
    assert_build_source_file_supports_exponent_assignment_on_mutable_binding_in_input(
        ApiSurface::Deno,
        "ts",
    );
}

#[test]
fn build_source_file_supports_exponent_assignment_on_mutable_binding_in_browser_api_surface_in_js_input(
) {
    assert_build_source_file_supports_exponent_assignment_on_mutable_binding_in_input(
        ApiSurface::Browser,
        "js",
    );
}

#[test]
fn build_source_file_supports_exponent_assignment_on_mutable_binding_in_browser_api_surface_in_ts_input(
) {
    assert_build_source_file_supports_exponent_assignment_on_mutable_binding_in_input(
        ApiSurface::Browser,
        "ts",
    );
}
