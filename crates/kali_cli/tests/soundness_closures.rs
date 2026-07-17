//! Stage C (closures) C1 — synchronous scalar capture end-to-end.
//!
//! A nested function that shares an enclosing scalar local with its owner must
//! read and write the SAME storage cell as the owner (JS captures variables,
//! not values). Before C1 the write path hard-failed E5506 and the read path
//! silently produced `0`; these tests pin the sound behavior byte-for-byte
//! against node.

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

/// Run a `*.test.js` fixture through the `kali test` harness. Registered
/// `Kali.test(...)` callbacks are the ONE deferred-callback surface codegen
/// actually emits (`test_register`); they are invoked LATER (after the
/// registering function has returned) via `invoke_callback`, exactly the
/// deferred path Phase C3 threads `env_ptr` through. `kali run` cannot exercise
/// this because codegen emits no call to `queueMicrotask`/`setTimeout` (those
/// host imports exist but no generated module imports them), so the brief's
/// `queueMicrotask` fixture never schedules anything — see the Task 6 report.
fn run_kali_test(source: &str) -> std::process::Output {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("main.test.js");
    fs::write(&path, source).expect("write source");
    Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&path)
        .output()
        .expect("run kali test")
}

/// Assert the program is REJECTED with `E5506` (the reject-don't-miscompile
/// contract) rather than silently producing a value.
fn assert_e5506(source: &str) {
    let out = run_kali(source);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "expected E5506 rejection, but the program succeeded with stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        stderr.contains("E5506"),
        "expected E5506 in stderr, got: {stderr}"
    );
}

/// Nested function mutates an enclosing scalar local; the enclosing scope reads
/// the mutation back. Pre-C1: `c += 1` hard-fails E5506. node prints 2.
#[test]
fn sync_scalar_capture_write_is_visible_to_owner() {
    let out = run_kali(
        "function outer(){ let c = 0; function inc(){ c += 1; } inc(); inc(); console.log(c); } outer();\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "2\n");
}

/// C1 review Finding 1 (owner-repr capture gate). The OWNER declares an `F64`
/// scalar (`let c = 0.5`), so `lower.rs` does NOT promote it to an env cell (it
/// stays an f64 WASM local). A nested `c += 1` must therefore be REJECTED, not
/// silently routed to a phantom i64 env cell the owner never allocated. Before
/// the fix the capturer gated on ITS OWN repr namespace — where the undeclared
/// name `c` defaults to `I64` — and emitted a cell write the owner never read,
/// printing `0.5` (node: `2.5`). The owner's F64 verdict now decides: E5506.
#[test]
fn capture_gate_owner_f64_compound_assign_rejects_not_miscompiles() {
    assert_e5506(
        "function outer(){ let c = 0.5; function inc(){ c += 1; } inc(); inc(); console.log(c); } outer();\n",
    );
}

/// Finding 1, update-expression shape (`c++` on an owner-`F64` capture). Same
/// divergence class as the compound-assign pin above — must reject, not write a
/// phantom cell.
#[test]
fn capture_gate_owner_f64_update_expr_rejects_not_miscompiles() {
    assert_e5506(
        "function outer(){ let c = 0.5; function inc(){ c++; } inc(); console.log(c); } outer();\n",
    );
}

/// C1 review Finding 2 (module-global precedence in the update path). A
/// module-scope binding must never resolve depth-0 against `current_env` as if
/// it were an env cell. `c++` on a module global has no dedicated lowering, so
/// it is REJECTED (E5506) — the point is it does NOT silently write raw memory
/// at `8 + offset`. The write path (`c += 1`) already routes to the
/// module-global `GlobalSet` lane and is pinned below.
#[test]
fn module_global_update_does_not_route_to_env_cell() {
    assert_e5506("let c = 0;\nfunction f(){ c++; }\nf();\nconsole.log(c);\n");
}

/// Finding 2 companion: a module-scope compound-assign still reaches the
/// module-global path (prints `1`), unaffected by the env-cell machinery.
#[test]
fn module_global_compound_assign_still_routes_to_global() {
    let out = run_kali("let c = 0;\nfunction f(){ c += 1; }\nf();\nconsole.log(c);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1\n");
}

/// Nested function READS an enclosing scalar. Pre-C1: silently returns 0. node
/// prints 7.
#[test]
fn sync_scalar_capture_read_returns_value_not_zero() {
    let out = run_kali(
        "function outer(){ let c = 7; function rd(){ return c; } console.log(rd()); } outer();\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "7\n");
}

/// Step 6 permanent re-mask pin: calling `outer()` twice must NOT accumulate
/// the env across activations — each call gets a fresh env record, so the
/// second call prints `2`, not `4`. Guards the per-activation prologue alloc.
#[test]
fn sync_scalar_capture_env_does_not_leak_across_activations() {
    let out = run_kali(
        "function outer(){ let c = 0; function inc(){ c += 1; } inc(); inc(); console.log(c); } outer(); outer();\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "2\n2\n");
}

/// Step 6 permanent re-mask pin for the epilogue/return RESTORE (not the
/// per-call alloc, which the twice-called pin above already covers). `outer`
/// owns an env; between two `inc()` calls it invokes a SIBLING function `sib`
/// that ALSO owns an env — `sib`'s prologue clobbers `current_env`. Only a
/// correct restore on `sib`'s exit puts `current_env` back to `outer`'s record,
/// so the final `inc()` and the read of `c` address `outer`'s cell, not `sib`'s.
/// A broken restore leaves `current_env` pointing at `sib`'s freed record and
/// the program prints the wrong number. node prints 2.
#[test]
fn sync_scalar_capture_restore_survives_sibling_env_owner() {
    let out = run_kali(
        "function outer(){ let c = 0; function inc(){ c += 1; } function sib(){ let d = 5; function bump(){ d += 1; } bump(); return d; } inc(); sib(); inc(); console.log(c); } outer();\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "2\n");
}

/// Stage C C2 (heap captures). A nested function captures an enclosing HEAP
/// object and reads a field. The captured object must heap-materialize into the
/// never-reset global region (owner-keyed lockstep promotion of the object env
/// cell) and the capturer's member access must resolve the loaded pointer's
/// shape (owner object-shape propagated into the capturer's repr namespace).
/// Pre-C2: silent `0` (the object never materializes and `rd`'s `obj.n` has no
/// object shape). node prints 1.
#[test]
fn sync_heap_capture_reads_field() {
    let out = run_kali(
        "function outer(){ let obj = { n: 1 }; function rd(){ return obj.n; } console.log(rd()); } outer();\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1\n");
}

/// Stage C C2 heap-write behavior pin (field write through the capture). A
/// nested function writes a field of the captured object (`obj.n = 2`); the
/// owner reads the mutation back. Both address the SAME env-cell pointer, so
/// the write is visible — byte-parity with node (2). Documents that the C2
/// read plumbing also makes captured field WRITES correct (the base loads the
/// shared pointer at both sites), not just reads.
#[test]
fn sync_heap_capture_field_write_visible_to_owner() {
    let out = run_kali(
        "function outer(){ let obj = { n: 1 }; function wr(){ obj.n = 2; } wr(); console.log(obj.n); } outer();\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "2\n");
}

/// Stage C C2 (env chains, depth > 1). Grandparent read: `c` reads `a`'s `g`
/// through an INTERMEDIATE `b` that owns no cell. `b` is transparent to the env
/// chain (a no-cell function allocates no record and does not touch
/// `current_env`, spec §3.4), so when `c` runs `current_env` still points at
/// `a`'s record directly — env-walk depth 0. The MIR capture depth counts
/// env-OWNING ancestors (only `a`), so it is 1, and the existing depth-1 access
/// lane resolves it. Pre-C2: the read silently produced `0`. node prints 5.
#[test]
fn env_chain_grandparent_read() {
    let out = run_kali(
        "function a(){ let g = 5; function b(){ function c(){ return g; } return c(); } console.log(b()); } a();\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "5\n");
}

/// Stage C C2 (env chains) — a GENUINE parent-pointer walk (env-walk depth 1).
/// The capturer `c` OWNS a promotable env of its own (`k`, captured by `d`), so
/// when `c` runs `current_env` is `c`'s OWN record, whose parent header points
/// at `a`'s record (the intermediate `b` owns no cell and is transparent). To
/// read `a`'s `g`, `c` must follow ONE parent link — `emit_env_base_addr` with
/// depth 1. Pre-this-task the env-owning capturer fell through to baseline and
/// `g` read as `0` (kali printed 10 = 0 + d()); the sound value is 5 + 10 = 15.
#[test]
fn env_chain_owning_capturer_parent_walk() {
    let out = run_kali(
        "function a(){ let g = 5; function b(){ function c(){ let k = 10; function d(){ return k; } return g + d(); } return c(); } return b(); } console.log(a());\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "15\n");
}

/// Stage C C2 heap-write scope pin (whole-object REASSIGNMENT through the
/// capture). Reassigning the captured binding itself (`obj = {…}` from a nested
/// function) is OUT of C2's read scope: the scalar-only write gate keeps it on
/// the baseline path, which rejects `=` on an object reference FAIL-CLOSED
/// (E5506), never a silent miscompile. (node would print 9; kali rejects rather
/// than mislower — reject-don't-miscompile.)
#[test]
fn sync_heap_capture_reassign_through_capture_rejected() {
    assert_e5506(
        "function outer(){ let obj = { n: 1 }; function wr(){ obj = { n: 9 }; } wr(); console.log(obj.n); } outer();\n",
    );
}

/// Phase C3 (deferred host threading). A `Kali.test(...)` callback captures an
/// enclosing scalar (`base`) of its registering function and reads it back when
/// the host invokes it LATER — after the registering function has returned. The
/// callback runs through `invoke_callback`, which must set `current_env` to the
/// `env_ptr` captured at registration time (the registering activation's env
/// record, threaded through the `test_register` import) so the read resolves to
/// the live cell instead of env 0.
///
/// Two independent suites with different captured values assert BOTH that each
/// callback runs with its OWN env AND that `invoke_callback` restores the prior
/// `current_env` between callbacks (no env leaks across the queue). Pre-C3 both
/// callbacks read `current_env` = 0 and printed `a=0` / `b=0` (silent
/// read-zero — the C3 deferred-callback bug). node prints `a=41` / `b=7`.
#[test]
fn deferred_test_callback_runs_with_its_env() {
    let out = run_kali_test(
        "function suiteA(){ let base = 41; Kali.test(\"a\", function(){ console.log(\"a=\"+base); }); }\nfunction suiteB(){ let base = 7; Kali.test(\"b\", function(){ console.log(\"b=\"+base); }); }\nsuiteA();\nsuiteB();\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("a=41"), "stdout: {}", s);
    assert!(s.contains("b=7"), "stdout: {}", s);
}

// ============================================================================
// Phase C4 — fail-closed boundaries (brief Steps 1-2)
//
// These pin the boundaries around the lowered closure surface. Every case below
// was run on HEAD (6bb617b11) AND on the pre-Stage-C base (a57cd09d5): the
// behaviors are BYTE-IDENTICAL across the two, i.e. Stage C introduced NONE of
// them. The array-callback and object-literal-as-direct-call-arg cases already
// fail closed E5506; the remaining "exotic-position" cases are PRE-EXISTING
// indirect-function-value invocation miscompiles (a function VALUE reached via
// `arr[i]`/`o.f`/`o?.f` and then called returns a clean `0`) — reproduced with
// AND without a capture, so they are NOT capture-lowering failures and get no
// new guard (that would be new indirect-call lowering-boundary work, out of
// this task's scope). They are pinned here as tripwires and are recorded as a
// pre-existing follow-up. Crucially, the F-AB-2 danger (a string/growable
// capture in such a body silently lowering to i64) is NOT reachable: those
// forms fail closed E3200/E5506 or return the same pre-existing clean `0`.
// ============================================================================

/// Brief Step 1, fixture 1. An array per-element callback that captures an
/// enclosing local — the array-callback ABI is a separate future stage. Already
/// fails closed E5506 (array callback methods were E5506-gated in Stage B); this
/// is a permanent PIN, no guard was needed.
#[test]
fn array_callback_capture_fails_closed() {
    let out = run_kali(
        "function outer(){ let base = 10; let r = [1,2,3].map(function(x){ return x + base; }); console.log(r.join(\",\")); } outer();\n",
    );
    assert!(
        !out.status.success(),
        "expected E5506, got stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("E5506"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Brief Step 1, fixture 2 (F-AB-2 exotic position). A fn-expr inside an object
/// literal passed DIRECTLY as a call argument, capturing an enclosing local,
/// with the callee actually invoking it (`o.f()`). Already fails closed E5506
/// ("an object literal passed directly as a call argument is unavailable …";
/// `repr_infer.rs`) — MUST NOT silently print the wrong captured value. This is
/// the exact object-literal-as-direct-call-arg gap the F-AB-2 lockstep gap test
/// characterizes on the repr side. Permanent PIN, no guard needed.
#[test]
fn exotic_object_literal_arg_capture_fails_closed() {
    let out = run_kali(
        "function sink(o){ return o.f(); } function outer(){ let c = 1; console.log(sink({ f: function(){ return c; } })); } outer();\n",
    );
    assert!(
        !out.status.success(),
        "expected E5506, got stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    // And it certainly must not silently print a wrong captured value.
    assert!(
        !String::from_utf8_lossy(&out.stdout).contains('1'),
        "stdout leaked a value: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// PRE-EXISTING tripwire (NOT capture-introduced — identical pre/post Stage C,
/// and reproduces WITHOUT a capture). A fn-expr stored in an array literal and
/// invoked indirectly (`arr[0]()`) returns a clean `0` instead of the captured
/// value. This is an indirect-function-value invocation miscompile that predates
/// the closures project; it is pinned here so a future stage that wires
/// first-class function values (either fixing it to print `9` or failing it
/// closed E5506) trips this test and revisits the pin. The load-bearing
/// soundness property held today: it does NOT leak a garbage/heap value.
#[test]
fn exotic_array_element_indirect_call_is_preexisting_zero_not_garbage() {
    let out = run_kali(
        "function outer(){ let c = 9; let arr = [function(){ return c; }]; let g = arr[0]; console.log(g()); } outer();\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // Pre-existing behavior: clean `0` (node prints `9`). No garbage leak.
    assert_eq!(String::from_utf8_lossy(&out.stdout), "0\n");
}

/// PRE-EXISTING tripwire, optional-chain form (`o?.f()`). Same pre-existing
/// indirect-invocation `0`, reproduced with and without a capture. Pinned so a
/// future first-class-function stage revisits it.
#[test]
fn exotic_optional_chain_indirect_call_is_preexisting_zero_not_garbage() {
    let out = run_kali(
        "function outer(){ let c = 5; let o = { f: function(){ return c; } }; console.log(o?.f()); } outer();\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "0\n");
}

/// F-AB-2 danger NOT reachable: a STRING capture in an exotic-position body
/// fails closed (E3200) rather than silently lowering the string handle to i64.
/// This is the concrete "string-element in an exotic body would silently lower
/// to i64" scenario the F-AB-2 note warns about — pinned as fail-closed.
#[test]
fn exotic_string_capture_in_array_element_fails_closed() {
    let out = run_kali(
        "function outer(){ let s = \"hi\"; let arr = [function(){ return s + \"!\"; }]; console.log(arr[0]()); } outer();\n",
    );
    assert!(
        !out.status.success(),
        "expected fail-closed, got stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("E3200") || stderr.contains("E5506"),
        "stderr: {stderr}"
    );
}

/// F-AB-2 danger NOT reachable, growable form: a growable-array capture in an
/// exotic-position body fails closed E5506 (the "capture by a nested function"
/// growable gate), not a silent i64 length.
#[test]
fn exotic_growable_capture_in_array_element_fails_closed() {
    assert_e5506(
        "function outer(){ let a = []; a.push(1); let arr = [function(){ return a.length; }]; console.log(arr[0]()); } outer();\n",
    );
}

// ============================================================================
// Task 8 — adversarial whole-stage sweep (brief Step 3 + controller amendments)
//
// Every fixture below was run on the FINAL Stage C binary AND on node v26.5.0.
// The escaping-closure and deferred-surface cases were additionally run on the
// pre-Stage-C base (a57cd09d5) to classify each divergence as PRE-EXISTING vs
// Stage-C-introduced. Full node-vs-kali evidence and the base cross-check are
// recorded in docs/superpowers/followups/stageC-closures-triage.md (§6-§8).
// ============================================================================

/// Brief Step 2 / amendment 2 — recursion / distinct envs, ESCAPING form.
/// Each activation of `make` should create a distinct closure over its own `n`;
/// node prints `10 20`. But the closure is RETURNED out of `make` and invoked
/// later via a plain call (`a()` / `b()`): at that call site `current_env` is
/// the module env, not `make`'s freed activation record, so the promoted cell
/// reads `0`. kali prints `0 0` — a PRE-EXISTING silent miscompile,
/// BYTE-IDENTICAL on the pre-Stage-C base a57cd09d5, in the same escaping /
/// first-class-function-value class as the `arr[0]()` tripwire above. The
/// load-bearing soundness property holds: clean `0`, NOT a garbage/stale-heap
/// leak. Pinned so the escaping-capture-region follow-up (which will either fix
/// this to `10 20` or fail it closed E5506) trips this test and revisits it.
#[test]
fn recursion_distinct_envs_is_preexisting_escaping_zero() {
    let out = run_kali(
        "function make(n){ return function(){ return n; }; } let a = make(10); let b = make(20); console.log(a() + \" \" + b());\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // node: "10 20\n". kali (pre-existing escaping-closure zero): "0 0\n".
    assert_eq!(String::from_utf8_lossy(&out.stdout), "0 0\n");
}

/// Amendment 3a — returned-closure LATE READ. A closure is RETURNED out of
/// `outer()` and invoked AFTER outer's other locals (the loop scratch `filler`)
/// would have been arena-reclaimed. This is the shape that WOULD prove captured
/// cells live in the never-reset region (Task 4 reviewer's unproven risk) — IF
/// escaping closures resolved their env. They do not: the plain later call reads
/// `current_env` = module env, so `rd()` returns `0`, not `7`. PRE-EXISTING
/// (byte-identical on base a57cd09d5), clean `0`, no garbage leak. The
/// never-reset-region SURVIVAL property is instead proven by the DEFERRED
/// (`Kali.test`) path — see `deferred_test_callback_runs_with_its_env`, where a
/// cell written in an already-returned suite is read correctly during a later
/// drain.
#[test]
fn returned_closure_late_read_is_preexisting_escaping_zero() {
    let out = run_kali(
        "function outer(){ let c = 7; let filler = 0; for (let i=0;i<3;i++){ filler += i; } function rd(){ return c; } return rd; } let g = outer(); console.log(g());\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // node: "7\n". kali (pre-existing escaping-closure zero): "0\n".
    assert_eq!(String::from_utf8_lossy(&out.stdout), "0\n");
}

/// Brief Step 3 — a nested function that captures NOTHING. `noop` shares no
/// binding with `outer`, so no env record is allocated and `current_env` is
/// never touched; the direct call resolves normally. Matches node (`42`). Guards
/// against the env machinery firing for capture-free nested functions.
#[test]
fn capture_free_nested_fn_allocates_no_env() {
    let out = run_kali(
        "function outer(){ function noop(){ return 42; } console.log(noop()); } outer();\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "42\n");
}

/// Amendment 3b — `current_env` drain-cleanliness. Two capture-owning suites
/// (`suiteA`/`suiteB`) register callbacks that read their own `base`; a THIRD
/// callback then performs a fresh capture-owning call (`outer()`, which owns an
/// env of its own) DURING the drain, AFTER the first two callbacks have run and
/// restored `current_env`. It resolves correctly (`c=2`), proving
/// `invoke_callback` restores `current_env` cleanly between drained callbacks —
/// no env leaks across the queue. node order matches (a=41, b=7, c=2). On base
/// a57cd09d5 this FAILED closed (E5506 on the `c += 1` capture-write); Stage C's
/// capture-write lowering is what makes it run.
#[test]
fn deferred_drain_cleanliness_post_drain_capture_resolves() {
    let out = run_kali_test(
        "function suiteA(){ let base = 41; Kali.test(\"a\", function(){ console.log(\"a=\"+base); }); }\nfunction suiteB(){ let base = 7; Kali.test(\"b\", function(){ console.log(\"b=\"+base); }); }\nfunction suiteC(){ Kali.test(\"c\", function(){ function outer(){ let c = 0; function inc(){ c += 1; } inc(); inc(); return c; } console.log(\"c=\"+outer()); }); }\nsuiteA();\nsuiteB();\nsuiteC();\n",
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("a=41"), "stdout: {s}");
    assert!(s.contains("b=7"), "stdout: {s}");
    assert!(s.contains("c=2"), "stdout: {s}");
}

// ---------------------------------------------------------------------------
// Deferred-surface fail-closed pins (rows o/p/q — Concern-2 fix). Codegen emits
// NO call to `queueMicrotask` / `setTimeout` / `setInterval` /
// `addEventListener` (Task 6 finding: the host imports exist but no generated
// module imports them; such a call reaches the generic zero-placeholder
// fallback). So a scheduled callback is silently dropped along with its capture.
//
// UNMASK → RE-CLOSE: on base a57cd09d5 these CAPTURING callbacks FAILED CLOSED
// (E5506 on the callback's `base += 1` capture-write). Stage C lowered that
// capture-write, removing the E5506 and unmasking the pre-existing scheduler
// no-op, so the program RAN and silently dropped the callback (Task 8 Concern
// 2). The Concern-2 fix restores the rejection at the single choke point all
// four surfaces converge on (`emit/call.rs::emit_call`, guarded by
// `is_undrained_scheduling_surface` + `call_has_capturing_closure_arg`): a
// CAPTURING callback (its `derive_env_plans` `captured` non-empty) passed to an
// un-emittable scheduling surface now fails closed E5506. Module-scope and
// non-capturing callbacks — silently dropped at base too — are NOT this class
// and stay running (pinned by the two boundary guards below).
// ---------------------------------------------------------------------------

/// Row o. queueMicrotask capturing callback → NOW RUNS (Stage D task D2).
/// Pre-Stage-D: fail closed E5506 (Stage C Concern-2 guard — codegen emitted
/// no call to `queueMicrotask`, so a capturing callback would have been
/// silently dropped along with its captured env). D2 wired the real
/// registration lane (env_ptr ABI + env_safety registration edges), so the
/// capturing callback is deferred and runs with its owner's env record after
/// `outer()` returns. node v26.5.0: "sync=5\nmt=6\n".
#[test]
fn deferred_queue_microtask_capturing_callback_now_runs() {
    let out = run_kali(
        "function outer(){ let base = 5; queueMicrotask(function(){ base += 1; console.log(\"mt=\"+base); }); console.log(\"sync=\"+base); } outer();\n",
    );
    assert!(
        out.status.success(),
        "capturing microtask must now run; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "sync=5\nmt=6\n");
}

/// Row p. setTimeout(cb, 0) capturing callback — Stage D task D2 wired the
/// timer-set registration emit (`emit_timer_set_call`), so this now RUNS
/// with its owner's env record instead of failing closed. Named
/// `..._row_p_now_runs` (not the brief's plain `_now_runs`) because that
/// exact name collides with the Step 1 ordering-matrix fixture
/// `deferred_set_timeout_capturing_callback_now_runs`, which independently
/// covers the same "capturing setTimeout callback now runs" class — kept
/// distinct here for the row-p provenance trail.
/// node v26.5.0 (verified against this exact source): "sync=5\nst=6\n".
#[test]
fn deferred_set_timeout_capturing_callback_row_p_now_runs() {
    let out = run_kali(
        "function outer(){ let base = 5; setTimeout(function(){ base += 1; console.log(\"st=\"+base); }, 0); console.log(\"sync=\"+base); } outer();\n",
    );
    assert!(
        out.status.success(),
        "capturing setTimeout callback must now run; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "sync=5\nst=6\n");
}

/// Row q. addEventListener capturing callback → fail closed. (The program also
/// uses `EventTarget`/`CustomEvent`/`dispatchEvent`, which emit E3100 fallback
/// WARNINGS; the load-bearing rejection is the E5506 on the capturing listener.)
#[test]
/// DELIBERATE CAPABILITY FLIP (Stage D event lane, Task 4): this fixture's
/// receiver is a declarator-bound `let t = new EventTarget()` (in-lane), so the
/// capturing listener now registers and — because the fixture dispatches — RUNS
/// synchronously, mutating the captured `base` through its env cell. Previously
/// `addEventListener` took the Stage C backstop and failed closed E5506. The
/// env_safety member edge (registration inherits Record(owner)) keeps the
/// capture sound. node v26.5.0: "ev=6\nsync=6\n".
#[test]
fn deferred_add_event_listener_capturing_callback_now_runs() {
    let out = run_kali(
        "function outer(){ let base = 5; let t = new EventTarget(); t.addEventListener(\"tick\", function(){ base += 1; console.log(\"ev=\"+base); }); t.dispatchEvent(new CustomEvent(\"tick\")); console.log(\"sync=\"+base); } outer();\n",
    );
    assert!(
        out.status.success(),
        "capturing listener on an in-lane EventTarget must run; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "ev=6\nsync=6\n");
}

/// Boundary pin: module-scope capturing listener registers and fires via
/// dispatch (the bg-series analog for the event lane).
/// node v26.5.0: "sync=0\nev=1\n".
#[test]
fn event_module_scope_capture_listener_now_runs() {
    let out = run_kali(
        r#"let base = 0;
const t = new EventTarget();
t.addEventListener("tick", function () {
  base += 1;
  console.log("ev=" + base);
});
console.log("sync=" + base);
t.dispatchEvent(new CustomEvent("tick"));
"#,
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "sync=0\nev=1\n");
}

/// Boundary guard 1: a MODULE-SCOPE callback mutating a module global via
/// `queueMicrotask` captures NO env cell (`derive_env_plans` excludes module
/// globals → empty `captured`), so it never fell to the Stage C Concern-2
/// guard. Pre-Stage-D: the callback was SILENTLY DROPPED (codegen emitted no
/// `queueMicrotask` call), so only `sync=0` printed. Stage D task D2 wired the
/// registration lane, so the callback now RUNS and mutates the module global
/// through its WASM global (not an env cell). node v26.5.0: "sync=0\nmt=1\n".
#[test]
fn deferred_queue_microtask_module_scope_capture_now_runs() {
    let out = run_kali(
        "let count = 0; queueMicrotask(function(){ count += 1; console.log(\"mt=\"+count); }); console.log(\"sync=\"+count);\n",
    );
    assert!(
        out.status.success(),
        "module-scope microtask must not fail closed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "sync=0\nmt=1\n");
}

/// Boundary guard 2: a NON-CAPTURING function-scope callback passed to
/// `queueMicrotask` captures nothing (empty `captured`), so it never fell to
/// the Stage C Concern-2 guard. Pre-Stage-D: SILENTLY DROPPED (codegen emitted
/// no `queueMicrotask` call), so only `sync` printed. Stage D task D2 wired the
/// registration lane, so the callback now RUNS during the post-`_start` drain.
/// node v26.5.0: "sync\nmt\n".
#[test]
fn deferred_queue_microtask_non_capturing_callback_now_runs() {
    let out = run_kali(
        "function outer(){ queueMicrotask(function(){ console.log(\"mt\"); }); console.log(\"sync\"); } outer();\n",
    );
    assert!(
        out.status.success(),
        "non-capturing microtask must not fail closed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "sync\nmt\n");
}

/// Row q2. setInterval capturing callback (Finding B) — Stage D task D2 wired
/// the timer-set registration emit, so this now RUNS with its owner's env
/// record instead of failing closed.
///
/// ADAPTATION from the brief's literal source: the original fixture
/// (`setInterval(function(){ base += 1; ...}, 0)` with NO `clearInterval`)
/// never terminates once the capturing callback actually resolves and runs —
/// node-verified: it ticks forever (`iv=6`, `iv=7`, `iv=8`, ... unbounded,
/// confirmed via a 3s timeout that still showed `iv=1834` and rising), and
/// under kali it would hit the SAME bounded-drain "did not quiesce" trap as
/// `deferred_uncleared_interval_fails_loudly_not_hangs` — so the brief's
/// claimed terminating output ("sync=5\niv=6\n") is not reachable from that
/// exact source either in node or in kali. Added a `clearInterval(t)` inside
/// the callback (self-clearing after one tick) to make the fixture
/// deterministic and terminating while preserving the row's essential shape
/// (an inline capturing callback passed directly to `setInterval`); this adds
/// a second captured binding (the timer id `t`) alongside `base`, which does
/// not change the provenance class under test. node v26.5.0 (verified against
/// this exact modified source): "sync=5\niv=6\n".
#[test]
fn deferred_set_interval_capturing_callback_row_q2_now_runs() {
    let out = run_kali(
        "function outer(){ let base = 5; const t = setInterval(function(){ base += 1; console.log(\"iv=\"+base); clearInterval(t); }, 0); console.log(\"sync=\"+base); } outer();\n",
    );
    assert!(
        out.status.success(),
        "capturing setInterval callback must now run; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "sync=5\niv=6\n");
}

/// Row q3 (Finding C) — INDIRECT capturing callback via a binding: the
/// callback is a function VALUE held in a local (`let cb = function(){…}`)
/// and passed by name (`setTimeout(cb, 0)`), not inline. The resolver
/// resolves `cb` to its `__kali_fn_N` plan through declaration provenance
/// (`fn_valued_locals`); Stage D task D2 wired the timer-set registration
/// emit, so this now RUNS with its owner's env record instead of failing
/// closed.
///
/// ADAPTATION from the brief's literal source: the original fixture printed
/// `console.log(base)` AFTER `setTimeout(cb, 0)` but OUTSIDE the callback —
/// that print observes only the SYNCHRONOUS pre-tick state (`base` is still
/// 0 at that point; node-verified real output for that exact source is
/// "0\n", not the brief's claimed "1"), so it can never observe whether the
/// deferred callback actually ran. Moved the print INSIDE the callback body
/// (after the `base += 1` capture-write, dropping the now-redundant outer
/// print) so the assertion proves the deferred capturing callback executed
/// with its captured environment, matching the brief's intended "1" output.
/// node v26.5.0 (verified against this exact modified source): "1\n".
#[test]
fn deferred_set_timeout_indirect_capturing_callback_row_q3_now_runs() {
    let out = run_kali(
        "function outer(){ let base = 0; let cb = function(){ base += 1; console.log(base); }; setTimeout(cb, 0); } outer();\n",
    );
    assert!(
        out.status.success(),
        "indirect capturing setTimeout callback must now run; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1\n");
}

/// Boundary guard 3 (Finding C boundary, bg3) — INDIRECT NON-capturing
/// callback via a binding: the local holds a closure that captures NOTHING,
/// so its plan has an empty `captured` and the default-deny guard never fired
/// (this shape was already provably safe pre-D2). Pre-Stage-D-task-D2 the
/// callback was still SILENTLY DROPPED because codegen emitted no `setTimeout`
/// call at all (the `is_undrained_scheduling_surface` fallback), so only
/// `sync\n` printed. Stage D task D2 wired the registration lane, so the
/// callback now RUNS during the drain — full node-parity stdout.
/// node v26.5.0 (verified against this exact source): "sync\ncb\n".
#[test]
fn deferred_set_timeout_indirect_non_capturing_callback_now_runs() {
    let out = run_kali(
        "function outer(){ let cb = function(){ console.log(\"cb\"); }; setTimeout(cb, 0); console.log(\"sync\"); } outer();\n",
    );
    assert!(
        out.status.success(),
        "indirect non-capturing setTimeout must not fail closed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "sync\ncb\n");
}

/// Reassignment-stale provenance — CLOSED by the stage-review default-deny
/// (IMPORTANT-1). `fn_valued_locals` is recorded only at DECLARATOR-emit time,
/// so a REASSIGNMENT (`let cb = function(){}; cb = function(){ base += 1; }`)
/// leaves the stale NON-capturing mapping in place; the pre-fix guard resolved
/// `cb` to the original (empty-`captured`) plan, did NOT fire, and the
/// capturing callback was silently dropped (this test previously PINNED that
/// fail-open as a tripwire: success, `0\n`; node prints `1`). The default-deny
/// guard now refuses to resolve ANY name that is reassigned or re-declared in
/// the function (`unstable_provenance_names`), so this fails closed E5506 —
/// matching base a57cd09d5, which rejected the `base += 1` capture-write E5506.
#[test]
fn deferred_reassigned_callback_provenance_fails_closed() {
    assert_e5506(
        "function outer(){ let base = 0; let cb = function(){}; cb = function(){ base += 1; }; setTimeout(cb, 0); console.log(base); } outer();\n",
    );
}

// ============================================================================
// Stage-review fix wave (2026-07-16) — dynamic-env safety gate (CRITICAL) +
// scheduling-surface default-deny (IMPORTANT-1) + Kali.test fallback
// (IMPORTANT-2). Every fixture below was verified RED on pre-fix HEAD
// 3a1545b95 (the corrupted/silent output recorded in its doc comment) and on
// the pre-Stage-C base a57cd09d5 all capture shapes were E5506.
// ============================================================================

/// Run a `*.test.js` fixture and assert E5506 rejection (test-lane twin of
/// `assert_e5506`).
fn assert_e5506_test(source: &str) {
    let out = run_kali_test(source);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "expected E5506 rejection, but the harness succeeded with stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        stderr.contains("E5506"),
        "expected E5506 in stderr, got: {stderr}"
    );
}

/// CRITICAL reproducer 1 — sibling-invoked capture WRITE corrupts a foreign
/// cell. `inc` captures `outer`'s `c`; it is invoked from inside `sib`, a
/// SIBLING env-owner whose record is the active `current_env` at that point,
/// so `inc`'s cell write landed in `sib`'s `d` cell. Pre-fix HEAD printed
/// `102` / `0` (node: `101` / `1`) — silent cross-binding memory corruption.
/// The dynamic-env safety gate now rejects the unprovable invocation E5506
/// (base a57cd09d5 rejected this program E5506 at the capture-write).
#[test]
fn dynamic_env_sibling_write_capturer_fails_closed() {
    assert_e5506(
        "function outer(){ let c=0; function inc(){ c+=1; } function sib(){ let d=100; function bump(){ d+=1; } bump(); inc(); return d; } console.log(sib()); console.log(c); } outer();\n",
    );
}

/// CRITICAL reproducer 2 — sibling-invoked capture READ resolves a foreign
/// cell. `rd` captures `outer`'s `c` (7) but, invoked from inside sibling
/// env-owner `sib`, read `sib`'s `d` cell instead: pre-fix HEAD printed `101`
/// (node: `7`). Fails closed E5506 (base rejected E5506).
#[test]
fn dynamic_env_sibling_read_capturer_fails_closed() {
    assert_e5506(
        "function outer(){ let c=7; function rd(){ return c; } function sib(){ let d=100; function bump(){ d+=1; } bump(); return rd(); } console.log(sib()); } outer();\n",
    );
}

/// CRITICAL reproducer 3 — an ENV-OWNING capturer invoked from a sibling
/// env-owner. `cap` owns its own record (cell `k`) AND captures `outer`'s `c`
/// through its parent link; called from `sib`, its record's parent is `sib`'s
/// record, so the one-hop walk read `sib`'s `d` cell: pre-fix HEAD printed
/// `11` (node: `7`). Fails closed E5506 (base rejected E5506).
#[test]
fn dynamic_env_owning_capturer_from_sibling_env_owner_fails_closed() {
    assert_e5506(
        "function outer(){ let c=6; function cap(){ let k=1; function g(){ return k; } return c + g(); } function sib(){ let d=9; function b(){ d+=1; } b(); return cap(); } console.log(sib()); } outer();\n",
    );
}

/// CRITICAL, registration-time variant — `Kali.test` called from inside an
/// env-owning NON-owner context captures the WRONG `env_ptr`. `cb` captures
/// `outer`'s `c` (41) but is registered from inside sibling env-owner `sib`,
/// so the `env_ptr` stored at registration was `sib`'s record and the drained
/// callback read `sib`'s `d` cell: pre-fix HEAD printed `c=8` + `ok 1`
/// (node: `c=41`). The registration site inherits the same safety
/// requirement as a direct call — fails closed E5506.
#[test]
fn dynamic_env_test_registration_from_sibling_env_owner_fails_closed() {
    assert_e5506_test(
        "function outer(){ let c = 41; function cb(){ console.log(\"c=\" + c); } function sib(){ let d = 7; function bump(){ d += 1; } bump(); Kali.test(\"t\", cb); } sib(); } outer();\n",
    );
}

/// IMPORTANT-1 — ALIASED capturing callback (`let cb2 = cb`). The alias is not
/// declarator-bound to a function expression, so its provenance is
/// unresolvable; the pre-fix guard default-ALLOWED it and the capturing
/// callback was silently dropped (pre-fix HEAD: success, printed `0`; base
/// rejected E5506). Default-deny now fails it closed E5506.
#[test]
fn deferred_set_timeout_aliased_capturing_callback_fails_closed() {
    assert_e5506(
        "function outer(){ let base = 0; let cb = function(){ base += 1; }; let cb2 = cb; setTimeout(cb2, 0); console.log(base); } outer();\n",
    );
}

/// IMPORTANT-1 — CALL-RESULT callback (`setTimeout(makeCb(), 0)`). A call
/// result has no resolvable callback provenance; the pre-fix guard
/// default-ALLOWED it and silently dropped the (capturing) callback (pre-fix
/// HEAD: success, printed `0`; base rejected E5506). Default-deny → E5506.
#[test]
fn deferred_set_timeout_call_result_callback_fails_closed() {
    assert_e5506(
        "function outer(){ let base = 0; function makeCb(){ return function(){ base += 1; }; } setTimeout(makeCb(), 0); console.log(base); } outer();\n",
    );
}

/// IMPORTANT-2 — the `Kali.test` fallback was a FIFTH unguarded surface: an
/// unresolvable callback (here a parameter) produced only a WARNING and
/// registered nothing, so the harness printed `ok 1` with zero tests and
/// exited 0 (pre-fix HEAD: success, `ok 1`, the callback body never ran).
/// Folded into the same default-deny: unresolvable → E5506.
#[test]
fn kali_test_unresolvable_callback_fails_closed() {
    assert_e5506_test(
        "function suite(cb){ Kali.test(\"a\", cb); }\nsuite(function(){ console.log(\"ran\"); });\n",
    );
}

/// Stage D: a NON-capturing function-expression microtask callback must be
/// deferred and actually RUN during the post-_start drain.
/// node v26.5.0: "sync\nmt\n".
#[test]
fn deferred_queue_microtask_fn_expr_runs_after_sync_code() {
    let out = run_kali(
        r#"queueMicrotask(function () {
  console.log("mt");
});
console.log("sync");
"#,
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "sync\nmt\n");
}

/// Stage D: a CAPTURING microtask callback runs with its owner's env record
/// (the C3 env_ptr restore), reading and writing the captured cell correctly
/// AFTER the owner has returned (never-reset-region property).
/// node v26.5.0: "sync=5\nmt=6\n".
#[test]
fn deferred_queue_microtask_capturing_callback_runs_with_env() {
    let out = run_kali(
        r#"function owner() {
  let base = 5;
  queueMicrotask(function () {
    base += 1;
    console.log("mt=" + base);
  });
  console.log("sync=" + base);
}
owner();
"#,
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "sync=5\nmt=6\n");
}

// ============================================================================
// Stage D task D2 — timer lanes (setTimeout / setInterval / clearTimeout /
// clearInterval). Every expected stdout below is re-verified against
// `node v26.5.0` on the fixture's ACTUAL source (see task-5-report.md).
// ============================================================================

/// Bonus closure discovered while diagnosing the `clearTimeout` fixture below:
/// a `const` binding to a scheduling-registration call with NO wasm local
/// slot (`self.locals`) falls onto the generic codegen `const` fold-alias
/// (`FunctionEmitter::bindings`), which RE-EMITS the recorded init node at
/// every later read of the bound name instead of reading back a stored
/// value. This was harmless before each surface's registration emit landed
/// (the call lowered through a dropped zero-placeholder, so duplicating it
/// duplicated nothing real), but `queueMicrotask` (Stage D task D2's earlier
/// lane) is a REAL side-effecting host call — pre-fix HEAD ran the callback
/// TWICE for `const m = queueMicrotask(fn); console.log(m);` (verified: kali
/// printed "m=0\nmt=1\nmt=2\n"). Closed at the exact same choke point as the
/// `setTimeout`/`setInterval` fix below
/// (`collect_function_locals_from_node`'s `is_scheduling_registration_call`,
/// `lower.rs`), which now also promotes a `queueMicrotask`-initialized
/// `const` to a real local so it is evaluated exactly once.
/// node v26.5.0: "m=undefined\nmt=1\n" (kali does not model `undefined` — see
/// the report's follow-ups — so this pins kali's own `m=0`, not node's
/// `m=undefined`; the load-bearing assertion is the callback running exactly
/// ONCE, not the `m=` line's exact text).
#[test]
fn deferred_queue_microtask_bound_return_value_runs_callback_once() {
    let out = run_kali(
        r#"function main() {
  let n = 0;
  const m = queueMicrotask(function () {
    n += 1;
    console.log("mt=" + n);
  });
  console.log("m=" + m);
}
main();
"#,
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "m=0\nmt=1\n");
}

/// Stage D: microtasks drain before timers; timers fire in delay order with
/// registration-order tiebreak — full ordering matrix in one fixture.
/// node v26.5.0: "sync\nm\na\nb\n".
#[test]
fn deferred_ordering_microtasks_then_timers_in_delay_order() {
    let out = run_kali(
        r#"setTimeout(function () { console.log("b"); }, 10);
setTimeout(function () { console.log("a"); }, 5);
queueMicrotask(function () { console.log("m"); });
console.log("sync");
"#,
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "sync\nm\na\nb\n");
}

/// Stage D: a capturing setTimeout callback runs with its owner's env record
/// after the owner returned (the never-reset-region property via timers).
/// node v26.5.0: "sync=5\nst=6\n".
#[test]
fn deferred_set_timeout_capturing_callback_now_runs() {
    let out = run_kali(
        r#"function owner() {
  let base = 5;
  setTimeout(function () {
    base += 1;
    console.log("st=" + base);
  }, 0);
  console.log("sync=" + base);
}
owner();
"#,
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "sync=5\nst=6\n");
}

/// Stage D: setInterval ticks repeatedly and clearInterval (with the captured
/// timer id) stops it. Function-scope variant: `n` and `t` are env cells.
/// node v26.5.0: "sync\ntick=1\ntick=2\ntick=3\n".
#[test]
fn deferred_set_interval_ticks_until_cleared() {
    let out = run_kali(
        r#"function main() {
  let n = 0;
  const t = setInterval(function () {
    n += 1;
    console.log("tick=" + n);
    if (n >= 3) {
      clearInterval(t);
    }
  }, 0);
  console.log("sync");
}
main();
"#,
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "sync\ntick=1\ntick=2\ntick=3\n"
    );
}

/// Stage D: clearTimeout cancels a pending timer — the callback never runs.
/// node v26.5.0: "sync\n".
#[test]
fn deferred_clear_timeout_cancels_pending_callback() {
    let out = run_kali(
        r#"function main() {
  const t = setTimeout(function () {
    console.log("never");
  }, 0);
  clearTimeout(t);
  console.log("sync");
}
main();
"#,
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "sync\n");
}

/// Stage D bounded drain, end to end: an uncleared interval must trap loudly
/// (exit != 0, "did not quiesce"), never hang. (node would hang here — the
/// one deliberate divergence, spec decision 3.)
#[test]
fn deferred_uncleared_interval_fails_loudly_not_hangs() {
    let out = run_kali(
        r#"setInterval(function () {}, 0);
console.log("sync");
"#,
    );
    assert!(
        !out.status.success(),
        "expected the non-quiescence trap, got exit 0"
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("did not quiesce"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Stage D envelope: a non-literal delay fails closed (precision follow-up).
#[test]
fn deferred_set_timeout_non_literal_delay_fails_closed() {
    let out = run_kali(
        r#"let d = 5;
setTimeout(function () { console.log("x"); }, d);
"#,
    );
    assert!(!out.status.success(), "expected E5506, got exit 0");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("E5506"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Stage D envelope: extra forwarded args fail closed (node passes them to
/// the callback; kali has no arg-forwarding lane — reject, don't drop).
#[test]
fn deferred_set_timeout_extra_args_fail_closed() {
    let out = run_kali(
        r#"setTimeout(function () { console.log("x"); }, 0, 42);
"#,
    );
    assert!(!out.status.success(), "expected E5506, got exit 0");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("E5506"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
