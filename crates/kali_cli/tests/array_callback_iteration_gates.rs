use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn array_callback_iteration_source() -> &'static str {
    r#"function main() {
  const values = [1, 2];
  for (const item of values.map((value) => value)) {
    console.log(item);
  }
}
main();
"#
}

fn assert_array_callback_iteration_source_rejects(command: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, array_callback_iteration_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg(command)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert_eq!(output.status.code(), Some(1));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("for-of array iteration lowering is unavailable"),
        "stderr: {stderr}"
    );
}

#[test]
fn run_rejects_array_callback_iteration_lowering_in_js_input() {
    assert_array_callback_iteration_source_rejects("run");
}

#[test]
fn test_rejects_array_callback_iteration_lowering_in_js_input() {
    assert_array_callback_iteration_source_rejects("test");
}

#[test]
fn check_rejects_array_callback_iteration_lowering_in_js_input() {
    assert_array_callback_iteration_source_rejects("check");
}

#[test]
fn build_rejects_array_callback_iteration_lowering_in_js_input() {
    assert_array_callback_iteration_source_rejects("build");
}
