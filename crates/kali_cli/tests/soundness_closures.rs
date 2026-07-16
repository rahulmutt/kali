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
