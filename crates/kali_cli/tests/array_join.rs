use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

#[test]
fn run_supports_static_array_join() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("array-join.js");
    fs::write(
        &source_path,
        "console.log([1, true, null, 'x'].join('-'));\nconsole.log(['a', 'b'].join());\n",
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
    assert_eq!(String::from_utf8_lossy(&output.stdout), "1-true--x\na,b\n");
}

#[test]
fn run_supports_static_array_to_string() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("array-to-string.js");
    fs::write(
        &source_path,
        "console.log([1, true, null, 'x'].toString());\nconsole.log(['a', 'b'].toString());\n",
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
    assert_eq!(String::from_utf8_lossy(&output.stdout), "1,true,,x\na,b\n");
}

#[test]
fn check_rejects_dynamic_array_join_operand() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("array-join-dynamic.js");
    fs::write(
        &source_path,
        "function join(separator) { return [1, 2].join(separator); }\n",
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
    assert!(stderr.contains("Array.prototype.join"), "stderr: {stderr}");
}

#[test]
fn check_rejects_argument_bearing_static_array_to_string() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("array-to-string-args.js");
    fs::write(&source_path, "console.log([1, 2].toString('-'));\n").expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--output")
        .arg("json")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success(), "expected check to fail");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"code\":\"E5506\""), "stdout: {stdout}");
    assert!(
        stdout.contains("Array.prototype.toString"),
        "stdout: {stdout}"
    );
}
