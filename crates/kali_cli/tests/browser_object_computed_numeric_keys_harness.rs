use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn computed_numeric_keys_source(command: &str) -> String {
    let body = r#"const obj = { [-1]: 'neg', [+2]: 'pos', [(-0)]: 'zero' };
console.log(obj[-1]);
console.log(obj[2]);
console.log(obj[0]);
"#;

    match command {
        "test" => format!("Kali.test('computed numeric object keys', () => {{\n{body}}});\n"),
        _ => body.to_string(),
    }
}

fn computed_numeric_keys_with_await_wrappers_source(command: &str) -> String {
    let body = r#"async function computedNumericObjectKeysWithAwaitWrappers() {
  const obj = {
    [await 1]: 'neg',
    [+(await 2)]: 'pos',
    [(0, await 0)]: 'zero',
  };
  console.log(obj[1]);
  console.log(obj[2]);
  console.log(obj[0]);
}
computedNumericObjectKeysWithAwaitWrappers();
"#;

    match command {
        "test" => format!(
            "Kali.test('computed numeric object keys with await wrappers', () => {{\n{body}  return computedNumericObjectKeysWithAwaitWrappers();\n}});\n"
        ),
        _ => body.to_string(),
    }
}

fn assert_browser_harness_computed_numeric_keys(
    command: &str,
    filename: &str,
    json_output: bool,
    source: String,
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

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
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
        let stdout = json["stdout"].as_str().expect("stdout string");
        assert!(stdout.contains("neg"), "json: {json}");
        assert!(stdout.contains("pos"), "json: {json}");
        assert!(stdout.contains("zero"), "json: {json}");
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("neg"), "stdout: {stdout}");
        assert!(stdout.contains("pos"), "stdout: {stdout}");
        assert!(stdout.contains("zero"), "stdout: {stdout}");
    }
}

#[test]
fn run_supports_computed_numeric_object_keys_when_browser_harness_is_configured_in_js_ts_jsx_and_tsx_input(
) {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_browser_harness_computed_numeric_keys(
            "run",
            filename,
            false,
            computed_numeric_keys_source("run"),
        );
    }
}

#[test]
fn json_run_supports_computed_numeric_object_keys_when_browser_harness_is_configured_in_js_ts_jsx_and_tsx_input(
) {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_browser_harness_computed_numeric_keys(
            "run",
            filename,
            true,
            computed_numeric_keys_source("run"),
        );
    }
}

#[test]
fn test_supports_computed_numeric_object_keys_when_browser_harness_is_configured_in_js_ts_jsx_and_tsx_input(
) {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_browser_harness_computed_numeric_keys(
            "test",
            filename,
            false,
            computed_numeric_keys_source("test"),
        );
    }
}

#[test]
fn json_test_supports_computed_numeric_object_keys_when_browser_harness_is_configured_in_js_ts_jsx_and_tsx_input(
) {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_browser_harness_computed_numeric_keys(
            "test",
            filename,
            true,
            computed_numeric_keys_source("test"),
        );
    }
}

#[test]
fn run_supports_await_wrapped_computed_numeric_object_keys_when_browser_harness_is_configured_in_js_ts_jsx_and_tsx_input(
) {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_browser_harness_computed_numeric_keys(
            "run",
            filename,
            false,
            computed_numeric_keys_with_await_wrappers_source("run"),
        );
    }
}

#[test]
fn json_run_supports_await_wrapped_computed_numeric_object_keys_when_browser_harness_is_configured_in_js_ts_jsx_and_tsx_input(
) {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_browser_harness_computed_numeric_keys(
            "run",
            filename,
            true,
            computed_numeric_keys_with_await_wrappers_source("run"),
        );
    }
}

#[test]
fn test_supports_await_wrapped_computed_numeric_object_keys_when_browser_harness_is_configured_in_js_ts_jsx_and_tsx_input(
) {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_browser_harness_computed_numeric_keys(
            "test",
            filename,
            false,
            computed_numeric_keys_with_await_wrappers_source("test"),
        );
    }
}

#[test]
fn json_test_supports_await_wrapped_computed_numeric_object_keys_when_browser_harness_is_configured_in_js_ts_jsx_and_tsx_input(
) {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_browser_harness_computed_numeric_keys(
            "test",
            filename,
            true,
            computed_numeric_keys_with_await_wrappers_source("test"),
        );
    }
}
