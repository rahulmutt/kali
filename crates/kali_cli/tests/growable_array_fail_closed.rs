//! Stage 4 Task 6: out-of-scope growable-array shapes fail CLOSED (E5506),
//! never a silent wrong answer. A growable-shape binding (`const o = []`) that
//! is a `.push` receiver but cannot be promoted — because it escapes, is
//! aliased, is pushed through a computed/optional-chain call, is captured by a
//! closure, is mutated by a non-`push` method, is pushed with the wrong arity,
//! or would carry an unsupported (float/object) element — must be rejected at
//! compile time, not left on the pre-existing push-no-op lane whose length/
//! index reads silently diverge from node.
//!
//! Modeled on `growable_array_core.rs` (kali_bin() helper, tempdir, `run`),
//! asserting `!status.success()` + stderr contains `error[E5506]` for each.

use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

/// Runs `src` (as both `.js` and `.ts`) and asserts a fail-closed E5506:
/// non-zero exit, no stdout (never a silent partial run), stderr `error[E5506]`.
fn assert_fail_closed_e5506(src: &str) {
    for extension in ["js", "ts"] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(format!("smoke.test.{extension}"));
        fs::write(&source_path, src).expect("write source");

        let output = Command::new(kali_bin())
            .current_dir(dir.path())
            .arg("run")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(
            !output.status.success(),
            "expected E5506 fail-closed, not a silent run ({extension}): {output:?}"
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.is_empty(),
            "expected NO stdout (never a silent wrong-output run) ({extension}): {stdout}"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("error[E5506]"),
            "expected error[E5506] ({extension}): {stderr}"
        );
    }
}

/// Brief shape 1 — map materialization: `[1,2,3].map(...)` bound to a const is
/// unavailable (design repro D). Already E5506; pinned here so no future change
/// silently promotes it to a wrong-length answer.
#[test]
fn map_materialization_bind_fails_closed() {
    assert_fail_closed_e5506(
        "function m(){const out=[1,2,3].map(v=>v*2);console.log(out.length);}m();",
    );
}

/// Brief shape 2 — escaping growable: a pushed array returned from its function
/// escapes the safe-position allowlist. Node prints 3; the push-no-op lane
/// printed 1 — a silent miscompile that must now reject.
#[test]
fn escaping_via_return_fails_closed() {
    assert_fail_closed_e5506(
        "function make(){const o=[];o.push(1);o.push(2);o.push(3);return o;}\
         function m(){const a=make();console.log(a.length);}m();",
    );
}

/// Brief shape 3 — non-push mutator: `.pop()` has no growable recognizer, so a
/// growable-shape push receiver that also `.pop()`s must reject (the pushes
/// would otherwise silently no-op).
#[test]
fn pop_mutator_fails_closed() {
    assert_fail_closed_e5506(
        "function m(){const o=[];o.push(1);o.push(2);o.pop();console.log(o.length);}m();",
    );
}

/// Silent-poison class (Task 4 re-review shapes): aliasing the growable binding.
#[test]
fn alias_binding_fails_closed() {
    assert_fail_closed_e5506(
        "function m(){const o=[];o.push(1);o.push(2);o.push(3);const p=o;console.log(o.length);}m();",
    );
}

/// Silent-poison class: a computed `o["push"](v)` call — no clean `.push`
/// occurrence exists, so the binding never promoted and silently printed 0.
#[test]
fn computed_push_call_fails_closed() {
    assert_fail_closed_e5506(
        "function m(){const o=[];o[\"push\"](1);o[\"push\"](2);console.log(o.length);}m();",
    );
}

/// Silent-poison class: an optional-chain `o?.push(v)` call.
#[test]
fn optional_chain_push_call_fails_closed() {
    assert_fail_closed_e5506(
        "function m(){const o=[];o?.push(1);o?.push(2);console.log(o.length);}m();",
    );
}

/// Silent-poison class: capturing the growable binding in a closure.
#[test]
fn closure_capture_fails_closed() {
    assert_fail_closed_e5506(
        "function m(){const o=[];o.push(1);o.push(2);const f=()=>o.push(3);console.log(o.length);}m();",
    );
}

/// Silent-poison class: `push` with the wrong arity (two args). Node pushes
/// both; the no-op lane printed 0.
#[test]
fn wrong_arity_push_fails_closed() {
    assert_fail_closed_e5506("function m(){const o=[];o.push(1,2);console.log(o.length);}m();");
}

/// Repr-gate class: float elements are unsupported (constraints doc: F64 fails
/// closed). The syntactic candidate solves a float element axis and must reject
/// rather than silently no-op (printed 0, node 2).
#[test]
fn float_element_push_fails_closed() {
    assert_fail_closed_e5506(
        "function m(){const o=[];o.push(1.5);o.push(2.5);console.log(o.length);}m();",
    );
}
