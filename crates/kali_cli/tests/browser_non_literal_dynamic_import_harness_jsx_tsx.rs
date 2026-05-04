use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn non_literal_dynamic_import_source() -> &'static str {
    "let specifier; import(specifier);"
}

fn non_literal_dynamic_import_test_source() -> &'static str {
    "Kali.test('dynamic import', () => { let specifier; return import(specifier); });\n"
}

fn assert_non_literal_dynamic_import_rejection_text(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("non-literal dynamic import()")
            || stderr.contains("statically known import specifier"),
        "stderr: {stderr}"
    );
}

fn assert_non_literal_dynamic_import_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().all(|error| error["code"] == "E5506"),
        "unexpected errors: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error["message"]
            .as_str()
            .expect("error message")
            .contains("non-literal dynamic import()")
            || error["message"]
                .as_str()
                .expect("error message")
                .contains("statically known import specifier")),
        "missing non-literal dynamic import in {errors:?}"
    );
}

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
}

fn assert_browser_harness_rejects_non_literal_dynamic_import(
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
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node");
    if json_output {
        cli.arg("--output").arg("json");
    }
    let output = cli
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));

    if json_output {
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        let errors = json["errors"].as_array().expect("errors array");
        assert_non_literal_dynamic_import_rejection_json(errors);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_non_literal_dynamic_import_rejection_text(&stderr);
    }
}

#[test]
fn run_rejects_non_literal_dynamic_import_targets_in_browser_api_surface_with_harness_jsx_and_tsx_input(
) {
    for extension in ["jsx", "tsx"] {
        assert_browser_harness_rejects_non_literal_dynamic_import(
            "run",
            &format!("main.{extension}"),
            non_literal_dynamic_import_source(),
            false,
        );
        assert_browser_harness_rejects_non_literal_dynamic_import(
            "run",
            &format!("main.{extension}"),
            non_literal_dynamic_import_source(),
            true,
        );
    }
}

#[test]
fn test_rejects_non_literal_dynamic_import_targets_in_browser_api_surface_with_harness_jsx_and_tsx_input(
) {
    for extension in ["jsx", "tsx"] {
        assert_browser_harness_rejects_non_literal_dynamic_import(
            "test",
            &format!("smoke.test.{extension}"),
            non_literal_dynamic_import_test_source(),
            false,
        );
        assert_browser_harness_rejects_non_literal_dynamic_import(
            "test",
            &format!("smoke.test.{extension}"),
            non_literal_dynamic_import_test_source(),
            true,
        );
    }
}
