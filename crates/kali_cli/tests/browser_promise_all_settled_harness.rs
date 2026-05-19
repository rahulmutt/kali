use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

use kali_common::promise_all_settled_browser_body_source;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_promise_all_settled_run_source() -> String {
    format!(
        "async function browserPromiseAllSettled() {{\n{}\n}}\n\nasync function main() {{\n  await browserPromiseAllSettled();\n  console.log('browser promise allSettled ok');\n}}\n\nmain();\n",
        promise_all_settled_browser_body_source()
    )
}

fn browser_promise_all_settled_test_source() -> String {
    format!(
        "async function browserPromiseAllSettled() {{\n{}\n}}\n\nKali.test('browser promise allSettled', () => browserPromiseAllSettled());\n",
        promise_all_settled_browser_body_source()
    )
}
fn assert_browser_requested_promise_all_settled(command: &str, filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        browser_promise_all_settled_test_source()
    } else {
        browser_promise_all_settled_run_source()
    };
    fs::write(&source_path, source).expect("write source");

    let mut command_line = Command::new(kali_bin());
    command_line
        .current_dir(dir.path())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node");
    if json_output {
        command_line.arg("--output").arg("json");
    }
    let output = command_line
        .arg(command)
        .arg("--api")
        .arg("browser")
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
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        if command == "run" {
            assert_eq!(json["payload"]["exitCode"], 0);
            assert!(
                json["stdout"]
                    .as_str()
                    .expect("stdout")
                    .contains("browser promise allSettled ok"),
                "json: {json}"
            );
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
            assert_eq!(json["stdout"], "");
        }
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if command == "run" {
        assert!(
            stdout.contains("browser promise allSettled ok"),
            "stdout: {stdout}"
        );
    } else {
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn run_supports_promise_all_settled_in_js_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("run", "main.js", false);
}

#[test]
fn run_supports_promise_all_settled_in_ts_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("run", "main.ts", false);
}

#[test]
fn run_supports_promise_all_settled_in_jsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("run", "main.jsx", false);
}

#[test]
fn run_supports_promise_all_settled_in_tsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("run", "main.tsx", false);
}

#[test]
fn test_supports_promise_all_settled_in_js_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("test", "smoke.test.js", false);
}

#[test]
fn test_supports_promise_all_settled_in_ts_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("test", "smoke.test.ts", false);
}

#[test]
fn test_supports_promise_all_settled_in_jsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("test", "smoke.test.jsx", false);
}

#[test]
fn test_supports_promise_all_settled_in_tsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("test", "smoke.test.tsx", false);
}

#[test]
fn json_run_supports_promise_all_settled_in_js_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("run", "main.js", true);
}

#[test]
fn json_run_supports_promise_all_settled_in_ts_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("run", "main.ts", true);
}

#[test]
fn json_run_supports_promise_all_settled_in_jsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("run", "main.jsx", true);
}

#[test]
fn json_run_supports_promise_all_settled_in_tsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("run", "main.tsx", true);
}

#[test]
fn json_test_supports_promise_all_settled_in_js_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("test", "smoke.test.js", true);
}

#[test]
fn json_test_supports_promise_all_settled_in_ts_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("test", "smoke.test.ts", true);
}

#[test]
fn json_test_supports_promise_all_settled_in_jsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("test", "smoke.test.jsx", true);
}

#[test]
fn json_test_supports_promise_all_settled_in_tsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("test", "smoke.test.tsx", true);
}
