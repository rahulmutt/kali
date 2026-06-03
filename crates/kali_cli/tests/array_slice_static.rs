use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

#[test]
fn run_supports_static_literal_array_slice_indexed_access() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("array-slice.js");
    fs::write(
        &source_path,
        "console.log([10, 20, 30, 40].slice(1, 3)[0]);\nconsole.log([10, 20, 30, 40].slice(-2)[1]);\nconsole.log([10, 20, 30, 40].slice(1.8, 3.2)[1]);\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "20\n40\n30\n");
}

#[test]
fn check_rejects_dynamic_literal_array_slice_bounds() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("array-slice-dynamic.js");
    fs::write(
        &source_path,
        "function get(start) { return [10, 20, 30].slice(start)[0]; }\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success(), "expected check to fail");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("Array.prototype.slice"), "stderr: {stderr}");
}
