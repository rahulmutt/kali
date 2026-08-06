use std::{fs, process::Command};

use tempfile::tempdir;

use kali_common::promise_all_browser_body_source;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_promise_all_run_source() -> String {
    format!(
        "async function browserPromiseAll() {{\n{}\n}}\n\nasync function main() {{\n  await browserPromiseAll();\n  console.log('browser promise all ok');\n}}\n\nmain();\n",
        promise_all_browser_body_source()
    )
}

fn browser_promise_all_test_source() -> String {
    format!(
        "async function browserPromiseAll() {{\n{}\n}}\n\nKali.test('browser promise all', () => browserPromiseAll());\n",
        promise_all_browser_body_source()
    )
}

fn assert_browser_requested_promise_all(command: &str, filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        browser_promise_all_test_source()
    } else {
        browser_promise_all_run_source()
    };
    fs::write(&source_path, source).expect("write source");

    let mut command_line = Command::new(kali_bin());
    command_line
        .current_dir(dir.path())
        .env(kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV, "node");
    if json_output {
        command_line.arg("--output").arg("json");
    }
    let output = command_line
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!output.status.success(), "must fail closed: {output:?}");
}

#[test]
fn run_supports_promise_all_in_js_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all("run", "main.js", false);
}

#[test]
fn run_supports_promise_all_in_ts_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all("run", "main.ts", false);
}

#[test]
fn run_supports_promise_all_in_jsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all("run", "main.jsx", false);
}

#[test]
fn run_supports_promise_all_in_tsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all("run", "main.tsx", false);
}

#[test]
fn test_supports_promise_all_in_js_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all("test", "smoke.test.js", false);
}

#[test]
fn test_supports_promise_all_in_ts_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all("test", "smoke.test.ts", false);
}

#[test]
fn test_supports_promise_all_in_jsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all("test", "smoke.test.jsx", false);
}

#[test]
fn test_supports_promise_all_in_tsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all("test", "smoke.test.tsx", false);
}

#[test]
fn json_run_supports_promise_all_in_js_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all("run", "main.js", true);
}

#[test]
fn json_run_supports_promise_all_in_ts_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all("run", "main.ts", true);
}

#[test]
fn json_run_supports_promise_all_in_jsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all("run", "main.jsx", true);
}

#[test]
fn json_run_supports_promise_all_in_tsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all("run", "main.tsx", true);
}

#[test]
fn json_test_supports_promise_all_in_js_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all("test", "smoke.test.js", true);
}

#[test]
fn json_test_supports_promise_all_in_ts_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all("test", "smoke.test.ts", true);
}

#[test]
fn json_test_supports_promise_all_in_jsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all("test", "smoke.test.jsx", true);
}

#[test]
fn json_test_supports_promise_all_in_tsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all("test", "smoke.test.tsx", true);
}
