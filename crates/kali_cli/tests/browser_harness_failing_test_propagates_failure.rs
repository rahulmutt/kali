use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

/// A registered test whose self-check fails via `throw`. Under the print-then-
/// trap `throw` lowering, the test's body executes inline during `_start` and
/// the trap escapes the JS harness's per-callback try/catch, killing the process
/// before the summary is written. The Rust crash-lane counts this as a failed
/// test (no compile/trap diagnostic) — the exact trap-swallow class.
fn failing_browser_test_source() -> &'static str {
    r#"Kali.test('self-check throw propagates as a failure', () => {
  const actual = 1;
  if (actual !== 2) {
    throw 'expected 2';
  }
});
"#
}

#[test]
fn browser_harness_failing_registered_test_reports_failure_and_nonzero_exit() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("failing.test.js");
    fs::write(&source_path, failing_browser_test_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "process should exit non-zero for a failing registered test\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], false, "json: {json}");
    assert_eq!(json["exitCode"], 1, "json: {json}");
    assert_eq!(json["payload"]["total"], 1, "json: {json}");
    assert_eq!(json["payload"]["passed"], 0, "json: {json}");
    assert_eq!(json["payload"]["failed"], 1, "json: {json}");
    // The failure is carried purely by the failed-test count, not a diagnostic:
    // this is what distinguishes the trap-swallow class from a compile error.
    assert!(
        json["errors"].as_array().expect("errors array").is_empty(),
        "expected no compile/trap diagnostics, only a failed test count; json: {json}"
    );
}
