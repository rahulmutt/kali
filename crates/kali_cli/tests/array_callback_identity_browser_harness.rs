use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_array_callback_identity_source(command: &str) -> String {
    let body = r#"function browserArrayCallbackIdentitySlices() {
  const values = [1, 2];
  const observed = [];
  for (const item of values.map((value) => value)) {
    observed.push(item);
  }
  for (const item of values.filter((value) => value)) {
    observed.push(item);
  }
  for (const item of Array.from(values.filter((value) => value))) {
    observed.push(item);
  }
  for (const item of [...values.filter((value) => value)]) {
    observed.push(item);
  }
  for (const item of values.flatMap((value) => [value])) {
    observed.push(item);
  }
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

fn assert_browser_harness_array_callback_identity_rejects(
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

    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert_eq!(output.status.code(), Some(1));

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        assert_eq!(json["exitCode"], 1);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(
            !errors.is_empty(),
            "errors array should not be empty: {json}"
        );
        assert_eq!(errors[0]["code"], "E5506");
        assert!(
            errors[0]["message"]
                .as_str()
                .expect("error message")
                .contains("literal array"),
            "unexpected error message: {}",
            errors[0]["message"]
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(stderr.contains("literal array"), "unexpected stderr: {stderr}");
    }
}

#[test]
fn run_rejects_array_callback_identity_slices_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_array_callback_identity_rejects("run", "main.js", false);
}

#[test]
fn test_rejects_array_callback_identity_slices_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_array_callback_identity_rejects("test", "smoke.test.js", false);
}

#[test]
fn json_run_rejects_array_callback_identity_slices_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_array_callback_identity_rejects("run", "main.js", true);
}

#[test]
fn json_test_rejects_array_callback_identity_slices_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_array_callback_identity_rejects("test", "smoke.test.js", true);
}

#[test]
fn run_rejects_array_callback_identity_slices_in_browser_api_surface_with_harness_ts_input() {
    assert_browser_harness_array_callback_identity_rejects("run", "main.ts", false);
}

#[test]
fn test_rejects_array_callback_identity_slices_in_browser_api_surface_with_harness_ts_input() {
    assert_browser_harness_array_callback_identity_rejects("test", "smoke.test.ts", false);
}

#[test]
fn json_run_rejects_array_callback_identity_slices_in_browser_api_surface_with_harness_ts_input() {
    assert_browser_harness_array_callback_identity_rejects("run", "main.ts", true);
}

#[test]
fn json_test_rejects_array_callback_identity_slices_in_browser_api_surface_with_harness_ts_input() {
    assert_browser_harness_array_callback_identity_rejects("test", "smoke.test.ts", true);
}

#[test]
fn run_rejects_array_callback_identity_slices_in_browser_api_surface_with_harness_jsx_input() {
    assert_browser_harness_array_callback_identity_rejects("run", "main.jsx", false);
}

#[test]
fn test_rejects_array_callback_identity_slices_in_browser_api_surface_with_harness_jsx_input() {
    assert_browser_harness_array_callback_identity_rejects("test", "smoke.test.jsx", false);
}

#[test]
fn json_run_rejects_array_callback_identity_slices_in_browser_api_surface_with_harness_jsx_input() {
    assert_browser_harness_array_callback_identity_rejects("run", "main.jsx", true);
}

#[test]
fn json_test_rejects_array_callback_identity_slices_in_browser_api_surface_with_harness_jsx_input()
{
    assert_browser_harness_array_callback_identity_rejects("test", "smoke.test.jsx", true);
}

#[test]
fn run_rejects_array_callback_identity_slices_in_browser_api_surface_with_harness_tsx_input() {
    assert_browser_harness_array_callback_identity_rejects("run", "main.tsx", false);
}

#[test]
fn test_rejects_array_callback_identity_slices_in_browser_api_surface_with_harness_tsx_input() {
    assert_browser_harness_array_callback_identity_rejects("test", "smoke.test.tsx", false);
}

#[test]
fn json_run_rejects_array_callback_identity_slices_in_browser_api_surface_with_harness_tsx_input() {
    assert_browser_harness_array_callback_identity_rejects("run", "main.tsx", true);
}

#[test]
fn json_test_rejects_array_callback_identity_slices_in_browser_api_surface_with_harness_tsx_input()
{
    assert_browser_harness_array_callback_identity_rejects("test", "smoke.test.tsx", true);
}
