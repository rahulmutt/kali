use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn reflect_own_keys_source() -> &'static str {
    r#"const obj = { "b": 1, "2": 2, "a": 3, "1": 4 };
const frozenObj = Object.freeze(obj);
const keys = globalThis.Reflect.ownKeys(obj);
const frozenKeys = globalThis.Reflect.ownKeys(frozenObj);
const mixedRootKeys = globalThis["Reflect"].ownKeys(obj);
const mixedBracketedKeys = globalThis.Reflect["ownKeys"](obj);
const bracketedKeys = globalThis["Reflect"]["ownKeys"](obj);
let syncCount = 0;
for (const key of globalThis.Reflect.ownKeys(obj)) {
  syncCount += 1;
}
let frozenSyncCount = 0;
for (const key of globalThis.Reflect.ownKeys(frozenObj)) {
  frozenSyncCount += 1;
}
let sequenceCount = 0;
for (const key of (0, globalThis.Reflect.ownKeys(obj))) {
  sequenceCount += 1;
}
let frozenSequenceCount = 0;
for (const key of (0, globalThis.Reflect.ownKeys(frozenObj))) {
  frozenSequenceCount += 1;
}
let asyncCount = 0;
for await (const key of globalThis.Reflect.ownKeys(obj)) {
  asyncCount += 1;
}
let frozenAsyncCount = 0;
for await (const key of globalThis.Reflect.ownKeys(frozenObj)) {
  frozenAsyncCount += 1;
}
let asyncSequenceCount = 0;
for await (const key of (0, globalThis.Reflect.ownKeys(obj))) {
  asyncSequenceCount += 1;
}
let frozenAsyncSequenceCount = 0;
for await (const key of (0, globalThis.Reflect.ownKeys(frozenObj))) {
  frozenAsyncSequenceCount += 1;
}
if (
  keys.length !== 4 ||
  keys[0] !== '1' ||
  keys[1] !== '2' ||
  keys[2] !== 'b' ||
  keys[3] !== 'a' ||
  frozenKeys.length !== 4 ||
  frozenKeys[0] !== '1' ||
  frozenKeys[1] !== '2' ||
  frozenKeys[2] !== 'b' ||
  frozenKeys[3] !== 'a' ||
  mixedRootKeys.length !== 4 ||
  mixedRootKeys[0] !== '1' ||
  mixedRootKeys[1] !== '2' ||
  mixedRootKeys[2] !== 'b' ||
  mixedRootKeys[3] !== 'a' ||
  mixedBracketedKeys.length !== 4 ||
  mixedBracketedKeys[0] !== '1' ||
  mixedBracketedKeys[1] !== '2' ||
  mixedBracketedKeys[2] !== 'b' ||
  mixedBracketedKeys[3] !== 'a' ||
  bracketedKeys.length !== 4 ||
  bracketedKeys[0] !== '1' ||
  bracketedKeys[1] !== '2' ||
  bracketedKeys[2] !== 'b' ||
  bracketedKeys[3] !== 'a' ||
  syncCount !== 4 ||
  frozenSyncCount !== 4 ||
  sequenceCount !== 4 ||
  frozenSequenceCount !== 4 ||
  asyncCount !== 4 ||
  frozenAsyncCount !== 4 ||
  asyncSequenceCount !== 4 ||
  frozenAsyncSequenceCount !== 4
) {
  throw new Error('unexpected Reflect.ownKeys ordering');
}
console.log('reflect ownKeys ok');
"#
}

fn reflect_own_keys_test_source() -> &'static str {
    r#"Kali.test('reflect ownKeys', () => {
  const obj = { "b": 1, "2": 2, "a": 3, "1": 4 };
  const frozenObj = Object.freeze(obj);
  const keys = globalThis.Reflect.ownKeys(obj);
  const frozenKeys = globalThis.Reflect.ownKeys(frozenObj);
  const mixedRootKeys = globalThis["Reflect"].ownKeys(obj);
  const mixedBracketedKeys = globalThis.Reflect["ownKeys"](obj);
  const bracketedKeys = globalThis["Reflect"]["ownKeys"](obj);
  let syncCount = 0;
  for (const key of globalThis.Reflect.ownKeys(obj)) {
    syncCount += 1;
  }
  let frozenSyncCount = 0;
  for (const key of globalThis.Reflect.ownKeys(frozenObj)) {
    frozenSyncCount += 1;
  }
  let sequenceCount = 0;
  for (const key of (0, globalThis.Reflect.ownKeys(obj))) {
    sequenceCount += 1;
  }
  let frozenSequenceCount = 0;
  for (const key of (0, globalThis.Reflect.ownKeys(frozenObj))) {
    frozenSequenceCount += 1;
  }
  let asyncCount = 0;
  for await (const key of globalThis.Reflect.ownKeys(obj)) {
    asyncCount += 1;
  }
  let frozenAsyncCount = 0;
  for await (const key of globalThis.Reflect.ownKeys(frozenObj)) {
    frozenAsyncCount += 1;
  }
  let asyncSequenceCount = 0;
  for await (const key of (0, globalThis.Reflect.ownKeys(obj))) {
    asyncSequenceCount += 1;
  }
  let frozenAsyncSequenceCount = 0;
  for await (const key of (0, globalThis.Reflect.ownKeys(frozenObj))) {
    frozenAsyncSequenceCount += 1;
  }
  if (
    keys.length !== 4 ||
    keys[0] !== '1' ||
    keys[1] !== '2' ||
    keys[2] !== 'b' ||
    keys[3] !== 'a' ||
    frozenKeys.length !== 4 ||
    frozenKeys[0] !== '1' ||
    frozenKeys[1] !== '2' ||
    frozenKeys[2] !== 'b' ||
    frozenKeys[3] !== 'a' ||
    mixedRootKeys.length !== 4 ||
    mixedRootKeys[0] !== '1' ||
    mixedRootKeys[1] !== '2' ||
    mixedRootKeys[2] !== 'b' ||
    mixedRootKeys[3] !== 'a' ||
    mixedBracketedKeys.length !== 4 ||
    mixedBracketedKeys[0] !== '1' ||
    mixedBracketedKeys[1] !== '2' ||
    mixedBracketedKeys[2] !== 'b' ||
    mixedBracketedKeys[3] !== 'a' ||
    bracketedKeys.length !== 4 ||
    bracketedKeys[0] !== '1' ||
    bracketedKeys[1] !== '2' ||
    bracketedKeys[2] !== 'b' ||
    bracketedKeys[3] !== 'a' ||
    syncCount !== 4 ||
    frozenSyncCount !== 4 ||
    sequenceCount !== 4 ||
    frozenSequenceCount !== 4 ||
    asyncCount !== 4 ||
    frozenAsyncCount !== 4 ||
    asyncSequenceCount !== 4 ||
    frozenAsyncSequenceCount !== 4
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
  const frozenObj = Object.freeze(obj);
  const keys = Reflect.ownKeys(obj);
  const frozenKeys = Reflect.ownKeys(frozenObj);
  const mixedRootKeys = globalThis["Reflect"].ownKeys(obj);
  const mixedBracketedKeys = globalThis.Reflect["ownKeys"](obj);
  const bracketedKeys = globalThis["Reflect"]["ownKeys"](obj);
  let syncCount = 0;
  for (const key of Reflect.ownKeys(obj)) {
    syncCount += 1;
  }
  let frozenSyncCount = 0;
  for (const key of Reflect.ownKeys(frozenObj)) {
    frozenSyncCount += 1;
  }
  let sequenceCount = 0;
  for (const key of (0, Reflect.ownKeys(obj))) {
    sequenceCount += 1;
  }
  let frozenSequenceCount = 0;
  for (const key of (0, Reflect.ownKeys(frozenObj))) {
    frozenSequenceCount += 1;
  }
  let asyncCount = 0;
  for await (const key of Reflect.ownKeys(obj)) {
    asyncCount += 1;
  }
  let frozenAsyncCount = 0;
  for await (const key of Reflect.ownKeys(frozenObj)) {
    frozenAsyncCount += 1;
  }
  let asyncSequenceCount = 0;
  for await (const key of (0, Reflect.ownKeys(obj))) {
    asyncSequenceCount += 1;
  }
  let frozenAsyncSequenceCount = 0;
  for await (const key of (0, Reflect.ownKeys(frozenObj))) {
    frozenAsyncSequenceCount += 1;
  }
  if (
    keys.length !== 4 ||
    keys[0] !== '1' ||
    keys[1] !== '2' ||
    keys[2] !== 'b' ||
    keys[3] !== 'a' ||
    frozenKeys.length !== 4 ||
    frozenKeys[0] !== '1' ||
    frozenKeys[1] !== '2' ||
    frozenKeys[2] !== 'b' ||
    frozenKeys[3] !== 'a' ||
    mixedRootKeys.length !== 4 ||
    mixedRootKeys[0] !== '1' ||
    mixedRootKeys[1] !== '2' ||
    mixedRootKeys[2] !== 'b' ||
    mixedRootKeys[3] !== 'a' ||
    mixedBracketedKeys.length !== 4 ||
    mixedBracketedKeys[0] !== '1' ||
    mixedBracketedKeys[1] !== '2' ||
    mixedBracketedKeys[2] !== 'b' ||
    mixedBracketedKeys[3] !== 'a' ||
    bracketedKeys.length !== 4 ||
    bracketedKeys[0] !== '1' ||
    bracketedKeys[1] !== '2' ||
    bracketedKeys[2] !== 'b' ||
    bracketedKeys[3] !== 'a' ||
    syncCount !== 4 ||
    frozenSyncCount !== 4 ||
    sequenceCount !== 4 ||
    frozenSequenceCount !== 4 ||
    asyncCount !== 4 ||
    frozenAsyncCount !== 4 ||
    asyncSequenceCount !== 4 ||
    frozenAsyncSequenceCount !== 4
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
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

fn assert_inherited_browser_api_surface_reflect_own_keys(
    command: &str,
    filename: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        reflect_own_keys_test_source()
    } else {
        reflect_own_keys_source()
    };
    fs::write(&source_path, source).expect("write source");
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser"
  }
}"#,
    )
    .expect("write manifest");

    let mut command_line = Command::new(kali_bin());
    command_line
        .current_dir(dir.path())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node");
    if json_output {
        command_line.arg("--output").arg("json");
    }
    let output = command_line
        .arg(command)
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
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if command == "run" {
        assert!(stdout.contains("reflect ownKeys ok"), "stdout: {stdout}");
    } else {
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
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

fn assert_browser_checked_reflect_own_keys(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, reflect_own_keys_source()).expect("write source");

    let mut command = Command::new(kali_bin());
    command
        .current_dir(dir.path())
        .arg("check")
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
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "check");
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
        assert_eq!(json["payload"]["filesChecked"], 1);
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    }
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
fn run_supports_reflect_own_keys_in_ts_input_when_browser_harness_is_configured() {
    assert_browser_requested_reflect_own_keys("run", "main.ts");
}

#[test]
fn run_supports_reflect_own_keys_in_jsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_reflect_own_keys("run", "main.jsx");
}

#[test]
fn run_supports_reflect_own_keys_in_tsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_reflect_own_keys("run", "main.tsx");
}

#[test]
fn test_supports_reflect_own_keys_in_ts_input_when_browser_harness_is_configured() {
    assert_browser_requested_reflect_own_keys("test", "smoke.test.ts");
}

#[test]
fn test_supports_reflect_own_keys_in_jsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_reflect_own_keys("test", "smoke.test.jsx");
}

#[test]
fn test_supports_reflect_own_keys_in_tsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_reflect_own_keys("test", "smoke.test.tsx");
}

#[test]
fn run_supports_reflect_own_keys_in_js_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("run", "main.js", false);
}

#[test]
fn test_supports_reflect_own_keys_in_js_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("test", "smoke.test.js", false);
}

#[test]
fn run_supports_reflect_own_keys_in_ts_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("run", "main.ts", false);
}

#[test]
fn run_supports_reflect_own_keys_in_jsx_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("run", "main.jsx", false);
}

#[test]
fn run_supports_reflect_own_keys_in_tsx_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("run", "main.tsx", false);
}

#[test]
fn test_supports_reflect_own_keys_in_ts_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("test", "smoke.test.ts", false);
}

#[test]
fn test_supports_reflect_own_keys_in_jsx_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("test", "smoke.test.jsx", false);
}

#[test]
fn test_supports_reflect_own_keys_in_tsx_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("test", "smoke.test.tsx", false);
}

#[test]
fn json_run_supports_reflect_own_keys_in_js_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("run", "main.js", true);
}

#[test]
fn json_test_supports_reflect_own_keys_in_js_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("test", "smoke.test.js", true);
}

#[test]
fn json_run_supports_reflect_own_keys_in_ts_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("run", "main.ts", true);
}

#[test]
fn json_run_supports_reflect_own_keys_in_jsx_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("run", "main.jsx", true);
}

#[test]
fn json_run_supports_reflect_own_keys_in_tsx_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("run", "main.tsx", true);
}

#[test]
fn json_test_supports_reflect_own_keys_in_ts_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("test", "smoke.test.ts", true);
}

#[test]
fn json_test_supports_reflect_own_keys_in_jsx_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("test", "smoke.test.jsx", true);
}

#[test]
fn json_test_supports_reflect_own_keys_in_tsx_input_when_browser_api_surface_is_inherited() {
    assert_inherited_browser_api_surface_reflect_own_keys("test", "smoke.test.tsx", true);
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
fn json_run_supports_reflect_own_keys_in_ts_input_when_browser_harness_is_configured() {
    assert_json_browser_requested_reflect_own_keys("run", "main.ts");
}

#[test]
fn json_run_supports_reflect_own_keys_in_jsx_input_when_browser_harness_is_configured() {
    assert_json_browser_requested_reflect_own_keys("run", "main.jsx");
}

#[test]
fn json_run_supports_reflect_own_keys_in_tsx_input_when_browser_harness_is_configured() {
    assert_json_browser_requested_reflect_own_keys("run", "main.tsx");
}

#[test]
fn json_test_supports_reflect_own_keys_in_ts_input_when_browser_harness_is_configured() {
    assert_json_browser_requested_reflect_own_keys("test", "smoke.test.ts");
}

#[test]
fn json_test_supports_reflect_own_keys_in_jsx_input_when_browser_harness_is_configured() {
    assert_json_browser_requested_reflect_own_keys("test", "smoke.test.jsx");
}

#[test]
fn json_test_supports_reflect_own_keys_in_tsx_input_when_browser_harness_is_configured() {
    assert_json_browser_requested_reflect_own_keys("test", "smoke.test.tsx");
}

#[test]
fn check_accepts_reflect_own_keys_in_jsx_input_on_browser_surface() {
    assert_browser_checked_reflect_own_keys("main.jsx", false);
}

#[test]
fn check_accepts_reflect_own_keys_in_tsx_input_on_browser_surface() {
    assert_browser_checked_reflect_own_keys("main.tsx", false);
}

#[test]
fn json_check_accepts_reflect_own_keys_in_jsx_input_on_browser_surface() {
    assert_browser_checked_reflect_own_keys("main.jsx", true);
}

#[test]
fn json_check_accepts_reflect_own_keys_in_tsx_input_on_browser_surface() {
    assert_browser_checked_reflect_own_keys("main.tsx", true);
}

#[test]
fn build_emits_browser_bundle_reflect_own_keys_semantics_in_js_input() {
    assert_browser_bundle_reflect_own_keys("app.js", false);
}

#[test]
fn build_emits_browser_bundle_reflect_own_keys_semantics_in_ts_input() {
    assert_browser_bundle_reflect_own_keys("app.ts", false);
}

#[test]
fn build_emits_browser_bundle_reflect_own_keys_semantics_in_jsx_input() {
    assert_browser_bundle_reflect_own_keys("app.jsx", false);
}

#[test]
fn build_emits_browser_bundle_reflect_own_keys_semantics_in_tsx_input() {
    assert_browser_bundle_reflect_own_keys("app.tsx", false);
}

#[test]
fn json_build_emits_browser_bundle_reflect_own_keys_semantics_in_js_input() {
    assert_browser_bundle_reflect_own_keys("app.js", true);
}

#[test]
fn json_build_emits_browser_bundle_reflect_own_keys_semantics_in_ts_input() {
    assert_browser_bundle_reflect_own_keys("app.ts", true);
}

#[test]
fn json_build_emits_browser_bundle_reflect_own_keys_semantics_in_jsx_input() {
    assert_browser_bundle_reflect_own_keys("app.jsx", true);
}

#[test]
fn json_build_emits_browser_bundle_reflect_own_keys_semantics_in_tsx_input() {
    assert_browser_bundle_reflect_own_keys("app.tsx", true);
}
