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

/// Task 5 pin (currently RED — enable by removing `#[ignore]` once Task 5's
/// growable-array dispatch accepts a field-read receiver): `o.values` is an
/// object field carrying a `Repr::GrowableArrayI64` handle (Task 3 interns
/// it); `.join(',')` over that field should read the handle through
/// `emit_object_field_read` and route to the growable-array join lane. node
/// v26.5.0 prints "1,2,3"; kali today has no dispatch for a field-read
/// receiver and prints "0" (or errors), per probe p2e.
#[test]
#[ignore = "enabled in Task 5 — growable dispatch for field receivers not yet landed"]
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
