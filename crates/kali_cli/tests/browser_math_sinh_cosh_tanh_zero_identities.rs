use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_math_sinh_cosh_tanh_run_source() -> &'static str {
    "const zero = 0; console.log(Math.sinh(zero)); console.log(Math.cosh(zero)); console.log(Math.tanh(zero));\n"
}

fn browser_harness_math_sinh_cosh_tanh_test_source() -> &'static str {
    r#"Kali.test('math sinh/cosh/tanh zero identities', () => {
  const zero = 0;
  console.log(Math.sinh(zero));
  console.log(Math.cosh(zero));
  console.log(Math.tanh(zero));
});
"#
}

fn assert_browser_harness_math_sinh_cosh_tanh(command: &str, filename: &str, source: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg(command)
        .arg("--api")
        .arg("browser")
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
    assert!(stdout.contains("0\n"), "stdout: {stdout}");
    assert!(stdout.contains("1\n"), "stdout: {stdout}");
}

#[test]
fn run_supports_math_sinh_cosh_and_tanh_zero_identities_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_harness_math_sinh_cosh_tanh(
        "run",
        "main.ts",
        browser_harness_math_sinh_cosh_tanh_run_source(),
    );
}

#[test]
fn run_supports_math_sinh_cosh_and_tanh_zero_identities_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_harness_math_sinh_cosh_tanh(
        "run",
        "main.js",
        browser_harness_math_sinh_cosh_tanh_run_source(),
    );
}

#[test]
fn test_supports_math_sinh_cosh_and_tanh_zero_identities_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_harness_math_sinh_cosh_tanh(
        "test",
        "smoke.test.ts",
        browser_harness_math_sinh_cosh_tanh_test_source(),
    );
}

#[test]
fn test_supports_math_sinh_cosh_and_tanh_zero_identities_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_harness_math_sinh_cosh_tanh(
        "test",
        "smoke.test.js",
        browser_harness_math_sinh_cosh_tanh_test_source(),
    );
}
