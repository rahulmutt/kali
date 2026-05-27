use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn find_family_source() -> &'static str {
    r#"function main() {
  console.log([0, 1, 2].find((value) => value > 1));
  console.log([0, 1, 2].findIndex((value) => value > 1));
  console.log([0, 1, 2, 3].findLast((value) => value > 1));
  console.log([0, 1, 2, 3].findLastIndex((value) => value > 1));
}
main();
"#
}

fn assert_find_family_succeeds(command: &str, extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("smoke.test.{extension}"));
    fs::write(&source_path, find_family_source()).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path()).arg(command);
    let output = cli.arg(&source_path).output().expect("run kali");

    assert!(output.status.success(), "{command} failed: {:?}", output);

    if command == "run" {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "2\n2\n3\n3\n", "unexpected stdout: {stdout}");
    } else if command == "test" {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("2\n2\n3\n3\n"),
            "unexpected stdout: {stdout}"
        );
    }
}

fn browser_find_family_source(command: &str) -> String {
    let body = r#"function browserFindFamilySlices() {
  console.log([0, 1, 2].find((value) => value > 1));
  console.log([0, 1, 2].findIndex((value) => value > 1));
  console.log([0, 1, 2, 3].findLast((value) => value > 1));
  console.log([0, 1, 2, 3].findLastIndex((value) => value > 1));
  console.log('browser find family ok');
}
browserFindFamilySlices();
"#;

    match command {
        "test" => format!("Kali.test('find family', () => {{\n{body}}});\n"),
        _ => body.to_string(),
    }
}

fn assert_browser_find_family_succeeds(command: &str, filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, browser_find_family_source(command)).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node");
    if json_output {
        cli.arg("--output").arg("json");
    }
    cli.arg(command);
    if command == "build" {
        cli.arg("--bundle");
    }
    cli.arg("--api").arg("browser");
    if matches!(command, "run" | "test") {
        cli.arg("--max-threads")
            .arg("0")
            .arg("--max-spawned-processes")
            .arg("0");
    }
    let output = cli.arg(&source_path).output().expect("run kali");

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
        if command == "run" {
            assert_eq!(json["exitCode"], 0);
            assert_eq!(json["stdout"], "2\n2\n3\n3\nbrowser find family ok\n");
        } else {
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
            assert_eq!(json["stdout"], "2\n2\n3\n3\nbrowser find family ok\n");
        }
    } else if matches!(command, "run" | "test") {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.starts_with("2\n2\n3\n3\n"), "stdout: {stdout}");
        if command == "test" {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn check_supports_find_family_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_find_family_succeeds("check", extension);
    }
}

#[test]
fn build_supports_find_family_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_find_family_succeeds("build", extension);
    }
}

#[test]
fn run_supports_find_family_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_find_family_succeeds("run", extension);
    }
}

#[test]
fn test_supports_find_family_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_find_family_succeeds("test", extension);
    }
}

#[test]
fn check_supports_find_family_in_browser_api_surface_js_ts_jsx_and_tsx_input() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_browser_find_family_succeeds("check", &format!("main.{extension}"), false);
    }
}

#[test]
fn build_supports_find_family_in_browser_api_surface_js_ts_jsx_and_tsx_input() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_browser_find_family_succeeds("build", &format!("main.{extension}"), false);
    }
}

#[test]
fn run_supports_find_family_in_browser_api_surface_with_harness_js_input() {
    assert_browser_find_family_succeeds("run", "main.js", false);
}

#[test]
fn test_supports_find_family_in_browser_api_surface_with_harness_js_input() {
    assert_browser_find_family_succeeds("test", "smoke.test.js", false);
}

#[test]
fn json_run_supports_find_family_in_browser_api_surface_with_harness_js_input() {
    assert_browser_find_family_succeeds("run", "main.js", true);
}

#[test]
fn json_test_supports_find_family_in_browser_api_surface_with_harness_js_input() {
    assert_browser_find_family_succeeds("test", "smoke.test.js", true);
}

#[test]
fn run_supports_find_family_in_browser_api_surface_with_harness_ts_input() {
    assert_browser_find_family_succeeds("run", "main.ts", false);
}

#[test]
fn test_supports_find_family_in_browser_api_surface_with_harness_ts_input() {
    assert_browser_find_family_succeeds("test", "smoke.test.ts", false);
}

#[test]
fn json_run_supports_find_family_in_browser_api_surface_with_harness_ts_input() {
    assert_browser_find_family_succeeds("run", "main.ts", true);
}

#[test]
fn json_test_supports_find_family_in_browser_api_surface_with_harness_ts_input() {
    assert_browser_find_family_succeeds("test", "smoke.test.ts", true);
}

#[test]
fn run_supports_find_family_in_browser_api_surface_with_harness_jsx_input() {
    assert_browser_find_family_succeeds("run", "main.jsx", false);
}

#[test]
fn test_supports_find_family_in_browser_api_surface_with_harness_jsx_input() {
    assert_browser_find_family_succeeds("test", "smoke.test.jsx", false);
}

#[test]
fn json_run_supports_find_family_in_browser_api_surface_with_harness_jsx_input() {
    assert_browser_find_family_succeeds("run", "main.jsx", true);
}

#[test]
fn json_test_supports_find_family_in_browser_api_surface_with_harness_jsx_input() {
    assert_browser_find_family_succeeds("test", "smoke.test.jsx", true);
}

#[test]
fn run_supports_find_family_in_browser_api_surface_with_harness_tsx_input() {
    assert_browser_find_family_succeeds("run", "main.tsx", false);
}

#[test]
fn test_supports_find_family_in_browser_api_surface_with_harness_tsx_input() {
    assert_browser_find_family_succeeds("test", "smoke.test.tsx", false);
}

#[test]
fn json_run_supports_find_family_in_browser_api_surface_with_harness_tsx_input() {
    assert_browser_find_family_succeeds("run", "main.tsx", true);
}

#[test]
fn json_test_supports_find_family_in_browser_api_surface_with_harness_tsx_input() {
    assert_browser_find_family_succeeds("test", "smoke.test.tsx", true);
}
