use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn object_keys_iteration_run_source() -> &'static str {
    r#"const values = { "b": 1, "a": 2 };
const alias = values;
const keys = [];
for (const key of Object.keys(alias)) {
  keys.push(key);
}
if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {
  throw new Error('unexpected Object.keys iteration semantics');
}
"#
}

fn object_keys_iteration_direct_run_source() -> &'static str {
    r#"const keys = [];
for (const key of Object.keys({ "b": 1, "a": 2 })) {
  keys.push(key);
}
if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {
  throw new Error('unexpected Object.keys iteration semantics');
}
"#
}

fn object_keys_iteration_test_source() -> &'static str {
    r#"Kali.test('object keys iteration', () => {
  const values = { "b": 1, "a": 2 };
  const alias = values;
  const keys = [];
  for (const key of Object.keys(alias)) {
    keys.push(key);
  }
  if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {
    throw new Error('unexpected Object.keys iteration semantics');
  }
});
"#
}

fn object_keys_iteration_direct_test_source() -> &'static str {
    r#"Kali.test('object keys iteration', () => {
  const keys = [];
  for (const key of Object.keys({ "b": 1, "a": 2 })) {
    keys.push(key);
  }
  if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {
    throw new Error('unexpected Object.keys iteration semantics');
  }
});
"#
}

fn global_object_keys_iteration_run_source() -> &'static str {
    r#"const values = { "b": 1, "a": 2 };
const alias = values;
const keys = [];
for (const key of globalThis.Object.keys(alias)) {
  keys.push(key);
}
if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {
  throw new Error('unexpected Object.keys iteration semantics');
}
"#
}

fn global_object_keys_iteration_test_source() -> &'static str {
    r#"Kali.test('object keys iteration', () => {
  const values = { "b": 1, "a": 2 };
  const alias = values;
  const keys = [];
  for (const key of globalThis.Object.keys(alias)) {
    keys.push(key);
  }
  if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {
    throw new Error('unexpected Object.keys iteration semantics');
  }
});
"#
}

fn object_values_iteration_run_source() -> &'static str {
    r#"const values = { "b": 1, "a": 2 };
const alias = values;
const seen = [];
for (const value of Object.values(alias)) {
  seen.push(value);
}
if (seen.length !== 2 || seen[0] !== 1 || seen[1] !== 2) {
  throw new Error('unexpected Object.values iteration semantics');
}
"#
}

fn object_values_iteration_test_source() -> &'static str {
    r#"Kali.test('object values iteration', () => {
  const values = { "b": 1, "a": 2 };
  const alias = values;
  const seen = [];
  for (const value of Object.values(alias)) {
    seen.push(value);
  }
  if (seen.length !== 2 || seen[0] !== 1 || seen[1] !== 2) {
    throw new Error('unexpected Object.values iteration semantics');
  }
});
"#
}

fn assert_object_keys_iteration(command: &str, filename: &str, source: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg(command)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if command == "run" {
        assert!(
            String::from_utf8_lossy(&output.stdout).is_empty(),
            "stdout: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
}

#[test]
fn run_supports_object_keys_iteration_in_js_input() {
    assert_object_keys_iteration("run", "main.js", object_keys_iteration_run_source());
}

#[test]
fn run_supports_object_keys_iteration_in_ts_input() {
    assert_object_keys_iteration("run", "main.ts", object_keys_iteration_run_source());
}

#[test]
fn test_supports_object_keys_iteration_in_js_input() {
    assert_object_keys_iteration("test", "smoke.test.js", object_keys_iteration_test_source());
}

#[test]
fn test_supports_object_keys_iteration_in_ts_input() {
    assert_object_keys_iteration("test", "smoke.test.ts", object_keys_iteration_test_source());
}

#[test]
fn run_supports_object_keys_iteration_with_direct_literal_object_in_js_input() {
    assert_object_keys_iteration("run", "main.js", object_keys_iteration_direct_run_source());
}

#[test]
fn run_supports_object_keys_iteration_with_direct_literal_object_in_ts_input() {
    assert_object_keys_iteration("run", "main.ts", object_keys_iteration_direct_run_source());
}

#[test]
fn test_supports_object_keys_iteration_with_direct_literal_object_in_js_input() {
    assert_object_keys_iteration(
        "test",
        "smoke.test.js",
        object_keys_iteration_direct_test_source(),
    );
}

#[test]
fn test_supports_object_keys_iteration_with_direct_literal_object_in_ts_input() {
    assert_object_keys_iteration(
        "test",
        "smoke.test.ts",
        object_keys_iteration_direct_test_source(),
    );
}

#[test]
fn run_supports_global_object_keys_iteration_in_js_input() {
    assert_object_keys_iteration("run", "main.js", global_object_keys_iteration_run_source());
}

#[test]
fn run_supports_global_object_keys_iteration_in_ts_jsx_tsx_input() {
    for filename in ["main.ts", "main.jsx", "main.tsx"] {
        assert_object_keys_iteration("run", filename, global_object_keys_iteration_run_source());
    }
}

#[test]
fn test_supports_global_object_keys_iteration_in_js_input() {
    assert_object_keys_iteration(
        "test",
        "smoke.test.js",
        global_object_keys_iteration_test_source(),
    );
}

#[test]
fn test_supports_global_object_keys_iteration_in_ts_jsx_tsx_input() {
    for filename in ["smoke.test.ts", "smoke.test.jsx", "smoke.test.tsx"] {
        assert_object_keys_iteration("test", filename, global_object_keys_iteration_test_source());
    }
}

#[test]
fn run_supports_object_values_iteration_in_js_input() {
    assert_object_keys_iteration("run", "main.js", object_values_iteration_run_source());
}

#[test]
fn run_supports_object_values_iteration_in_ts_input() {
    assert_object_keys_iteration("run", "main.ts", object_values_iteration_run_source());
}

#[test]
fn test_supports_object_values_iteration_in_js_input() {
    assert_object_keys_iteration(
        "test",
        "smoke.test.js",
        object_values_iteration_test_source(),
    );
}

#[test]
fn test_supports_object_values_iteration_in_ts_input() {
    assert_object_keys_iteration(
        "test",
        "smoke.test.ts",
        object_values_iteration_test_source(),
    );
}
