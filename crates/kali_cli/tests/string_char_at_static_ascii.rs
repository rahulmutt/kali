use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn supported_source() -> &'static str {
    r#"console.log("hello".charAt());
console.log("hello".charAt(1));
console.log("hello".charAt(-1));
console.log("hello".charAt(99));
"#
}

#[test]
fn run_supports_static_ascii_string_char_at() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, supported_source()).expect("write source");

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
    assert_eq!(String::from_utf8_lossy(&output.stdout), "h\ne\n\n\n");
}

#[test]
fn json_check_accepts_static_ascii_string_char_at() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, supported_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
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
    let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], true);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

fn assert_check_gates_unsupported_string_char_at(source: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5506");
}

#[test]
fn check_gates_string_char_at_dynamic_index() {
    assert_check_gates_unsupported_string_char_at(
        "function pick(index) { return 'hello'.charAt(index); }\n",
    );
}

#[test]
fn check_gates_string_char_at_non_integer_index() {
    assert_check_gates_unsupported_string_char_at("console.log('hello'.charAt(1.5));\n");
}

#[test]
fn check_gates_non_ascii_static_string_char_at_receiver() {
    assert_check_gates_unsupported_string_char_at("console.log('héllo'.charAt(1));\n");
}

#[test]
fn check_gates_dynamic_string_char_at_receiver() {
    assert_check_gates_unsupported_string_char_at(
        "function pick(value) { return value.charAt(1); }\n",
    );
}
