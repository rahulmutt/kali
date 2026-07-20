use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-objtaint-{}-{}-{}",
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

// Declarator form — already rejected (Spec 7 Task 2); pin it.
#[test]
fn declarator_object_compound_rejects() {
    let out = run_source("var o = { x: 1 };\no += 1;\nconsole.log(o);\n");
    assert!(!out.status.success(), "{out:?}");
}

// No-initializer form — the gap: the taint was seeded only from
// declarator RHS shapes, so `var o; o = {x:1}; o += 1` slipped through.
#[test]
fn late_assigned_object_compound_rejects() {
    let out = run_source("var o;\no = { x: 1 };\no += 1;\nconsole.log(o);\n");
    assert!(!out.status.success(), "{out:?}");
}

// Reassignment-after-scalar form: same seed, via a later assignment.
#[test]
fn reassigned_to_object_compound_rejects() {
    let out = run_source("var o = 0;\no = { x: 1 };\no += 1;\nconsole.log(o);\n");
    assert!(!out.status.success(), "{out:?}");
}
