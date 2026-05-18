use std::{fs, process::Command};

use kali_common::{
    math_floor_trunc_ceil_frozen_callable_aliases, object_has_own_frozen_callable_aliases,
};
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn object_has_own_frozen_callable_invocations_source() -> String {
    object_has_own_frozen_callable_aliases()
        .iter()
        .map(|alias| format!(r#"console.log({alias}(object, "a"));"#))
        .collect::<Vec<_>>()
        .join(" ")
}

fn math_floor_trunc_ceil_frozen_callable_invocations_source() -> String {
    math_floor_trunc_ceil_frozen_callable_aliases()
        .iter()
        .map(|alias| format!(r#"console.log({alias}(1.6));"#))
        .collect::<Vec<_>>()
        .join(" ")
}

fn frozen_callable_helpers_stdout() -> String {
    let mut stdout = String::new();

    for _ in object_has_own_frozen_callable_aliases() {
        stdout.push_str("1\n");
    }

    for alias in math_floor_trunc_ceil_frozen_callable_aliases() {
        if alias.contains("ceil") {
            stdout.push_str("2\n");
        } else {
            stdout.push_str("1\n");
        }
    }

    stdout
}

fn frozen_callable_helpers_source() -> String {
    format!(
        "const object = {{ a: 1 }}; {} {}",
        object_has_own_frozen_callable_invocations_source(),
        math_floor_trunc_ceil_frozen_callable_invocations_source(),
    )
}

fn frozen_callable_helpers_test_source() -> String {
    format!(
        "Kali.test('freeze-wrapped callable helpers', () => {{ const object = {{ a: 1 }}; {} {} }});",
        object_has_own_frozen_callable_invocations_source(),
        math_floor_trunc_ceil_frozen_callable_invocations_source(),
    )
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

    let expected_stdout = frozen_callable_helpers_stdout();

    if json_output {
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "run");
        assert_eq!(json["success"], true);
        assert_eq!(json["stdout"], expected_stdout);
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, expected_stdout, "stdout: {stdout}");
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

    let expected_stdout = frozen_callable_helpers_stdout();

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
        assert!(stdout.contains(&expected_stdout), "stdout: {stdout}");
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
