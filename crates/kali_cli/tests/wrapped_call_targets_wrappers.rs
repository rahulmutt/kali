use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source() -> &'static str {
    r#"const add = (left, right) => left + right;
console.log((add as unknown)(1, 2));
console.log((add satisfies unknown)(3, 4));
"#
}

fn test_source() -> &'static str {
    r#"Kali.test('wrapped call targets', () => {
  const add = (left, right) => left + right;
  console.log((add as unknown)(1, 2));
  console.log((add satisfies unknown)(3, 4));
});
"#
}

fn assert_wrapped_call_targets_rejected(
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

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        assert!(json["errors"].as_array().expect("errors array").iter().any(|error| {
            error["code"] == "E5506"
                && error["message"]
                    .as_str()
                    .is_some_and(|message| message.contains("wrapped call targets using type assertions or satisfies expressions are unavailable"))
        }));
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(
            stderr.contains("wrapped call targets using type assertions or satisfies expressions are unavailable"),
            "stderr: {stderr}"
        );
    }
}

#[test]
fn run_rejects_satisfies_wrapped_call_targets_in_ts_input() {
    assert_wrapped_call_targets_rejected("run", "main.ts", run_source(), false, false);
}

#[test]
fn test_rejects_satisfies_wrapped_call_targets_in_ts_input() {
    assert_wrapped_call_targets_rejected("test", "smoke.test.ts", test_source(), false, false);
}

#[test]
fn json_run_rejects_satisfies_wrapped_call_targets_when_browser_harness_is_configured_in_tsx_input()
{
    assert_wrapped_call_targets_rejected("run", "main.tsx", run_source(), true, true);
}

#[test]
fn json_test_rejects_satisfies_wrapped_call_targets_when_browser_harness_is_configured_in_tsx_input(
) {
    assert_wrapped_call_targets_rejected("test", "smoke.test.tsx", test_source(), true, true);
}
