use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
}

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn broader_intl_js_source() -> &'static str {
    "Intl; globalThis.Intl; globalThis.Intl.NumberFormat; globalThis.Intl.DateTimeFormat; Intl.NumberFormat; Intl.DateTimeFormat;"
}

fn assert_broader_intl_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    for expected in [
        "Intl",
        "globalThis.Intl",
        "globalThis.Intl.NumberFormat",
        "globalThis.Intl.DateTimeFormat",
        "Intl.NumberFormat",
        "Intl.DateTimeFormat",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

fn assert_broader_intl_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().all(|error| error["code"] == "E5506"),
        "unexpected errors: {errors:?}"
    );
    let messages = errors
        .iter()
        .map(|error| error["message"].as_str().expect("error message"))
        .collect::<Vec<_>>();
    for expected in [
        "Intl",
        "globalThis.Intl",
        "globalThis.Intl.NumberFormat",
        "globalThis.Intl.DateTimeFormat",
        "Intl.NumberFormat",
        "Intl.DateTimeFormat",
    ] {
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "missing {expected} in {messages:?}"
        );
    }
}

#[test]
fn check_rejects_broader_intl_support_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, broader_intl_js_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_broader_intl_rejection(&stderr);
}

#[test]
fn check_rejects_broader_intl_support_in_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, broader_intl_js_source()).expect("write source");

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
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_broader_intl_rejection_json(errors);
}

#[test]
fn run_rejects_broader_intl_support_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, broader_intl_js_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_broader_intl_rejection(&stderr);
}

#[test]
fn run_rejects_broader_intl_support_in_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, broader_intl_js_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_broader_intl_rejection_json(errors);
}

#[test]
fn test_rejects_broader_intl_support_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, broader_intl_js_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_broader_intl_rejection(&stderr);
}

#[test]
fn test_rejects_broader_intl_support_in_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, broader_intl_js_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_broader_intl_rejection_json(errors);
}
