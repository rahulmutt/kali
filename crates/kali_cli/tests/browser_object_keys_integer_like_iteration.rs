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
