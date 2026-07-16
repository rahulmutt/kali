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
