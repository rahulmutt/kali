use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_object_keys_run_source() -> &'static str {
    r##"function assertObjectKeysIteration(keys) {
  if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {
    throw new Error('unexpected Object.keys iteration semantics');
  }
}

function browserObjectKeysIteration() {
  const values = { "b": 1, "a": 2 };
  const alias = values;
  const keys = [];
  for (const key of Object.keys(alias)) {
    keys.push(key);
  }
  assertObjectKeysIteration(keys);
  console.log('browser object keys iteration ok');
}

browserObjectKeysIteration();
"##
}

fn browser_harness_object_keys_test_source() -> &'static str {
    r##"Kali.test('object keys iteration', () => {
  function assertObjectKeysIteration(keys) {
    if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {
      throw new Error('unexpected Object.keys iteration semantics');
    }
  }

  const values = { "b": 1, "a": 2 };
  const alias = values;
  const keys = [];
  for (const key of Object.keys(alias)) {
    keys.push(key);
  }
  assertObjectKeysIteration(keys);
  console.log('browser object keys iteration ok');
});
"##
}

fn browser_harness_object_keys_const_bound_run_source() -> &'static str {
    r##"function assertObjectKeysIteration(keys) {
  if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {
    throw new Error('unexpected Object.keys iteration semantics');
  }
}

function browserObjectKeysConstBoundIteration() {
  const values = { "b": 1, "a": 2 };
  const keys = [];
  for (const key of Object.keys(values)) {
    keys.push(key);
  }
  assertObjectKeysIteration(keys);
  console.log('browser object keys iteration ok');
}

browserObjectKeysConstBoundIteration();
"##
}

fn browser_harness_object_keys_const_bound_test_source() -> &'static str {
    r##"Kali.test('const-bound object keys iteration', () => {
  function assertObjectKeysIteration(keys) {
    if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {
      throw new Error('unexpected Object.keys iteration semantics');
    }
  }

  const values = { "b": 1, "a": 2 };
  const keys = [];
  for (const key of Object.keys(values)) {
    keys.push(key);
  }
  assertObjectKeysIteration(keys);
  console.log('browser object keys iteration ok');
});
"##
}

fn browser_harness_object_keys_direct_run_source() -> &'static str {
    r##"function assertObjectKeysIteration(keys) {
  if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {
    throw new Error('unexpected Object.keys iteration semantics');
  }
}

function browserDirectObjectKeysIteration() {
  const keys = [];
  for (const key of Object.keys({ "b": 1, "a": 2 })) {
    keys.push(key);
  }
  assertObjectKeysIteration(keys);
  console.log('browser object keys iteration ok');
}

browserDirectObjectKeysIteration();
"##
}

fn browser_harness_object_keys_direct_test_source() -> &'static str {
    r##"Kali.test('object keys iteration', () => {
  function assertObjectKeysIteration(keys) {
    if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {
      throw new Error('unexpected Object.keys iteration semantics');
    }
  }

  const keys = [];
  for (const key of Object.keys({ "b": 1, "a": 2 })) {
    keys.push(key);
  }
  assertObjectKeysIteration(keys);
  console.log('browser object keys iteration ok');
});
"##
}

fn browser_harness_global_object_keys_run_source() -> &'static str {
    r##"function assertObjectKeysIteration(keys) {
  if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {
    throw new Error('unexpected Object.keys iteration semantics');
  }
}

function browserGlobalObjectKeysIteration() {
  const values = { "b": 1, "a": 2 };
  const alias = values;
  const keys = [];
  for (const key of globalThis.Object.keys(alias)) {
    keys.push(key);
  }
  const mixed = [];
  for (const key of globalThis.Object["keys"](alias)) {
    mixed.push(key);
  }
  const mixedBracketed = [];
  for (const key of globalThis["Object"].keys(alias)) {
    mixedBracketed.push(key);
  }
  const bracketed = [];
  for (const key of globalThis["Object"]["keys"](alias)) {
    bracketed.push(key);
  }
  assertObjectKeysIteration(keys);
  assertObjectKeysIteration(mixed);
  assertObjectKeysIteration(mixedBracketed);
  assertObjectKeysIteration(bracketed);
  console.log('browser object keys iteration ok');
}

browserGlobalObjectKeysIteration();
"##
}

fn browser_harness_global_object_keys_test_source() -> &'static str {
    r##"Kali.test('global object keys iteration', () => {
  function assertObjectKeysIteration(keys) {
    if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {
      throw new Error('unexpected Object.keys iteration semantics');
    }
  }

  const values = { "b": 1, "a": 2 };
  const alias = values;
  const keys = [];
  for (const key of globalThis.Object.keys(alias)) {
    keys.push(key);
  }
  const mixed = [];
  for (const key of globalThis.Object["keys"](alias)) {
    mixed.push(key);
  }
  const mixedBracketed = [];
  for (const key of globalThis["Object"].keys(alias)) {
    mixedBracketed.push(key);
  }
  const bracketed = [];
  for (const key of globalThis["Object"]["keys"](alias)) {
    bracketed.push(key);
  }
  assertObjectKeysIteration(keys);
  assertObjectKeysIteration(mixed);
  assertObjectKeysIteration(mixedBracketed);
  assertObjectKeysIteration(bracketed);
  console.log('browser object keys iteration ok');
});
"##
}

fn assert_browser_harness_object_keys(
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
            stdout.contains("browser object keys iteration ok"),
            "json: {json}"
        );
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("browser object keys iteration ok"),
            "stdout: {stdout}"
        );
        if command == "test" {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn run_supports_object_keys_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_keys(
        "run",
        "main.js",
        browser_harness_object_keys_run_source(),
        false,
    );
}

#[test]
fn run_supports_object_keys_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_keys(
        "run",
        "main.ts",
        browser_harness_object_keys_run_source(),
        false,
    );
}

#[test]
fn run_supports_object_keys_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_keys(
        "run",
        "main.jsx",
        browser_harness_object_keys_run_source(),
        false,
    );
}

#[test]
fn run_supports_object_keys_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_keys(
        "run",
        "main.tsx",
        browser_harness_object_keys_run_source(),
        false,
    );
}

#[test]
fn test_supports_object_keys_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_keys(
        "test",
        "smoke.test.js",
        browser_harness_object_keys_test_source(),
        false,
    );
}

#[test]
fn test_supports_object_keys_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_keys(
        "test",
        "smoke.test.ts",
        browser_harness_object_keys_test_source(),
        false,
    );
}

#[test]
fn test_supports_object_keys_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_keys(
        "test",
        "smoke.test.jsx",
        browser_harness_object_keys_test_source(),
        false,
    );
}

#[test]
fn test_supports_object_keys_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_keys(
        "test",
        "smoke.test.tsx",
        browser_harness_object_keys_test_source(),
        false,
    );
}

#[test]
fn json_run_supports_object_keys_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_keys(
        "run",
        "main.js",
        browser_harness_object_keys_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_object_keys_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_keys(
        "run",
        "main.ts",
        browser_harness_object_keys_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_object_keys_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_keys(
        "run",
        "main.jsx",
        browser_harness_object_keys_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_object_keys_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_keys(
        "run",
        "main.tsx",
        browser_harness_object_keys_run_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_keys_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_keys(
        "test",
        "smoke.test.js",
        browser_harness_object_keys_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_keys_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_keys(
        "test",
        "smoke.test.ts",
        browser_harness_object_keys_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_keys_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_keys(
        "test",
        "smoke.test.jsx",
        browser_harness_object_keys_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_keys_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_keys(
        "test",
        "smoke.test.tsx",
        browser_harness_object_keys_test_source(),
        true,
    );
}

#[test]
fn run_supports_direct_object_keys_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_keys(
        "run",
        "main.js",
        browser_harness_object_keys_direct_run_source(),
        false,
    );
}

#[test]
fn run_supports_direct_object_keys_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_keys(
        "run",
        "main.ts",
        browser_harness_object_keys_direct_run_source(),
        false,
    );
}

#[test]
fn run_supports_direct_object_keys_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_keys(
        "run",
        "main.jsx",
        browser_harness_object_keys_direct_run_source(),
        false,
    );
}

#[test]
fn run_supports_direct_object_keys_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_keys(
        "run",
        "main.tsx",
        browser_harness_object_keys_direct_run_source(),
        false,
    );
}

#[test]
fn test_supports_direct_object_keys_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_keys(
        "test",
        "smoke.test.js",
        browser_harness_object_keys_direct_test_source(),
        false,
    );
}

#[test]
fn test_supports_direct_object_keys_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_keys(
        "test",
        "smoke.test.ts",
        browser_harness_object_keys_direct_test_source(),
        false,
    );
}

#[test]
fn test_supports_direct_object_keys_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_keys(
        "test",
        "smoke.test.jsx",
        browser_harness_object_keys_direct_test_source(),
        false,
    );
}

#[test]
fn test_supports_direct_object_keys_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_keys(
        "test",
        "smoke.test.tsx",
        browser_harness_object_keys_direct_test_source(),
        false,
    );
}

#[test]
fn json_run_supports_direct_object_keys_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_keys(
        "run",
        "main.js",
        browser_harness_object_keys_direct_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_direct_object_keys_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_keys(
        "run",
        "main.ts",
        browser_harness_object_keys_direct_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_direct_object_keys_iteration_when_browser_harness_is_configured_in_jsx_input()
{
    assert_browser_harness_object_keys(
        "run",
        "main.jsx",
        browser_harness_object_keys_direct_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_direct_object_keys_iteration_when_browser_harness_is_configured_in_tsx_input()
{
    assert_browser_harness_object_keys(
        "run",
        "main.tsx",
        browser_harness_object_keys_direct_run_source(),
        true,
    );
}

#[test]
fn json_test_supports_direct_object_keys_iteration_when_browser_harness_is_configured_in_js_input()
{
    assert_browser_harness_object_keys(
        "test",
        "smoke.test.js",
        browser_harness_object_keys_direct_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_direct_object_keys_iteration_when_browser_harness_is_configured_in_ts_input()
{
    assert_browser_harness_object_keys(
        "test",
        "smoke.test.ts",
        browser_harness_object_keys_direct_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_direct_object_keys_iteration_when_browser_harness_is_configured_in_jsx_input()
{
    assert_browser_harness_object_keys(
        "test",
        "smoke.test.jsx",
        browser_harness_object_keys_direct_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_direct_object_keys_iteration_when_browser_harness_is_configured_in_tsx_input()
{
    assert_browser_harness_object_keys(
        "test",
        "smoke.test.tsx",
        browser_harness_object_keys_direct_test_source(),
        true,
    );
}

#[test]
fn run_supports_const_bound_object_keys_iteration_when_browser_harness_is_configured_in_js_ts_jsx_tsx_input(
) {
    for command in ["run", "test"] {
        let expect_test_runner = command == "test";
        for json_output in [false, true] {
            for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
                let source = if expect_test_runner {
                    browser_harness_object_keys_const_bound_test_source()
                } else {
                    browser_harness_object_keys_const_bound_run_source()
                };
                assert_browser_harness_object_keys(command, filename, source, json_output);
            }
        }
    }
}

#[test]
fn run_supports_global_object_keys_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_keys(
        "run",
        "main.js",
        browser_harness_global_object_keys_run_source(),
        false,
    );
}

#[test]
fn test_supports_global_object_keys_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_keys(
        "test",
        "smoke.test.js",
        browser_harness_global_object_keys_test_source(),
        false,
    );
}

#[test]
fn json_run_supports_global_object_keys_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_keys(
        "run",
        "main.js",
        browser_harness_global_object_keys_run_source(),
        true,
    );
}

#[test]
fn json_test_supports_global_object_keys_iteration_when_browser_harness_is_configured_in_js_input()
{
    assert_browser_harness_object_keys(
        "test",
        "smoke.test.js",
        browser_harness_global_object_keys_test_source(),
        true,
    );
}

#[test]
fn run_supports_global_object_keys_iteration_when_browser_harness_is_configured_in_ts_jsx_tsx_input(
) {
    for filename in ["main.ts", "main.jsx", "main.tsx"] {
        assert_browser_harness_object_keys(
            "run",
            filename,
            browser_harness_global_object_keys_run_source(),
            false,
        );
    }
}

#[test]
fn test_supports_global_object_keys_iteration_when_browser_harness_is_configured_in_ts_jsx_tsx_input(
) {
    for filename in ["smoke.test.ts", "smoke.test.jsx", "smoke.test.tsx"] {
        assert_browser_harness_object_keys(
            "test",
            filename,
            browser_harness_global_object_keys_test_source(),
            false,
        );
    }
}

#[test]
fn json_run_supports_global_object_keys_iteration_when_browser_harness_is_configured_in_ts_jsx_tsx_input(
) {
    for filename in ["main.ts", "main.jsx", "main.tsx"] {
        assert_browser_harness_object_keys(
            "run",
            filename,
            browser_harness_global_object_keys_run_source(),
            true,
        );
    }
}

#[test]
fn json_test_supports_global_object_keys_iteration_when_browser_harness_is_configured_in_ts_jsx_tsx_input(
) {
    for filename in ["smoke.test.ts", "smoke.test.jsx", "smoke.test.tsx"] {
        assert_browser_harness_object_keys(
            "test",
            filename,
            browser_harness_global_object_keys_test_source(),
            true,
        );
    }
}
