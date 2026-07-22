// Stage P5 — String() coercion + TextEncoder/TextDecoder soundness pins.
use std::fs;
use std::process::{Command, Output};
use tempfile::tempdir;

fn kali_bin() -> String {
    env!("CARGO_BIN_EXE_kali").to_string()
}

fn run(source: &str) -> Output {
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

/// Compile+run, assert success, return trimmed stdout.
fn run_ok(source: &str) -> String {
    let out = run(source);
    assert!(
        out.status.success(),
        "expected success\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Compile+run, assert fail-closed, return stderr.
fn run_e5506(source: &str) -> String {
    let out = run(source);
    assert!(
        !out.status.success(),
        "expected fail-closed E5506\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(stderr.contains("E5506"), "expected E5506, got: {stderr}");
    stderr
}

#[test]
fn string_of_i64_renders_decimal() {
    assert_eq!(run_ok("console.log(String(40n + 2n));"), "42");
}

#[test]
fn string_of_negative_i64_renders_sign() {
    assert_eq!(run_ok("console.log(String(0n - 7n));"), "-7");
}

#[test]
fn string_of_float_renders() {
    assert_eq!(run_ok("console.log(String(3.5));"), "3.5");
}

#[test]
fn string_of_boolean_renders_word() {
    assert_eq!(run_ok("console.log(String(1n === 1n));"), "true");
}

#[test]
fn string_of_string_is_identity() {
    assert_eq!(run_ok("console.log(String('hi'));"), "hi");
}

#[test]
fn string_of_object_fails_closed() {
    run_e5506("const o = { a: 1n }; console.log(String(o));");
}

#[test]
fn string_zero_arg_fails_closed() {
    run_e5506("console.log(String());");
}

#[test]
fn string_multi_arg_fails_closed() {
    run_e5506("console.log(String(1n, 2n));");
}
