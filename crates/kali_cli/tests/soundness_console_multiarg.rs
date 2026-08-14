//! Multi-argument `console.*` calls: the ONE test this family could not
//! migrate to a case file.
//!
//! Every other `#[test]` from the original 10 in this file is migrated to
//! `tests/cases/soundness/console_multiarg.toml` (17 cases, audited clean).
//! This file keeps exactly `multi_arg_never_exits_zero_with_a_dropped_argument`,
//! because its assertion is a genuine DISJUNCTION the case-runner format
//! cannot express: `if out.status.success() { assert full line present }
//! else { assert combined stdout+stderr contains "E5506" }`. Neither branch
//! alone is what the source claims -- pinning only the currently-observed
//! branch (success, full line) would either silently drop the source's
//! `E5506` alternative (weakening it) or require asserting the OPPOSITE of
//! that alternative on a literal that lives only in the untaken branch's
//! code (`stderr_absent = ["E5506"]`, which a first attempt at this
//! migration did -- confirmed to actually flip the source's own semantics:
//! if a future fix makes this construct fail closed with an E5506 warning
//! on stderr while ALSO exiting 0, the source test still passes but the
//! wrongly-added assertion would fail, exactly the "asserting something the
//! source never claimed" degradation this format exists to prevent). Both
//! an audit-exception mechanism and a reachability analysis to justify that
//! shape were explicitly rejected. Kept hand-written per spec 5.11's
//! "outliers" bucket, trimmed to just this test and the two helpers it
//! needs.

use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_source(src: &str) -> std::process::Output {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "kali-soundness-consolemultiarg-{}-{}-{}",
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

/// The no-silent-loss invariant stated directly: whatever the runtime can or
/// cannot render, a multi-argument call must either print every argument or
/// fail closed with a diagnostic. It must never exit 0 having printed a
/// strict prefix of its arguments.
#[test]
fn multi_arg_never_exits_zero_with_a_dropped_argument() {
    let src = "function f(x) { return x + 1; }\n\
               console.log(\"head\", f(1), \"tail-should-print\");\n";
    let out = run_source(&src);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    if out.status.success() {
        assert!(
            stdout.lines().any(|l| l == "head 2 tail-should-print"),
            "SILENT ARGUMENT LOSS: exited 0 without printing every argument.\n\
             stdout: {stdout}\nstderr: {stderr}"
        );
    } else {
        let combined = format!("{stdout}{stderr}");
        assert!(
            combined.contains("E5506"),
            "a rejected multi-argument console call must fail closed with an \
             E5506 naming the construct.\nstdout: {stdout}\nstderr: {stderr}"
        );
    }
}
