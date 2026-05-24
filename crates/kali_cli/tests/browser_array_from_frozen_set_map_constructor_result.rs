use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_array_from_frozen_set_map_run_source() -> &'static str {
    r##"async function browserArrayFromFrozenSetMapWrappers() {
  const setValues = [1, 2, 1];
  const mapValues = [[1, 2], [1, 3], [4, 5]];
  for (const value of Array.from(Object.freeze(new Set(setValues)))) {
    console.log(value);
  }
  for (const value of Array.from(Object.freeze(new globalThis["Set"](setValues)))) {
    console.log(value);
  }
  for await (const value of Array.from(Object.freeze((new Set(setValues))))) {
    console.log(value);
  }
  for (const value of Array.from(Object.freeze((null ?? new Set(setValues))))) {
    console.log(value);
  }
  for await (const value of Array.from(Object.freeze((false || new Set(setValues))))) {
    console.log(value);
  }
  for (const value of Array.from(Object.freeze((new (true && Set)(setValues))))) {
    console.log(value);
  }
  for (const entry of Array.from(Object.freeze(new Map(mapValues)))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of Array.from(Object.freeze(new globalThis['Map'](mapValues)))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for await (const entry of Array.from(Object.freeze((new Map(mapValues))))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of Array.from(Object.freeze((null ?? new Map(mapValues))))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for await (const entry of Array.from(Object.freeze((false || new Map(mapValues))))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of Array.from(Object.freeze((new (true && Map)(mapValues))))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
}

browserArrayFromFrozenSetMapWrappers();
"##
}

fn browser_array_from_frozen_set_map_test_source() -> &'static str {
    r##"Kali.test('array.from frozen set/map constructor results', () => {
  async function browserArrayFromFrozenSetMapWrappers() {
    const setValues = [1, 2, 1];
    const mapValues = [[1, 2], [1, 3], [4, 5]];
    for (const value of Array.from(Object.freeze(new Set(setValues)))) {
      console.log(value);
    }
    for (const value of Array.from(Object.freeze(new globalThis["Set"](setValues)))) {
      console.log(value);
    }
    for await (const value of Array.from(Object.freeze((new Set(setValues))))) {
      console.log(value);
    }
    for (const value of Array.from(Object.freeze((null ?? new Set(setValues))))) {
      console.log(value);
    }
    for await (const value of Array.from(Object.freeze((false || new Set(setValues))))) {
      console.log(value);
    }
    for (const value of Array.from(Object.freeze((new (true && Set)(setValues))))) {
      console.log(value);
    }
    for (const entry of Array.from(Object.freeze(new Map(mapValues)))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
    for (const entry of Array.from(Object.freeze(new globalThis['Map'](mapValues)))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
    for await (const entry of Array.from(Object.freeze((new Map(mapValues))))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
    for (const entry of Array.from(Object.freeze((null ?? new Map(mapValues))))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
    for await (const entry of Array.from(Object.freeze((false || new Map(mapValues))))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
    for (const entry of Array.from(Object.freeze((new (true && Map)(mapValues))))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
  }

  return browserArrayFromFrozenSetMapWrappers();
});
"##
}

fn browser_array_from_frozen_set_map_bundle_source() -> &'static str {
    r##"// kali-tree-shake: browserArrayFromFrozenSetMapWrappers
export async function browserArrayFromFrozenSetMapWrappers() {
  const setValues = [1, 2, 1];
  const mapValues = [[1, 2], [1, 3], [4, 5]];
  for (const value of Array.from(Object.freeze(new Set(setValues)))) {
    console.log(value);
  }
  for (const value of Array.from(Object.freeze(new globalThis["Set"](setValues)))) {
    console.log(value);
  }
  for await (const value of Array.from(Object.freeze((new Set(setValues))))) {
    console.log(value);
  }
  for (const value of Array.from(Object.freeze((null ?? new Set(setValues))))) {
    console.log(value);
  }
  for await (const value of Array.from(Object.freeze((false || new Set(setValues))))) {
    console.log(value);
  }
  for (const value of Array.from(Object.freeze((new (true && Set)(setValues))))) {
    console.log(value);
  }
  for (const entry of Array.from(Object.freeze(new Map(mapValues)))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of Array.from(Object.freeze(new globalThis['Map'](mapValues)))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for await (const entry of Array.from(Object.freeze((new Map(mapValues))))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of Array.from(Object.freeze((null ?? new Map(mapValues))))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for await (const entry of Array.from(Object.freeze((false || new Map(mapValues))))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of Array.from(Object.freeze((new (true && Map)(mapValues))))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
}
"##
}

fn assert_browser_harness_array_from_frozen_set_map(
    command: &str,
    filename: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        browser_array_from_frozen_set_map_test_source()
    } else {
        browser_array_from_frozen_set_map_run_source()
    };
    fs::write(&source_path, source).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
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
        assert!(
            stdout.contains("1\n2\n1\n2\n1\n2\n1\n2\n1\n2\n1\n2\n1\n3\n4\n5\n1\n3\n4\n5\n1\n3\n4\n5\n1\n3\n4\n5\n1\n3\n4\n5\n1\n3\n4\n5\n"),
            "json: {json}"
        );
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("1\n2\n1\n2\n1\n2\n1\n2\n1\n2\n1\n2\n1\n3\n4\n5\n1\n3\n4\n5\n1\n3\n4\n5\n1\n3\n4\n5\n1\n3\n4\n5\n1\n3\n4\n5\n"),
        "stdout: {stdout}"
    );
    if command == "test" {
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

fn assert_browser_check_array_from_frozen_set_map(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, browser_array_from_frozen_set_map_run_source()).expect("write source");

    let mut command = Command::new(kali_bin());
    command
        .current_dir(dir.path())
        .arg("check")
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
        assert_eq!(envelope["command"], "check");
        assert_eq!(envelope["success"], true);
        assert_eq!(envelope["exitCode"], 0);
        let payload = envelope["payload"].as_object().expect("payload object");
        assert_eq!(payload["errorCount"], 0);
        assert_eq!(payload["filesChecked"], 1);
        assert_eq!(payload["warningCount"], 0);
        assert!(envelope["errors"]
            .as_array()
            .expect("errors array")
            .is_empty());
        assert!(envelope["warnings"]
            .as_array()
            .expect("warnings array")
            .is_empty());
        assert!(envelope["stdout"].is_null());
        assert!(envelope["stderr"].is_null());
    } else {
        assert_eq!(String::from_utf8_lossy(&output.stderr), "");
    }
}

fn assert_browser_bundle_array_from_frozen_set_map(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_array_from_frozen_set_map_bundle_source(),
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
        "const mod = await import(bundleJs.href);\nawait mod.browserArrayFromFrozenSetMapWrappers();\n",
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
        stdout.contains("1\n2\n1\n2\n1\n2\n1\n2\n1\n2\n1\n2\n1\n3\n4\n5\n1\n3\n4\n5\n1\n3\n4\n5\n1\n3\n4\n5\n1\n3\n4\n5\n1\n3\n4\n5\n"),
        "stdout: {stdout}"
    );
}

#[test]
fn check_supports_array_from_frozen_set_map_constructor_results_in_js_input() {
    assert_browser_check_array_from_frozen_set_map("main.js", false);
}

#[test]
fn check_supports_array_from_frozen_set_map_constructor_results_in_ts_jsx_and_tsx_input() {
    for filename in ["main.ts", "main.jsx", "main.tsx"] {
        assert_browser_check_array_from_frozen_set_map(filename, false);
    }
}

#[test]
fn json_check_supports_array_from_frozen_set_map_constructor_results_in_js_ts_jsx_and_tsx_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_browser_check_array_from_frozen_set_map(filename, true);
    }
}

#[test]
fn run_supports_array_from_frozen_set_map_constructor_results_in_js_input() {
    assert_browser_harness_array_from_frozen_set_map("run", "main.js", false);
}

#[test]
fn run_supports_array_from_frozen_set_map_constructor_results_in_ts_jsx_and_tsx_input() {
    for filename in ["main.ts", "main.jsx", "main.tsx"] {
        assert_browser_harness_array_from_frozen_set_map("run", filename, false);
    }
}

#[test]
fn json_run_supports_array_from_frozen_set_map_constructor_results_in_js_ts_jsx_and_tsx_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_browser_harness_array_from_frozen_set_map("run", filename, true);
    }
}

#[test]
fn test_supports_array_from_frozen_set_map_constructor_results_in_js_input() {
    assert_browser_harness_array_from_frozen_set_map("test", "smoke.test.js", false);
}

#[test]
fn test_supports_array_from_frozen_set_map_constructor_results_in_ts_jsx_and_tsx_input() {
    for filename in ["smoke.test.ts", "smoke.test.jsx", "smoke.test.tsx"] {
        assert_browser_harness_array_from_frozen_set_map("test", filename, false);
    }
}

#[test]
fn json_test_supports_array_from_frozen_set_map_constructor_results_in_js_ts_jsx_and_tsx_input() {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_browser_harness_array_from_frozen_set_map("test", filename, true);
    }
}

#[test]
fn build_emits_array_from_frozen_set_map_constructor_results_in_js_input() {
    assert_browser_bundle_array_from_frozen_set_map("app.js", false);
}

#[test]
fn json_build_emits_array_from_frozen_set_map_constructor_results_in_js_input() {
    assert_browser_bundle_array_from_frozen_set_map("app.js", true);
}

#[test]
fn build_emits_array_from_frozen_set_map_constructor_results_in_ts_input() {
    assert_browser_bundle_array_from_frozen_set_map("app.ts", false);
}

#[test]
fn json_build_emits_array_from_frozen_set_map_constructor_results_in_ts_input() {
    assert_browser_bundle_array_from_frozen_set_map("app.ts", true);
}

#[test]
fn build_emits_array_from_frozen_set_map_constructor_results_in_jsx_and_tsx_input() {
    for filename in ["app.jsx", "app.tsx"] {
        assert_browser_bundle_array_from_frozen_set_map(filename, false);
        assert_browser_bundle_array_from_frozen_set_map(filename, true);
    }
}
