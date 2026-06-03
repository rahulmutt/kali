use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn supported_source() -> &'static str {
    r#"const suffix = 'llo';
console.log('he'.concat('llo'));
console.log('he'.concat('l', 'lo'));
console.log('hello'.concat());
console.log(Object.freeze('he').concat(Object.freeze(suffix)));
console.log('he'["concat"]('llo'));
"#
}

#[test]
fn run_supports_static_ascii_string_concat() {
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
        "hello\nhello\nhello\nhello\nhello\n"
    );
}

#[test]
fn json_check_accepts_static_ascii_string_concat() {
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
fn browser_bundle_accepts_static_ascii_string_concat_across_source_classes() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("string-concat.{extension}"));
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
            "extension: {extension}\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn json_browser_bundle_accepts_static_ascii_string_concat_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("string-concat.tsx");
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

fn assert_check_gates_unsupported_string_concat(source: &str) {
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
    assert!(json["errors"][0]["message"]
        .as_str()
        .expect("message")
        .contains("String.prototype.concat"));
}

#[test]
fn check_gates_string_concat_dynamic_operand() {
    assert_check_gates_unsupported_string_concat(
        "function join(suffix) { return 'he'.concat(suffix); }\n",
    );
}

#[test]
fn check_gates_string_concat_dynamic_receiver() {
    assert_check_gates_unsupported_string_concat(
        "function join(value) { return value.concat('llo'); }\n",
    );
}

#[test]
fn check_gates_non_ascii_static_string_concat_receiver() {
    assert_check_gates_unsupported_string_concat("console.log('hé'.concat('llo'));\n");
}

#[test]
fn check_gates_non_ascii_static_string_concat_operand() {
    assert_check_gates_unsupported_string_concat("console.log('he'.concat('lló'));\n");
}
