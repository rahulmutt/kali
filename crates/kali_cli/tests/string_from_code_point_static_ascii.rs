use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn supported_source() -> &'static str {
    r#"console.log(String.fromCodePoint(72, 105));
console.log(globalThis.String.fromCodePoint(79, 75));
console.log(globalThis["String"]["fromCodePoint"](65));
const fromCodePoint = Object.freeze(String.fromCodePoint);
const message = fromCodePoint(66, Object.freeze(121), 101);
console.log(message);
"#
}

#[test]
fn run_supports_static_ascii_string_from_code_point() {
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
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Hi\nOK\nA\nBye\n");
}

#[test]
fn json_check_accepts_static_ascii_string_from_code_point() {
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

fn assert_check_gates_unsupported_string_from_code_point(source: &str) {
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
    assert!(
        json["errors"][0]["message"]
            .as_str()
            .expect("message")
            .contains("String.fromCodePoint"),
        "json: {json}"
    );
}

#[test]
fn check_gates_string_from_code_point_dynamic_argument() {
    assert_check_gates_unsupported_string_from_code_point(
        "function make(code) { return String.fromCodePoint(code); }\n",
    );
}

#[test]
fn check_gates_string_from_code_point_non_ascii_argument() {
    assert_check_gates_unsupported_string_from_code_point(
        "console.log(String.fromCodePoint(233));\n",
    );
}

#[test]
fn check_gates_string_from_code_point_fractional_argument() {
    assert_check_gates_unsupported_string_from_code_point(
        "console.log(String.fromCodePoint(65.5));\n",
    );
}
