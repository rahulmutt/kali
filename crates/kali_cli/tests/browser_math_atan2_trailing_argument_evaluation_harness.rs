use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("json stdout")
}

fn atan2_trailing_argument_source(command: &str) -> &'static str {
    match command {
        "test" => {
            r#"Kali.test('atan2 trailing argument evaluation', () => {
  const bump = () => { console.log("bump"); return 2; };
  console.log(Math.atan2(0, 1, bump()));
});
"#
        }
        _ => {
            r#"const bump = () => { console.log("bump"); return 2; };
console.log(Math.atan2(0, 1, bump()));
"#
        }
    }
}

fn assert_browser_harness_atan2_trailing_argument_evaluation(command: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, atan2_trailing_argument_source(command)).expect("write source");

    for output_json in [false, true] {
        let mut cli = Command::new(kali_bin());
        cli.env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
            .current_dir(dir.path());
        if output_json {
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

        if output_json {
            let json = parse_json_stdout(&output);
            assert_eq!(json["schemaVersion"], 1);
            assert_eq!(json["command"], command);
            assert_eq!(json["success"], true);
            assert_eq!(json["payload"]["hostContract"], "browser-requested");
            assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
            if command == "run" {
                assert_eq!(json["payload"]["exitCode"], 0);
            } else {
                assert_eq!(json["payload"]["total"], 1);
                assert_eq!(json["payload"]["passed"], 1);
                assert_eq!(json["payload"]["failed"], 0);
            }
            let stdout = json["stdout"].as_str().expect("stdout");
            assert!(stdout.contains("bump"), "json: {json}");
            assert!(stdout.contains('0'), "json: {json}");
            assert_eq!(json["stderr"], "");
            assert!(json["errors"].as_array().expect("errors array").is_empty());
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("bump"), "stdout: {stdout}");
            assert!(stdout.contains('0'), "stdout: {stdout}");
        }
    }
}

#[test]
fn run_supports_math_atan2_trailing_argument_evaluation_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_harness_atan2_trailing_argument_evaluation("run", "main.ts");
}

#[test]
fn run_supports_math_atan2_trailing_argument_evaluation_when_browser_harness_is_configured_in_jsx_input(
) {
    assert_browser_harness_atan2_trailing_argument_evaluation("run", "main.jsx");
}

#[test]
fn run_supports_math_atan2_trailing_argument_evaluation_when_browser_harness_is_configured_in_tsx_input(
) {
    assert_browser_harness_atan2_trailing_argument_evaluation("run", "main.tsx");
}

#[test]
fn test_supports_math_atan2_trailing_argument_evaluation_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_harness_atan2_trailing_argument_evaluation("test", "smoke.test.ts");
}

#[test]
fn test_supports_math_atan2_trailing_argument_evaluation_when_browser_harness_is_configured_in_jsx_input(
) {
    assert_browser_harness_atan2_trailing_argument_evaluation("test", "smoke.test.jsx");
}

#[test]
fn test_supports_math_atan2_trailing_argument_evaluation_when_browser_harness_is_configured_in_tsx_input(
) {
    assert_browser_harness_atan2_trailing_argument_evaluation("test", "smoke.test.tsx");
}
