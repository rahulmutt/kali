use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn unsupported_permission_query_source() -> &'static str {
    "Deno.permissions.query({ name: \"ffi\" });\nDeno.permissions.query({ name: \"sys\" });\nDeno.permissions[\"query\"]({ name: \"ffi\" });\nDeno.permissions[\"query\"]({ name: \"sys\" });\nDeno[\"permissions\"].query({ name: \"ffi\" });\nDeno[\"permissions\"].query({ name: \"sys\" });\nDeno[\"permissions\"][\"query\"]({ name: \"ffi\" });\nDeno[\"permissions\"][\"query\"]({ name: \"sys\" });\nglobalThis.Deno.permissions.query({ name: \"ffi\" });\nglobalThis.Deno.permissions.query({ name: \"sys\" });\nglobalThis.Deno.permissions[\"query\"]({ name: \"ffi\" });\nglobalThis.Deno.permissions[\"query\"]({ name: \"sys\" });\nglobalThis.Deno[\"permissions\"].query({ name: \"ffi\" });\nglobalThis.Deno[\"permissions\"].query({ name: \"sys\" });\nglobalThis.Deno[\"permissions\"][\"query\"]({ name: \"ffi\" });\nglobalThis.Deno[\"permissions\"][\"query\"]({ name: \"sys\" });\nglobalThis[\"Deno\"][\"permissions\"].query({ name: \"ffi\" });\nglobalThis[\"Deno\"][\"permissions\"].query({ name: \"sys\" });\nglobalThis[\"Deno\"][\"permissions\"][\"query\"]({ name: \"ffi\" });\nglobalThis[\"Deno\"][\"permissions\"][\"query\"]({ name: \"sys\" });"
}

fn supported_permission_query_const_binding_source() -> &'static str {
    "const read_descriptor = \"read\";\nconst write_descriptor = \"write\";\nconst env_descriptor = \"env\";\nconst net_descriptor = \"net\";\nDeno.permissions.query(({ name: read_descriptor }));\nDeno.permissions[\"query\"]({ name: read_descriptor });\nDeno.permissions.query(({ name: write_descriptor }));\nDeno.permissions[\"query\"]({ name: write_descriptor });\nDeno.permissions.query(({ name: env_descriptor }));\nDeno.permissions[\"query\"]({ name: env_descriptor });\nDeno.permissions.query(({ name: net_descriptor }));\nDeno.permissions[\"query\"]({ name: net_descriptor });\nglobalThis.Deno.permissions.query(({ name: read_descriptor }));\nglobalThis.Deno.permissions[\"query\"]({ name: read_descriptor });\nglobalThis.Deno.permissions.query(({ name: write_descriptor }));\nglobalThis.Deno.permissions[\"query\"]({ name: write_descriptor });\nglobalThis.Deno.permissions.query(({ name: env_descriptor }));\nglobalThis.Deno.permissions[\"query\"]({ name: env_descriptor });\nglobalThis.Deno.permissions.query(({ name: net_descriptor }));\nglobalThis.Deno.permissions[\"query\"]({ name: net_descriptor });\nglobalThis[\"Deno\"][\"permissions\"].query(({ name: read_descriptor }));\nglobalThis[\"Deno\"][\"permissions\"][\"query\"]({ name: read_descriptor });\nglobalThis[\"Deno\"][\"permissions\"].query(({ name: write_descriptor }));\nglobalThis[\"Deno\"][\"permissions\"][\"query\"]({ name: write_descriptor });\nglobalThis[\"Deno\"][\"permissions\"].query(({ name: env_descriptor }));\nglobalThis[\"Deno\"][\"permissions\"][\"query\"]({ name: env_descriptor });\nglobalThis[\"Deno\"][\"permissions\"].query(({ name: net_descriptor }));\nglobalThis[\"Deno\"][\"permissions\"][\"query\"]({ name: net_descriptor });"
}

fn supported_permission_query_runtime_source() -> String {
    format!(
        "async function main() {{\n{}\n  console.log('permission query const bindings ok');\n}}\nmain();\n",
        supported_permission_query_const_binding_source()
    )
}

fn supported_permission_query_test_source() -> String {
    r#"async function main() {
const read_descriptor = "read";
const write_descriptor = "write";
const env_descriptor = "env";
const net_descriptor = "net";
await Deno.permissions.query(({ name: read_descriptor }));
await Deno.permissions.query(({ name: write_descriptor }));
await Deno.permissions.query(({ name: env_descriptor }));
await Deno.permissions.query(({ name: net_descriptor }));
await Deno.permissions["query"]({ name: read_descriptor });
await Deno.permissions["query"]({ name: write_descriptor });
await Deno.permissions["query"]({ name: env_descriptor });
await Deno.permissions["query"]({ name: net_descriptor });
await globalThis["Deno"]["permissions"].query(({ name: read_descriptor }));
await globalThis["Deno"]["permissions"]["query"]({ name: read_descriptor });
await globalThis["Deno"]["permissions"].query(({ name: write_descriptor }));
await globalThis["Deno"]["permissions"]["query"]({ name: write_descriptor });
await globalThis["Deno"]["permissions"].query(({ name: env_descriptor }));
await globalThis["Deno"]["permissions"]["query"]({ name: env_descriptor });
await globalThis["Deno"]["permissions"].query(({ name: net_descriptor }));
await globalThis["Deno"]["permissions"]["query"]({ name: net_descriptor });
}
Kali.test('permission query const bindings', () => main());
"#
    .to_string()
}

fn assert_supported_permission_query_output(stdout: &str) {
    assert!(
        stdout.contains("permission query const bindings ok"),
        "stdout: {stdout}"
    );
}

fn assert_supported_permission_query_json_output(output: &std::process::Output, command: &str) {
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    if command == "run" {
        assert!(
            json["stdout"]
                .as_str()
                .expect("run stdout")
                .contains("permission query const bindings ok"),
            "json: {json}"
        );
    } else {
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert_eq!(json["payload"]["skipped"], 0);
        assert_eq!(json["stdout"], "");
    }
    assert_eq!(json["stderr"], "");
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

fn run_supported_permission_query_command(extension: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        supported_permission_query_const_binding_source(),
    )
    .expect("write source");

    let mut command = Command::new(kali_bin());
    command.current_dir(dir.path());
    if json_output {
        command.arg("--output").arg("json");
    }
    let output = command
        .arg("check")
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
    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "check");
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
        assert_eq!(json["errors"].as_array().expect("errors array").len(), 0);
    }
}

fn run_supported_permission_query_build(extension: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(
        &source_path,
        supported_permission_query_const_binding_source(),
    )
    .expect("write source");

    let mut command = Command::new(kali_bin());
    command.current_dir(dir.path());
    if json_output {
        command.arg("--output").arg("json");
    }
    let output = command
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
    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "build");
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
        assert!(json["errors"].as_array().expect("errors array").is_empty());
        assert_eq!(json["payload"]["artifactKind"], "executable");
    } else {
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
}

fn run_supported_permission_query_runtime(extension: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, supported_permission_query_runtime_source()).expect("write source");

    let mut command = Command::new(kali_bin());
    command.current_dir(dir.path());
    if json_output {
        command.arg("--output").arg("json");
    }
    let output = command
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
    if json_output {
        assert_supported_permission_query_json_output(&output, "run");
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_supported_permission_query_output(&stdout);
    }
}

fn run_supported_permission_query_test(extension: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.test.{extension}"));
    fs::write(&source_path, supported_permission_query_test_source()).expect("write source");

    let mut command = Command::new(kali_bin());
    command.current_dir(dir.path());
    if json_output {
        command.arg("--output").arg("json");
    }
    let output = command
        .arg("test")
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
    if json_output {
        assert_supported_permission_query_json_output(&output, "test");
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
}

fn run_unsupported_permission_query_command(command: &str, extension: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, unsupported_permission_query_source()).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path());
    if json_output {
        cli.arg("--output").arg("json");
    }
    let output = cli
        .arg(command)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        let errors = json["errors"].as_array().expect("errors array");
        assert_unsupported_permission_query_rejection_json(errors);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_unsupported_permission_query_rejection(&stderr);
    }
}

#[test]
fn check_accepts_supported_permission_query_descriptor_const_bindings_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        run_supported_permission_query_command(extension, false);
    }
}

#[test]
fn build_accepts_supported_permission_query_descriptor_const_bindings_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        run_supported_permission_query_build(extension, false);
    }
}

#[test]
fn run_accepts_supported_permission_query_descriptor_const_bindings_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        run_supported_permission_query_runtime(extension, false);
    }
}

#[test]
fn json_run_accepts_supported_permission_query_descriptor_const_bindings_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        run_supported_permission_query_runtime(extension, true);
    }
}

#[test]
fn test_accepts_supported_permission_query_descriptor_const_bindings_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        run_supported_permission_query_test(extension, false);
    }
}

#[test]
fn json_test_accepts_supported_permission_query_descriptor_const_bindings_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        run_supported_permission_query_test(extension, true);
    }
}

#[test]
fn check_rejects_unsupported_permission_query_descriptor_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        run_unsupported_permission_query_command("check", extension, false);
    }
}

#[test]
fn check_rejects_unsupported_permission_query_descriptor_in_jsx_and_tsx_input_in_json() {
    for extension in ["jsx", "tsx"] {
        run_unsupported_permission_query_command("check", extension, true);
    }
}

#[test]
fn build_rejects_unsupported_permission_query_descriptor_in_jsx_and_tsx_input() {
    for extension in ["jsx", "tsx"] {
        run_unsupported_permission_query_command("build", extension, false);
    }
}

#[test]
fn build_rejects_unsupported_permission_query_descriptor_in_jsx_and_tsx_input_in_json() {
    for extension in ["jsx", "tsx"] {
        run_unsupported_permission_query_command("build", extension, true);
    }
}
