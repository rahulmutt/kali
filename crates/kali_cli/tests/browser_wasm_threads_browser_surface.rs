use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
}

fn write_browser_api_surface_manifest(dir: &tempfile::TempDir) {
    fs::write(
        dir.path().join("kali.json"),
        r#"{
  "schemaVersion": 1,
  "compilerOptions": {
    "apiSurface": "browser",
    "runtimeProfiles": ["wasm-threads"]
  }
}"#,
    )
    .expect("write manifest");
}

fn assert_browser_wasm_threads_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("runtime profile") || stderr.contains("wasm-threads"),
        "stderr: {stderr}"
    );
}

fn assert_browser_wasm_threads_rejection_for_command(
    command: &str,
    command_args: &[&str],
    source_name: &str,
    with_browser_api_surface_manifest: bool,
    with_explicit_browser_api_surface: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(source_name);
    fs::write(&source_path, "let value = 1 + 2; value;\n").expect("write source");
    if with_browser_api_surface_manifest {
        write_browser_api_surface_manifest(&dir);
    }

    for json_output in [false, true] {
        let mut cli = Command::new(kali_bin());
        cli.current_dir(dir.path());
        if json_output {
            cli.arg("--output").arg("json");
        }
        cli.arg(command);
        for arg in command_args {
            cli.arg(arg);
        }
        if with_explicit_browser_api_surface {
            cli.arg("--api").arg("browser");
            cli.arg("--wasm-threads");
        }
        cli.arg(&source_path);

        let output = cli.output().expect("run kali");
        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(5));

        if json_output {
            let json = parse_json_stdout(&output);
            assert_eq!(json["schemaVersion"], 1);
            assert_eq!(json["command"], command);
            assert_eq!(json["success"], false);
            let errors = json["errors"].as_array().expect("errors array");
            assert!(!errors.is_empty(), "errors: {errors:?}");
            assert_eq!(errors[0]["code"], "E5506");
            assert!(errors[0]["message"]
                .as_str()
                .expect("error message")
                .contains("runtime profile"));
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert_browser_wasm_threads_rejection(&stderr);
        }
    }
}

#[test]
fn check_rejects_browser_api_surface_with_wasm_threads_in_jsx_and_tsx_inputs() {
    for source_name in ["app.jsx", "app.tsx"] {
        assert_browser_wasm_threads_rejection_for_command("check", &[], source_name, false, true);
        assert_browser_wasm_threads_rejection_for_command("check", &[], source_name, true, false);
    }
}

#[test]
fn build_rejects_browser_api_surface_with_wasm_threads_in_jsx_and_tsx_inputs() {
    for source_name in ["app.jsx", "app.tsx"] {
        assert_browser_wasm_threads_rejection_for_command(
            "build",
            &["--bundle"],
            source_name,
            false,
            true,
        );
        assert_browser_wasm_threads_rejection_for_command(
            "build",
            &["--bundle"],
            source_name,
            true,
            false,
        );
    }
}
