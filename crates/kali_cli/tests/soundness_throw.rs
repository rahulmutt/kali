use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-throw-{}-{}-{}",
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

// node: `throw new Error("boom")` prints "Uncaught Error: boom" on stderr and
// exits nonzero; statements after the throw must NOT run. Today kali silently
// falls through and prints "after" with exit 0 — the headline no-op miscompile.
#[test]
fn throw_error_literal_aborts_with_message() {
    let out = run_source("throw new Error(\"boom\");\nconsole.log(\"after\");\n");
    assert!(!out.status.success(), "throw must abort: {out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Uncaught Error: boom"), "stderr: {stderr}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.contains("after"), "must not execute past throw: {stdout}");
}

#[test]
fn throw_string_literal_aborts_with_message() {
    let out = run_source("throw \"plain\";\nconsole.log(\"after\");\n");
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Uncaught plain"), "stderr: {stderr}");
    assert!(!String::from_utf8_lossy(&out.stdout).contains("after"));
}

// Non-literal argument: no message proof — generic message, still aborts.
#[test]
fn throw_computed_value_still_aborts() {
    let out = run_source("let x = 3;\nthrow x + 1;\nconsole.log(\"after\");\n");
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("Uncaught exception"));
    assert!(!String::from_utf8_lossy(&out.stdout).contains("after"));
}

// A throw on a never-taken path must not affect the taken path — the whole
// suite's defensive self-check throws depend on this staying compilable.
#[test]
fn unreached_throw_is_harmless() {
    let out = run_source(
        "function check(n) { if (n > 10) { throw new Error(\"nope\"); } return n; }\nconsole.log(check(5));\n",
    );
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "5");
}
