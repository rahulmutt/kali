//! Deny-lane reproducer pins for the object-enum family (PR #16 merge
//! readiness). kali has no runtime materialization of enumeration-result
//! arrays (`Object.keys/values/entries/fromEntries`, `Reflect.ownKeys`,
//! frozen-object enumeration): iterating, spreading, or otherwise
//! materializing such a result at runtime silently miscompiles to garbage
//! i64 handles / `0` placeholders at exit 0. This suite pins the fail-closed
//! E5506 deny lane that closes that class. The pure-static consumers
//! (`.length`, static-index folds) never materialize a runtime array and stay
//! admitted — see the companion `keeps_*` pins.
//!
//! Flip-back: pr16-honest-repin-inventory.md#object-enum.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-objenum-{}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        src.len()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("main.ts");
    std::fs::write(&path, src).unwrap();
    Command::new(kali_bin())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali")
}

fn assert_e5506(src: &str) {
    let out = run_source(src);
    assert!(
        !out.status.success(),
        "expected fail-closed reject, got success: {out:?}"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("E5506"),
        "expected E5506 in stderr, got: {stderr}"
    );
}

/// Deny lane (PR #16 merge readiness, family object-enum): kali has no runtime
/// materialization of enumeration-result arrays; silent placeholder miscompile
/// closed to E5506. Flip-back: pr16-honest-repin-inventory.md#object-enum.
#[test]
fn for_of_object_keys_rejects_e5506() {
    // node: iterates 'b','a'. Pre-lane kali: for-of + push accumulates garbage
    // handles, `keys.length` reads 0. Now fail-closed E5506.
    assert_e5506(
        "const keys = [];\nfor (const k of Object.keys({ b: 1, a: 2 })) { keys.push(k); }\nif (keys.length !== 2) { throw new Error('x'); }\n",
    );
}

/// Deny lane (PR #16 merge readiness, family object-enum): kali has no runtime
/// materialization of enumeration-result arrays; silent placeholder miscompile
/// closed to E5506. Flip-back: pr16-honest-repin-inventory.md#object-enum.
#[test]
fn spread_object_values_rejects_e5506() {
    // node: [1,2]. Pre-lane kali: spread materializes `0` placeholders. Now E5506.
    assert_e5506("const v = [...Object.values({ b: 1, a: 2 })];\nconsole.log(\"s=\" + v[0]);\n");
}

/// Deny lane (PR #16 merge readiness, family object-enum): kali has no runtime
/// materialization of enumeration-result arrays; silent placeholder miscompile
/// closed to E5506. Flip-back: pr16-honest-repin-inventory.md#object-enum.
#[test]
fn for_of_object_entries_rejects_e5506() {
    // node: iterates ['b',1],['a',2]. Pre-lane kali: garbage. Now E5506.
    assert_e5506(
        "const seen = [];\nfor (const e of Object.entries({ b: 1, a: 2 })) { seen.push(e[0]); }\nif (seen.length !== 2) { throw new Error('x'); }\n",
    );
}

/// Deny lane (PR #16 merge readiness, family object-enum): kali has no runtime
/// materialization of enumeration-result arrays; silent placeholder miscompile
/// closed to E5506. Flip-back: pr16-honest-repin-inventory.md#object-enum.
#[test]
fn for_of_frozen_object_enumeration_rejects_e5506() {
    // node: iterates 'a','b'. Pre-lane kali: garbage. Now E5506.
    assert_e5506(
        "const keys = [];\nfor (const k of Object.keys(Object.freeze({ a: 1, b: 2 }))) { keys.push(k); }\nif (keys.length !== 2) { throw new Error('x'); }\n",
    );
}

/// Deny lane (PR #16 merge readiness, family object-enum): `Object.fromEntries`
/// enumeration results materialized via for-of are fail-closed E5506.
/// Flip-back: pr16-honest-repin-inventory.md#object-enum.
#[test]
fn for_of_from_entries_values_rejects_e5506() {
    assert_e5506(
        "const o = Object.fromEntries([[\"b\", 1], [\"a\", 2]]);\nconst seen = [];\nfor (const v of Object.values(o)) { seen.push(v); }\nif (seen.length !== 2) { throw new Error('x'); }\n",
    );
}

// --- Admitted static lanes (must stay green: the deny lane must not over-reject) ---

/// Admitted: `.length` on an enumeration result is a compile-time-known count;
/// it never materializes a runtime array, so it stays supported.
#[test]
fn keeps_object_keys_length() {
    let out = run_source("console.log(Object.keys({ b: 1, a: 2 }).length);\n");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "2");
}

/// Admitted: a fully static nested index read folds to the real key at compile
/// time (no runtime materialization).
#[test]
fn keeps_object_entries_static_index() {
    let out = run_source("console.log(Object.entries({ a: 1, b: 2, c: 3 })[0][0]);\n");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "a");
}

/// Admitted: a fully static `Object.values(...)[i]` fold stays correct.
#[test]
fn keeps_object_values_static_index() {
    let out = run_source(
        "const cfg = { PATH: \"1\", OTHER: \"2\" };\ndelete cfg.PATH;\nconsole.log(Object.values(cfg)[0]);\n",
    );
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "2");
}
