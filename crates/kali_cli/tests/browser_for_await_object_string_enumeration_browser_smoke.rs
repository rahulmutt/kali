use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_for_await_object_string_enumeration_source() -> &'static str {
    r##"export async function browserObjectStringEnumerationAwait() {
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
  const singleBracketedKeys = [];
  for await (const key of globalThis['Object']['keys']('ab')) {
    singleBracketedKeys.push(key);
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
  const singleBracketedValues = [];
  for await (const value of globalThis['Object']['values']('ab')) {
    singleBracketedValues.push(value);
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
  const singleBracketedEntries = [];
  for await (const entry of globalThis['Object']['entries']('ab')) {
    singleBracketedEntries.push(entry);
  }

  assertObjectKeysIteration(keys);
  assertObjectKeysIteration(globalKeys);
  assertObjectKeysIteration(mixedKeys);
  assertObjectKeysIteration(bracketedKeys);
  assertObjectKeysIteration(fullyBracketedKeys);
  assertObjectKeysIteration(singleBracketedKeys);
  assertObjectValuesIteration(values);
  assertObjectValuesIteration(globalValues);
  assertObjectValuesIteration(mixedValues);
  assertObjectValuesIteration(bracketedValues);
  assertObjectValuesIteration(fullyBracketedValues);
  assertObjectValuesIteration(singleBracketedValues);
  assertObjectEntriesIteration(entries);
  assertObjectEntriesIteration(globalEntries);
  assertObjectEntriesIteration(mixedEntries);
  assertObjectEntriesIteration(bracketedEntries);
  assertObjectEntriesIteration(fullyBracketedEntries);
  assertObjectEntriesIteration(singleBracketedEntries);
}
"##
}

fn assert_browser_for_await_object_string_enumeration_support(
    command: &str,
    filename: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_for_await_object_string_enumeration_source(),
    )
    .expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path())
        .arg(command)
        .arg("--api")
        .arg("browser");
    if command == "build" {
        cli.arg("--bundle");
    }
    if json_output {
        cli.arg("--output").arg("json");
    }

    let output = cli.arg(&source_path).output().expect("run kali");

    if command == "build" {
        assert!(
            output.status.success(),
            "stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        if json_output {
            let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
            assert_eq!(json["schemaVersion"], 1);
            assert_eq!(json["command"], command);
            assert_eq!(json["success"], true);
            assert!(json["errors"].as_array().expect("errors array").is_empty());
        }
        return;
    }

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    }
}

#[test]
fn check_supports_for_await_object_string_enumeration_in_browser_analysis_context_in_js_input() {
    assert_browser_for_await_object_string_enumeration_support("check", "main.js", false);
}

#[test]
fn json_check_supports_for_await_object_string_enumeration_in_browser_analysis_context_in_js_input()
{
    assert_browser_for_await_object_string_enumeration_support("check", "main.js", true);
}

#[test]
fn check_supports_for_await_object_string_enumeration_in_browser_analysis_context_in_ts_input() {
    assert_browser_for_await_object_string_enumeration_support("check", "main.ts", false);
}

#[test]
fn json_check_supports_for_await_object_string_enumeration_in_browser_analysis_context_in_ts_input()
{
    assert_browser_for_await_object_string_enumeration_support("check", "main.ts", true);
}

#[test]
fn check_supports_for_await_object_string_enumeration_in_browser_analysis_context_in_jsx_input() {
    assert_browser_for_await_object_string_enumeration_support("check", "main.jsx", false);
}

#[test]
fn json_check_supports_for_await_object_string_enumeration_in_browser_analysis_context_in_jsx_input(
) {
    assert_browser_for_await_object_string_enumeration_support("check", "main.jsx", true);
}

#[test]
fn check_supports_for_await_object_string_enumeration_in_browser_analysis_context_in_tsx_input() {
    assert_browser_for_await_object_string_enumeration_support("check", "main.tsx", false);
}

#[test]
fn json_check_supports_for_await_object_string_enumeration_in_browser_analysis_context_in_tsx_input(
) {
    assert_browser_for_await_object_string_enumeration_support("check", "main.tsx", true);
}

#[test]
fn build_supports_for_await_object_string_enumeration_in_browser_bundle_context_in_js_input() {
    assert_browser_for_await_object_string_enumeration_support("build", "main.js", false);
}

#[test]
fn json_build_supports_for_await_object_string_enumeration_in_browser_bundle_context_in_js_input() {
    assert_browser_for_await_object_string_enumeration_support("build", "main.js", true);
}

#[test]
fn build_supports_for_await_object_string_enumeration_in_browser_bundle_context_in_ts_input() {
    assert_browser_for_await_object_string_enumeration_support("build", "main.ts", false);
}

#[test]
fn json_build_supports_for_await_object_string_enumeration_in_browser_bundle_context_in_ts_input() {
    assert_browser_for_await_object_string_enumeration_support("build", "main.ts", true);
}

#[test]
fn build_supports_for_await_object_string_enumeration_in_browser_bundle_context_in_jsx_input() {
    assert_browser_for_await_object_string_enumeration_support("build", "main.jsx", false);
}

#[test]
fn json_build_supports_for_await_object_string_enumeration_in_browser_bundle_context_in_jsx_input()
{
    assert_browser_for_await_object_string_enumeration_support("build", "main.jsx", true);
}

#[test]
fn build_supports_for_await_object_string_enumeration_in_browser_bundle_context_in_tsx_input() {
    assert_browser_for_await_object_string_enumeration_support("build", "main.tsx", false);
}

#[test]
fn json_build_supports_for_await_object_string_enumeration_in_browser_bundle_context_in_tsx_input()
{
    assert_browser_for_await_object_string_enumeration_support("build", "main.tsx", true);
}
