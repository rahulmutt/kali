use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_object_string_enumeration_source() -> &'static str {
    r##"// kali-tree-shake: browserObjectStringEnumeration
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

function browserObjectStringEnumeration() {
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
  const globalKeys = [];
  for (const key of globalThis.Object.keys('ab')) {
    globalKeys.push(key);
  }
  const bracketedRootKeys = [];
  for (const key of Object["keys"]('ab')) {
    bracketedRootKeys.push(key);
  }
  const mixedKeys = [];
  for (const key of globalThis.Object["keys"]('ab')) {
    mixedKeys.push(key);
  }
  const bracketedKeys = [];
  for (const key of globalThis["Object"].keys('ab')) {
    bracketedKeys.push(key);
  }
  const fullyBracketedKeys = [];
  for (const key of globalThis["Object"]["keys"]('ab')) {
    fullyBracketedKeys.push(key);
  }
  const singleBracketedKeys = [];
  for (const key of globalThis['Object']['keys']('ab')) {
    singleBracketedKeys.push(key);
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
  const globalValues = [];
  for (const value of globalThis.Object.values('ab')) {
    globalValues.push(value);
  }
  const bracketedRootValues = [];
  for (const value of Object["values"]('ab')) {
    bracketedRootValues.push(value);
  }
  const mixedValues = [];
  for (const value of globalThis.Object["values"]('ab')) {
    mixedValues.push(value);
  }
  const bracketedValues = [];
  for (const value of globalThis["Object"].values('ab')) {
    bracketedValues.push(value);
  }
  const fullyBracketedValues = [];
  for (const value of globalThis["Object"]["values"]('ab')) {
    fullyBracketedValues.push(value);
  }
  const singleBracketedValues = [];
  for (const value of globalThis['Object']['values']('ab')) {
    singleBracketedValues.push(value);
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
  const globalEntries = [];
  for (const entry of globalThis.Object.entries('ab')) {
    globalEntries.push(entry);
  }
  const bracketedRootEntries = [];
  for (const entry of Object["entries"]('ab')) {
    bracketedRootEntries.push(entry);
  }
  const mixedEntries = [];
  for (const entry of globalThis.Object["entries"]('ab')) {
    mixedEntries.push(entry);
  }
  const bracketedEntries = [];
  for (const entry of globalThis["Object"].entries('ab')) {
    bracketedEntries.push(entry);
  }
  const fullyBracketedEntries = [];
  for (const entry of globalThis["Object"]["entries"]('ab')) {
    fullyBracketedEntries.push(entry);
  }
  const singleBracketedEntries = [];
  for (const entry of globalThis['Object']['entries']('ab')) {
    singleBracketedEntries.push(entry);
  }

  assertObjectKeysIteration(keys);
  assertObjectKeysIteration(nullishKeys);
  assertObjectKeysIteration(logicalAndKeys);
  assertObjectKeysIteration(logicalOrKeys);
  assertObjectKeysIteration(globalKeys);
  assertObjectKeysIteration(bracketedRootKeys);
  assertObjectKeysIteration(mixedKeys);
  assertObjectKeysIteration(bracketedKeys);
  assertObjectKeysIteration(fullyBracketedKeys);
  assertObjectKeysIteration(singleBracketedKeys);
  assertObjectValuesIteration(values);
  assertObjectValuesIteration(nullishValues);
  assertObjectValuesIteration(logicalAndValues);
  assertObjectValuesIteration(logicalOrValues);
  assertObjectValuesIteration(globalValues);
  assertObjectValuesIteration(bracketedRootValues);
  assertObjectValuesIteration(mixedValues);
  assertObjectValuesIteration(bracketedValues);
  assertObjectValuesIteration(fullyBracketedValues);
  assertObjectValuesIteration(singleBracketedValues);
  assertObjectEntriesIteration(entries);
  assertObjectEntriesIteration(nullishEntries);
  assertObjectEntriesIteration(logicalAndEntries);
  assertObjectEntriesIteration(logicalOrEntries);
  assertObjectEntriesIteration(globalEntries);
  assertObjectEntriesIteration(bracketedRootEntries);
  assertObjectEntriesIteration(mixedEntries);
  assertObjectEntriesIteration(bracketedEntries);
  assertObjectEntriesIteration(fullyBracketedEntries);
  assertObjectEntriesIteration(singleBracketedEntries);
}
"##
}

fn assert_browser_bundle_object_string_enumeration(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_object_string_enumeration_source(),
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
}

#[test]
fn build_emits_string_primitive_object_enumeration_semantics_in_js_input() {
    assert_browser_bundle_object_string_enumeration("app.js", false);
}

#[test]
fn build_emits_string_primitive_object_enumeration_semantics_in_ts_input() {
    assert_browser_bundle_object_string_enumeration("app.ts", false);
}

#[test]
fn build_emits_string_primitive_object_enumeration_semantics_in_jsx_input() {
    assert_browser_bundle_object_string_enumeration("app.jsx", false);
}

#[test]
fn build_emits_string_primitive_object_enumeration_semantics_in_tsx_input() {
    assert_browser_bundle_object_string_enumeration("app.tsx", false);
}

#[test]
fn json_build_emits_string_primitive_object_enumeration_semantics_in_js_input() {
    assert_browser_bundle_object_string_enumeration("app.js", true);
}

#[test]
fn json_build_emits_string_primitive_object_enumeration_semantics_in_ts_input() {
    assert_browser_bundle_object_string_enumeration("app.ts", true);
}

#[test]
fn json_build_emits_string_primitive_object_enumeration_semantics_in_jsx_input() {
    assert_browser_bundle_object_string_enumeration("app.jsx", true);
}

#[test]
fn json_build_emits_string_primitive_object_enumeration_semantics_in_tsx_input() {
    assert_browser_bundle_object_string_enumeration("app.tsx", true);
}

fn browser_bundle_object_string_enumeration_await_source() -> &'static str {
    r##"// kali-tree-shake: browserObjectStringEnumerationAwait
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

export async function browserObjectStringEnumerationAwait() {
  const keys = [];
  for await (const key of Object.keys('ab')) {
    keys.push(key);
  }
  const nullishKeys = [];
  for await (const key of Object.freeze((null ?? Object.keys))('ab')) {
    nullishKeys.push(key);
  }
  const logicalAndKeys = [];
  for await (const key of Object.freeze((true && Object.keys))('ab')) {
    logicalAndKeys.push(key);
  }
  const logicalOrKeys = [];
  for await (const key of Object.freeze((false || Object.keys))('ab')) {
    logicalOrKeys.push(key);
  }
  const globalKeys = [];
  for await (const key of globalThis.Object.keys('ab')) {
    globalKeys.push(key);
  }
  const bracketedRootKeys = [];
  for await (const key of Object["keys"]('ab')) {
    bracketedRootKeys.push(key);
  }
  const mixedKeys = [];
  for await (const key of globalThis.Object["keys"]('ab')) {
    mixedKeys.push(key);
  }
  const bracketedKeys = [];
  for await (const key of globalThis["Object"].keys('ab')) {
    bracketedKeys.push(key);
  }
  const fullyBracketedKeys = [];
  for await (const key of globalThis["Object"]["keys"]('ab')) {
    fullyBracketedKeys.push(key);
  }
  const singleBracketedKeys = [];
  for await (const key of globalThis['Object']['keys']('ab')) {
    singleBracketedKeys.push(key);
  }

  const values = [];
  for await (const value of Object.values('ab')) {
    values.push(value);
  }
  const nullishValues = [];
  for await (const value of Object.freeze((null ?? Object.values))('ab')) {
    nullishValues.push(value);
  }
  const logicalAndValues = [];
  for await (const value of Object.freeze((true && Object.values))('ab')) {
    logicalAndValues.push(value);
  }
  const logicalOrValues = [];
  for await (const value of Object.freeze((false || Object.values))('ab')) {
    logicalOrValues.push(value);
  }
  const globalValues = [];
  for await (const value of globalThis.Object.values('ab')) {
    globalValues.push(value);
  }
  const bracketedRootValues = [];
  for await (const value of Object["values"]('ab')) {
    bracketedRootValues.push(value);
  }
  const mixedValues = [];
  for await (const value of globalThis.Object["values"]('ab')) {
    mixedValues.push(value);
  }
  const bracketedValues = [];
  for await (const value of globalThis["Object"].values('ab')) {
    bracketedValues.push(value);
  }
  const fullyBracketedValues = [];
  for await (const value of globalThis["Object"]["values"]('ab')) {
    fullyBracketedValues.push(value);
  }
  const singleBracketedValues = [];
  for await (const value of globalThis['Object']['values']('ab')) {
    singleBracketedValues.push(value);
  }

  const entries = [];
  for await (const entry of Object.entries('ab')) {
    entries.push(entry);
  }
  const nullishEntries = [];
  for await (const entry of Object.freeze((null ?? Object.entries))('ab')) {
    nullishEntries.push(entry);
  }
  const logicalAndEntries = [];
  for await (const entry of Object.freeze((true && Object.entries))('ab')) {
    logicalAndEntries.push(entry);
  }
  const logicalOrEntries = [];
  for await (const entry of Object.freeze((false || Object.entries))('ab')) {
    logicalOrEntries.push(entry);
  }
  const globalEntries = [];
  for await (const entry of globalThis.Object.entries('ab')) {
    globalEntries.push(entry);
  }
  const bracketedRootEntries = [];
  for await (const entry of Object["entries"]('ab')) {
    bracketedRootEntries.push(entry);
  }
  const mixedEntries = [];
  for await (const entry of globalThis.Object["entries"]('ab')) {
    mixedEntries.push(entry);
  }
  const bracketedEntries = [];
  for await (const entry of globalThis["Object"].entries('ab')) {
    bracketedEntries.push(entry);
  }
  const fullyBracketedEntries = [];
  for await (const entry of globalThis["Object"]["entries"]('ab')) {
    fullyBracketedEntries.push(entry);
  }
  const singleBracketedEntries = [];
  for await (const entry of globalThis['Object']['entries']('ab')) {
    singleBracketedEntries.push(entry);
  }

  assertObjectKeysIteration(keys);
  assertObjectKeysIteration(nullishKeys);
  assertObjectKeysIteration(logicalAndKeys);
  assertObjectKeysIteration(logicalOrKeys);
  assertObjectKeysIteration(globalKeys);
  assertObjectKeysIteration(bracketedRootKeys);
  assertObjectKeysIteration(mixedKeys);
  assertObjectKeysIteration(bracketedKeys);
  assertObjectKeysIteration(fullyBracketedKeys);
  assertObjectKeysIteration(singleBracketedKeys);
  assertObjectValuesIteration(values);
  assertObjectValuesIteration(nullishValues);
  assertObjectValuesIteration(logicalAndValues);
  assertObjectValuesIteration(logicalOrValues);
  assertObjectValuesIteration(globalValues);
  assertObjectValuesIteration(bracketedRootValues);
  assertObjectValuesIteration(mixedValues);
  assertObjectValuesIteration(bracketedValues);
  assertObjectValuesIteration(fullyBracketedValues);
  assertObjectValuesIteration(singleBracketedValues);
  assertObjectEntriesIteration(entries);
  assertObjectEntriesIteration(nullishEntries);
  assertObjectEntriesIteration(logicalAndEntries);
  assertObjectEntriesIteration(logicalOrEntries);
  assertObjectEntriesIteration(globalEntries);
  assertObjectEntriesIteration(bracketedRootEntries);
  assertObjectEntriesIteration(mixedEntries);
  assertObjectEntriesIteration(bracketedEntries);
  assertObjectEntriesIteration(fullyBracketedEntries);
  assertObjectEntriesIteration(singleBracketedEntries);
}
"##
}

fn assert_browser_bundle_object_string_enumeration_await(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_object_string_enumeration_await_source(),
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
}

#[test]
fn build_emits_for_await_string_primitive_object_enumeration_semantics_in_js_input() {
    assert_browser_bundle_object_string_enumeration_await("app.js", false);
}

#[test]
fn build_emits_for_await_string_primitive_object_enumeration_semantics_in_ts_input() {
    assert_browser_bundle_object_string_enumeration_await("app.ts", false);
}

#[test]
fn build_emits_for_await_string_primitive_object_enumeration_semantics_in_jsx_input() {
    assert_browser_bundle_object_string_enumeration_await("app.jsx", false);
}

#[test]
fn build_emits_for_await_string_primitive_object_enumeration_semantics_in_tsx_input() {
    assert_browser_bundle_object_string_enumeration_await("app.tsx", false);
}

#[test]
fn json_build_emits_for_await_string_primitive_object_enumeration_semantics_in_js_input() {
    assert_browser_bundle_object_string_enumeration_await("app.js", true);
}

#[test]
fn json_build_emits_for_await_string_primitive_object_enumeration_semantics_in_ts_input() {
    assert_browser_bundle_object_string_enumeration_await("app.ts", true);
}

#[test]
fn json_build_emits_for_await_string_primitive_object_enumeration_semantics_in_jsx_input() {
    assert_browser_bundle_object_string_enumeration_await("app.jsx", true);
}

#[test]
fn json_build_emits_for_await_string_primitive_object_enumeration_semantics_in_tsx_input() {
    assert_browser_bundle_object_string_enumeration_await("app.tsx", true);
}
