use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn object_enumeration_finalization_run_source() -> &'static str {
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

assertSyncFinalization();
console.log('object enumeration finalization ok');
"#
}

fn object_enumeration_finalization_test_source() -> &'static str {
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

Kali.test('object enumeration finalization', () => {
  assertSyncFinalization();
});
"#
}

fn assert_object_enumeration_finalization(command: &str, filename: &str, source: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
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
    if command == "test" {
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    } else {
        assert!(
            stdout.contains("object enumeration finalization ok"),
            "stdout: {stdout}"
        );
    }
}

#[test]
fn run_supports_object_enumeration_finalization_in_js_input() {
    assert_object_enumeration_finalization(
        "run",
        "main.js",
        object_enumeration_finalization_run_source(),
    );
}

#[test]
fn run_supports_object_enumeration_finalization_in_ts_input() {
    assert_object_enumeration_finalization(
        "run",
        "main.ts",
        object_enumeration_finalization_run_source(),
    );
}

#[test]
fn test_supports_object_enumeration_finalization_in_js_input() {
    assert_object_enumeration_finalization(
        "test",
        "smoke.test.js",
        object_enumeration_finalization_test_source(),
    );
}

#[test]
fn test_supports_object_enumeration_finalization_in_ts_input() {
    assert_object_enumeration_finalization(
        "test",
        "smoke.test.ts",
        object_enumeration_finalization_test_source(),
    );
}
