//! Stage P2 Lane 1 (structuredClone deep-clone lane) — Task 4: field-read
//! produces a growable-array handle.
//!
//! `object_field_is_growable_array` (crates/kali_codegen/src/emit/object.rs)
//! lets downstream growable-array dispatch (push/join/length/index/for-of,
//! Task 5) accept a `base.field` receiver, not only a named binding. This
//! file's first test pins the pre-Task-5 (still-fail-closed) behavior: a
//! `.join` call on an object field that holds a growable i64 array has no
//! dispatch yet, so kali does not print the joined string. The test is
//! `#[ignore]`d here (a deliberate deviation from the brief — see the Task 4
//! report) so the workspace gate stays 0-newly-red; Task 5 removes the
//! `#[ignore]` once the dispatch lands.

use std::{fs, process::Command};
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_kali(source: &str) -> std::process::Output {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.js");
    fs::write(&path, source).expect("write source");
    Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali")
}

/// Run `kali run`, assert it succeeded, and return stdout (caller trims).
fn run_kali_run(source: &str) -> String {
    let out = run_kali(source);
    assert!(
        out.status.success(),
        "expected success; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Run `kali run` expecting a fail-closed compile (nonzero exit); return stderr
/// so the caller can assert the diagnostic code (E5506).
fn run_kali_run_expect_error(source: &str) -> String {
    let out = run_kali(source);
    assert!(
        !out.status.success(),
        "expected a fail-closed compile (nonzero exit), got success; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// Task 5 pin (currently RED — enable by removing `#[ignore]` once Task 5's
/// growable-array dispatch accepts a field-read receiver): `o.values` is an
/// object field carrying a `Repr::GrowableArrayI64` handle (Task 3 interns
/// it); `.join(',')` over that field should read the handle through
/// `emit_object_field_read` and route to the growable-array join lane. node
/// v26.5.0 prints "1,2,3"; kali today has no dispatch for a field-read
/// receiver and prints "0" (or errors), per probe p2e.
#[test]
fn object_array_field_read_only_join_round_trips() {
    let src = "const o = { count: 1, values: [1, 2, 3] };\n\
               console.log(o.values.join(','));\n";
    let out = run_kali(src);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.trim(),
        "1,2,3",
        "expected node-equivalent output; stdout: {stdout:?}, stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Task 5: `.push` / `.join` / `.length` over a `GrowableArrayI64` object field.
///
/// DEVIATION FROM THE BRIEF (documented): the brief's body used a single
/// MULTI-argument `console.log(o.count, o.values.join(','), o.values.length)`.
/// Runtime multi-argument `console.log` where an argument reads a growable
/// array is a pre-existing UNSUPPORTED shape that fails closed E5506 by an
/// established Stage 4 soundness contract (the dynamic console lane prints a
/// single value; a green pin —
/// `growable_array_fail_closed::multi_arg_console_log_with_growable_read_fails_closed`
/// — asserts that fail-close). Enabling multi-arg console would turn that pin
/// newly-red, so this task keeps it. The growable-field push/join/length lane
/// itself is what Task 5 delivers; it is exercised here via single-argument
/// `console.log` (the supported surface), asserting the SAME node-equivalent
/// values. See the Task 5 report for the full multi-arg-console analysis.
#[test]
fn object_array_field_push_join_length_round_trip() {
    let src = "const o = { count: 1, values: [1, 2, 3] };\n\
               o.values.push(4);\n\
               console.log(o.count);\n\
               console.log(o.values.join(','));\n\
               console.log(o.values.length);\n";
    let out = run_kali_run(src);
    assert_eq!(out.trim(), "1\n1,2,3,4\n4"); // node: 1 / 1,2,3,4 / 4
}

/// Task 5: index read `o.values[i]` and `for (const x of o.values)` over a
/// `GrowableArrayI64` object field. (Single-argument `console.log` per value —
/// see `object_array_field_push_join_length_round_trip` for why the brief's
/// multi-arg form is not used.)
#[test]
fn object_array_field_index_and_for_of() {
    let src = "const o = { values: [10, 20, 30] };\n\
               let s = 0;\n\
               for (const x of o.values) { s += x; }\n\
               console.log(o.values[1]);\n\
               console.log(s);\n";
    let out = run_kali_run(src);
    assert_eq!(out.trim(), "20\n60");
}

/// Task 5 soundness pin: a growable-array FIELD read inside a MULTI-argument
/// `console.log` fails closed E5506 (not a silent argument drop) — the
/// field twin of the named-binding
/// `multi_arg_console_log_with_growable_read_fails_closed` contract. This is
/// why the two tests above log each value on its own line.
#[test]
fn multi_arg_console_with_growable_field_fails_closed() {
    let src = "const o = { count: 1, values: [1, 2, 3] };\n\
               console.log(o.count, o.values.length);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

/// Task 5 Lane 1 tripwire: a STRING-element array field must NOT dispatch
/// through the i64 growable lane — Task 3 conflicts a string array field to
/// E5506 (fail closed), never a silent miscompile.
#[test]
fn structured_clone_string_array_field_fails_closed() {
    let src = "const o = { vals: ['a', 'b'] };\n\
               o.vals.push('c');\n\
               console.log(o.vals.length);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}
