use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn frozen_object_enumeration_spread_source() -> &'static str {
    r#"const frozenFromEntries = Object.freeze(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]));
const frozenKeys = [...Object.keys(frozenFromEntries)];
const frozenValues = [...Object.values(frozenFromEntries)];
const frozenEntries = [...Object.entries(frozenFromEntries)];
if (
  frozenKeys.length !== 2 ||
  frozenKeys[0] !== 'b' ||
  frozenKeys[1] !== 'a' ||
  frozenValues.length !== 2 ||
  frozenValues[0] !== 3 ||
  frozenValues[1] !== 2 ||
  frozenEntries.length !== 2 ||
  frozenEntries[0][0] !== 'b' ||
  frozenEntries[0][1] !== 3 ||
  frozenEntries[1][0] !== 'a' ||
  frozenEntries[1][1] !== 2
) {
  throw new Error('unexpected frozen object enumeration spread semantics');
}
for (const value of frozenValues) {
  console.log(value);
}
for (const key of frozenKeys) {
  console.log(key);
}
for (const entry of frozenEntries) {
  console.log(entry[0]);
  console.log(entry[1]);
}
console.log('frozen object enumeration spread ok');
"#
}

fn frozen_object_enumeration_spread_test_source() -> &'static str {
    r#"Kali.test('frozen object enumeration spread', () => {
  const frozenFromEntries = Object.freeze(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]));
  const frozenKeys = [...Object.keys(frozenFromEntries)];
  const frozenValues = [...Object.values(frozenFromEntries)];
  const frozenEntries = [...Object.entries(frozenFromEntries)];
  if (
    frozenKeys.length !== 2 ||
    frozenKeys[0] !== 'b' ||
    frozenKeys[1] !== 'a' ||
    frozenValues.length !== 2 ||
    frozenValues[0] !== 3 ||
    frozenValues[1] !== 2 ||
    frozenEntries.length !== 2 ||
    frozenEntries[0][0] !== 'b' ||
    frozenEntries[0][1] !== 3 ||
    frozenEntries[1][0] !== 'a' ||
    frozenEntries[1][1] !== 2
  ) {
    throw new Error('unexpected frozen object enumeration spread semantics');
  }
});
"#
}

fn assert_frozen_object_enumeration_spread(command: &str, filename: &str, source: &'static str) {
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
            stdout.contains("frozen object enumeration spread ok"),
            "stdout: {stdout}"
        );
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
}

#[test]
fn run_accepts_frozen_object_enumeration_spread_in_js_input() {
    assert_frozen_object_enumeration_spread(
        "run",
        "main.js",
        frozen_object_enumeration_spread_source(),
    );
}

#[test]
fn run_accepts_frozen_object_enumeration_spread_in_ts_input() {
    assert_frozen_object_enumeration_spread(
        "run",
        "main.ts",
        frozen_object_enumeration_spread_source(),
    );
}

#[test]
fn json_run_accepts_frozen_object_enumeration_spread_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, frozen_object_enumeration_spread_source()).expect("write source");

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
            .contains("frozen object enumeration spread ok"),
        "json: {json}"
    );
}

#[test]
fn json_run_accepts_frozen_object_enumeration_spread_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, frozen_object_enumeration_spread_source()).expect("write source");

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
            .contains("frozen object enumeration spread ok"),
        "json: {json}"
    );
}

#[test]
fn test_accepts_frozen_object_enumeration_spread_in_js_input() {
    assert_frozen_object_enumeration_spread(
        "test",
        "main.test.js",
        frozen_object_enumeration_spread_test_source(),
    );
}

#[test]
fn test_accepts_frozen_object_enumeration_spread_in_ts_input() {
    assert_frozen_object_enumeration_spread(
        "test",
        "main.test.ts",
        frozen_object_enumeration_spread_test_source(),
    );
}

#[test]
fn json_test_accepts_frozen_object_enumeration_spread_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.js");
    fs::write(&source_path, frozen_object_enumeration_spread_test_source()).expect("write source");

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

#[test]
fn json_test_accepts_frozen_object_enumeration_spread_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.ts");
    fs::write(&source_path, frozen_object_enumeration_spread_test_source()).expect("write source");

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
