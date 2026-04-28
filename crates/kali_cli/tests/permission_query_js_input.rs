use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn unsupported_permission_query_source() -> &'static str {
    "Deno.permissions.query({ name: \"ffi\" });\nDeno.permissions.query({ name: \"sys\" });\nglobalThis[\"Deno\"][\"permissions\"][\"query\"]({ name: \"ffi\" });\nglobalThis[\"Deno\"][\"permissions\"][\"query\"]({ name: \"sys\" });"
}

fn supported_permission_query_const_binding_source() -> &'static str {
    "const read_descriptor = \"read\";\nconst write_descriptor = \"write\";\nconst env_descriptor = \"env\";\nconst net_descriptor = \"net\";\nDeno.permissions.query({ name: read_descriptor });\nDeno.permissions[\"query\"]({ name: read_descriptor });\nDeno.permissions.query({ name: write_descriptor });\nDeno.permissions[\"query\"]({ name: write_descriptor });\nDeno.permissions.query({ name: env_descriptor });\nDeno.permissions[\"query\"]({ name: env_descriptor });\nDeno.permissions.query({ name: net_descriptor });\nDeno.permissions[\"query\"]({ name: net_descriptor });\nglobalThis[\"Deno\"][\"permissions\"].query({ name: read_descriptor });\nglobalThis[\"Deno\"][\"permissions\"][\"query\"]({ name: read_descriptor });\nglobalThis[\"Deno\"][\"permissions\"].query({ name: write_descriptor });\nglobalThis[\"Deno\"][\"permissions\"][\"query\"]({ name: write_descriptor });\nglobalThis[\"Deno\"][\"permissions\"].query({ name: env_descriptor });\nglobalThis[\"Deno\"][\"permissions\"][\"query\"]({ name: env_descriptor });\nglobalThis[\"Deno\"][\"permissions\"].query({ name: net_descriptor });\nglobalThis[\"Deno\"][\"permissions\"][\"query\"]({ name: net_descriptor });"
}

fn supported_permission_query_runtime_source() -> String {
    format!(
        "async function main() {{\n{}\n  console.log('permission query const bindings ok');\n}}\nmain();\n",
        supported_permission_query_const_binding_source()
    )
}

fn assert_unsupported_permission_query_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("permission query descriptor 'ffi'"),
        "stderr: {stderr}"
    );
    assert!(
        stderr.contains("permission query descriptor 'sys'"),
        "stderr: {stderr}"
    );
}

fn assert_unsupported_permission_query_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().all(|error| error["code"] == "E5506"),
        "unexpected errors: {errors:?}"
    );
    assert!(errors.iter().any(|error| {
        error["message"]
            .as_str()
            .expect("error message")
            .contains("permission query descriptor 'ffi'")
    }));
    assert!(errors.iter().any(|error| {
        error["message"]
            .as_str()
            .expect("error message")
            .contains("permission query descriptor 'sys'")
    }));
}

#[test]
fn check_accepts_supported_permission_query_descriptor_const_bindings_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        supported_permission_query_const_binding_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn build_accepts_supported_permission_query_descriptor_const_bindings_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        supported_permission_query_const_binding_source(),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Built executable artifact at"),
        "stdout: {stdout}"
    );
    assert!(
        source_path.with_file_name("main.wasm").exists(),
        "expected build artifact"
    );
}

#[test]
fn run_accepts_supported_permission_query_descriptor_const_bindings_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, supported_permission_query_runtime_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("permission query const bindings ok"),
        "stdout: {stdout}"
    );
}

#[test]
fn json_run_accepts_supported_permission_query_descriptor_const_bindings_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, supported_permission_query_runtime_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert!(
        json["stdout"]
            .as_str()
            .expect("run stdout")
            .contains("permission query const bindings ok"),
        "json: {json}"
    );
}

#[test]
fn check_rejects_unsupported_permission_query_descriptor_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, unsupported_permission_query_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_unsupported_permission_query_rejection(&stderr);
}

#[test]
fn check_rejects_unsupported_permission_query_descriptor_in_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, unsupported_permission_query_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_unsupported_permission_query_rejection_json(errors);
}

#[test]
fn build_rejects_unsupported_permission_query_descriptor_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, unsupported_permission_query_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_unsupported_permission_query_rejection(&stderr);
}

#[test]
fn build_rejects_unsupported_permission_query_descriptor_in_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, unsupported_permission_query_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_unsupported_permission_query_rejection_json(errors);
}
