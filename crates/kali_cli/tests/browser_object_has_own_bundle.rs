use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_object_has_own_js_source() -> &'static str {
    r##"// kali-tree-shake: browserObjectHasOwn
function browserObjectHasOwn() {
  const object = { a: 1, "b": 2 };
  const alias = object;
  if (!globalThis["Object"]["prototype"].hasOwnProperty["call"](alias, "a") || !globalThis["Object"]["hasOwn"](alias, "a") || !globalThis.Object["hasOwn"](alias, "a") || !globalThis["Object"].hasOwn(alias, "a") || !globalThis["Object"].prototype["hasOwnProperty"]["call"](alias, "a") || !globalThis["Object"].prototype.hasOwnProperty.call(alias, "a") || !globalThis.Object.prototype["hasOwnProperty"]["call"](alias, "a") || !globalThis.Object["prototype"].hasOwnProperty.call(alias, "a") || !globalThis.Object["prototype"]["hasOwnProperty"]["call"](alias, "a") || !globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](alias, "a")) {
    throw new Error('unexpected browser Object.hasOwn result');
  }
  console.log('browser object hasOwn ok');
}
"##
}

fn browser_bundle_object_has_own_ts_source() -> &'static str {
    r##"// kali-tree-shake: browserObjectHasOwn
function browserObjectHasOwn() {
  const object = ({ a: 1, "b": 2 } as const);
  const alias = object;
  if (!globalThis["Object"]["prototype"].hasOwnProperty["call"](alias, "a") || !globalThis["Object"]["hasOwn"](alias, "a") || !globalThis.Object["hasOwn"](alias, "a") || !globalThis["Object"].hasOwn(alias, "a") || !globalThis["Object"].prototype["hasOwnProperty"]["call"](alias, "a") || !globalThis["Object"].prototype.hasOwnProperty.call(alias, "a") || !globalThis.Object.prototype["hasOwnProperty"]["call"](alias, "a") || !globalThis.Object["prototype"].hasOwnProperty.call(alias, "a") || !globalThis.Object["prototype"]["hasOwnProperty"]["call"](alias, "a") || !globalThis["Object"]["prototype"]["hasOwnProperty"]["call"](alias, "a")) {
    throw new Error('unexpected browser Object.hasOwn result');
  }
  console.log('browser object hasOwn ok');
}
"##
}

fn assert_browser_bundle_object_has_own(filename: &str, json_output: bool, source: &'static str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

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
        assert!(envelope["errors"]
            .as_array()
            .expect("errors array")
            .is_empty());
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
await mod.browserObjectHasOwn();
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
    assert!(
        stdout.contains("browser object hasOwn ok"),
        "stdout: {stdout}"
    );
}

#[test]
fn build_emits_browser_object_has_own_in_js_input() {
    assert_browser_bundle_object_has_own(
        "app.js",
        false,
        browser_bundle_object_has_own_js_source(),
    );
}

#[test]
fn build_emits_browser_object_has_own_in_jsx_input() {
    assert_browser_bundle_object_has_own(
        "app.jsx",
        false,
        browser_bundle_object_has_own_js_source(),
    );
}

#[test]
fn build_emits_browser_object_has_own_in_ts_input() {
    assert_browser_bundle_object_has_own(
        "app.ts",
        false,
        browser_bundle_object_has_own_ts_source(),
    );
}

#[test]
fn build_emits_browser_object_has_own_in_tsx_input() {
    assert_browser_bundle_object_has_own(
        "app.tsx",
        false,
        browser_bundle_object_has_own_ts_source(),
    );
}

#[test]
fn json_build_emits_browser_object_has_own_in_js_input() {
    assert_browser_bundle_object_has_own("app.js", true, browser_bundle_object_has_own_js_source());
}

#[test]
fn json_build_emits_browser_object_has_own_in_jsx_input() {
    assert_browser_bundle_object_has_own(
        "app.jsx",
        true,
        browser_bundle_object_has_own_js_source(),
    );
}

#[test]
fn json_build_emits_browser_object_has_own_in_ts_input() {
    assert_browser_bundle_object_has_own("app.ts", true, browser_bundle_object_has_own_ts_source());
}

#[test]
fn json_build_emits_browser_object_has_own_in_tsx_input() {
    assert_browser_bundle_object_has_own(
        "app.tsx",
        true,
        browser_bundle_object_has_own_ts_source(),
    );
}
