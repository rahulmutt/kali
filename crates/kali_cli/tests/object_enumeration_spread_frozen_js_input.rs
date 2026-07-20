use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn frozen_object_enumeration_spread_source() -> &'static str {
    r#"const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);
const frozenFromEntries = Object.freeze(fromEntries);
const frozenKeys = [...Object.keys(frozenFromEntries)];
const frozenValues = [...Object.values(frozenFromEntries)];
const frozenFromEntriesValues = [...Object.freeze(Object.values(fromEntries))];
const frozenEntries = [...Object.entries(frozenFromEntries)];
if (
  frozenKeys.length !== 2 ||
  frozenKeys[0] !== 'b' ||
  frozenKeys[1] !== 'a' ||
  frozenValues.length !== 2 ||
  frozenValues[0] !== 3 ||
  frozenValues[1] !== 2 ||
  frozenFromEntriesValues.length !== 2 ||
  frozenFromEntriesValues[0] !== 3 ||
  frozenFromEntriesValues[1] !== 2 ||
  frozenEntries.length !== 2 ||
  frozenEntries[0][0] !== 'b' ||
  frozenEntries[0][1] !== 3 ||
  frozenEntries[1][0] !== 'a' ||
  frozenEntries[1][1] !== 2
) {
  throw new Error('unexpected frozen object enumeration spread semantics');
}
for (const value of frozenValues) {
  console.log(value);
}
for (const key of frozenKeys) {
  console.log(key);
}
for (const entry of frozenEntries) {
  console.log(entry[0]);
  console.log(entry[1]);
}
console.log('frozen object enumeration spread ok');
"#
}

fn frozen_object_enumeration_spread_test_source() -> &'static str {
    r#"Kali.test('frozen object enumeration spread', () => {
  const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);
  const frozenFromEntries = Object.freeze(fromEntries);
  const frozenKeys = [...Object.keys(frozenFromEntries)];
  const frozenValues = [...Object.values(frozenFromEntries)];
  const frozenFromEntriesValues = [...Object.freeze(Object.values(fromEntries))];
  const frozenEntries = [...Object.entries(frozenFromEntries)];
  if (
    frozenKeys.length !== 2 ||
    frozenKeys[0] !== 'b' ||
    frozenKeys[1] !== 'a' ||
    frozenValues.length !== 2 ||
    frozenValues[0] !== 3 ||
    frozenValues[1] !== 2 ||
    frozenFromEntriesValues.length !== 2 ||
    frozenFromEntriesValues[0] !== 3 ||
    frozenFromEntriesValues[1] !== 2 ||
    frozenEntries.length !== 2 ||
    frozenEntries[0][0] !== 'b' ||
    frozenEntries[0][1] !== 3 ||
    frozenEntries[1][0] !== 'a' ||
    frozenEntries[1][1] !== 2
  ) {
    throw new Error('unexpected frozen object enumeration spread semantics');
  }
});
"#
}

fn assert_frozen_object_enumeration_spread(command: &str, filename: &str, source: &'static str) {
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
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "must fail closed: {output:?}");
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn run_accepts_frozen_object_enumeration_spread_in_js_input() {
    assert_frozen_object_enumeration_spread(
        "run",
        "main.js",
        frozen_object_enumeration_spread_source(),
    );
}

#[test]
fn run_accepts_frozen_object_enumeration_spread_in_ts_input() {
    assert_frozen_object_enumeration_spread(
        "run",
        "main.ts",
        frozen_object_enumeration_spread_source(),
    );
}

#[test]
fn json_run_accepts_frozen_object_enumeration_spread_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, frozen_object_enumeration_spread_source()).expect("write source");

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
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["success"], false);
    assert!(
        json["errors"]
            .as_array()
            .expect("errors array")
            .iter()
            .any(|e| e["code"] == "E5506"),
        "json: {json}"
    );
}

#[test]
fn json_run_accepts_frozen_object_enumeration_spread_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    fs::write(&source_path, frozen_object_enumeration_spread_source()).expect("write source");

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
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["success"], false);
    assert!(
        json["errors"]
            .as_array()
            .expect("errors array")
            .iter()
            .any(|e| e["code"] == "E5506"),
        "json: {json}"
    );
}

#[test]
fn test_accepts_frozen_object_enumeration_spread_in_js_input() {
    assert_frozen_object_enumeration_spread(
        "test",
        "main.test.js",
        frozen_object_enumeration_spread_test_source(),
    );
}

#[test]
fn test_accepts_frozen_object_enumeration_spread_in_ts_input() {
    assert_frozen_object_enumeration_spread(
        "test",
        "main.test.ts",
        frozen_object_enumeration_spread_test_source(),
    );
}

#[test]
fn json_test_accepts_frozen_object_enumeration_spread_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.js");
    fs::write(&source_path, frozen_object_enumeration_spread_test_source()).expect("write source");

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
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["success"], false);
    assert!(
        json["errors"]
            .as_array()
            .expect("errors array")
            .iter()
            .any(|e| e["code"] == "E5506"),
        "json: {json}"
    );
}

#[test]
fn json_test_accepts_frozen_object_enumeration_spread_in_ts_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.test.ts");
    fs::write(&source_path, frozen_object_enumeration_spread_test_source()).expect("write source");

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
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["success"], false);
    assert!(
        json["errors"]
            .as_array()
            .expect("errors array")
            .iter()
            .any(|e| e["code"] == "E5506"),
        "json: {json}"
    );
}
