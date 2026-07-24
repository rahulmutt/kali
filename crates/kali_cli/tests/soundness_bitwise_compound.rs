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
