//! Soundness pins for R-11: bitwise compound assignment (`&= |= ^= <<= >>= >>>=`).
//!
//! All six were silent no-ops on every assignment target (48/48 in the
//! 2026-07-24 register re-derivation): `let n=6; n<<=2` returned the unmodified
//! `6` at exit 0. The fix reuses the plain-operator int32 lowering
//! (`emit_bitwise`) at every assignment target arm, lowering integer targets and
//! failing closed (E5506) on float/string/unadmitted targets.
//!
//! Every expected value here was captured from node v26.5.0.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-bitwise-compound-{}-{}-{}",
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

fn assert_stdout(src: &str, expected: &str) {
    let out = run_source(src);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout), expected, "{out:?}");
}

fn assert_fails_closed(src: &str, needle: &str) {
    let out = run_source(src);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !out.status.success(),
        "expected a fail-closed diagnostic, got success: {out:?}"
    );
    assert!(stderr.contains("E5506"), "expected E5506, got: {stderr}");
    assert!(
        stderr.contains(needle),
        "expected {needle:?}, got: {stderr}"
    );
}

// --- Task 1: plain binary bitwise operators stay correct (refactor is neutral) ---

#[test]
fn plain_binary_bitwise_operators_unchanged() {
    assert_stdout("console.log(6 & 3);\n", "2\n");
    assert_stdout("console.log(6 | 8);\n", "14\n");
    assert_stdout("console.log(6 ^ 1);\n", "7\n");
    assert_stdout("console.log(6 << 2);\n", "24\n");
    assert_stdout("console.log(6 >> 1);\n", "3\n");
    assert_stdout("console.log(6 >>> 1);\n", "3\n");
    assert_stdout("console.log(-1 >>> 0);\n", "4294967295\n");
    assert_stdout("console.log(1 << 31);\n", "-2147483648\n");
    assert_stdout("console.log(1 << 32);\n", "1\n");
}

// --- Task 1.5: the front end no longer silently mis-parses the six ops ---

#[test]
fn bitwise_compound_ops_are_not_silently_misparsed() {
    // Before Task 1.5 these lexed as two unrelated tokens and the statement
    // decayed into no-ops at exit 0 with ZERO diagnostics — the true R-11 root
    // cause. After Task 1.5 the op reaches codegen; Task 2 makes it compute the
    // right value. Here we pin only that the silent-garbage parse is gone: the
    // program must NOT exit 0 while printing the unmodified operand.
    for src in [
        "let n = 6; n &= 3; console.log(n);\n",
        "let n = 6; n |= 8; console.log(n);\n",
        "let n = 6; n ^= 1; console.log(n);\n",
        "let n = 6; n <<= 2; console.log(n);\n",
        "let n = 6; n >>= 1; console.log(n);\n",
        "let n = 6; n >>>= 1; console.log(n);\n",
    ] {
        let out = run_source(src);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            !(out.status.success() && stdout.trim() == "6"),
            "silent no-op survived for {src:?}: {out:?}"
        );
    }
}

// --- Task 1.5 review follow-up: pin the resolve-stage gate's BOUNDARY ---
//
// `bitwise_compound_ops_are_not_silently_misparsed` above only asserts
// `!(exit 0 && stdout == "6")`, which passes for a great many wrong
// behaviors (a crash, an unrelated diagnostic, a wrong-but-nonzero value,
// ...). It does not pin that the six ops specifically produce the
// `kali_types::resolve::expression` fail-closed gate
// (`bitwise_compound_assign_op_text`, `crates/kali_types/src/resolve/expression.rs`),
// nor does it exercise any target shape other than a plain mutable scalar
// local. The two tests below close that gap: they pin the EXACT E5506
// diagnostic text the gate emits, across all six operators on the
// local-scalar case and across one representative operator on every other
// target shape codegen has not validated.
//
// These are BOUNDARY pins, not permanent truth: as Task 2 admits a shape
// (starting with the local-scalar arm, per its own task description), the
// corresponding row here is expected to flip from `assert_fails_closed` to
// an `assert_stdout` value assertion — that is progress, not a regression.
// What must NOT happen is a row silently going from "denied by this specific
// gate" to "denied by nothing" (i.e. admitted with no codegen support) or
// "denied by some unrelated diagnostic" without a deliberate, reviewed
// change to this file. If Task 2 deletes the whole `if let Some(op_text) =
// bitwise_compound_assign_op_text(...)` block in one shot instead of
// narrowing it shape-by-shape, every row below whose shape codegen still
// does not support will fail here (not silently pass), because the message
// text pinned is specific to this gate and no other diagnostic in this
// codebase reuses it.

#[test]
fn bitwise_compound_fails_closed_on_plain_scalar_all_six_ops() {
    for (src, op_text) in [
        ("let n = 6; n &= 3; console.log(n);\n", "&="),
        ("let n = 6; n |= 8; console.log(n);\n", "|="),
        ("let n = 6; n ^= 1; console.log(n);\n", "^="),
        ("let n = 6; n <<= 2; console.log(n);\n", "<<="),
        ("let n = 6; n >>= 1; console.log(n);\n", ">>="),
        ("let n = 6; n >>>= 1; console.log(n);\n", ">>>="),
    ] {
        assert_fails_closed(src, op_text);
    }
}

#[test]
fn bitwise_compound_fails_closed_on_every_target_shape() {
    // One representative op (`&=`) is enough per shape: the gate this pins
    // decides purely on `AssignmentOperator`, before it ever looks at the
    // LHS shape (`crates/kali_types/src/resolve/expression.rs:1794-1820` runs
    // ahead of every shape-specific admit path), so all six operators take
    // the same route through every shape below. `bitwise_compound_fails_closed_on_plain_scalar_all_six_ops`
    // above already covers the cross-operator axis on the one shape Task 2
    // is expected to admit first; this test covers the cross-shape axis.
    let needle = "&=";
    assert_fails_closed(
        // Member target (`o.a`).
        "let o = { a: 6 }; o.a &= 3; console.log(o.a);\n",
        needle,
    );
    assert_fails_closed(
        // Array element (`a[0]`).
        "let a = [6, 1, 2]; a[0] &= 3; console.log(a[0]);\n",
        needle,
    );
    assert_fails_closed(
        // For-in-key computed target (`o[k]`).
        "let o = { a: 6, b: 6 }; for (const k in o) { o[k] &= 3; } console.log(o.a);\n",
        needle,
    );
    assert_fails_closed(
        // Closure-captured variable.
        "function outer(){ let x = 6; function g(){ x &= 3; } g(); console.log(x); } outer();\n",
        needle,
    );
    assert_fails_closed(
        // Module global written from a function.
        "let g = 6; function f(){ g &= 3; } f(); console.log(g);\n",
        needle,
    );
    assert_fails_closed(
        // Float target.
        "let f = 6.5; f &= 3; console.log(f);\n",
        needle,
    );
    assert_fails_closed(
        // String target.
        "let s = \"6\"; s &= 3; console.log(s);\n",
        needle,
    );
    assert_fails_closed(
        // `const` target.
        "const c = 6; c &= 3; console.log(c);\n",
        needle,
    );
}
