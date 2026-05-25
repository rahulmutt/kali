use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_object_keys_entries_spread_source() -> &'static str {
    r##"// kali-tree-shake: browserObjectKeysEntriesSpread
function assertObjectKeysIteration(keys) {
  if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {
    throw new Error('unexpected Object.keys spread iteration semantics');
  }
}

function assertObjectEntriesIteration(entries) {
  if (
    entries.length !== 2 ||
    entries[0][0] !== 'b' ||
    entries[0][1] !== 3 ||
    entries[1][0] !== 'a' ||
    entries[1][1] !== 2
  ) {
    throw new Error('unexpected Object.entries spread iteration semantics');
  }
}

function browserObjectKeysEntriesSpread() {
  const fromEntries = Object.freeze(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]));
  const collectedKeys = [...Object.keys(fromEntries)];
  const globalKeys = [...globalThis.Object.keys(fromEntries)];
  const mixedKeys = [...globalThis.Object["keys"](fromEntries)];
  const mixedBracketedKeys = [...globalThis["Object"].keys(fromEntries)];
  const bracketedKeys = [...globalThis["Object"]["keys"](fromEntries)];
  const singleBracketedKeys = [...globalThis['Object']['keys'](fromEntries)];
  const parenthesizedReceiverBracketedKeys = [...Object.freeze((globalThis["Object"])["keys"])(fromEntries)];
  const parenthesizedSingleQuotedReceiverBracketedKeys = [...Object.freeze((globalThis['Object'])['keys'])(fromEntries)];
  const parenthesizedBracketedKeys = [...Object.freeze((globalThis["Object"]).keys)(fromEntries)];
  const collectedEntries = [...Object.entries(fromEntries)];
  const globalEntries = [...globalThis.Object.entries(fromEntries)];
  const mixedEntries = [...globalThis.Object["entries"](fromEntries)];
  const mixedBracketedEntries = [...globalThis["Object"].entries(fromEntries)];
  const bracketedEntries = [...globalThis["Object"]["entries"](fromEntries)];
  const singleBracketedEntries = [...globalThis['Object']['entries'](fromEntries)];
  const parenthesizedReceiverBracketedEntries = [...Object.freeze((globalThis["Object"])["entries"])(fromEntries)];
  const parenthesizedSingleQuotedReceiverBracketedEntries = [...Object.freeze((globalThis['Object'])['entries'])(fromEntries)];
  const parenthesizedBracketedEntries = [...Object.freeze((globalThis["Object"]).entries)(fromEntries)];

  assertObjectKeysIteration(collectedKeys);
  assertObjectKeysIteration(globalKeys);
  assertObjectKeysIteration(mixedKeys);
  assertObjectKeysIteration(mixedBracketedKeys);
  assertObjectKeysIteration(bracketedKeys);
  assertObjectKeysIteration(singleBracketedKeys);
  assertObjectKeysIteration(parenthesizedReceiverBracketedKeys);
  assertObjectKeysIteration(parenthesizedSingleQuotedReceiverBracketedKeys);
  assertObjectKeysIteration(parenthesizedBracketedKeys);
  assertObjectEntriesIteration(collectedEntries);
  assertObjectEntriesIteration(globalEntries);
  assertObjectEntriesIteration(mixedEntries);
  assertObjectEntriesIteration(mixedBracketedEntries);
  assertObjectEntriesIteration(bracketedEntries);
  assertObjectEntriesIteration(singleBracketedEntries);
  assertObjectEntriesIteration(parenthesizedReceiverBracketedEntries);
  assertObjectEntriesIteration(parenthesizedSingleQuotedReceiverBracketedEntries);
  assertObjectEntriesIteration(parenthesizedBracketedEntries);
}
"##
}

fn assert_browser_bundle_object_keys_entries_spread(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_object_keys_entries_spread_source(),
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
await mod.browserObjectKeysEntriesSpread();
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
fn build_emits_object_keys_entries_spread_semantics_in_ts_input() {
    assert_browser_bundle_object_keys_entries_spread("app.ts", false);
}

#[test]
fn build_emits_object_keys_entries_spread_semantics_in_js_input() {
    assert_browser_bundle_object_keys_entries_spread("app.js", false);
}

#[test]
fn build_emits_object_keys_entries_spread_semantics_in_jsx_tsx_input() {
    for filename in ["app.jsx", "app.tsx"] {
        assert_browser_bundle_object_keys_entries_spread(filename, false);
    }
}

#[test]
fn json_build_emits_object_keys_entries_spread_semantics_in_ts_input() {
    assert_browser_bundle_object_keys_entries_spread("app.ts", true);
}

#[test]
fn json_build_emits_object_keys_entries_spread_semantics_in_js_input() {
    assert_browser_bundle_object_keys_entries_spread("app.js", true);
}

#[test]
fn json_build_emits_object_keys_entries_spread_semantics_in_jsx_tsx_input() {
    for filename in ["app.jsx", "app.tsx"] {
        assert_browser_bundle_object_keys_entries_spread(filename, true);
    }
}
