//! Regression tests for the const-bound-arrow return-escape miscompile
//! (baseline classes 2/4/5): declaring an arrow whose body contains a return
//! (explicit, or the implicit return synthesized for an expression body) used
//! to emit a real wasm `return` into the ENCLOSING function, silently
//! truncating execution with exit 0. Arrows must instead compile as
//! standalone wasm functions — the lane named functions and unnamed function
//! expressions already ride.

use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(filename: &str, source: &str) -> std::process::Output {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali")
}

fn assert_run_stdout(filename: &str, source: &str, expected_stdout: &str) {
    let output = run_source(filename, source);
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected_stdout,
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn run_executes_statements_after_const_expression_bodied_arrow_declaration() {
    assert_run_stdout(
        "decl-only.js",
        "console.log(\"A\");\nconst f = (x) => x;\nconsole.log(\"B\");\n",
        "A\nB\n",
    );
}

#[test]
fn run_calls_const_expression_bodied_arrow_via_binding() {
    assert_run_stdout(
        "call-arrow.js",
        "const h = (x) => x + 1;\nconsole.log(h(41));\nconsole.log(\"after\");\n",
        "42\nafter\n",
    );
}

#[test]
fn run_executes_block_bodied_arrow_body_at_call_time_not_declaration() {
    // Class-2 shape: the trailing argument must be evaluated (printing "bump")
    // at the Math.atan2 call, then the fold result 0 prints, then execution
    // continues. node ground truth: bump, 0, after.
    assert_run_stdout(
        "block-arrow.js",
        "const bump = () => { console.log(\"bump\"); return 2; };\nconsole.log(Math.atan2(0, 1, bump()));\nconsole.log(\"after\");\n",
        "bump\n0\nafter\n",
    );
}

#[test]
fn run_object_enumeration_survives_const_arrow_preamble() {
    // Class-4/5 shape: the consumeArray declaration/calls must not truncate
    // the top-level enumeration logs. node ground truth: 2.
    assert_run_stdout(
        "enum-preamble.js",
        r#"const obj = { "a": 1, "b": 2 };
const keys = Object.keys(obj);
const consumeArray = (items, value) => items[0] + items[1] + value;
const arrayLiteralFirst = consumeArray([1n, 2n], 1n);
console.log(keys.length);
"#,
        "2\n",
    );
}
