use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_object_is_alias_chain_source(command: &str) -> String {
    let body = r#"const object = { a: 1 };
const objectAlias = object;
const frozenObject = Object.freeze(object);
const array = [1, 2];
const arrayAlias = array;
const frozenArray = Object.freeze(array);
if (
  Object.is(objectAlias, object) !== true ||
  globalThis["Object"]["is"](objectAlias, object) !== true ||
  globalThis.Object["is"](objectAlias, object) !== true ||
  globalThis["Object"].is(objectAlias, object) !== true ||
  globalThis.Object.is(objectAlias, object) !== true ||
  Object["is"](objectAlias, object) !== true ||
  Object.is(frozenObject, object) !== true ||
  globalThis["Object"]["is"](frozenObject, object) !== true ||
  globalThis.Object["is"](frozenObject, object) !== true ||
  globalThis["Object"].is(frozenObject, object) !== true ||
  globalThis.Object.is(frozenObject, object) !== true ||
  Object["is"](frozenObject, object) !== true ||
  Object.is(arrayAlias, array) !== true ||
  Object.is(frozenArray, array) !== true ||
  Object.is({}, {}) !== false ||
  Object.is([], []) !== false
) {
  throw new Error('unexpected browser Object.is alias chain result');
}
"#;

    match command {
        "test" => format!("Kali.test('browser object is alias chain', () => {{\n{body}}});\n"),
        _ => format!("{body}console.log('browser object is alias chain ok');\n"),
    }
}

fn assert_browser_harness_object_is_alias_chain(command: &str, filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, browser_object_is_alias_chain_source(command)).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path());
    if json_output {
        cli.arg("--output").arg("json");
    }
    let output = cli
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
    assert_eq!(output.status.code(), Some(0));

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        if command == "run" {
            assert_eq!(json["payload"]["exitCode"], 0);
            assert_eq!(json["stdout"], "browser object is alias chain ok\n");
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
            assert_eq!(json["stdout"], "");
        }
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if command == "run" {
            assert!(
                stdout.contains("browser object is alias chain ok"),
                "stdout: {stdout}"
            );
        } else {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn run_supports_object_is_alias_chain_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_is_alias_chain("run", "main.js", false);
}

#[test]
fn test_supports_object_is_alias_chain_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_is_alias_chain("test", "smoke.test.js", false);
}

#[test]
fn json_run_supports_object_is_alias_chain_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_is_alias_chain("run", "main.js", true);
}

#[test]
fn json_test_supports_object_is_alias_chain_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_is_alias_chain("test", "smoke.test.js", true);
}

#[test]
fn run_supports_object_is_alias_chain_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_is_alias_chain("run", "main.ts", false);
}

#[test]
fn test_supports_object_is_alias_chain_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_is_alias_chain("test", "smoke.test.ts", false);
}

#[test]
fn json_run_supports_object_is_alias_chain_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_is_alias_chain("run", "main.ts", true);
}

#[test]
fn json_test_supports_object_is_alias_chain_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_is_alias_chain("test", "smoke.test.ts", true);
}

#[test]
fn run_supports_object_is_alias_chain_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_is_alias_chain("run", "main.jsx", false);
}

#[test]
fn test_supports_object_is_alias_chain_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_is_alias_chain("test", "smoke.test.jsx", false);
}

#[test]
fn json_run_supports_object_is_alias_chain_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_is_alias_chain("run", "main.jsx", true);
}

#[test]
fn json_test_supports_object_is_alias_chain_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_is_alias_chain("test", "smoke.test.jsx", true);
}

#[test]
fn run_supports_object_is_alias_chain_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_is_alias_chain("run", "main.tsx", false);
}

#[test]
fn test_supports_object_is_alias_chain_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_is_alias_chain("test", "smoke.test.tsx", false);
}

#[test]
fn json_run_supports_object_is_alias_chain_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_is_alias_chain("run", "main.tsx", true);
}

#[test]
fn json_test_supports_object_is_alias_chain_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_is_alias_chain("test", "smoke.test.tsx", true);
}
