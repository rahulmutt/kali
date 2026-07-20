//! Soundness pins for R-19/R-20/R-15 (Stream A): String()/Boolean()/
//! x.toString()/JSON.stringify()/runtime-split slipped past emit_call's E5506
//! deny and landed on the terminal warning+`i64.const 0` placeholder,
//! returning 0 at exit 0. The fix (Task A2b) fails those closed via a small
//! POSITIVE deny-set (`deny_placeholder_lowering`) at that terminal, while the
//! warn+0 escape hatch stays the DEFAULT for everything else (unresolved
//! cross-package imports + unimplemented host surfaces), so the deny is narrow
//! (Number() is not in it — it fails at E3100 resolve, never reaching the
//! terminal). The static-fold lanes upstream of the fallback (split
//! static-ASCII, static toString) are preserved. Every expected value from node v26.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-unimpl-builtins-call-{}-{}-{}",
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
fn string_call_fails_closed() {
    // node: r=42. Pre-fix kali: r=0 at exit 0.
    assert_fails_closed(r#"const x=42; console.log("r="+String(x));"#, "unavailable");
}

#[test]
fn to_string_method_fails_closed() {
    // node: r=42. Pre-fix kali: r=0 at exit 0.
    assert_fails_closed(
        r#"const n=42; console.log("r="+n.toString());"#,
        "unavailable",
    );
}

#[test]
fn json_stringify_fails_closed() {
    // node: r={"f":1}. Pre-fix kali: r=0 at exit 0.
    assert_fails_closed(
        r#"const o={f:1}; console.log("r="+JSON.stringify(o));"#,
        "unavailable",
    );
}

#[test]
fn boolean_call_fails_closed() {
    // node: r=true. Pre-fix kali: r=0 at exit 0 (silent-0 coercion, same class
    // as String()). Number() is NOT pinned: it fails at E3100 resolve as an
    // undefined identifier and never reaches the terminal call fallback.
    assert_fails_closed(r#"const x=1; console.log("r="+Boolean(x));"#, "unavailable");
}

#[test]
fn split_static_ascii_lane_is_preserved() {
    // Preserve pin: static-receiver split indexing still folds. node: a.
    assert_stdout(r#"console.log("abc".split("")[0]);"#, "a\n");
}
