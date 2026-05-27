use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn identity_filter_source() -> &'static str {
    r#"function main() {
  for (const item of [1, 2].filter((value) => value)) {
    console.log(item);
  }
}
main();
"#
}

fn assert_identity_filter_succeeds(command: &str, extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("smoke.test.{extension}"));
    fs::write(&source_path, identity_filter_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg(command)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "{command} failed: {:?}", output);

    if command == "run" {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "1\n2\n", "unexpected stdout: {stdout}");
    } else if command == "test" {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("1\n2\n"), "unexpected stdout: {stdout}");
    }
}

fn assert_identity_filter_browser_check_build_succeeds(command: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, identity_filter_source()).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path());
    cli.arg(command);
    if command == "build" {
        cli.arg("--bundle");
    }
    let output = cli
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "{command} failed: {:?}", output);
}

#[test]
fn check_supports_truthy_identity_array_filter_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_identity_filter_succeeds("check", extension);
    }
}

#[test]
fn build_supports_truthy_identity_array_filter_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_identity_filter_succeeds("build", extension);
    }
}

#[test]
fn run_supports_truthy_identity_array_filter_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_identity_filter_succeeds("run", extension);
    }
}

#[test]
fn test_supports_truthy_identity_array_filter_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_identity_filter_succeeds("test", extension);
    }
}

#[test]
fn check_supports_truthy_identity_array_filter_in_browser_api_surface_js_input() {
    assert_identity_filter_browser_check_build_succeeds("check");
}

#[test]
fn build_supports_truthy_identity_array_filter_in_browser_api_surface_js_input() {
    assert_identity_filter_browser_check_build_succeeds("build");
}
