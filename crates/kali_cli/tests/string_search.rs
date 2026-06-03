use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn supported_source() -> &'static str {
    "console.log(\"hello\".includes(\"ell\"));\nconsole.log(\"hello\".indexOf(\"l\", 3));\nconsole.log(\"hello\".lastIndexOf(\"l\"));\nconsole.log(\"hello\".lastIndexOf(\"l\", 2));\nconsole.log(\"hello\".lastIndexOf(\"l\", -1));\nconsole.log(\"hello\".startsWith(\"he\"));\nconsole.log(\"hello\".startsWith(\"ll\", 2));\nconsole.log(\"hello\".endsWith(\"lo\"));\nconsole.log(\"hello\".endsWith(\"ell\", 4));\nconsole.log(\"hello\".endsWith(\"he\", 4));\n"
}

#[test]
fn run_supports_static_ascii_string_search_helpers() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("string-search.js");
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
        "1\n3\n3\n2\n-1\n1\n1\n1\n1\n0\n"
    );
}

#[test]
fn json_check_accepts_static_ascii_string_search_helpers_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("string-search.ts");
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
fn browser_bundle_accepts_static_ascii_string_search_helpers_across_source_classes() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("string-search.{extension}"));
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
fn json_browser_bundle_accepts_static_ascii_string_search_helpers_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("string-search.jsx");
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
fn check_rejects_dynamic_static_string_search_operand() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("string-search-dynamic.js");
    fs::write(
        &source_path,
        "function has(needle) { return \"hello\".startsWith(needle); }\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success(), "expected check to fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("string search method"), "stderr: {stderr}");
}
