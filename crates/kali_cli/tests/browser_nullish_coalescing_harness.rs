use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
}

fn browser_run_source() -> &'static str {
    "const nullValue = null ?? 1; const voidValue = void 0 ?? 2; console.log(nullValue + voidValue);\n"
}

fn browser_test_source() -> &'static str {
    "Kali.test('browser nullish coalescing', () => { const nullValue = null ?? 1; const voidValue = void 0 ?? 2; return nullValue + voidValue; });\n"
}

fn assert_browser_harness_nullish_coalescing(
    command: &str,
    filename: &str,
    source: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
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
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    }
}

#[test]
fn run_supports_nullish_coalescing_in_browser_api_surface_with_harness_input_matrix() {
    for filename in ["main.ts", "main.jsx", "main.tsx"] {
        assert_browser_harness_nullish_coalescing("run", filename, browser_run_source(), false);
    }
}

#[test]
fn json_run_supports_nullish_coalescing_in_browser_api_surface_with_harness_input_matrix() {
    for filename in ["main.ts", "main.jsx", "main.tsx"] {
        assert_browser_harness_nullish_coalescing("run", filename, browser_run_source(), true);
    }
}

#[test]
fn test_supports_nullish_coalescing_in_browser_api_surface_with_harness_input_matrix() {
    for filename in ["smoke.test.ts", "smoke.test.jsx", "smoke.test.tsx"] {
        assert_browser_harness_nullish_coalescing("test", filename, browser_test_source(), false);
    }
}

#[test]
fn json_test_supports_nullish_coalescing_in_browser_api_surface_with_harness_input_matrix() {
    for filename in ["smoke.test.ts", "smoke.test.jsx", "smoke.test.tsx"] {
        assert_browser_harness_nullish_coalescing("test", filename, browser_test_source(), true);
    }
}
