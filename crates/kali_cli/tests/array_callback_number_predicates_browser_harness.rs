use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_number_predicates_source(command: &str) -> String {
    let body = r#"function browserNumberPredicatesSlices() {
  console.log([0, 1, 2].some((value) => value > 1));
  console.log([0, 1].some((value) => value > 1));
  console.log([2, 3].every((value) => value > 1));
  console.log([1, 2].every((value) => value > 1));
}
browserNumberPredicatesSlices();
"#;

    match command {
        "test" => format!("Kali.test('number predicates', () => {{\n{body}}});\n"),
        _ => body.to_string(),
    }
}

fn assert_browser_harness_number_predicates(command: &str, filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, browser_number_predicates_source(command)).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node");
    if json_output {
        cli.arg("--output").arg("json");
    }
    let output = cli
        .arg(command)
        .arg("--max-threads")
        .arg("0")
        .arg("--max-spawned-processes")
        .arg("0")
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
            assert_eq!(json["stdout"], "1\n0\n1\n0\n");
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
            assert_eq!(json["payload"]["skipped"], 0);
            assert_eq!(
                json["stdout"].as_str().expect("stdout string"),
                "1\n0\n1\n0\n"
            );
        }
        assert_eq!(json["stderr"], "");
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.starts_with("1\n0\n1\n0\n"), "stdout: {stdout}");
        if command == "test" {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn run_supports_number_predicates_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_number_predicates("run", "main.js", false);
}

#[test]
fn test_supports_number_predicates_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_number_predicates("test", "smoke.test.js", false);
}

#[test]
fn json_run_supports_number_predicates_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_number_predicates("run", "main.js", true);
}

#[test]
fn json_test_supports_number_predicates_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_number_predicates("test", "smoke.test.js", true);
}

#[test]
fn run_supports_number_predicates_in_browser_api_surface_with_harness_ts_input() {
    assert_browser_harness_number_predicates("run", "main.ts", false);
}

#[test]
fn test_supports_number_predicates_in_browser_api_surface_with_harness_ts_input() {
    assert_browser_harness_number_predicates("test", "smoke.test.ts", false);
}

#[test]
fn json_run_supports_number_predicates_in_browser_api_surface_with_harness_ts_input() {
    assert_browser_harness_number_predicates("run", "main.ts", true);
}

#[test]
fn json_test_supports_number_predicates_in_browser_api_surface_with_harness_ts_input() {
    assert_browser_harness_number_predicates("test", "smoke.test.ts", true);
}

#[test]
fn run_supports_number_predicates_in_browser_api_surface_with_harness_jsx_input() {
    assert_browser_harness_number_predicates("run", "main.jsx", false);
}

#[test]
fn test_supports_number_predicates_in_browser_api_surface_with_harness_jsx_input() {
    assert_browser_harness_number_predicates("test", "smoke.test.jsx", false);
}

#[test]
fn json_run_supports_number_predicates_in_browser_api_surface_with_harness_jsx_input() {
    assert_browser_harness_number_predicates("run", "main.jsx", true);
}

#[test]
fn json_test_supports_number_predicates_in_browser_api_surface_with_harness_jsx_input() {
    assert_browser_harness_number_predicates("test", "smoke.test.jsx", true);
}

#[test]
fn run_supports_number_predicates_in_browser_api_surface_with_harness_tsx_input() {
    assert_browser_harness_number_predicates("run", "main.tsx", false);
}

#[test]
fn test_supports_number_predicates_in_browser_api_surface_with_harness_tsx_input() {
    assert_browser_harness_number_predicates("test", "smoke.test.tsx", false);
}

#[test]
fn json_run_supports_number_predicates_in_browser_api_surface_with_harness_tsx_input() {
    assert_browser_harness_number_predicates("run", "main.tsx", true);
}

#[test]
fn json_test_supports_number_predicates_in_browser_api_surface_with_harness_tsx_input() {
    assert_browser_harness_number_predicates("test", "smoke.test.tsx", true);
}
