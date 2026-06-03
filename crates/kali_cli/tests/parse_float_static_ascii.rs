use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn supported_source() -> &'static str {
    r#"console.log(parseFloat('42.0px'));
console.log(globalThis.parseFloat('-1.2e1tail'));
console.log(globalThis["parseFloat"]('10e1'));
console.log(Number.parseFloat('7.000'));
console.log(globalThis["Number"]["parseFloat"]('6.02e2'));
console.log(globalThis.Number['parseFloat'](Object.freeze('9.0')));
const parse = Object.freeze(parseFloat);
const numberParse = Object.freeze(globalThis["Number"]["parseFloat"]);
console.log(parse(Object.freeze('77.0')));
console.log(numberParse('8e1'));
"#
}

#[test]
fn run_supports_static_ascii_parse_float_integer_slice() {
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
        "42\n-12\n100\n7\n602\n9\n77\n80\n"
    );
}

#[test]
fn json_check_accepts_static_ascii_parse_float_integer_slice() {
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
fn browser_bundle_accepts_static_ascii_parse_float_across_source_classes() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("parse-float.{extension}"));
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
fn json_browser_bundle_accepts_static_ascii_parse_float_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("parse-float.tsx");
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

fn assert_check_gates_unsupported_parse_float(source: &str) {
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
        .contains("parseFloat"));
}

#[test]
fn check_gates_parse_float_dynamic_input() {
    assert_check_gates_unsupported_parse_float(
        "function parse(value) { return parseFloat(value); }\n",
    );
}

#[test]
fn check_gates_parse_float_non_ascii_input() {
    assert_check_gates_unsupported_parse_float("console.log(parseFloat('é'));\n");
}

#[test]
fn check_gates_parse_float_fractional_result() {
    assert_check_gates_unsupported_parse_float("console.log(parseFloat('1.5'));\n");
}

#[test]
fn check_gates_parse_float_additional_arguments() {
    assert_check_gates_unsupported_parse_float("console.log(parseFloat('10', 10));\n");
}

#[test]
fn check_gates_parse_float_nan_result() {
    assert_check_gates_unsupported_parse_float("console.log(parseFloat('nope'));\n");
}
