use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_direct_object_values_iteration_source() -> &'static str {
    r##"// kali-tree-shake: browserDirectObjectValuesIteration
function assertObjectValuesIteration(values) {
  if (values.length !== 2 || values[0] !== 1 || values[1] !== 2) {
    throw new Error('unexpected Object.values iteration semantics');
  }
}

function browserDirectObjectValuesIteration() {
  const seen = [];
  for (const value of Object.values({ "b": 1, "a": 2 })) {
    seen.push(value);
  }
  const mixed = [];
  for (const value of globalThis.Object["values"]({ "b": 1, "a": 2 })) {
    mixed.push(value);
  }
  const mixedBracketed = [];
  for (const value of globalThis["Object"].values({ "b": 1, "a": 2 })) {
    mixedBracketed.push(value);
  }
  const singleQuotedProperty = [];
  for (const value of globalThis['Object'].values({ "b": 1, "a": 2 })) {
    singleQuotedProperty.push(value);
  }
  const doubleQuotedSingleQuoted = [];
  for (const value of globalThis["Object"]['values']({ "b": 1, "a": 2 })) {
    doubleQuotedSingleQuoted.push(value);
  }
  const mixedSingleQuotedBracketed = [];
  for (const value of globalThis['Object']["values"]({ "b": 1, "a": 2 })) {
    mixedSingleQuotedBracketed.push(value);
  }
  const mixedSingleQuoted = [];
  for (const value of globalThis['Object']['values']({ "b": 1, "a": 2 })) {
    mixedSingleQuoted.push(value);
  }
  const bracketed = [];
  for (const value of globalThis["Object"]["values"]({ "b": 1, "a": 2 })) {
    bracketed.push(value);
  }
  const frozenBracketRootValues = Object.freeze((globalThis["Object"]))["values"]({ "b": 1, "a": 2 });
  const frozenSingleQuotedReceiverBracketedValues = Object.freeze((globalThis['Object'])["values"])({ "b": 1, "a": 2 });
  const frozenSingleQuotedReceiverPropertyValues = Object.freeze((globalThis['Object']).values)({ "b": 1, "a": 2 });
  assertObjectValuesIteration(seen);
  assertObjectValuesIteration(frozenBracketRootValues);
  assertObjectValuesIteration(frozenSingleQuotedReceiverBracketedValues);
  assertObjectValuesIteration(frozenSingleQuotedReceiverPropertyValues);
  assertObjectValuesIteration(singleQuotedProperty);
  assertObjectValuesIteration(mixed);
  assertObjectValuesIteration(mixedBracketed);
  assertObjectValuesIteration(doubleQuotedSingleQuoted);
  assertObjectValuesIteration(mixedSingleQuotedBracketed);
  assertObjectValuesIteration(mixedSingleQuoted);
  assertObjectValuesIteration(bracketed);
}
"##
}

fn browser_bundle_global_object_values_iteration_source() -> &'static str {
    r##"// kali-tree-shake: browserGlobalObjectValuesIteration
function assertObjectValuesIteration(values) {
  if (values.length !== 2 || values[0] !== 1 || values[1] !== 2) {
    throw new Error('unexpected Object.values iteration semantics');
  }
}

function browserGlobalObjectValuesIteration() {
  const values = { "b": 1, "a": 2 };
  const alias = values;
  const seen = [];
  for (const value of globalThis.Object.values(alias)) {
    seen.push(value);
  }
  const mixed = [];
  for (const value of globalThis.Object["values"](alias)) {
    mixed.push(value);
  }
  const mixedBracketed = [];
  for (const value of globalThis["Object"].values(alias)) {
    mixedBracketed.push(value);
  }
  const doubleQuotedSingleQuoted = [];
  for (const value of globalThis["Object"]['values'](alias)) {
    doubleQuotedSingleQuoted.push(value);
  }
  const bracketed = [];
  for (const value of globalThis["Object"]["values"](alias)) {
    bracketed.push(value);
  }
  const parenthesizedReceiverBracketed = [];
  for (const value of Object.freeze((globalThis["Object"])["values"])(alias)) {
    parenthesizedReceiverBracketed.push(value);
  }
  const parenthesizedSingleQuotedReceiverBracketed = [];
  for (const value of Object.freeze((globalThis['Object'])["values"])(alias)) {
    parenthesizedSingleQuotedReceiverBracketed.push(value);
  }
  const parenthesizedSingleQuotedReceiverBracketedProperty = [];
  for (const value of Object.freeze((globalThis['Object']).values)(alias)) {
    parenthesizedSingleQuotedReceiverBracketedProperty.push(value);
  }
  const parenthesizedBracketed = [];
  for (const value of Object.freeze((globalThis["Object"]).values)(alias)) {
    parenthesizedBracketed.push(value);
  }
  const frozenBracketRootValues = Object.freeze((globalThis["Object"]))["values"](alias);
  assertObjectValuesIteration(seen);
  assertObjectValuesIteration(frozenBracketRootValues);
  assertObjectValuesIteration(mixed);
  assertObjectValuesIteration(mixedBracketed);
  assertObjectValuesIteration(doubleQuotedSingleQuoted);
  assertObjectValuesIteration(bracketed);
  assertObjectValuesIteration(parenthesizedReceiverBracketed);
  assertObjectValuesIteration(parenthesizedSingleQuotedReceiverBracketed);
  assertObjectValuesIteration(parenthesizedSingleQuotedReceiverBracketedProperty);
  assertObjectValuesIteration(parenthesizedBracketed);
}
"##
}

fn assert_browser_bundle_direct_object_values_iteration(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_direct_object_values_iteration_source(),
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

fn assert_browser_bundle_global_object_values_iteration(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_global_object_values_iteration_source(),
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
fn build_emits_global_object_values_iteration_semantics_in_js_input() {
    assert_browser_bundle_global_object_values_iteration("app.js", false);
}

#[test]
fn build_emits_global_object_values_iteration_semantics_in_ts_jsx_tsx_input() {
    for filename in ["app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle_global_object_values_iteration(filename, false);
    }
}

#[test]
fn json_build_emits_global_object_values_iteration_semantics_in_js_input() {
    assert_browser_bundle_global_object_values_iteration("app.js", true);
}

#[test]
fn json_build_emits_global_object_values_iteration_semantics_in_ts_jsx_tsx_input() {
    for filename in ["app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle_global_object_values_iteration(filename, true);
    }
}

#[test]
fn build_emits_direct_object_values_iteration_semantics_in_js_ts_jsx_tsx_input() {
    for filename in ["app.js", "app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle_direct_object_values_iteration(filename, false);
    }
}

#[test]
fn json_build_emits_direct_object_values_iteration_semantics_in_js_ts_jsx_tsx_input() {
    for filename in ["app.js", "app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle_direct_object_values_iteration(filename, true);
    }
}
