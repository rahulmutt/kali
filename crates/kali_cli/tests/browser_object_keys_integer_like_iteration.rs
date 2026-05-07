use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn integer_like_object_keys_iteration_source() -> &'static str {
    r##"// kali-tree-shake: browserIntegerLikeObjectKeysIteration
function browserIntegerLikeObjectKeysIteration() {
  const keys = [];
  const values = [];
  for (const key of Object.keys({ 10: 10, 2: 2, 1: 1, 0: 0, b: 5, a: 6 })) {
    keys.push(key);
  }
  for (const value of Object.values({ 10: 10, 2: 2, 1: 1, 0: 0, b: 5, a: 6 })) {
    values.push(value);
  }
  if (
    keys.length !== 6 ||
    keys[0] !== '0' ||
    keys[1] !== '1' ||
    keys[2] !== '2' ||
    keys[3] !== '10' ||
    keys[4] !== 'b' ||
    keys[5] !== 'a' ||
    values.length !== 6 ||
    values[0] !== 0 ||
    values[1] !== 1 ||
    values[2] !== 2 ||
    values[3] !== 10 ||
    values[4] !== 5 ||
    values[5] !== 6
  ) {
    throw new Error('unexpected integer-like object enumeration ordering');
  }
}
"##
}

fn assert_integer_like_object_keys_iteration(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, integer_like_object_keys_iteration_source()).expect("write source");

    let mut command = Command::new(kali_bin());
    command
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser");
    if json_output {
        command.arg("--output").arg("json");
    }
    let output = command.arg(&source_path).output().expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if json_output {
        let envelope: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(envelope["schemaVersion"], 1);
        assert_eq!(envelope["command"], "build");
        assert_eq!(envelope["success"], true);
        assert_eq!(envelope["exitCode"], 0);
        assert!(
            envelope["errors"]
                .as_array()
                .expect("errors array")
                .is_empty(),
            "json: {envelope}"
        );
    }

    let bundle_dir = dir.path().join("app");
    let harness_path = bundle_dir
        .parent()
        .expect("bundle root parent")
        .join("browser-bundle-smoke.mjs");
    let harness = kali_runtime::browser_bundle_harness_script(
        "app",
        false,
        r#"const mod = await import(bundleJs.href);
await mod.browserIntegerLikeObjectKeysIteration();
"#,
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = kali_runtime::browser_harness_command_parts_for(
        std::env::var("KALI_BROWSER_BUNDLE_HARNESS_COMMAND")
            .ok()
            .as_deref(),
    );
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(&bundle_dir)
        .args(&harness_command)
        .arg(&harness_path)
        .output()
        .expect("run browser bundle harness");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).is_empty(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

fn integer_like_object_keys_iteration_run_source() -> &'static str {
    r##"function browserIntegerLikeObjectKeysIteration() {
  const keys = [];
  const values = [];
  for (const key of Object.keys({ 10: 10, 2: 2, 1: 1, 0: 0, b: 5, a: 6 })) {
    keys.push(key);
  }
  for (const value of Object.values({ 10: 10, 2: 2, 1: 1, 0: 0, b: 5, a: 6 })) {
    values.push(value);
  }
  if (
    keys.length !== 6 ||
    keys[0] !== '0' ||
    keys[1] !== '1' ||
    keys[2] !== '2' ||
    keys[3] !== '10' ||
    keys[4] !== 'b' ||
    keys[5] !== 'a' ||
    values.length !== 6 ||
    values[0] !== 0 ||
    values[1] !== 1 ||
    values[2] !== 2 ||
    values[3] !== 10 ||
    values[4] !== 5 ||
    values[5] !== 6
  ) {
    throw new Error('unexpected integer-like object enumeration ordering');
  }
  console.log('integer-like object enumeration ok');
}

browserIntegerLikeObjectKeysIteration();
"##
}

fn integer_like_object_keys_iteration_test_source() -> &'static str {
    r##"Kali.test('integer-like object enumeration', () => {
  const keys = [];
  const values = [];
  for (const key of Object.keys({ 10: 10, 2: 2, 1: 1, 0: 0, b: 5, a: 6 })) {
    keys.push(key);
  }
  for (const value of Object.values({ 10: 10, 2: 2, 1: 1, 0: 0, b: 5, a: 6 })) {
    values.push(value);
  }
  if (
    keys.length !== 6 ||
    keys[0] !== '0' ||
    keys[1] !== '1' ||
    keys[2] !== '2' ||
    keys[3] !== '10' ||
    keys[4] !== 'b' ||
    keys[5] !== 'a' ||
    values.length !== 6 ||
    values[0] !== 0 ||
    values[1] !== 1 ||
    values[2] !== 2 ||
    values[3] !== 10 ||
    values[4] !== 5 ||
    values[5] !== 6
  ) {
    throw new Error('unexpected integer-like object enumeration ordering');
  }
  console.log('integer-like object enumeration ok');
});
"##
}

fn assert_browser_harness_integer_like_object_keys_iteration(
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
            stdout.contains("integer-like object enumeration ok"),
            "json: {json}"
        );
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("integer-like object enumeration ok"),
            "stdout: {stdout}"
        );
        if command == "test" {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn build_emits_integer_like_object_keys_iteration_semantics_in_js_input() {
    assert_integer_like_object_keys_iteration("app.js", false);
}

#[test]
fn build_emits_integer_like_object_keys_iteration_semantics_in_ts_input() {
    assert_integer_like_object_keys_iteration("app.ts", false);
}

#[test]
fn json_build_emits_integer_like_object_keys_iteration_semantics_in_js_input() {
    assert_integer_like_object_keys_iteration("app.js", true);
}

#[test]
fn json_build_emits_integer_like_object_keys_iteration_semantics_in_ts_input() {
    assert_integer_like_object_keys_iteration("app.ts", true);
}

#[test]
fn run_supports_integer_like_object_keys_iteration_when_browser_harness_is_configured_in_js_input()
{
    assert_browser_harness_integer_like_object_keys_iteration(
        "run",
        "main.js",
        integer_like_object_keys_iteration_run_source(),
        false,
    );
}

#[test]
fn run_supports_integer_like_object_keys_iteration_when_browser_harness_is_configured_in_ts_jsx_tsx_input(
) {
    let source = integer_like_object_keys_iteration_run_source();
    for filename in ["main.ts", "main.jsx", "main.tsx"] {
        assert_browser_harness_integer_like_object_keys_iteration("run", filename, source, false);
        assert_browser_harness_integer_like_object_keys_iteration("run", filename, source, true);
    }
}

#[test]
fn test_supports_integer_like_object_keys_iteration_when_browser_harness_is_configured_in_js_input()
{
    assert_browser_harness_integer_like_object_keys_iteration(
        "test",
        "smoke.test.js",
        integer_like_object_keys_iteration_test_source(),
        false,
    );
}

#[test]
fn test_supports_integer_like_object_keys_iteration_when_browser_harness_is_configured_in_ts_jsx_tsx_input(
) {
    let source = integer_like_object_keys_iteration_test_source();
    for filename in ["smoke.test.ts", "smoke.test.jsx", "smoke.test.tsx"] {
        assert_browser_harness_integer_like_object_keys_iteration("test", filename, source, false);
        assert_browser_harness_integer_like_object_keys_iteration("test", filename, source, true);
    }
}

#[test]
fn json_run_supports_integer_like_object_keys_iteration_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_harness_integer_like_object_keys_iteration(
        "run",
        "main.js",
        integer_like_object_keys_iteration_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_integer_like_object_keys_iteration_when_browser_harness_is_configured_in_ts_jsx_tsx_input(
) {
    let source = integer_like_object_keys_iteration_run_source();
    for filename in ["main.ts", "main.jsx", "main.tsx"] {
        assert_browser_harness_integer_like_object_keys_iteration("run", filename, source, true);
    }
}

#[test]
fn json_test_supports_integer_like_object_keys_iteration_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_harness_integer_like_object_keys_iteration(
        "test",
        "smoke.test.js",
        integer_like_object_keys_iteration_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_integer_like_object_keys_iteration_when_browser_harness_is_configured_in_ts_jsx_tsx_input(
) {
    let source = integer_like_object_keys_iteration_test_source();
    for filename in ["smoke.test.ts", "smoke.test.jsx", "smoke.test.tsx"] {
        assert_browser_harness_integer_like_object_keys_iteration("test", filename, source, true);
    }
}
