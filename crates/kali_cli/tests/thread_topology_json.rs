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
