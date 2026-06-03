use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn supported_source() -> &'static str {
    r#"console.log("hello".normalize());
console.log("ASCII".normalize("NFC"));
console.log(Object.freeze("wrapped").normalize(Object.freeze("NFD")));
console.log("compat".normalize("NFKC"));
console.log("decomp".normalize("NFKD"));
"#
}

#[test]
fn run_supports_static_ascii_string_normalize() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("string-normalize.js");
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
        "hello\nASCII\nwrapped\ncompat\ndecomp\n"
    );
}

#[test]
fn json_check_accepts_static_ascii_string_normalize() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("string-normalize.ts");
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
fn browser_bundle_accepts_static_ascii_string_normalize_across_source_classes() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("string-normalize.{extension}"));
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

fn assert_check_gates_unsupported_string_normalize(source: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("string-normalize-dynamic.js");
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
        .contains("String.prototype.normalize"));
}

#[test]
fn check_gates_dynamic_string_normalize_form() {
    assert_check_gates_unsupported_string_normalize(
        "function normalize(form) { return 'hello'.normalize(form); }\n",
    );
}

#[test]
fn check_gates_non_ascii_static_string_normalize_receiver() {
    assert_check_gates_unsupported_string_normalize("console.log('é'.normalize());\n");
}

#[test]
fn check_gates_invalid_static_string_normalize_form() {
    assert_check_gates_unsupported_string_normalize("console.log('hello'.normalize('BAD'));\n");
}

#[test]
fn check_gates_extra_string_normalize_arguments() {
    assert_check_gates_unsupported_string_normalize(
        "console.log('hello'.normalize('NFC', 'NFD'));\n",
    );
}
