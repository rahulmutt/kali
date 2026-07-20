use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_object_keys_iteration_source() -> &'static str {
    r##"// kali-tree-shake: browserObjectKeysIteration
function assertObjectKeysIteration(keys) {
  if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {
    throw new Error('unexpected Object.keys iteration semantics');
  }
}

function browserObjectKeysIteration() {
  const values = { "b": 1, "a": 2 };
  const alias = values;
  const keys = [];
  for (const key of Object.keys(alias)) {
    keys.push(key);
  }
  const mixedBracketed = [];
  for (const key of globalThis["Object"].keys(alias)) {
    mixedBracketed.push(key);
  }
  const singleQuotedProperty = [];
  for (const key of globalThis['Object'].keys(alias)) {
    singleQuotedProperty.push(key);
  }
  const doubleQuotedSingleQuoted = [];
  for (const key of globalThis["Object"]['keys'](alias)) {
    doubleQuotedSingleQuoted.push(key);
  }
  const mixedSingleQuoted = [];
  for (const key of globalThis['Object']["keys"](alias)) {
    mixedSingleQuoted.push(key);
  }
  const parenthesizedReceiverBracketed = [];
  for (const key of Object.freeze((globalThis["Object"])["keys"])(alias)) {
    parenthesizedReceiverBracketed.push(key);
  }
  const frozenSingleQuotedReceiverBracketedKeys = Object.freeze((globalThis['Object'])["keys"])(alias);
  const parenthesizedSingleQuotedReceiverBracketed = [];
  for (const key of Object.freeze((globalThis['Object'])["keys"])(alias)) {
    parenthesizedSingleQuotedReceiverBracketed.push(key);
  }
  const parenthesizedSingleQuotedReceiverBracketedProperty = [];
  for (const key of Object.freeze((globalThis['Object']).keys)(alias)) {
    parenthesizedSingleQuotedReceiverBracketedProperty.push(key);
  }
  const parenthesizedBracketed = [];
  for (const key of Object.freeze((globalThis["Object"]).keys)(alias)) {
    parenthesizedBracketed.push(key);
  }
  assertObjectKeysIteration(mixedBracketed);
  assertObjectKeysIteration(singleQuotedProperty);
  assertObjectKeysIteration(doubleQuotedSingleQuoted);
  assertObjectKeysIteration(mixedSingleQuoted);
  assertObjectKeysIteration(parenthesizedReceiverBracketed);
  assertObjectKeysIteration(frozenSingleQuotedReceiverBracketedKeys);
  assertObjectKeysIteration(parenthesizedSingleQuotedReceiverBracketed);
  assertObjectKeysIteration(parenthesizedSingleQuotedReceiverBracketedProperty);
  assertObjectKeysIteration(parenthesizedBracketed);
  assertObjectKeysIteration(keys);
}
"##
}

fn browser_bundle_direct_object_keys_iteration_source() -> &'static str {
    r##"// kali-tree-shake: browserDirectObjectKeysIteration
function assertObjectKeysIteration(keys) {
  if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {
    throw new Error('unexpected Object.keys iteration semantics');
  }
}

function browserDirectObjectKeysIteration() {
  const keys = [];
  for (const key of Object.keys({ "b": 1, "a": 2 })) {
    keys.push(key);
  }
  assertObjectKeysIteration(keys);
}
"##
}

fn browser_bundle_global_object_keys_iteration_source() -> &'static str {
    r##"// kali-tree-shake: browserGlobalObjectKeysIteration
function assertObjectKeysIteration(keys) {
  if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {
    throw new Error('unexpected Object.keys iteration semantics');
  }
}

function browserGlobalObjectKeysIteration() {
  const values = { "b": 1, "a": 2 };
  const alias = values;
  const keys = [];
  for (const key of globalThis.Object.keys(alias)) {
    keys.push(key);
  }
  const mixed = [];
  for (const key of globalThis.Object["keys"](alias)) {
    mixed.push(key);
  }
  const mixedBracketed = [];
  for (const key of globalThis["Object"].keys(alias)) {
    mixedBracketed.push(key);
  }
  const doubleQuotedSingleQuoted = [];
  for (const key of globalThis["Object"]['keys'](alias)) {
    doubleQuotedSingleQuoted.push(key);
  }
  const bracketed = [];
  for (const key of globalThis["Object"]["keys"](alias)) {
    bracketed.push(key);
  }
  const mixedSingleQuoted = [];
  for (const key of globalThis['Object']["keys"](alias)) {
    mixedSingleQuoted.push(key);
  }
  const mixedSingleQuotedBracketed = [];
  for (const key of globalThis['Object']['keys'](alias)) {
    mixedSingleQuotedBracketed.push(key);
  }
  const parenthesizedReceiverBracketed = [];
  for (const key of Object.freeze((globalThis["Object"])["keys"])(alias)) {
    parenthesizedReceiverBracketed.push(key);
  }
  const parenthesizedSingleQuotedReceiverBracketed = [];
  for (const key of Object.freeze((globalThis['Object'])["keys"])(alias)) {
    parenthesizedSingleQuotedReceiverBracketed.push(key);
  }
  const parenthesizedSingleQuotedReceiverBracketedProperty = [];
  for (const key of Object.freeze((globalThis['Object']).keys)(alias)) {
    parenthesizedSingleQuotedReceiverBracketedProperty.push(key);
  }
  const parenthesizedBracketed = [];
  for (const key of Object.freeze((globalThis["Object"]).keys)(alias)) {
    parenthesizedBracketed.push(key);
  }
  assertObjectKeysIteration(keys);
  assertObjectKeysIteration(mixed);
  assertObjectKeysIteration(mixedBracketed);
  assertObjectKeysIteration(doubleQuotedSingleQuoted);
  assertObjectKeysIteration(bracketed);
  assertObjectKeysIteration(mixedSingleQuoted);
  assertObjectKeysIteration(mixedSingleQuotedBracketed);
  assertObjectKeysIteration(parenthesizedReceiverBracketed);
  assertObjectKeysIteration(parenthesizedSingleQuotedReceiverBracketed);
  assertObjectKeysIteration(parenthesizedSingleQuotedReceiverBracketedProperty);
  assertObjectKeysIteration(parenthesizedBracketed);
}
"##
}
fn browser_bundle_await_wrapped_static_object_helpers_source() -> &'static str {
    r##"// kali-tree-shake: browserAwaitWrappedStaticObjectHelpers
async function browserAwaitWrappedStaticObjectHelpers() {
  if (!Object.hasOwn((0, { "a": 1 }), 'a')) {
    throw new Error('unexpected sequence-wrapped Object.hasOwn semantics');
  }
  const keys = Object.keys(await { "b": 1, "a": 2 });
  if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {
    throw new Error('unexpected await-wrapped Object.keys iteration semantics');
  }
  const sequencedKeys = Object.keys((0, { "b": 1, "a": 2 }));
  if (sequencedKeys.length !== 2 || sequencedKeys[0] !== 'b' || sequencedKeys[1] !== 'a') {
    throw new Error('unexpected sequence-wrapped Object.keys iteration semantics');
  }
  const ownKeys = Reflect.ownKeys(await Object.freeze({ "b": 1, "a": 2 }));
  if (ownKeys.length !== 2 || ownKeys[0] !== 'b' || ownKeys[1] !== 'a') {
    throw new Error('unexpected await-wrapped Reflect.ownKeys iteration semantics');
  }
  const sequencedOwnKeys = Reflect.ownKeys((0, Object.freeze({ "b": 1, "a": 2 })));
  if (sequencedOwnKeys.length !== 2 || sequencedOwnKeys[0] !== 'b' || sequencedOwnKeys[1] !== 'a') {
    throw new Error('unexpected sequence-wrapped Reflect.ownKeys iteration semantics');
  }
  console.log('browser await-wrapped static object helpers ok');
}
"##
}

fn browser_bundle_const_bound_object_keys_iteration_source() -> &'static str {
    r##"// kali-tree-shake: browserConstBoundObjectKeysIteration
function assertObjectKeysIteration(keys) {
  if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {
    throw new Error('unexpected Object.keys iteration semantics');
  }
}

function browserConstBoundObjectKeysIteration() {
  const values = { "b": 1, "a": 2 };
  const keys = [];
  for (const key of Object.keys(values)) {
    keys.push(key);
  }
  assertObjectKeysIteration(keys);
}
"##
}

fn browser_bundle_object_values_iteration_source() -> &'static str {
    r##"// kali-tree-shake: browserObjectValuesIteration
function assertObjectValuesIteration(values) {
  if (values.length !== 2 || values[0] !== 1 || values[1] !== 2) {
    throw new Error('unexpected Object.values iteration semantics');
  }
}

function browserObjectValuesIteration() {
  const values = { "b": 1, "a": 2 };
  const alias = values;
  const seen = [];
  for (const value of Object.values(alias)) {
    seen.push(value);
  }
  assertObjectValuesIteration(seen);
}
"##
}

fn assert_browser_bundle_object_keys_iteration(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, browser_bundle_object_keys_iteration_source()).expect("write source");

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

fn assert_browser_bundle_direct_object_keys_iteration(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_direct_object_keys_iteration_source(),
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

fn assert_browser_bundle_await_wrapped_static_object_helpers(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_await_wrapped_static_object_helpers_source(),
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

fn browser_bundle_object_keys_break_continue_iteration_source() -> &'static str {
    r##"// kali-tree-shake: browserObjectKeysBreakContinueIteration
function browserObjectKeysBreakContinueIteration() {
  const values = { "b": 1, "a": 2 };
  const alias = values;
  const keys = [];
  for (const key of Object.keys(alias)) {
    if (key === 'b') {
      continue;
    }
    keys.push(key);
    break;
  }
  if (keys.length !== 1 || keys[0] !== 'a') {
    throw new Error('unexpected Object.keys break/continue iteration semantics');
  }
}
"##
}

#[path = "browser_object_keys_iteration/build_json.rs"]
mod build_json;

#[path = "browser_object_keys_iteration/build.rs"]
mod build;
