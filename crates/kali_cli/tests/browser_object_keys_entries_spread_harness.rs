use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_object_keys_entries_spread_source(test_mode: bool) -> String {
    if test_mode {
        return r#"Kali.test('object keys and entries spread iteration', () => {
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

  const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);
  const collectedKeys = [...Object.keys(fromEntries)];
  const globalKeys = [...globalThis.Object.keys(fromEntries)];
  const mixedKeys = [...globalThis.Object["keys"](fromEntries)];
  const mixedBracketedKeys = [...globalThis["Object"].keys(fromEntries)];
  const bracketedKeys = [...globalThis["Object"]["keys"](fromEntries)];
  const collectedEntries = [...Object.entries(fromEntries)];
  const globalEntries = [...globalThis.Object.entries(fromEntries)];
  const mixedEntries = [...globalThis.Object["entries"](fromEntries)];
  const mixedBracketedEntries = [...globalThis["Object"].entries(fromEntries)];
  const bracketedEntries = [...globalThis["Object"]["entries"](fromEntries)];

  assertObjectKeysIteration(collectedKeys);
  assertObjectKeysIteration(globalKeys);
  assertObjectKeysIteration(mixedKeys);
  assertObjectKeysIteration(mixedBracketedKeys);
  assertObjectKeysIteration(bracketedKeys);
  assertObjectEntriesIteration(collectedEntries);
  assertObjectEntriesIteration(globalEntries);
  assertObjectEntriesIteration(mixedEntries);
  assertObjectEntriesIteration(mixedBracketedEntries);
  assertObjectEntriesIteration(bracketedEntries);
  console.log('browser object keys and entries spread iteration ok');
});
"#
        .to_string();
    }

    r#"function browserObjectKeysEntriesSpreadIteration() {
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

  const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);
  const collectedKeys = [...Object.keys(fromEntries)];
  const globalKeys = [...globalThis.Object.keys(fromEntries)];
  const mixedKeys = [...globalThis.Object["keys"](fromEntries)];
  const mixedBracketedKeys = [...globalThis["Object"].keys(fromEntries)];
  const bracketedKeys = [...globalThis["Object"]["keys"](fromEntries)];
  const collectedEntries = [...Object.entries(fromEntries)];
  const globalEntries = [...globalThis.Object.entries(fromEntries)];
  const mixedEntries = [...globalThis.Object["entries"](fromEntries)];
  const mixedBracketedEntries = [...globalThis["Object"].entries(fromEntries)];
  const bracketedEntries = [...globalThis["Object"]["entries"](fromEntries)];

  assertObjectKeysIteration(collectedKeys);
  assertObjectKeysIteration(globalKeys);
  assertObjectKeysIteration(mixedKeys);
  assertObjectKeysIteration(mixedBracketedKeys);
  assertObjectKeysIteration(bracketedKeys);
  assertObjectEntriesIteration(collectedEntries);
  assertObjectEntriesIteration(globalEntries);
  assertObjectEntriesIteration(mixedEntries);
  assertObjectEntriesIteration(mixedBracketedEntries);
  assertObjectEntriesIteration(bracketedEntries);
  console.log('browser object keys and entries spread iteration ok');
}

browserObjectKeysEntriesSpreadIteration();
"#
    .to_string()
}

fn browser_harness_object_keys_entries_frozen_spread_source(test_mode: bool) -> String {
    browser_harness_object_keys_entries_spread_source(test_mode).replace(
        "  const fromEntries = Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]);",
        "  const fromEntries = Object.freeze(Object.fromEntries([[\"b\", 1], [\"a\", 2], [\"b\", 3]]));",
    )
}

fn assert_browser_harness_object_keys_entries_spread(
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
            stdout.contains("browser object keys and entries spread iteration ok"),
            "json: {json}"
        );
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("browser object keys and entries spread iteration ok"),
            "stdout: {stdout}"
        );
        if command == "test" {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn run_supports_object_keys_and_entries_spread_iteration_when_browser_harness_is_configured() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_browser_harness_object_keys_entries_spread(
            "run",
            filename,
            &browser_harness_object_keys_entries_spread_source(false),
            false,
        );
        assert_browser_harness_object_keys_entries_spread(
            "run",
            filename,
            &browser_harness_object_keys_entries_spread_source(false),
            true,
        );
        assert_browser_harness_object_keys_entries_spread(
            "run",
            filename,
            &browser_harness_object_keys_entries_frozen_spread_source(false),
            false,
        );
        assert_browser_harness_object_keys_entries_spread(
            "run",
            filename,
            &browser_harness_object_keys_entries_frozen_spread_source(false),
            true,
        );
    }
}

#[test]
fn test_supports_object_keys_and_entries_spread_iteration_when_browser_harness_is_configured() {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_browser_harness_object_keys_entries_spread(
            "test",
            filename,
            &browser_harness_object_keys_entries_spread_source(true),
            false,
        );
        assert_browser_harness_object_keys_entries_spread(
            "test",
            filename,
            &browser_harness_object_keys_entries_spread_source(true),
            true,
        );
        assert_browser_harness_object_keys_entries_spread(
            "test",
            filename,
            &browser_harness_object_keys_entries_frozen_spread_source(true),
            false,
        );
        assert_browser_harness_object_keys_entries_spread(
            "test",
            filename,
            &browser_harness_object_keys_entries_frozen_spread_source(true),
            true,
        );
    }
}
