use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn reduce_source(command: &str) -> String {
    let body = r#"function reduceSlices() {
  console.log([1, 2, 3].reduce((acc, value) => acc + value, 0));
  console.log([1, 2, 3].reduceRight((acc, value) => acc - value, 0));
  console.log([1, 2, 3].reduce((acc, value) => acc + value));
  console.log([1, 2, 3].reduceRight((acc, value) => acc - value));
}
reduceSlices();
"#;

    match command {
        "test" => format!("Kali.test('reduce slices', () => {{\n{body}}});\n"),
        _ => body.to_string(),
    }
}

fn assert_reduce_succeeds(command: &str, extension: &str, browser: bool, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(format!("reduce.test.{extension}"));
    fs::write(&source_path, reduce_source(command)).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path());
    if browser {
        cli.env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node");
    }
    if json_output {
        cli.arg("--output").arg("json");
    }
    cli.arg(command);
    if command == "build" && browser {
        cli.arg("--bundle");
    }
    if browser {
        cli.arg("--api").arg("browser");
    }
    if browser && matches!(command, "run" | "test") {
        cli.arg("--max-threads")
            .arg("0")
            .arg("--max-spawned-processes")
            .arg("0");
    }
    let output = cli.arg(&source_path).output().expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        if matches!(command, "run" | "test") {
            assert!(
                json["stdout"]
                    .as_str()
                    .expect("stdout")
                    .contains("6\n-6\n6\n0\n"),
                "json: {json:?}"
            );
        }
    } else if matches!(command, "run" | "test") {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("6\n-6\n6\n0\n"), "stdout: {stdout}");
    }
}

#[test]
fn check_build_run_and_test_support_numeric_reduce_slices() {
    for command in ["check", "build", "run", "test"] {
        for extension in ["js", "ts"] {
            assert_reduce_succeeds(command, extension, false, false);
        }
    }
}

#[test]
fn browser_check_and_bundle_support_numeric_reduce_slices() {
    for command in ["check", "build"] {
        for extension in ["js", "ts", "jsx", "tsx"] {
            assert_reduce_succeeds(command, extension, true, false);
        }
    }
}

#[test]
fn browser_harness_run_and_test_support_numeric_reduce_slices() {
    for command in ["run", "test"] {
        for extension in ["js", "ts", "jsx", "tsx"] {
            assert_reduce_succeeds(command, extension, true, false);
            assert_reduce_succeeds(command, extension, true, true);
        }
    }
}
