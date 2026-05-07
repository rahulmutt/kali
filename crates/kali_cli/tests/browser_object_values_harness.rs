use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_object_values_run_source() -> &'static str {
    r##"function assertObjectValuesIteration(values) {
  if (values.length !== 2 || values[0] !== 1 || values[1] !== 2) {
    throw new Error('unexpected Object.values iteration semantics');
  }
}

function browserObjectValuesIteration() {
  const values = { "b": 1, "a": 2 };
  const alias = values;
  const collected = [];
  for (const value of Object.values(alias)) {
    collected.push(value);
  }
  assertObjectValuesIteration(collected);
  console.log('browser object values iteration ok');
}

browserObjectValuesIteration();
"##
}

fn browser_harness_object_values_test_source() -> &'static str {
    r##"Kali.test('object values iteration', () => {
  function assertObjectValuesIteration(values) {
    if (values.length !== 2 || values[0] !== 1 || values[1] !== 2) {
      throw new Error('unexpected Object.values iteration semantics');
    }
  }

  const values = { "b": 1, "a": 2 };
  const alias = values;
  const collected = [];
  for (const value of Object.values(alias)) {
    collected.push(value);
  }
  assertObjectValuesIteration(collected);
  console.log('browser object values iteration ok');
});
"##
}

fn browser_harness_global_object_values_run_source() -> &'static str {
    r##"function assertObjectValuesIteration(values) {
  if (values.length !== 2 || values[0] !== 1 || values[1] !== 2) {
    throw new Error('unexpected Object.values iteration semantics');
  }
}

function browserGlobalObjectValuesIteration() {
  const values = { "b": 1, "a": 2 };
  const alias = values;
  const collected = [];
  for (const value of globalThis.Object.values(alias)) {
    collected.push(value);
  }
  const mixed = [];
  for (const value of globalThis.Object["values"](alias)) {
    mixed.push(value);
  }
  const mixedBracketed = [];
  for (const value of globalThis["Object"].values(alias)) {
    mixedBracketed.push(value);
  }
  const bracketed = [];
  for (const value of globalThis["Object"]["values"](alias)) {
    bracketed.push(value);
  }
  assertObjectValuesIteration(collected);
  assertObjectValuesIteration(mixed);
  assertObjectValuesIteration(mixedBracketed);
  assertObjectValuesIteration(bracketed);
  console.log('browser object values iteration ok');
}

browserGlobalObjectValuesIteration();
"##
}

fn browser_harness_global_object_values_test_source() -> &'static str {
    r##"Kali.test('global object values iteration', () => {
  function assertObjectValuesIteration(values) {
    if (values.length !== 2 || values[0] !== 1 || values[1] !== 2) {
      throw new Error('unexpected Object.values iteration semantics');
    }
  }

  const values = { "b": 1, "a": 2 };
  const alias = values;
  const collected = [];
  for (const value of globalThis.Object.values(alias)) {
    collected.push(value);
  }
  const mixed = [];
  for (const value of globalThis.Object["values"](alias)) {
    mixed.push(value);
  }
  const mixedBracketed = [];
  for (const value of globalThis["Object"].values(alias)) {
    mixedBracketed.push(value);
  }
  const bracketed = [];
  for (const value of globalThis["Object"]["values"](alias)) {
    bracketed.push(value);
  }
  assertObjectValuesIteration(collected);
  assertObjectValuesIteration(mixed);
  assertObjectValuesIteration(mixedBracketed);
  assertObjectValuesIteration(bracketed);
  console.log('browser object values iteration ok');
});
"##
}

fn assert_browser_harness_object_values(
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
            stdout.contains("browser object values iteration ok"),
            "json: {json}"
        );
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("browser object values iteration ok"),
            "stdout: {stdout}"
        );
        if command == "test" {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

fn assert_browser_harness_object_values_spread(
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
            stdout.contains("browser object values spread iteration ok"),
            "json: {json}"
        );
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("browser object values spread iteration ok"),
            "stdout: {stdout}"
        );
        if command == "test" {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn run_supports_object_values_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_values(
        "run",
        "main.js",
        browser_harness_object_values_run_source(),
        false,
    );
}

#[test]
fn run_supports_object_values_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_values(
        "run",
        "main.ts",
        browser_harness_object_values_run_source(),
        false,
    );
}

#[test]
fn run_supports_object_values_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_values(
        "run",
        "main.jsx",
        browser_harness_object_values_run_source(),
        false,
    );
}

#[test]
fn run_supports_object_values_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_values(
        "run",
        "main.tsx",
        browser_harness_object_values_run_source(),
        false,
    );
}

#[test]
fn test_supports_object_values_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.js",
        browser_harness_object_values_test_source(),
        false,
    );
}

#[test]
fn test_supports_object_values_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.ts",
        browser_harness_object_values_test_source(),
        false,
    );
}

#[test]
fn test_supports_object_values_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.jsx",
        browser_harness_object_values_test_source(),
        false,
    );
}

#[test]
fn test_supports_object_values_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.tsx",
        browser_harness_object_values_test_source(),
        false,
    );
}

#[test]
fn json_run_supports_object_values_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_values(
        "run",
        "main.js",
        browser_harness_object_values_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_object_values_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_values(
        "run",
        "main.ts",
        browser_harness_object_values_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_object_values_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_values(
        "run",
        "main.jsx",
        browser_harness_object_values_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_object_values_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_values(
        "run",
        "main.tsx",
        browser_harness_object_values_run_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_values_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.js",
        browser_harness_object_values_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_values_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.ts",
        browser_harness_object_values_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_values_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.jsx",
        browser_harness_object_values_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_values_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.tsx",
        browser_harness_object_values_test_source(),
        true,
    );
}

#[test]
fn run_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_values(
        "run",
        "main.js",
        browser_harness_global_object_values_run_source(),
        false,
    );
}

#[test]
fn run_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_values(
        "run",
        "main.ts",
        browser_harness_global_object_values_run_source(),
        false,
    );
}

#[test]
fn run_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_values(
        "run",
        "main.jsx",
        browser_harness_global_object_values_run_source(),
        false,
    );
}

#[test]
fn run_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_values(
        "run",
        "main.tsx",
        browser_harness_global_object_values_run_source(),
        false,
    );
}

#[test]
fn test_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.js",
        browser_harness_global_object_values_test_source(),
        false,
    );
}

#[test]
fn test_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.ts",
        browser_harness_global_object_values_test_source(),
        false,
    );
}

#[test]
fn test_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.jsx",
        browser_harness_global_object_values_test_source(),
        false,
    );
}

#[test]
fn test_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.tsx",
        browser_harness_global_object_values_test_source(),
        false,
    );
}

#[test]
fn json_run_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_js_input()
{
    assert_browser_harness_object_values(
        "run",
        "main.js",
        browser_harness_global_object_values_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_ts_input()
{
    assert_browser_harness_object_values(
        "run",
        "main.ts",
        browser_harness_global_object_values_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_jsx_input(
) {
    assert_browser_harness_object_values(
        "run",
        "main.jsx",
        browser_harness_global_object_values_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_tsx_input(
) {
    assert_browser_harness_object_values(
        "run",
        "main.tsx",
        browser_harness_global_object_values_run_source(),
        true,
    );
}

#[test]
fn json_test_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.js",
        browser_harness_global_object_values_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.ts",
        browser_harness_global_object_values_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_jsx_input(
) {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.jsx",
        browser_harness_global_object_values_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_direct_object_values_iteration_when_browser_harness_is_configured_in_tsx_input(
) {
    assert_browser_harness_object_values(
        "test",
        "smoke.test.tsx",
        browser_harness_global_object_values_test_source(),
        true,
    );
}

fn browser_harness_object_values_spread_source(test_mode: bool) -> String {
    if test_mode {
        return r#"Kali.test('object values spread iteration', () => {
  function assertObjectValuesSpreadIteration(values) {
    if (values.length !== 2 || values[0] !== 3 || values[1] !== 2) {
      throw new Error('unexpected Object.values spread iteration semantics');
    }
  }

  const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);
  const bracketedFromEntries = globalThis["Object"]["fromEntries"]([["b", 1], ["a", 2], ["b", 3]]);
  const collected = [...Object.values(fromEntries)];
  const globalCollected = [...globalThis.Object.values(fromEntries)];
  const bracketedCollected = [...Object.values(bracketedFromEntries)];
  const mixedCollected = [...globalThis.Object["values"](fromEntries)];
  const mixedBracketedCollected = [...globalThis["Object"].values(fromEntries)];
  const singleBracketedCollected = [...globalThis['Object']['values'](fromEntries)];
  const singleBracketedPropertyCollected = [...globalThis['Object'].values(fromEntries)];
  const bracketedAliasCollected = [...globalThis["Object"]["values"](fromEntries)];
  const bracketedAliasFromEntriesCollected = [...globalThis["Object"]["values"](bracketedFromEntries)];
  assertObjectValuesSpreadIteration(collected);
  assertObjectValuesSpreadIteration(globalCollected);
  assertObjectValuesSpreadIteration(bracketedCollected);
  assertObjectValuesSpreadIteration(mixedCollected);
  assertObjectValuesSpreadIteration(mixedBracketedCollected);
  assertObjectValuesSpreadIteration(singleBracketedCollected);
  assertObjectValuesSpreadIteration(singleBracketedPropertyCollected);
  assertObjectValuesSpreadIteration(bracketedAliasCollected);
  assertObjectValuesSpreadIteration(bracketedAliasFromEntriesCollected);
  console.log('browser object values spread iteration ok');
});
"#
        .to_string();
    }

    r#"function browserObjectValuesSpreadIteration() {
  function assertObjectValuesSpreadIteration(values) {
    if (values.length !== 2 || values[0] !== 3 || values[1] !== 2) {
      throw new Error('unexpected Object.values spread iteration semantics');
    }
  }

  const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);
  const bracketedFromEntries = globalThis["Object"]["fromEntries"]([["b", 1], ["a", 2], ["b", 3]]);
  const collected = [...Object.values(fromEntries)];
  const globalCollected = [...globalThis.Object.values(fromEntries)];
  const bracketedCollected = [...Object.values(bracketedFromEntries)];
  const mixedCollected = [...globalThis.Object["values"](fromEntries)];
  const mixedBracketedCollected = [...globalThis["Object"].values(fromEntries)];
  const singleBracketedCollected = [...globalThis['Object']['values'](fromEntries)];
  const singleBracketedPropertyCollected = [...globalThis['Object'].values(fromEntries)];
  const bracketedAliasCollected = [...globalThis["Object"]["values"](fromEntries)];
  const bracketedAliasFromEntriesCollected = [...globalThis["Object"]["values"](bracketedFromEntries)];
  assertObjectValuesSpreadIteration(collected);
  assertObjectValuesSpreadIteration(globalCollected);
  assertObjectValuesSpreadIteration(bracketedCollected);
  assertObjectValuesSpreadIteration(mixedCollected);
  assertObjectValuesSpreadIteration(mixedBracketedCollected);
  assertObjectValuesSpreadIteration(singleBracketedCollected);
  assertObjectValuesSpreadIteration(singleBracketedPropertyCollected);
  assertObjectValuesSpreadIteration(bracketedAliasCollected);
  assertObjectValuesSpreadIteration(bracketedAliasFromEntriesCollected);
  console.log('browser object values spread iteration ok');
}

browserObjectValuesSpreadIteration();
"#
    .to_string()
}

fn browser_harness_object_values_frozen_spread_source(test_mode: bool) -> String {
    browser_harness_object_values_spread_source(test_mode).replace(
        "  const fromEntries = Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]);",
        "  const fromEntries = Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]));",
    )
}

#[test]
fn run_supports_object_values_spread_iteration_when_browser_harness_is_configured() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_browser_harness_object_values_spread(
            "run",
            filename,
            &browser_harness_object_values_spread_source(false),
            false,
        );
        assert_browser_harness_object_values_spread(
            "run",
            filename,
            &browser_harness_object_values_spread_source(false),
            true,
        );
    }
}

#[test]
fn run_supports_frozen_object_values_spread_iteration_when_browser_harness_is_configured() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_browser_harness_object_values_spread(
            "run",
            filename,
            &browser_harness_object_values_frozen_spread_source(false),
            false,
        );
        assert_browser_harness_object_values_spread(
            "run",
            filename,
            &browser_harness_object_values_frozen_spread_source(false),
            true,
        );
    }
}

#[test]
fn test_supports_object_values_spread_iteration_when_browser_harness_is_configured() {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_browser_harness_object_values_spread(
            "test",
            filename,
            &browser_harness_object_values_spread_source(true),
            false,
        );
        assert_browser_harness_object_values_spread(
            "test",
            filename,
            &browser_harness_object_values_spread_source(true),
            true,
        );
    }
}

#[test]
fn test_supports_frozen_object_values_spread_iteration_when_browser_harness_is_configured() {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_browser_harness_object_values_spread(
            "test",
            filename,
            &browser_harness_object_values_frozen_spread_source(true),
            false,
        );
        assert_browser_harness_object_values_spread(
            "test",
            filename,
            &browser_harness_object_values_frozen_spread_source(true),
            true,
        );
    }
}
