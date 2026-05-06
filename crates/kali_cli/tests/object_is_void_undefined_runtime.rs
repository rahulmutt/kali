use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn object_is_void_undefined_source() -> &'static str {
    r#"const alias = void 0;
console.log(Object.is(alias, void 0));
console.log(Object.is(void 0, void 0));
console.log(globalThis["Object"]["is"](alias, void 0));
console.log(globalThis.Object["is"](alias, void 0));
console.log(globalThis["Object"].is(alias, void 0));
console.log(globalThis.Object.is(alias, void 0));
"#
}

fn object_is_void_undefined_test_source() -> &'static str {
    r#"Kali.test('object is void undefined', () => {
  const alias = void 0;
  console.log(Object.is(alias, void 0));
  console.log(Object.is(void 0, void 0));
  console.log(globalThis["Object"]["is"](alias, void 0));
  console.log(globalThis.Object["is"](alias, void 0));
  console.log(globalThis["Object"].is(alias, void 0));
  console.log(globalThis.Object.is(alias, void 0));
});
"#
}

fn assert_run_supports_object_is_void_undefined_in_js_input(json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, object_is_void_undefined_source()).expect("write source");

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
        let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "run");
        assert_eq!(json["success"], true);
        assert_eq!(json["stdout"], "1\n1\n1\n1\n1\n1\n");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.trim(), "1\n1\n1\n1\n1\n1", "stdout: {stdout}");
    }
}

fn assert_test_supports_object_is_void_undefined_in_js_input(json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, object_is_void_undefined_test_source()).expect("write source");

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
        let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "test");
        assert_eq!(json["success"], true);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("1\n1\n1\n1\n1\n1"), "stdout: {stdout}");
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
}

#[test]
fn run_supports_object_is_void_undefined_in_js_input() {
    assert_run_supports_object_is_void_undefined_in_js_input(false);
}

#[test]
fn json_run_supports_object_is_void_undefined_in_js_input() {
    assert_run_supports_object_is_void_undefined_in_js_input(true);
}

#[test]
fn test_supports_object_is_void_undefined_in_js_input() {
    assert_test_supports_object_is_void_undefined_in_js_input(false);
}

#[test]
fn json_test_supports_object_is_void_undefined_in_js_input() {
    assert_test_supports_object_is_void_undefined_in_js_input(true);
}
