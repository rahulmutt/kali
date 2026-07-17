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

/// Task 6 (Lane 3): same-shape object identity. `q = p` is aliasing (same
/// heap pointer); `r` is a separately-allocated same-shape object. `p === q`
/// must be real pointer identity (true); `p === r` must be false (distinct
/// allocations), proving the allow lane does real pointer comparison, not a
/// blanket true.
///
/// DEVIATION FROM THE BRIEF, two orthogonal pre-existing limitations
/// (documented; same pattern as Task 5's
/// `object_array_field_push_join_length_round_trip`):
///
/// 1. Multi-arg `console.log`: the brief's snippet used a single
///    `console.log(p === q, p === r)`. `crates/kali_codegen/src/emit/
///    call.rs`'s dynamic console lane emits only the FIRST argument and
///    silently drops the rest (see the "Stage 4 Task 6 re-review fix"
///    comment there) — reproduces identically with plain scalars
///    (`console.log(1 === 1, 1 === 2)` also prints a single `1`), unrelated
///    to objects. This test uses one `console.log` per comparison instead.
///
/// 2. Dynamic-boolean rendering: the brief asserted the output renders as
///    `"true"`/`"false"` text like node. That holds ONLY for a
///    compile-time-foldable literal (`console.log(true)` does print
///    `"true"`, via `render_console_call`'s static lane) — a genuinely
///    DYNAMIC boolean (the runtime lane `emit_console_argument`, which has
///    no `ValueShape::Boolean` arm, only `Float`) prints the raw `1`/`0` i64
///    unconverted. This is general and pre-existing — plain scalar
///    `let a=1,b=2; console.log(a===a); console.log(a===b);` also prints
///    `1`/`0`, and `array_callback_number_predicates_runtime.rs` already
///    pins this exact "1\n0\n..." convention as kali's accepted (if
///    node-diverging) behavior for runtime-computed booleans. `p === q` and
///    `p === r` are genuinely dynamic (real pointer identity, not a static
///    fold — that IS the point of this test), so they hit this same
///    pre-existing lane and print `1`/`0`. Fixing dynamic Boolean-to-text
///    rendering is a general, separate, higher-blast-radius change (it would
///    flip the expected output of every already-green test that prints a
///    runtime comparison) — out of scope for Lane 3, which is about WHETHER
///    the comparison is sound, not how its result is printed.
#[test]
fn same_shape_object_identity_alias_is_true() {
    let src = "const p = { x: 1 };\nconst q = p;\nconst r = { x: 2 };\n\
               console.log(p === q);\n\
               console.log(p === r);\n";
    let out = run_kali_run(src);
    assert_eq!(out.trim(), "1\n0"); // node: true / false (see doc comment)
}

/// Task 6 (Lane 3): cross-shape `===` still fails closed. This may pass "by
/// accident" even before the allow lane exists (the pre-existing blanket
/// object-misuse gate already E5506s any object-involving `===`) — the
/// alias test above is what actually proves the allow lane exists. This test
/// guards against a future regression where the allow lane is loosened to
/// admit cross-shape comparisons.
#[test]
fn structured_clone_cross_shape_identity_fails_closed() {
    let src = "const a = { x: 1 };\nconst b = { y: 1, z: 2 };\n\
               console.log(a === b);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

/// Task 6 (Lane 3) soundness pin: closes the p2a fail-open. One operand (`o`)
/// has a proven object shape; the other (`u`, an unknown-repr parameter) does
/// not. The allow lane requires BOTH operands proven same-shape — an
/// unknown-repr operand must not slip through to a scalar `===` arm (which
/// would silently compare a raw heap pointer against a scalar, or vice
/// versa). Falls to the blanket gate → E5506.
#[test]
fn object_identity_against_unknown_repr_fails_closed() {
    let src = "function f(u) { const o = { x: 1 }; return o === u; }\n\
               console.log(f(0));\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

/// Task 6 (Lane 3): `!==` must not be inverted relative to `===`. The brief's
/// tests only cover `===`; this pins `!==` on the same alias/distinct pair —
/// `p !== q` (alias) is false, `p !== r` (distinct same-shape) is true.
/// Single-argument `console.log` per comparison, raw `0`/`1` output — see
/// `same_shape_object_identity_alias_is_true`'s doc comment for both
/// deviations (multi-arg console.log drop; dynamic-boolean 1/0 rendering).
#[test]
fn same_shape_object_identity_not_equal() {
    let src = "const p = { x: 1 };\nconst q = p;\nconst r = { x: 2 };\n\
               console.log(p !== q);\n\
               console.log(p !== r);\n";
    let out = run_kali_run(src);
    assert_eq!(out.trim(), "0\n1"); // node: false / true (see doc comment)
}

/// Task 8 (Lane 2b): `structuredClone` of an in-envelope object (every field a
/// scalar or a `GrowableArrayI64` array) DEEP-CLONES it — the clone shares no
/// mutable storage with the source, so a later `push` into the SOURCE's array
/// does not appear in the clone.
///
/// DEVIATION FROM THE BRIEF (documented; Tasks 5/6 precedent, controller-
/// ratified): the brief's body used a single MULTI-argument
/// `console.log(cloned.count, cloned.values.join(','), original.values.join(','))`.
/// Multi-argument `console.log` emits only the FIRST argument (the dynamic
/// console lane drops the rest) and, where an argument reads a growable array,
/// fails closed by an established Stage 4 soundness contract (see
/// `multi_arg_console_with_growable_field_fails_closed`). Each value is logged
/// on its own line here, asserting the SAME semantic facts: the clone's scalar
/// field is preserved (`1`), the clone's array is a DEEP copy unaffected by the
/// push into the source (`1,2,3`), and the source's array did grow (`1,2,3,4`).
#[test]
fn structured_clone_deep_clones_scalar_and_array_object() {
    let src = "const original = { count: 1, values: [1, 2, 3] };\n\
               const cloned = structuredClone(original);\n\
               original.values.push(4);\n\
               console.log(cloned.count);\n\
               console.log(cloned.values.join(','));\n\
               console.log(original.values.join(','));\n";
    let out = run_kali_run(src);
    // clone unaffected by the push into original.values (node: 1 / 1,2,3 / 1,2,3,4)
    assert_eq!(out.trim(), "1\n1,2,3\n1,2,3,4");
}

/// Task 8 (Lane 2b): the clone is a DISTINCT allocation — `cloned === original`
/// is false. `cloned.values === original.values` is likewise false (the array
/// storage was deep-copied into a fresh handle).
///
/// DEVIATION FROM THE BRIEF (documented; same two pre-existing limits as
/// `same_shape_object_identity_alias_is_true`): dynamic booleans render as
/// `1`/`0` (the runtime console lane has no Boolean arm), and multi-argument
/// `console.log` drops trailing arguments — so each comparison is logged alone
/// and the raw `0` (false) is asserted. `cloned === original` is genuine
/// runtime pointer identity (real allocations, not a static fold), which is
/// exactly what proves the clone is not the source object.
#[test]
fn structured_clone_result_identity_is_false() {
    let src = "const original = { count: 1, values: [1, 2, 3] };\n\
               const cloned = structuredClone(original);\n\
               console.log(cloned === original);\n\
               console.log(cloned.values === original.values);\n";
    let out = run_kali_run(src);
    assert_eq!(out.trim(), "0\n0"); // node: false / false (see doc comment)
}

/// Task 8 (Lane 2b) soundness pin: `structuredClone` of an argument whose shape
/// is NOT provable (an unknown-repr parameter) fails closed E5506 — never a
/// silent shallow copy or a zero placeholder that misreports the clone. The
/// call sits in an uncalled function; codegen still emits its body, so the
/// dispatch fires and denies.
#[test]
fn structured_clone_of_unproven_argument_fails_closed() {
    let src = "function f(u) { return structuredClone(u); }\nconsole.log(1);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}
