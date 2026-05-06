use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn for_of_spread_source() -> &'static str {
    r##"// kali-tree-shake: forOfArrayIterationSpreadWrapper
export function forOfArrayIterationSpreadWrapper() {
  const values = [1, 2];
  for (const item of [...values]) {
    console.log(item);
  }
}
"##
}

fn for_await_spread_source() -> &'static str {
    r##"// kali-tree-shake: forAwaitArrayIterationSpreadWrapper
export async function forAwaitArrayIterationSpreadWrapper() {
  const values = [1, 2];
  for await (const item of [...values]) {
    console.log(item);
  }
}
"##
}

fn object_enumeration_spread_source() -> &'static str {
    r##"// kali-tree-shake: objectEnumerationSpreadWrapper
export async function objectEnumerationSpreadWrapper() {
  const frozenFromEntries = Object.freeze(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]));
  const frozenValues = [...Object.values(frozenFromEntries)];
  const frozenKeys = [...Object.keys(frozenFromEntries)];
  const frozenEntries = [...Object.entries(frozenFromEntries)];

  for (const value of [...Object.values(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]))]) {
    console.log(value);
  }
  for (const value of [...globalThis.Object.values(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]))]) {
    console.log(value);
  }
  for (const value of [...globalThis.Object["values"](Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]))]) {
    console.log(value);
  }
  for (const value of [...globalThis["Object"].values(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]))]) {
    console.log(value);
  }
  for (const value of [...globalThis["Object"]["values"](Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]))]) {
    console.log(value);
  }
  for (const key of [...Object.keys(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]))]) {
    console.log(key);
  }
  for (const key of [...globalThis.Object.keys(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]))]) {
    console.log(key);
  }
  for (const key of [...globalThis.Object["keys"](Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]))]) {
    console.log(key);
  }
  for (const key of [...globalThis["Object"].keys(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]))]) {
    console.log(key);
  }
  for (const key of [...globalThis["Object"]["keys"](Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]))]) {
    console.log(key);
  }
  for (const entry of [...Object.entries(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]))]) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of [...globalThis.Object.entries(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]))]) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of [...globalThis.Object["entries"](Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]))]) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of [...globalThis["Object"].entries(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]))]) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of [...globalThis["Object"]["entries"](Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]))]) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for await (const value of [...Object.values(Object.fromEntries([["c", 4], ["d", 5], ["c", 6]]))]) {
    console.log(value);
  }
  for await (const value of [...globalThis.Object.values(Object.fromEntries([["c", 4], ["d", 5], ["c", 6]]))]) {
    console.log(value);
  }
  for await (const value of [...globalThis.Object["values"](Object.fromEntries([["c", 4], ["d", 5], ["c", 6]]))]) {
    console.log(value);
  }
  for await (const value of [...globalThis["Object"].values(Object.fromEntries([["c", 4], ["d", 5], ["c", 6]]))]) {
    console.log(value);
  }
  for await (const value of [...globalThis["Object"]["values"](Object.fromEntries([["c", 4], ["d", 5], ["c", 6]]))]) {
    console.log(value);
  }
  for await (const key of [...Object.keys(Object.fromEntries([["c", 4], ["d", 5], ["c", 6]]))]) {
    console.log(key);
  }
  for await (const key of [...globalThis.Object.keys(Object.fromEntries([["c", 4], ["d", 5], ["c", 6]]))]) {
    console.log(key);
  }
  for await (const key of [...globalThis.Object["keys"](Object.fromEntries([["c", 4], ["d", 5], ["c", 6]]))]) {
    console.log(key);
  }
  for await (const key of [...globalThis["Object"].keys(Object.fromEntries([["c", 4], ["d", 5], ["c", 6]]))]) {
    console.log(key);
  }
  for await (const key of [...globalThis["Object"]["keys"](Object.fromEntries([["c", 4], ["d", 5], ["c", 6]]))]) {
    console.log(key);
  }
  for await (const entry of [...Object.entries(Object.fromEntries([["c", 4], ["d", 5], ["c", 6]]))]) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for await (const entry of [...globalThis.Object.entries(Object.fromEntries([["c", 4], ["d", 5], ["c", 6]]))]) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for await (const entry of [...globalThis.Object["entries"](Object.fromEntries([["c", 4], ["d", 5], ["c", 6]]))]) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for await (const entry of [...globalThis["Object"].entries(Object.fromEntries([["c", 4], ["d", 5], ["c", 6]]))]) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for await (const entry of [...globalThis["Object"]["entries"](Object.fromEntries([["c", 4], ["d", 5], ["c", 6]]))]) {
    console.log(entry[0]);
    console.log(entry[1]);
  }

  if (
    frozenValues.length !== 2 ||
    frozenValues[0] !== 3 ||
    frozenValues[1] !== 2 ||
    frozenKeys.length !== 2 ||
    frozenKeys[0] !== 'b' ||
    frozenKeys[1] !== 'a' ||
    frozenEntries.length !== 2 ||
    frozenEntries[0][0] !== 'b' ||
    frozenEntries[0][1] !== 3 ||
    frozenEntries[1][0] !== 'a' ||
    frozenEntries[1][1] !== 2
  ) {
    throw new Error('unexpected frozen Object.fromEntries spread iteration semantics');
  }
}
"##
}
fn assert_browser_bundle_array_iteration_spread(
    filename: &str,
    json_output: bool,
    source: &str,
    harness_function: &str,
) {
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
        &format!(
            r#"const mod = await import(bundleJs.href);
await mod.{harness_function}();
"#
        ),
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
    assert!(stdout.contains("1\n"), "stdout: {stdout}");
    assert!(stdout.contains("2\n"), "stdout: {stdout}");
}

fn assert_browser_bundle_object_enumeration_spread(
    filename: &str,
    json_output: bool,
    source: &str,
    harness_function: &str,
) {
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
        &format!(
            r#"const mod = await import(bundleJs.href);
await mod.{harness_function}();
"#
        ),
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
    let lines: Vec<&str> = stdout.lines().collect();
    let mut expected = Vec::new();
    for _ in 0..5 {
        expected.extend(["3", "2"]);
    }
    for _ in 0..5 {
        expected.extend(["b", "a"]);
    }
    for _ in 0..5 {
        expected.extend(["b", "3", "a", "2"]);
    }
    for _ in 0..5 {
        expected.extend(["6", "5"]);
    }
    for _ in 0..5 {
        expected.extend(["c", "d"]);
    }
    for _ in 0..5 {
        expected.extend(["c", "6", "d", "5"]);
    }
    assert_eq!(lines, expected);
}

#[test]
fn build_emits_for_of_spread_in_js_input() {
    assert_browser_bundle_array_iteration_spread(
        "app.js",
        false,
        for_of_spread_source(),
        "forOfArrayIterationSpreadWrapper",
    );
}

#[test]
fn json_build_emits_for_of_spread_in_js_input() {
    assert_browser_bundle_array_iteration_spread(
        "app.js",
        true,
        for_of_spread_source(),
        "forOfArrayIterationSpreadWrapper",
    );
}

#[test]
fn build_emits_for_of_spread_in_ts_input() {
    assert_browser_bundle_array_iteration_spread(
        "app.ts",
        false,
        for_of_spread_source(),
        "forOfArrayIterationSpreadWrapper",
    );
}

#[test]
fn json_build_emits_for_of_spread_in_ts_input() {
    assert_browser_bundle_array_iteration_spread(
        "app.ts",
        true,
        for_of_spread_source(),
        "forOfArrayIterationSpreadWrapper",
    );
}

#[test]
fn build_emits_for_await_spread_in_js_input() {
    assert_browser_bundle_array_iteration_spread(
        "app.js",
        false,
        for_await_spread_source(),
        "forAwaitArrayIterationSpreadWrapper",
    );
}

#[test]
fn json_build_emits_for_await_spread_in_js_input() {
    assert_browser_bundle_array_iteration_spread(
        "app.js",
        true,
        for_await_spread_source(),
        "forAwaitArrayIterationSpreadWrapper",
    );
}

#[test]
fn build_emits_for_await_spread_in_ts_input() {
    assert_browser_bundle_array_iteration_spread(
        "app.ts",
        false,
        for_await_spread_source(),
        "forAwaitArrayIterationSpreadWrapper",
    );
}

#[test]
fn json_build_emits_for_await_spread_in_ts_input() {
    assert_browser_bundle_array_iteration_spread(
        "app.ts",
        true,
        for_await_spread_source(),
        "forAwaitArrayIterationSpreadWrapper",
    );
}

#[test]
fn build_emits_for_of_spread_in_jsx_and_tsx_input() {
    for filename in ["app.jsx", "app.tsx"] {
        assert_browser_bundle_array_iteration_spread(
            filename,
            false,
            for_of_spread_source(),
            "forOfArrayIterationSpreadWrapper",
        );
        assert_browser_bundle_array_iteration_spread(
            filename,
            true,
            for_of_spread_source(),
            "forOfArrayIterationSpreadWrapper",
        );
    }
}

#[test]
fn build_emits_for_await_spread_in_jsx_and_tsx_input() {
    for filename in ["app.jsx", "app.tsx"] {
        assert_browser_bundle_array_iteration_spread(
            filename,
            false,
            for_await_spread_source(),
            "forAwaitArrayIterationSpreadWrapper",
        );
        assert_browser_bundle_array_iteration_spread(
            filename,
            true,
            for_await_spread_source(),
            "forAwaitArrayIterationSpreadWrapper",
        );
    }
}

#[test]
fn build_emits_object_enumeration_spread_in_js_input() {
    assert_browser_bundle_object_enumeration_spread(
        "app.js",
        false,
        object_enumeration_spread_source(),
        "objectEnumerationSpreadWrapper",
    );
}

#[test]
fn json_build_emits_object_enumeration_spread_in_js_input() {
    assert_browser_bundle_object_enumeration_spread(
        "app.js",
        true,
        object_enumeration_spread_source(),
        "objectEnumerationSpreadWrapper",
    );
}

#[test]
fn build_emits_object_enumeration_spread_in_ts_input() {
    assert_browser_bundle_object_enumeration_spread(
        "app.ts",
        false,
        object_enumeration_spread_source(),
        "objectEnumerationSpreadWrapper",
    );
}

#[test]
fn json_build_emits_object_enumeration_spread_in_ts_input() {
    assert_browser_bundle_object_enumeration_spread(
        "app.ts",
        true,
        object_enumeration_spread_source(),
        "objectEnumerationSpreadWrapper",
    );
}

#[test]
fn build_emits_object_enumeration_spread_in_jsx_and_tsx_input() {
    for filename in ["app.jsx", "app.tsx"] {
        assert_browser_bundle_object_enumeration_spread(
            filename,
            false,
            object_enumeration_spread_source(),
            "objectEnumerationSpreadWrapper",
        );
        assert_browser_bundle_object_enumeration_spread(
            filename,
            true,
            object_enumeration_spread_source(),
            "objectEnumerationSpreadWrapper",
        );
    }
}
