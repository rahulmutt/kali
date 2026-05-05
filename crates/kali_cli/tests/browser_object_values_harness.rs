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
  assertObjectValuesIteration(collected);
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
  assertObjectValuesIteration(collected);
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
