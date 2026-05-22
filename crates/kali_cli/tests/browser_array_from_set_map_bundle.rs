use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_array_from_set_map_source() -> &'static str {
    r##"// kali-tree-shake: browserArrayFromSetMapWrappers
export async function browserArrayFromSetMapWrappers() {
  const setValues = [1, 2, 1];
  const mapValues = [[1, 2], [1, 3], [4, 5]];
  for (const value of Array.from(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze(globalThis["Array"].from)(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze(globalThis["Array"]["from"])(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze(globalThis['Array']['from'])(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze(globalThis.Array['from'])(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze(globalThis['Array'].from)(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis["Array"]))["from"](new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis['Array']))["from"](new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis.Array))["from"](new Set(setValues))) {
    console.log(value);
  }
  for await (const value of Array.from(new Set(setValues))) {
    console.log(value);
  }
  for (const entry of Array.from(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for await (const entry of Array.from(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
}
"##
}

fn assert_browser_bundle_array_from_set_map(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = browser_bundle_array_from_set_map_source();
    assert!(source.contains("Object.freeze((globalThis[\"Array\"]))[\"from\"]"));
    assert!(source.contains("Object.freeze((globalThis['Array']))[\"from\"]"));
    assert!(source.contains("Object.freeze((globalThis.Array))[\"from\"]"));
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
        "const mod = await import(bundleJs.href);\nawait mod.browserArrayFromSetMapWrappers();\n",
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
        stdout.contains("1\n2\n1\n2\n1\n2\n1\n2\n1\n3\n4\n5\n1\n3\n4\n5\n"),
        "stdout: {stdout}"
    );
}

#[test]
fn build_emits_array_from_new_set_and_new_map_in_js_input() {
    assert_browser_bundle_array_from_set_map("app.js", false);
}

#[test]
fn json_build_emits_array_from_new_set_and_new_map_in_js_input() {
    assert_browser_bundle_array_from_set_map("app.js", true);
}

#[test]
fn build_emits_array_from_new_set_and_new_map_in_ts_input() {
    assert_browser_bundle_array_from_set_map("app.ts", false);
}

#[test]
fn json_build_emits_array_from_new_set_and_new_map_in_ts_input() {
    assert_browser_bundle_array_from_set_map("app.ts", true);
}

#[test]
fn build_emits_array_from_new_set_and_new_map_in_jsx_and_tsx_input() {
    for filename in ["app.jsx", "app.tsx"] {
        assert_browser_bundle_array_from_set_map(filename, false);
        assert_browser_bundle_array_from_set_map(filename, true);
    }
}
