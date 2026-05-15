use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_object_values_spread_run_source() -> &'static str {
    r##"function assertObjectValuesSpreadIteration(values) {
  if (values.length !== 2 || values[0] !== 3 || values[1] !== 2) {
    throw new Error('unexpected Object.values spread iteration semantics');
  }
}

function browserObjectValuesSpreadIteration() {
  const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);
  const frozenFromEntries = Object.freeze(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]));
  const collected = [...Object.values(fromEntries)];
  const globalCollected = [...globalThis.Object.values(fromEntries)];
  const mixedCollected = [...globalThis.Object["values"](fromEntries)];
  const mixedBracketedCollected = [...globalThis["Object"].values(fromEntries)];
  const bracketedCollected = [...globalThis["Object"]["values"](fromEntries)];
  const singleBracketedCollected = [...globalThis['Object']['values'](fromEntries)];
  const frozenCollected = [...globalThis["Object"]["values"](frozenFromEntries)];
  assertObjectValuesSpreadIteration(collected);
  assertObjectValuesSpreadIteration(globalCollected);
  assertObjectValuesSpreadIteration(mixedCollected);
  assertObjectValuesSpreadIteration(mixedBracketedCollected);
  assertObjectValuesSpreadIteration(bracketedCollected);
  assertObjectValuesSpreadIteration(singleBracketedCollected);
  assertObjectValuesSpreadIteration(frozenCollected);
  console.log('browser object values spread iteration ok');
}

browserObjectValuesSpreadIteration();
"##
}

fn browser_harness_object_values_spread_test_source() -> &'static str {
    r##"Kali.test('object values spread iteration', () => {
  function assertObjectValuesSpreadIteration(values) {
    if (values.length !== 2 || values[0] !== 3 || values[1] !== 2) {
      throw new Error('unexpected Object.values spread iteration semantics');
    }
  }

  function browserObjectValuesSpreadIteration() {
    const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);
    const frozenFromEntries = Object.freeze(Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]));
    const collected = [...Object.values(fromEntries)];
    const globalCollected = [...globalThis.Object.values(fromEntries)];
    const mixedCollected = [...globalThis.Object["values"](fromEntries)];
    const mixedBracketedCollected = [...globalThis["Object"].values(fromEntries)];
    const bracketedCollected = [...globalThis["Object"]["values"](fromEntries)];
    const singleBracketedCollected = [...globalThis['Object']['values'](fromEntries)];
    const frozenCollected = [...globalThis["Object"]["values"](frozenFromEntries)];
    assertObjectValuesSpreadIteration(collected);
    assertObjectValuesSpreadIteration(globalCollected);
    assertObjectValuesSpreadIteration(mixedCollected);
    assertObjectValuesSpreadIteration(mixedBracketedCollected);
    assertObjectValuesSpreadIteration(bracketedCollected);
    assertObjectValuesSpreadIteration(singleBracketedCollected);
    assertObjectValuesSpreadIteration(frozenCollected);
    console.log('browser object values spread iteration ok');
  }

  browserObjectValuesSpreadIteration();
});
"##
}

fn assert_browser_harness_object_values_spread(
    command: &str,
    filename: &str,
    source: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut output = Command::new(kali_bin());
    output
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path());
    if json_output {
        output.arg("--output").arg("json");
    }
    let output = output
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
            stdout.contains("browser object values spread iteration ok"),
            "json: {json}"
        );
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("browser object values spread iteration ok"),
            "stdout: {stdout}"
        );
        if command == "test" {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn run_supports_object_values_spread_iteration_when_browser_harness_is_configured() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        for json_output in [false, true] {
            assert_browser_harness_object_values_spread(
                "run",
                filename,
                browser_harness_object_values_spread_run_source(),
                json_output,
            );
        }
    }
}

#[test]
fn test_supports_object_values_spread_iteration_when_browser_harness_is_configured() {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        for json_output in [false, true] {
            assert_browser_harness_object_values_spread(
                "test",
                filename,
                browser_harness_object_values_spread_test_source(),
                json_output,
            );
        }
    }
}
