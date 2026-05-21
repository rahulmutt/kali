use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

use kali_runtime::{
    browser_bundle_harness_script, browser_harness_command_parts_for, BROWSER_HARNESS_COMMAND_ENV,
};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_frozen_set_map_constructor_result_run_source() -> &'static str {
    r##"async function browserFrozenSetMapConstructorResult() {
  const values = [1, 2, 1];
  for await (const value of Object.freeze(new Set(values))) {
    console.log(value);
  }
  for await (const entry of Object.freeze(new Map([[1, 2], [1, 3], [4, 5]]))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
}

browserFrozenSetMapConstructorResult();
"##
}

fn browser_frozen_set_map_constructor_result_test_source() -> &'static str {
    r##"Kali.test('for await frozen set/map constructor results', () => {
  async function browserFrozenSetMapConstructorResult() {
    const values = [1, 2, 1];
    for await (const value of Object.freeze(new Set(values))) {
      console.log(value);
    }
    for await (const entry of Object.freeze(new Map([[1, 2], [1, 3], [4, 5]]))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
  }

  return browserFrozenSetMapConstructorResult();
});
"##
}

fn browser_frozen_set_map_constructor_result_bundle_source() -> &'static str {
    r##"// kali-tree-shake: browserFrozenSetMapConstructorResult
export async function browserFrozenSetMapConstructorResult() {
  const values = [1, 2, 1];
  for await (const value of Object.freeze(new Set(values))) {
    console.log(value);
  }
  for await (const entry of Object.freeze(new Map([[1, 2], [1, 3], [4, 5]]))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
}
"##
}

fn browser_parenthesized_frozen_set_map_constructor_result_run_source() -> &'static str {
    r##"async function browserParenthesizedFrozenSetMapConstructorResult() {
  const values = [1, 2, 1];
  for await (const value of Object.freeze((new Set(values)))) {
    console.log(value);
  }
  for await (const entry of Object.freeze((new Map([[1, 2], [1, 3], [4, 5]])))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
}

browserParenthesizedFrozenSetMapConstructorResult();
"##
}

fn browser_parenthesized_frozen_set_map_constructor_result_test_source() -> &'static str {
    r##"Kali.test('for await parenthesized frozen set/map constructor results', () => {
  async function browserParenthesizedFrozenSetMapConstructorResult() {
    const values = [1, 2, 1];
    for await (const value of Object.freeze((new Set(values)))) {
      console.log(value);
    }
    for await (const entry of Object.freeze((new Map([[1, 2], [1, 3], [4, 5]])))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
  }

  return browserParenthesizedFrozenSetMapConstructorResult();
});
"##
}

fn browser_parenthesized_frozen_set_map_constructor_result_bundle_source() -> &'static str {
    r##"// kali-tree-shake: browserParenthesizedFrozenSetMapConstructorResult
export async function browserParenthesizedFrozenSetMapConstructorResult() {
  const values = [1, 2, 1];
  for await (const value of Object.freeze((new Set(values)))) {
    console.log(value);
  }
  for await (const entry of Object.freeze((new Map([[1, 2], [1, 3], [4, 5]])))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
}
"##
}

fn browser_frozen_object_helper_iteration_run_source() -> &'static str {
    r##"async function browserFrozenObjectHelperIterationTargets() {
  const object = Object.fromEntries([["b", 1], ["a", 2]]);
  for await (const key of Object.freeze(Object.keys)(object)) {
    console.log(key);
  }
  for await (const value of Object.freeze(Object.values)(object)) {
    console.log(value);
  }
  for await (const entry of Object.freeze(globalThis["Object"]["entries"])(object)) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
}

browserFrozenObjectHelperIterationTargets();
"##
}

fn browser_frozen_object_helper_iteration_test_source() -> &'static str {
    r##"Kali.test('for await frozen object helper iteration targets', () => {
  async function browserFrozenObjectHelperIterationTargets() {
    const object = Object.fromEntries([["b", 1], ["a", 2]]);
    for await (const key of Object.freeze(Object.keys)(object)) {
      console.log(key);
    }
    for await (const value of Object.freeze(Object.values)(object)) {
      console.log(value);
    }
    for await (const entry of Object.freeze(globalThis["Object"]["entries"])(object)) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
  }

  return browserFrozenObjectHelperIterationTargets();
});
"##
}

fn browser_frozen_object_helper_iteration_bundle_source() -> &'static str {
    r##"// kali-tree-shake: browserFrozenObjectHelperIterationTargets
export async function browserFrozenObjectHelperIterationTargets() {
  const object = Object.fromEntries([["b", 1], ["a", 2]]);
  for await (const key of Object.freeze(Object.keys)(object)) {
    console.log(key);
  }
  for await (const value of Object.freeze(Object.values)(object)) {
    console.log(value);
  }
  for await (const entry of Object.freeze(globalThis["Object"]["entries"])(object)) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
}
"##
}

fn assert_browser_requested_frozen_set_map_constructor_result(
    command: &str,
    filename: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        browser_frozen_set_map_constructor_result_test_source()
    } else {
        browser_frozen_set_map_constructor_result_run_source()
    };
    fs::write(&source_path, source).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.env(BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path());
    if json_output {
        cli.arg("--output").arg("json");
    }
    let output = cli
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg("--max-threads")
        .arg("0")
        .arg("--max-spawned-processes")
        .arg("0")
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
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        assert_eq!(json["payload"]["threadTopology"]["totalInstances"], 0);
        assert_eq!(json["payload"]["threadTopology"]["terminatedInstances"], 0);
        assert_eq!(
            json["payload"]["threadTopology"]["liveInstances"],
            serde_json::json!([])
        );
        if command == "run" {
            assert_eq!(json["exitCode"], 0);
            assert_eq!(json["payload"]["exitCode"], 0);
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
        }
        let stdout = json["stdout"].as_str().expect("stdout string");
        assert!(stdout.contains("1\n2\n1\n3\n4\n5\n"), "json: {json}");
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1\n2\n1\n3\n4\n5\n"), "stdout: {stdout}");
    if command == "test" {
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

fn assert_browser_bundle_frozen_set_map_constructor_result(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_frozen_set_map_constructor_result_bundle_source(),
    )
    .expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser");
    if json_output {
        cli.arg("--output").arg("json");
    }
    let output = cli.arg(&source_path).output().expect("run kali");

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
    let harness = browser_bundle_harness_script(
        "app",
        false,
        "const mod = await import(bundleJs.href);\nawait mod.browserFrozenSetMapConstructorResult();\nconsole.log('browser for await frozen set/map constructor results ok');\n",
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = browser_harness_command_parts_for(
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
    assert!(stdout.contains("1\n2\n1\n3\n4\n5\n"), "stdout: {stdout}");
}

fn assert_browser_requested_parenthesized_frozen_set_map_constructor_result(
    command: &str,
    filename: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        browser_parenthesized_frozen_set_map_constructor_result_test_source()
    } else {
        browser_parenthesized_frozen_set_map_constructor_result_run_source()
    };
    fs::write(&source_path, source).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.env(BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path());
    if json_output {
        cli.arg("--output").arg("json");
    }
    let output = cli
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg("--max-threads")
        .arg("0")
        .arg("--max-spawned-processes")
        .arg("0")
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
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        assert_eq!(json["payload"]["threadTopology"]["totalInstances"], 0);
        assert_eq!(json["payload"]["threadTopology"]["terminatedInstances"], 0);
        assert_eq!(
            json["payload"]["threadTopology"]["liveInstances"],
            serde_json::json!([])
        );
        if command == "run" {
            assert_eq!(json["exitCode"], 0);
            assert_eq!(json["payload"]["exitCode"], 0);
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
        }
        let stdout = json["stdout"].as_str().expect("stdout string");
        assert!(stdout.contains("1\n2\n1\n3\n4\n5\n"), "json: {json}");
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1\n2\n1\n3\n4\n5\n"), "stdout: {stdout}");
    if command == "test" {
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

fn assert_browser_bundle_parenthesized_frozen_set_map_constructor_result(
    filename: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_parenthesized_frozen_set_map_constructor_result_bundle_source(),
    )
    .expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser");
    if json_output {
        cli.arg("--output").arg("json");
    }
    let output = cli.arg(&source_path).output().expect("run kali");

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
    let harness = browser_bundle_harness_script(
        "app",
        false,
        "const mod = await import(bundleJs.href);\nawait mod.browserParenthesizedFrozenSetMapConstructorResult();\nconsole.log('browser for await parenthesized frozen set/map constructor results ok');\n",
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = browser_harness_command_parts_for(
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
    assert!(stdout.contains("1\n2\n1\n3\n4\n5\n"), "stdout: {stdout}");
}

fn assert_browser_requested_frozen_object_helper_iteration_targets(
    command: &str,
    filename: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        browser_frozen_object_helper_iteration_test_source()
    } else {
        browser_frozen_object_helper_iteration_run_source()
    };
    fs::write(&source_path, source).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.env(BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path());
    if json_output {
        cli.arg("--output").arg("json");
    }
    let output = cli
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg("--max-threads")
        .arg("0")
        .arg("--max-spawned-processes")
        .arg("0")
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
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        assert_eq!(json["payload"]["threadTopology"]["totalInstances"], 0);
        assert_eq!(json["payload"]["threadTopology"]["terminatedInstances"], 0);
        assert_eq!(
            json["payload"]["threadTopology"]["liveInstances"],
            serde_json::json!([])
        );
        if command == "run" {
            assert_eq!(json["exitCode"], 0);
            assert_eq!(json["payload"]["exitCode"], 0);
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
        }
        let stdout = json["stdout"].as_str().expect("stdout string");
        assert!(stdout.contains("b\na\n1\n2\nb\n1\na\n2\n"), "json: {json}");
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("b\na\n1\n2\nb\n1\na\n2\n"),
        "stdout: {stdout}"
    );
    if command == "test" {
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

fn assert_browser_bundle_frozen_object_helper_iteration_targets(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_frozen_object_helper_iteration_bundle_source(),
    )
    .expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser");
    if json_output {
        cli.arg("--output").arg("json");
    }
    let output = cli.arg(&source_path).output().expect("run kali");

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
    let harness = browser_bundle_harness_script(
        "app",
        false,
        "const mod = await import(bundleJs.href);\nawait mod.browserFrozenObjectHelperIterationTargets();\nconsole.log('browser for await frozen object helper iteration targets ok');\n",
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = browser_harness_command_parts_for(
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
        stdout.contains("b\na\n1\n2\nb\n1\na\n2\n"),
        "stdout: {stdout}"
    );
}

#[test]
fn run_supports_for_await_frozen_set_map_constructor_result_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_set_map_constructor_result("run", "main.js", false);
}

#[test]
fn run_supports_for_await_frozen_set_map_constructor_result_in_ts_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_set_map_constructor_result("run", "main.ts", false);
}

#[test]
fn run_supports_for_await_frozen_set_map_constructor_result_in_jsx_and_tsx_input_when_browser_harness_is_configured(
) {
    for filename in ["main.jsx", "main.tsx"] {
        assert_browser_requested_frozen_set_map_constructor_result("run", filename, false);
    }
}

#[test]
fn test_supports_for_await_frozen_set_map_constructor_result_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_set_map_constructor_result("test", "smoke.test.js", false);
}

#[test]
fn test_supports_for_await_frozen_set_map_constructor_result_in_ts_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_set_map_constructor_result("test", "smoke.test.ts", false);
}

#[test]
fn test_supports_for_await_frozen_set_map_constructor_result_in_jsx_and_tsx_input_when_browser_harness_is_configured(
) {
    for filename in ["smoke.test.jsx", "smoke.test.tsx"] {
        assert_browser_requested_frozen_set_map_constructor_result("test", filename, false);
    }
}

#[test]
fn json_run_supports_for_await_frozen_set_map_constructor_result_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_set_map_constructor_result("run", "main.js", true);
}

#[test]
fn json_run_supports_for_await_frozen_set_map_constructor_result_in_ts_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_set_map_constructor_result("run", "main.ts", true);
}

#[test]
fn json_run_supports_for_await_frozen_set_map_constructor_result_in_jsx_and_tsx_input_when_browser_harness_is_configured(
) {
    for filename in ["main.jsx", "main.tsx"] {
        assert_browser_requested_frozen_set_map_constructor_result("run", filename, true);
    }
}

#[test]
fn json_test_supports_for_await_frozen_set_map_constructor_result_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_set_map_constructor_result("test", "smoke.test.js", true);
}

#[test]
fn json_test_supports_for_await_frozen_set_map_constructor_result_in_ts_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_set_map_constructor_result("test", "smoke.test.ts", true);
}

#[test]
fn json_test_supports_for_await_frozen_set_map_constructor_result_in_jsx_and_tsx_input_when_browser_harness_is_configured(
) {
    for filename in ["smoke.test.jsx", "smoke.test.tsx"] {
        assert_browser_requested_frozen_set_map_constructor_result("test", filename, true);
    }
}

#[test]
fn build_emits_for_await_frozen_set_map_constructor_result_in_js_input_when_browser_bundle_smoke_is_configured(
) {
    assert_browser_bundle_frozen_set_map_constructor_result("app.js", false);
}

#[test]
fn json_build_emits_for_await_frozen_set_map_constructor_result_in_js_input_when_browser_bundle_smoke_is_configured(
) {
    assert_browser_bundle_frozen_set_map_constructor_result("app.js", true);
}

#[test]
fn build_emits_for_await_frozen_set_map_constructor_result_in_ts_input_when_browser_bundle_smoke_is_configured(
) {
    assert_browser_bundle_frozen_set_map_constructor_result("app.ts", false);
}

#[test]
fn json_build_emits_for_await_frozen_set_map_constructor_result_in_ts_input_when_browser_bundle_smoke_is_configured(
) {
    assert_browser_bundle_frozen_set_map_constructor_result("app.ts", true);
}

#[test]
fn build_emits_for_await_frozen_set_map_constructor_result_in_jsx_and_tsx_input_when_browser_bundle_smoke_is_configured(
) {
    for filename in ["app.jsx", "app.tsx"] {
        assert_browser_bundle_frozen_set_map_constructor_result(filename, false);
        assert_browser_bundle_frozen_set_map_constructor_result(filename, true);
    }
}

#[test]
fn run_supports_for_await_frozen_object_helper_iteration_targets_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_object_helper_iteration_targets("run", "main.js", false);
}

#[test]
fn run_supports_for_await_frozen_object_helper_iteration_targets_in_ts_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_object_helper_iteration_targets("run", "main.ts", false);
}

#[test]
fn run_supports_for_await_frozen_object_helper_iteration_targets_in_jsx_and_tsx_input_when_browser_harness_is_configured(
) {
    for filename in ["main.jsx", "main.tsx"] {
        assert_browser_requested_frozen_object_helper_iteration_targets("run", filename, false);
    }
}

#[test]
fn test_supports_for_await_frozen_object_helper_iteration_targets_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_object_helper_iteration_targets("test", "smoke.test.js", false);
}

#[test]
fn test_supports_for_await_frozen_object_helper_iteration_targets_in_ts_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_object_helper_iteration_targets("test", "smoke.test.ts", false);
}

#[test]
fn test_supports_for_await_frozen_object_helper_iteration_targets_in_jsx_and_tsx_input_when_browser_harness_is_configured(
) {
    for filename in ["smoke.test.jsx", "smoke.test.tsx"] {
        assert_browser_requested_frozen_object_helper_iteration_targets("test", filename, false);
    }
}

#[test]
fn json_run_supports_for_await_frozen_object_helper_iteration_targets_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_object_helper_iteration_targets("run", "main.js", true);
}

#[test]
fn json_run_supports_for_await_frozen_object_helper_iteration_targets_in_ts_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_object_helper_iteration_targets("run", "main.ts", true);
}

#[test]
fn json_run_supports_for_await_frozen_object_helper_iteration_targets_in_jsx_and_tsx_input_when_browser_harness_is_configured(
) {
    for filename in ["main.jsx", "main.tsx"] {
        assert_browser_requested_frozen_object_helper_iteration_targets("run", filename, true);
    }
}

#[test]
fn json_test_supports_for_await_frozen_object_helper_iteration_targets_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_object_helper_iteration_targets("test", "smoke.test.js", true);
}

#[test]
fn json_test_supports_for_await_frozen_object_helper_iteration_targets_in_ts_input_when_browser_harness_is_configured(
) {
    assert_browser_requested_frozen_object_helper_iteration_targets("test", "smoke.test.ts", true);
}

#[test]
fn json_test_supports_for_await_frozen_object_helper_iteration_targets_in_jsx_and_tsx_input_when_browser_harness_is_configured(
) {
    for filename in ["smoke.test.jsx", "smoke.test.tsx"] {
        assert_browser_requested_frozen_object_helper_iteration_targets("test", filename, true);
    }
}

#[test]
fn build_emits_for_await_frozen_object_helper_iteration_targets_in_js_input_when_browser_bundle_smoke_is_configured(
) {
    assert_browser_bundle_frozen_object_helper_iteration_targets("app.js", false);
}

#[test]
fn json_build_emits_for_await_frozen_object_helper_iteration_targets_in_js_input_when_browser_bundle_smoke_is_configured(
) {
    assert_browser_bundle_frozen_object_helper_iteration_targets("app.js", true);
}

#[test]
fn build_emits_for_await_frozen_object_helper_iteration_targets_in_ts_input_when_browser_bundle_smoke_is_configured(
) {
    assert_browser_bundle_frozen_object_helper_iteration_targets("app.ts", false);
}

#[test]
fn json_build_emits_for_await_frozen_object_helper_iteration_targets_in_ts_input_when_browser_bundle_smoke_is_configured(
) {
    assert_browser_bundle_frozen_object_helper_iteration_targets("app.ts", true);
}

#[test]
fn build_emits_for_await_frozen_object_helper_iteration_targets_in_jsx_and_tsx_input_when_browser_bundle_smoke_is_configured(
) {
    for filename in ["app.jsx", "app.tsx"] {
        assert_browser_bundle_frozen_object_helper_iteration_targets(filename, false);
        assert_browser_bundle_frozen_object_helper_iteration_targets(filename, true);
    }
}

#[test]
fn run_supports_for_await_parenthesized_frozen_set_map_constructor_result_in_all_browser_harness_input_variants_when_configured(
) {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_browser_requested_parenthesized_frozen_set_map_constructor_result(
            "run", filename, false,
        );
    }
}

#[test]
fn test_supports_for_await_parenthesized_frozen_set_map_constructor_result_in_all_browser_harness_input_variants_when_configured(
) {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_browser_requested_parenthesized_frozen_set_map_constructor_result(
            "test", filename, false,
        );
    }
}

#[test]
fn json_run_supports_for_await_parenthesized_frozen_set_map_constructor_result_in_all_browser_harness_input_variants_when_configured(
) {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_browser_requested_parenthesized_frozen_set_map_constructor_result(
            "run", filename, true,
        );
    }
}

#[test]
fn json_test_supports_for_await_parenthesized_frozen_set_map_constructor_result_in_all_browser_harness_input_variants_when_configured(
) {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_browser_requested_parenthesized_frozen_set_map_constructor_result(
            "test", filename, true,
        );
    }
}

#[test]
fn build_emits_for_await_parenthesized_frozen_set_map_constructor_result_in_all_browser_bundle_input_variants_when_configured(
) {
    for filename in ["app.js", "app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle_parenthesized_frozen_set_map_constructor_result(filename, false);
    }
}

#[test]
fn json_build_emits_for_await_parenthesized_frozen_set_map_constructor_result_in_all_browser_bundle_input_variants_when_configured(
) {
    for filename in ["app.js", "app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle_parenthesized_frozen_set_map_constructor_result(filename, true);
    }
}
