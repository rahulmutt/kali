//! End-to-end: an unsupported parameter shape must never silently truncate the
//! module.
//!
//! The defect: `function g(b = 5) { ... }` left the token stream parked on `=`,
//! `parse_block_statement` advanced over it, and every remaining token in the
//! file was absorbed into `g`'s body. The program exited 0 having silently
//! skipped every statement after the declaration. Any fixture containing a
//! default parameter had its self-checks vacuously "pass".
//!
//! The invariant: an unsupported parameter shape fails closed (E5506, nonzero
//! exit). It never produces a partial module with a zero exit code.

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

/// The whole point of the test: a program using an unsupported parameter shape
/// must NOT exit 0 with the post-declaration statements silently dropped. It
/// must produce an E5506 naming the construct and a nonzero exit.
fn assert_fails_closed(src: &str, construct: &str) {
    let out = run_source(src);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert!(
        !out.status.success(),
        "SILENT TRUNCATION: exited 0 for an unsupported `{construct}`.\n\
         stdout: {stdout}\nstderr: {stderr}"
    );
    let combined = format!("{stdout}{stderr}");
    assert!(
        combined.contains("E5506"),
        "expected an E5506 diagnostic, got:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        combined.contains(construct),
        "expected the diagnostic to name `{construct}`, got:\n\
         stdout: {stdout}\nstderr: {stderr}"
    );
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

#[test]
fn default_param_function_declaration_fails_closed() {
    assert_fails_closed(
        "function g(b = 5) { return b; }\nconsole.log(\"after\");\n",
        "default parameter",
    );
}

#[test]
fn default_param_function_expression_fails_closed() {
    assert_fails_closed(
        "const g = function (b = 5) { return b; };\nconsole.log(\"after\");\n",
        "default parameter",
    );
}

#[test]
fn default_param_arrow_fails_closed() {
    assert_fails_closed(
        "const g = (b = 5) => b;\nconsole.log(\"after\");\n",
        "default parameter",
    );
}

#[test]
fn default_param_class_method_fails_closed() {
    assert_fails_closed(
        "class C { m(b = 5) { return b; } }\nconsole.log(\"after\");\n",
        "default parameter",
    );
}

#[test]
fn rest_param_fails_closed() {
    assert_fails_closed(
        "function g(...r) { return 1; }\nconsole.log(\"after\");\n",
        "rest parameter",
    );
}

#[test]
fn object_destructured_param_fails_closed() {
    assert_fails_closed(
        "function g({ x }) { return x; }\nconsole.log(\"after\");\n",
        "destructured parameter",
    );
}

#[test]
fn array_destructured_param_fails_closed() {
    assert_fails_closed(
        "function g([x]) { return x; }\nconsole.log(\"after\");\n",
        "destructured parameter",
    );
}

/// Negative control: a trailing comma is a SUPPORTED shape and must keep
/// working. If the fix over-rejects, this goes red.
#[test]
fn trailing_comma_param_still_runs() {
    let src = "function g(a,) { return a; }\nconsole.log(\"v=\" + g(7));\n";
    let out = run_source(src);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "v=7");
}

/// Negative control: ordinary parameters are unaffected.
#[test]
fn plain_params_still_run() {
    let src = "function g(a, b) { return a + b; }\nconsole.log(\"v=\" + g(3, 4));\n";
    let out = run_source(src);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "v=7");
}
