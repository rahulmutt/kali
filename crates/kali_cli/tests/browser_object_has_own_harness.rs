use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

use kali_common::{
    object_has_own_frozen_callable_condition_source, object_has_own_frozen_callable_source,
    object_has_own_property_call_binding_source,
    object_has_own_property_call_frozen_callable_condition_source,
    object_has_own_property_call_frozen_callable_source,
};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_object_has_own_run_source() -> String {
    let frozen_callable_condition_source = format!(
        "{} || {}",
        object_has_own_frozen_callable_condition_source("wrapped", r#""a""#),
        object_has_own_property_call_frozen_callable_condition_source("wrapped", r#""a""#)
    );
    let has_own_property_call_binding_source =
        object_has_own_property_call_binding_source("hasOwnPropertyCall");
    let frozen_callable_source = format!(
        "{} {}",
        object_has_own_frozen_callable_source(),
        object_has_own_property_call_frozen_callable_source()
    );
    format!(
        r#"const object = Object.fromEntries([["a", 1], ["b", 2]]);
const alias = object;
const hasOwn = Object.hasOwn;
{}
{}
const wrapped = (0, alias);
if (
  !Object.hasOwn(wrapped, "a") ||
  !hasOwn(wrapped, "a") ||
  !Object["hasOwn"](wrapped, "a") ||
  !globalThis.Object["hasOwn"](wrapped, "a") ||
  !globalThis["Object"]["hasOwn"](wrapped, "a") ||
  !globalThis.Object["hasOwn"](wrapped, "a") ||
  !globalThis["Object"].hasOwn(wrapped, "a") ||
  {} ||
  !Object.prototype.hasOwnProperty.call(wrapped, "a") ||
  !Object["hasOwnProperty"].call(wrapped, "a") ||
  !Object["hasOwnProperty"]["call"](wrapped, "a") ||
  !globalThis.Object.hasOwnProperty.call(wrapped, "a") ||
  !globalThis["Object"]["hasOwnProperty"].call(wrapped, "a") ||
  !globalThis["Object"]["hasOwnProperty"]["call"](wrapped, "a") ||
  !globalThis["Object"].hasOwnProperty.call(wrapped, "a") ||
  !hasOwnPropertyCall(wrapped, "a") ||
  !globalThis.Object.prototype["hasOwnProperty"]["call"](wrapped, "a") ||
  !globalThis.Object.prototype.hasOwnProperty["call"](wrapped, "a") ||
  !globalThis.Object["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||
  !globalThis["Object"].prototype["hasOwnProperty"]["call"](wrapped, "a") ||
  !globalThis["Object"].prototype.hasOwnProperty.call(wrapped, "a") ||
  !globalThis.Object["prototype"].hasOwnProperty.call(wrapped, "a") ||
  !globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||
  !globalThis["Object"]["prototype"].hasOwnProperty["call"](wrapped, "a")
) {{
  throw new Error('unexpected browser Object.hasOwn result');
}}
console.log('browser object hasOwn ok');
"#,
        has_own_property_call_binding_source,
        frozen_callable_source,
        frozen_callable_condition_source
    )
}

fn browser_harness_object_has_own_test_source() -> String {
    let frozen_callable_condition_source = format!(
        "{} || {}",
        object_has_own_frozen_callable_condition_source("wrapped", r#""a""#),
        object_has_own_property_call_frozen_callable_condition_source("wrapped", r#""a""#)
    );
    let has_own_property_call_binding_source =
        object_has_own_property_call_binding_source("hasOwnPropertyCall");
    let frozen_callable_source = format!(
        "{} {}",
        object_has_own_frozen_callable_source(),
        object_has_own_property_call_frozen_callable_source()
    );
    format!(
        r#"Kali.test('object hasOwn primitive literals', () => {{
  const object = Object.fromEntries([["a", 1], ["b", 2]]);
  const alias = object;
  const hasOwn = Object.hasOwn;
  {}
  {}
  const wrapped = (0, alias);
  if (
    !Object.hasOwn(wrapped, "a") ||
    !hasOwn(wrapped, "a") ||
    !Object["hasOwn"](wrapped, "a") ||
    !globalThis.Object["hasOwn"](wrapped, "a") ||
    !globalThis["Object"]["hasOwn"](wrapped, "a") ||
    !globalThis.Object["hasOwn"](wrapped, "a") ||
    !globalThis["Object"].hasOwn(wrapped, "a") ||
    {} ||
    !Object.prototype.hasOwnProperty.call(wrapped, "a") ||
    !Object["hasOwnProperty"].call(wrapped, "a") ||
    !Object["hasOwnProperty"]["call"](wrapped, "a") ||
    !globalThis.Object.hasOwnProperty.call(wrapped, "a") ||
    !globalThis["Object"]["hasOwnProperty"].call(wrapped, "a") ||
    !globalThis["Object"]["hasOwnProperty"]["call"](wrapped, "a") ||
    !globalThis["Object"].hasOwnProperty.call(wrapped, "a") ||
    !hasOwnPropertyCall(wrapped, "a") ||
    !globalThis.Object.prototype["hasOwnProperty"]["call"](wrapped, "a") ||
    !globalThis.Object.prototype.hasOwnProperty["call"](wrapped, "a") ||
    !globalThis.Object["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||
    !globalThis["Object"].prototype["hasOwnProperty"]["call"](wrapped, "a") ||
    !globalThis["Object"].prototype.hasOwnProperty.call(wrapped, "a") ||
    !globalThis.Object["prototype"].hasOwnProperty.call(wrapped, "a") ||
    !globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](wrapped, "a") ||
    !globalThis["Object"]["prototype"].hasOwnProperty["call"](wrapped, "a")
  ) {{
    throw new Error('unexpected browser Object.hasOwn result');
  }}
  console.log('browser object hasOwn ok');
}});
"#,
        has_own_property_call_binding_source,
        frozen_callable_source,
        frozen_callable_condition_source
    )
}

fn assert_browser_harness_object_has_own<S: AsRef<str>>(
    command: &str,
    filename: &str,
    source: S,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source.as_ref()).expect("write source");

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
        assert!(stdout.contains("browser object hasOwn ok"), "json: {json}");
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("browser object hasOwn ok"),
            "stdout: {stdout}"
        );
        if command == "test" {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn run_supports_object_has_own_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_has_own(
        "run",
        "main.js",
        &browser_harness_object_has_own_run_source(),
        false,
    );
}

#[test]
fn run_supports_object_has_own_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_has_own(
        "run",
        "main.jsx",
        &browser_harness_object_has_own_run_source(),
        false,
    );
}

#[test]
fn run_supports_object_has_own_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_has_own(
        "run",
        "main.ts",
        &browser_harness_object_has_own_run_source(),
        false,
    );
}

#[test]
fn run_supports_object_has_own_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_has_own(
        "run",
        "main.tsx",
        &browser_harness_object_has_own_run_source(),
        false,
    );
}

#[test]
fn test_supports_object_has_own_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_has_own(
        "test",
        "smoke.test.js",
        &browser_harness_object_has_own_test_source(),
        false,
    );
}

#[test]
fn test_supports_object_has_own_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_has_own(
        "test",
        "smoke.test.jsx",
        &browser_harness_object_has_own_test_source(),
        false,
    );
}

#[test]
fn test_supports_object_has_own_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_has_own(
        "test",
        "smoke.test.ts",
        &browser_harness_object_has_own_test_source(),
        false,
    );
}

#[test]
fn test_supports_object_has_own_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_has_own(
        "test",
        "smoke.test.tsx",
        &browser_harness_object_has_own_test_source(),
        false,
    );
}

#[test]
fn json_run_supports_object_has_own_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_has_own(
        "run",
        "main.js",
        browser_harness_object_has_own_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_object_has_own_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_has_own(
        "run",
        "main.jsx",
        browser_harness_object_has_own_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_object_has_own_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_has_own(
        "run",
        "main.ts",
        browser_harness_object_has_own_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_object_has_own_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_has_own(
        "run",
        "main.tsx",
        browser_harness_object_has_own_run_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_has_own_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_has_own(
        "test",
        "smoke.test.js",
        browser_harness_object_has_own_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_has_own_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_has_own(
        "test",
        "smoke.test.jsx",
        browser_harness_object_has_own_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_has_own_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_has_own(
        "test",
        "smoke.test.ts",
        browser_harness_object_has_own_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_has_own_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_has_own(
        "test",
        "smoke.test.tsx",
        browser_harness_object_has_own_test_source(),
        true,
    );
}
