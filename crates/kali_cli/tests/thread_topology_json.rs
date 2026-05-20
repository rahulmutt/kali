use serde_json::Value;
use std::{fs, path::PathBuf, process::Command};
use tempfile::tempdir;

fn kali_bin() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_kali")
        .map(PathBuf::from)
        .expect("kali binary path")
}

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
}

fn assert_empty_thread_topology(value: &Value) {
    assert_eq!(value["totalInstances"], 0);
    assert_eq!(value["terminatedInstances"], 0);
    assert_eq!(value["liveInstances"], serde_json::json!([]));
}

fn assert_empty_thread_topology_when_browser_api_is_explicit(
    command: &str,
    source_name: &str,
    source: &str,
) {
    let dir = tempdir().expect("temp dir");
    let source_path = dir.path().join(source_name);
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
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

    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], command);
    assert_eq!(json["success"], true);
    let payload = &json["payload"];
    assert_eq!(payload["hostContract"], "browser-requested");
    assert_eq!(payload["runtimeBackend"], "browser-harness");
    assert_empty_thread_topology(&payload["threadTopology"]);
    if command == "run" {
        assert_eq!(json["exitCode"], 0);
        assert_eq!(payload["exitCode"], 0);
    } else {
        assert_eq!(payload["total"], 1);
        assert_eq!(payload["passed"], 1);
        assert_eq!(payload["failed"], 0);
        assert_eq!(payload["skipped"], 0);
    }
    assert_eq!(json["stderr"], "");
    assert!(json["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn json_run_payload_includes_an_empty_thread_topology_snapshot() {
    let dir = tempdir().expect("temp dir");
    let source_path = dir.path().join("run-thread-topology.ts");
    fs::write(&source_path, "console.log('run thread topology');\n").expect("write source");

    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali run");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    let payload = &json["payload"];
    assert_eq!(payload["hostContract"], "kali-hosted");
    assert_eq!(payload["runtimeBackend"], "wasmtime");
    assert_empty_thread_topology(&payload["threadTopology"]);
}

#[test]
fn json_test_payload_includes_an_empty_thread_topology_snapshot() {
    let dir = tempdir().expect("temp dir");
    let source_path = dir.path().join("test-thread-topology.test.ts");
    fs::write(
        &source_path,
        "Kali.test('thread topology', () => { console.log('thread topology'); });\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali test");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    let payload = &json["payload"];
    assert_eq!(payload["hostContract"], "kali-hosted");
    assert_eq!(payload["runtimeBackend"], "wasmtime");
    assert_empty_thread_topology(&payload["threadTopology"]);
}

#[test]
fn json_run_payload_includes_an_empty_thread_topology_snapshot_on_the_browser_api_surface() {
    assert_empty_thread_topology_when_browser_api_is_explicit(
        "run",
        "browser-run-thread-topology.ts",
        "console.log('browser run thread topology');\n",
    );
}

#[test]
fn json_test_payload_includes_an_empty_thread_topology_snapshot_on_the_browser_api_surface() {
    assert_empty_thread_topology_when_browser_api_is_explicit(
        "test",
        "browser-test-thread-topology.test.ts",
        "Kali.test('browser thread topology', () => { console.log('browser test thread topology'); });\n",
    );
}
