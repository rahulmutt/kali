//! Stage D event-surface lane (Task 3) — `new EventTarget()` construction lane
//! + handle-escape discipline.
//!
//! The construction lane emits an opaque i64 host handle for a declarator-bound
//! `new EventTarget()`, records the binding's provenance, and fails closed on
//! every read that would leak the raw handle (spec §2.4). The
//! addEventListener/dispatchEvent emit arms land in Task 4; until then a
//! `t.addEventListener(...)` still takes the Stage C backstop.

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

/// Stage D event lane: `new EventTarget()` in a declarator compiles and the
/// program runs (the handle is opaque; nothing observable yet).
/// node v26.5.0: "done\n".
#[test]
fn event_target_construction_in_declarator_compiles_and_runs() {
    let out = run_kali(
        r#"const t = new EventTarget();
console.log("done");
"#,
    );
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "done\n");
}

/// Handle-escape discipline (spec §2.4): an EventTarget binding read outside
/// the lane's allowed positions fails closed — the handle must never leak as
/// a number (node prints `EventTarget {}`; kali would print the raw handle).
#[test]
fn event_target_handle_escape_fails_closed() {
    let out = run_kali(
        r#"const t = new EventTarget();
console.log(t);
"#,
    );
    assert!(!out.status.success(), "expected E5506, got exit 0");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("E5506"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Reassigned target bindings lose stable provenance — every later lane use
/// fails closed (the unstable_provenance_names rule).
#[test]
fn event_target_reassigned_binding_fails_closed() {
    let out = run_kali(
        r#"let t = new EventTarget();
t = new EventTarget();
t.addEventListener("tick", function () {});
"#,
    );
    assert!(!out.status.success(), "expected E5506, got exit 0");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("E5506"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// `new EventTarget()` OUTSIDE a declarator has no recordable provenance —
/// fail closed, don't emit an untracked handle.
#[test]
fn event_target_non_declarator_construction_fails_closed() {
    let out = run_kali(
        r#"new EventTarget();
console.log("done");
"#,
    );
    assert!(!out.status.success(), "expected E5506, got exit 0");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("E5506"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
