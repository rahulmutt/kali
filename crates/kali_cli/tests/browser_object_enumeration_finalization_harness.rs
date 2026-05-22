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

  let valuesReturnFinally = false;
  function valuesReturnProbe() {
    try {
      for (const value of Object.values(values)) {
        return value;
      }
      throw new Error('unexpected empty Object.values iteration');
    } finally {
      valuesReturnFinally = true;
    }
  }
  const valuesReturnValue = valuesReturnProbe();
  if (valuesReturnValue !== 1 || !valuesReturnFinally) {
    throw new Error('unexpected Object.values return/finally semantics');
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

  let valuesThrowFinally = false;
  function valuesThrowProbe() {
    try {
      for (const value of Object.values(values)) {
        if (value === 1) {
          throw new Error('boom');
        }
      }
      throw new Error('unexpected empty Object.values iteration');
    } finally {
      valuesThrowFinally = true;
    }
  }
  let valuesThrew = false;
  try {
    valuesThrowProbe();
  } catch {
    valuesThrew = true;
  }
  if (!valuesThrew || !valuesThrowFinally) {
    throw new Error('unexpected Object.values throw/finally semantics');
  }

  let valuesBreakFinally = false;
  let valuesBreakSeen = false;
  function valuesBreakProbe() {
    try {
      for (const value of Object.values(values)) {
        if (value === 1) {
          continue;
        }
        valuesBreakSeen = true;
        break;
      }
    } finally {
      valuesBreakFinally = true;
    }
  }
  valuesBreakProbe();
  if (!valuesBreakSeen || !valuesBreakFinally) {
    throw new Error('unexpected Object.values break/continue semantics');
  }

  let entriesBreakFinally = false;
  let entriesBreakSeen = false;
  function entriesBreakProbe() {
    try {
      for (const entry of Object.entries(values)) {
        if (entry[0] === 'b') {
          continue;
        }
        entriesBreakSeen = true;
        break;
      }
    } finally {
      entriesBreakFinally = true;
    }
  }
  entriesBreakProbe();
  if (!entriesBreakSeen || !entriesBreakFinally) {
    throw new Error('unexpected Object.entries break/continue semantics');
  }

  let reflectReturnFinally = false;
  function reflectReturnProbe() {
    try {
      for (const key of Reflect.ownKeys(values)) {
        return key;
      }
      throw new Error('unexpected empty Reflect.ownKeys iteration');
    } finally {
      reflectReturnFinally = true;
    }
  }
  const reflectReturnValue = reflectReturnProbe();
  if (reflectReturnValue !== 'b' || !reflectReturnFinally) {
    throw new Error('unexpected Reflect.ownKeys return/finally semantics');
  }

  let reflectThrowFinally = false;
  function reflectThrowProbe() {
    try {
      for (const key of Reflect.ownKeys(values)) {
        if (key === 'b') {
          throw new Error('boom');
        }
      }
      throw new Error('unexpected empty Reflect.ownKeys iteration');
    } finally {
      reflectThrowFinally = true;
    }
  }
  let reflectThrew = false;
  try {
    reflectThrowProbe();
  } catch {
    reflectThrew = true;
  }
  if (!reflectThrew || !reflectThrowFinally) {
    throw new Error('unexpected Reflect.ownKeys throw/finally semantics');
  }

  const asyncValues = { "b": 1, "a": 2 };
  let asyncFinallySeen = false;
  let asyncThrew = false;
  try {
    for await (const key of Object.keys(asyncValues)) {
      if (key === 'b') {
        throw new Error('boom');
      }
    }
    throw new Error('unexpected empty async Object.keys iteration');
  } catch {
    asyncThrew = true;
  } finally {
    asyncFinallySeen = true;
  }
  if (!asyncThrew || !asyncFinallySeen) {
    throw new Error('unexpected async Object.keys throw/finally semantics');
  }

  let asyncValuesFinallySeen = false;
  let asyncValuesThrew = false;
  try {
    for await (const value of Object.values(asyncValues)) {
      if (value === 1) {
        throw new Error('boom');
      }
    }
    throw new Error('unexpected empty async Object.values iteration');
  } catch {
    asyncValuesThrew = true;
  } finally {
    asyncValuesFinallySeen = true;
  }
  if (!asyncValuesThrew || !asyncValuesFinallySeen) {
    throw new Error('unexpected async Object.values throw/finally semantics');
  }

  let asyncValuesBreakFinally = false;
  let asyncValuesBreakSeen = false;
  try {
    for await (const value of Object.values(asyncValues)) {
      if (value === 1) {
        continue;
      }
      asyncValuesBreakSeen = true;
      break;
    }
  } finally {
    asyncValuesBreakFinally = true;
  }
  if (!asyncValuesBreakSeen || !asyncValuesBreakFinally) {
    throw new Error('unexpected async Object.values break/continue semantics');
  }

  let asyncEntriesBreakFinally = false;
  let asyncEntriesBreakSeen = false;
  try {
    for await (const entry of Object.entries(asyncValues)) {
      if (entry[0] === 'b') {
        continue;
      }
      asyncEntriesBreakSeen = true;
      break;
    }
  } finally {
    asyncEntriesBreakFinally = true;
  }
  if (!asyncEntriesBreakSeen || !asyncEntriesBreakFinally) {
    throw new Error('unexpected async Object.entries break/continue semantics');
  }

  let asyncEntriesFinallySeen = false;
  let asyncEntriesThrew = false;
  try {
    for await (const entry of Object.entries(asyncValues)) {
      if (entry[0] === 'b') {
        throw new Error('boom');
      }
    }
    throw new Error('unexpected empty async Object.entries iteration');
  } catch {
    asyncEntriesThrew = true;
  } finally {
    asyncEntriesFinallySeen = true;
  }
  if (!asyncEntriesThrew || !asyncEntriesFinallySeen) {
    throw new Error('unexpected async Object.entries throw/finally semantics');
  }

  let asyncReflectReturnFinallySeen = false;
  function asyncReflectReturnProbe() {
    try {
      for await (const key of Reflect.ownKeys(asyncValues)) {
        return key;
      }
      throw new Error('unexpected empty async Reflect.ownKeys iteration');
    } finally {
      asyncReflectReturnFinallySeen = true;
    }
  }
  const asyncReflectReturnValue = asyncReflectReturnProbe();
  if (asyncReflectReturnValue !== 'b' || !asyncReflectReturnFinallySeen) {
    throw new Error('unexpected async Reflect.ownKeys return/finally semantics');
  }

  let asyncReflectThrowFinallySeen = false;
  let asyncReflectThrew = false;
  try {
    for await (const key of Reflect.ownKeys(asyncValues)) {
      if (key === 'b') {
        throw new Error('boom');
      }
    }
    throw new Error('unexpected empty async Reflect.ownKeys iteration');
  } catch {
    asyncReflectThrew = true;
  } finally {
    asyncReflectThrowFinallySeen = true;
  }
  if (!asyncReflectThrew || !asyncReflectThrowFinallySeen) {
    throw new Error('unexpected async Reflect.ownKeys throw/finally semantics');
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

  let reflectReturnFinally = false;
  function reflectReturnProbe() {
    try {
      for (const key of Reflect.ownKeys(values)) {
        return key;
      }
      throw new Error('unexpected empty Reflect.ownKeys iteration');
    } finally {
      reflectReturnFinally = true;
    }
  }
  const reflectReturnValue = reflectReturnProbe();
  if (reflectReturnValue !== 'b' || !reflectReturnFinally) {
    throw new Error('unexpected Reflect.ownKeys return/finally semantics');
  }

  let reflectThrowFinally = false;
  function reflectThrowProbe() {
    try {
      for (const key of Reflect.ownKeys(values)) {
        if (key === 'b') {
          throw new Error('boom');
        }
      }
      throw new Error('unexpected empty Reflect.ownKeys iteration');
    } finally {
      reflectThrowFinally = true;
    }
  }
  let reflectThrew = false;
  try {
    reflectThrowProbe();
  } catch {
    reflectThrew = true;
  }
  if (!reflectThrew || !reflectThrowFinally) {
    throw new Error('unexpected Reflect.ownKeys throw/finally semantics');
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
