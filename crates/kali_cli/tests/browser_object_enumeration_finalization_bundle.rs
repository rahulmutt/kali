use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_object_enumeration_finalization_source() -> &'static str {
    r#"// kali-tree-shake: browserObjectEnumerationFinalizationWrapper
async function browserObjectEnumerationFinalizationWrapper() {
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

  console.log('browser object enumeration finalization ok');
}
"#
}

fn assert_browser_object_enumeration_finalization(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_object_enumeration_finalization_source(),
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

    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!output.status.success(), "must fail closed: {output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stderr.contains("E5506") || stdout.contains("E5506"),
        "stdout: {stdout}\nstderr: {stderr}"
    );
}

#[test]
fn build_emits_object_enumeration_finalization_in_js_input() {
    assert_browser_object_enumeration_finalization("app.js", false);
}

#[test]
fn json_build_emits_object_enumeration_finalization_in_js_input() {
    assert_browser_object_enumeration_finalization("app.js", true);
}

#[test]
fn build_emits_object_enumeration_finalization_in_ts_input() {
    assert_browser_object_enumeration_finalization("app.ts", false);
}

#[test]
fn json_build_emits_object_enumeration_finalization_in_ts_input() {
    assert_browser_object_enumeration_finalization("app.ts", true);
}

#[test]
fn build_emits_object_enumeration_finalization_in_jsx_input() {
    assert_browser_object_enumeration_finalization("app.jsx", false);
}

#[test]
fn json_build_emits_object_enumeration_finalization_in_jsx_input() {
    assert_browser_object_enumeration_finalization("app.jsx", true);
}

#[test]
fn build_emits_object_enumeration_finalization_in_tsx_input() {
    assert_browser_object_enumeration_finalization("app.tsx", false);
}

#[test]
fn json_build_emits_object_enumeration_finalization_in_tsx_input() {
    assert_browser_object_enumeration_finalization("app.tsx", true);
}
