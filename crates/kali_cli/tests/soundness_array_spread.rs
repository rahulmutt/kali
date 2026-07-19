//! Soundness pins for R-25: array spread `[...a]` was mis-classified as a
//! 1-element array literal (`is_array_literal` accepted a textless Value whose
//! child is a `spread` node), so `.length` folded to 1 and `[0]` to 0 at exit
//! 0. There is no spread-expansion lowering; the honest target is
//! REJECT-DON'T-MISCOMPILE (E5506), mirroring object spread `{...o}` which
//! already fails closed. Static array literals without a spread child keep
//! folding. Every expected value captured from node v26.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-unimpl-builtins-spread-{}-{}-{}",
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
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !out.status.success(),
        "expected a fail-closed diagnostic, got success with stdout {stdout:?}: {out:?}"
    );
    assert!(
        stderr.contains("E5506"),
        "expected E5506 in stderr, got: {stderr}"
    );
    assert!(
        stderr.contains(needle),
        "expected {needle:?} in stderr, got: {stderr}"
    );
}

#[test]
fn array_spread_of_binding_fails_closed() {
    // node: len=2 e0=1. Pre-fix kali: len=1 e0=0 at exit 0.
    assert_fails_closed(
        r#"const a=[1,2]; const b=[...a]; console.log("len="+b.length);"#,
        "spread",
    );
}

#[test]
fn array_spread_of_literal_fails_closed() {
    // node: 2. Pre-fix kali: 1 at exit 0.
    assert_fails_closed(r#"const b=[...[1,2]]; console.log(b.length);"#, "spread");
}

#[test]
fn plain_array_literal_still_folds() {
    // Preserve pin: no spread child, must keep working.
    assert_stdout(
        r#"const a=[1,2,3]; console.log("len="+a.length);"#,
        "len=3\n",
    );
}
