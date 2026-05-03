use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn wrapped_object_enumeration_source() -> &'static str {
    r#"const wrapped = ({ "b": 1, "2": 2, "a": 3, "1": 4 });
const alias = wrapped;
const keys = Object.keys(alias);
const entries = Object.entries(alias);
const values = Object.values(alias);
if (
  keys.length !== 4 ||
  keys[0] !== '1' ||
  keys[1] !== '2' ||
  keys[2] !== 'b' ||
  keys[3] !== 'a' ||
  entries.length !== 4 ||
  entries[0][0] !== '1' ||
  entries[0][1] !== 4 ||
  entries[1][0] !== '2' ||
  entries[1][1] !== 2 ||
  entries[2][0] !== 'b' ||
  entries[2][1] !== 1 ||
  entries[3][0] !== 'a' ||
  entries[3][1] !== 3 ||
  values.length !== 4 ||
  values[0] !== 4 ||
  values[1] !== 2 ||
  values[2] !== 1 ||
  values[3] !== 3
) {
  throw new Error('unexpected wrapped object enumeration ordering');
}
console.log('wrapped object enumeration ok');
"#
}

fn wrapped_object_enumeration_test_source() -> &'static str {
    r#"Kali.test('wrapped object enumeration', () => {
  const wrapped = ({ "b": 1, "2": 2, "a": 3, "1": 4 });
  const alias = wrapped;
  const keys = Object.keys(alias);
  const entries = Object.entries(alias);
  const values = Object.values(alias);
  if (
    keys.length !== 4 ||
    keys[0] !== '1' ||
    keys[1] !== '2' ||
    keys[2] !== 'b' ||
    keys[3] !== 'a' ||
    entries.length !== 4 ||
    entries[0][0] !== '1' ||
    entries[0][1] !== 4 ||
    entries[1][0] !== '2' ||
    entries[1][1] !== 2 ||
    entries[2][0] !== 'b' ||
    entries[2][1] !== 1 ||
    entries[3][0] !== 'a' ||
    entries[3][1] !== 3 ||
    values.length !== 4 ||
    values[0] !== 4 ||
    values[1] !== 2 ||
    values[2] !== 1 ||
    values[3] !== 3
  ) {
    throw new Error('unexpected wrapped object enumeration ordering');
  }
});
"#
}

fn assert_wrapped_object_enumeration(command: &str, filename: &str, source: &'static str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg(command)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));

    if command == "run" {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("wrapped object enumeration ok"),
            "stdout: {stdout}"
        );
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
}

#[test]
fn run_accepts_wrapped_object_enumeration_in_js_input() {
    assert_wrapped_object_enumeration("run", "main.js", wrapped_object_enumeration_source());
}

#[test]
fn json_run_accepts_wrapped_object_enumeration_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, wrapped_object_enumeration_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
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
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert!(
        json["stdout"]
            .as_str()
            .expect("run stdout")
            .contains("wrapped object enumeration ok"),
        "json: {json}"
    );
}

#[test]
fn test_accepts_wrapped_object_enumeration_in_js_input() {
    assert_wrapped_object_enumeration(
        "test",
        "main.test.js",
        wrapped_object_enumeration_test_source(),
    );
}

#[test]
fn json_test_accepts_wrapped_object_enumeration_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.js");
    fs::write(&source_path, wrapped_object_enumeration_test_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
    assert_eq!(json["stdout"], "");
    assert_eq!(json["stderr"], "");
}
