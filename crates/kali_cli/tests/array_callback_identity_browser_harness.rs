use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_array_callback_identity_source(command: &str) -> String {
    let body = r#"function browserArrayCallbackIdentitySlices() {
  const observed = [];
  for (const item of [1, 2].map((value) => value)) {
    observed.push(item);
  }
  for (const item of [1, 2].filter((value) => value)) {
    observed.push(item);
  }
  for (const item of Array.from([1, 2].filter((value) => value))) {
    observed.push(item);
  }
  for (const item of [...[1, 2].filter((value) => value)]) {
    observed.push(item);
  }
  for (const item of [1, 2].flatMap((value) => [value])) {
    observed.push(item);
  }
  console.log(`some:${[0, 1].some((value) => value)}`);
  console.log(`every:${[1, 0].every((value) => value)}`);
  if (observed.join(",") !== "1,2,1,2,1,2,1,2,1,2") {
    throw new Error('unexpected array callback identity semantics');
  }
  console.log(observed.join("\n"));
}
browserArrayCallbackIdentitySlices();
"#;

    match command {
        "test" => format!("Kali.test('array callback identity slices', () => {{\n{body}}});\n"),
        _ => body.to_string(),
    }
}

fn assert_browser_harness_array_callback_identity_succeeds(
    command: &str,
    filename: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_array_callback_identity_source(command),
    )
    .expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.env(kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV, "node")
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

    assert!(output.status.success(), "command unexpectedly failed");

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
    }
}

#[test]
fn run_supports_array_callback_identity_slices_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_array_callback_identity_succeeds("run", "main.js", false);
}

#[test]
fn test_supports_array_callback_identity_slices_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_array_callback_identity_succeeds("test", "smoke.test.js", false);
}

#[test]
fn json_run_supports_array_callback_identity_slices_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_array_callback_identity_succeeds("run", "main.js", true);
}

#[test]
fn json_test_supports_array_callback_identity_slices_in_browser_api_surface_with_harness_js_input()
{
    assert_browser_harness_array_callback_identity_succeeds("test", "smoke.test.js", true);
}

#[test]
fn run_supports_array_callback_identity_slices_in_browser_api_surface_with_harness_ts_input() {
    assert_browser_harness_array_callback_identity_succeeds("run", "main.ts", false);
}

#[test]
fn test_supports_array_callback_identity_slices_in_browser_api_surface_with_harness_ts_input() {
    assert_browser_harness_array_callback_identity_succeeds("test", "smoke.test.ts", false);
}

#[test]
fn json_run_supports_array_callback_identity_slices_in_browser_api_surface_with_harness_ts_input() {
    assert_browser_harness_array_callback_identity_succeeds("run", "main.ts", true);
}

#[test]
fn json_test_supports_array_callback_identity_slices_in_browser_api_surface_with_harness_ts_input()
{
    assert_browser_harness_array_callback_identity_succeeds("test", "smoke.test.ts", true);
}

#[test]
fn run_supports_array_callback_identity_slices_in_browser_api_surface_with_harness_jsx_input() {
    assert_browser_harness_array_callback_identity_succeeds("run", "main.jsx", false);
}

#[test]
fn test_supports_array_callback_identity_slices_in_browser_api_surface_with_harness_jsx_input() {
    assert_browser_harness_array_callback_identity_succeeds("test", "smoke.test.jsx", false);
}

#[test]
fn json_run_supports_array_callback_identity_slices_in_browser_api_surface_with_harness_jsx_input()
{
    assert_browser_harness_array_callback_identity_succeeds("run", "main.jsx", true);
}

#[test]
fn json_test_supports_array_callback_identity_slices_in_browser_api_surface_with_harness_jsx_input()
{
    assert_browser_harness_array_callback_identity_succeeds("test", "smoke.test.jsx", true);
}

#[test]
fn run_supports_array_callback_identity_slices_in_browser_api_surface_with_harness_tsx_input() {
    assert_browser_harness_array_callback_identity_succeeds("run", "main.tsx", false);
}

#[test]
fn test_supports_array_callback_identity_slices_in_browser_api_surface_with_harness_tsx_input() {
    assert_browser_harness_array_callback_identity_succeeds("test", "smoke.test.tsx", false);
}

#[test]
fn json_run_supports_array_callback_identity_slices_in_browser_api_surface_with_harness_tsx_input()
{
    assert_browser_harness_array_callback_identity_succeeds("run", "main.tsx", true);
}

#[test]
fn json_test_supports_array_callback_identity_slices_in_browser_api_surface_with_harness_tsx_input()
{
    assert_browser_harness_array_callback_identity_succeeds("test", "smoke.test.tsx", true);
}
