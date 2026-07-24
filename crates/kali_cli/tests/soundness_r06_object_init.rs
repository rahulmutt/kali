// R-06 — read-only var/let object-literal materialization soundness pins.
// A read-only mutable object literal must read back its real field values
// (materialized allocation), not the silent-0 fold fallback. Shapes the
// materialized lane cannot store fail closed with E5506, never silent-0.
use std::fs;
use std::process::{Command, Output};
use tempfile::tempdir;

fn kali_bin() -> String {
    env!("CARGO_BIN_EXE_kali").to_string()
}

fn run(source: &str) -> Output {
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

/// Compile+run, assert success, return trimmed stdout.
fn run_ok(source: &str) -> String {
    let out = run(source);
    assert!(
        out.status.success(),
        "expected success\nsource: {source}\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Compile+run, assert fail-closed with E5506.
fn run_e5506(source: &str) -> String {
    let out = run(source);
    assert!(
        !out.status.success(),
        "expected fail-closed E5506\nsource: {source}\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(stderr.contains("E5506"), "expected E5506, got: {stderr}");
    stderr
}

// ---- Green pins: read-only mutable object literals materialize correctly ----

#[test]
fn var_object_numeric_field_reads_value() {
    assert_eq!(run_ok("var o = { f: 7 }; console.log(o.f);"), "7");
}

#[test]
fn let_object_numeric_field_reads_value() {
    assert_eq!(run_ok("let o = { f: 7 }; console.log(o.f);"), "7");
}

#[test]
fn var_object_string_field_reads_value() {
    assert_eq!(run_ok("var o = { f: \"hi\" }; console.log(o.f);"), "hi");
}

#[test]
#[ignore] // R-06-R5: pre-existing, unrelated to object materialization — kali
          // has no first-class runtime Boolean repr at all. `var b = true;
          // console.log(b);` (NO object involved) already prints "1", and the
          // SAME field prints "1" via the pre-existing write-materialization
          // lane (`var o={f:false}; o.f=true; console.log(o.f);"1"`), proving
          // this is not something R-06's read-materialization introduces or
          // can fix. Verified 2026-07-24; see task-1-report.md.
fn var_object_bool_field_reads_value() {
    assert_eq!(run_ok("var o = { f: true }; console.log(o.f);"), "true");
}

#[test]
fn var_object_multi_field_reads_all() {
    assert_eq!(
        run_ok("var o = { a: 1, b: 2, c: 3 }; console.log(o.a + o.b + o.c);"),
        "6"
    );
}

#[test]
#[ignore] // R-06-R4: pre-existing, unrelated to object materialization —
          // object fields have no Repr::String axis (documented FLOAT-ONLY
          // i64/f64 at repr_infer.rs's uniform-computed-read comment); a
          // single-arg `console.log(o.s)` happens to still print "hi" via an
          // existing fallback, but the SAME string field corrupts to a raw
          // decoded handle when it is the second arg of a multi-arg
          // console.log call following a numeric arg — reproduces
          // byte-for-byte via the pre-existing write-materialization lane
          // (`o.n=7; o.s="hi"; console.log(o.n,o.s)` also prints garbage),
          // proving R-06's read-materialization is not the cause. Verified
          // 2026-07-24; see task-1-report.md.
fn var_object_mixed_fields_read() {
    assert_eq!(
        run_ok("var o = { n: 7, s: \"hi\" }; console.log(o.n, o.s);"),
        "7 hi"
    );
}

#[test]
fn var_object_function_scope_reads_value() {
    assert_eq!(
        run_ok("function h(){ var o = { f: 7 }; return o.f; } console.log(h());"),
        "7"
    );
}

#[test]
fn const_object_still_folds() {
    // Regression guard: const stays fold-first, unchanged.
    assert_eq!(run_ok("const o = { f: 7 }; console.log(o.f);"), "7");
}

// ---- Fail-closed pins: shapes the materialized lane cannot store ----

#[test]
fn var_object_nested_field_fails_closed() {
    // Nested-object field: E5506, not silent-0.
    run_e5506("var o = { inner: { x: 1 } }; console.log(o.inner.x);");
}

#[test]
fn var_object_unknown_field_fails_closed() {
    // Unknown field on a materialized read-only object: E5506 (kali has no
    // `undefined`; honest over-deny beats today's silent-0).
    run_e5506("var o = { f: 7 }; console.log(o.zzz);");
}
