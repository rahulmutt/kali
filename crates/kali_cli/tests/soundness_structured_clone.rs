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

/// Task 9: run `kali build --bundle --api browser <src>` and return
/// (success, stderr). `--api browser` is required here because the fixtures
/// this helper exercises construct `new Blob(...)` — a browser-surface
/// global; a plain `kali build --bundle` (no `--api browser`) fails closed
/// with E5508 ("requires the effective browser API surface") before ever
/// reaching the structuredClone dispatch, which would test the wrong thing.
/// This mirrors the browser-bundle build invocation used throughout
/// `package_corpus/browser_corpus.rs` (e.g. `assert_browser_bundle_object_has_own`
/// and the corpus's `write_browser_string_web_baseline_package` fixture, which
/// also constructs `new Blob([...])` ahead of a `build --bundle --api browser`
/// build) and `string_pad_static_ascii.rs`'s
/// `browser_bundle_accepts_static_ascii_string_pad_across_source_classes`.
///
/// Review finding (Task 9, ratified): `kali build`'s SUCCESS path never
/// surfaces warnings at all — `cmd_build.rs`'s `Ok(build_result)` arm passes
/// hardcoded `vec![]` for both errors and warnings to `print_envelope`, and
/// the underlying `kali_cli::build` compile pipeline (`compile.rs`) only
/// ever returns its accumulated `diagnostics` Vec via the `Err` branch (every
/// checkpoint is `if has_errors(&diagnostics) { return Err(diagnostics); }` —
/// a warnings-only accumulation is simply dropped on the `Ok` path, and
/// `CompileOutput`/`BuildOutput` carry no diagnostics field at all). This is
/// general and pre-existing, not specific to structuredClone: verified `kali
/// check`, `kali run`, and `kali build` all report zero warnings in stderr
/// AND in `--output json`'s `"warnings"` array for a program that
/// demonstrably DID take a warning-emitting codegen path (its runtime output
/// proves the codegen branch executed). Practical consequence: a caller that
/// needs to observe a warning-level diagnostic must pair it with an
/// unrelated, independently-established fail-closed case so the overall
/// build fails — the FAILURE path is the only one that returns the full
/// diagnostics list (warnings included). See
/// `structured_clone_of_placeholder_construct_emits_e8001_warning`.
fn build_bundle_output(source: &str) -> (bool, String) {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.js");
    fs::write(&path, source).expect("write source");
    let out = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&path)
        .output()
        .expect("run kali build --bundle");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Task 9: run `kali build --bundle --api browser <src>` and return whether it
/// succeeded (discarding stderr — use `build_bundle_output` if the caller
/// needs to inspect diagnostics, e.g. to observe a warning, which requires
/// pairing with a deliberate failure per that helper's doc comment).
fn build_bundle_succeeds(source: &str) -> bool {
    let (success, stderr) = build_bundle_output(source);
    if !success {
        eprintln!("build --bundle --api browser failed; stderr: {stderr}");
    }
    success
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

/// Task 8 review CRITICAL fix: an object-POINTER field via an IDENTIFIER
/// (`{ a: 1, inner: inner }` where `inner` is object-shaped) must FAIL CLOSED
/// E5506, not intern as a plain `I64` pointer slot. Without the fix, the field
/// passed the clone envelope and the clone verbatim-copied the pointer —
/// SHALLOW-SHARING the nested object: `cloned.inner === original.inner` printed
/// `1` (node: `false`). The rejection is at the inference field-repr choke
/// point (`repr_infer.rs` `resolve_objects`), so it closes the class for ANY
/// consumer; `structuredClone` here is what materializes `original` and unmasks
/// the shape. The inline-literal twin (`{ inner: { b: 2 } }`) is already
/// rejected at record time — this pins the identifier-RHS twin.
#[test]
fn structured_clone_object_pointer_field_identity_fails_closed() {
    let src = "const inner = { b: 2 };\n\
               const original = { a: 1, inner: inner };\n\
               const cloned = structuredClone(original);\n\
               console.log(cloned.inner === original.inner);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

/// Task 8 review CRITICAL fix (read form): the same object-pointer-field
/// program read through the nested property (`cloned.inner.b`) must also FAIL
/// CLOSED E5506 — without the fix it printed `0` (node: `2`), a silent
/// miscompile of the shallow-shared nested object.
#[test]
fn structured_clone_object_pointer_field_nested_read_fails_closed() {
    let src = "const inner = { b: 2 };\n\
               const original = { a: 1, inner: inner };\n\
               const cloned = structuredClone(original);\n\
               console.log(cloned.inner.b);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

/// Task 8 review positive control (NO over-closure): a SCALAR-identifier field
/// (`{ a: n }` where `n` solves to a number) must still clone correctly — the
/// clone-safety allowlist admits a field whose source proves non-object, so a
/// scalar identifier is never flagged. The growable `values` field forces
/// materialization (so the runtime clone lane, not the fold lane, is
/// exercised).
#[test]
fn structured_clone_scalar_identifier_field_still_clones() {
    let src = "const n = 7;\n\
               const original = { a: n, values: [1, 2, 3] };\n\
               const cloned = structuredClone(original);\n\
               console.log(cloned.a);\n\
               console.log(cloned.values.join(','));\n";
    let out = run_kali_run(src);
    assert_eq!(out.trim(), "7\n1,2,3");
}

/// Task 8 re-review CRITICAL (class closure): the clone envelope is a
/// PROVEN-SOURCE ALLOWLIST — a field whose source is a CALL RETURN of an object
/// (`{ inner: mk() }`) is an object pointer that would shallow-share, so the
/// clone fails closed E5506. (Without the allowlist: `cloned.inner ===
/// original.inner` → `1` / node `false`.) The rejection is at the clone
/// choke-point clone-safety bit (NOT at materialization), so a call-return
/// object field remains usable by non-clone consumers (e.g. binary-trees'
/// `{ left: bottomUpTree(d) }`, which never clones).
#[test]
fn structured_clone_call_return_object_field_identity_fails_closed() {
    let src = "function mk() { return { b: 2 }; }\n\
               const original = { a: 1, inner: mk() };\n\
               const cloned = structuredClone(original);\n\
               console.log(cloned.inner === original.inner);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

/// Task 8 re-review CRITICAL (read form): the same call-return object field read
/// through the nested property (`cloned.inner.b`) must also fail closed E5506
/// (without the allowlist it printed `0` / node `2`).
#[test]
fn structured_clone_call_return_object_field_nested_read_fails_closed() {
    let src = "function mk() { return { b: 2 }; }\n\
               const original = { a: 1, inner: mk() };\n\
               const cloned = structuredClone(original);\n\
               console.log(cloned.inner.b);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

/// Task 8 re-review CRITICAL (index-source form): a field whose source is an
/// array element (`{ inner: arr[0] }` over an object-element array) is likewise
/// an object pointer → fail closed E5506 (without the allowlist: `1` / node
/// `false`). Closes the `arr[i]` source shape alongside identifier and call.
#[test]
fn structured_clone_index_source_object_field_fails_closed() {
    let src = "const arr = [ { b: 2 } ];\n\
               const original = { a: 1, inner: arr[0] };\n\
               const cloned = structuredClone(original);\n\
               console.log(cloned.inner === original.inner);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

/// Task 8 re-review CRITICAL (growable-mutation probe): a call-returned inner
/// object carrying a growable array, mutated through the SOURCE after the clone,
/// must fail closed E5506 — a shallow-shared inner would let the clone observe
/// the push (node deep-clones and does not). The clone-safety allowlist denies
/// the whole object (its `inner` field is an object pointer) before any of this
/// can be observed.
#[test]
fn structured_clone_call_return_growable_inner_mutation_fails_closed() {
    let src = "function mk() { return { vals: [1, 2, 3] }; }\n\
               const original = { a: 1, inner: mk() };\n\
               const cloned = structuredClone(original);\n\
               console.log(cloned.a);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

/// Task 8 corpus-regression fix: the package-corpus re-clone shape
/// (`const b = structuredClone(new Blob(['x'])); structuredClone(b);`) must
/// keep BUILDING. `b` is a `const` bound to a `structuredClone` of a
/// zero-placeholder construct (`new Blob`), so it has PROVABLE placeholder
/// provenance and the re-clone stays on the warn-build lane. (Before the fix
/// the identifier arg hit Lane 3 E5506 and 18 corpus builds failed.)
#[test]
fn structured_clone_of_placeholder_derived_const_builds() {
    let src = "const b = structuredClone(new Blob(['x']));\n\
               structuredClone(b);\n\
               console.log(1);\n";
    let out = run_kali_run(src);
    assert_eq!(out.trim(), "1");
}

/// Task 8 corpus-regression fix: placeholder provenance CHAINS through `const`
/// bindings (`const a = new Blob(...); const b = a; structuredClone(b);`) —
/// `b` aliases a const placeholder construct, so it too warn-builds.
#[test]
fn structured_clone_of_placeholder_const_chain_builds() {
    let src = "const a = new Blob(['x']);\n\
               const b = a;\n\
               structuredClone(b);\n\
               console.log(1);\n";
    let out = run_kali_run(src);
    assert_eq!(out.trim(), "1");
}

/// Task 8 corpus-regression NEGATIVE control (const-only, no `let` chasing): a
/// `let` binding REASSIGNED to a scalar is NOT placeholder-provable (only
/// `const` single-init bindings qualify), so `structuredClone(b)` must NOT be
/// admitted to the placeholder lane — with a scalar `b` it provably fails closed
/// E5506 via Lane 3. If a future change wrongly admitted mutable bindings to the
/// placeholder lane, this would build (exit 0) and the test would go red.
#[test]
fn structured_clone_of_reassigned_let_scalar_fails_closed() {
    let src = "let b = structuredClone(new Blob(['x']));\n\
               b = 5;\n\
               structuredClone(b);\n\
               console.log(1);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

/// Task 8 corpus-regression NEGATIVE control (reviewer's exact shape): a `let`
/// reassigned to an OBJECT LITERAL. It provably does NOT take the placeholder
/// lane (const-only) — instead the in-envelope reassignment `{ x: 1 }` makes it
/// a Lane-1 clone, so it BUILDS (per the contract, Lane 1 here is acceptable;
/// what matters is the mutable binding is never chased into the placeholder
/// lane). Documented deviation: `b.x` is a pre-existing 0 (the
/// placeholder-init-then-object-reassign pattern mis-reprs `b` independently of
/// cloning), so only build-success is asserted.
#[test]
fn structured_clone_of_reassigned_let_object_builds_via_lane1() {
    let src = "let b = structuredClone(new Blob(['x']));\n\
               b = { x: 1 };\n\
               structuredClone(b);\n\
               console.log(2);\n";
    let out = run_kali_run(src);
    assert_eq!(out.trim(), "2");
}

/// Task 9 (Lane 2 tripwire): the corpus shape `structuredClone(new Blob([...]))`
/// (see `package_corpus.rs`'s `write_browser_string_web_baseline_package` and
/// `package_corpus/browser_corpus.rs`'s inline `structuredClone(new
/// Blob(['browser corpus']))` fixtures) must keep BUILDING under `kali build
/// --bundle` (Task 8's Lane 2 warn-and-placeholder entry). `Blob` has no real
/// construct lowering (`declarator_init_is_placeholder_construct`,
/// `crates/kali_codegen/src/lower.rs`, treats any bare `new X()` other than
/// `Array`/`Uint8Array`/`EventTarget` as a zero-placeholder), so
/// `structuredClone` of it takes entry 2 (warn + keep the placeholder-0
/// lowering) rather than entry 3 (fail closed E5506). This test pins that the
/// corpus-shaped program still builds; it must go RED if entry 2 is ever
/// tightened to deny placeholder-construct arguments, since that would break
/// the corpus's `structuredClone(new Blob(...))` / `new File(...)` pins.
#[test]
fn structured_clone_of_placeholder_construct_still_builds() {
    // Corpus shape: structuredClone(new Blob([...])) must BUILD (check/bundle).
    let src = "structuredClone(new Blob(['x']));\nexport default function root() { return 1; }\n";
    assert!(build_bundle_succeeds(src));
}

/// Task 9 (Lane 2): CURRENT-RENDERING PIN, not a guard. This asserts what
/// `console.log(typeof b)` prints TODAY for `const b =
/// structuredClone(new Blob(['x']))` — kali's placeholder-0 lowering renders
/// as `"0"`, which happens to differ from node's real `"object"`. It does
/// NOT reliably detect Blob gaining a real construct lowering: review
/// (Task 9) stress-tested the claim and found kali's `typeof <non-literal
/// binding>` prints the raw scalar value for ANY binding, including a real,
/// successfully-cloned in-envelope object (`typeof` only hits a type-name
/// fold lane for a literal operand, e.g. `typeof 5` prints `"number"`; `let
/// x = 5; console.log(typeof x);` prints `"5"`, not `"number"`, and the same
/// holds for an object binding). So this assertion would keep passing even
/// after Blob gained a real lowering, as long as the result still isn't a
/// literal — it has no discriminating power over that event. The REAL
/// forward guard is
/// `structured_clone_of_placeholder_construct_emits_e8001_warning` below,
/// which asserts the Lane-2 warn diagnostic itself, not a rendering
/// side-effect of it. This test is kept only as a current-behavior pin
/// (useful for noticing an unrelated `typeof`-rendering change), with its
/// guard claim removed.
#[test]
fn structured_clone_of_placeholder_construct_tripwire() {
    let src = "const b = structuredClone(new Blob(['x']));\nconsole.log(typeof b);\n";
    let out = run_kali_run(src);
    // kali: placeholder 0 → prints its scalar rendering ("0"); node: "object".
    assert_ne!(
        out.trim(),
        "object",
        "current-rendering pin broke — see structured_clone_of_placeholder_construct_emits_e8001_warning for the actual Blob-exclusion guard"
    );
}

/// Task 9 review fix: THE REAL forward guard for the Blob placeholder-
/// construct lane. Review found the `typeof`-divergence test above cannot
/// discriminate Blob gaining a real lowering (see its doc comment). The
/// actual signal that `structuredClone(new Blob(...))` took the Lane-2
/// warn-build path — i.e., that `Blob` is STILL in the zero-placeholder
/// exclusion set — is the **E8001** "no-op placeholder" diagnostic
/// (`crates/kali_codegen/src/emit/call.rs`, pushed right before the Lane-2
/// warn-build return). That diagnostic is emitted IFF
/// `declarator_init_is_placeholder_construct` (or its placeholder-provenance
/// identifier extension, `a0770edce`) matches the argument — it disappears
/// the day `Blob` is removed from the exclusion list (a real lowering routes
/// through Lane 1, a dedicated Blob lane, or Lane 3 instead, none of which
/// emit this warning). THIS is the guard: it goes RED (E8001 stops
/// appearing) exactly when Blob's exclusion-list membership changes, forcing
/// the decision the tripwire exists to force.
///
/// Why this is paired with an UNRELATED, independently-established
/// fail-closed case (`structured_clone_of_unproven_argument_fails_closed`'s
/// exact `function f(u) { return structuredClone(u); }` shape) rather than
/// asserted on a standalone successful build: `kali build`'s SUCCESS path
/// structurally never returns warnings (see `build_bundle_output`'s doc
/// comment) — verified empirically that a program which demonstrably
/// executes the Lane-2 codegen branch (its `kali run` output is the
/// placeholder value) still reports zero warnings via `kali build`/`run`/
/// `check`, in both plain-text stderr and `--output json`'s `"warnings"`
/// array. Pairing with an unrelated failure routes the SAME diagnostics list
/// (E8001 for the Blob line, accumulated during the same codegen pass)
/// through the CLI's failure path, which is the only path that returns it.
/// The two structuredClone call sites are independent (different arguments,
/// different lanes), so the E8001 assertion is scoped precisely to the Blob
/// line and is not a side effect of the unrelated E5506.
#[test]
fn structured_clone_of_placeholder_construct_emits_e8001_warning() {
    let src = "structuredClone(new Blob(['x']));\n\
               function f(u) { return structuredClone(u); }\n\
               console.log(1);\n";
    let (success, stderr) = build_bundle_output(src);
    assert!(
        !success,
        "expected the unrelated f(u) unproven-argument case to fail closed; stderr: {stderr}"
    );
    assert!(
        stderr.contains("E5506"),
        "expected the unrelated fail-closed case's E5506 diagnostic; stderr: {stderr}"
    );
    assert!(
        stderr.contains("E8001") && stderr.contains("no-op placeholder"),
        "expected the Lane-2 warn-build E8001 diagnostic for `new Blob(...)` — \
         this is the REAL guard: it must disappear the day Blob gains a real \
         construct lowering, forcing the exclusion-list decision; stderr: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// Stage P2 whole-stage review fix-wave pins (C-1, C-2, I-2, I-3, I-1, M-3).
// Every soundness rule is ALLOWLIST-at-choke-point / default-deny: an
// out-of-envelope growable-array-field use is E5506 (fail closed), NEVER a
// silent miscompile or a raw-handle escape.
// ---------------------------------------------------------------------------

/// C-1a (silent corruption close): reassigning a `GrowableArrayI64` object
/// field (`o.values = [4,5]`) must FAIL CLOSED E5506, not store a bogus i64
/// over the valid handle. RED before the fix: the fixed-shape field-store
/// `_ =>` arm admitted the growable slot and stored `I64Const(0)` (or a
/// non-handle) — `o.values.join(',')` then printed empty; node prints `4,5`.
/// The deny is the sound minimal close (no re-seeding this wave).
#[test]
fn growable_field_reassignment_fails_closed() {
    let src = "const o = { values: [1, 2, 3] };\n\
               o.values = [4, 5];\n\
               console.log(o.values.join(','));\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

/// C-1b (silent drop close): an element WRITE through a `GrowableArrayI64`
/// field (`o.values[0] = 9`) must FAIL CLOSED E5506, mirroring the named-lane
/// twin (`a[0] = 9`, which already E5506s). RED before the fix: the write was
/// silently DROPPED (`o.values.join(',')` printed `1,2,3`; node prints `9,2,3`).
#[test]
fn growable_field_element_write_fails_closed() {
    let src = "const o = { values: [1, 2, 3] };\n\
               o.values[0] = 9;\n\
               console.log(o.values.join(','));\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

/// C-2 (raw-handle escape close): reading a `GrowableArrayI64` field as a
/// plain value in a NON-allowlisted position must FAIL CLOSED E5506. RED
/// before the fix: `console.log(o.values)` printed the tagged handle bits
/// (`4611686018427392016` — a heap-address leak). The position gate admits a
/// growable field read ONLY where a growable-aware recognizer consumes the
/// receiver (push/join/length/index/for-of/`===`/clone).
#[test]
fn growable_field_bare_read_fails_closed() {
    let src = "const o = { values: [1, 2, 3] };\n\
               console.log(o.values);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

/// C-2 (raw-handle escape close, arithmetic position): `o.values + 1` must
/// FAIL CLOSED E5506. RED before the fix: printed `handle + 1`
/// (`4611686018427392017`), arithmetic on the raw tagged handle.
#[test]
fn growable_field_arithmetic_read_fails_closed() {
    let src = "const o = { values: [1, 2, 3] };\n\
               console.log(o.values + 1);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

/// I-3 (growable-field aliasing declarator deny): `const a = o.values` binds a
/// raw growable handle in a non-allowlisted value position, so C-2's position
/// gate already fails it closed E5506 (verified: no second gate needed). RED
/// before the fix: `a.length` printed `0`; node prints `3`.
#[test]
fn growable_field_alias_declarator_fails_closed() {
    let src = "const o = { values: [1, 2, 3] };\n\
               const a = o.values;\n\
               console.log(a.length);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

/// I-3 named-side alias: DELIBERATE TRIPWIRE (known fail-open, pre-existing,
/// OUT OF STAGE — not closed this wave to avoid named-lane regressions). A
/// named-growable alias `const b = a` yields a SEPARATE broken binding that
/// does not track `a`'s growth: after `a.push(4)`, `b.join(',')` prints
/// `1,2,3` while node prints `1,2,3,4`. Pinned to document the current WRONG
/// output; if the named-alias class is ever closed (or fixed), this pin goes
/// red and forces the inventory decision. See the fix-wave report I-3 entry.
#[test]
fn named_growable_alias_is_broken_tripwire() {
    let src = "const a = [1, 2, 3];\n\
               a.push(4);\n\
               const b = a;\n\
               console.log(b.join(','));\n";
    let out = run_kali_run(src);
    // kali: `1,2,3` (alias `b` snapshots pre-push storage); node: `1,2,3,4`.
    assert_eq!(
        out.trim(),
        "1,2,3",
        "named-growable-alias tripwire changed — see fix-wave report I-3"
    );
}

/// I-2 (structuredClone-scoped member-on-call close): a member read on an
/// UNBOUND `structuredClone(...)` result (`structuredClone(o).count`) must
/// FAIL CLOSED E5506, directing the user to bind the result first. RED before
/// the fix: printed `0` (the pre-existing member-on-call placeholder hole —
/// `is_structured_clone_result` only promotes const declarators). Scoped
/// strictly to a `structuredClone` callee; the general member-on-call class is
/// pre-existing and inventoried, untouched here.
#[test]
fn structured_clone_member_on_call_fails_closed() {
    let src = "const o = { count: 1, values: [1, 2, 3] };\n\
               console.log(structuredClone(o).count);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

/// I-1: DELIBERATE TRIPWIRE (pre-existing object-reassignment gap, out of
/// stage). `let o = {v:[1,2]}; o = {v:3}; console.log(o.v)` — node prints `3`;
/// kali cannot do object reassignment. BEFORE this fix-wave it printed a silent
/// `0` (reassignment zeroed reads). C-2's growable-field position gate now
/// UPGRADES this to fail-closed E5506: `v` is inferred `GrowableArrayI64` (from
/// the first literal; the `kali_types` `repr_infer.rs` `record_object_array_field`
/// AND-merge only demotes array-vs-array, so the array-vs-scalar reassignment
/// does NOT demote `v` to scalar), and reading `o.v` in a bare `console.log`
/// position is not an allowlisted growable receiver — so it denies rather than
/// silently reads 0. Strictly more sound than the prior silent `0`. This pin
/// documents the CURRENT (post-C-2) masked behavior; if object reassignment
/// ever lands a real lowering, the `GrowableArrayI64`-vs-scalar intern
/// confusion named above must be revisited before the gate is relaxed.
///
/// Reviewer Minor note (inventory, R-wave): a `.length` READ of the reassigned
/// field (`let o={v:[1,2]}; o={v:3}; console.log(o.v.length)`) still silently
/// reads `0` — `.length` is an ALLOWLISTED growable-field read position, so C-2
/// does not gate it; the wrong value is the same object-reassignment mis-repr.
/// Closes automatically the day object reassignment is implemented (a real
/// lowering) OR denied wholesale; NOT separately gated here (gating an
/// allowlisted read position would need reassignment-awareness, out of stage).
#[test]
fn object_reassignment_field_read_fails_closed_tripwire() {
    let src = "let o = { v: [1, 2] };\n\
               o = { v: 3 };\n\
               console.log(o.v);\n";
    let stderr = run_kali_run_expect_error(src);
    // kali: E5506 (C-2 gate; was silent `0` pre-fix); node: `3`.
    assert!(
        stderr.contains("E5506"),
        "object-reassignment tripwire changed — revisit GrowableArrayI64/scalar \
         intern confusion in repr_infer.rs record_object_array_field (see fix-wave report I-1); stderr: {stderr}"
    );
}

// M-3 (inventory only, no product code): the NAMED-growable `===` lane
// (`a === b` over two named growable bindings) currently compares the raw i64
// tagged handles directly (the scalar `===` arm), which is COINCIDENTALLY
// correct for pointer identity (equal handles ⇔ same header) but is NOT routed
// through the Lane-3 same-shape allowlist the FIELD pair uses
// (`both_growable_field` in operators.rs `emit_binary`). It is sound today only
// because a growable handle IS its identity; if the handle encoding ever gains
// non-identity bits (e.g. a generation tag), the named lane must be re-pinned
// to the allowlisted compare. Field-pair `===` is already gated (Lane 3).

// ---------------------------------------------------------------------------
// Stage P2 review RIDERS (R1, R2): named/field write-and-join asymmetries.
// ---------------------------------------------------------------------------

/// R1 (silent no-op close): a `.length` WRITE through a `GrowableArrayI64`
/// field (`o.values.length = 1`) must FAIL CLOSED E5506 — node TRUNCATES the
/// array to length 1, so a dropped store is a silent miscompile. The named
/// twin (`a.length = 1`) already fails closed. RED before the fix: the write
/// fell through every recognizer (the C-1b element guard's 1-child arm
/// exempted `text == "length"`, correct for READS but not writes) and was
/// silently DROPPED — `o.values.join(',')` printed `1,2,3`; node prints `1`.
#[test]
fn growable_field_length_write_fails_closed() {
    let src = "const o = { values: [1, 2, 3] };\n\
               o.values.length = 1;\n\
               console.log(o.values.join(','));\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

/// R2 (NUL-garbage close): a growable-array FIELD `join` with a separator that
/// is NOT string-provable (`o.values.join(o.values.length)` — an i64 length)
/// must FAIL CLOSED E5506. RED before the fix: the field arm skipped the
/// separator string-proof the named lane enforces (in `kali_types`, keyed on an
/// Identifier receiver — a field receiver is a MemberExpression, so it slipped
/// the gate); the raw i64 separator was read as a string handle → NUL-bearing
/// garbage stdout (`31 00 00 00 32 00 00 00 33 0a`); node prints `13233`. The
/// named twin (`a.join(a.length)`) already E5506s.
#[test]
fn growable_field_join_numeric_separator_fails_closed() {
    let src = "const o = { values: [1, 2, 3] };\n\
               console.log(o.values.join(o.values.length));\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

/// R2 positive control (allowlist's green side — reviewer-verified correct
/// today, must NOT regress): a growable-field `join` with a STRING-binding
/// separator round-trips, and an OMITTED separator (default ",") round-trips.
#[test]
fn growable_field_join_string_and_omitted_separator_stay_green() {
    let with_string = "const o = { values: [1, 2, 3] };\n\
                       const sep = '-';\n\
                       console.log(o.values.join(sep));\n";
    assert_eq!(run_kali_run(with_string).trim(), "1-2-3");
    let omitted = "const o = { values: [1, 2, 3] };\n\
                   console.log(o.values.join());\n";
    assert_eq!(run_kali_run(omitted).trim(), "1,2,3");
}
