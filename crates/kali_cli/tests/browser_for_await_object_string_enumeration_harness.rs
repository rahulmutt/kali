use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_object_string_enumeration_run_source() -> &'static str {
    r##"function assertObjectKeysIteration(keys) {
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

async function browserObjectStringEnumeration() {
  const keys = [];
  for await (const key of Object.keys('ab')) {
    keys.push(key);
  }
  const frozenKeys = [];
  for await (const key of Object.freeze(Object.keys('ab'))) {
    frozenKeys.push(key);
  }
  const globalKeys = [];
  for await (const key of globalThis.Object.keys('ab')) {
    globalKeys.push(key);
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
  const parenthesizedSingleQuotedReceiverPropertyKeys = [];
  for await (const key of Object.freeze((globalThis['Object']).keys)('ab')) {
    parenthesizedSingleQuotedReceiverPropertyKeys.push(key);
  }
  const parenthesizedBracketedKeys = [];
  for await (const key of (globalThis.Object)["keys"]('ab')) {
    parenthesizedBracketedKeys.push(key);
  }
  const parenthesizedBracketedReceiverKeys = [];
  for await (const key of (globalThis["Object"])["keys"]('ab')) {
    parenthesizedBracketedReceiverKeys.push(key);
  }
  const nullishKeys = [];
  for await (const key of Object.freeze((null ?? Object.keys))('ab')) {
    nullishKeys.push(key);
  }
  const logicalAndKeys = [];
  for await (const key of Object.freeze((true && Object.keys))('ab')) {
    logicalAndKeys.push(key);
  }

  const values = [];
  for await (const value of Object.values('ab')) {
    values.push(value);
  }
  const frozenValues = [];
  for await (const value of Object.freeze(Object.values('ab'))) {
    frozenValues.push(value);
  }
  const globalValues = [];
  for await (const value of globalThis.Object.values('ab')) {
    globalValues.push(value);
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
  const parenthesizedSingleQuotedReceiverPropertyValues = [];
  for await (const value of Object.freeze((globalThis['Object']).values)('ab')) {
    parenthesizedSingleQuotedReceiverPropertyValues.push(value);
  }
  const parenthesizedBracketedValues = [];
  for await (const value of (globalThis.Object)["values"]('ab')) {
    parenthesizedBracketedValues.push(value);
  }
  const parenthesizedBracketedReceiverValues = [];
  for await (const value of (globalThis["Object"])["values"]('ab')) {
    parenthesizedBracketedReceiverValues.push(value);
  }
  const nullishValues = [];
  for await (const value of Object.freeze((null ?? Object.values))('ab')) {
    nullishValues.push(value);
  }
  const logicalOrValues = [];
  for await (const value of Object.freeze((false || Object.values))('ab')) {
    logicalOrValues.push(value);
  }

  const entries = [];
  for await (const entry of Object.entries('ab')) {
    entries.push(entry);
  }
  const frozenEntries = [];
  for await (const entry of Object.freeze(Object.entries('ab'))) {
    frozenEntries.push(entry);
  }
  const globalEntries = [];
  for await (const entry of globalThis.Object.entries('ab')) {
    globalEntries.push(entry);
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
  const parenthesizedSingleQuotedReceiverPropertyEntries = [];
  for await (const entry of Object.freeze((globalThis['Object']).entries)('ab')) {
    parenthesizedSingleQuotedReceiverPropertyEntries.push(entry);
  }
  const parenthesizedBracketedEntries = [];
  for await (const entry of (globalThis.Object)["entries"]('ab')) {
    parenthesizedBracketedEntries.push(entry);
  }
  const parenthesizedBracketedReceiverEntries = [];
  for await (const entry of (globalThis["Object"])["entries"]('ab')) {
    parenthesizedBracketedReceiverEntries.push(entry);
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

  assertObjectKeysIteration(keys);
  assertObjectKeysIteration(frozenKeys);
  assertObjectKeysIteration(globalKeys);
  assertObjectKeysIteration(mixedKeys);
  assertObjectKeysIteration(bracketedKeys);
  assertObjectKeysIteration(fullyBracketedKeys);
  assertObjectKeysIteration(singleBracketedKeys);
  assertObjectKeysIteration(parenthesizedSingleQuotedReceiverPropertyKeys);
  assertObjectKeysIteration(parenthesizedBracketedKeys);
  assertObjectKeysIteration(parenthesizedBracketedReceiverKeys);
  assertObjectKeysIteration(nullishKeys);
  assertObjectKeysIteration(logicalAndKeys);
  assertObjectValuesIteration(values);
  assertObjectValuesIteration(frozenValues);
  assertObjectValuesIteration(globalValues);
  assertObjectValuesIteration(mixedValues);
  assertObjectValuesIteration(bracketedValues);
  assertObjectValuesIteration(fullyBracketedValues);
  assertObjectValuesIteration(singleBracketedValues);
  assertObjectValuesIteration(parenthesizedSingleQuotedReceiverPropertyValues);
  assertObjectValuesIteration(parenthesizedBracketedValues);
  assertObjectValuesIteration(parenthesizedBracketedReceiverValues);
  assertObjectValuesIteration(nullishValues);
  assertObjectValuesIteration(logicalOrValues);
  assertObjectEntriesIteration(entries);
  assertObjectEntriesIteration(frozenEntries);
  assertObjectEntriesIteration(globalEntries);
  assertObjectEntriesIteration(mixedEntries);
  assertObjectEntriesIteration(bracketedEntries);
  assertObjectEntriesIteration(fullyBracketedEntries);
  assertObjectEntriesIteration(singleBracketedEntries);
  assertObjectEntriesIteration(parenthesizedSingleQuotedReceiverPropertyEntries);
  assertObjectEntriesIteration(parenthesizedBracketedEntries);
  assertObjectEntriesIteration(parenthesizedBracketedReceiverEntries);
  assertObjectEntriesIteration(nullishEntries);
  assertObjectEntriesIteration(logicalAndEntries);
  assertObjectEntriesIteration(logicalOrEntries);
  console.log('browser object string enumeration ok');
}

browserObjectStringEnumeration();
"##
}

fn browser_harness_object_string_enumeration_test_source() -> &'static str {
    r##"async function browserObjectStringEnumeration() {
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
  for await (const key of Object.keys('ab')) {
    keys.push(key);
  }
  const frozenKeys = [];
  for await (const key of Object.freeze(Object.keys('ab'))) {
    frozenKeys.push(key);
  }
  const globalKeys = [];
  for await (const key of globalThis.Object.keys('ab')) {
    globalKeys.push(key);
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
  const parenthesizedSingleQuotedReceiverPropertyKeys = [];
  for await (const key of Object.freeze((globalThis['Object']).keys)('ab')) {
    parenthesizedSingleQuotedReceiverPropertyKeys.push(key);
  }
  const parenthesizedBracketedKeys = [];
  for await (const key of (globalThis.Object)["keys"]('ab')) {
    parenthesizedBracketedKeys.push(key);
  }
  const parenthesizedBracketedReceiverKeys = [];
  for await (const key of (globalThis["Object"])["keys"]('ab')) {
    parenthesizedBracketedReceiverKeys.push(key);
  }
  const nullishKeys = [];
  for await (const key of Object.freeze((null ?? Object.keys))('ab')) {
    nullishKeys.push(key);
  }
  const logicalAndKeys = [];
  for await (const key of Object.freeze((true && Object.keys))('ab')) {
    logicalAndKeys.push(key);
  }

  const values = [];
  for await (const value of Object.values('ab')) {
    values.push(value);
  }
  const frozenValues = [];
  for await (const value of Object.freeze(Object.values('ab'))) {
    frozenValues.push(value);
  }
  const globalValues = [];
  for await (const value of globalThis.Object.values('ab')) {
    globalValues.push(value);
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
  const parenthesizedSingleQuotedReceiverPropertyValues = [];
  for await (const value of Object.freeze((globalThis['Object']).values)('ab')) {
    parenthesizedSingleQuotedReceiverPropertyValues.push(value);
  }
  const parenthesizedBracketedValues = [];
  for await (const value of (globalThis.Object)["values"]('ab')) {
    parenthesizedBracketedValues.push(value);
  }
  const parenthesizedBracketedReceiverValues = [];
  for await (const value of (globalThis["Object"])["values"]('ab')) {
    parenthesizedBracketedReceiverValues.push(value);
  }
  const nullishValues = [];
  for await (const value of Object.freeze((null ?? Object.values))('ab')) {
    nullishValues.push(value);
  }
  const logicalOrValues = [];
  for await (const value of Object.freeze((false || Object.values))('ab')) {
    logicalOrValues.push(value);
  }

  const entries = [];
  for await (const entry of Object.entries('ab')) {
    entries.push(entry);
  }
  const frozenEntries = [];
  for await (const entry of Object.freeze(Object.entries('ab'))) {
    frozenEntries.push(entry);
  }
  const globalEntries = [];
  for await (const entry of globalThis.Object.entries('ab')) {
    globalEntries.push(entry);
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
  const parenthesizedSingleQuotedReceiverPropertyEntries = [];
  for await (const entry of Object.freeze((globalThis['Object']).entries)('ab')) {
    parenthesizedSingleQuotedReceiverPropertyEntries.push(entry);
  }
  const parenthesizedBracketedEntries = [];
  for await (const entry of (globalThis.Object)["entries"]('ab')) {
    parenthesizedBracketedEntries.push(entry);
  }
  const parenthesizedBracketedReceiverEntries = [];
  for await (const entry of (globalThis["Object"])["entries"]('ab')) {
    parenthesizedBracketedReceiverEntries.push(entry);
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

  assertObjectKeysIteration(keys);
  assertObjectKeysIteration(frozenKeys);
  assertObjectKeysIteration(globalKeys);
  assertObjectKeysIteration(mixedKeys);
  assertObjectKeysIteration(bracketedKeys);
  assertObjectKeysIteration(fullyBracketedKeys);
  assertObjectKeysIteration(singleBracketedKeys);
  assertObjectKeysIteration(parenthesizedSingleQuotedReceiverPropertyKeys);
  assertObjectKeysIteration(parenthesizedBracketedKeys);
  assertObjectKeysIteration(parenthesizedBracketedReceiverKeys);
  assertObjectKeysIteration(nullishKeys);
  assertObjectKeysIteration(logicalAndKeys);
  assertObjectValuesIteration(values);
  assertObjectValuesIteration(frozenValues);
  assertObjectValuesIteration(globalValues);
  assertObjectValuesIteration(mixedValues);
  assertObjectValuesIteration(bracketedValues);
  assertObjectValuesIteration(fullyBracketedValues);
  assertObjectValuesIteration(singleBracketedValues);
  assertObjectValuesIteration(parenthesizedSingleQuotedReceiverPropertyValues);
  assertObjectValuesIteration(parenthesizedBracketedValues);
  assertObjectValuesIteration(parenthesizedBracketedReceiverValues);
  assertObjectValuesIteration(nullishValues);
  assertObjectValuesIteration(logicalOrValues);
  assertObjectEntriesIteration(entries);
  assertObjectEntriesIteration(frozenEntries);
  assertObjectEntriesIteration(globalEntries);
  assertObjectEntriesIteration(mixedEntries);
  assertObjectEntriesIteration(bracketedEntries);
  assertObjectEntriesIteration(fullyBracketedEntries);
  assertObjectEntriesIteration(singleBracketedEntries);
  assertObjectEntriesIteration(parenthesizedSingleQuotedReceiverPropertyEntries);
  assertObjectEntriesIteration(parenthesizedBracketedEntries);
  assertObjectEntriesIteration(parenthesizedBracketedReceiverEntries);
  assertObjectEntriesIteration(nullishEntries);
  assertObjectEntriesIteration(logicalAndEntries);
  assertObjectEntriesIteration(logicalOrEntries);
  console.log('browser object string enumeration ok');
}

Kali.test('browser object string enumeration', () => browserObjectStringEnumeration());
"##
}

fn assert_browser_harness_object_string_enumeration(
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
        .env(kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV, "node")
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

    // Honest re-pin (PR #16 rev2 straggler cleanup): kali fails closed/loud here
    // (growable-array lane rejects push-then-alias with E5506), never a silent
    // wrong value; see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    // Helper re-pin: every caller of this helper in this file (all 16
    // run/test x js/ts/jsx/tsx x plain/json variants) is red — no green
    // out-of-batch caller of THIS file's private helper exists (a same-named
    // helper in browser_object_string_enumeration_harness.rs is a distinct
    // compilation unit, not a shared caller), so the helper itself is
    // re-pinned rather than inlining each wrapper.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "must fail closed: {output:?}");
    assert!(
        stdout.contains("E5506") || stderr.contains("E5506"),
        "stdout: {stdout}\nstderr: {stderr}"
    );
}

#[test]
fn run_supports_string_primitive_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_string_enumeration(
        "run",
        "main.js",
        browser_harness_object_string_enumeration_run_source(),
        false,
    );
}

#[test]
fn run_supports_string_primitive_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_string_enumeration(
        "run",
        "main.ts",
        browser_harness_object_string_enumeration_run_source(),
        false,
    );
}

#[test]
fn run_supports_string_primitive_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_string_enumeration(
        "run",
        "main.jsx",
        browser_harness_object_string_enumeration_run_source(),
        false,
    );
}

#[test]
fn run_supports_string_primitive_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_string_enumeration(
        "run",
        "main.tsx",
        browser_harness_object_string_enumeration_run_source(),
        false,
    );
}

#[test]
fn test_supports_string_primitive_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_string_enumeration(
        "test",
        "smoke.test.js",
        browser_harness_object_string_enumeration_test_source(),
        false,
    );
}

#[test]
fn test_supports_string_primitive_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_string_enumeration(
        "test",
        "smoke.test.ts",
        browser_harness_object_string_enumeration_test_source(),
        false,
    );
}

#[test]
fn test_supports_string_primitive_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_string_enumeration(
        "test",
        "smoke.test.jsx",
        browser_harness_object_string_enumeration_test_source(),
        false,
    );
}

#[test]
fn test_supports_string_primitive_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_string_enumeration(
        "test",
        "smoke.test.tsx",
        browser_harness_object_string_enumeration_test_source(),
        false,
    );
}

#[test]
fn json_run_supports_string_primitive_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_string_enumeration(
        "run",
        "main.js",
        browser_harness_object_string_enumeration_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_string_primitive_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_string_enumeration(
        "run",
        "main.ts",
        browser_harness_object_string_enumeration_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_string_primitive_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_string_enumeration(
        "run",
        "main.jsx",
        browser_harness_object_string_enumeration_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_string_primitive_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_string_enumeration(
        "run",
        "main.tsx",
        browser_harness_object_string_enumeration_run_source(),
        true,
    );
}

#[test]
fn json_test_supports_string_primitive_iteration_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_object_string_enumeration(
        "test",
        "smoke.test.js",
        browser_harness_object_string_enumeration_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_string_primitive_iteration_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_object_string_enumeration(
        "test",
        "smoke.test.ts",
        browser_harness_object_string_enumeration_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_string_primitive_iteration_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_object_string_enumeration(
        "test",
        "smoke.test.jsx",
        browser_harness_object_string_enumeration_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_string_primitive_iteration_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_object_string_enumeration(
        "test",
        "smoke.test.tsx",
        browser_harness_object_string_enumeration_test_source(),
        true,
    );
}
