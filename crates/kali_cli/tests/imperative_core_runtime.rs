use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_js(source: &str) -> String {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, source).expect("write source");

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

    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn relational_operators_compute_booleans() {
    assert_eq!(run_js("console.log(3 < 5);\n"), "1\n");
    assert_eq!(run_js("console.log(5 < 3);\n"), "0\n");
    assert_eq!(run_js("console.log(5 > 3);\n"), "1\n");
    assert_eq!(run_js("console.log(3 >= 3);\n"), "1\n");
    assert_eq!(run_js("console.log(2 <= 1);\n"), "0\n");
}
