use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_for_of_break_continue_run_source() -> &'static str {
    r##"function browserForOfArrayIterationBreakContinue() {
  const values = [0, 1, 1];
  const valuesAlias = values;
  const items = [];
  for (const value of valuesAlias) {
    if (!value) {
      continue;
    }
    items.push(value);
    break;
  }
  if (items.length !== 1 || items[0] !== 1) {
    throw new Error('unexpected for-of break/continue iteration semantics');
  }
  console.log('browser for-of array iteration break/continue ok');
}

browserForOfArrayIterationBreakContinue();
"##
}

fn browser_harness_for_of_break_continue_test_source() -> &'static str {
    r##"Kali.test('for-of array iteration break/continue', () => {
  const values = [0, 1, 1];
  const valuesAlias = values;
  const items = [];
  for (const value of valuesAlias) {
    if (!value) {
      continue;
    }
    items.push(value);
    break;
  }
  if (items.length !== 1 || items[0] !== 1) {
    throw new Error('unexpected for-of break/continue iteration semantics');
  }
  console.log('browser for-of array iteration break/continue ok');
});
"##
}

fn browser_harness_for_await_break_continue_run_source() -> &'static str {
    r##"async function browserForAwaitArrayIterationBreakContinue() {
  const values = [0, 1, 1];
  const valuesAlias = values;
  const items = [];
  for await (const value of valuesAlias) {
    if (!value) {
      continue;
    }
    items.push(value);
    break;
  }
  if (items.length !== 1 || items[0] !== 1) {
    throw new Error('unexpected for-await break/continue iteration semantics');
  }
  console.log('browser for-await array iteration break/continue ok');
}

browserForAwaitArrayIterationBreakContinue();
"##
}

fn browser_harness_for_await_break_continue_test_source() -> &'static str {
    r##"async function browserForAwaitArrayIterationBreakContinue() {
  const values = [0, 1, 1];
  const valuesAlias = values;
  const items = [];
  for await (const value of valuesAlias) {
    if (!value) {
      continue;
    }
    items.push(value);
    break;
  }
  if (items.length !== 1 || items[0] !== 1) {
    throw new Error('unexpected for-await break/continue iteration semantics');
  }
  console.log('browser for-await array iteration break/continue ok');
}

Kali.test('for-await array iteration break/continue', () => browserForAwaitArrayIterationBreakContinue());
"##
}

fn assert_browser_harness_break_continue(
    command: &str,
    filename: &str,
    source: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg("--max-threads")
        .arg("0")
        .arg("--max-spawned-processes")
        .arg("0");
    if json_output {
        cli.arg("--output").arg("json");
    }
    let output = cli.arg(&source_path).output().expect("run kali");

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
        if source.contains("browserForAwaitArrayIterationBreakContinue") && command == "test" {
            assert_eq!(stdout, "", "json: {json}");
        } else {
            assert!(stdout.contains("browser for-"), "json: {json}");
        }
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if command == "run" {
            assert!(stdout.contains("browser for-"), "stdout: {stdout}");
        } else {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn run_supports_for_of_break_continue_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_break_continue(
        "run",
        "main.js",
        browser_harness_for_of_break_continue_run_source(),
        false,
    );
}

#[test]
fn run_supports_for_of_break_continue_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_break_continue(
        "run",
        "main.ts",
        browser_harness_for_of_break_continue_run_source(),
        false,
    );
}

#[test]
fn run_supports_for_of_break_continue_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_break_continue(
        "run",
        "main.jsx",
        browser_harness_for_of_break_continue_run_source(),
        false,
    );
}

#[test]
fn run_supports_for_of_break_continue_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_break_continue(
        "run",
        "main.tsx",
        browser_harness_for_of_break_continue_run_source(),
        false,
    );
}

#[test]
fn test_supports_for_of_break_continue_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_break_continue(
        "test",
        "smoke.test.js",
        browser_harness_for_of_break_continue_test_source(),
        false,
    );
}

#[test]
fn test_supports_for_of_break_continue_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_break_continue(
        "test",
        "smoke.test.ts",
        browser_harness_for_of_break_continue_test_source(),
        false,
    );
}

#[test]
fn test_supports_for_of_break_continue_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_break_continue(
        "test",
        "smoke.test.jsx",
        browser_harness_for_of_break_continue_test_source(),
        false,
    );
}

#[test]
fn test_supports_for_of_break_continue_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_break_continue(
        "test",
        "smoke.test.tsx",
        browser_harness_for_of_break_continue_test_source(),
        false,
    );
}

#[test]
fn json_run_supports_for_of_break_continue_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_break_continue(
        "run",
        "main.js",
        browser_harness_for_of_break_continue_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_for_of_break_continue_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_break_continue(
        "run",
        "main.ts",
        browser_harness_for_of_break_continue_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_for_of_break_continue_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_break_continue(
        "run",
        "main.jsx",
        browser_harness_for_of_break_continue_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_for_of_break_continue_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_break_continue(
        "run",
        "main.tsx",
        browser_harness_for_of_break_continue_run_source(),
        true,
    );
}

#[test]
fn json_test_supports_for_of_break_continue_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_break_continue(
        "test",
        "smoke.test.js",
        browser_harness_for_of_break_continue_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_for_of_break_continue_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_break_continue(
        "test",
        "smoke.test.ts",
        browser_harness_for_of_break_continue_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_for_of_break_continue_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_break_continue(
        "test",
        "smoke.test.jsx",
        browser_harness_for_of_break_continue_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_for_of_break_continue_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_break_continue(
        "test",
        "smoke.test.tsx",
        browser_harness_for_of_break_continue_test_source(),
        true,
    );
}

#[test]
fn run_supports_for_await_break_continue_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_break_continue(
        "run",
        "main.js",
        browser_harness_for_await_break_continue_run_source(),
        false,
    );
}

#[test]
fn run_supports_for_await_break_continue_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_break_continue(
        "run",
        "main.ts",
        browser_harness_for_await_break_continue_run_source(),
        false,
    );
}

#[test]
fn run_supports_for_await_break_continue_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_break_continue(
        "run",
        "main.jsx",
        browser_harness_for_await_break_continue_run_source(),
        false,
    );
}

#[test]
fn run_supports_for_await_break_continue_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_break_continue(
        "run",
        "main.tsx",
        browser_harness_for_await_break_continue_run_source(),
        false,
    );
}

#[test]
fn test_supports_for_await_break_continue_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_break_continue(
        "test",
        "smoke.test.js",
        browser_harness_for_await_break_continue_test_source(),
        false,
    );
}

#[test]
fn test_supports_for_await_break_continue_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_break_continue(
        "test",
        "smoke.test.ts",
        browser_harness_for_await_break_continue_test_source(),
        false,
    );
}

#[test]
fn test_supports_for_await_break_continue_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_break_continue(
        "test",
        "smoke.test.jsx",
        browser_harness_for_await_break_continue_test_source(),
        false,
    );
}

#[test]
fn test_supports_for_await_break_continue_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_break_continue(
        "test",
        "smoke.test.tsx",
        browser_harness_for_await_break_continue_test_source(),
        false,
    );
}

#[test]
fn json_run_supports_for_await_break_continue_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_break_continue(
        "run",
        "main.js",
        browser_harness_for_await_break_continue_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_for_await_break_continue_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_break_continue(
        "run",
        "main.ts",
        browser_harness_for_await_break_continue_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_for_await_break_continue_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_break_continue(
        "run",
        "main.jsx",
        browser_harness_for_await_break_continue_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_for_await_break_continue_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_break_continue(
        "run",
        "main.tsx",
        browser_harness_for_await_break_continue_run_source(),
        true,
    );
}

#[test]
fn json_test_supports_for_await_break_continue_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_break_continue(
        "test",
        "smoke.test.js",
        browser_harness_for_await_break_continue_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_for_await_break_continue_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_break_continue(
        "test",
        "smoke.test.ts",
        browser_harness_for_await_break_continue_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_for_await_break_continue_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_break_continue(
        "test",
        "smoke.test.jsx",
        browser_harness_for_await_break_continue_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_for_await_break_continue_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_break_continue(
        "test",
        "smoke.test.tsx",
        browser_harness_for_await_break_continue_test_source(),
        true,
    );
}
