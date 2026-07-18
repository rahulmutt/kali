//! Stage P3 (AbortController/AbortSignal lane) — Task 3: real
//! `new AbortController()` lowering (8-byte global abort cell), `c.abort()`
//! dispatch, the bare-identifier position gate, and capture allowlist entry 3.
//!
//! The atomic core of the stage: the exclusion-list flip
//! (`declarator_init_is_placeholder_construct`) lands with the real lowering,
//! entry 3, and `.abort()` dispatch in one commit — intermediate states are
//! unsound/red. These pins prove the visible-no-op-but-runs surface, the
//! fail-closed reads (raw print / unknown method / `let`-declared), the sound
//! deferred-capture-by-value case, and the b2 red-proof (plain objects still
//! deny).

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

#[test]
fn abort_flips_nothing_visible_but_runs() {
    let src = "const c = new AbortController();\nc.abort();\nconsole.log(\"ok\");\n";
    assert_eq!(run_kali_run(src).trim(), "ok");
}

#[test]
fn abort_handle_raw_print_fails_closed() {
    let src = "const c = new AbortController();\nconsole.log(c);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn unknown_method_on_abort_handle_fails_closed() {
    let src = "const c = new AbortController();\nc.reset();\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn let_declared_controller_fails_closed_on_abort() {
    let src = "let c = new AbortController();\nc.abort();\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn captured_abort_handle_in_deferred_callback_runs() {
    // Allowlist entry 3: the handle is an i64 pointer to a never-reclaimed
    // global cell — by-value restore is sound after the owner frame dies.
    let src = "function m() {\n  const c = new AbortController();\n  setTimeout(function() { c.abort(); console.log(\"cb\"); }, 0);\n}\nm();\n";
    assert_eq!(run_kali_run(src).trim(), "cb");
}

#[test]
fn captured_plain_object_still_fails_closed() {
    // b2 stays red-proof: entry 3 must not widen to arena-backed objects.
    let src = "function m() {\n  const o = { x: 4 };\n  setTimeout(function() { console.log(\"x=\" + o.x); }, 0);\n}\nm();\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn loop_allocated_controllers_each_get_a_fresh_cell() {
    let src = "for (let i = 0; i < 3; i = i + 1) {\n  const c = new AbortController();\n  c.abort();\n}\nconsole.log(\"done\");\n";
    assert_eq!(run_kali_run(src).trim(), "done");
}

// --- Task 4: `.signal` identity, `.aborted` read, signal alias ---------------

#[test]
fn aborted_flag_reads_zero_then_one() {
    // Dynamic booleans render 1/0 (ratified P2 convention; node prints
    // true/false — documented divergence, never used in byte-for-byte
    // acceptance fixtures).
    let src = "const c = new AbortController();\nconsole.log(c.signal.aborted);\nc.abort();\nconsole.log(c.signal.aborted);\n";
    assert_eq!(run_kali_run(src).trim(), "0\n1");
}

#[test]
fn signal_alias_reads_shared_cell() {
    let src = "const c = new AbortController();\nconst s = c.signal;\nc.abort();\nconsole.log(s.aborted);\n";
    assert_eq!(run_kali_run(src).trim(), "1");
}

#[test]
fn aborted_in_boolean_position_branches() {
    let src = "const c = new AbortController();\nc.abort();\nif (c.signal.aborted) { console.log(\"yes\"); } else { console.log(\"no\"); }\n";
    assert_eq!(run_kali_run(src).trim(), "yes");
}

#[test]
fn signal_raw_print_fails_closed() {
    let src = "const c = new AbortController();\nconsole.log(c.signal);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn aborted_write_fails_closed() {
    let src = "const c = new AbortController();\nc.signal.aborted = 1;\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn signal_field_write_fails_closed() {
    let src = "const c = new AbortController();\nconst s = c.signal;\nc.signal = s;\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn add_event_listener_on_signal_fails_closed() {
    let src = "const c = new AbortController();\nc.signal.addEventListener(\"abort\", function() { console.log(\"x\"); });\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn captured_handle_full_roundtrip_in_listener() {
    // The acceptance-listener shape end-to-end: capture, abort inside a
    // synchronously-dispatched listener, observe the flag outside.
    let src = "function m() {\n  const c = new AbortController();\n  const t = new EventTarget();\n  let count = 0;\n  t.addEventListener(\"tick\", function() { count += 1; c.abort(); });\n  t.dispatchEvent(new CustomEvent(\"tick\"));\n  console.log(\"count=\" + count);\n  console.log(\"aborted=\" + c.signal.aborted);\n}\nm();\n";
    assert_eq!(run_kali_run(src).trim(), "count=1\naborted=1");
}

#[test]
fn sibling_closures_capture_distinct_controllers() {
    // Env-safety probe (Stage C sibling-extent lesson): two controllers in
    // sibling scopes must not share a cell.
    let src = "function a() {\n  const c = new AbortController();\n  c.abort();\n  console.log(\"a=\" + c.signal.aborted);\n}\nfunction b() {\n  const c = new AbortController();\n  console.log(\"b=\" + c.signal.aborted);\n}\na();\nb();\n";
    assert_eq!(run_kali_run(src).trim(), "a=1\nb=0");
}

#[test]
fn module_scope_captured_abort_write_fails_closed() {
    // Controller decision (Task 7 acceptance follow-up): a MODULE-scope
    // `const controller = new AbortController()` captured by a closure that
    // calls `controller.abort()` is NOT the ratified capture lane — that lane
    // is FUNCTION-scoped (owner-keyed captured handle, entry 3). The local /
    // depth-1-captured `is_abort_handle` proof does not admit a module binding,
    // so before this gate the call fell THROUGH to the generic undefined-call
    // fallback and was silently dropped through an E3100 zero placeholder: the
    // abort became a no-op (node aborts; kali did not) — a semantic miscompile
    // on P3's own value class. It now denies fail-closed, mirroring the
    // read-position `module_binding_names` gate, keyed on the module-level
    // (`_start`) repr being `AbortHandle`.
    let src = "const controller = new AbortController();\nsetTimeout(function() { controller.abort(); }, 0);\nconsole.log(\"aborted=\" + controller.signal.aborted);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn module_scope_captured_abort_read_already_fails_closed() {
    // The read-position twin of `module_scope_captured_abort_write_fails_closed`,
    // pinned so the write/read asymmetry cannot silently return. Reading a
    // module-scope binding from a closure was ALREADY fail-closed (the identifier
    // choke point's `module_binding_names` E5506 gate); this asserts that stays
    // true alongside the new write-position deny.
    let src = "const controller = new AbortController();\nsetTimeout(function() { console.log(\"aborted=\" + controller.signal.aborted); }, 0);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn unknown_field_on_handle_fails_closed() {
    // t3-m2 closure: an unrecognized field on a proven handle must E5506
    // (default-deny at the identifier choke point), never silently print 0.
    let src = "const c = new AbortController();\nconsole.log(c.reason2);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn signal_instanceof_abort_signal_folds_true() {
    let src = "const c = new AbortController();\nconsole.log(c.signal instanceof AbortSignal ? 1 : 0);\nconst s = c.signal;\nconsole.log(s instanceof AbortSignal ? 1 : 0);\n";
    assert_eq!(run_kali_run(src).trim(), "1\n1");
}

#[test]
fn instanceof_with_shadowed_abort_signal_stays_trapped() {
    // p03c precedent: a user binding shadowing the builtin must not hit the
    // allow lane; the blanket runtime trap fires instead.
    let src = "function AbortSignal() { return 1; }\nconst c = new AbortController();\nconsole.log(c.signal instanceof AbortSignal ? 1 : 0);\n";
    let out = run_kali(src);
    assert!(!out.status.success(), "must trap: {out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("instanceof"), "stderr: {stderr}");
}

#[test]
fn unproven_left_operand_instanceof_stays_trapped() {
    let src = "const x = 1;\nconsole.log(x instanceof AbortSignal ? 1 : 0);\n";
    let out = run_kali(src);
    assert!(!out.status.success(), "must trap: {out:?}");
}

#[test]
fn array_wrapped_signal_left_operand_instanceof_stays_trapped() {
    // Reject-don't-miscompile: `[c.signal]` is a JS array, NOT an AbortSignal,
    // so `[c.signal] instanceof AbortSignal` is `false` in JS. The left-proof
    // must NOT tunnel the single-element array literal (structurally a textless
    // one-child Value, like a grouping wrapper) into `c.signal` and wrongly
    // fold to true — it falls through to the runtime trap instead.
    let src = "const c = new AbortController();\nconsole.log([c.signal] instanceof AbortSignal ? 1 : 0);\n";
    let out = run_kali(src);
    assert!(!out.status.success(), "must trap: {out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("instanceof"), "stderr: {stderr}");
}

#[test]
fn controller_instanceof_abort_controller_stays_trapped() {
    // Inventoried for P3b, deliberately NOT implemented this stage.
    let src = "const c = new AbortController();\nconsole.log(c instanceof AbortController ? 1 : 0);\n";
    let out = run_kali(src);
    assert!(!out.status.success(), "must trap: {out:?}");
}

// --- Task 6: fail-closed enumeration wave (store sites + generic sinks) ----
//
// The P2 standing lesson executed up front: pin every store site and generic
// value sink for the abort-handle value class NOW, proving the Task-3
// position gate (`emit_abort_receiver_handle` / `admit_abort_handle_read`)
// covers the whole deny surface, not just the shapes earlier tasks happened
// to exercise.

#[test]
fn abort_handle_string_concat_fails_closed() {
    let src = "const c = new AbortController();\nconst s = c.signal;\nconsole.log(\"v=\" + c);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn abort_handle_template_interpolation_fails_closed() {
    let src = "const c = new AbortController();\nconst s = c.signal;\nconsole.log(`v=${c}`);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn abort_handle_arithmetic_fails_closed() {
    let src = "const c = new AbortController();\nconst s = c.signal;\nconsole.log(c + 1);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn abort_handle_identity_compare_fails_closed() {
    let src = "const c = new AbortController();\nconst s = c.signal;\nconsole.log(c === c ? 1 : 0);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn signal_identity_compare_fails_closed() {
    let src = "const c = new AbortController();\nconst s = c.signal;\nconsole.log(s === c.signal ? 1 : 0);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn abort_handle_json_stringify_fails_closed() {
    let src = "const c = new AbortController();\nconst s = c.signal;\nconsole.log(JSON.stringify(c));\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn abort_handle_return_position_fails_closed() {
    let src = "function f() { const k = new AbortController(); return k; }\nf();\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn abort_handle_argument_position_fails_closed() {
    let src = "const c = new AbortController();\nconst s = c.signal;\nfunction f(x) { return 1; }\nf(c);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn abort_handle_object_literal_field_fails_closed() {
    let src = "const c = new AbortController();\nconst s = c.signal;\nconst o = { h: c };\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn abort_handle_array_element_fails_closed() {
    let src = "const c = new AbortController();\nconst s = c.signal;\nconst a = [c];\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn abort_handle_growable_push_fails_closed() {
    let src = "const c = new AbortController();\nconst s = c.signal;\nconst a = [];\na.push(c);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn abort_handle_computed_member_read_fails_closed() {
    let src = "const c = new AbortController();\nconst s = c.signal;\nconst k = \"abort\";\nc[k]();\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn abort_handle_computed_member_write_fails_closed() {
    let src = "const c = new AbortController();\nconst s = c.signal;\nc[0] = 1;\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn signal_onabort_member_fails_closed() {
    let src = "const c = new AbortController();\nconst s = c.signal;\nconsole.log(s.onabort);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn signal_reason_member_fails_closed() {
    let src = "const c = new AbortController();\nconst s = c.signal;\nconsole.log(s.reason);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn signal_throw_if_aborted_fails_closed() {
    let src = "const c = new AbortController();\nconst s = c.signal;\ns.throwIfAborted();\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn abort_signal_static_timeout_fails() {
    // Leak found by this wave: before the Task 6 fix, an unrecognized
    // `AbortSignal.timeout(...)` call reached the generic "undefined call
    // target" fallback (`emitter.rs::push_placeholder_fallback_diagnostic`),
    // which is WARNING-only — the build exited 0 printing "1", silently
    // discarding the construct. Fixed at the choke point: `emit_call` now
    // denies any static method call on the unshadowed `AbortSignal` builtin
    // (`is_abort_signal_static_call`) before it can reach that fallback.
    // Observed stderr post-fix: "error[E5506]: AbortSignal static methods
    // (timeout/abort/any) are unavailable in the current phase: ...".
    let src = "const s = AbortSignal.timeout(5);\nconsole.log(1);\n";
    let out = run_kali(src);
    assert!(!out.status.success(), "must fail closed: {out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn abort_signal_static_abort_fails() {
    // Same leak/fix as `abort_signal_static_timeout_fails` (see its comment):
    // observed stderr post-fix carries the same "error[E5506]: AbortSignal
    // static methods (timeout/abort/any) are unavailable ..." text.
    let src = "const s = AbortSignal.abort();\nconsole.log(1);\n";
    let out = run_kali(src);
    assert!(!out.status.success(), "must fail closed: {out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn abort_signal_static_computed_literal_fails() {
    // Reviewer follow-up (Task 6 review): the dot-shape fix
    // (`is_abort_signal_static_call`) only matched a 1-child callee with a
    // known property `text`, so the COMPUTED shape `AbortSignal["timeout"](5)`
    // bypassed it entirely and hit the same silent generic-fallback leak the
    // dot-shape fix closed (verified pre-fix: exit 0, prints "ran:0"). The
    // recognizer now also matches the 2-child computed-member callee shape
    // (`[receiver, key]`, non-operator text) keyed on the receiver alone —
    // the property text/key value is irrelevant to the deny.
    let src = "const s = AbortSignal[\"timeout\"](5);\nconsole.log(\"ran:\" + s);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn abort_signal_static_computed_var_fails() {
    // Same leak/fix as `abort_signal_static_computed_literal_fails`, with the
    // computed key sourced from a variable instead of a literal — the
    // recognizer keys on the receiver's identity, not the key's shape, so
    // this must deny identically.
    let src = "const k = \"timeout\";\nconst s = AbortSignal[k](5);\nconsole.log(\"ran:\" + s);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn abort_signal_static_call_with_shadowed_receiver_not_denied() {
    // Shadow regression pin (reviewer follow-up; mirrors the sibling
    // `instanceof_with_shadowed_abort_signal_stays_trapped` pin for the
    // `instanceof` lane, which this static-call lane lacked). A user binding
    // named `AbortSignal` must take the normal user-value lane, NOT the
    // builtin deny — `is_abort_signal_static_call` refutes on the same
    // five-namespace shadow guard `instanceof_right_is_unshadowed` uses.
    // Observed pre-existing (and unchanged post-widening) behavior: this
    // reaches the generic "undefined call target" WARNING-only placeholder
    // fallback (`AbortSignal.timeout` is not a real member of the number
    // `5`), so the build still exits 0 printing "ran:0" — NOT E5506. Pinned
    // as-is (out of scope to also close the generic fallback here) so this
    // specific shadow invariant cannot silently regress.
    let src = "const AbortSignal = 5;\nconst s = AbortSignal.timeout(5);\nconsole.log(\"ran:\" + s);\n";
    assert_eq!(run_kali_run(src).trim(), "ran:0");
}

#[test]
fn abort_handle_inline_new_in_arg_position() {
    // Leak-shape triage (brief Step 2, second shape): a `new
    // AbortController()` that never becomes a `const` declarator init never
    // reaches the declarator-scoped real-cell lowering
    // (`emit/control_flow.rs`'s `is_const` intercept, which is the ONLY site
    // that calls `__alloc_global` for a real abort cell) — it falls through
    // the generic text-less aggregate path (`emit_aggregate_literal`), whose
    // unresolved `AbortController()` call target resolves through the
    // warning-only zero-placeholder fallback and is dropped. No real cell is
    // ever allocated, so there is no handle to leak: `f` receives a plain
    // placeholder `0`, not a raw abort-cell pointer. Confirmed sound as-is —
    // no fix needed, this pins the placeholder path against regression.
    let src = "function f(x) { return 1; }\nf(new AbortController());\nconsole.log(\"ok\");\n";
    assert_eq!(run_kali_run(src).trim(), "ok");
}

// --- Task 7: acceptance (byte-for-byte vs node) ------------------------------

/// Runs `node <main_path>` with `dir` as the working directory. The acceptance
/// fixture is valid, unmodified ES that node executes directly, so a straight
/// `node` run is a faithful oracle for "what should this program print".
/// (Copied from `module_namespace_link.rs:36-42`.)
fn node_output(dir: &std::path::Path, main_path: &std::path::Path) -> std::process::Output {
    Command::new("node")
        .current_dir(dir)
        .arg(main_path)
        .output()
        .expect("run node")
}

/// Stage P3 acceptance: the web-baseline smoke PREFIX — structuredClone (P2) +
/// AbortController/`instanceof AbortSignal` (P3) + EventTarget/CustomEvent
/// dispatch with a capturing listener that calls `controller.abort()` (Stage D)
/// — composed into one program and proven byte-for-byte against a real `node`
/// oracle. This is the stage's integration evidence: the abort surface works in
/// composition, not just in the isolated per-lane pins above.
///
/// FIXTURE PROVENANCE (two deliberate divergences from the live web-baseline
/// fixture `structured_clone_and_event_primitives_source` in `runtime_smoke.rs`):
///   1. MINUS the `new Event('tick')` / `event.type` self-check block. That
///      block trips a PRE-EXISTING, out-of-P3-scope gap (`Event.type` reads 0,
///      not 'tick'); it is the fixture's current fail-closed point and belongs
///      to a P-series follow-up, not this acceptance run.
///   2. MINUS everything from `URLSearchParams` onward (P4 URL / P5
///      TextEncoder scope — not yet implemented).
/// So: acceptance prefix = web-baseline fixture − Event-type block − (P4/P5 tail).
///
/// FIXTURE ADAPTATION (recorded for the Task-8 doc entry): the program body is
/// wrapped in `function main() { ... } main();` rather than run at module top
/// level. `controller` is captured by the `addEventListener` arrow, and the
/// ratified capture lane (Task 3 allowlist entry 3, pinned by
/// `captured_handle_full_roundtrip_in_listener`) is FUNCTION-scoped: the handle
/// is a function-local i64 cell pointer restored by-value into the callback env.
/// Reading a MODULE-scope binding from a closure is separately fail-closed
/// (E5506, "reading module binding from a function is only available for
/// compile-time-constant const initializers") — and in the write position
/// (`controller.abort()` inside the closure) it currently fails OPEN through the
/// E3100 zero-placeholder fallback (silent no-op → `aborted` stays 0). That
/// module-scope-capture asymmetry is a pre-existing gap outside P3's ratified
/// lanes; the function-scope wrap keeps this fixture squarely inside the
/// ratified shape. node prints identical stdout either way, so byte-for-byte
/// equality is preserved.
#[test]
fn acceptance_web_baseline_prefix_matches_node_byte_for_byte() {
    let src = r#"function main() {
  const original = { count: 1, values: [1, 2, 3] };
  const cloned = structuredClone(original);
  if (cloned === original || cloned.values === original.values) {
    throw new Error('structuredClone should deep-clone object graphs');
  }
  original.values.push(4);
  if (cloned.count !== 1 || cloned.values.join(',') !== '1,2,3') {
    throw new Error('unexpected structuredClone result');
  }
  const controller = new AbortController();
  if (!(controller.signal instanceof AbortSignal)) {
    throw new Error('expected AbortSignal from AbortController');
  }
  const target = new EventTarget();
  let count = 0;
  target.addEventListener('tick', () => {
    count += 1;
    controller.abort();
  });
  const dispatched = target.dispatchEvent(new CustomEvent('tick'));
  if (!dispatched || count !== 1 || !controller.signal.aborted) {
    throw new Error('unexpected event primitive behavior');
  }
  console.log('web baseline prefix ok');
}
main();
"#;
    let dir = tempdir().expect("tempdir");
    let main_path = dir.path().join("main.js");
    fs::write(&main_path, src).expect("write");
    let kali = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&main_path)
        .output()
        .expect("run kali");
    assert!(
        kali.status.success(),
        "kali stderr: {}",
        String::from_utf8_lossy(&kali.stderr)
    );
    let node = node_output(dir.path(), &main_path);
    assert!(
        node.status.success(),
        "node stderr: {}",
        String::from_utf8_lossy(&node.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&kali.stdout),
        String::from_utf8_lossy(&node.stdout),
        "kali must byte-match node"
    );
    assert_eq!(
        String::from_utf8_lossy(&kali.stdout).trim(),
        "web baseline prefix ok"
    );
}
