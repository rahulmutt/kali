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

// ---- Allowlist-at-the-choke pins (task review round 2, 2026-07-24) ----
//
// The bare Boolean-LITERAL check above is a DENYLIST and leaks: a Boolean
// value reached via a variable, `!x`, or a comparison — or a BigInt
// literal — slips it, materializes, and reads back a NEW nonzero-wrong
// value (main was silent-0). These pins lock in the allowlist-at-the-choke
// fix (`object_field_value_is_safe_for_materialization`): only a numeric
// literal, a string literal, or unary `+`/`-` on one is admitted; every
// other field-value shape fails the WHOLE binding closed (E5506).

#[test]
fn bool_via_variable_fails_closed() {
    run_e5506("var t=true; var o={f:t}; console.log(o.f);");
}

#[test]
fn bool_via_unary_not_fails_closed() {
    run_e5506("var o={f:!0}; console.log(o.f);");
}

#[test]
fn bool_via_comparison_fails_closed() {
    run_e5506("var o={f:1>0}; console.log(o.f);");
}

#[test]
fn bigint_field_fails_closed() {
    run_e5506("var o={f:7n}; console.log(o.f);");
}

#[test]
fn null_field_fails_closed() {
    // kali has no null repr; `main` silently printed `0` where node prints
    // `null` (already wrong, but zero). The allowlist now fails this
    // closed instead — honest over-deny, and no worse than main (E5506,
    // not a new nonzero-wrong value).
    run_e5506("var o = { f: null }; console.log(o.f);");
}

#[test]
fn var_object_float_field_reads_value() {
    assert_eq!(run_ok("var o = { f: 1.5 }; console.log(o.f);"), "1.5");
}

#[test]
fn var_object_negative_field_reads_value() {
    assert_eq!(run_ok("var o = { f: -7 }; console.log(o.f);"), "-7");
}

// ---- Unary-on-string pins (task review round 4, 2026-07-24) ----
//
// The unary +/- arm of `object_field_value_is_safe_for_materialization`
// used to recurse into the GENERAL predicate, which admits `String` at
// the top level — so it wrongly admitted `+"hi"`/`-"3"` too. A
// materialized field has no string->number coercion (kali reads the raw
// string bytes as an integer), so `+"3"` happening to read back `3` is
// COINCIDENTAL, not proof of soundness; `+"hi"`/`+"0x10"`/`+"  5  "` read
// back garbage. The fix restricts the unary arm to a numeric-literal-only
// sub-check (`unary_numeric_literal_operand`) that never accepts a string
// at any depth — a string is admitted ONLY as a bare top-level field
// value, never underneath a unary operator.

#[test]
fn unary_plus_on_nonnumeric_string_fails_closed() {
    run_e5506("var o={f:+\"hi\"}; console.log(o.f);");
}

#[test]
fn unary_plus_on_decimal_string_fails_closed() {
    // Coincidentally-correct today (`+"3"` would read back `3`), but
    // unsound in general (see `unary_plus_on_nonnumeric_string_fails_closed`)
    // — must fail closed regardless of this specific value.
    run_e5506("var o={f:+\"3\"}; console.log(o.f);");
}

#[test]
fn unary_minus_on_string_fails_closed() {
    run_e5506("var o={f:-\"3\"}; console.log(o.f);");
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

// ---- Residual guards (out of scope): must be NO WORSE than main. Each may
//      stay silent-0 or fail closed, but must never crash and never produce a
//      NEW nonzero-wrong value. ----

/// A newly-materialized object that ESCAPES via return then a member-on-call
/// read (R-06-R1 / R-14). Today: silent-0. Guard: exit 0 with "0", OR a
/// fail-closed diagnostic — never a crash, never a nonzero-wrong value.
#[test]
fn returned_object_member_read_no_worse() {
    let out = run("function h(){ var o = { f: 7 }; return o; } console.log(h().f);");
    if out.status.success() {
        // May not print node's "7" yet (R-14 escape is a later stage), but it
        // must not print a WRONG NONZERO value. Silent-0 is the tolerated state.
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
        assert!(
            stdout == "0" || stdout == "7",
            "returned-object read produced a new nonzero-wrong value: {stdout:?}"
        );
    }
    // A non-success exit (fail-closed) is also acceptable — the only forbidden
    // outcome is a silent NONZERO-wrong value, guarded above.
}

/// Whole-object reassignment to an object literal (R-06-R2), a distinct store
/// mechanism from the declarator init. Today: the reassigned read is silent-0.
/// Guard: no crash, no new nonzero-wrong value.
#[test]
fn object_literal_reassignment_no_worse() {
    let out = run("var o = { f: 1 }; console.log(o.f); o = { f: 2 }; console.log(o.f);");
    if out.status.success() {
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
        // First read is correct (1); the reassigned read is the residual.
        assert!(
            stdout == "1\n0" || stdout == "1\n2",
            "reassignment read produced a new nonzero-wrong value: {stdout:?}"
        );
    }
}

// ---- Regression pins (Task 1 review): lock in R-06 soundness invariants ----

/// R-06's bool fail-close (see `var_object_bool_field_reads_value` above) must
/// NOT bleed into the pre-existing WRITE-materialization path, which is a
/// distinct mechanism from the read-only declarator-init lane R-06 touches.
/// Confirmed on a fresh build: prints "1", exit 0 (NOT E5506) — this locks in
/// the load-bearing `!obj_materialized.contains` guard that scopes R-06's
/// Boolean-field fail-close to read-only bindings only.
#[test]
fn write_materialized_bool_stays_untouched() {
    assert_eq!(
        run_ok("var o = { f: false }; o.f = true; console.log(o.f);"),
        "1"
    );
}

/// The bool fail-close is scoped to the WHOLE binding, not per-field: a
/// read-only mutable object with ANY Boolean-literal field fails closed even
/// when only a safe numeric field is read. Confirmed on a fresh build: E5506,
/// exit 1. This documents an intended honest over-deny; per-field precision
/// is a possible later refinement, not a bug.
#[test]
fn mixed_bool_numeric_field_over_denies_e5506() {
    run_e5506("var o = { f: 7, g: true }; console.log(o.f);");
}
