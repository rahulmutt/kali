//! Out-of-lane `delete` is a fail-closed compile error (throw-fallout
//! Stage 2 Lane C). In-lane deletes (straight-line top-level
//! delete+reinsert over a const-bound literal whose only other uses are
//! folded enumerations) are consumed by the optimizer's timeline and never
//! reach codegen — everything else must reject, never silently no-op:
//! before Stage 2 `delete r.b` compiled as a bare member read (the parser
//! swallowed the token), so ANY silent path here re-opens a miscompile.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_expect_reject(src: &str) -> (String, i32) {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-delete-gate-{}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        src.len()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("main.ts");
    std::fs::write(&path, src).expect("write source");
    let out = Command::new(kali_bin())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali");
    (
        String::from_utf8_lossy(&out.stderr).to_string(),
        out.status.code().unwrap_or(-1),
    )
}

#[test]
fn delete_inside_a_branch_rejects_e5506() {
    // node: prints c=2. kali: must NOT run — conditional delete is
    // outside the static timeline.
    let (stderr, code) = run_expect_reject(
        "const r = { a: 1, b: 2 };\nif (r.a) { delete r.b; }\nconsole.log('c=' + Object.keys(r).length);",
    );
    assert_ne!(code, 0);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("delete"), "stderr: {stderr}");
}

#[test]
fn delete_then_member_read_rejects_e5506() {
    // node: prints undefined. kali: a runtime member read of a
    // deleted-not-reinserted key is untested surface — fail closed
    // (the read disqualifies the binding from the timeline, so the
    // delete reaches codegen).
    let (stderr, code) =
        run_expect_reject("const r = { a: 1, b: 2 };\ndelete r.b;\nconsole.log(r.b);");
    assert_ne!(code, 0);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn delete_of_aliased_object_rejects_e5506() {
    // Aliasing + mutation: both names must see the mutation (node
    // semantics) — outside the timeline, fail closed.
    let (stderr, code) = run_expect_reject(
        "const r = { a: 1, b: 2 };\nconst s = r;\ndelete r.b;\nconsole.log(Object.keys(s).length);",
    );
    assert_ne!(code, 0);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}
