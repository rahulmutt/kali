use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_math_sinh_cosh_tanh_run_source() -> &'static str {
    "const zero = 0; console.log(Math.sinh(zero)); console.log(Math.cosh(zero)); console.log(Math.tanh(zero));\n"
}

fn browser_harness_math_sinh_cosh_tanh_test_source() -> &'static str {
    r#"Kali.test('math sinh/cosh/tanh zero identities', () => {
  const zero = 0;
  console.log(Math.sinh(zero));
  console.log(Math.cosh(zero));
  console.log(Math.tanh(zero));
});
"#
}

fn assert_browser_harness_math_sinh_cosh_tanh(
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
        }
        let stdout = json["stdout"].as_str().expect("stdout string");
        assert!(stdout.contains("0\n"), "json: {json}");
        assert!(stdout.contains("1\n"), "json: {json}");
        assert_eq!(json["stderr"], "");
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("0\n"), "stdout: {stdout}");
        assert!(stdout.contains("1\n"), "stdout: {stdout}");
    }
}

#[test]
fn run_supports_math_sinh_cosh_and_tanh_zero_identities_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_harness_math_sinh_cosh_tanh(
        "run",
        "main.ts",
        browser_harness_math_sinh_cosh_tanh_run_source(),
        false,
    );
}

#[test]
fn run_supports_math_sinh_cosh_and_tanh_zero_identities_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_harness_math_sinh_cosh_tanh(
        "run",
        "main.js",
        browser_harness_math_sinh_cosh_tanh_run_source(),
        false,
    );
}

#[test]
fn test_supports_math_sinh_cosh_and_tanh_zero_identities_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_harness_math_sinh_cosh_tanh(
        "test",
        "smoke.test.ts",
        browser_harness_math_sinh_cosh_tanh_test_source(),
        false,
    );
}

#[test]
fn test_supports_math_sinh_cosh_and_tanh_zero_identities_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_harness_math_sinh_cosh_tanh(
        "test",
        "smoke.test.js",
        browser_harness_math_sinh_cosh_tanh_test_source(),
        false,
    );
}

#[test]
fn run_supports_math_sinh_cosh_and_tanh_zero_identities_when_browser_harness_is_configured_in_json_ts_input(
) {
    assert_browser_harness_math_sinh_cosh_tanh(
        "run",
        "main.ts",
        browser_harness_math_sinh_cosh_tanh_run_source(),
        true,
    );
}

#[test]
fn run_supports_math_sinh_cosh_and_tanh_zero_identities_when_browser_harness_is_configured_in_json_js_input(
) {
    assert_browser_harness_math_sinh_cosh_tanh(
        "run",
        "main.js",
        browser_harness_math_sinh_cosh_tanh_run_source(),
        true,
    );
}

#[test]
fn test_supports_math_sinh_cosh_and_tanh_zero_identities_when_browser_harness_is_configured_in_json_ts_input(
) {
    assert_browser_harness_math_sinh_cosh_tanh(
        "test",
        "smoke.test.ts",
        browser_harness_math_sinh_cosh_tanh_test_source(),
        true,
    );
}

#[test]
fn test_supports_math_sinh_cosh_and_tanh_zero_identities_when_browser_harness_is_configured_in_json_js_input(
) {
    assert_browser_harness_math_sinh_cosh_tanh(
        "test",
        "smoke.test.js",
        browser_harness_math_sinh_cosh_tanh_test_source(),
        true,
    );
}

#[test]
fn run_supports_math_sinh_cosh_and_tanh_zero_identities_when_browser_harness_is_configured_in_jsx_input(
) {
    assert_browser_harness_math_sinh_cosh_tanh(
        "run",
        "main.jsx",
        browser_harness_math_sinh_cosh_tanh_run_source(),
        false,
    );
}

#[test]
fn run_supports_math_sinh_cosh_and_tanh_zero_identities_when_browser_harness_is_configured_in_tsx_input(
) {
    assert_browser_harness_math_sinh_cosh_tanh(
        "run",
        "main.tsx",
        browser_harness_math_sinh_cosh_tanh_run_source(),
        false,
    );
}

#[test]
fn test_supports_math_sinh_cosh_and_tanh_zero_identities_when_browser_harness_is_configured_in_jsx_input(
) {
    assert_browser_harness_math_sinh_cosh_tanh(
        "test",
        "smoke.test.jsx",
        browser_harness_math_sinh_cosh_tanh_test_source(),
        false,
    );
}

#[test]
fn test_supports_math_sinh_cosh_and_tanh_zero_identities_when_browser_harness_is_configured_in_tsx_input(
) {
    assert_browser_harness_math_sinh_cosh_tanh(
        "test",
        "smoke.test.tsx",
        browser_harness_math_sinh_cosh_tanh_test_source(),
        false,
    );
}

#[test]
fn run_supports_math_sinh_cosh_and_tanh_zero_identities_when_browser_harness_is_configured_in_json_jsx_input(
) {
    assert_browser_harness_math_sinh_cosh_tanh(
        "run",
        "main.jsx",
        browser_harness_math_sinh_cosh_tanh_run_source(),
        true,
    );
}

#[test]
fn run_supports_math_sinh_cosh_and_tanh_zero_identities_when_browser_harness_is_configured_in_json_tsx_input(
) {
    assert_browser_harness_math_sinh_cosh_tanh(
        "run",
        "main.tsx",
        browser_harness_math_sinh_cosh_tanh_run_source(),
        true,
    );
}

#[test]
fn test_supports_math_sinh_cosh_and_tanh_zero_identities_when_browser_harness_is_configured_in_json_jsx_input(
) {
    assert_browser_harness_math_sinh_cosh_tanh(
        "test",
        "smoke.test.jsx",
        browser_harness_math_sinh_cosh_tanh_test_source(),
        true,
    );
}

#[test]
fn test_supports_math_sinh_cosh_and_tanh_zero_identities_when_browser_harness_is_configured_in_json_tsx_input(
) {
    assert_browser_harness_math_sinh_cosh_tanh(
        "test",
        "smoke.test.tsx",
        browser_harness_math_sinh_cosh_tanh_test_source(),
        true,
    );
}
