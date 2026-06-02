use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn filter_predicate_source() -> &'static str {
    r#"function main() {
  const values = [0, 1, 2, 3];
  for (const item of values.filter((value) => value > 1)) {
    console.log(item);
  }
}
main();
"#
}

fn assert_filter_predicate_succeeds(command: &str, extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("smoke.test.{extension}"));
    fs::write(&source_path, filter_predicate_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg(command)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(output.status.success(), "{command} failed: {:?}", output);

    if command == "run" {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout, "2\n3\n", "unexpected stdout: {stdout}");
    } else if command == "test" {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("2\n3\n"), "unexpected stdout: {stdout}");
    }
}

fn assert_filter_predicate_browser_check_build_succeeds(command: &str, extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, filter_predicate_source()).expect("write source");

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
fn check_supports_static_array_filter_predicate_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_filter_predicate_succeeds("check", extension);
    }
}

#[test]
fn build_supports_static_array_filter_predicate_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_filter_predicate_succeeds("build", extension);
    }
}

#[test]
fn run_supports_static_array_filter_predicate_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_filter_predicate_succeeds("run", extension);
    }
}

#[test]
fn test_supports_static_array_filter_predicate_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_filter_predicate_succeeds("test", extension);
    }
}

#[test]
fn check_supports_static_array_filter_predicate_in_browser_api_surface_js_ts_jsx_and_tsx_input() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_filter_predicate_browser_check_build_succeeds("check", extension);
    }
}

#[test]
fn build_supports_static_array_filter_predicate_in_browser_api_surface_js_ts_jsx_and_tsx_input() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_filter_predicate_browser_check_build_succeeds("build", extension);
    }
}
