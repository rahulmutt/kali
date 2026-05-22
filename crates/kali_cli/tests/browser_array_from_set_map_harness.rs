use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_array_from_set_map_run_source() -> &'static str {
    r##"async function browserArrayFromSetMapWrappers() {
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

browserArrayFromSetMapWrappers();
"##
}

fn browser_harness_array_from_set_map_test_source() -> &'static str {
    r##"Kali.test('array.from set/map wrappers', () => {
  async function browserArrayFromSetMapWrappers() {
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
    for (const value of Object.freeze((globalThis["Array"]))["from"](new Set(setValues))) {
      console.log(value);
    }
    for (const value of Object.freeze((globalThis['Array']))["from"](new Set(setValues))) {
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

  return browserArrayFromSetMapWrappers();
});
"##
}

fn assert_browser_harness_array_from_set_map(command: &str, filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        browser_harness_array_from_set_map_test_source()
    } else {
        browser_harness_array_from_set_map_run_source()
    };
    assert!(source.contains("Object.freeze((globalThis[\"Array\"]))[\"from\"]"));
    assert!(source.contains("Object.freeze((globalThis['Array']))[\"from\"]"));
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
            stdout.contains("1\n2\n1\n2\n1\n2\n1\n2\n1\n3\n4\n5\n1\n3\n4\n5\n"),
            "json: {json}"
        );
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("1\n2\n1\n2\n1\n2\n1\n2\n1\n3\n4\n5\n1\n3\n4\n5\n"),
            "stdout: {stdout}"
        );
        if command == "test" {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn run_supports_array_from_new_set_and_new_map_in_js_input() {
    assert_browser_harness_array_from_set_map("run", "main.js", false);
}

#[test]
fn run_supports_array_from_new_set_and_new_map_in_ts_input() {
    assert_browser_harness_array_from_set_map("run", "main.ts", false);
}

#[test]
fn run_supports_array_from_new_set_and_new_map_in_jsx_and_tsx_input() {
    for filename in ["main.jsx", "main.tsx"] {
        assert_browser_harness_array_from_set_map("run", filename, false);
    }
}

#[test]
fn json_run_supports_array_from_new_set_and_new_map_in_js_ts_jsx_and_tsx_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_browser_harness_array_from_set_map("run", filename, true);
    }
}

#[test]
fn test_supports_array_from_new_set_and_new_map_in_js_input() {
    assert_browser_harness_array_from_set_map("test", "smoke.test.js", false);
}

#[test]
fn test_supports_array_from_new_set_and_new_map_in_ts_input() {
    assert_browser_harness_array_from_set_map("test", "smoke.test.ts", false);
}

#[test]
fn test_supports_array_from_new_set_and_new_map_in_jsx_and_tsx_input() {
    for filename in ["smoke.test.jsx", "smoke.test.tsx"] {
        assert_browser_harness_array_from_set_map("test", filename, false);
    }
}

#[test]
fn json_test_supports_array_from_new_set_and_new_map_in_js_ts_jsx_and_tsx_input() {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_browser_harness_array_from_set_map("test", filename, true);
    }
}
