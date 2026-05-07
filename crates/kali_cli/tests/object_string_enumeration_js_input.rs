use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn object_string_enumeration_run_source() -> &'static str {
    r#"function assertObjectKeysIteration(keys) {
  if (keys.length !== 2 || keys[0] !== '0' || keys[1] !== '1') {
    throw new Error('unexpected Object.keys string-primitive iteration semantics');
  }
}

function assertObjectValuesIteration(values) {
  if (values.length !== 2 || values[0] !== 'a' || values[1] !== 'b') {
    throw new Error('unexpected Object.values string-primitive iteration semantics');
  }
}

function assertObjectEntriesIteration(entries) {
  if (
    entries.length !== 2 ||
    entries[0][0] !== '0' ||
    entries[0][1] !== 'a' ||
    entries[1][0] !== '1' ||
    entries[1][1] !== 'b'
  ) {
    throw new Error('unexpected Object.entries string-primitive iteration semantics');
  }
}

function objectStringEnumeration() {
  const keys = [];
  for (const key of Object.keys('ab')) {
    keys.push(key);
  }
  const values = [];
  for (const value of Object.values('ab')) {
    values.push(value);
  }
  const entries = [];
  for (const entry of Object.entries('ab')) {
    entries.push(entry);
  }
  assertObjectKeysIteration(keys);
  assertObjectValuesIteration(values);
  assertObjectEntriesIteration(entries);
  console.log('object string enumeration ok');
}

objectStringEnumeration();
"#
}

fn object_string_enumeration_test_source() -> &'static str {
    r#"Kali.test('object string enumeration', () => {
  function assertObjectKeysIteration(keys) {
    if (keys.length !== 2 || keys[0] !== '0' || keys[1] !== '1') {
      throw new Error('unexpected Object.keys string-primitive iteration semantics');
    }
  }

  function assertObjectValuesIteration(values) {
    if (values.length !== 2 || values[0] !== 'a' || values[1] !== 'b') {
      throw new Error('unexpected Object.values string-primitive iteration semantics');
    }
  }

  function assertObjectEntriesIteration(entries) {
    if (
      entries.length !== 2 ||
      entries[0][0] !== '0' ||
      entries[0][1] !== 'a' ||
      entries[1][0] !== '1' ||
      entries[1][1] !== 'b'
    ) {
      throw new Error('unexpected Object.entries string-primitive iteration semantics');
    }
  }

  const keys = [];
  for (const key of Object.keys('ab')) {
    keys.push(key);
  }
  const values = [];
  for (const value of Object.values('ab')) {
    values.push(value);
  }
  const entries = [];
  for (const entry of Object.entries('ab')) {
    entries.push(entry);
  }
  assertObjectKeysIteration(keys);
  assertObjectValuesIteration(values);
  assertObjectEntriesIteration(entries);
  console.log('object string enumeration ok');
});
"#
}

fn assert_object_string_enumeration(command: &str, filename: &str, source: &'static str) {
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
            stdout.contains("object string enumeration ok"),
            "stdout: {stdout}"
        );
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
}

#[test]
#[ignore = "browser string-enumeration run/test remains E5506-gated"]
fn run_accepts_object_string_enumeration_in_js_input() {
    assert_object_string_enumeration("run", "main.js", object_string_enumeration_run_source());
}

#[test]
#[ignore = "browser string-enumeration run/test remains E5506-gated"]
fn run_accepts_object_string_enumeration_in_ts_input() {
    assert_object_string_enumeration("run", "main.ts", object_string_enumeration_run_source());
}

#[test]
#[ignore = "browser string-enumeration run/test remains E5506-gated"]
fn run_accepts_object_string_enumeration_in_jsx_input() {
    assert_object_string_enumeration("run", "main.jsx", object_string_enumeration_run_source());
}

#[test]
#[ignore = "browser string-enumeration run/test remains E5506-gated"]
fn run_accepts_object_string_enumeration_in_tsx_input() {
    assert_object_string_enumeration("run", "main.tsx", object_string_enumeration_run_source());
}

#[test]
#[ignore = "browser string-enumeration run/test remains E5506-gated"]
fn json_run_accepts_object_string_enumeration_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, object_string_enumeration_run_source()).expect("write source");

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
            .contains("object string enumeration ok"),
        "json: {json}"
    );
}

#[test]
#[ignore = "browser string-enumeration run/test remains E5506-gated"]
fn json_run_accepts_object_string_enumeration_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, object_string_enumeration_run_source()).expect("write source");

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
            .contains("object string enumeration ok"),
        "json: {json}"
    );
}

#[test]
#[ignore = "browser string-enumeration run/test remains E5506-gated"]
fn json_run_accepts_object_string_enumeration_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.jsx");
    fs::write(&source_path, object_string_enumeration_run_source()).expect("write source");

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
            .contains("object string enumeration ok"),
        "json: {json}"
    );
}

#[test]
#[ignore = "browser string-enumeration run/test remains E5506-gated"]
fn json_run_accepts_object_string_enumeration_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(&source_path, object_string_enumeration_run_source()).expect("write source");

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
            .contains("object string enumeration ok"),
        "json: {json}"
    );
}

#[test]
#[ignore = "browser string-enumeration run/test remains E5506-gated"]
fn test_accepts_object_string_enumeration_in_js_input() {
    assert_object_string_enumeration(
        "test",
        "main.test.js",
        object_string_enumeration_test_source(),
    );
}

#[test]
#[ignore = "browser string-enumeration run/test remains E5506-gated"]
fn test_accepts_object_string_enumeration_in_ts_input() {
    assert_object_string_enumeration(
        "test",
        "main.test.ts",
        object_string_enumeration_test_source(),
    );
}

#[test]
#[ignore = "browser string-enumeration run/test remains E5506-gated"]
fn test_accepts_object_string_enumeration_in_jsx_input() {
    assert_object_string_enumeration(
        "test",
        "main.test.jsx",
        object_string_enumeration_test_source(),
    );
}

#[test]
#[ignore = "browser string-enumeration run/test remains E5506-gated"]
fn test_accepts_object_string_enumeration_in_tsx_input() {
    assert_object_string_enumeration(
        "test",
        "main.test.tsx",
        object_string_enumeration_test_source(),
    );
}

#[test]
#[ignore = "browser string-enumeration run/test remains E5506-gated"]
fn json_test_accepts_object_string_enumeration_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.js");
    fs::write(&source_path, object_string_enumeration_test_source()).expect("write source");

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
    assert_eq!(json["stdout"], "object string enumeration ok\n");
    assert_eq!(json["stderr"], "");
}

#[test]
#[ignore = "browser string-enumeration run/test remains E5506-gated"]
fn json_test_accepts_object_string_enumeration_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.ts");
    fs::write(&source_path, object_string_enumeration_test_source()).expect("write source");

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
    assert_eq!(json["stdout"], "object string enumeration ok\n");
    assert_eq!(json["stderr"], "");
}

#[test]
#[ignore = "browser string-enumeration run/test remains E5506-gated"]
fn json_test_accepts_object_string_enumeration_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.jsx");
    fs::write(&source_path, object_string_enumeration_test_source()).expect("write source");

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
    assert_eq!(json["stdout"], "object string enumeration ok\n");
    assert_eq!(json["stderr"], "");
}

#[test]
#[ignore = "browser string-enumeration run/test remains E5506-gated"]
fn json_test_accepts_object_string_enumeration_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.tsx");
    fs::write(&source_path, object_string_enumeration_test_source()).expect("write source");

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
    assert_eq!(json["stdout"], "object string enumeration ok\n");
    assert_eq!(json["stderr"], "");
}
