use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_direct_object_values_iteration_source() -> &'static str {
    r##"// kali-tree-shake: browserDirectObjectValuesIteration
function assertObjectValuesIteration(values) {
  if (values.length !== 2 || values[0] !== 1 || values[1] !== 2) {
    throw new Error('unexpected Object.values iteration semantics');
  }
}

function browserDirectObjectValuesIteration() {
  const seen = [];
  for (const value of Object.values({ "b": 1, "a": 2 })) {
    seen.push(value);
  }
  const mixed = [];
  for (const value of globalThis.Object["values"]({ "b": 1, "a": 2 })) {
    mixed.push(value);
  }
  const mixedBracketed = [];
  for (const value of globalThis["Object"].values({ "b": 1, "a": 2 })) {
    mixedBracketed.push(value);
  }
  const bracketed = [];
  for (const value of globalThis["Object"]["values"]({ "b": 1, "a": 2 })) {
    bracketed.push(value);
  }
  assertObjectValuesIteration(seen);
  assertObjectValuesIteration(mixed);
  assertObjectValuesIteration(mixedBracketed);
  assertObjectValuesIteration(bracketed);
}
"##
}

fn browser_bundle_global_object_values_iteration_source() -> &'static str {
    r##"// kali-tree-shake: browserGlobalObjectValuesIteration
function assertObjectValuesIteration(values) {
  if (values.length !== 2 || values[0] !== 1 || values[1] !== 2) {
    throw new Error('unexpected Object.values iteration semantics');
  }
}

function browserGlobalObjectValuesIteration() {
  const values = { "b": 1, "a": 2 };
  const alias = values;
  const seen = [];
  for (const value of globalThis.Object.values(alias)) {
    seen.push(value);
  }
  const mixed = [];
  for (const value of globalThis.Object["values"](alias)) {
    mixed.push(value);
  }
  const mixedBracketed = [];
  for (const value of globalThis["Object"].values(alias)) {
    mixedBracketed.push(value);
  }
  const bracketed = [];
  for (const value of globalThis["Object"]["values"](alias)) {
    bracketed.push(value);
  }
  assertObjectValuesIteration(seen);
  assertObjectValuesIteration(mixed);
  assertObjectValuesIteration(mixedBracketed);
  assertObjectValuesIteration(bracketed);
}
"##
}

fn assert_browser_bundle_direct_object_values_iteration(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_direct_object_values_iteration_source(),
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
await mod.browserDirectObjectValuesIteration();
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

fn assert_browser_bundle_global_object_values_iteration(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_global_object_values_iteration_source(),
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
await mod.browserGlobalObjectValuesIteration();
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
fn build_emits_global_object_values_iteration_semantics_in_js_input() {
    assert_browser_bundle_global_object_values_iteration("app.js", false);
}

#[test]
fn build_emits_global_object_values_iteration_semantics_in_ts_jsx_tsx_input() {
    for filename in ["app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle_global_object_values_iteration(filename, false);
    }
}

#[test]
fn json_build_emits_global_object_values_iteration_semantics_in_js_input() {
    assert_browser_bundle_global_object_values_iteration("app.js", true);
}

#[test]
fn json_build_emits_global_object_values_iteration_semantics_in_ts_jsx_tsx_input() {
    for filename in ["app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle_global_object_values_iteration(filename, true);
    }
}

#[test]
fn build_emits_direct_object_values_iteration_semantics_in_js_ts_jsx_tsx_input() {
    for filename in ["app.js", "app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle_direct_object_values_iteration(filename, false);
    }
}

#[test]
fn json_build_emits_direct_object_values_iteration_semantics_in_js_ts_jsx_tsx_input() {
    for filename in ["app.js", "app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle_direct_object_values_iteration(filename, true);
    }
}
