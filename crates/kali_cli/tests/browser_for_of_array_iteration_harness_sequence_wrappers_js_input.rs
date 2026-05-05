use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn for_of_sequence_source() -> &'static str {
    r##"// kali-tree-shake: forOfArrayIterationSequenceWrapper
let count = 0;
for (const item of (0, [(0, 1), (0, 2)])) {
  console.log(++count);
}
"##
}

fn assert_browser_harness_for_of_sequence_wrapper_is_gated(command: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, for_of_sequence_source()).expect("write source");

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
        !output.status.success(),
        "expected sequence wrappers to remain gated"
    );

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        assert_eq!(json["exitCode"], 1);
        assert!(json["errors"]
            .as_array()
            .expect("errors array")
            .iter()
            .any(|error| {
                error["code"] == "E5506"
                    && error["message"]
                        .as_str()
                        .expect("error message")
                        .contains("for-of array iteration lowering is unavailable")
            }));
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
    }
}

#[test]
fn run_rejects_for_of_array_iteration_with_sequence_wrappers_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_harness_for_of_sequence_wrapper_is_gated("run", false);
}

#[test]
fn test_rejects_for_of_array_iteration_with_sequence_wrappers_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_harness_for_of_sequence_wrapper_is_gated("test", false);
}

#[test]
fn json_run_rejects_for_of_array_iteration_with_sequence_wrappers_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_harness_for_of_sequence_wrapper_is_gated("run", true);
}

#[test]
fn json_test_rejects_for_of_array_iteration_with_sequence_wrappers_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_harness_for_of_sequence_wrapper_is_gated("test", true);
}
