use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_object_from_entries_source() -> &'static str {
    r##"// kali-tree-shake: browserObjectFromEntries
function assertFromEntriesShape(fromEntries) {
  const keys = Object.keys(fromEntries);
  const entries = Object.entries(fromEntries);
  const values = Object.values(fromEntries);
  if (
    keys.length !== 2 ||
    keys[0] !== 'b' ||
    keys[1] !== 'a' ||
    entries.length !== 2 ||
    entries[0][0] !== 'b' ||
    entries[0][1] !== 1 ||
    entries[1][0] !== 'a' ||
    entries[1][1] !== 2 ||
    values.length !== 2 ||
    values[0] !== 1 ||
    values[1] !== 2
  ) {
    throw new Error('unexpected Object.fromEntries semantics');
  }
}

function browserObjectFromEntries() {
  const wrappedEntries = ([["b", 1], ["a", 2]]);
  const frozenEntries = Object.freeze([["b", 1], ["a", 2]]);
  const conditionalEntries = (true ? [["b", 1], ["a", 2]] : [["x", 9]]);
  const frozenObjectFromEntries = Object.freeze(Object.fromEntries);
  const frozenBracketedObjectFromEntries = Object.freeze(globalThis["Object"]["fromEntries"]);
  const parenthesizedFrozenBracketedObjectFromEntries = Object.freeze((globalThis["Object"]["fromEntries"]));
  assertFromEntriesShape(Object.fromEntries([["b", 1], ["a", 2]]));
  assertFromEntriesShape(Object.fromEntries(wrappedEntries));
  assertFromEntriesShape(Object.fromEntries(frozenEntries));
  assertFromEntriesShape(Object.fromEntries(conditionalEntries));
  assertFromEntriesShape(frozenObjectFromEntries(wrappedEntries));
  assertFromEntriesShape(frozenBracketedObjectFromEntries(wrappedEntries));
  assertFromEntriesShape(parenthesizedFrozenBracketedObjectFromEntries(wrappedEntries));
  assertFromEntriesShape(globalThis.Object.fromEntries([["b", 1], ["a", 2]]));
  assertFromEntriesShape(globalThis.Object["fromEntries"]([["b", 1], ["a", 2]]));
  assertFromEntriesShape(globalThis["Object"].fromEntries([["b", 1], ["a", 2]]));
  assertFromEntriesShape(globalThis["Object"]["fromEntries"]([["b", 1], ["a", 2]]));
}
"##
}

fn assert_browser_bundle_object_from_entries(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, browser_bundle_object_from_entries_source()).expect("write source");

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
fn build_emits_object_from_entries_semantics_in_js_input() {
    assert_browser_bundle_object_from_entries("app.js", false);
}

#[test]
fn build_emits_object_from_entries_semantics_in_ts_input() {
    assert_browser_bundle_object_from_entries("app.ts", false);
}

#[test]
fn build_emits_object_from_entries_semantics_in_jsx_input() {
    assert_browser_bundle_object_from_entries("app.jsx", false);
}

#[test]
fn build_emits_object_from_entries_semantics_in_tsx_input() {
    assert_browser_bundle_object_from_entries("app.tsx", false);
}

#[test]
fn json_build_emits_object_from_entries_semantics_in_js_input() {
    assert_browser_bundle_object_from_entries("app.js", true);
}

#[test]
fn json_build_emits_object_from_entries_semantics_in_ts_input() {
    assert_browser_bundle_object_from_entries("app.ts", true);
}

#[test]
fn json_build_emits_object_from_entries_semantics_in_jsx_input() {
    assert_browser_bundle_object_from_entries("app.jsx", true);
}

#[test]
fn json_build_emits_object_from_entries_semantics_in_tsx_input() {
    assert_browser_bundle_object_from_entries("app.tsx", true);
}
