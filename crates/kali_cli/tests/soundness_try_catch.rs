//! Soundness pins for try/catch/finally: rejected fail-closed.
//!
//! kali has no exception-unwinding machinery. A `TryStatement` used to lower
//! to a bare-text Branch LIR node that fell into the generic emit_branch arm,
//! which treated the try body as an `if` condition and the catch block as the
//! `then` arm — a silent miscompile that only looked correct while `throw`
//! was itself a no-op. With throw fixed (print-then-trap), the only sound
//! option is a compile-time feature-unavailable reject.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-try-catch-{}-{}-{}",
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

fn assert_rejected(src: &str) {
    let out = run_source(src);
    assert!(!out.status.success(), "must not compile: {out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("try/catch/finally is unavailable"),
        "stderr: {stderr}"
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).is_empty(),
        "nothing may execute: {out:?}"
    );
}

#[test]
fn try_catch_is_rejected() {
    assert_rejected(
        "let caught = false;\ntry {\n  throw 'boom';\n} catch {\n  caught = true;\n}\nconsole.log(caught);\n",
    );
}

#[test]
fn try_catch_with_param_is_rejected() {
    assert_rejected("try {\n  console.log('body');\n} catch (e) {\n  console.log('caught');\n}\n");
}

#[test]
fn try_finally_is_rejected() {
    assert_rejected("try {\n  0;\n} finally {\n  console.log(2);\n}\nconsole.log(1);\n");
}
