use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_object_entries_run_source() -> &'static str {
    r##"function assertObjectEntriesIteration(entries) {
  if (
    entries.length !== 2 ||
    entries[0][0] !== 'b' ||
    entries[0][1] !== 1 ||
    entries[1][0] !== 'a' ||
    entries[1][1] !== 2
  ) {
    throw new Error('unexpected Object.entries iteration semantics');
  }
}

function browserObjectEntriesIteration() {
  const values = { "b": 1, "a": 2 };
  const alias = values;
  const entries = Object.entries(alias);
  assertObjectEntriesIteration(entries);
  console.log('browser object entries iteration ok');
}

browserObjectEntriesIteration();
"##
}

fn browser_harness_object_entries_test_source() -> &'static str {
    r##"Kali.test('object entries iteration', () => {
  function assertObjectEntriesIteration(entries) {
    if (
      entries.length !== 2 ||
      entries[0][0] !== 'b' ||
      entries[0][1] !== 1 ||
      entries[1][0] !== 'a' ||
      entries[1][1] !== 2
    ) {
      throw new Error('unexpected Object.entries iteration semantics');
    }
  }

  const values = { "b": 1, "a": 2 };
  const alias = values;
  const entries = Object.entries(alias);
  assertObjectEntriesIteration(entries);
  console.log('browser object entries iteration ok');
});
"##
}

fn assert_browser_harness_object_entries(
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
            stdout.contains("browser object entries iteration ok"),
            "json: {json}"
        );
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("browser object entries iteration ok"),
            "stdout: {stdout}"
        );
        if command == "test" {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn run_supports_object_entries_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_entries(
        "run",
        "main.js",
        browser_harness_object_entries_run_source(),
        false,
    );
}

#[test]
fn run_supports_object_entries_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_entries(
        "run",
        "main.ts",
        browser_harness_object_entries_run_source(),
        false,
    );
}

#[test]
fn run_supports_object_entries_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_entries(
        "run",
        "main.jsx",
        browser_harness_object_entries_run_source(),
        false,
    );
}

#[test]
fn run_supports_object_entries_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_entries(
        "run",
        "main.tsx",
        browser_harness_object_entries_run_source(),
        false,
    );
}

#[test]
fn test_supports_object_entries_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_entries(
        "test",
        "smoke.test.js",
        browser_harness_object_entries_test_source(),
        false,
    );
}

#[test]
fn test_supports_object_entries_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_entries(
        "test",
        "smoke.test.ts",
        browser_harness_object_entries_test_source(),
        false,
    );
}

#[test]
fn test_supports_object_entries_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_entries(
        "test",
        "smoke.test.jsx",
        browser_harness_object_entries_test_source(),
        false,
    );
}

#[test]
fn test_supports_object_entries_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_entries(
        "test",
        "smoke.test.tsx",
        browser_harness_object_entries_test_source(),
        false,
    );
}

#[test]
fn json_run_supports_object_entries_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_entries(
        "run",
        "main.js",
        browser_harness_object_entries_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_object_entries_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_entries(
        "run",
        "main.ts",
        browser_harness_object_entries_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_object_entries_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_entries(
        "run",
        "main.jsx",
        browser_harness_object_entries_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_object_entries_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_entries(
        "run",
        "main.tsx",
        browser_harness_object_entries_run_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_entries_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_entries(
        "test",
        "smoke.test.js",
        browser_harness_object_entries_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_entries_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_entries(
        "test",
        "smoke.test.ts",
        browser_harness_object_entries_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_entries_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_entries(
        "test",
        "smoke.test.jsx",
        browser_harness_object_entries_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_entries_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_entries(
        "test",
        "smoke.test.tsx",
        browser_harness_object_entries_test_source(),
        true,
    );
}
