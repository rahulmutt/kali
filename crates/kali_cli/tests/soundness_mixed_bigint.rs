use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-mixbig-{}-{}-{}",
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

// node: `3n / 2` throws TypeError (cannot mix BigInt and Number). kali
// silently floated it (F64Div → prints 1.5). Reject at compile time.
#[test]
fn mixed_bigint_arithmetic_rejects() {
    for src in [
        "console.log(3n / 2);\n",
        "console.log(3n * 2);\n",
        "console.log(3n + 2);\n",
        "console.log(2 - 3n);\n",
    ] {
        let out = run_source(src);
        assert!(!out.status.success(), "{src:?} must reject, got: {out:?}");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("E3202"),
            "expected E3202 for {src:?}: {out:?}"
        );
    }
}

// All-BigInt stays green (existing lane).
#[test]
fn all_bigint_arithmetic_still_works() {
    let out = run_source("console.log(6n / 2n);\nconsole.log(2n * 3n);\n");
    assert!(out.status.success(), "{out:?}");
}

// Promoted operator sweep: every gated operator (`+ - * / % **`) rejects when
// exactly one side is a proven BigInt literal and the other a proven numeric
// literal — including under unary minus and parens (node throws TypeError for
// all of these).
#[test]
fn mixed_bigint_operator_sweep_rejects() {
    for src in [
        "console.log(-3n * 2);\n",  // unary-minus BigInt side
        "console.log(3n % 2);\n",   // %
        "console.log(2 ** 3n);\n",  // ** with numeric base, BigInt exponent
        "console.log((3n) + 2);\n", // parenthesized BigInt side
        "console.log(3n - (2));\n", // parenthesized numeric side
    ] {
        let out = run_source(src);
        assert!(!out.status.success(), "{src:?} must reject, got: {out:?}");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("E3202"),
            "expected E3202 for {src:?}: {out:?}"
        );
    }
}

// Anti-over-reject: the gate requires TWO-SIDED literal proof. Correct
// all-BigInt code whose operands are NOT bare literals (a `const` binding, a
// function parameter) must NOT trip E3202 — "recognizer can't prove BigInt"
// is not "proven non-BigInt". `const x = 7n; x / 2n` is exactly the
// regression an earlier one-sided `left != right` gate caused (it broke the
// bigint-*-chain benchmark fixtures); pin it closed. node prints `3n`; kali's
// runtime BigInt console format prints `3` (pre-existing, see Task 2).
#[test]
fn unproven_bigint_operand_does_not_over_reject() {
    let out = run_source("const x = 7n;\nconsole.log(x / 2n);\n");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "3");
}

// Function-parameter form, mirroring the bigint-addition/multiplication-chain
// benchmark fixtures a one-sided gate broke: `hot(value)` does BigInt
// arithmetic where `value` is a parameter (unproven), invoked only with
// BigInt-literal arguments. Must compile and run, not reject.
#[test]
fn parameter_bigint_operand_does_not_over_reject() {
    let out = run_source("function hot(v) { return (v + 0n) + 1n; }\nconsole.log(hot(2n));\n");
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "3");
}
