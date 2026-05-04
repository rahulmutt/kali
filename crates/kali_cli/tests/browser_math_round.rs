use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_math_round_run_source() -> &'static str {
    "console.log(Math.round(1.6));\n"
}

fn browser_harness_math_round_test_source() -> &'static str {
    r#"Kali.test('math round', () => {
  console.log(Math.round(1.6));
});
"#
}

fn assert_browser_harness_math_round(command: &str, filename: &str, source: &str) {
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
    assert!(stdout.contains('2'), "stdout: {stdout}");
}

#[test]
fn run_supports_math_round_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_round("run", "main.ts", browser_harness_math_round_run_source());
}

#[test]
fn run_supports_math_round_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_round("run", "main.js", browser_harness_math_round_run_source());
}

#[test]
fn run_supports_math_round_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_math_round("run", "main.jsx", browser_harness_math_round_run_source());
}

#[test]
fn run_supports_math_round_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_math_round("run", "main.tsx", browser_harness_math_round_run_source());
}

#[test]
fn test_supports_math_round_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_round(
        "test",
        "smoke.test.ts",
        browser_harness_math_round_test_source(),
    );
}

#[test]
fn test_supports_math_round_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_round(
        "test",
        "smoke.test.js",
        browser_harness_math_round_test_source(),
    );
}

#[test]
fn test_supports_math_round_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_math_round(
        "test",
        "smoke.test.jsx",
        browser_harness_math_round_test_source(),
    );
}

#[test]
fn test_supports_math_round_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_math_round(
        "test",
        "smoke.test.tsx",
        browser_harness_math_round_test_source(),
    );
}

fn browser_harness_math_round_alias_run_source() -> &'static str {
    "const value = 1.6; const alias = value; console.log(Math.round(alias));\n"
}

fn browser_harness_math_round_alias_test_source() -> &'static str {
    r#"Kali.test('math round alias chain', () => {
  const value = 1.6;
  const alias = value;
  console.log(Math.round(alias));
});
"#
}

#[test]
fn run_supports_math_round_alias_chain_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_round(
        "run",
        "main.ts",
        browser_harness_math_round_alias_run_source(),
    );
}

#[test]
fn run_supports_math_round_alias_chain_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_round(
        "run",
        "main.js",
        browser_harness_math_round_alias_run_source(),
    );
}

#[test]
fn test_supports_math_round_alias_chain_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_round(
        "test",
        "smoke.test.ts",
        browser_harness_math_round_alias_test_source(),
    );
}

#[test]
fn test_supports_math_round_alias_chain_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_round(
        "test",
        "smoke.test.js",
        browser_harness_math_round_alias_test_source(),
    );
}
