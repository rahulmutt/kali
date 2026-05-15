use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
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

fn assert_empty_thread_topology(value: &Value) {
    assert_eq!(value["totalInstances"], 0);
    assert_eq!(value["terminatedInstances"], 0);
    assert_eq!(value["liveInstances"], serde_json::json!([]));
}

fn assert_browser_wasm_threads_acceptance_for_command(command: &str, source_name: &str) {
    for json_output in [false, true] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(source_name);
        let source = if command == "test" {
            r#"Kali.test('browser wasm threads', () => {
  console.log('browser wasm threads ok');
});
"#
        } else {
            "console.log('browser wasm threads ok');\n"
        };
        fs::write(&source_path, source).expect("write source");
        write_browser_api_surface_manifest(&dir);

        let mut cli = Command::new(kali_bin());
        cli.env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
            .current_dir(dir.path());
        if json_output {
            cli.arg("--output").arg("json");
        }
        let output = cli
            .arg(command)
            .arg("--max-threads")
            .arg("1")
            .arg("--max-spawned-processes")
            .arg("0")
            .arg(&source_path)
            .output()
            .expect("run kali");

        assert!(output.status.success());

        if json_output {
            let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
            assert_eq!(json["schemaVersion"], 1);
            assert_eq!(json["command"], command);
            assert_eq!(json["success"], true);
            assert_eq!(json["payload"]["hostContract"], "browser-requested");
            assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
            assert_empty_thread_topology(&json["payload"]["threadTopology"]);
            if command == "run" {
                assert_eq!(json["exitCode"], 0);
                assert_eq!(json["payload"]["exitCode"], 0);
            } else {
                assert_eq!(json["payload"]["total"], 1);
                assert_eq!(json["payload"]["passed"], 1);
                assert_eq!(json["payload"]["failed"], 0);
            }
            assert!(
                json["stdout"]
                    .as_str()
                    .expect("stdout string")
                    .contains("browser wasm threads ok"),
                "json: {json}"
            );
            assert_eq!(json["stderr"], "");
            assert!(json["errors"].as_array().expect("errors array").is_empty());
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                stdout.contains("browser wasm threads ok"),
                "stdout: {stdout}"
            );
            if command == "test" {
                assert!(stdout.contains("ok 1"), "stdout: {stdout}");
            }
        }
    }
}

#[test]
fn run_supports_inherited_browser_api_surface_with_wasm_threads_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_wasm_threads_acceptance_for_command("run", "main.js");
}

#[test]
fn test_supports_inherited_browser_api_surface_with_wasm_threads_in_js_input_when_browser_harness_is_configured(
) {
    assert_browser_wasm_threads_acceptance_for_command("test", "smoke.test.js");
}
