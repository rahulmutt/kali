use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn late_js_compatibility_source() -> &'static str {
    "Intl; globalThis.Intl; globalThis.Intl.NumberFormat; Intl.NumberFormat; Deno.pid; globalThis.Deno.pid; globalThis.Deno.cwd; Deno.chdir; globalThis.Deno.chdir; globalThis.Deno.exit; process.pid; globalThis.process.pid; globalThis.process.cwd; process.chdir; globalThis.process.chdir; globalThis.process.exit; Proxy; globalThis.Proxy; new WeakMap(); globalThis.WeakMap; new WeakSet(); globalThis.WeakSet; new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry;"
}

fn assert_late_js_compatibility_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("E3100"), "stderr: {stderr}");
    assert!(
        stderr.contains("undefined identifier 'process'"),
        "stderr: {stderr}"
    );
    for expected in [
        "Intl",
        "globalThis.Intl",
        "globalThis.Intl.NumberFormat",
        "Deno.pid",
        "globalThis.Deno.pid",
        "globalThis.Deno.cwd",
        "Deno.chdir",
        "globalThis.Deno.chdir",
        "globalThis.Deno.exit",
        "process.pid",
        "globalThis.process.pid",
        "globalThis.process.cwd",
        "process.chdir",
        "globalThis.process.chdir",
        "globalThis.process.exit",
        "Proxy",
        "globalThis.Proxy",
        "WeakMap",
        "WeakSet",
        "FinalizationRegistry",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

#[test]
fn check_rejects_late_compatibility_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_js_compatibility_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_late_js_compatibility_rejection(&stderr);
}

#[test]
fn run_rejects_late_compatibility_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_js_compatibility_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_late_js_compatibility_rejection(&stderr);
}

#[test]
fn test_rejects_late_compatibility_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_js_compatibility_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_late_js_compatibility_rejection(&stderr);
}
