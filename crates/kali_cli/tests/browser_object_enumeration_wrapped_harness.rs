use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_wrapped_object_enumeration_run_source() -> &'static str {
    r##"function assertWrappedObjectEnumeration(keys, values, entries) {
  if (
    keys.length !== 4 ||
    keys[0] !== '1' ||
    keys[1] !== '2' ||
    keys[2] !== 'b' ||
    keys[3] !== 'a' ||
    values.length !== 4 ||
    values[0] !== 4 ||
    values[1] !== 2 ||
    values[2] !== 1 ||
    values[3] !== 3 ||
    entries.length !== 4 ||
    entries[0][0] !== '1' ||
    entries[0][1] !== 4 ||
    entries[1][0] !== '2' ||
    entries[1][1] !== 2 ||
    entries[2][0] !== 'b' ||
    entries[2][1] !== 1 ||
    entries[3][0] !== 'a' ||
    entries[3][1] !== 3
  ) {
    throw new Error('unexpected wrapped object enumeration ordering');
  }
}

function browserWrappedObjectEnumeration() {
  const wrappedConst = ({ "b": 1, "2": 2, "a": 3, "1": 4 } as const);
  const wrappedSatisfies = ({ "b": 1, "2": 2, "a": 3, "1": 4 } satisfies unknown);
  const frozenFromEntries = Object.freeze(Object.fromEntries([["b", 1], ["2", 2], ["a", 3], ["1", 4]]));

  const constKeys = Object.keys(wrappedConst);
  const constValues = Object.values(wrappedConst);
  const constEntries = Object.entries(wrappedConst);
  assertWrappedObjectEnumeration(constKeys, constValues, constEntries);

  const satisfiesKeys = Object.keys(wrappedSatisfies);
  const satisfiesValues = Object.values(wrappedSatisfies);
  const satisfiesEntries = Object.entries(wrappedSatisfies);
  assertWrappedObjectEnumeration(satisfiesKeys, satisfiesValues, satisfiesEntries);

  const frozenKeys = Object.keys(frozenFromEntries);
  const frozenValues = Object.values(frozenFromEntries);
  const frozenEntries = Object.entries(frozenFromEntries);
  assertWrappedObjectEnumeration(frozenKeys, frozenValues, frozenEntries);
  console.log('browser wrapped object enumeration ok');
}

browserWrappedObjectEnumeration();
"##
}

fn browser_harness_wrapped_object_enumeration_js_run_source() -> &'static str {
    r##"function assertWrappedObjectEnumeration(keys, values, entries) {
  if (
    keys.length !== 4 ||
    keys[0] !== '1' ||
    keys[1] !== '2' ||
    keys[2] !== 'b' ||
    keys[3] !== 'a' ||
    values.length !== 4 ||
    values[0] !== 4 ||
    values[1] !== 2 ||
    values[2] !== 1 ||
    values[3] !== 3 ||
    entries.length !== 4 ||
    entries[0][0] !== '1' ||
    entries[0][1] !== 4 ||
    entries[1][0] !== '2' ||
    entries[1][1] !== 2 ||
    entries[2][0] !== 'b' ||
    entries[2][1] !== 1 ||
    entries[3][0] !== 'a' ||
    entries[3][1] !== 3
  ) {
    throw new Error('unexpected wrapped object enumeration ordering');
  }
}

function browserWrappedObjectEnumeration() {
  const wrappedObject = { "b": 1, "2": 2, "a": 3, "1": 4 };
  const frozenFromEntries = Object.freeze(Object.fromEntries([["b", 1], ["2", 2], ["a", 3], ["1", 4]]));

  const objectKeys = Object.keys(wrappedObject);
  const objectValues = Object.values(wrappedObject);
  const objectEntries = Object.entries(wrappedObject);
  assertWrappedObjectEnumeration(objectKeys, objectValues, objectEntries);

  const spreadObjectKeys = [...Object.keys(wrappedObject)];
  const spreadObjectValues = [...Object.values(wrappedObject)];
  const spreadObjectEntries = [...Object.entries(wrappedObject)];
  assertWrappedObjectEnumeration(spreadObjectKeys, spreadObjectValues, spreadObjectEntries);

  const frozenKeys = Object.keys(frozenFromEntries);
  const frozenValues = Object.values(frozenFromEntries);
  const frozenEntries = Object.entries(frozenFromEntries);
  assertWrappedObjectEnumeration(frozenKeys, frozenValues, frozenEntries);

  const spreadFrozenKeys = [...Object.keys(frozenFromEntries)];
  const spreadFrozenValues = [...Object.values(frozenFromEntries)];
  const spreadFrozenEntries = [...Object.entries(frozenFromEntries)];
  assertWrappedObjectEnumeration(spreadFrozenKeys, spreadFrozenValues, spreadFrozenEntries);
  console.log('browser wrapped object enumeration ok');
}

browserWrappedObjectEnumeration();
"##
}

fn browser_harness_wrapped_object_enumeration_test_source() -> &'static str {
    r##"Kali.test('wrapped object enumeration', () => {
  function assertWrappedObjectEnumeration(keys, values, entries) {
    if (
      keys.length !== 4 ||
      keys[0] !== '1' ||
      keys[1] !== '2' ||
      keys[2] !== 'b' ||
      keys[3] !== 'a' ||
      values.length !== 4 ||
      values[0] !== 4 ||
      values[1] !== 2 ||
      values[2] !== 1 ||
      values[3] !== 3 ||
      entries.length !== 4 ||
      entries[0][0] !== '1' ||
      entries[0][1] !== 4 ||
      entries[1][0] !== '2' ||
      entries[1][1] !== 2 ||
      entries[2][0] !== 'b' ||
      entries[2][1] !== 1 ||
      entries[3][0] !== 'a' ||
      entries[3][1] !== 3
    ) {
      throw new Error('unexpected wrapped object enumeration ordering');
    }
  }

  const wrappedConst = ({ "b": 1, "2": 2, "a": 3, "1": 4 } as const);
  const wrappedSatisfies = ({ "b": 1, "2": 2, "a": 3, "1": 4 } satisfies unknown);
  const frozenFromEntries = Object.freeze(Object.fromEntries([["b", 1], ["2", 2], ["a", 3], ["1", 4]]));

  const constKeys = Object.keys(wrappedConst);
  const constValues = Object.values(wrappedConst);
  const constEntries = Object.entries(wrappedConst);
  assertWrappedObjectEnumeration(constKeys, constValues, constEntries);

  const satisfiesKeys = Object.keys(wrappedSatisfies);
  const satisfiesValues = Object.values(wrappedSatisfies);
  const satisfiesEntries = Object.entries(wrappedSatisfies);
  assertWrappedObjectEnumeration(satisfiesKeys, satisfiesValues, satisfiesEntries);

  const frozenKeys = Object.keys(frozenFromEntries);
  const frozenValues = Object.values(frozenFromEntries);
  const frozenEntries = Object.entries(frozenFromEntries);
  assertWrappedObjectEnumeration(frozenKeys, frozenValues, frozenEntries);

  console.log('browser wrapped object enumeration ok');
});
"##
}

fn browser_harness_wrapped_object_enumeration_js_test_source() -> &'static str {
    r##"Kali.test('wrapped object enumeration', () => {
  function assertWrappedObjectEnumeration(keys, values, entries) {
    if (
      keys.length !== 4 ||
      keys[0] !== '1' ||
      keys[1] !== '2' ||
      keys[2] !== 'b' ||
      keys[3] !== 'a' ||
      values.length !== 4 ||
      values[0] !== 4 ||
      values[1] !== 2 ||
      values[2] !== 1 ||
      values[3] !== 3 ||
      entries.length !== 4 ||
      entries[0][0] !== '1' ||
      entries[0][1] !== 4 ||
      entries[1][0] !== '2' ||
      entries[1][1] !== 2 ||
      entries[2][0] !== 'b' ||
      entries[2][1] !== 1 ||
      entries[3][0] !== 'a' ||
      entries[3][1] !== 3
    ) {
      throw new Error('unexpected wrapped object enumeration ordering');
    }
  }

  const wrappedObject = { "b": 1, "2": 2, "a": 3, "1": 4 };
  const frozenFromEntries = Object.freeze(Object.fromEntries([["b", 1], ["2", 2], ["a", 3], ["1", 4]]));

  const objectKeys = Object.keys(wrappedObject);
  const objectValues = Object.values(wrappedObject);
  const objectEntries = Object.entries(wrappedObject);
  assertWrappedObjectEnumeration(objectKeys, objectValues, objectEntries);

  const frozenKeys = Object.keys(frozenFromEntries);
  const frozenValues = Object.values(frozenFromEntries);
  const frozenEntries = Object.entries(frozenFromEntries);
  assertWrappedObjectEnumeration(frozenKeys, frozenValues, frozenEntries);

  console.log('browser wrapped object enumeration ok');
});
"##
}

fn assert_browser_harness_wrapped_object_enumeration(
    command: &str,
    filename: &str,
    source: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut output = Command::new(kali_bin());
    output
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path());
    if json_output {
        output.arg("--output").arg("json");
    }
    let output = output
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg("--max-threads")
        .arg("0")
        .arg("--max-spawned-processes")
        .arg("0")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        if command == "run" {
            assert_eq!(json["exitCode"], 0);
            assert_eq!(json["payload"]["exitCode"], 0);
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
        }
        let stdout = json["stdout"].as_str().expect("stdout string");
        assert!(
            stdout.contains("browser wrapped object enumeration ok"),
            "json: {json}"
        );
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("browser wrapped object enumeration ok"),
            "stdout: {stdout}"
        );
        if command == "test" {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn run_supports_wrapped_object_enumeration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_wrapped_object_enumeration(
        "run",
        "main.ts",
        browser_harness_wrapped_object_enumeration_run_source(),
        false,
    );
}

#[test]
fn run_supports_wrapped_object_enumeration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_wrapped_object_enumeration(
        "run",
        "main.js",
        browser_harness_wrapped_object_enumeration_js_run_source(),
        false,
    );
}

#[test]
fn run_supports_wrapped_object_enumeration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_wrapped_object_enumeration(
        "run",
        "main.jsx",
        browser_harness_wrapped_object_enumeration_run_source(),
        false,
    );
}

#[test]
fn run_supports_wrapped_object_enumeration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_wrapped_object_enumeration(
        "run",
        "main.tsx",
        browser_harness_wrapped_object_enumeration_run_source(),
        false,
    );
}

#[test]
fn test_supports_wrapped_object_enumeration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_wrapped_object_enumeration(
        "test",
        "smoke.test.ts",
        browser_harness_wrapped_object_enumeration_test_source(),
        false,
    );
}

#[test]
fn test_supports_wrapped_object_enumeration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_wrapped_object_enumeration(
        "test",
        "smoke.test.js",
        browser_harness_wrapped_object_enumeration_js_test_source(),
        false,
    );
}

#[test]
fn test_supports_wrapped_object_enumeration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_wrapped_object_enumeration(
        "test",
        "smoke.test.jsx",
        browser_harness_wrapped_object_enumeration_test_source(),
        false,
    );
}

#[test]
fn test_supports_wrapped_object_enumeration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_wrapped_object_enumeration(
        "test",
        "smoke.test.tsx",
        browser_harness_wrapped_object_enumeration_test_source(),
        false,
    );
}

#[test]
fn json_run_supports_wrapped_object_enumeration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_wrapped_object_enumeration(
        "run",
        "main.ts",
        browser_harness_wrapped_object_enumeration_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_wrapped_object_enumeration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_wrapped_object_enumeration(
        "run",
        "main.js",
        browser_harness_wrapped_object_enumeration_js_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_wrapped_object_enumeration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_wrapped_object_enumeration(
        "run",
        "main.jsx",
        browser_harness_wrapped_object_enumeration_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_wrapped_object_enumeration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_wrapped_object_enumeration(
        "run",
        "main.tsx",
        browser_harness_wrapped_object_enumeration_run_source(),
        true,
    );
}

#[test]
fn json_test_supports_wrapped_object_enumeration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_wrapped_object_enumeration(
        "test",
        "smoke.test.ts",
        browser_harness_wrapped_object_enumeration_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_wrapped_object_enumeration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_wrapped_object_enumeration(
        "test",
        "smoke.test.js",
        browser_harness_wrapped_object_enumeration_js_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_wrapped_object_enumeration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_wrapped_object_enumeration(
        "test",
        "smoke.test.jsx",
        browser_harness_wrapped_object_enumeration_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_wrapped_object_enumeration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_wrapped_object_enumeration(
        "test",
        "smoke.test.tsx",
        browser_harness_wrapped_object_enumeration_test_source(),
        true,
    );
}
