//! Stage P4 (URL + URLSearchParams lane) — Task 3: compile-time-parse
//! construction of both handles, the URL component reads (pure loads over the
//! parsed-at-compile-time arena struct), USP construction, and the shared
//! escape-restricted position gate (a raw read of a URL/USP handle fails closed
//! E5506). USP methods land in Task 4; here construction only proves it
//! compiles + runs.

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

/// Run `kali run`, assert it succeeded, and return stdout (caller trims).
fn run_kali_run(source: &str) -> String {
    let out = run_kali(source);
    assert!(
        out.status.success(),
        "expected success; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Run `kali run` expecting a fail-closed compile (nonzero exit); return stderr
/// so the caller can assert the diagnostic code (E5506).
fn run_kali_run_expect_error(source: &str) -> String {
    let out = run_kali(source);
    assert!(
        !out.status.success(),
        "expected a fail-closed compile (nonzero exit), got success; stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn url_pathname_reads_the_parsed_component() {
    let src = "const u = new URL('https://example.com/browser?alpha=1#fragment');\nconsole.log(u.pathname);\n";
    assert_eq!(run_kali_run(src).trim(), "/browser");
}

#[test]
fn url_origin_search_hash_href_read_parsed_components() {
    let src = "const u = new URL('https://example.com/browser?alpha=1#fragment');\nconsole.log(u.origin);\nconsole.log(u.search);\nconsole.log(u.hash);\nconsole.log(u.href);\n";
    assert_eq!(
        run_kali_run(src).trim(),
        "https://example.com\n?alpha=1\n#fragment\nhttps://example.com/browser?alpha=1#fragment"
    );
}

#[test]
fn usp_construction_builds_and_runs() {
    // USP methods land in Task 4; here we only prove construction compiles/runs.
    let src = "const q = new URLSearchParams('alpha=1&beta=two+words');\nconsole.log('ok');\n";
    assert_eq!(run_kali_run(src).trim(), "ok");
}

#[test]
fn url_raw_print_fails_closed() {
    let src = "const u = new URL('https://x/');\nconsole.log(u);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn usp_raw_print_fails_closed() {
    let src = "const q = new URLSearchParams('a=1');\nconsole.log(q);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn usp_from_url_searchParams_raw_print_fails_closed() {
    let src = "const u = new URL('https://example.com/p?a=1');\nconsole.log(u.searchParams);\n";
    let stderr = run_kali_run_expect_error(src);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}

#[test]
fn non_literal_url_arg_fails_closed() {
    let src = "const s = 'https://x/';\nconst u = new URL(s);\nconsole.log(u.href);\n";
    let out = run_kali(src);
    assert!(!out.status.success(), "must fail closed: {out:?}");
}
