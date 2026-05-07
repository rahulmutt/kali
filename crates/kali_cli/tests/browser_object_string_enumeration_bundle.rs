use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_object_string_enumeration_source() -> &'static str {
    r##"// kali-tree-shake: browserObjectStringEnumeration
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

function browserObjectStringEnumeration() {
  const keys = [];
  for (const key of Object.keys('ab')) {
    keys.push(key);
  }
  const globalKeys = [];
  for (const key of globalThis.Object.keys('ab')) {
    globalKeys.push(key);
  }
  const mixedKeys = [];
  for (const key of globalThis.Object["keys"]('ab')) {
    mixedKeys.push(key);
  }
  const bracketedKeys = [];
  for (const key of globalThis["Object"].keys('ab')) {
    bracketedKeys.push(key);
  }
  const fullyBracketedKeys = [];
  for (const key of globalThis["Object"]["keys"]('ab')) {
    fullyBracketedKeys.push(key);
  }

  const values = [];
  for (const value of Object.values('ab')) {
    values.push(value);
  }
  const globalValues = [];
  for (const value of globalThis.Object.values('ab')) {
    globalValues.push(value);
  }
  const mixedValues = [];
  for (const value of globalThis.Object["values"]('ab')) {
    mixedValues.push(value);
  }
  const bracketedValues = [];
  for (const value of globalThis["Object"].values('ab')) {
    bracketedValues.push(value);
  }
  const fullyBracketedValues = [];
  for (const value of globalThis["Object"]["values"]('ab')) {
    fullyBracketedValues.push(value);
  }

  const entries = [];
  for (const entry of Object.entries('ab')) {
    entries.push(entry);
  }
  const globalEntries = [];
  for (const entry of globalThis.Object.entries('ab')) {
    globalEntries.push(entry);
  }
  const mixedEntries = [];
  for (const entry of globalThis.Object["entries"]('ab')) {
    mixedEntries.push(entry);
  }
  const bracketedEntries = [];
  for (const entry of globalThis["Object"].entries('ab')) {
    bracketedEntries.push(entry);
  }
  const fullyBracketedEntries = [];
  for (const entry of globalThis["Object"]["entries"]('ab')) {
    fullyBracketedEntries.push(entry);
  }

  assertObjectKeysIteration(keys);
  assertObjectKeysIteration(globalKeys);
  assertObjectKeysIteration(mixedKeys);
  assertObjectKeysIteration(bracketedKeys);
  assertObjectKeysIteration(fullyBracketedKeys);
  assertObjectValuesIteration(values);
  assertObjectValuesIteration(globalValues);
  assertObjectValuesIteration(mixedValues);
  assertObjectValuesIteration(bracketedValues);
  assertObjectValuesIteration(fullyBracketedValues);
  assertObjectEntriesIteration(entries);
  assertObjectEntriesIteration(globalEntries);
  assertObjectEntriesIteration(mixedEntries);
  assertObjectEntriesIteration(bracketedEntries);
  assertObjectEntriesIteration(fullyBracketedEntries);
}
"##
}

fn assert_browser_bundle_object_string_enumeration(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_object_string_enumeration_source(),
    )
    .expect("write source");

    let mut command = Command::new(kali_bin());
    command
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser");
    if json_output {
        command.arg("--output").arg("json");
    }
    let output = command.arg(&source_path).output().expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if json_output {
        let envelope: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(envelope["schemaVersion"], 1);
        assert_eq!(envelope["command"], "build");
        assert_eq!(envelope["success"], true);
        assert!(envelope["errors"]
            .as_array()
            .expect("errors array")
            .is_empty());
    }

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_eq!(metadata["apiSurface"], "browser");
    assert_eq!(metadata["artifactKind"], "bundle");

    let harness_path = bundle_dir
        .parent()
        .expect("bundle root parent")
        .join("browser-bundle-smoke.mjs");
    let harness = kali_runtime::browser_bundle_harness_script(
        "app",
        false,
        r#"const mod = await import(bundleJs.href);
await mod.browserObjectStringEnumeration();
"#,
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = kali_runtime::browser_harness_command_parts_for(
        std::env::var("KALI_BROWSER_BUNDLE_HARNESS_COMMAND")
            .ok()
            .as_deref(),
    );
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(&bundle_dir)
        .args(&harness_command)
        .arg(&harness_path)
        .output()
        .expect("run browser bundle harness");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).is_empty(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn build_emits_string_primitive_object_enumeration_semantics_in_js_input() {
    assert_browser_bundle_object_string_enumeration("app.js", false);
}

#[test]
fn build_emits_string_primitive_object_enumeration_semantics_in_ts_input() {
    assert_browser_bundle_object_string_enumeration("app.ts", false);
}

#[test]
fn build_emits_string_primitive_object_enumeration_semantics_in_jsx_input() {
    assert_browser_bundle_object_string_enumeration("app.jsx", false);
}

#[test]
fn build_emits_string_primitive_object_enumeration_semantics_in_tsx_input() {
    assert_browser_bundle_object_string_enumeration("app.tsx", false);
}

#[test]
fn json_build_emits_string_primitive_object_enumeration_semantics_in_js_input() {
    assert_browser_bundle_object_string_enumeration("app.js", true);
}

#[test]
fn json_build_emits_string_primitive_object_enumeration_semantics_in_ts_input() {
    assert_browser_bundle_object_string_enumeration("app.ts", true);
}

#[test]
fn json_build_emits_string_primitive_object_enumeration_semantics_in_jsx_input() {
    assert_browser_bundle_object_string_enumeration("app.jsx", true);
}

#[test]
fn json_build_emits_string_primitive_object_enumeration_semantics_in_tsx_input() {
    assert_browser_bundle_object_string_enumeration("app.tsx", true);
}

fn browser_bundle_object_string_enumeration_await_source() -> &'static str {
    r##"// kali-tree-shake: browserObjectStringEnumerationAwait
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

export async function browserObjectStringEnumerationAwait() {
  const keys = [];
  for await (const key of Object.keys('ab')) {
    keys.push(key);
  }
  const globalKeys = [];
  for await (const key of globalThis.Object.keys('ab')) {
    globalKeys.push(key);
  }
  const mixedKeys = [];
  for await (const key of globalThis.Object["keys"]('ab')) {
    mixedKeys.push(key);
  }
  const bracketedKeys = [];
  for await (const key of globalThis["Object"].keys('ab')) {
    bracketedKeys.push(key);
  }
  const fullyBracketedKeys = [];
  for await (const key of globalThis["Object"]["keys"]('ab')) {
    fullyBracketedKeys.push(key);
  }

  const values = [];
  for await (const value of Object.values('ab')) {
    values.push(value);
  }
  const globalValues = [];
  for await (const value of globalThis.Object.values('ab')) {
    globalValues.push(value);
  }
  const mixedValues = [];
  for await (const value of globalThis.Object["values"]('ab')) {
    mixedValues.push(value);
  }
  const bracketedValues = [];
  for await (const value of globalThis["Object"].values('ab')) {
    bracketedValues.push(value);
  }
  const fullyBracketedValues = [];
  for await (const value of globalThis["Object"]["values"]('ab')) {
    fullyBracketedValues.push(value);
  }

  const entries = [];
  for await (const entry of Object.entries('ab')) {
    entries.push(entry);
  }
  const globalEntries = [];
  for await (const entry of globalThis.Object.entries('ab')) {
    globalEntries.push(entry);
  }
  const mixedEntries = [];
  for await (const entry of globalThis.Object["entries"]('ab')) {
    mixedEntries.push(entry);
  }
  const bracketedEntries = [];
  for await (const entry of globalThis["Object"].entries('ab')) {
    bracketedEntries.push(entry);
  }
  const fullyBracketedEntries = [];
  for await (const entry of globalThis["Object"]["entries"]('ab')) {
    fullyBracketedEntries.push(entry);
  }

  assertObjectKeysIteration(keys);
  assertObjectKeysIteration(globalKeys);
  assertObjectKeysIteration(mixedKeys);
  assertObjectKeysIteration(bracketedKeys);
  assertObjectKeysIteration(fullyBracketedKeys);
  assertObjectValuesIteration(values);
  assertObjectValuesIteration(globalValues);
  assertObjectValuesIteration(mixedValues);
  assertObjectValuesIteration(bracketedValues);
  assertObjectValuesIteration(fullyBracketedValues);
  assertObjectEntriesIteration(entries);
  assertObjectEntriesIteration(globalEntries);
  assertObjectEntriesIteration(mixedEntries);
  assertObjectEntriesIteration(bracketedEntries);
  assertObjectEntriesIteration(fullyBracketedEntries);
}
"##
}

fn assert_browser_bundle_object_string_enumeration_await(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_object_string_enumeration_await_source(),
    )
    .expect("write source");

    let mut command = Command::new(kali_bin());
    command
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser");
    if json_output {
        command.arg("--output").arg("json");
    }
    let output = command.arg(&source_path).output().expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if json_output {
        let envelope: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(envelope["schemaVersion"], 1);
        assert_eq!(envelope["command"], "build");
        assert_eq!(envelope["success"], true);
        assert!(envelope["errors"]
            .as_array()
            .expect("errors array")
            .is_empty());
    }

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_eq!(metadata["apiSurface"], "browser");
    assert_eq!(metadata["artifactKind"], "bundle");

    let harness_path = bundle_dir
        .parent()
        .expect("bundle root parent")
        .join("browser-bundle-smoke.mjs");
    let harness = kali_runtime::browser_bundle_harness_script(
        "app",
        false,
        r#"const mod = await import(bundleJs.href);
await mod.browserObjectStringEnumerationAwait();
"#,
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = kali_runtime::browser_harness_command_parts_for(
        std::env::var("KALI_BROWSER_BUNDLE_HARNESS_COMMAND")
            .ok()
            .as_deref(),
    );
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(&bundle_dir)
        .args(&harness_command)
        .arg(&harness_path)
        .output()
        .expect("run browser bundle harness");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).is_empty(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn build_emits_for_await_string_primitive_object_enumeration_semantics_in_js_input() {
    assert_browser_bundle_object_string_enumeration_await("app.js", false);
}

#[test]
fn build_emits_for_await_string_primitive_object_enumeration_semantics_in_ts_input() {
    assert_browser_bundle_object_string_enumeration_await("app.ts", false);
}

#[test]
fn build_emits_for_await_string_primitive_object_enumeration_semantics_in_jsx_input() {
    assert_browser_bundle_object_string_enumeration_await("app.jsx", false);
}

#[test]
fn build_emits_for_await_string_primitive_object_enumeration_semantics_in_tsx_input() {
    assert_browser_bundle_object_string_enumeration_await("app.tsx", false);
}

#[test]
fn json_build_emits_for_await_string_primitive_object_enumeration_semantics_in_js_input() {
    assert_browser_bundle_object_string_enumeration_await("app.js", true);
}

#[test]
fn json_build_emits_for_await_string_primitive_object_enumeration_semantics_in_ts_input() {
    assert_browser_bundle_object_string_enumeration_await("app.ts", true);
}

#[test]
fn json_build_emits_for_await_string_primitive_object_enumeration_semantics_in_jsx_input() {
    assert_browser_bundle_object_string_enumeration_await("app.jsx", true);
}

#[test]
fn json_build_emits_for_await_string_primitive_object_enumeration_semantics_in_tsx_input() {
    assert_browser_bundle_object_string_enumeration_await("app.tsx", true);
}
