use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn assert_browser_harness_unsupported_math_rejection(
    command: &str,
    filename: &str,
    source: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node");
    if json_output {
        cli.arg("--output").arg("json");
    }
    cli.arg(command)
        .arg("--api")
        .arg("browser")
        .arg(&source_path);

    let output = cli.output().expect("run kali");
    assert!(
        !output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(1));

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(!errors.is_empty(), "errors array should not be empty");
        assert!(
            errors.iter().all(|error| error["code"] == "E5506"),
            "unexpected errors: {errors:?}"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(
            stderr.contains("Math.sqrt")
                || stderr.contains("Math.atan2")
                || stderr.contains("unsupported math"),
            "stderr: {stderr}"
        );
    }
}

fn browser_harness_run_source() -> &'static str {
    "console.log(Math.sqrt(1.6));\nconsole.log(globalThis.Math[\"sqrt\"](1.6));\nconsole.log(globalThis[\"Math\"][\"sqrt\"](1.6));\n"
}

fn browser_harness_test_source() -> &'static str {
    r#"Kali.test('unsupported math member', () => {
  console.log(Math.sqrt(1.6));
  console.log(globalThis.Math["sqrt"](1.6));
  console.log(globalThis["Math"]["sqrt"](1.6));
});
"#
}

fn browser_harness_run_atan2_source() -> &'static str {
    "console.log(Math.atan2(1, 1));\nconsole.log(globalThis.Math[\"atan2\"](1, 1));\nconsole.log(globalThis[\"Math\"][\"atan2\"](1, 1));\n"
}

fn browser_harness_test_atan2_source() -> &'static str {
    r#"Kali.test('unsupported math member', () => {
  console.log(Math.atan2(1, 1));
  console.log(globalThis.Math["atan2"](1, 1));
  console.log(globalThis["Math"]["atan2"](1, 1));
});
"#
}

#[test]
fn run_rejects_unsupported_math_member_calls_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_browser_harness_unsupported_math_rejection(
            "run",
            &format!("main.{extension}"),
            browser_harness_run_source(),
            false,
        );
        assert_browser_harness_unsupported_math_rejection(
            "run",
            &format!("main.{extension}"),
            browser_harness_run_source(),
            true,
        );
    }
}

#[test]
fn run_rejects_broader_math_atan2_member_calls_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_browser_harness_unsupported_math_rejection(
            "run",
            &format!("main.{extension}"),
            browser_harness_run_atan2_source(),
            false,
        );
        assert_browser_harness_unsupported_math_rejection(
            "run",
            &format!("main.{extension}"),
            browser_harness_run_atan2_source(),
            true,
        );
    }
}

#[test]
fn test_rejects_unsupported_math_member_calls_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_browser_harness_unsupported_math_rejection(
            "test",
            &format!("smoke.test.{extension}"),
            browser_harness_test_source(),
            false,
        );
        assert_browser_harness_unsupported_math_rejection(
            "test",
            &format!("smoke.test.{extension}"),
            browser_harness_test_source(),
            true,
        );
    }
}

#[test]
fn test_rejects_broader_math_atan2_member_calls_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_browser_harness_unsupported_math_rejection(
            "test",
            &format!("smoke.test.{extension}"),
            browser_harness_test_atan2_source(),
            false,
        );
        assert_browser_harness_unsupported_math_rejection(
            "test",
            &format!("smoke.test.{extension}"),
            browser_harness_test_atan2_source(),
            true,
        );
    }
}
