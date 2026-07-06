//! Interpolated template literals evaluate their `${...}` expressions at
//! runtime with string-`+` semantics (floats via `float_to_string`).
//! Regression for templates printing their raw source text.
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
fn run_interpolates_float_expressions() {
    let (ok, stdout, stderr) =
        run_fixture("console.log(`v: ${7 / 2}`);\nconst x = 7 / 2;\nconsole.log(`x=${x}`);\n");
    assert!(ok, "stdout: {stdout}\nstderr: {stderr}");
    assert_eq!(stdout, "v: 3.5\nx=3.5\n");
}

#[test]
fn run_interpolates_ints_strings_and_adjacent_segments() {
    let (ok, stdout, stderr) = run_fixture(
        "console.log(`${1}${2}`);\nconsole.log(`sum: ${1 + 2}`);\nconsole.log(`hi ${\"kali\"}!`);\n",
    );
    assert!(ok, "stdout: {stdout}\nstderr: {stderr}");
    assert_eq!(stdout, "12\nsum: 3\nhi kali!\n");
}

#[test]
fn run_interpolates_string_variables() {
    // Runtime string value flow: a string-typed variable operand in the
    // desugared `+` chain is proven `Repr::String` by the repr inference, so
    // the template interpolates correctly at runtime (previously rejected
    // with E3200) — see string_typed_variable_plus_operands_flow_at_runtime
    // in imperative_core_runtime.rs and runtime_string_value_flow.rs.
    let (ok, stdout, stderr) = run_fixture("const name = \"kali\";\nconsole.log(`hi ${name}!`);\n");
    assert!(ok, "stdout: {stdout}\nstderr: {stderr}");
    assert_eq!(stdout, "hi kali!\n");
}

#[test]
fn run_keeps_plain_templates_unchanged() {
    let (ok, stdout, stderr) = run_fixture("console.log(`hello`);\n");
    assert!(ok, "stdout: {stdout}\nstderr: {stderr}");
    assert_eq!(stdout, "hello\n");
}

#[test]
fn run_rejects_escaped_interpolation_with_e2004() {
    // `\${` would silently interpolate (template escapes are not processed);
    // reject cleanly instead — never silent divergence from JS.
    let (ok, stdout, stderr) = run_fixture("console.log(`cost: \\${5}`);\n");
    assert!(!ok, "expected rejection, got stdout: {stdout}");
    assert!(
        (stdout.clone() + &stderr).contains("E2004"),
        "stdout: {stdout}\nstderr: {stderr}"
    );
}

#[test]
fn run_interpolates_inside_functions() {
    let (ok, stdout, stderr) =
        run_fixture("function show(v) {\n  console.log(`p: ${v}`);\n}\nshow(9 / 2);\n");
    assert!(ok, "stdout: {stdout}\nstderr: {stderr}");
    assert_eq!(stdout, "p: 4.5\n");
}
