use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn supported_source() -> &'static str {
    r#"console.log(String.fromCharCode());
console.log(Object.freeze(globalThis.String.fromCharCode)());
console.log(String.fromCharCode(72, 105));
console.log(globalThis.String.fromCharCode(79, 75));
console.log(globalThis["String"]["fromCharCode"](65));
console.log(globalThis["String"]['fromCharCode'](67));
console.log(Object.freeze(globalThis['String'])["fromCharCode"](68));
const fromCharCode = Object.freeze(String.fromCharCode);
const message = fromCharCode(66, Object.freeze(121), 101);
console.log(message);
"#
}

#[test]
fn run_supports_static_ascii_string_from_char_code() {
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
        "\n\nHi\nOK\nA\nC\nD\nBye\n"
    );
}

#[test]
fn browser_bundle_accepts_static_ascii_string_from_char_code_across_source_classes() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir
            .path()
            .join(format!("string-from-char-code.{extension}"));
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
fn json_browser_bundle_accepts_static_ascii_string_from_char_code_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("string-from-char-code.tsx");
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

#[test]
fn json_check_accepts_static_ascii_string_from_char_code() {
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

fn assert_check_gates_unsupported_string_from_char_code(source: &str) {
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
fn check_gates_string_from_char_code_dynamic_argument() {
    assert_check_gates_unsupported_string_from_char_code(
        "function make(code) { return String.fromCharCode(code); }\n",
    );
}

#[test]
fn check_gates_string_from_char_code_non_ascii_argument() {
    assert_check_gates_unsupported_string_from_char_code(
        "console.log(String.fromCharCode(233));\n",
    );
}

#[test]
fn check_gates_string_from_char_code_fractional_argument() {
    assert_check_gates_unsupported_string_from_char_code(
        "console.log(String.fromCharCode(65.5));\n",
    );
}
