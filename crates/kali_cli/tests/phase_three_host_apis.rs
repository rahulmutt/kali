use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn phase_three_host_api_source() -> &'static str {
    "new Deno.Command('sh').spawn();\nDeno.connect('127.0.0.1', 1);\nDeno.listen('127.0.0.1', 0);\nDeno.serve('127.0.0.1', 0);\n"
}

fn assert_phase_three_host_apis_rejected(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    for expected in ["Deno.Command", "Deno.connect", "Deno.listen", "Deno.serve"] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
    assert!(
        stderr.contains("subprocess spawning API")
            || stderr.contains("socket/listener networking API"),
        "stderr: {stderr}"
    );
}

fn assert_phase_three_host_apis_rejected_for_source(source_name: &str, command: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(source_name);
    fs::write(&source_path, phase_three_host_api_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg(command)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_phase_three_host_apis_rejected(&stderr);
    if command == "build" {
        assert!(
            !dir.path().join("main.wasm").exists(),
            "build should not emit an artifact when phase-three host APIs are rejected"
        );
    }
}

#[test]
fn check_rejects_phase_three_host_apis_in_js_input() {
    assert_phase_three_host_apis_rejected_for_source("main.js", "check");
}

#[test]
fn check_rejects_phase_three_host_apis_in_jsx_input() {
    assert_phase_three_host_apis_rejected_for_source("main.jsx", "check");
}

#[test]
fn check_rejects_phase_three_host_apis_in_tsx_input() {
    assert_phase_three_host_apis_rejected_for_source("main.tsx", "check");
}

#[test]
fn build_rejects_phase_three_host_apis_in_js_input() {
    assert_phase_three_host_apis_rejected_for_source("main.js", "build");
}

#[test]
fn build_rejects_phase_three_host_apis_in_jsx_input() {
    assert_phase_three_host_apis_rejected_for_source("main.jsx", "build");
}

#[test]
fn build_rejects_phase_three_host_apis_in_tsx_input() {
    assert_phase_three_host_apis_rejected_for_source("main.tsx", "build");
}
