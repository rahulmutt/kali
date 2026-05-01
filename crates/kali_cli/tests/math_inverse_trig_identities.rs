use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn math_inverse_trig_source() -> &'static str {
    "console.log(Math.asin(0));\nconsole.log(Math.acos(1));\nconsole.log(Math.atan(0));\n"
}

fn assert_check_supports_math_inverse_trig_identity_literals_in_js_input(json: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, math_inverse_trig_source()).expect("write source");

    let mut command = Command::new(kali_bin());
    command.current_dir(dir.path());
    if json {
        command.arg("--output").arg("json");
    }
    let output = command
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));

    if json {
        let json: Value = serde_json::from_slice(&output.stdout).expect("parse json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], "check");
        assert_eq!(json["success"], true);
    }
}

#[test]
fn check_supports_math_inverse_trig_identity_literals_in_js_input() {
    assert_check_supports_math_inverse_trig_identity_literals_in_js_input(false);
}

#[test]
fn json_check_supports_math_inverse_trig_identity_literals_in_js_input() {
    assert_check_supports_math_inverse_trig_identity_literals_in_js_input(true);
}

#[test]
fn run_supports_math_inverse_trig_identity_literals_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, math_inverse_trig_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.matches("0\n").count() >= 3, "stdout: {stdout}");
}
