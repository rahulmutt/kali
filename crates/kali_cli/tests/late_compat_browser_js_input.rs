use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn late_process_control_source() -> &'static str {
    "Deno.pid; globalThis.Deno.pid; globalThis.Deno.cwd; Deno.chdir; globalThis.Deno.chdir; globalThis.Deno.exit; process.pid; globalThis.process.pid; globalThis.process.cwd; process.chdir; globalThis.process.chdir; globalThis.process.exit;"
}

fn assert_browser_late_process_control_rejection(stderr: &str) {
    assert!(stderr.contains("E3100"), "stderr: {stderr}");
    assert!(
        stderr.contains("undefined identifier 'process'"),
        "stderr: {stderr}"
    );
    for expected in [
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
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

#[test]
fn check_rejects_late_process_control_members_in_browser_api_surface_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_process_control_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_process_control_rejection(&stderr);
}

#[test]
fn build_rejects_late_process_control_members_in_browser_bundle_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_process_control_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_process_control_rejection(&stderr);
}
