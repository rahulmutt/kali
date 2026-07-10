use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-bigintdiv-{}-{}-{}",
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

// node: `1n/0n` throws "RangeError: Division by zero". kali already traps
// (correct abort) but with the generic unreachable envelope; pin the
// node-shaped message. Also pin that a nonzero literal divide still works.
#[test]
fn bigint_division_by_zero_traps_with_range_error() {
    let out = run_source("console.log(7n / 0n);\n");
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("RangeError: Division by zero"));
}

#[test]
fn bigint_division_nonzero_still_truncates() {
    let out = run_source("console.log(7n / 2n);\n");
    assert!(out.status.success(), "{out:?}");
    // kali's existing (pre-existing, out of scope for this task) bigint
    // console.log format prints the runtime i64 result without the `n`
    // suffix for a non-literal-folded binary op result — see e.g. `7n + 2n`
    // -> "9" — even though node prints "3n". Pin what kali ALREADY prints;
    // do not change output formatting here.
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "3");
}
