use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn flat_map_identity_source() -> &'static str {
    r#"function main() {
  for (const item of [1, 2].flatMap((value) => [value])) {
    console.log(item);
  }
}
main();
"#
}

fn assert_flat_map_identity_succeeds(command: &str, extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("smoke.test.{extension}"));
    fs::write(&source_path, flat_map_identity_source()).expect("write source");

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

fn assert_flat_map_identity_browser_check_build_succeeds(command: &str, extension: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("main.{extension}"));
    fs::write(&source_path, flat_map_identity_source()).expect("write source");

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
fn check_supports_identity_array_flat_map_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_flat_map_identity_succeeds("check", extension);
    }
}

#[test]
fn build_supports_identity_array_flat_map_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_flat_map_identity_succeeds("build", extension);
    }
}

#[test]
fn run_supports_identity_array_flat_map_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_flat_map_identity_succeeds("run", extension);
    }
}

#[test]
fn test_supports_identity_array_flat_map_in_js_and_ts_input() {
    for extension in ["js", "ts"] {
        assert_flat_map_identity_succeeds("test", extension);
    }
}

#[test]
fn check_supports_identity_array_flat_map_in_browser_api_surface_js_ts_jsx_and_tsx_input() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_flat_map_identity_browser_check_build_succeeds("check", extension);
    }
}

#[test]
fn build_supports_identity_array_flat_map_in_browser_api_surface_js_ts_jsx_and_tsx_input() {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_flat_map_identity_browser_check_build_succeeds("build", extension);
    }
}
