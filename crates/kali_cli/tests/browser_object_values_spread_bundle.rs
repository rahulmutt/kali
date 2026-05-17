use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_object_values_spread_source() -> &'static str {
    r##"// kali-tree-shake: browserObjectValuesSpreadIteration
function assertObjectValuesSpreadIteration(values) {
  if (values.length !== 2 || values[0] !== 3 || values[1] !== 2) {
    throw new Error('unexpected Object.values spread iteration semantics');
  }
}

function browserObjectValuesSpreadIteration() {
  const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);
  const bracketedFromEntries = globalThis["Object"]["fromEntries"]([["b", 1], ["a", 2], ["b", 3]]);
  const frozenFromEntries = Object.freeze(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]));
  const collected = [...Object.values(fromEntries)];
  const globalCollected = [...globalThis.Object.values(fromEntries)];
  const bracketedCollected = [...Object.values(bracketedFromEntries)];
  const frozenCollected = [...Object.values(frozenFromEntries)];
  const mixedCollected = [...globalThis.Object["values"](fromEntries)];
  const mixedBracketedCollected = [...globalThis["Object"].values(fromEntries)];
  const singleBracketedCollected = [...globalThis['Object']['values'](fromEntries)];
  const singleBracketedPropertyCollected = [...globalThis['Object'].values(fromEntries)];
  const bracketedAliasCollected = [...globalThis["Object"]["values"](fromEntries)];
  const bracketedAliasFromEntriesCollected = [...globalThis["Object"]["values"](bracketedFromEntries)];
  const bracketedAliasFrozenCollected = [...globalThis["Object"]["values"](frozenFromEntries)];
  assertObjectValuesSpreadIteration(collected);
  assertObjectValuesSpreadIteration(globalCollected);
  assertObjectValuesSpreadIteration(bracketedCollected);
  assertObjectValuesSpreadIteration(frozenCollected);
  assertObjectValuesSpreadIteration(mixedCollected);
  assertObjectValuesSpreadIteration(mixedBracketedCollected);
  assertObjectValuesSpreadIteration(singleBracketedCollected);
  assertObjectValuesSpreadIteration(singleBracketedPropertyCollected);
  assertObjectValuesSpreadIteration(bracketedAliasCollected);
  assertObjectValuesSpreadIteration(bracketedAliasFromEntriesCollected);
  assertObjectValuesSpreadIteration(bracketedAliasFrozenCollected);
  console.log('browser object values spread iteration ok');
}
"##
}

fn assert_browser_bundle_object_values_spread(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, browser_bundle_object_values_spread_source()).expect("write source");

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
await mod.browserObjectValuesSpreadIteration();
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
        stdout.contains("browser object values spread iteration ok"),
        "stdout: {stdout}"
    );
}

#[test]
fn build_emits_object_values_spread_iteration_in_js_input() {
    assert_browser_bundle_object_values_spread("app.js", false);
}

#[test]
fn build_emits_object_values_spread_iteration_in_ts_input() {
    assert_browser_bundle_object_values_spread("app.ts", false);
}

#[test]
fn build_emits_object_values_spread_iteration_in_jsx_input() {
    assert_browser_bundle_object_values_spread("app.jsx", false);
}

#[test]
fn build_emits_object_values_spread_iteration_in_tsx_input() {
    assert_browser_bundle_object_values_spread("app.tsx", false);
}

#[test]
fn json_build_emits_object_values_spread_iteration_in_js_input() {
    assert_browser_bundle_object_values_spread("app.js", true);
}

#[test]
fn json_build_emits_object_values_spread_iteration_in_ts_input() {
    assert_browser_bundle_object_values_spread("app.ts", true);
}

#[test]
fn json_build_emits_object_values_spread_iteration_in_jsx_input() {
    assert_browser_bundle_object_values_spread("app.jsx", true);
}

#[test]
fn json_build_emits_object_values_spread_iteration_in_tsx_input() {
    assert_browser_bundle_object_values_spread("app.tsx", true);
}
