use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn for_of_spread_source() -> &'static str {
    "const values = [1, 2]; for (const item of [...(values)]) { console.log(item); }\n"
}

fn for_await_spread_source() -> &'static str {
    "const values = [1, 2]; for await (const item of [...(values)]) { console.log(item); }\n"
}

fn assert_browser_requested_spread(
    command: &str,
    filename: &str,
    source: &str,
    json_output: bool,
    expect_test_runner: bool,
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
        let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
        if !expect_test_runner {
            let stdout = json["stdout"].as_str().expect("stdout string");
            assert!(stdout.contains('1'), "json: {json}");
            assert!(stdout.contains('2'), "json: {json}");
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
        }
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains('1'), "stdout: {stdout}");
        assert!(stdout.contains('2'), "stdout: {stdout}");
    }
}

#[test]
fn supports_for_of_spread_of_const_bound_literal_arrays_in_browser_api_surface_with_harness_jsx_input(
) {
    for command in ["run", "test"] {
        let expect_test_runner = command == "test";
        for json_output in [false, true] {
            assert_browser_requested_spread(
                command,
                if expect_test_runner {
                    "smoke.test.jsx"
                } else {
                    "main.jsx"
                },
                for_of_spread_source(),
                json_output,
                expect_test_runner,
            );
        }
    }
}

#[test]
fn supports_for_of_spread_of_const_bound_literal_arrays_in_browser_api_surface_with_harness_tsx_input(
) {
    for command in ["run", "test"] {
        let expect_test_runner = command == "test";
        for json_output in [false, true] {
            assert_browser_requested_spread(
                command,
                if expect_test_runner {
                    "smoke.test.tsx"
                } else {
                    "main.tsx"
                },
                for_of_spread_source(),
                json_output,
                expect_test_runner,
            );
        }
    }
}

#[test]
fn supports_for_await_spread_of_const_bound_literal_arrays_in_browser_api_surface_with_harness_jsx_input(
) {
    for command in ["run", "test"] {
        let expect_test_runner = command == "test";
        for json_output in [false, true] {
            assert_browser_requested_spread(
                command,
                if expect_test_runner {
                    "smoke.test.jsx"
                } else {
                    "main.jsx"
                },
                for_await_spread_source(),
                json_output,
                expect_test_runner,
            );
        }
    }
}

#[test]
fn supports_for_await_spread_of_const_bound_literal_arrays_in_browser_api_surface_with_harness_tsx_input(
) {
    for command in ["run", "test"] {
        let expect_test_runner = command == "test";
        for json_output in [false, true] {
            assert_browser_requested_spread(
                command,
                if expect_test_runner {
                    "smoke.test.tsx"
                } else {
                    "main.tsx"
                },
                for_await_spread_source(),
                json_output,
                expect_test_runner,
            );
        }
    }
}
