//! Runtime float values print through console with JS `String(number)`
//! semantics. Regression for the emitter passing raw f64 into the i64-typed
//! console imports (wasm validation failure: "expected type i64, found
//! f64.div").
use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn run_fixture(source: &str) -> (bool, String, String) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("app.ts");
    fs::write(&source_path, source).expect("write source");
    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn run_prints_runtime_float_division_results() {
    let (ok, stdout, stderr) = run_fixture(
        "console.log(7 / 2);\nconsole.log(6 / 2);\nconsole.log(1.5 + 2);\nconst x = 7 / 2;\nconsole.log(x);\n",
    );
    assert!(ok, "stdout: {stdout}\nstderr: {stderr}");
    assert_eq!(stdout, "3.5\n3\n3.5\n3.5\n");
}

#[test]
fn run_prints_js_special_float_values() {
    let (ok, stdout, stderr) = run_fixture(
        "console.log(7 / 0);\nconsole.log(-7 / 0);\nconsole.log(0 / 0);\nconsole.log(0 / -1);\n",
    );
    assert!(ok, "stdout: {stdout}\nstderr: {stderr}");
    assert_eq!(stdout, "Infinity\n-Infinity\nNaN\n0\n");
}

#[test]
fn run_concatenates_runtime_floats_into_strings() {
    let (ok, stdout, stderr) = run_fixture("console.log(\"v: \" + (7 / 2));\n");
    assert!(ok, "stdout: {stdout}\nstderr: {stderr}");
    assert_eq!(stdout, "v: 3.5\n");
}

#[test]
fn run_prints_floats_read_from_mutable_locals_and_params() {
    let (ok, stdout, stderr) = run_fixture(
        "let x = 7 / 2;\nconsole.log(x);\nconsole.log(\"v: \" + x);\nlet y = 1.5;\nconsole.log(y);\nfunction show(v) {\n  console.log(v);\n  console.log(\"p: \" + v);\n}\nshow(9 / 2);\n",
    );
    assert!(ok, "stdout: {stdout}\nstderr: {stderr}");
    assert_eq!(stdout, "3.5\nv: 3.5\n1.5\n4.5\np: 4.5\n");
}

#[test]
fn run_prints_small_magnitudes_with_js_exponent_notation() {
    // Was the recorded reachable divergence: host printed 0.0000001 while the
    // browser mirrors printed 1e-7.
    let (ok, stdout, stderr) = run_fixture("console.log(1 / 10000000);\n");
    assert!(ok, "stdout: {stdout}\nstderr: {stderr}");
    assert_eq!(stdout, "1e-7\n");
}
