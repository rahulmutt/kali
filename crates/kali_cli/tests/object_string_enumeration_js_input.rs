use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn object_string_enumeration_run_source() -> &'static str {
    r#"function assertObjectKeysIteration(keys) {
  if (keys.length !== 2 || keys[0] !== '0' || keys[1] !== '1') {
    throw new Error('unexpected Object.keys string-primitive iteration semantics');
  }
}

function assertObjectValuesIteration(values) {
  if (values.length !== 2 || values[0] !== 'a' || values[1] !== 'b') {
    throw new Error('unexpected Object.values string-primitive iteration semantics');
  }
}

function assertObjectEntriesIteration(entries) {
  if (
    entries.length !== 2 ||
    entries[0][0] !== '0' ||
    entries[0][1] !== 'a' ||
    entries[1][0] !== '1' ||
    entries[1][1] !== 'b'
  ) {
    throw new Error('unexpected Object.entries string-primitive iteration semantics');
  }
}

function objectStringEnumeration() {
  const keys = [];
  for (const key of Object.keys('ab')) {
    keys.push(key);
  }
  const nullishKeys = [];
  for (const key of Object.freeze((null ?? Object.keys))('ab')) {
    nullishKeys.push(key);
  }
  const logicalAndKeys = [];
  for (const key of Object.freeze((true && Object.keys))('ab')) {
    logicalAndKeys.push(key);
  }
  const logicalOrKeys = [];
  for (const key of Object.freeze((false || Object.keys))('ab')) {
    logicalOrKeys.push(key);
  }
  const values = [];
  for (const value of Object.values('ab')) {
    values.push(value);
  }
  const nullishValues = [];
  for (const value of Object.freeze((null ?? Object.values))('ab')) {
    nullishValues.push(value);
  }
  const logicalAndValues = [];
  for (const value of Object.freeze((true && Object.values))('ab')) {
    logicalAndValues.push(value);
  }
  const logicalOrValues = [];
  for (const value of Object.freeze((false || Object.values))('ab')) {
    logicalOrValues.push(value);
  }
  const entries = [];
  for (const entry of Object.entries('ab')) {
    entries.push(entry);
  }
  const nullishEntries = [];
  for (const entry of Object.freeze((null ?? Object.entries))('ab')) {
    nullishEntries.push(entry);
  }
  const logicalAndEntries = [];
  for (const entry of Object.freeze((true && Object.entries))('ab')) {
    logicalAndEntries.push(entry);
  }
  const logicalOrEntries = [];
  for (const entry of Object.freeze((false || Object.entries))('ab')) {
    logicalOrEntries.push(entry);
  }
  assertObjectKeysIteration(keys);
  assertObjectKeysIteration(nullishKeys);
  assertObjectKeysIteration(logicalAndKeys);
  assertObjectKeysIteration(logicalOrKeys);
  assertObjectValuesIteration(values);
  assertObjectValuesIteration(nullishValues);
  assertObjectValuesIteration(logicalAndValues);
  assertObjectValuesIteration(logicalOrValues);
  assertObjectEntriesIteration(entries);
  assertObjectEntriesIteration(nullishEntries);
  assertObjectEntriesIteration(logicalAndEntries);
  assertObjectEntriesIteration(logicalOrEntries);
  console.log('object string enumeration ok');
}

objectStringEnumeration();
"#
}

fn object_string_enumeration_test_source() -> &'static str {
    r#"Kali.test('object string enumeration', () => {
  function assertObjectKeysIteration(keys) {
    if (keys.length !== 2 || keys[0] !== '0' || keys[1] !== '1') {
      throw new Error('unexpected Object.keys string-primitive iteration semantics');
    }
  }

  function assertObjectValuesIteration(values) {
    if (values.length !== 2 || values[0] !== 'a' || values[1] !== 'b') {
      throw new Error('unexpected Object.values string-primitive iteration semantics');
    }
  }

  function assertObjectEntriesIteration(entries) {
    if (
      entries.length !== 2 ||
      entries[0][0] !== '0' ||
      entries[0][1] !== 'a' ||
      entries[1][0] !== '1' ||
      entries[1][1] !== 'b'
    ) {
      throw new Error('unexpected Object.entries string-primitive iteration semantics');
    }
  }

  const keys = [];
  for (const key of Object.keys('ab')) {
    keys.push(key);
  }
  const nullishKeys = [];
  for (const key of Object.freeze((null ?? Object.keys))('ab')) {
    nullishKeys.push(key);
  }
  const logicalAndKeys = [];
  for (const key of Object.freeze((true && Object.keys))('ab')) {
    logicalAndKeys.push(key);
  }
  const logicalOrKeys = [];
  for (const key of Object.freeze((false || Object.keys))('ab')) {
    logicalOrKeys.push(key);
  }
  const values = [];
  for (const value of Object.values('ab')) {
    values.push(value);
  }
  const nullishValues = [];
  for (const value of Object.freeze((null ?? Object.values))('ab')) {
    nullishValues.push(value);
  }
  const logicalAndValues = [];
  for (const value of Object.freeze((true && Object.values))('ab')) {
    logicalAndValues.push(value);
  }
  const logicalOrValues = [];
  for (const value of Object.freeze((false || Object.values))('ab')) {
    logicalOrValues.push(value);
  }
  const entries = [];
  for (const entry of Object.entries('ab')) {
    entries.push(entry);
  }
  const nullishEntries = [];
  for (const entry of Object.freeze((null ?? Object.entries))('ab')) {
    nullishEntries.push(entry);
  }
  const logicalAndEntries = [];
  for (const entry of Object.freeze((true && Object.entries))('ab')) {
    logicalAndEntries.push(entry);
  }
  const logicalOrEntries = [];
  for (const entry of Object.freeze((false || Object.entries))('ab')) {
    logicalOrEntries.push(entry);
  }
  assertObjectKeysIteration(keys);
  assertObjectKeysIteration(nullishKeys);
  assertObjectKeysIteration(logicalAndKeys);
  assertObjectKeysIteration(logicalOrKeys);
  assertObjectValuesIteration(values);
  assertObjectValuesIteration(nullishValues);
  assertObjectValuesIteration(logicalAndValues);
  assertObjectValuesIteration(logicalOrValues);
  assertObjectEntriesIteration(entries);
  assertObjectEntriesIteration(nullishEntries);
  assertObjectEntriesIteration(logicalAndEntries);
  assertObjectEntriesIteration(logicalOrEntries);
  console.log('object string enumeration ok');
});
"#
}

fn assert_object_string_enumeration(command: &str, filename: &str, source: &'static str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg(command)
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!output.status.success(), "must fail closed: {output:?}");
}

#[test]
fn run_accepts_object_string_enumeration_in_js_input() {
    assert_object_string_enumeration("run", "main.js", object_string_enumeration_run_source());
}

#[test]
fn run_accepts_object_string_enumeration_in_ts_input() {
    assert_object_string_enumeration("run", "main.ts", object_string_enumeration_run_source());
}

#[test]
fn run_accepts_object_string_enumeration_in_jsx_input() {
    assert_object_string_enumeration("run", "main.jsx", object_string_enumeration_run_source());
}

#[test]
fn run_accepts_object_string_enumeration_in_tsx_input() {
    assert_object_string_enumeration("run", "main.tsx", object_string_enumeration_run_source());
}

#[test]
fn json_run_accepts_object_string_enumeration_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, object_string_enumeration_run_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!output.status.success(), "must fail closed: {output:?}");
}

#[test]
fn json_run_accepts_object_string_enumeration_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, object_string_enumeration_run_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!output.status.success(), "must fail closed: {output:?}");
}

#[test]
fn json_run_accepts_object_string_enumeration_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.jsx");
    fs::write(&source_path, object_string_enumeration_run_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!output.status.success(), "must fail closed: {output:?}");
}

#[test]
fn json_run_accepts_object_string_enumeration_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(&source_path, object_string_enumeration_run_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!output.status.success(), "must fail closed: {output:?}");
}

#[test]
fn test_accepts_object_string_enumeration_in_js_input() {
    assert_object_string_enumeration(
        "test",
        "main.test.js",
        object_string_enumeration_test_source(),
    );
}

#[test]
fn test_accepts_object_string_enumeration_in_ts_input() {
    assert_object_string_enumeration(
        "test",
        "main.test.ts",
        object_string_enumeration_test_source(),
    );
}

#[test]
fn test_accepts_object_string_enumeration_in_jsx_input() {
    assert_object_string_enumeration(
        "test",
        "main.test.jsx",
        object_string_enumeration_test_source(),
    );
}

#[test]
fn test_accepts_object_string_enumeration_in_tsx_input() {
    assert_object_string_enumeration(
        "test",
        "main.test.tsx",
        object_string_enumeration_test_source(),
    );
}

#[test]
fn json_test_accepts_object_string_enumeration_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.js");
    fs::write(&source_path, object_string_enumeration_test_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!output.status.success(), "must fail closed: {output:?}");
}

#[test]
fn json_test_accepts_object_string_enumeration_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.ts");
    fs::write(&source_path, object_string_enumeration_test_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!output.status.success(), "must fail closed: {output:?}");
}

#[test]
fn json_test_accepts_object_string_enumeration_in_jsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.jsx");
    fs::write(&source_path, object_string_enumeration_test_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!output.status.success(), "must fail closed: {output:?}");
}

#[test]
fn json_test_accepts_object_string_enumeration_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.tsx");
    fs::write(&source_path, object_string_enumeration_test_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!output.status.success(), "must fail closed: {output:?}");
}
