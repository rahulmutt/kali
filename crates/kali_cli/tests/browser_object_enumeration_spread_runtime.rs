use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn object_enumeration_spread_source() -> &'static str {
    r##"const keys = [...Object.keys(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]))];
const entries = [...Object.entries(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]))];

if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {
  throw new Error('unexpected Object.keys spread semantics');
}
if (
  entries.length !== 2 ||
  entries[0][0] !== 'b' ||
  entries[0][1] !== 3 ||
  entries[1][0] !== 'a' ||
  entries[1][1] !== 2
) {
  throw new Error('unexpected Object.entries spread semantics');
}

for (const key of keys) {
  console.log(key);
}
for await (const entry of entries) {
  console.log(entry[0]);
  console.log(entry[1]);
}
console.log('browser object enumeration spread ok');
"##
}

fn assert_browser_requested_object_enumeration_spread(
    command: &str,
    filename: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, object_enumeration_spread_source()).expect("write source");

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
        assert!(
            stdout.contains("browser object enumeration spread ok"),
            "json: {json}"
        );
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("browser object enumeration spread ok"),
            "stdout: {stdout}"
        );
        if command == "test" {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn run_supports_object_enumeration_spread_in_browser_api_surface_with_harness_jsx_input() {
    assert_browser_requested_object_enumeration_spread("run", "main.jsx", false);
}

#[test]
fn json_run_supports_object_enumeration_spread_in_browser_api_surface_with_harness_jsx_input() {
    assert_browser_requested_object_enumeration_spread("run", "main.jsx", true);
}

#[test]
fn test_supports_object_enumeration_spread_in_browser_api_surface_with_harness_jsx_input() {
    assert_browser_requested_object_enumeration_spread("test", "smoke.test.jsx", false);
}

#[test]
fn json_test_supports_object_enumeration_spread_in_browser_api_surface_with_harness_jsx_input() {
    assert_browser_requested_object_enumeration_spread("test", "smoke.test.jsx", true);
}

#[test]
fn run_supports_object_enumeration_spread_in_browser_api_surface_with_harness_tsx_input() {
    assert_browser_requested_object_enumeration_spread("run", "main.tsx", false);
}

#[test]
fn json_run_supports_object_enumeration_spread_in_browser_api_surface_with_harness_tsx_input() {
    assert_browser_requested_object_enumeration_spread("run", "main.tsx", true);
}

#[test]
fn test_supports_object_enumeration_spread_in_browser_api_surface_with_harness_tsx_input() {
    assert_browser_requested_object_enumeration_spread("test", "smoke.test.tsx", false);
}

#[test]
fn json_test_supports_object_enumeration_spread_in_browser_api_surface_with_harness_tsx_input() {
    assert_browser_requested_object_enumeration_spread("test", "smoke.test.tsx", true);
}
