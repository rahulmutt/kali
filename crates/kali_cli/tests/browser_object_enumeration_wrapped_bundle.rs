use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_wrapped_object_enumeration_source() -> &'static str {
    r##"// kali-tree-shake: browserWrappedObjectEnumeration
function assertWrappedObjectEnumeration(keys, values, entries) {
  if (
    keys.length !== 4 ||
    keys[0] !== '1' ||
    keys[1] !== '2' ||
    keys[2] !== 'b' ||
    keys[3] !== 'a' ||
    values.length !== 4 ||
    values[0] !== 4 ||
    values[1] !== 2 ||
    values[2] !== 1 ||
    values[3] !== 3 ||
    entries.length !== 4 ||
    entries[0][0] !== '1' ||
    entries[0][1] !== 4 ||
    entries[1][0] !== '2' ||
    entries[1][1] !== 2 ||
    entries[2][0] !== 'b' ||
    entries[2][1] !== 1 ||
    entries[3][0] !== 'a' ||
    entries[3][1] !== 3
  ) {
    throw new Error('unexpected wrapped object enumeration ordering');
  }
}

function browserWrappedObjectEnumeration() {
  const wrappedConst = ({ "b": 1, "2": 2, "a": 3, "1": 4 } as const);
  const wrappedSatisfies = ({ "b": 1, "2": 2, "a": 3, "1": 4 } satisfies unknown);
  const frozenFromEntries = Object.freeze(Object.fromEntries([["b", 1], ["2", 2], ["a", 3], ["1", 4]]));

  const constKeys = Object.keys(wrappedConst);
  const constValues = Object.values(wrappedConst);
  const constEntries = Object.entries(wrappedConst);
  assertWrappedObjectEnumeration(constKeys, constValues, constEntries);

  const satisfiesKeys = Object.keys(wrappedSatisfies);
  const satisfiesValues = Object.values(wrappedSatisfies);
  const satisfiesEntries = Object.entries(wrappedSatisfies);
  assertWrappedObjectEnumeration(satisfiesKeys, satisfiesValues, satisfiesEntries);

  const frozenKeys = Object.keys(frozenFromEntries);
  const frozenValues = Object.values(frozenFromEntries);
  const frozenEntries = Object.entries(frozenFromEntries);
  assertWrappedObjectEnumeration(frozenKeys, frozenValues, frozenEntries);
}
"##
}

fn assert_browser_bundle_wrapped_object_enumeration(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_wrapped_object_enumeration_source(),
    )
    .expect("write source");

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
await mod.browserWrappedObjectEnumeration();
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
fn build_emits_wrapped_object_enumeration_semantics_in_ts_input() {
    assert_browser_bundle_wrapped_object_enumeration("app.ts", false);
}

#[test]
fn build_emits_wrapped_object_enumeration_semantics_in_jsx_tsx_input() {
    for filename in ["app.jsx", "app.tsx"] {
        assert_browser_bundle_wrapped_object_enumeration(filename, false);
    }
}

#[test]
fn json_build_emits_wrapped_object_enumeration_semantics_in_ts_input() {
    assert_browser_bundle_wrapped_object_enumeration("app.ts", true);
}

#[test]
fn json_build_emits_wrapped_object_enumeration_semantics_in_jsx_tsx_input() {
    for filename in ["app.jsx", "app.tsx"] {
        assert_browser_bundle_wrapped_object_enumeration(filename, true);
    }
}
