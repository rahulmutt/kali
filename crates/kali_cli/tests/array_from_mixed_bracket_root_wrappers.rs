use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

use kali_runtime::{
    browser_bundle_harness_script, browser_harness_command_parts_for, BROWSER_HARNESS_COMMAND_ENV,
};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn mixed_bracket_array_from_source() -> &'static str {
    r##"const values = [1, 2, 1]; const mapValues = [[1, 2], [1, 3], [4, 5]]; const mixedBracketArrayFrom = Object.freeze(globalThis["Array"]['from']); const mixedRootArrayFrom = Object.freeze(globalThis['Array']["from"]); for (const value of mixedBracketArrayFrom(new Set(values))) { console.log(value); } for (const value of mixedRootArrayFrom(new Set(values))) { console.log(value); } for (const entry of mixedBracketArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of mixedRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); }"##
}

fn mixed_bracket_array_from_test_source() -> &'static str {
    r##"Kali.test('mixed-quote bracketed Array.from aliases', () => { const values = [1, 2, 1]; const mapValues = [[1, 2], [1, 3], [4, 5]]; const mixedBracketArrayFrom = Object.freeze(globalThis["Array"]['from']); const mixedRootArrayFrom = Object.freeze(globalThis['Array']["from"]); for (const value of mixedBracketArrayFrom(new Set(values))) { console.log(value); } for (const value of mixedRootArrayFrom(new Set(values))) { console.log(value); } for (const entry of mixedBracketArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of mixedRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } });"##
}

fn browser_requested_mixed_bracket_array_from_run_source() -> &'static str {
    r##"async function mixedBracketArrayFromWrappers() {
  const values = [1, 2, 1];
  const mapValues = [[1, 2], [1, 3], [4, 5]];
  const mixedBracketArrayFrom = Object.freeze(globalThis["Array"]['from']);
  const mixedRootArrayFrom = Object.freeze(globalThis['Array']["from"]);
  for (const value of mixedBracketArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const value of mixedRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const entry of mixedBracketArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of mixedRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
}

mixedBracketArrayFromWrappers();
"##
}

fn browser_requested_mixed_bracket_array_from_test_source() -> &'static str {
    r##"Kali.test('mixed-quote bracketed Array.from aliases', () => {
  async function mixedBracketArrayFromWrappers() {
    const values = [1, 2, 1];
    const mapValues = [[1, 2], [1, 3], [4, 5]];
    const mixedBracketArrayFrom = Object.freeze(globalThis["Array"]['from']);
    const mixedRootArrayFrom = Object.freeze(globalThis['Array']["from"]);
    for (const value of mixedBracketArrayFrom(new Set(values))) {
      console.log(value);
    }
    for (const value of mixedRootArrayFrom(new Set(values))) {
      console.log(value);
    }
    for (const entry of mixedBracketArrayFrom(new Map(mapValues))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
    for (const entry of mixedRootArrayFrom(new Map(mapValues))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
  }

  return mixedBracketArrayFromWrappers();
});
"##
}

fn browser_bundle_mixed_bracket_array_from_source() -> &'static str {
    r##"// kali-tree-shake: mixedBracketArrayFromWrappers
export async function mixedBracketArrayFromWrappers() {
  const values = [1, 2, 1];
  const mapValues = [[1, 2], [1, 3], [4, 5]];
  const mixedBracketArrayFrom = Object.freeze(globalThis["Array"]['from']);
  const mixedRootArrayFrom = Object.freeze(globalThis['Array']["from"]);
  for (const value of mixedBracketArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const value of mixedRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const entry of mixedBracketArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of mixedRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
}
"##
}

fn assert_standalone_mixed_bracket_array_from(command: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        mixed_bracket_array_from_test_source()
    } else {
        mixed_bracket_array_from_source()
    };
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
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

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("1\n2\n1\n2\n1\n3\n4\n5\n1\n3\n4\n5\n"),
        "stdout: {stdout}"
    );
    if command == "test" {
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
}

fn assert_browser_requested_mixed_bracket_array_from(
    command: &str,
    filename: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        browser_requested_mixed_bracket_array_from_test_source()
    } else {
        browser_requested_mixed_bracket_array_from_run_source()
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
        let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
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
        assert!(
            stdout.contains("1\n2\n1\n2\n1\n3\n4\n5\n1\n3\n4\n5\n"),
            "json: {json}"
        );
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("1\n2\n1\n2\n1\n3\n4\n5\n1\n3\n4\n5\n"),
            "stdout: {stdout}"
        );
        if command == "test" {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

fn assert_browser_bundle_mixed_bracket_array_from(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = browser_bundle_mixed_bracket_array_from_source();
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
    let harness = browser_bundle_harness_script(
        "app",
        false,
        "const mod = await import(bundleJs.href);\nawait mod.mixedBracketArrayFromWrappers();\n",
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
        stdout.contains("1\n2\n1\n2\n1\n3\n4\n5\n1\n3\n4\n5\n"),
        "stdout: {stdout}"
    );
}

#[test]
fn run_supports_mixed_quote_bracketed_array_from_aliases_in_js_input() {
    assert_standalone_mixed_bracket_array_from("run", "main.js");
}

#[test]
fn test_supports_mixed_quote_bracketed_array_from_aliases_in_js_input() {
    assert_standalone_mixed_bracket_array_from("test", "smoke.test.js");
}

#[test]
fn run_supports_mixed_quote_bracketed_array_from_aliases_in_browser_harness_js_input_when_configured(
) {
    assert_browser_requested_mixed_bracket_array_from("run", "main.js", false);
}

#[test]
fn json_run_supports_mixed_quote_bracketed_array_from_aliases_in_browser_harness_js_input_when_configured(
) {
    assert_browser_requested_mixed_bracket_array_from("run", "main.js", true);
}

#[test]
fn test_supports_mixed_quote_bracketed_array_from_aliases_in_browser_harness_js_input_when_configured(
) {
    assert_browser_requested_mixed_bracket_array_from("test", "smoke.test.js", false);
}

#[test]
fn json_test_supports_mixed_quote_bracketed_array_from_aliases_in_browser_harness_js_input_when_configured(
) {
    assert_browser_requested_mixed_bracket_array_from("test", "smoke.test.js", true);
}

#[test]
fn build_emits_mixed_quote_bracketed_array_from_aliases_in_js_input() {
    assert_browser_bundle_mixed_bracket_array_from("app.js", false);
}

#[test]
fn json_build_emits_mixed_quote_bracketed_array_from_aliases_in_js_input() {
    assert_browser_bundle_mixed_bracket_array_from("app.js", true);
}

#[test]
fn build_emits_mixed_quote_bracketed_array_from_aliases_in_ts_input() {
    assert_browser_bundle_mixed_bracket_array_from("app.ts", false);
}

#[test]
fn json_build_emits_mixed_quote_bracketed_array_from_aliases_in_ts_input() {
    assert_browser_bundle_mixed_bracket_array_from("app.ts", true);
}

#[test]
fn build_emits_mixed_quote_bracketed_array_from_aliases_in_jsx_and_tsx_input() {
    for filename in ["app.jsx", "app.tsx"] {
        assert_browser_bundle_mixed_bracket_array_from(filename, false);
        assert_browser_bundle_mixed_bracket_array_from(filename, true);
    }
}
