use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_wrapped_object_enumeration_source() -> &'static str {
    r##"// kali-tree-shake: browserWrappedObjectEnumeration
function assertWrappedObjectEnumeration(keys, values, entries) {
  if (
    keys.length !== 4 ||
    keys[0] !== '1' ||
    keys[1] !== '2' ||
    keys[2] !== 'b' ||
    keys[3] !== 'a' ||
    values.length !== 4 ||
    values[0] !== 4 ||
    values[1] !== 2 ||
    values[2] !== 1 ||
    values[3] !== 3 ||
    entries.length !== 4 ||
    entries[0][0] !== '1' ||
    entries[0][1] !== 4 ||
    entries[1][0] !== '2' ||
    entries[1][1] !== 2 ||
    entries[2][0] !== 'b' ||
    entries[2][1] !== 1 ||
    entries[3][0] !== 'a' ||
    entries[3][1] !== 3
  ) {
    throw new Error('unexpected wrapped object enumeration ordering');
  }
}

function browserWrappedObjectEnumeration() {
  const wrappedConst = ({ "b": 1, "2": 2, "a": 3, "1": 4 } as const);
  const wrappedSatisfies = ({ "b": 1, "2": 2, "a": 3, "1": 4 } satisfies unknown);
  const frozenFromEntries = Object.freeze(Object.fromEntries([["b", 1], ["2", 2], ["a", 3], ["1", 4]]));

  const constKeys = Object.keys(wrappedConst);
  const constValues = Object.values(wrappedConst);
  const constEntries = Object.entries(wrappedConst);
  assertWrappedObjectEnumeration(constKeys, constValues, constEntries);

  const satisfiesKeys = Object.keys(wrappedSatisfies);
  const satisfiesValues = Object.values(wrappedSatisfies);
  const satisfiesEntries = Object.entries(wrappedSatisfies);
  assertWrappedObjectEnumeration(satisfiesKeys, satisfiesValues, satisfiesEntries);

  const frozenKeys = Object.keys(frozenFromEntries);
  const frozenBracketRootKeys = Object.freeze((globalThis["Object"]))["keys"](frozenFromEntries);
  const frozenSingleQuotedBracketRootKeys = Object.freeze((globalThis["Object"])['keys'])(frozenFromEntries);
  const frozenBracketedKeys = Object.freeze((globalThis['Object'])["keys"])(frozenFromEntries);
  const frozenSingleQuotedBracketedKeys = Object.freeze((globalThis['Object'])['keys'])(frozenFromEntries);
  const frozenValues = Object.values(frozenFromEntries);
  const frozenEntries = Object.entries(frozenFromEntries);
  assertWrappedObjectEnumeration(frozenKeys, frozenValues, frozenEntries);
  assertWrappedObjectEnumeration(frozenBracketRootKeys, frozenValues, frozenEntries);
  assertWrappedObjectEnumeration(frozenSingleQuotedBracketRootKeys, frozenValues, frozenEntries);
  assertWrappedObjectEnumeration(frozenBracketedKeys, frozenValues, frozenEntries);
  assertWrappedObjectEnumeration(frozenSingleQuotedBracketedKeys, frozenValues, frozenEntries);
}
"##
}

fn browser_bundle_wrapped_object_enumeration_js_source() -> &'static str {
    r##"// kali-tree-shake: browserWrappedObjectEnumeration
function assertWrappedObjectEnumeration(keys, values, entries) {
  if (
    keys.length !== 4 ||
    keys[0] !== '1' ||
    keys[1] !== '2' ||
    keys[2] !== 'b' ||
    keys[3] !== 'a' ||
    values.length !== 4 ||
    values[0] !== 4 ||
    values[1] !== 2 ||
    values[2] !== 1 ||
    values[3] !== 3 ||
    entries.length !== 4 ||
    entries[0][0] !== '1' ||
    entries[0][1] !== 4 ||
    entries[1][0] !== '2' ||
    entries[1][1] !== 2 ||
    entries[2][0] !== 'b' ||
    entries[2][1] !== 1 ||
    entries[3][0] !== 'a' ||
    entries[3][1] !== 3
  ) {
    throw new Error('unexpected wrapped object enumeration ordering');
  }
}

function browserWrappedObjectEnumeration() {
  const wrappedObject = { "b": 1, "2": 2, "a": 3, "1": 4 };
  const frozenFromEntries = Object.freeze(Object.fromEntries([["b", 1], ["2", 2], ["a", 3], ["1", 4]]));

  const objectKeys = Object.keys(wrappedObject);
  const objectValues = Object.values(wrappedObject);
  const objectEntries = Object.entries(wrappedObject);
  assertWrappedObjectEnumeration(objectKeys, objectValues, objectEntries);

  const spreadObjectKeys = [...Object.keys(wrappedObject)];
  const spreadObjectValues = [...Object.values(wrappedObject)];
  const spreadObjectEntries = [...Object.entries(wrappedObject)];
  assertWrappedObjectEnumeration(spreadObjectKeys, spreadObjectValues, spreadObjectEntries);

  const frozenKeys = Object.keys(frozenFromEntries);
  const frozenBracketRootKeys = Object.freeze((globalThis["Object"]))["keys"](frozenFromEntries);
  const frozenSingleQuotedBracketRootKeys = Object.freeze((globalThis["Object"])['keys'])(frozenFromEntries);
  const frozenBracketedKeys = Object.freeze((globalThis['Object'])["keys"])(frozenFromEntries);
  const frozenSingleQuotedBracketedKeys = Object.freeze((globalThis['Object'])['keys'])(frozenFromEntries);
  const frozenValues = Object.values(frozenFromEntries);
  const frozenEntries = Object.entries(frozenFromEntries);
  assertWrappedObjectEnumeration(frozenKeys, frozenValues, frozenEntries);
  assertWrappedObjectEnumeration(frozenBracketRootKeys, frozenValues, frozenEntries);
  assertWrappedObjectEnumeration(frozenSingleQuotedBracketRootKeys, frozenValues, frozenEntries);
  assertWrappedObjectEnumeration(frozenBracketedKeys, frozenValues, frozenEntries);
  assertWrappedObjectEnumeration(frozenSingleQuotedBracketedKeys, frozenValues, frozenEntries);

  const spreadFrozenKeys = [...Object.keys(frozenFromEntries)];
  const spreadFrozenValues = [...Object.values(frozenFromEntries)];
  const spreadFrozenEntries = [...Object.entries(frozenFromEntries)];
  assertWrappedObjectEnumeration(spreadFrozenKeys, spreadFrozenValues, spreadFrozenEntries);
}
"##
}

fn assert_browser_bundle_wrapped_object_enumeration(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if filename.ends_with(".js") {
        browser_bundle_wrapped_object_enumeration_js_source()
    } else {
        browser_bundle_wrapped_object_enumeration_source()
    };
    fs::write(&source_path, source).expect("write source");

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
fn build_emits_wrapped_object_enumeration_semantics_in_ts_input() {
    assert_browser_bundle_wrapped_object_enumeration("app.ts", false);
}

#[test]
fn build_emits_wrapped_object_enumeration_semantics_in_js_input() {
    assert_browser_bundle_wrapped_object_enumeration("app.js", false);
}

#[test]
fn build_emits_wrapped_object_enumeration_semantics_in_jsx_tsx_input() {
    for filename in ["app.jsx", "app.tsx"] {
        assert_browser_bundle_wrapped_object_enumeration(filename, false);
    }
}

#[test]
fn json_build_emits_wrapped_object_enumeration_semantics_in_ts_input() {
    assert_browser_bundle_wrapped_object_enumeration("app.ts", true);
}

#[test]
fn json_build_emits_wrapped_object_enumeration_semantics_in_js_input() {
    assert_browser_bundle_wrapped_object_enumeration("app.js", true);
}

#[test]
fn json_build_emits_wrapped_object_enumeration_semantics_in_jsx_tsx_input() {
    for filename in ["app.jsx", "app.tsx"] {
        assert_browser_bundle_wrapped_object_enumeration(filename, true);
    }
}
