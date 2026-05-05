use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn for_await_sequence_source() -> &'static str {
    r##"// kali-tree-shake: forAwaitArrayIterationSequenceWrapper
let count = 0;
for await (const item of (0, [(0, 1), (0, 2)])) {
  console.log(++count);
}
"##
}

fn assert_browser_harness_for_await_sequence_wrapper(command: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, for_await_sequence_source()).expect("write source");

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
        assert!(json["stdout"]
            .as_str()
            .expect("stdout string")
            .contains("1"));
        assert!(json["stdout"]
            .as_str()
            .expect("stdout string")
            .contains("2"));
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("1"), "stdout: {stdout}");
        assert!(stdout.contains("2"), "stdout: {stdout}");
    }
}

#[test]
fn run_supports_for_await_array_iteration_with_sequence_wrappers_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_harness_for_await_sequence_wrapper("run", false);
}

#[test]
fn test_supports_for_await_array_iteration_with_sequence_wrappers_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_harness_for_await_sequence_wrapper("test", false);
}

#[test]
fn json_run_supports_for_await_array_iteration_with_sequence_wrappers_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_harness_for_await_sequence_wrapper("run", true);
}

#[test]
fn json_test_supports_for_await_array_iteration_with_sequence_wrappers_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_harness_for_await_sequence_wrapper("test", true);
}
