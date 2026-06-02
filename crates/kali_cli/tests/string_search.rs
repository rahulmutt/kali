use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

#[test]
fn run_supports_static_ascii_string_search_helpers() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("string-search.js");
    fs::write(
        &source_path,
        "console.log(\"hello\".includes(\"ell\"));\nconsole.log(\"hello\".indexOf(\"l\", 3));\nconsole.log(\"hello\".lastIndexOf(\"l\"));\nconsole.log(\"hello\".lastIndexOf(\"l\", 2));\nconsole.log(\"hello\".lastIndexOf(\"l\", -1));\n",
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
    assert_eq!(String::from_utf8_lossy(&output.stdout), "1\n3\n3\n2\n-1\n");
}

#[test]
fn check_rejects_dynamic_static_string_search_operand() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("string-search-dynamic.js");
    fs::write(
        &source_path,
        "function has(needle) { return \"hello\".includes(needle); }\n",
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
    assert!(stderr.contains("string search method"), "stderr: {stderr}");
}
