//! Default-parameter truncation: the ONE test this family could not migrate
//! to a case file.
//!
//! Every other `#[test]` from the original 10 in this file is migrated to
//! `tests/cases/soundness/param_truncation.toml` (9 cases, audited clean).
//! This file keeps exactly `repro_default_param_does_not_silently_truncate_module`,
//! because its assertion is a genuine DISJUNCTION the case-runner format
//! cannot express: `if out.status.success() { assert stdout contains
//! "after-should-print" } else { assert combined stdout+stderr contains
//! "E5506" }`. Neither branch alone is what the source claims -- pinning
//! only the currently-observed branch (failure, E5506) would either
//! silently drop the source's success-branch alternative (weakening it) or
//! require asserting the OPPOSITE of that alternative on a literal that
//! lives only in the untaken branch's code (`stdout_absent =
//! ["after-should-print"]`, which a first attempt at this migration did --
//! confirmed to actually invert the source's own claim: the source
//! EXPECTS "after-should-print" to appear the day default parameters are
//! supported for real, and explicitly says so in its own comment; the
//! wrongly-added assertion would fail on that exact day, exactly the
//! "asserting something the source never claimed" degradation this format
//! exists to prevent, made worse here by directly contradicting the
//! source's stated forward-looking intent). Both an audit-exception
//! mechanism and a reachability analysis to justify that shape were
//! explicitly rejected. Kept hand-written per spec 5.11's "outliers"
//! bucket, trimmed to just this test and the helper it needs.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-paramtrunc-{}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
        src.len()
    ));
    std::fs::create_dir_all(&dir).expect("create temp dir");
    let path = dir.join("main.ts");
    std::fs::write(&path, src).expect("write source");
    Command::new(kali_bin())
        .arg("run")
        .arg(&path)
        .output()
        .expect("run kali")
}

/// The controller's exact repro. `after-should-print` is the positive control:
/// it appears ONLY after the default-param declaration, so an assertion on it
/// is precisely an assertion that the module was not truncated. Under the old
/// behavior this printed `before=1` and exited 0.
#[test]
fn repro_default_param_does_not_silently_truncate_module() {
    let src = "function f(a) { return a; }\n\
               console.log(\"before=\" + f(1));\n\
               function g(b = 5) { return b; }\n\
               console.log(\"after-should-print\");\n";
    let out = run_source(src);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    if out.status.success() {
        // If default params are ever supported for real, the ONLY acceptable
        // success is one that also ran the trailing statement.
        assert!(
            stdout.contains("after-should-print"),
            "SILENT TRUNCATION: exited 0 without running the statement after \
             the default-param declaration.\nstdout: {stdout}\nstderr: {stderr}"
        );
    } else {
        assert!(
            format!("{stdout}{stderr}").contains("E5506"),
            "expected E5506 fail-closed, got:\nstdout: {stdout}\nstderr: {stderr}"
        );
    }
}
