use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn reflect_own_keys_source() -> &'static str {
    r#"const obj = { "b": 1, "2": 2, "a": 3, "1": 4 };
const keys = globalThis.Reflect.ownKeys(obj);
const bracketedKeys = globalThis["Reflect"]["ownKeys"](obj);
if (
  keys.length !== 4 ||
  keys[0] !== '1' ||
  keys[1] !== '2' ||
  keys[2] !== 'b' ||
  keys[3] !== 'a' ||
  bracketedKeys.length !== 4 ||
  bracketedKeys[0] !== '1' ||
  bracketedKeys[1] !== '2' ||
  bracketedKeys[2] !== 'b' ||
  bracketedKeys[3] !== 'a'
) {
  throw new Error('unexpected Reflect.ownKeys ordering');
}
console.log('reflect ownKeys ok');
"#
}

fn reflect_own_keys_test_source() -> &'static str {
    r#"Kali.test('reflect ownKeys', () => {
  const obj = { "b": 1, "2": 2, "a": 3, "1": 4 };
  const keys = globalThis.Reflect.ownKeys(obj);
  const bracketedKeys = globalThis["Reflect"]["ownKeys"](obj);
  if (
    keys.length !== 4 ||
    keys[0] !== '1' ||
    keys[1] !== '2' ||
    keys[2] !== 'b' ||
    keys[3] !== 'a' ||
    bracketedKeys.length !== 4 ||
    bracketedKeys[0] !== '1' ||
    bracketedKeys[1] !== '2' ||
    bracketedKeys[2] !== 'b' ||
    bracketedKeys[3] !== 'a'
  ) {
    throw new Error('unexpected Reflect.ownKeys ordering');
  }
});
"#
}

fn browser_bundle_reflect_own_keys_source() -> &'static str {
    r##"// kali-tree-shake: reflectOwnKeysSmoke
async function reflectOwnKeysSmoke(left, right) {
  const obj = { "b": 1, "2": 2, "a": 3, "1": 4 };
  const keys = Reflect.ownKeys(obj);
  const bracketedKeys = globalThis["Reflect"]["ownKeys"](obj);
  if (
    keys.length !== 4 ||
    keys[0] !== '1' ||
    keys[1] !== '2' ||
    keys[2] !== 'b' ||
    keys[3] !== 'a' ||
    bracketedKeys.length !== 4 ||
    bracketedKeys[0] !== '1' ||
    bracketedKeys[1] !== '2' ||
    bracketedKeys[2] !== 'b' ||
    bracketedKeys[3] !== 'a'
  ) {
    throw new Error('unexpected Reflect.ownKeys ordering');
  }
  return left - left + right - right;
}
"##
}

fn assert_browser_requested_reflect_own_keys(command: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        reflect_own_keys_test_source()
    } else {
        reflect_own_keys_source()
    };
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
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

    let stdout = String::from_utf8_lossy(&output.stdout);
    if command == "run" {
        assert!(stdout.contains("reflect ownKeys ok"), "stdout: {stdout}");
    } else {
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
}

fn assert_json_browser_requested_reflect_own_keys(command: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        reflect_own_keys_test_source()
    } else {
        reflect_own_keys_source()
    };
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
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

    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], true);
    assert_eq!(json["exitCode"], 0);
    assert_eq!(json["payload"]["hostContract"], "browser-requested");
    assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
    if command == "run" {
        assert_eq!(json["payload"]["exitCode"], 0);
        assert!(
            json["stdout"]
                .as_str()
                .expect("stdout")
                .contains("reflect ownKeys ok"),
            "json: {json}"
        );
    } else {
        assert_eq!(json["payload"]["total"], 1);
        assert_eq!(json["payload"]["passed"], 1);
        assert_eq!(json["payload"]["failed"], 0);
        assert_eq!(json["stdout"], "");
    }
    assert_eq!(json["stderr"], "");
}

fn assert_browser_bundle_reflect_own_keys(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, browser_bundle_reflect_own_keys_source()).expect("write source");

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
        let payload = envelope["payload"].as_object().expect("payload object");
        assert_eq!(payload["artifactKind"], "bundle");
        assert_eq!(payload["bundleFormat"], "esm");
    }

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_eq!(metadata["apiSurface"], "browser");
    assert_eq!(metadata["artifactKind"], "bundle");

    let harness_path = bundle_dir
        .parent()
        .expect("bundle root parent")
        .join("browser-bundle-smoke.mjs");
    let harness = kali_runtime::browser_bundle_harness_script(
        "app",
        false,
        r#"const mod = await import(bundleJs.href);
const result = await mod.reflectOwnKeysSmoke(1n, 2n);
if (result !== 0n) {
  throw new Error(`unexpected result ${result}`);
}
console.log(String(result));
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains('0'), "stdout: {stdout}");
}

#[test]
fn run_supports_reflect_own_keys_in_js_input_when_browser_harness_is_configured() {
    assert_browser_requested_reflect_own_keys("run", "main.js");
}

#[test]
fn test_supports_reflect_own_keys_in_js_input_when_browser_harness_is_configured() {
    assert_browser_requested_reflect_own_keys("test", "smoke.test.js");
}

#[test]
fn json_run_supports_reflect_own_keys_in_js_input_when_browser_harness_is_configured() {
    assert_json_browser_requested_reflect_own_keys("run", "main.js");
}

#[test]
fn json_test_supports_reflect_own_keys_in_js_input_when_browser_harness_is_configured() {
    assert_json_browser_requested_reflect_own_keys("test", "smoke.test.js");
}

#[test]
fn build_emits_browser_bundle_reflect_own_keys_semantics_in_js_input() {
    assert_browser_bundle_reflect_own_keys("app.js", false);
}

#[test]
fn json_build_emits_browser_bundle_reflect_own_keys_semantics_in_js_input() {
    assert_browser_bundle_reflect_own_keys("app.js", true);
}
