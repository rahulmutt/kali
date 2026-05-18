use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn non_literal_set_source() -> &'static str {
    r#"function main() {
  let values = [1, 2];
  values = values;
  for (const value of new Set(values.filter(Boolean))) {
    console.log(value);
  }
}
main();
"#
}

fn non_literal_map_source() -> &'static str {
    r#"function main() {
  let values = [[1, 2], [3, 4]];
  values = values;
  for (const entry of new Map(values.filter(Boolean))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
}
main();
"#
}

fn assert_standalone_iterator_source_rejects(
    source: &str,
    filename: &str,
    json_output: bool,
    command: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut cmd = Command::new(kali_bin());
    if json_output {
        cmd.arg("--output").arg("json");
    }
    cmd.current_dir(dir.path());
    let output = cmd
        .arg(command)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert_eq!(output.status.code(), Some(1));

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
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
fn standalone_non_literal_set_and_map_sources_reject_in_run() {
    for (source, filename) in [
        (non_literal_set_source(), "set-main.js"),
        (non_literal_map_source(), "map-main.js"),
    ] {
        assert_standalone_iterator_source_rejects(source, filename, false, "run");
        assert_standalone_iterator_source_rejects(source, filename, true, "run");
    }
}

#[test]
fn standalone_non_literal_set_and_map_sources_reject_in_test() {
    for (source, filename) in [
        (non_literal_set_source(), "set-main.js"),
        (non_literal_map_source(), "map-main.js"),
    ] {
        assert_standalone_iterator_source_rejects(source, filename, false, "test");
        assert_standalone_iterator_source_rejects(source, filename, true, "test");
    }
}
