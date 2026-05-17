use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_object_enumeration_finalization_run_source() -> &'static str {
    r#"function assertSyncFinalization() {
  const values = { "b": 1, "a": 2 };
  let returnFinally = false;
  function returnProbe() {
    try {
      for (const key of Object.keys(values)) {
        return key;
      }
      throw new Error('unexpected empty Object.keys iteration');
    } finally {
      returnFinally = true;
    }
  }
  const returnValue = returnProbe();
  if (returnValue !== 'b' || !returnFinally) {
    throw new Error('unexpected Object.keys return/finally semantics');
  }

  let throwFinally = false;
  function throwProbe() {
    try {
      for (const entry of Object.entries(values)) {
        if (entry[0] === 'b') {
          throw new Error('boom');
        }
      }
      throw new Error('unexpected empty Object.entries iteration');
    } finally {
      throwFinally = true;
    }
  }
  let threw = false;
  try {
    throwProbe();
  } catch {
    threw = true;
  }
  if (!threw || !throwFinally) {
    throw new Error('unexpected Object.entries throw/finally semantics');
  }
}

assertSyncFinalization();
console.log('browser object enumeration finalization ok');
"#
}

fn browser_object_enumeration_finalization_test_source() -> &'static str {
    r#"function assertSyncFinalization() {
  const values = { "b": 1, "a": 2 };
  let returnFinally = false;
  function returnProbe() {
    try {
      for (const key of Object.keys(values)) {
        return key;
      }
      throw new Error('unexpected empty Object.keys iteration');
    } finally {
      returnFinally = true;
    }
  }
  const returnValue = returnProbe();
  if (returnValue !== 'b' || !returnFinally) {
    throw new Error('unexpected Object.keys return/finally semantics');
  }

  let throwFinally = false;
  function throwProbe() {
    try {
      for (const entry of Object.entries(values)) {
        if (entry[0] === 'b') {
          throw new Error('boom');
        }
      }
      throw new Error('unexpected empty Object.entries iteration');
    } finally {
      throwFinally = true;
    }
  }
  let threw = false;
  try {
    throwProbe();
  } catch {
    threw = true;
  }
  if (!threw || !throwFinally) {
    throw new Error('unexpected Object.entries throw/finally semantics');
  }
}

Kali.test('object enumeration finalization', () => {
  assertSyncFinalization();
});
"#
}

fn assert_browser_object_enumeration_finalization(
    command: &str,
    filename: &str,
    source: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
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
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
        if command == "run" {
            assert_eq!(json["exitCode"], 0);
            assert_eq!(json["payload"]["exitCode"], 0);
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
        }
        if command == "run" {
            let stdout = json["stdout"].as_str().expect("stdout string");
            assert!(
                stdout.contains("browser object enumeration finalization ok"),
                "json: {json}"
            );
        }
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if command == "run" {
            assert!(
                stdout.contains("browser object enumeration finalization ok"),
                "stdout: {stdout}"
            );
        } else {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn run_supports_object_enumeration_finalization_when_browser_harness_is_configured_in_js_input() {
    assert_browser_object_enumeration_finalization(
        "run",
        "main.js",
        browser_object_enumeration_finalization_run_source(),
        false,
    );
}

#[test]
fn run_supports_object_enumeration_finalization_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_object_enumeration_finalization(
        "run",
        "main.ts",
        browser_object_enumeration_finalization_run_source(),
        false,
    );
}

#[test]
fn run_supports_object_enumeration_finalization_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_object_enumeration_finalization(
        "run",
        "main.jsx",
        browser_object_enumeration_finalization_run_source(),
        false,
    );
}

#[test]
fn run_supports_object_enumeration_finalization_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_object_enumeration_finalization(
        "run",
        "main.tsx",
        browser_object_enumeration_finalization_run_source(),
        false,
    );
}

#[test]
fn test_supports_object_enumeration_finalization_when_browser_harness_is_configured_in_js_input() {
    assert_browser_object_enumeration_finalization(
        "test",
        "smoke.test.js",
        browser_object_enumeration_finalization_test_source(),
        false,
    );
}

#[test]
fn test_supports_object_enumeration_finalization_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_object_enumeration_finalization(
        "test",
        "smoke.test.ts",
        browser_object_enumeration_finalization_test_source(),
        false,
    );
}

#[test]
fn test_supports_object_enumeration_finalization_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_object_enumeration_finalization(
        "test",
        "smoke.test.jsx",
        browser_object_enumeration_finalization_test_source(),
        false,
    );
}

#[test]
fn test_supports_object_enumeration_finalization_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_object_enumeration_finalization(
        "test",
        "smoke.test.tsx",
        browser_object_enumeration_finalization_test_source(),
        false,
    );
}

#[test]
fn json_run_supports_object_enumeration_finalization_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_object_enumeration_finalization(
        "run",
        "main.js",
        browser_object_enumeration_finalization_run_source(),
        true,
    );
}

#[test]
fn json_test_supports_object_enumeration_finalization_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_object_enumeration_finalization(
        "test",
        "smoke.test.js",
        browser_object_enumeration_finalization_test_source(),
        true,
    );
}
