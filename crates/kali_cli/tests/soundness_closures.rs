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
