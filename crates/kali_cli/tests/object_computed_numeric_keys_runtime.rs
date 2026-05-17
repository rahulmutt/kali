use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn assert_computed_numeric_object_keys_run(filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        "const obj = { [-1]: 'neg', [+2]: 'pos', [(-0)]: 'zero' };\nconsole.log(obj[-1]);\nconsole.log(obj[2]);\nconsole.log(obj[0]);\n",
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

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("neg\n"), "stdout: {stdout}");
    assert!(stdout.contains("pos\n"), "stdout: {stdout}");
    assert!(stdout.contains("zero\n"), "stdout: {stdout}");
}

fn assert_computed_numeric_object_keys_run_with_await_wrappers(filename: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        "async function computedNumericObjectKeysWithAwaitWrappers() {\n  const obj = { [await 1]: 'neg', [+(await 2)]: 'pos', [(0, await 0)]: 'zero' };\n  console.log(obj[1]);\n  console.log(obj[2]);\n  console.log(obj[0]);\n}\ncomputedNumericObjectKeysWithAwaitWrappers();\n",
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

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("neg\n"), "stdout: {stdout}");
    assert!(stdout.contains("pos\n"), "stdout: {stdout}");
    assert!(stdout.contains("zero\n"), "stdout: {stdout}");
}

#[test]
fn run_supports_unary_numeric_computed_object_keys_in_js_input() {
    assert_computed_numeric_object_keys_run("main.js");
}

#[test]
fn run_supports_unary_numeric_computed_object_keys_in_ts_input() {
    assert_computed_numeric_object_keys_run("main.ts");
}

#[test]
fn run_supports_await_wrapped_computed_numeric_object_keys_in_js_input() {
    assert_computed_numeric_object_keys_run_with_await_wrappers("main.js");
}

#[test]
fn run_supports_await_wrapped_computed_numeric_object_keys_in_ts_input() {
    assert_computed_numeric_object_keys_run_with_await_wrappers("main.ts");
}
