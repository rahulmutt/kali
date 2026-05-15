use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_number_predicates_run_source() -> &'static str {
    r#"const alias = 1;
console.log(Number.isFinite(alias));
console.log(Number.isInteger(alias));
console.log(Number.isSafeInteger(alias));
console.log(Number.isInteger(1.5));
console.log(Number.isFinite("hello"));
console.log(Number.isSafeInteger(1.5));
console.log(globalThis["Number"]["isNaN"](NaN));
console.log(globalThis.Number.isNaN(1));
console.log(globalThis["Number"].isNaN(1));
console.log(globalThis["Number"]["isFinite"](alias));
console.log(globalThis["Number"]["isInteger"](alias));
console.log(globalThis["Number"]["isSafeInteger"](alias));
console.log(globalThis.Number["isNaN"](1));
console.log(globalThis["Number"].isFinite(alias));
console.log(globalThis.Number["isInteger"](alias));
console.log(globalThis["Number"].isSafeInteger(alias));
console.log(Number["isFinite"](alias));
console.log(Number["isInteger"](alias));
console.log(Number["isSafeInteger"](alias));
console.log(Number["isNaN"](1));
"#
}

fn browser_number_predicates_test_source() -> &'static str {
    r#"Kali.test('number predicates', () => {
  const alias = 1;
  console.log(Number.isFinite(alias));
  console.log(Number.isInteger(alias));
  console.log(Number.isSafeInteger(alias));
  console.log(Number.isInteger(1.5));
  console.log(Number.isFinite("hello"));
  console.log(Number.isSafeInteger(1.5));
  console.log(globalThis["Number"]["isNaN"](NaN));
  console.log(globalThis.Number.isNaN(1));
  console.log(globalThis["Number"].isNaN(1));
  console.log(globalThis["Number"]["isFinite"](alias));
  console.log(globalThis["Number"]["isInteger"](alias));
  console.log(globalThis["Number"]["isSafeInteger"](alias));
  console.log(globalThis.Number["isNaN"](1));
  console.log(globalThis["Number"].isFinite(alias));
  console.log(globalThis.Number["isInteger"](alias));
  console.log(globalThis["Number"].isSafeInteger(alias));
  console.log(Number["isFinite"](alias));
  console.log(Number["isInteger"](alias));
  console.log(Number["isSafeInteger"](alias));
  console.log(Number["isNaN"](1));
});
"#
}

fn assert_browser_harness_number_predicates(
    command: &str,
    filename: &str,
    source: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut output = Command::new(kali_bin());
    output
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node");
    if json_output {
        output.arg("--output").arg("json");
    }
    let output = output
        .arg(command)
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
            assert_eq!(
                json["stdout"],
                "1\n1\n1\n0\n0\n0\n1\n0\n0\n1\n1\n1\n0\n1\n1\n1\n1\n1\n1\n0\n"
            );
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
            assert_eq!(json["payload"]["skipped"], 0);
            assert!(json["stdout"]
                .as_str()
                .expect("stdout string")
                .contains("1\n1\n1\n0\n0\n0\n1\n0\n0\n1\n1\n1\n0\n1\n1\n1\n1\n1\n1\n0\n"));
        }
        assert_eq!(json["stderr"], "");
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("1\n1\n1\n0\n0\n0\n1\n0\n0\n1\n1\n1\n0\n1\n1\n1\n1\n1\n1\n0\n"),
            "stdout: {stdout}"
        );
        if command == "test" {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn run_supports_number_predicates_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_number_predicates(
        "run",
        "main.js",
        browser_number_predicates_run_source(),
        false,
    );
}

#[test]
fn run_supports_number_predicates_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_number_predicates(
        "run",
        "main.ts",
        browser_number_predicates_run_source(),
        false,
    );
}

#[test]
fn run_supports_number_predicates_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_number_predicates(
        "run",
        "main.jsx",
        browser_number_predicates_run_source(),
        false,
    );
}

#[test]
fn run_supports_number_predicates_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_number_predicates(
        "run",
        "main.tsx",
        browser_number_predicates_run_source(),
        false,
    );
}

#[test]
fn test_supports_number_predicates_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_number_predicates(
        "test",
        "smoke.test.js",
        browser_number_predicates_test_source(),
        false,
    );
}

#[test]
fn test_supports_number_predicates_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_number_predicates(
        "test",
        "smoke.test.ts",
        browser_number_predicates_test_source(),
        false,
    );
}

#[test]
fn test_supports_number_predicates_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_number_predicates(
        "test",
        "smoke.test.jsx",
        browser_number_predicates_test_source(),
        false,
    );
}

#[test]
fn test_supports_number_predicates_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_number_predicates(
        "test",
        "smoke.test.tsx",
        browser_number_predicates_test_source(),
        false,
    );
}

#[test]
fn json_run_supports_number_predicates_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_number_predicates(
        "run",
        "main.js",
        browser_number_predicates_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_number_predicates_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_number_predicates(
        "run",
        "main.ts",
        browser_number_predicates_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_number_predicates_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_number_predicates(
        "run",
        "main.jsx",
        browser_number_predicates_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_number_predicates_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_number_predicates(
        "run",
        "main.tsx",
        browser_number_predicates_run_source(),
        true,
    );
}

#[test]
fn json_test_supports_number_predicates_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_number_predicates(
        "test",
        "smoke.test.js",
        browser_number_predicates_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_number_predicates_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_number_predicates(
        "test",
        "smoke.test.ts",
        browser_number_predicates_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_number_predicates_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_number_predicates(
        "test",
        "smoke.test.jsx",
        browser_number_predicates_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_number_predicates_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_number_predicates(
        "test",
        "smoke.test.tsx",
        browser_number_predicates_test_source(),
        true,
    );
}
