use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

use kali_runtime_contract::{
    browser_bundle_harness_script, browser_harness_command_parts_for, BROWSER_HARNESS_COMMAND_ENV,
};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn standalone_source() -> &'static str {
    r##"const values = [1, 2, 1]; const mapValues = [[1, 2], [1, 3], [4, 5]]; const nullishBracketRootArrayFrom = Object.freeze((null ?? globalThis["Array"].from)); const andBracketRootArrayFrom = Object.freeze((true && globalThis["Array"].from)); const orBracketRootArrayFrom = Object.freeze((false || globalThis["Array"].from)); const nullishSingleQuotedBracketRootArrayFrom = Object.freeze((null ?? globalThis['Array'].from)); const andSingleQuotedBracketRootArrayFrom = Object.freeze((true && globalThis['Array'].from)); const orSingleQuotedBracketRootArrayFrom = Object.freeze((false || globalThis['Array'].from)); const nullishFullyBracketedArrayFrom = Object.freeze((null ?? globalThis["Array"]["from"])); const andFullyBracketedArrayFrom = Object.freeze((true && globalThis["Array"]["from"])); const orFullyBracketedArrayFrom = Object.freeze((false || globalThis["Array"]["from"])); const nullishSingleQuotedFullyBracketedArrayFrom = Object.freeze((null ?? globalThis['Array']['from'])); const andSingleQuotedFullyBracketedArrayFrom = Object.freeze((true && globalThis['Array']['from'])); const orSingleQuotedFullyBracketedArrayFrom = Object.freeze((false || globalThis['Array']['from'])); for (const value of nullishBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const value of andBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const value of orBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const value of nullishSingleQuotedBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const value of andSingleQuotedBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const value of orSingleQuotedBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const entry of nullishBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of andBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of orBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of nullishSingleQuotedBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of andSingleQuotedBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of orSingleQuotedBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); }"##
}

fn test_source() -> &'static str {
    r##"Kali.test('nullish/logical bracketed globalThis.Array.from wrappers', () => { const values = [1, 2, 1]; const mapValues = [[1, 2], [1, 3], [4, 5]]; const nullishBracketRootArrayFrom = Object.freeze((null ?? globalThis["Array"].from)); const andBracketRootArrayFrom = Object.freeze((true && globalThis["Array"].from)); const orBracketRootArrayFrom = Object.freeze((false || globalThis["Array"].from)); const nullishSingleQuotedBracketRootArrayFrom = Object.freeze((null ?? globalThis['Array'].from)); const andSingleQuotedBracketRootArrayFrom = Object.freeze((true && globalThis['Array'].from)); const orSingleQuotedBracketRootArrayFrom = Object.freeze((false || globalThis['Array'].from)); const nullishFullyBracketedArrayFrom = Object.freeze((null ?? globalThis["Array"]["from"])); const andFullyBracketedArrayFrom = Object.freeze((true && globalThis["Array"]["from"])); const orFullyBracketedArrayFrom = Object.freeze((false || globalThis["Array"]["from"])); const nullishSingleQuotedFullyBracketedArrayFrom = Object.freeze((null ?? globalThis['Array']['from'])); const andSingleQuotedFullyBracketedArrayFrom = Object.freeze((true && globalThis['Array']['from'])); const orSingleQuotedFullyBracketedArrayFrom = Object.freeze((false || globalThis['Array']['from'])); for (const value of nullishBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const value of andBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const value of orBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const value of nullishSingleQuotedBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const value of andSingleQuotedBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const value of orSingleQuotedBracketRootArrayFrom(new Set(values))) { console.log(value); } for (const entry of nullishBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of andBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of orBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of nullishSingleQuotedBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of andSingleQuotedBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } for (const entry of orSingleQuotedBracketRootArrayFrom(new Map(mapValues))) { console.log(entry[0]); console.log(entry[1]); } });"##
}

fn browser_run_source() -> &'static str {
    r##"async function browserArrayFromBracketedRootWrappers() {
  const values = [1, 2, 1];
  const mapValues = [[1, 2], [1, 3], [4, 5]];
  const nullishBracketRootArrayFrom = Object.freeze((null ?? globalThis["Array"].from));
  const andBracketRootArrayFrom = Object.freeze((true && globalThis["Array"].from));
  const orBracketRootArrayFrom = Object.freeze((false || globalThis["Array"].from));
  const nullishSingleQuotedBracketRootArrayFrom = Object.freeze((null ?? globalThis['Array'].from));
  const andSingleQuotedBracketRootArrayFrom = Object.freeze((true && globalThis['Array'].from));
  const orSingleQuotedBracketRootArrayFrom = Object.freeze((false || globalThis['Array'].from)); const nullishFullyBracketedArrayFrom = Object.freeze((null ?? globalThis["Array"]["from"])); const andFullyBracketedArrayFrom = Object.freeze((true && globalThis["Array"]["from"])); const orFullyBracketedArrayFrom = Object.freeze((false || globalThis["Array"]["from"])); const nullishSingleQuotedFullyBracketedArrayFrom = Object.freeze((null ?? globalThis['Array']['from'])); const andSingleQuotedFullyBracketedArrayFrom = Object.freeze((true && globalThis['Array']['from'])); const orSingleQuotedFullyBracketedArrayFrom = Object.freeze((false || globalThis['Array']['from']));
  for (const value of nullishBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const value of andBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const value of orBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const value of nullishSingleQuotedBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const value of andSingleQuotedBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const value of orSingleQuotedBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const entry of nullishBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of andBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of orBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of nullishSingleQuotedBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of andSingleQuotedBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of orSingleQuotedBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
}

browserArrayFromBracketedRootWrappers();
"##
}

fn browser_test_source() -> &'static str {
    r##"Kali.test('nullish/logical bracketed globalThis.Array.from wrappers', () => {
  async function browserArrayFromBracketedRootWrappers() {
    const values = [1, 2, 1];
    const mapValues = [[1, 2], [1, 3], [4, 5]];
    const nullishBracketRootArrayFrom = Object.freeze((null ?? globalThis["Array"].from));
    const andBracketRootArrayFrom = Object.freeze((true && globalThis["Array"].from));
    const orBracketRootArrayFrom = Object.freeze((false || globalThis["Array"].from));
    const nullishSingleQuotedBracketRootArrayFrom = Object.freeze((null ?? globalThis['Array'].from));
    const andSingleQuotedBracketRootArrayFrom = Object.freeze((true && globalThis['Array'].from));
    const orSingleQuotedBracketRootArrayFrom = Object.freeze((false || globalThis['Array'].from)); const nullishFullyBracketedArrayFrom = Object.freeze((null ?? globalThis["Array"]["from"])); const andFullyBracketedArrayFrom = Object.freeze((true && globalThis["Array"]["from"])); const orFullyBracketedArrayFrom = Object.freeze((false || globalThis["Array"]["from"])); const nullishSingleQuotedFullyBracketedArrayFrom = Object.freeze((null ?? globalThis['Array']['from'])); const andSingleQuotedFullyBracketedArrayFrom = Object.freeze((true && globalThis['Array']['from'])); const orSingleQuotedFullyBracketedArrayFrom = Object.freeze((false || globalThis['Array']['from']));
    for (const value of nullishBracketRootArrayFrom(new Set(values))) {
      console.log(value);
    }
    for (const value of andBracketRootArrayFrom(new Set(values))) {
      console.log(value);
    }
    for (const value of orBracketRootArrayFrom(new Set(values))) {
      console.log(value);
    }
    for (const value of nullishSingleQuotedBracketRootArrayFrom(new Set(values))) {
      console.log(value);
    }
    for (const value of andSingleQuotedBracketRootArrayFrom(new Set(values))) {
      console.log(value);
    }
    for (const value of orSingleQuotedBracketRootArrayFrom(new Set(values))) {
      console.log(value);
    }
    for (const entry of nullishBracketRootArrayFrom(new Map(mapValues))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
    for (const entry of andBracketRootArrayFrom(new Map(mapValues))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
    for (const entry of orBracketRootArrayFrom(new Map(mapValues))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
    for (const entry of nullishSingleQuotedBracketRootArrayFrom(new Map(mapValues))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
    for (const entry of andSingleQuotedBracketRootArrayFrom(new Map(mapValues))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
    for (const entry of orSingleQuotedBracketRootArrayFrom(new Map(mapValues))) {
      console.log(entry[0]);
      console.log(entry[1]);
    }
  }

  return browserArrayFromBracketedRootWrappers();
});
"##
}

fn browser_bundle_source() -> &'static str {
    r##"// kali-tree-shake: browserArrayFromBracketedRootWrappers
export async function browserArrayFromBracketedRootWrappers() {
  const values = [1, 2, 1];
  const mapValues = [[1, 2], [1, 3], [4, 5]];
  const nullishBracketRootArrayFrom = Object.freeze((null ?? globalThis["Array"].from));
  const andBracketRootArrayFrom = Object.freeze((true && globalThis["Array"].from));
  const orBracketRootArrayFrom = Object.freeze((false || globalThis["Array"].from));
  const nullishSingleQuotedBracketRootArrayFrom = Object.freeze((null ?? globalThis['Array'].from));
  const andSingleQuotedBracketRootArrayFrom = Object.freeze((true && globalThis['Array'].from));
  const orSingleQuotedBracketRootArrayFrom = Object.freeze((false || globalThis['Array'].from)); const nullishFullyBracketedArrayFrom = Object.freeze((null ?? globalThis["Array"]["from"])); const andFullyBracketedArrayFrom = Object.freeze((true && globalThis["Array"]["from"])); const orFullyBracketedArrayFrom = Object.freeze((false || globalThis["Array"]["from"])); const nullishSingleQuotedFullyBracketedArrayFrom = Object.freeze((null ?? globalThis['Array']['from'])); const andSingleQuotedFullyBracketedArrayFrom = Object.freeze((true && globalThis['Array']['from'])); const orSingleQuotedFullyBracketedArrayFrom = Object.freeze((false || globalThis['Array']['from']));
  for (const value of nullishBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const value of andBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const value of orBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const value of nullishSingleQuotedBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const value of andSingleQuotedBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const value of orSingleQuotedBracketRootArrayFrom(new Set(values))) {
    console.log(value);
  }
  for (const entry of nullishBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of andBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of orBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of nullishSingleQuotedBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of andSingleQuotedBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const entry of orSingleQuotedBracketRootArrayFrom(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
}
"##
}

fn assert_standalone(command: &str, filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        test_source()
    } else {
        standalone_source()
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
    assert!(stdout.contains("1\n2\n1\n3\n4\n5\n"), "stdout: {stdout}");
    if command == "test" {
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
}

fn assert_browser_requested(command: &str, filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        browser_test_source()
    } else {
        browser_run_source()
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
}

fn assert_browser_bundle(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, browser_bundle_source()).expect("write source");

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
        "const mod = await import(bundleJs.href);\nawait mod.browserArrayFromBracketedRootWrappers();\nconsole.log('browser bracketed Array.from wrapper aliases ok');\n",
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = browser_harness_command_parts_for(
        std::env::var(BROWSER_HARNESS_COMMAND_ENV).ok().as_deref(),
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

#[test]
fn run_supports_nullish_and_logical_bracketed_global_this_array_from_wrappers_in_js_input() {
    assert_standalone("run", "main.js");
}

#[test]
fn test_supports_nullish_and_logical_bracketed_global_this_array_from_wrappers_in_js_input() {
    assert_standalone("test", "smoke.test.js");
}

#[test]
fn run_supports_nullish_and_logical_bracketed_global_this_array_from_wrappers_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested("run", "main.js", false);
}

#[test]
fn run_supports_nullish_and_logical_bracketed_global_this_array_from_wrappers_in_ts_jsx_and_tsx_input_when_browser_harness_is_configured(
) {
    for filename in ["main.ts", "main.jsx", "main.tsx"] {
        assert_browser_requested("run", filename, false);
    }
}

#[test]
fn test_supports_nullish_and_logical_bracketed_global_this_array_from_wrappers_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_requested("test", "smoke.test.js", false);
}

#[test]
fn test_supports_nullish_and_logical_bracketed_global_this_array_from_wrappers_in_ts_jsx_and_tsx_input_when_browser_harness_is_configured(
) {
    for filename in ["smoke.test.ts", "smoke.test.jsx", "smoke.test.tsx"] {
        assert_browser_requested("test", filename, false);
    }
}

#[test]
fn build_bundles_nullish_and_logical_bracketed_global_this_array_from_wrappers_in_js_input() {
    assert_browser_bundle("app.js", false);
}

#[test]
fn build_bundles_nullish_and_logical_bracketed_global_this_array_from_wrappers_in_ts_jsx_and_tsx_input(
) {
    for filename in ["app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle(filename, false);
    }
}
