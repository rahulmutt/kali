use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn supported_source() -> &'static str {
    r#"console.log("ABC".codePointAt());
console.log("ABC".codePointAt(1));
console.log("ABC".codePointAt(3));
console.log("ABC".codePointAt(-1));
console.log(Object.freeze("ABC").codePointAt(Object.freeze(2)));
const missing = "ABC".codePointAt(99);
console.log(missing);
"#
}

#[test]
fn run_supports_static_ascii_string_code_point_at() {
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
        "65\n66\nundefined\nundefined\n67\nundefined\n"
    );
}

#[test]
fn json_check_accepts_static_ascii_string_code_point_at() {
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

#[test]
fn browser_bundle_accepts_static_ascii_string_code_point_at_across_source_classes() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("string-code-point-at.{extension}"));
        fs::write(&source_path, supported_source()).expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("build")
            .arg("--api")
            .arg("browser")
            .arg("--bundle")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(
            output.status.success(),
            "extension {extension}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn json_browser_bundle_accepts_static_ascii_string_code_point_at_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("string-code-point-at.tsx");
    fs::write(&source_path, supported_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--api")
        .arg("browser")
        .arg("--bundle")
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
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], true);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

fn assert_check_gates_unsupported_string_code_point_at(source: &str) {
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
fn check_gates_string_code_point_at_extra_arguments() {
    assert_check_gates_unsupported_string_code_point_at("console.log('ABC'.codePointAt(0, 1));\n");
}

#[test]
fn check_gates_non_ascii_static_string_code_point_at_receiver() {
    assert_check_gates_unsupported_string_code_point_at("console.log('é'.codePointAt(0));\n");
}

#[test]
fn check_gates_dynamic_string_code_point_at_index() {
    assert_check_gates_unsupported_string_code_point_at(
        "function code(index) { return 'ABC'.codePointAt(index); }\n",
    );
}
