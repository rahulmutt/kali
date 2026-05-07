use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn object_is_browser_harness_source() -> &'static str {
    r#"const zero = 0; const zeroAlias = zero;
console.log(Object.is(zeroAlias, -0));
console.log(Object.is(-0, +0));
console.log(Object.is(+1, 1));
console.log(Object.is(true, true));
console.log(Object.is("hello", "hello"));
console.log(Object.is(1n, 1n));
console.log(Object.is(-1n, -1n));
console.log(Object.is(null, null));
console.log(Object.is(Infinity, Infinity));
console.log(Object.is(NaN, NaN));
console.log(Object.is(-Infinity, -Infinity));
console.log(globalThis["Object"]["is"](+1, 1));
console.log(globalThis.Object["is"](+1, 1));
console.log(globalThis["Object"].is(+1, 1));
console.log(globalThis.Object.is(+1, 1));
console.log('browser object is primitive literals ok');
"#
}

fn object_is_browser_harness_test_source() -> &'static str {
    r#"Kali.test('object is primitive literals', () => {
  const zero = 0;
  const zeroAlias = zero;
  console.log(Object.is(zeroAlias, -0));
  console.log(Object.is(-0, +0));
  console.log(Object.is(+1, 1));
  console.log(Object.is(true, true));
  console.log(Object.is("hello", "hello"));
  console.log(Object.is(1n, 1n));
  console.log(Object.is(-1n, -1n));
  console.log(Object.is(null, null));
  console.log(Object.is(Infinity, Infinity));
  console.log(Object.is(NaN, NaN));
  console.log(Object.is(-Infinity, -Infinity));
  console.log(globalThis["Object"]["is"](+1, 1));
  console.log(globalThis.Object["is"](+1, 1));
  console.log(globalThis["Object"].is(+1, 1));
  console.log(globalThis.Object.is(+1, 1));
  console.log('browser object is primitive literals ok');
});
"#
}

fn assert_browser_harness_object_is(
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
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path());
    if json_output {
        output.arg("--output").arg("json");
    }
    let output = output
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
            assert_eq!(json["exitCode"], 0);
            assert_eq!(json["payload"]["exitCode"], 0);
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
        }
        let stdout = json["stdout"].as_str().expect("stdout string");
        assert!(
            stdout.contains("browser object is primitive literals ok"),
            "json: {json}"
        );
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("browser object is primitive literals ok"),
            "stdout: {stdout}"
        );
        if command == "test" {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn run_supports_object_is_primitive_literals_when_browser_harness_is_configured_in_js_ts_jsx_tsx_input(
) {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_browser_harness_object_is(
            "run",
            filename,
            object_is_browser_harness_source(),
            false,
        );
    }
}

#[test]
fn json_run_supports_object_is_primitive_literals_when_browser_harness_is_configured_in_js_ts_jsx_tsx_input(
) {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_browser_harness_object_is("run", filename, object_is_browser_harness_source(), true);
    }
}

#[test]
fn test_supports_object_is_primitive_literals_when_browser_harness_is_configured_in_js_ts_jsx_tsx_input(
) {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_browser_harness_object_is(
            "test",
            filename,
            object_is_browser_harness_test_source(),
            false,
        );
    }
}

#[test]
fn json_test_supports_object_is_primitive_literals_when_browser_harness_is_configured_in_js_ts_jsx_tsx_input(
) {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_browser_harness_object_is(
            "test",
            filename,
            object_is_browser_harness_test_source(),
            true,
        );
    }
}
