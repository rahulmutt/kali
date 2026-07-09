use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-mod-obj-{}-{}-{}",
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

// A compound assign on an object-initialized binding has no scalar lowering
// (node string-coerces the object; kali cannot) — must reject fail-closed,
// never miscompile `0 + 1 = 1`.
#[test]
fn object_initialized_binding_compound_rejects() {
    let out = run_source("var o = {x:1};\no += 1;\nconsole.log(o);\n");
    assert!(!out.status.success(), "expected E5506 reject, got: {out:?}");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("E5506")
            || String::from_utf8_lossy(&out.stderr).contains("not a provably scalar")
    );
}

// A genuine numeric var local still compiles and runs — the fix must not
// over-reject scalars.
#[test]
fn numeric_var_local_compound_still_runs() {
    let out = run_source("var k = 0;\nk += 1;\nk += 41;\nconsole.log(k);\n");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "42\n");
}
