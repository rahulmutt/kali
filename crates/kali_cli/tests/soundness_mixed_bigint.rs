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
