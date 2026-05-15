use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source() -> &'static str {
    r#"function constant() { console.log(7); }
(constant as unknown)();
(constant satisfies unknown)();
"#
}

fn test_source() -> &'static str {
    r#"Kali.test('wrapped call targets', () => {
  function constant() { console.log(7); }
  (constant as unknown)();
  (constant satisfies unknown)();
});
"#
}

fn assert_wrapped_call_targets_supported(
    command: &str,
    filename: &str,
    source: &str,
    browser_harness: bool,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut output = Command::new(kali_bin());
    output.current_dir(dir.path()).arg(command);
    if browser_harness {
        output.env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node");
        output.arg("--api").arg("browser");
    }
    if json_output {
        output.arg("--output").arg("json");
    }

    let output = output.arg(&source_path).output().expect("run kali");

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
        assert!(stdout.contains("7\n7\n"), "json: {json}");
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("7\n7\n"), "stdout: {stdout}");
        if command == "test" {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn run_supports_type_assertion_and_satisfies_wrapped_call_targets_in_ts_input() {
    assert_wrapped_call_targets_supported("run", "main.ts", run_source(), false, false);
}

#[test]
fn test_supports_type_assertion_and_satisfies_wrapped_call_targets_in_ts_input() {
    assert_wrapped_call_targets_supported("test", "smoke.test.ts", test_source(), false, false);
}

#[test]
fn json_run_supports_type_assertion_and_satisfies_wrapped_call_targets_when_browser_harness_is_configured_in_tsx_input(
) {
    assert_wrapped_call_targets_supported("run", "main.tsx", run_source(), true, true);
}

#[test]
fn json_test_supports_type_assertion_and_satisfies_wrapped_call_targets_when_browser_harness_is_configured_in_tsx_input(
) {
    assert_wrapped_call_targets_supported("test", "smoke.test.tsx", test_source(), true, true);
}
