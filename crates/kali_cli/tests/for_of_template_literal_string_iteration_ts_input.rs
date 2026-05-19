use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

use kali_common::template_literal_string_iteration_body_source;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn template_literal_iteration_source(command: &str) -> String {
    let body = template_literal_string_iteration_body_source();

    match command {
        "test" => format!("Kali.test('template literal iteration', () => {{\n{body}\n}});\n"),
        _ => format!("{body}\n"),
    }
}

fn assert_stdout(stdout: &str) {
    let mut lines: Vec<&str> = stdout.lines().filter(|line| !line.is_empty()).collect();
    if lines.last() == Some(&"ok 1") {
        lines.pop();
    }
    assert_eq!(lines, ["h", "e", "l", "l", "o"], "stdout: {stdout}");
}

fn assert_template_literal_iteration(command: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(if command == "test" {
        "smoke.test.ts"
    } else {
        "main.ts"
    });
    fs::write(&source_path, template_literal_iteration_source(command)).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path());
    if json_output {
        cli.arg("--output").arg("json");
    }
    let output = cli
        .arg(command)
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
        assert_eq!(json["errors"].as_array().expect("errors array").len(), 0);
        if command == "run" {
            assert_eq!(json["payload"]["exitCode"], 0);
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
        }
    } else {
        assert_stdout(&String::from_utf8_lossy(&output.stdout));
    }
}

#[test]
fn run_supports_for_of_template_literal_string_iteration_in_ts_input() {
    assert_template_literal_iteration("run", false);
}

#[test]
fn test_supports_for_of_template_literal_string_iteration_in_ts_input() {
    assert_template_literal_iteration("test", false);
}

#[test]
fn json_run_supports_for_of_template_literal_string_iteration_in_ts_input() {
    assert_template_literal_iteration("run", true);
}

#[test]
fn json_test_supports_for_of_template_literal_string_iteration_in_ts_input() {
    assert_template_literal_iteration("test", true);
}
