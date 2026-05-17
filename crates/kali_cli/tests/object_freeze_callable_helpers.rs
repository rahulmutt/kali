use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn frozen_callable_helpers_source() -> &'static str {
    "const object = { a: 1 }; console.log(Object.freeze(globalThis[\"Object\"][\"hasOwn\"])(object, \"a\")); console.log(Object.freeze(globalThis[\"Math\"][\"floor\"])(1.6));\n"
}

fn frozen_callable_helpers_test_source() -> &'static str {
    "Kali.test('freeze-wrapped callable helpers', () => { const object = { a: 1 }; console.log(Object.freeze(globalThis[\"Object\"][\"hasOwn\"])(object, \"a\")); console.log(Object.freeze(globalThis[\"Math\"][\"floor\"])(1.6)); });\n"
}

fn assert_run_supports_frozen_callable_helpers_in_input(extension: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, frozen_callable_helpers_source()).expect("write source");

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

    if json_output {
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "run");
        assert_eq!(json["success"], true);
        assert_eq!(json["stdout"], "1\n1\n");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "1\n1\n", "stdout: {stdout}");
    }
}

fn assert_test_supports_frozen_callable_helpers_in_input(extension: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("smoke.test.{extension}"));
    fs::write(&source_path, frozen_callable_helpers_test_source()).expect("write source");

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

    if json_output {
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "test");
        assert_eq!(json["success"], true);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("1\n1\n"), "stdout: {stdout}");
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
}

#[test]
fn run_supports_frozen_callable_helpers_in_js_input() {
    assert_run_supports_frozen_callable_helpers_in_input("js", false);
}

#[test]
fn json_run_supports_frozen_callable_helpers_in_js_input() {
    assert_run_supports_frozen_callable_helpers_in_input("js", true);
}

#[test]
fn run_supports_frozen_callable_helpers_in_ts_jsx_and_tsx_input() {
    for extension in ["ts", "jsx", "tsx"] {
        assert_run_supports_frozen_callable_helpers_in_input(extension, false);
    }
}

#[test]
fn json_run_supports_frozen_callable_helpers_in_ts_jsx_and_tsx_input() {
    for extension in ["ts", "jsx", "tsx"] {
        assert_run_supports_frozen_callable_helpers_in_input(extension, true);
    }
}

#[test]
fn test_supports_frozen_callable_helpers_in_js_input() {
    assert_test_supports_frozen_callable_helpers_in_input("js", false);
}

#[test]
fn json_test_supports_frozen_callable_helpers_in_js_input() {
    assert_test_supports_frozen_callable_helpers_in_input("js", true);
}

#[test]
fn test_supports_frozen_callable_helpers_in_ts_jsx_and_tsx_input() {
    for extension in ["ts", "jsx", "tsx"] {
        assert_test_supports_frozen_callable_helpers_in_input(extension, false);
    }
}

#[test]
fn json_test_supports_frozen_callable_helpers_in_ts_jsx_and_tsx_input() {
    for extension in ["ts", "jsx", "tsx"] {
        assert_test_supports_frozen_callable_helpers_in_input(extension, true);
    }
}
