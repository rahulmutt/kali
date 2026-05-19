use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

use kali_common::reflect_own_keys_frozen_callable_source;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn reflect_own_keys_source() -> String {
    let frozen_callable_lines = reflect_own_keys_frozen_callable_source("obj");
    format!(
        r#"const obj = {{ "b": 1, "2": 2, "a": 3, "1": 4 }};
const keys = globalThis.Reflect.ownKeys(obj);
const mixedRootKeys = globalThis["Reflect"].ownKeys(obj);
const mixedBracketedKeys = globalThis.Reflect["ownKeys"](obj);
const bracketedKeys = globalThis["Reflect"]["ownKeys"](obj);
const fullyBracketedKeys = globalThis["Reflect"]["ownKeys"](obj);
const singleQuotedKeys = globalThis['Reflect']['ownKeys'](obj);
{frozen_callable_lines}
let syncCount = 0;
for (const key of Reflect.ownKeys(obj)) {{
  syncCount += 1;
}}
let sequenceCount = 0;
for (const key of (0, Reflect.ownKeys(obj))) {{
  sequenceCount += 1;
}}
let mixedSequenceCount = 0;
for (const key of (0, globalThis["Reflect"]["ownKeys"](obj))) {{
  mixedSequenceCount += 1;
}}
if (
  keys.length !== 4 ||
  keys[0] !== '1' ||
  keys[1] !== '2' ||
  keys[2] !== 'b' ||
  keys[3] !== 'a' ||
  mixedRootKeys.length !== 4 ||
  mixedRootKeys[0] !== '1' ||
  mixedRootKeys[1] !== '2' ||
  mixedRootKeys[2] !== 'b' ||
  mixedRootKeys[3] !== 'a' ||
  mixedBracketedKeys.length !== 4 ||
  mixedBracketedKeys[0] !== '1' ||
  mixedBracketedKeys[1] !== '2' ||
  mixedBracketedKeys[2] !== 'b' ||
  mixedBracketedKeys[3] !== 'a' ||
  bracketedKeys.length !== 4 ||
  bracketedKeys[0] !== '1' ||
  bracketedKeys[1] !== '2' ||
  bracketedKeys[2] !== 'b' ||
  bracketedKeys[3] !== 'a' ||
  fullyBracketedKeys.length !== 4 ||
  fullyBracketedKeys[0] !== '1' ||
  fullyBracketedKeys[1] !== '2' ||
  fullyBracketedKeys[2] !== 'b' ||
  fullyBracketedKeys[3] !== 'a' ||
  singleQuotedKeys.length !== 4 ||
  singleQuotedKeys[0] !== '1' ||
  singleQuotedKeys[1] !== '2' ||
  singleQuotedKeys[2] !== 'b' ||
  singleQuotedKeys[3] !== 'a' ||
  syncCount !== 4 ||
  sequenceCount !== 4 ||
  mixedSequenceCount !== 4
) {{
  throw new Error('unexpected Reflect.ownKeys ordering');
}}
console.log('reflect ownKeys ok');
"#
    )
}

fn reflect_own_keys_test_source() -> &'static str {
    r#"Kali.test('reflect ownKeys', () => {
  const obj = { "b": 1, "2": 2, "a": 3, "1": 4 };
  const keys = globalThis.Reflect.ownKeys(obj);
  const mixedRootKeys = globalThis["Reflect"].ownKeys(obj);
  const mixedBracketedKeys = globalThis.Reflect["ownKeys"](obj);
  const bracketedKeys = globalThis["Reflect"]["ownKeys"](obj);
  const fullyBracketedKeys = globalThis["Reflect"]["ownKeys"](obj);
  const singleQuotedKeys = globalThis['Reflect']['ownKeys'](obj);
  const frozenCallableKeys = Object.freeze(globalThis.Reflect.ownKeys)(obj);
  const frozenMixedBracketedKeys = Object.freeze(globalThis.Reflect["ownKeys"])(obj);
  const frozenBracketedKeys = Object.freeze(globalThis["Reflect"]["ownKeys"])(obj);
  const parenthesizedFrozenBracketedKeys = Object.freeze((globalThis["Reflect"]["ownKeys"]))(obj);
  const parenthesizedFrozenCallableKeys = Object.freeze((globalThis.Reflect.ownKeys))(obj);
  let syncCount = 0;
  for (const key of Reflect.ownKeys(obj)) {
    syncCount += 1;
  }
  let sequenceCount = 0;
  for (const key of (0, Reflect.ownKeys(obj))) {
    sequenceCount += 1;
  }
  let mixedSequenceCount = 0;
  for (const key of (0, globalThis["Reflect"]["ownKeys"](obj))) {
    mixedSequenceCount += 1;
  }
  if (
    keys.length !== 4 ||
    keys[0] !== '1' ||
    keys[1] !== '2' ||
    keys[2] !== 'b' ||
    keys[3] !== 'a' ||
    mixedRootKeys.length !== 4 ||
    mixedRootKeys[0] !== '1' ||
    mixedRootKeys[1] !== '2' ||
    mixedRootKeys[2] !== 'b' ||
    mixedRootKeys[3] !== 'a' ||
    mixedBracketedKeys.length !== 4 ||
    mixedBracketedKeys[0] !== '1' ||
    mixedBracketedKeys[1] !== '2' ||
    mixedBracketedKeys[2] !== 'b' ||
    mixedBracketedKeys[3] !== 'a' ||
    bracketedKeys.length !== 4 ||
    bracketedKeys[0] !== '1' ||
    bracketedKeys[1] !== '2' ||
    bracketedKeys[2] !== 'b' ||
    bracketedKeys[3] !== 'a' ||
    fullyBracketedKeys.length !== 4 ||
    fullyBracketedKeys[0] !== '1' ||
    fullyBracketedKeys[1] !== '2' ||
    fullyBracketedKeys[2] !== 'b' ||
    fullyBracketedKeys[3] !== 'a' ||
    singleQuotedKeys.length !== 4 ||
    frozenCallableKeys.length !== 4 ||
    frozenMixedBracketedKeys.length !== 4 ||
    frozenBracketedKeys.length !== 4 ||
    parenthesizedFrozenBracketedKeys.length !== 4 ||
    parenthesizedFrozenCallableKeys.length !== 4 ||
    singleQuotedKeys[0] !== '1' ||
    singleQuotedKeys[1] !== '2' ||
    singleQuotedKeys[2] !== 'b' ||
    singleQuotedKeys[3] !== 'a' ||
    syncCount !== 4 ||
    sequenceCount !== 4 ||
    mixedSequenceCount !== 4
  ) {
    throw new Error('unexpected Reflect.ownKeys ordering');
  }
});
"#
}

#[test]
fn check_accepts_reflect_own_keys_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, reflect_own_keys_source()).expect("write source");

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
fn run_accepts_reflect_own_keys_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, reflect_own_keys_source()).expect("write source");

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
    assert!(stdout.contains("reflect ownKeys ok"), "stdout: {stdout}");
}

#[test]
fn json_run_accepts_reflect_own_keys_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, reflect_own_keys_source()).expect("write source");

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
    assert!(
        json["stdout"]
            .as_str()
            .expect("run stdout")
            .contains("reflect ownKeys ok"),
        "json: {json}"
    );
}

#[test]
fn test_accepts_reflect_own_keys_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.js");
    fs::write(&source_path, reflect_own_keys_test_source()).expect("write source");

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
fn json_test_accepts_reflect_own_keys_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.js");
    fs::write(&source_path, reflect_own_keys_test_source()).expect("write source");

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
    assert_eq!(json["stdout"], "");
    assert_eq!(json["stderr"], "");
}
