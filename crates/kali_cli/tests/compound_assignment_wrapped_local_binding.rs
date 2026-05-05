use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn wrapped_compound_assignment_run_source() -> &'static str {
    "let value = 1; ((value)) += 2; ((value)) -= 1; ((value)) *= 5; ((value)) /= 2; ((value)) %= 4; ((value)) **= 3; console.log(value);\n"
}

fn wrapped_compound_assignment_test_source() -> &'static str {
    r#"Kali.test('wrapped compound assignment', () => {
  let value = 1;
  ((value)) += 2;
  ((value)) -= 1;
  ((value)) *= 5;
  ((value)) /= 2;
  ((value)) %= 4;
  ((value)) **= 3;
  return value;
});
"#
}

fn assert_wrapped_compound_assignment(
    command: &str,
    filename: &str,
    source: &'static str,
    json_output: bool,
    browser_harness: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path());
    if browser_harness {
        cli.env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node");
    }
    if json_output {
        cli.arg("--output").arg("json");
    }
    cli.arg(command);
    if browser_harness {
        cli.arg("--api").arg("browser");
        cli.arg("--max-threads").arg("0");
        cli.arg("--max-spawned-processes").arg("0");
    }
    cli.arg(&source_path);

    let output = cli.output().expect("run kali");
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
        assert!(json["errors"].as_array().expect("errors array").is_empty());
        if command == "run" {
            assert_eq!(json["exitCode"], 0);
        } else {
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
        }
        if browser_harness {
            assert_eq!(json["payload"]["hostContract"], "browser-requested");
            assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        }
    }
}

#[test]
fn run_supports_wrapped_compound_assignment_in_js_input() {
    assert_wrapped_compound_assignment(
        "run",
        "main.js",
        wrapped_compound_assignment_run_source(),
        false,
        false,
    );
}

#[test]
fn run_supports_wrapped_compound_assignment_in_ts_input() {
    assert_wrapped_compound_assignment(
        "run",
        "main.ts",
        wrapped_compound_assignment_run_source(),
        false,
        false,
    );
}

#[test]
fn json_run_supports_wrapped_compound_assignment_in_js_input() {
    assert_wrapped_compound_assignment(
        "run",
        "main.js",
        wrapped_compound_assignment_run_source(),
        true,
        false,
    );
}

#[test]
fn json_run_supports_wrapped_compound_assignment_in_ts_input() {
    assert_wrapped_compound_assignment(
        "run",
        "main.ts",
        wrapped_compound_assignment_run_source(),
        true,
        false,
    );
}

#[test]
fn run_supports_wrapped_compound_assignment_in_browser_harness_js_input() {
    assert_wrapped_compound_assignment(
        "run",
        "main.js",
        wrapped_compound_assignment_run_source(),
        false,
        true,
    );
}

#[test]
fn run_supports_wrapped_compound_assignment_in_browser_harness_ts_input() {
    assert_wrapped_compound_assignment(
        "run",
        "main.ts",
        wrapped_compound_assignment_run_source(),
        false,
        true,
    );
}

#[test]
fn test_supports_wrapped_compound_assignment_in_js_input() {
    assert_wrapped_compound_assignment(
        "test",
        "main.test.js",
        wrapped_compound_assignment_test_source(),
        false,
        false,
    );
}

#[test]
fn test_supports_wrapped_compound_assignment_in_ts_input() {
    assert_wrapped_compound_assignment(
        "test",
        "main.test.ts",
        wrapped_compound_assignment_test_source(),
        false,
        false,
    );
}

#[test]
fn json_test_supports_wrapped_compound_assignment_in_browser_harness_js_input() {
    assert_wrapped_compound_assignment(
        "test",
        "main.test.js",
        wrapped_compound_assignment_test_source(),
        true,
        true,
    );
}

#[test]
fn json_test_supports_wrapped_compound_assignment_in_browser_harness_ts_input() {
    assert_wrapped_compound_assignment(
        "test",
        "main.test.ts",
        wrapped_compound_assignment_test_source(),
        true,
        true,
    );
}
