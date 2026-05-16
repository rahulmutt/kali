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
    "apiSurface": "browser"
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

fn assert_browser_requested_accepts_zero_spawned_process_budget(
    command: &str,
    source_name: &str,
    source: &str,
) {
    for json_output in [false, true] {
        let dir = tempdir().expect("tempdir");
        let source_path = dir.path().join(source_name);
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
            .arg("--max-spawned-processes")
            .arg("0")
            .arg(&source_path)
            .output()
            .expect("run kali");

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
                assert_eq!(json["payload"]["skipped"], 0);
            }
            assert!(
                json["stdout"]
                    .as_str()
                    .expect("stdout string")
                    .contains("browser spawned process budget ok"),
                "json: {json}"
            );
            assert_eq!(json["stderr"], "");
            assert!(json["errors"].as_array().expect("errors array").is_empty());
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                stdout.contains("browser spawned process budget ok"),
                "stdout: {stdout}"
            );
            if command == "test" {
                assert!(stdout.contains("ok 1"), "stdout: {stdout}");
            }
        }
    }
}

#[test]
fn run_supports_zero_spawned_process_budget_when_browser_harness_is_configured_in_js_input() {
    assert_browser_requested_accepts_zero_spawned_process_budget(
        "run",
        "main.js",
        "console.log('browser spawned process budget ok');\n",
    );
}

#[test]
fn test_supports_zero_spawned_process_budget_when_browser_harness_is_configured_in_js_input() {
    assert_browser_requested_accepts_zero_spawned_process_budget(
        "test",
        "smoke.test.js",
        r#"Kali.test('browser spawned process budget', () => {
  console.log('browser spawned process budget ok');
});
"#,
    );
}
