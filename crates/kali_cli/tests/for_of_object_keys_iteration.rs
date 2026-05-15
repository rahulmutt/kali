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

fn object_keys_from_entries_iteration_run_source() -> &'static str {
    r#"const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);
const keys = [];
for (const key of Object.keys(fromEntries)) {
  keys.push(key);
}
if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {
  throw new Error('unexpected Object.keys(Object.fromEntries(...)) iteration semantics');
}
"#
}

fn object_keys_from_entries_iteration_test_source() -> &'static str {
    r#"Kali.test('object keys fromEntries iteration', () => {
  const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);
  const keys = [];
  for (const key of Object.keys(fromEntries)) {
    keys.push(key);
  }
  if (keys.length !== 2 || keys[0] !== 'b' || keys[1] !== 'a') {
    throw new Error('unexpected Object.keys(Object.fromEntries(...)) iteration semantics');
  }
});
"#
}

fn object_values_from_entries_iteration_run_source() -> &'static str {
    r#"const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);
const seen = [];
for (const value of Object.values(fromEntries)) {
  seen.push(value);
}
if (seen.length !== 2 || seen[0] !== 3 || seen[1] !== 2) {
  throw new Error('unexpected Object.values(Object.fromEntries(...)) iteration semantics');
}
"#
}

fn object_values_from_entries_iteration_test_source() -> &'static str {
    r#"Kali.test('object values fromEntries iteration', () => {
  const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);
  const seen = [];
  for (const value of Object.values(fromEntries)) {
    seen.push(value);
  }
  if (seen.length !== 2 || seen[0] !== 3 || seen[1] !== 2) {
    throw new Error('unexpected Object.values(Object.fromEntries(...)) iteration semantics');
  }
});
"#
}

fn object_entries_from_entries_iteration_run_source() -> &'static str {
    r#"const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);
const seen = [];
for (const entry of Object.entries(fromEntries)) {
  seen.push(entry[0]);
  seen.push(entry[1]);
}
if (seen.length !== 4 || seen[0] !== 'b' || seen[1] !== 3 || seen[2] !== 'a' || seen[3] !== 2) {
  throw new Error('unexpected Object.entries(Object.fromEntries(...)) iteration semantics');
}
"#
}

fn object_entries_from_entries_iteration_test_source() -> &'static str {
    r#"Kali.test('object entries fromEntries iteration', () => {
  const fromEntries = Object.fromEntries([["b", 1], ["a", 2], ["b", 3]]);
  const seen = [];
  for (const entry of Object.entries(fromEntries)) {
    seen.push(entry[0]);
    seen.push(entry[1]);
  }
  if (seen.length !== 4 || seen[0] !== 'b' || seen[1] !== 3 || seen[2] !== 'a' || seen[3] !== 2) {
    throw new Error('unexpected Object.entries(Object.fromEntries(...)) iteration semantics');
  }
});
"#
}

fn object_enumeration_frozen_literal_run_source() -> &'static str {
    r#"const keys = [];
for (const key of Object.keys(Object.freeze({ a: 1, b: 2 }))) {
  keys.push(key);
}
if (keys.length !== 2 || keys[0] !== 'a' || keys[1] !== 'b') {
  throw new Error('unexpected frozen object enumeration semantics');
}
"#
}

fn object_enumeration_frozen_literal_test_source() -> &'static str {
    r#"Kali.test('frozen object enumeration', () => {
  const keys = [];
  for (const key of Object.keys(Object.freeze({ a: 1, b: 2 }))) {
    keys.push(key);
  }
  if (keys.length !== 2 || keys[0] !== 'a' || keys[1] !== 'b') {
    throw new Error('unexpected frozen object enumeration semantics');
  }
});
"#
}

fn object_values_iteration_frozen_literal_run_source() -> &'static str {
    r#"const seen = [];
for (const value of Object.values(Object.freeze({ "b": 1, "a": 2 }))) {
  seen.push(value);
}
if (seen.length !== 2 || seen[0] !== 1 || seen[1] !== 2) {
  throw new Error('unexpected frozen Object.values iteration semantics');
}
"#
}

fn object_values_iteration_frozen_literal_test_source() -> &'static str {
    r#"Kali.test('frozen object values iteration', () => {
  const seen = [];
  for (const value of Object.values(Object.freeze({ "b": 1, "a": 2 }))) {
    seen.push(value);
  }
  if (seen.length !== 2 || seen[0] !== 1 || seen[1] !== 2) {
    throw new Error('unexpected frozen Object.values iteration semantics');
  }
});
"#
}

fn object_string_enumeration_run_source() -> &'static str {
    r#"const seen = [];
for (const value of 'ab') {
  seen.push(value);
}
if (seen.length !== 2 || seen[0] !== 'a' || seen[1] !== 'b') {
  throw new Error('unexpected string iteration semantics');
}
"#
}

fn object_string_enumeration_test_source() -> &'static str {
    r#"Kali.test('string iteration', () => {
  const seen = [];
  for (const value of 'ab') {
    seen.push(value);
  }
  if (seen.length !== 2 || seen[0] !== 'a' || seen[1] !== 'b') {
    throw new Error('unexpected string iteration semantics');
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

#[test]
fn run_supports_object_keys_from_entries_iteration_in_js_input() {
    assert_object_keys_iteration(
        "run",
        "main.js",
        object_keys_from_entries_iteration_run_source(),
    );
}

#[test]
fn run_supports_object_keys_from_entries_iteration_in_ts_input() {
    assert_object_keys_iteration(
        "run",
        "main.ts",
        object_keys_from_entries_iteration_run_source(),
    );
}

#[test]
fn test_supports_object_keys_from_entries_iteration_in_js_input() {
    assert_object_keys_iteration(
        "test",
        "smoke.test.js",
        object_keys_from_entries_iteration_test_source(),
    );
}

#[test]
fn test_supports_object_keys_from_entries_iteration_in_ts_input() {
    assert_object_keys_iteration(
        "test",
        "smoke.test.ts",
        object_keys_from_entries_iteration_test_source(),
    );
}

#[test]
fn run_supports_object_values_from_entries_iteration_in_js_input() {
    assert_object_keys_iteration(
        "run",
        "main.js",
        object_values_from_entries_iteration_run_source(),
    );
}

#[test]
fn run_supports_object_values_from_entries_iteration_in_ts_input() {
    assert_object_keys_iteration(
        "run",
        "main.ts",
        object_values_from_entries_iteration_run_source(),
    );
}

#[test]
fn test_supports_object_values_from_entries_iteration_in_js_input() {
    assert_object_keys_iteration(
        "test",
        "smoke.test.js",
        object_values_from_entries_iteration_test_source(),
    );
}

#[test]
fn test_supports_object_values_from_entries_iteration_in_ts_input() {
    assert_object_keys_iteration(
        "test",
        "smoke.test.ts",
        object_values_from_entries_iteration_test_source(),
    );
}

#[test]
fn run_supports_object_entries_from_entries_iteration_in_js_input() {
    assert_object_keys_iteration(
        "run",
        "main.js",
        object_entries_from_entries_iteration_run_source(),
    );
}

#[test]
fn run_supports_object_entries_from_entries_iteration_in_ts_input() {
    assert_object_keys_iteration(
        "run",
        "main.ts",
        object_entries_from_entries_iteration_run_source(),
    );
}

#[test]
fn test_supports_object_entries_from_entries_iteration_in_js_input() {
    assert_object_keys_iteration(
        "test",
        "smoke.test.js",
        object_entries_from_entries_iteration_test_source(),
    );
}

#[test]
fn test_supports_object_entries_from_entries_iteration_in_ts_input() {
    assert_object_keys_iteration(
        "test",
        "smoke.test.ts",
        object_entries_from_entries_iteration_test_source(),
    );
}

#[test]
fn run_supports_object_keys_iteration_in_jsx_and_tsx_input() {
    for filename in ["main.jsx", "main.tsx"] {
        assert_object_keys_iteration("run", filename, object_keys_iteration_run_source());
    }
}

#[test]
fn test_supports_object_keys_iteration_in_jsx_and_tsx_input() {
    for filename in ["smoke.test.jsx", "smoke.test.tsx"] {
        assert_object_keys_iteration("test", filename, object_keys_iteration_test_source());
    }
}

#[test]
fn run_supports_direct_object_keys_iteration_in_jsx_and_tsx_input() {
    for filename in ["main.jsx", "main.tsx"] {
        assert_object_keys_iteration("run", filename, object_keys_iteration_direct_run_source());
    }
}

#[test]
fn test_supports_direct_object_keys_iteration_in_jsx_and_tsx_input() {
    for filename in ["smoke.test.jsx", "smoke.test.tsx"] {
        assert_object_keys_iteration("test", filename, object_keys_iteration_direct_test_source());
    }
}

#[test]
fn run_supports_global_object_keys_iteration_in_jsx_and_tsx_input() {
    for filename in ["main.jsx", "main.tsx"] {
        assert_object_keys_iteration("run", filename, global_object_keys_iteration_run_source());
    }
}

#[test]
fn test_supports_global_object_keys_iteration_in_jsx_and_tsx_input() {
    for filename in ["smoke.test.jsx", "smoke.test.tsx"] {
        assert_object_keys_iteration("test", filename, global_object_keys_iteration_test_source());
    }
}

#[test]
fn run_supports_object_values_iteration_in_jsx_and_tsx_input() {
    for filename in ["main.jsx", "main.tsx"] {
        assert_object_keys_iteration("run", filename, object_values_iteration_run_source());
    }
}

#[test]
fn test_supports_object_values_iteration_in_jsx_and_tsx_input() {
    for filename in ["smoke.test.jsx", "smoke.test.tsx"] {
        assert_object_keys_iteration("test", filename, object_values_iteration_test_source());
    }
}

#[test]
fn run_supports_object_keys_from_entries_iteration_in_jsx_and_tsx_input() {
    for filename in ["main.jsx", "main.tsx"] {
        assert_object_keys_iteration(
            "run",
            filename,
            object_keys_from_entries_iteration_run_source(),
        );
    }
}

#[test]
fn test_supports_object_keys_from_entries_iteration_in_jsx_and_tsx_input() {
    for filename in ["smoke.test.jsx", "smoke.test.tsx"] {
        assert_object_keys_iteration(
            "test",
            filename,
            object_keys_from_entries_iteration_test_source(),
        );
    }
}

#[test]
fn run_supports_object_values_from_entries_iteration_in_jsx_and_tsx_input() {
    for filename in ["main.jsx", "main.tsx"] {
        assert_object_keys_iteration(
            "run",
            filename,
            object_values_from_entries_iteration_run_source(),
        );
    }
}

#[test]
fn test_supports_object_values_from_entries_iteration_in_jsx_and_tsx_input() {
    for filename in ["smoke.test.jsx", "smoke.test.tsx"] {
        assert_object_keys_iteration(
            "test",
            filename,
            object_values_from_entries_iteration_test_source(),
        );
    }
}

#[test]
fn run_supports_object_entries_from_entries_iteration_in_jsx_and_tsx_input() {
    for filename in ["main.jsx", "main.tsx"] {
        assert_object_keys_iteration(
            "run",
            filename,
            object_entries_from_entries_iteration_run_source(),
        );
    }
}

#[test]
fn test_supports_object_entries_from_entries_iteration_in_jsx_and_tsx_input() {
    for filename in ["smoke.test.jsx", "smoke.test.tsx"] {
        assert_object_keys_iteration(
            "test",
            filename,
            object_entries_from_entries_iteration_test_source(),
        );
    }
}

#[test]
fn run_supports_frozen_object_enumeration_iteration_in_js_ts_jsx_tsx_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_object_keys_iteration(
            "run",
            filename,
            object_enumeration_frozen_literal_run_source(),
        );
    }
}

#[test]
fn test_supports_frozen_object_enumeration_iteration_in_js_ts_jsx_tsx_input() {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_object_keys_iteration(
            "test",
            filename,
            object_enumeration_frozen_literal_test_source(),
        );
    }
}

#[test]
fn run_supports_frozen_object_values_iteration_in_js_ts_jsx_tsx_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_object_keys_iteration(
            "run",
            filename,
            object_values_iteration_frozen_literal_run_source(),
        );
    }
}

#[test]
fn test_supports_frozen_object_values_iteration_in_js_ts_jsx_tsx_input() {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_object_keys_iteration(
            "test",
            filename,
            object_values_iteration_frozen_literal_test_source(),
        );
    }
}

#[test]
fn run_supports_object_string_enumeration_iteration_in_js_ts_jsx_tsx_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_object_keys_iteration("run", filename, object_string_enumeration_run_source());
    }
}

#[test]
fn test_supports_object_string_enumeration_iteration_in_js_ts_jsx_tsx_input() {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_object_keys_iteration("test", filename, object_string_enumeration_test_source());
    }
}

fn object_keys_break_continue_run_source() -> &'static str {
    r#"const values = { "b": 1, "a": 2 };
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
"#
}

fn object_keys_break_continue_test_source() -> &'static str {
    r#"Kali.test('object keys break/continue iteration', () => {
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
});
"#
}

#[test]
fn run_supports_object_keys_break_continue_iteration_in_js_ts_jsx_tsx_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_object_keys_iteration("run", filename, object_keys_break_continue_run_source());
    }
}

#[test]
fn test_supports_object_keys_break_continue_iteration_in_js_ts_jsx_tsx_input() {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_object_keys_iteration("test", filename, object_keys_break_continue_test_source());
    }
}
