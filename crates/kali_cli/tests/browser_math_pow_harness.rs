use std::{fs, process::Command};

use kali_common::{math_pow_alias_inventory_source, math_pow_invocation_lines};
use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_math_pow_run_source() -> String {
    format!(
        "const exponent = 3; const alias = exponent; {}\n",
        math_pow_invocation_lines(&math_pow_alias_inventory_source(), "")
    )
}

fn browser_harness_math_pow_test_source() -> String {
    format!(
        r#"Kali.test('math pow alias chain', () => {{
  const exponent = 3;
  const alias = exponent;
  {}
}});
"#,
        math_pow_invocation_lines(&math_pow_alias_inventory_source(), "")
    )
}

fn assert_browser_harness_math_pow<S: AsRef<str>>(
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
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node");
    if json_output {
        output.arg("--output").arg("json");
    }
    let output = output
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
            assert_eq!(json["payload"]["skipped"], 0);
        }
        let stdout = json["stdout"].as_str().expect("stdout string");
        assert!(stdout.contains("8\n8\n8"), "json: {json}");
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("8\n8\n8"), "stdout: {stdout}");
        if command == "test" {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn run_supports_math_pow_alias_chain_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_pow(
        "run",
        "main.js",
        browser_harness_math_pow_run_source(),
        false,
    );
}

#[test]
fn run_supports_math_pow_alias_chain_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_pow(
        "run",
        "main.ts",
        browser_harness_math_pow_run_source(),
        false,
    );
}

#[test]
fn run_supports_math_pow_alias_chain_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_math_pow(
        "run",
        "main.jsx",
        browser_harness_math_pow_run_source(),
        false,
    );
}

#[test]
fn run_supports_math_pow_alias_chain_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_math_pow(
        "run",
        "main.tsx",
        browser_harness_math_pow_run_source(),
        false,
    );
}

#[test]
fn test_supports_math_pow_alias_chain_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_pow(
        "test",
        "smoke.test.js",
        browser_harness_math_pow_test_source(),
        false,
    );
}

#[test]
fn test_supports_math_pow_alias_chain_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_pow(
        "test",
        "smoke.test.ts",
        browser_harness_math_pow_test_source(),
        false,
    );
}

#[test]
fn test_supports_math_pow_alias_chain_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_math_pow(
        "test",
        "smoke.test.jsx",
        browser_harness_math_pow_test_source(),
        false,
    );
}

#[test]
fn test_supports_math_pow_alias_chain_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_math_pow(
        "test",
        "smoke.test.tsx",
        browser_harness_math_pow_test_source(),
        false,
    );
}

#[test]
fn json_run_supports_math_pow_alias_chain_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_pow(
        "run",
        "main.js",
        browser_harness_math_pow_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_math_pow_alias_chain_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_pow(
        "run",
        "main.ts",
        browser_harness_math_pow_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_math_pow_alias_chain_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_math_pow(
        "run",
        "main.jsx",
        browser_harness_math_pow_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_math_pow_alias_chain_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_math_pow(
        "run",
        "main.tsx",
        browser_harness_math_pow_run_source(),
        true,
    );
}

#[test]
fn json_test_supports_math_pow_alias_chain_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_pow(
        "test",
        "smoke.test.js",
        browser_harness_math_pow_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_math_pow_alias_chain_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_pow(
        "test",
        "smoke.test.ts",
        browser_harness_math_pow_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_math_pow_alias_chain_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_math_pow(
        "test",
        "smoke.test.jsx",
        browser_harness_math_pow_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_math_pow_alias_chain_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_math_pow(
        "test",
        "smoke.test.tsx",
        browser_harness_math_pow_test_source(),
        true,
    );
}
