use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn for_of_source() -> &'static str {
    r#"function main() {
  const values = Object.keys({});
  for (const item of values) {
    console.log(item);
  }
}
main();
"#
}

fn for_await_source() -> &'static str {
    r#"async function main() {
  const values = Object.keys({});
  for await (const item of values) {
    console.log(item);
  }
}
main();
"#
}

fn assert_browser_bundle_rejects_non_literal_iterator_source(
    source: &str,
    filename: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut command = Command::new(kali_bin());
    if json_output {
        command.arg("--output").arg("json");
    }
    let output = command
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert_eq!(output.status.code(), Some(1));

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "build");
        assert_eq!(json["success"], false);
        assert_eq!(json["exitCode"], 1);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(
            !errors.is_empty(),
            "errors array should not be empty: {json}"
        );
        let message = errors[0]["message"].as_str().expect("error message");
        assert_eq!(errors[0]["code"], "E5506");
        assert!(
            message.contains("literal array"),
            "unexpected error message: {message}"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(
            stderr.contains("literal array"),
            "unexpected stderr: {stderr}"
        );
    }
}

#[test]
fn build_rejects_non_literal_for_of_iterator_source_in_js_input() {
    assert_browser_bundle_rejects_non_literal_iterator_source(for_of_source(), "main.js", false);
}

#[test]
fn json_build_rejects_non_literal_for_of_iterator_source_in_js_input() {
    assert_browser_bundle_rejects_non_literal_iterator_source(for_of_source(), "main.js", true);
}

#[test]
fn build_rejects_non_literal_for_await_iterator_source_in_ts_input() {
    assert_browser_bundle_rejects_non_literal_iterator_source(for_await_source(), "main.ts", false);
}

#[test]
fn json_build_rejects_non_literal_for_await_iterator_source_in_ts_input() {
    assert_browser_bundle_rejects_non_literal_iterator_source(for_await_source(), "main.ts", true);
}
