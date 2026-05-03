use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn for_await_as_const_source() -> &'static str {
    r##"// kali-tree-shake: forAwaitArrayIterationAsConstWrapper
const value = 2;
for await (const item of ([1, (value)] as const)) {
  console.log(item);
}
"##
}

fn for_await_satisfies_source() -> &'static str {
    r##"// kali-tree-shake: forAwaitArrayIterationSatisfiesWrapper
const value = 2;
for await (const item of ([1, (value)] satisfies readonly [1, 2])) {
  console.log(item);
}
"##
}

fn assert_browser_for_await_array_iteration(output: &str) {
    assert!(output.contains("1"), "output: {output}");
    assert!(output.contains("2"), "output: {output}");
}

fn assert_browser_harness_for_await_wrapper(command: &str, source: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, source).expect("write source");

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
        assert!(
            json["stdout"]
                .as_str()
                .expect("stdout string")
                .contains("1"),
            "json: {json}"
        );
        assert!(
            json["stdout"]
                .as_str()
                .expect("stdout string")
                .contains("2"),
            "json: {json}"
        );
        assert_eq!(json["stderr"], "");
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_browser_for_await_array_iteration(&stdout);
    }
}

#[test]
fn run_supports_for_await_array_iteration_lowering_with_as_const_wrapper_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_harness_for_await_wrapper("run", for_await_as_const_source(), false);
}

#[test]
fn test_supports_for_await_array_iteration_lowering_with_as_const_wrapper_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_harness_for_await_wrapper("test", for_await_as_const_source(), false);
}

#[test]
fn json_run_supports_for_await_array_iteration_lowering_with_as_const_wrapper_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_harness_for_await_wrapper("run", for_await_as_const_source(), true);
}

#[test]
fn json_test_supports_for_await_array_iteration_lowering_with_as_const_wrapper_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_harness_for_await_wrapper("test", for_await_as_const_source(), true);
}

#[test]
fn run_supports_for_await_array_iteration_lowering_with_satisfies_wrapper_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_harness_for_await_wrapper("run", for_await_satisfies_source(), false);
}

#[test]
fn test_supports_for_await_array_iteration_lowering_with_satisfies_wrapper_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_harness_for_await_wrapper("test", for_await_satisfies_source(), false);
}

#[test]
fn json_run_supports_for_await_array_iteration_lowering_with_satisfies_wrapper_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_harness_for_await_wrapper("run", for_await_satisfies_source(), true);
}

#[test]
fn json_test_supports_for_await_array_iteration_lowering_with_satisfies_wrapper_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_harness_for_await_wrapper("test", for_await_satisfies_source(), true);
}
