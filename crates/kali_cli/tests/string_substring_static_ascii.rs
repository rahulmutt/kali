use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn supported_source() -> &'static str {
    r#"console.log("hello".substring());
console.log("hello".substring(1));
console.log("hello".substring(1, 4));
console.log("hello".substring(1.5, 4.9));
console.log("hello".substring(4, 1));
console.log("hello".substring(-2, 2));
"#
}

#[test]
fn run_supports_static_ascii_string_substring() {
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
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "hello\nello\nell\nell\nell\nhe\n"
    );
}

#[test]
fn json_check_accepts_static_ascii_string_substring() {
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

fn assert_check_gates_unsupported_string_substring(source: &str) {
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
fn check_gates_dynamic_string_substring_bound() {
    assert_check_gates_unsupported_string_substring(
        "function cut(start) {\n  console.log('hello'.substring(start));\n}\ncut(1);\n",
    );
}

#[test]
fn check_gates_non_ascii_static_string_substring_receiver() {
    assert_check_gates_unsupported_string_substring("console.log('héllo'.substring(1));\n");
}

#[test]
fn runs_ascii_provable_runtime_string_substring_receiver() {
    // A param that only ever holds an ASCII string is an ASCII-provable
    // runtime string receiver: Task 4 lowers `value.substring(1)` to the
    // synthetic `__substring` (byte-offset slicing is JS-correct for ASCII),
    // so this case that previously gated is now accepted and runs correctly.
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "function cut(value) {\n  console.log(value.substring(1));\n}\ncut('hello');\n",
    )
    .expect("write source");

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
    assert_eq!(String::from_utf8_lossy(&output.stdout), "ello\n");
}
