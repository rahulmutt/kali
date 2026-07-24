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
fn var_object_bool_field_reads_value() {
    // R-06 Boolean-field residual (task review 2026-07-24): kali has no
    // first-class runtime Boolean repr axis at all (`var b = true;
    // console.log(b);` — NO object involved — already prints "1", not
    // "true"), so a materialized object field can NEVER read a Boolean
    // field back correctly. Read-materializing this binding would turn the
    // pre-fix silent-`0` into a NEW nonzero-wrong value (silent `1`) —
    // exactly what R-06 must never introduce. It fails closed (E5506)
    // instead: honest over-deny beats a new silent-wrong value. A
    // write-materialized Boolean field's pre-existing silent-`1` behavior
    // (`var o={f:false}; o.f=true; console.log(o.f)` -> "1", unchanged) is
    // a separate, out-of-scope, pre-existing bug this fix deliberately
    // leaves untouched.
    run_e5506("var o = { f: true }; console.log(o.f);");
}

#[test]
fn var_object_multi_field_reads_all() {
    assert_eq!(
        run_ok("var o = { a: 1, b: 2, c: 3 }; console.log(o.a + o.b + o.c);"),
        "6"
    );
}

#[test]
#[ignore] // R-06-R4 (root cause corrected on task review 2026-07-24): this is
          // a pre-existing multi-arg `console.log` SINK bug, not a
          // field-repr gap. A single-arg `console.log(o.s)` reads a String
          // field correctly ("hi"), which disproves "object fields have no
          // Repr::String axis" — the field itself is fine. The corruption
          // is specific to a String field being a NON-SOLE argument of a
          // multi-arg `console.log` call: `const o={n:7,s:"hi"};
          // console.log(o.n,o.s)` corrupts IDENTICALLY, and `const` never
          // materializes / never touches R-06 at all, proving this is a
          // pre-existing downstream sink bug outside R-06's scope. R-06
          // merely routes the read-only var object to the same
          // already-broken sink. See task-1-report.md.
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
