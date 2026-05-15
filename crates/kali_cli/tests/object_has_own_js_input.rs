use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn object_has_own_source() -> &'static str {
    r#"const object = Object.fromEntries([["a", 1], ["b", 2]]);
const alias = object;
const wrapped = (0, alias);
if (!Object.hasOwn(wrapped, "a") || !Object["hasOwn"](wrapped, "a") || !globalThis.Object["hasOwn"](wrapped, "a") || !globalThis["Object"]["hasOwn"](wrapped, "a") || !globalThis.Object["hasOwn"](wrapped, "a") || !globalThis["Object"].hasOwn(wrapped, "a") || !Object["hasOwnProperty"].call(wrapped, "a") || !Object["hasOwnProperty"]["call"](wrapped, "a") || !globalThis.Object.hasOwnProperty.call(wrapped, "a") || !globalThis["Object"]["hasOwnProperty"].call(wrapped, "a") || !globalThis["Object"]["hasOwnProperty"]["call"](wrapped, "a") || !globalThis["Object"].hasOwnProperty.call(wrapped, "a") || !Object.prototype.hasOwnProperty.call(wrapped, "a")) {
  throw new Error('unexpected Object.hasOwn result');
}
console.log('object hasOwn ok');
"#
}

fn object_has_own_as_const_source() -> &'static str {
    r#"const object = ({ a: 1, "b": 2 } as const);
const wrapped = (0, object);
if (!Object.hasOwn(wrapped, "a") || !Object["hasOwn"](wrapped, "a") || !globalThis.Object["hasOwn"](wrapped, "a") || !globalThis["Object"]["hasOwn"](wrapped, "a") || !globalThis.Object["hasOwn"](wrapped, "a") || !globalThis["Object"].hasOwn(wrapped, "a") || !Object["hasOwnProperty"].call(wrapped, "a") || !Object["hasOwnProperty"]["call"](wrapped, "a") || !globalThis.Object.hasOwnProperty.call(wrapped, "a") || !globalThis["Object"]["hasOwnProperty"].call(wrapped, "a") || !globalThis["Object"]["hasOwnProperty"]["call"](wrapped, "a") || !globalThis["Object"].hasOwnProperty.call(wrapped, "a") || !Object.prototype.hasOwnProperty.call(wrapped, "a")) {
  throw new Error('unexpected as const Object.hasOwn result');
}
console.log('as const object hasOwn ok');
"#
}

fn object_has_own_test_source() -> &'static str {
    r#"Kali.test('object hasOwn', () => {
  const object = { a: 1, "b": 2 };
  const alias = object;
  const wrapped = (0, alias);
  if (!Object.hasOwn(wrapped, "a") || !Object["hasOwn"](wrapped, "a") || !globalThis.Object["hasOwn"](wrapped, "a") || !globalThis["Object"]["hasOwn"](wrapped, "a") || !globalThis.Object["hasOwn"](wrapped, "a") || !globalThis["Object"].hasOwn(wrapped, "a") || !Object["hasOwnProperty"].call(wrapped, "a") || !Object["hasOwnProperty"]["call"](wrapped, "a") || !globalThis.Object.hasOwnProperty.call(wrapped, "a") || !globalThis["Object"]["hasOwnProperty"].call(wrapped, "a") || !globalThis["Object"]["hasOwnProperty"]["call"](wrapped, "a") || !globalThis["Object"].hasOwnProperty.call(wrapped, "a") || !Object.prototype.hasOwnProperty.call(wrapped, "a")) {
    throw new Error('unexpected Object.hasOwn result');
  }
});
"#
}

#[test]
fn check_accepts_object_has_own_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, object_has_own_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn check_accepts_object_has_own_for_as_const_object_alias_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, object_has_own_as_const_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn run_accepts_object_has_own_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, object_has_own_source()).expect("write source");

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
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("object hasOwn ok"), "stdout: {stdout}");
}

#[test]
fn run_accepts_object_has_own_for_as_const_object_alias_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, object_has_own_as_const_source()).expect("write source");

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
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("as const object hasOwn ok"),
        "stdout: {stdout}"
    );
}

#[test]
fn json_run_accepts_object_has_own_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, object_has_own_source()).expect("write source");

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
    assert_eq!(output.status.code(), Some(0));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert!(json["stdout"]
        .as_str()
        .expect("run stdout")
        .contains("object hasOwn ok"));
}

#[test]
fn test_accepts_object_has_own_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.js");
    fs::write(&source_path, object_has_own_test_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
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
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn json_test_accepts_object_has_own_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.js");
    fs::write(&source_path, object_has_own_test_source()).expect("write source");

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
    assert_eq!(output.status.code(), Some(0));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
}

#[test]
fn check_accepts_object_has_own_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, object_has_own_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn run_accepts_object_has_own_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, object_has_own_source()).expect("write source");

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
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("object hasOwn ok"), "stdout: {stdout}");
}

#[test]
fn json_run_accepts_object_has_own_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, object_has_own_source()).expect("write source");

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
    assert_eq!(output.status.code(), Some(0));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert!(json["stdout"]
        .as_str()
        .expect("run stdout")
        .contains("object hasOwn ok"));
}

#[test]
fn test_accepts_object_has_own_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.ts");
    fs::write(&source_path, object_has_own_test_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
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
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn json_test_accepts_object_has_own_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.ts");
    fs::write(&source_path, object_has_own_test_source()).expect("write source");

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
    assert_eq!(output.status.code(), Some(0));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
}

#[test]
fn check_accepts_object_has_own_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.jsx");
    fs::write(&source_path, object_has_own_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn run_accepts_object_has_own_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.jsx");
    fs::write(&source_path, object_has_own_source()).expect("write source");

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
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("object hasOwn ok"), "stdout: {stdout}");
}

#[test]
fn json_run_accepts_object_has_own_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.jsx");
    fs::write(&source_path, object_has_own_source()).expect("write source");

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
    assert_eq!(output.status.code(), Some(0));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert!(json["stdout"]
        .as_str()
        .expect("run stdout")
        .contains("object hasOwn ok"));
}

#[test]
fn test_accepts_object_has_own_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.jsx");
    fs::write(&source_path, object_has_own_test_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
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
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn json_test_accepts_object_has_own_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.jsx");
    fs::write(&source_path, object_has_own_test_source()).expect("write source");

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
    assert_eq!(output.status.code(), Some(0));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
}

#[test]
fn check_accepts_object_has_own_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(&source_path, object_has_own_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn run_accepts_object_has_own_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(&source_path, object_has_own_source()).expect("write source");

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
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("object hasOwn ok"), "stdout: {stdout}");
}

#[test]
fn json_run_accepts_object_has_own_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(&source_path, object_has_own_source()).expect("write source");

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
    assert_eq!(output.status.code(), Some(0));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert!(json["stdout"]
        .as_str()
        .expect("run stdout")
        .contains("object hasOwn ok"));
}

#[test]
fn test_accepts_object_has_own_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.tsx");
    fs::write(&source_path, object_has_own_test_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
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
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn json_test_accepts_object_has_own_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.tsx");
    fs::write(&source_path, object_has_own_test_source()).expect("write source");

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
    assert_eq!(output.status.code(), Some(0));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["total"], 1);
    assert_eq!(json["payload"]["passed"], 1);
    assert_eq!(json["payload"]["failed"], 0);
    assert_eq!(json["payload"]["skipped"], 0);
}
