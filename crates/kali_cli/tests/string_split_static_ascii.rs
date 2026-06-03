use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn supported_source() -> &'static str {
    r#"const whole = "abc".split();
console.log(whole.length);
const comma = "a,b,c".split(",");
console.log(comma.length);
const chars = Object.freeze("abc").split(Object.freeze(""), Object.freeze(2));
console.log(chars.length);
"#
}

#[test]
fn json_check_accepts_static_ascii_string_split_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("string-split.ts");
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
fn browser_bundle_accepts_static_ascii_string_split_across_source_classes() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("string-split.{extension}"));
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
fn json_browser_bundle_accepts_static_ascii_string_split_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("string-split.tsx");
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

fn assert_check_gates_unsupported_string_split(source: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("string-split-dynamic.js");
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
        .contains("String.prototype.split"));
}

#[test]
fn check_gates_dynamic_string_split_separator() {
    assert_check_gates_unsupported_string_split(
        "function split(separator) { return 'a,b'.split(separator); }\n",
    );
}

#[test]
fn check_gates_non_ascii_static_string_split_receiver() {
    assert_check_gates_unsupported_string_split("console.log('á,b'.split(','));\n");
}

#[test]
fn check_gates_string_split_negative_limit() {
    assert_check_gates_unsupported_string_split("console.log('a,b'.split(',', -1));\n");
}
