use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_object_entries_iteration_source() -> &'static str {
    r##"// kali-tree-shake: browserObjectEntriesIteration
function assertObjectEntriesIteration(entries) {
  if (
    entries.length !== 2 ||
    entries[0][0] !== 'b' ||
    entries[0][1] !== 1 ||
    entries[1][0] !== 'a' ||
    entries[1][1] !== 2
  ) {
    throw new Error('unexpected Object.entries iteration semantics');
  }
}

function browserObjectEntriesIteration() {
  const values = { "b": 1, "a": 2 };
  const alias = values;
  const entries = Object.entries(alias);
  const frozenEntries = Object.freeze(Object.entries)(alias);
  const frozenGlobalEntries = Object.freeze(globalThis.Object.entries)(alias);
  const frozenBracketedEntries = Object.freeze(globalThis["Object"]["entries"])(alias);
  const frozenBracketRootEntries = Object.freeze((globalThis["Object"]))["entries"](alias);
  const frozenParenthesizedBracketedEntries = Object.freeze((globalThis["Object"]).entries)(alias);
  const frozenParenthesizedDotRootEntries = Object.freeze((globalThis.Object).entries)(alias);
  const mixedEntries = globalThis.Object["entries"](alias);
  const mixedBracketedEntries = globalThis["Object"].entries(alias);
  const parenthesizedReceiverBracketedEntries = (globalThis["Object"])["entries"](alias);
  const parenthesizedSingleQuotedReceiverBracketedEntries = (globalThis['Object'])["entries"](alias);
  const frozenParenthesizedReceiverBracketedEntries = Object.freeze((globalThis["Object"])["entries"])(alias);
  const frozenParenthesizedSingleQuotedReceiverBracketedEntries = Object.freeze((globalThis['Object'])["entries"])(alias);
  const frozenParenthesizedSingleQuotedReceiverBracketedPropertyEntries = Object.freeze((globalThis['Object']).entries)(alias);
  const singleQuotedProperty = [];
  for (const entry of globalThis['Object'].entries(alias)) {
    singleQuotedProperty.push(entry);
  }
  const doubleQuotedSingleQuotedEntries = globalThis["Object"]['entries'](alias);
  const mixedSingleQuotedBracketedEntries = globalThis['Object']["entries"](alias);
  const mixedSingleQuotedEntries = globalThis['Object']['entries'](alias);
  const bracketedEntries = globalThis["Object"]["entries"](alias);
  assertObjectEntriesIteration(entries);
  assertObjectEntriesIteration(frozenEntries);
  assertObjectEntriesIteration(frozenGlobalEntries);
  assertObjectEntriesIteration(frozenBracketedEntries);
  assertObjectEntriesIteration(frozenBracketRootEntries);
  assertObjectEntriesIteration(frozenParenthesizedBracketedEntries);
  assertObjectEntriesIteration(frozenParenthesizedDotRootEntries);
  assertObjectEntriesIteration(mixedEntries);
  assertObjectEntriesIteration(mixedBracketedEntries);
  assertObjectEntriesIteration(parenthesizedReceiverBracketedEntries);
  assertObjectEntriesIteration(parenthesizedSingleQuotedReceiverBracketedEntries);
  assertObjectEntriesIteration(frozenParenthesizedReceiverBracketedEntries);
  assertObjectEntriesIteration(frozenParenthesizedSingleQuotedReceiverBracketedEntries);
  assertObjectEntriesIteration(frozenParenthesizedSingleQuotedReceiverBracketedPropertyEntries);
  assertObjectEntriesIteration(singleQuotedProperty);
  assertObjectEntriesIteration(doubleQuotedSingleQuotedEntries);
  assertObjectEntriesIteration(mixedSingleQuotedBracketedEntries);
  assertObjectEntriesIteration(mixedSingleQuotedEntries);
  assertObjectEntriesIteration(bracketedEntries);
}
"##
}

fn browser_bundle_direct_object_entries_iteration_source() -> &'static str {
    r##"// kali-tree-shake: browserDirectObjectEntriesIteration
function assertObjectEntriesIteration(entries) {
  if (
    entries.length !== 2 ||
    entries[0][0] !== 'b' ||
    entries[0][1] !== 1 ||
    entries[1][0] !== 'a' ||
    entries[1][1] !== 2
  ) {
    throw new Error('unexpected Object.entries iteration semantics');
  }
}

function browserDirectObjectEntriesIteration() {
  const entries = Object.entries({ "b": 1, "a": 2 });
  const frozenEntries = Object.freeze(Object.entries)({ "b": 1, "a": 2 });
  const frozenGlobalEntries = Object.freeze(globalThis.Object.entries)({ "b": 1, "a": 2 });
  const frozenBracketedEntries = Object.freeze(globalThis["Object"]["entries"])({ "b": 1, "a": 2 });
  const frozenBracketRootEntries = Object.freeze((globalThis["Object"]))["entries"]({ "b": 1, "a": 2 });
  const frozenParenthesizedBracketedEntries = Object.freeze((globalThis["Object"]).entries)({ "b": 1, "a": 2 });
  const mixedEntries = globalThis.Object["entries"]({ "b": 1, "a": 2 });
  const mixedBracketedEntries = globalThis["Object"].entries({ "b": 1, "a": 2 });
  const doubleQuotedSingleQuotedEntries = globalThis["Object"]['entries']({ "b": 1, "a": 2 });
  const mixedSingleQuotedBracketedEntries = globalThis['Object']["entries"]({ "b": 1, "a": 2 });
  const mixedSingleQuotedEntries = globalThis['Object']['entries']({ "b": 1, "a": 2 });
  const bracketedEntries = globalThis["Object"]["entries"]({ "b": 1, "a": 2 });
  assertObjectEntriesIteration(entries);
  assertObjectEntriesIteration(frozenEntries);
  assertObjectEntriesIteration(frozenGlobalEntries);
  assertObjectEntriesIteration(frozenBracketedEntries);
  assertObjectEntriesIteration(frozenBracketRootEntries);
  assertObjectEntriesIteration(frozenParenthesizedBracketedEntries);
  assertObjectEntriesIteration(mixedEntries);
  assertObjectEntriesIteration(mixedBracketedEntries);
  assertObjectEntriesIteration(doubleQuotedSingleQuotedEntries);
  assertObjectEntriesIteration(mixedSingleQuotedBracketedEntries);
  assertObjectEntriesIteration(mixedSingleQuotedEntries);
  assertObjectEntriesIteration(bracketedEntries);
}
"##
}

fn browser_bundle_global_object_entries_iteration_source() -> &'static str {
    r##"// kali-tree-shake: browserGlobalObjectEntriesIteration
function assertObjectEntriesIteration(entries) {
  if (
    entries.length !== 2 ||
    entries[0][0] !== 'b' ||
    entries[0][1] !== 1 ||
    entries[1][0] !== 'a' ||
    entries[1][1] !== 2
  ) {
    throw new Error('unexpected Object.entries iteration semantics');
  }
}

function browserGlobalObjectEntriesIteration() {
  const values = { "b": 1, "a": 2 };
  const alias = values;
  const entries = globalThis.Object.entries(alias);
  const frozenEntries = Object.freeze(globalThis.Object.entries)(alias);
  const frozenBracketedEntries = Object.freeze(globalThis["Object"]["entries"])(alias);
  const frozenBracketRootEntries = Object.freeze((globalThis["Object"]))["entries"](alias);
  const frozenParenthesizedBracketedEntries = Object.freeze((globalThis["Object"]).entries)(alias);
  const mixedEntries = globalThis.Object["entries"](alias);
  const mixedBracketedEntries = globalThis["Object"].entries(alias);
  const parenthesizedSingleQuotedReceiverBracketedEntries = Object.freeze((globalThis['Object'])["entries"])(alias);
  const bracketed = globalThis["Object"]["entries"](alias);
  assertObjectEntriesIteration(entries);
  assertObjectEntriesIteration(frozenEntries);
  assertObjectEntriesIteration(frozenBracketedEntries);
  assertObjectEntriesIteration(frozenBracketRootEntries);
  assertObjectEntriesIteration(frozenParenthesizedBracketedEntries);
  assertObjectEntriesIteration(mixedEntries);
  assertObjectEntriesIteration(parenthesizedSingleQuotedReceiverBracketedEntries);
  assertObjectEntriesIteration(mixedBracketedEntries);
  assertObjectEntriesIteration(bracketed);
}
"##
}

fn assert_browser_bundle_object_entries_iteration(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_object_entries_iteration_source(),
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

    // Honest re-pin (PR #16 rev2, family `object-enum`): kali fails closed/loud here
    // (4 of this helper's 8 worklist callers were tagged class B by the automated
    // classifier, but direct verification shows every one of them panics on this
    // exact assertion too — a loud E5506 build-time rejection, not a silent wrong
    // value; re-pinned as class A for all 8 callers — see
    // docs/superpowers/followups/pr16-honest-repin-inventory.md).
    assert!(!output.status.success(), "must fail closed: {output:?}");
}

fn assert_browser_bundle_direct_object_entries_iteration(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_direct_object_entries_iteration_source(),
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

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if json_output {
        let envelope: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(envelope["schemaVersion"], 1);
        assert_eq!(envelope["command"], "build");
        assert_eq!(envelope["success"], true);
        assert_eq!(envelope["exitCode"], 0);
        assert!(envelope["errors"]
            .as_array()
            .expect("errors array")
            .is_empty());
    }

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_eq!(metadata["apiSurface"], "browser");
    assert_eq!(metadata["artifactKind"], "bundle");

    let harness_path = bundle_dir
        .parent()
        .expect("bundle root parent")
        .join("browser-bundle-smoke.mjs");
    let harness = kali_runtime::browser_bundle_harness_script(
        "app",
        false,
        r#"const mod = await import(bundleJs.href);
await mod.browserDirectObjectEntriesIteration();
"#,
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = kali_runtime::browser_harness_command_parts_for(
        std::env::var("KALI_BROWSER_BUNDLE_HARNESS_COMMAND")
            .ok()
            .as_deref(),
    );
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(&bundle_dir)
        .args(&harness_command)
        .arg(&harness_path)
        .output()
        .expect("run browser bundle harness");

    // Honest re-pin (PR #16 rev2, family `object-enum`): the build step succeeds
    // (its own success assert above holds honestly); kali fails closed/loud only
    // at browser-bundle execution here (a runtime trap surfacing the fixture's own
    // assertion throw) — this helper's 2 worklist callers are both class A — see
    // docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!output.status.success(), "must fail closed: {output:?}");
}

fn assert_browser_bundle_global_object_entries_iteration(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_global_object_entries_iteration_source(),
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

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if json_output {
        let envelope: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(envelope["schemaVersion"], 1);
        assert_eq!(envelope["command"], "build");
        assert_eq!(envelope["success"], true);
        assert_eq!(envelope["exitCode"], 0);
        assert!(envelope["errors"]
            .as_array()
            .expect("errors array")
            .is_empty());
    }

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_eq!(metadata["apiSurface"], "browser");
    assert_eq!(metadata["artifactKind"], "bundle");

    let harness_path = bundle_dir
        .parent()
        .expect("bundle root parent")
        .join("browser-bundle-smoke.mjs");
    let harness = kali_runtime::browser_bundle_harness_script(
        "app",
        false,
        r#"const mod = await import(bundleJs.href);
await mod.browserGlobalObjectEntriesIteration();
"#,
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = kali_runtime::browser_harness_command_parts_for(
        std::env::var("KALI_BROWSER_BUNDLE_HARNESS_COMMAND")
            .ok()
            .as_deref(),
    );
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(&bundle_dir)
        .args(&harness_command)
        .arg(&harness_path)
        .output()
        .expect("run browser bundle harness");

    // Honest re-pin (PR #16 rev2, family `object-enum`): the build step succeeds
    // (its own success assert above holds honestly); kali fails closed/loud only
    // at browser-bundle execution here (a runtime trap surfacing the fixture's own
    // assertion throw) — this helper's 8 worklist callers are all class A — see
    // docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!output.status.success(), "must fail closed: {output:?}");
}

#[test]
fn build_emits_object_entries_iteration_semantics_in_js_input() {
    assert_browser_bundle_object_entries_iteration("app.js", false);
}

#[test]
fn build_emits_object_entries_iteration_semantics_in_ts_input() {
    assert_browser_bundle_object_entries_iteration("app.ts", false);
}

#[test]
fn build_emits_object_entries_iteration_semantics_in_jsx_input() {
    assert_browser_bundle_object_entries_iteration("app.jsx", false);
}

#[test]
fn build_emits_object_entries_iteration_semantics_in_tsx_input() {
    assert_browser_bundle_object_entries_iteration("app.tsx", false);
}

#[test]
fn json_build_emits_object_entries_iteration_semantics_in_js_input() {
    assert_browser_bundle_object_entries_iteration("app.js", true);
}

#[test]
fn json_build_emits_object_entries_iteration_semantics_in_ts_input() {
    assert_browser_bundle_object_entries_iteration("app.ts", true);
}

#[test]
fn json_build_emits_object_entries_iteration_semantics_in_jsx_input() {
    assert_browser_bundle_object_entries_iteration("app.jsx", true);
}

#[test]
fn json_build_emits_object_entries_iteration_semantics_in_tsx_input() {
    assert_browser_bundle_object_entries_iteration("app.tsx", true);
}

#[test]
fn build_emits_direct_object_entries_iteration_semantics_in_js_ts_jsx_tsx_input() {
    for filename in ["app.js", "app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle_direct_object_entries_iteration(filename, false);
    }
}

#[test]
fn json_build_emits_direct_object_entries_iteration_semantics_in_js_ts_jsx_tsx_input() {
    for filename in ["app.js", "app.ts", "app.jsx", "app.tsx"] {
        assert_browser_bundle_direct_object_entries_iteration(filename, true);
    }
}

#[test]
fn build_emits_global_object_entries_iteration_semantics_in_js_input() {
    assert_browser_bundle_global_object_entries_iteration("app.js", false);
}

#[test]
fn build_emits_global_object_entries_iteration_semantics_in_ts_input() {
    assert_browser_bundle_global_object_entries_iteration("app.ts", false);
}

#[test]
fn build_emits_global_object_entries_iteration_semantics_in_jsx_input() {
    assert_browser_bundle_global_object_entries_iteration("app.jsx", false);
}

#[test]
fn build_emits_global_object_entries_iteration_semantics_in_tsx_input() {
    assert_browser_bundle_global_object_entries_iteration("app.tsx", false);
}

#[test]
fn json_build_emits_global_object_entries_iteration_semantics_in_js_input() {
    assert_browser_bundle_global_object_entries_iteration("app.js", true);
}

#[test]
fn json_build_emits_global_object_entries_iteration_semantics_in_ts_input() {
    assert_browser_bundle_global_object_entries_iteration("app.ts", true);
}

#[test]
fn json_build_emits_global_object_entries_iteration_semantics_in_jsx_input() {
    assert_browser_bundle_global_object_entries_iteration("app.jsx", true);
}

#[test]
fn json_build_emits_global_object_entries_iteration_semantics_in_tsx_input() {
    assert_browser_bundle_global_object_entries_iteration("app.tsx", true);
}
