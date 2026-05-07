use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_run_source() -> &'static str {
    "console.log('browser max threads ok');\n"
}

fn browser_harness_test_source() -> &'static str {
    r#"Kali.test('browser max threads', () => {
  console.log('browser max threads ok');
});
"#
}

fn assert_browser_harness_accepts_max_threads(command: &str, filename: &str, source: &str) {
    for json_output in [false, true] {
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
            .arg("--wasm-threads")
            .arg("--max-threads")
            .arg("1")
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
            } else {
                assert_eq!(json["payload"]["total"], 1);
                assert_eq!(json["payload"]["passed"], 1);
                assert_eq!(json["payload"]["failed"], 0);
            }
            assert!(
                json["stdout"]
                    .as_str()
                    .expect("stdout string")
                    .contains("browser max threads ok"),
                "json: {json}"
            );
            assert_eq!(json["stderr"], "");
            assert!(json["errors"].as_array().expect("errors array").is_empty());
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                stdout.contains("browser max threads ok"),
                "stdout: {stdout}"
            );
            if command == "test" {
                assert!(stdout.contains("ok 1"), "stdout: {stdout}");
            }
        }
    }
}

#[test]
fn run_supports_positive_max_threads_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_accepts_max_threads("run", "main.js", browser_harness_run_source());
}

#[test]
fn run_supports_positive_max_threads_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_accepts_max_threads("run", "main.ts", browser_harness_run_source());
}

#[test]
fn run_supports_positive_max_threads_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_accepts_max_threads("run", "main.jsx", browser_harness_run_source());
}

#[test]
fn run_supports_positive_max_threads_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_accepts_max_threads("run", "main.tsx", browser_harness_run_source());
}

#[test]
fn test_supports_positive_max_threads_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_accepts_max_threads(
        "test",
        "smoke.test.js",
        browser_harness_test_source(),
    );
}

#[test]
fn test_supports_positive_max_threads_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_accepts_max_threads(
        "test",
        "smoke.test.ts",
        browser_harness_test_source(),
    );
}

#[test]
fn test_supports_positive_max_threads_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_accepts_max_threads(
        "test",
        "smoke.test.jsx",
        browser_harness_test_source(),
    );
}

#[test]
fn test_supports_positive_max_threads_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_accepts_max_threads(
        "test",
        "smoke.test.tsx",
        browser_harness_test_source(),
    );
}
