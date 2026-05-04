use std::{fs, process::Command};

use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn phase_three_host_api_source() -> &'static str {
    "new Deno.Command('sh').spawn();\nDeno.connect('127.0.0.1', 1);\nDeno.listen('127.0.0.1', 0);\nDeno.serve('127.0.0.1', 0);\n"
}

fn phase_three_subprocess_source() -> &'static str {
    "new Deno.Command('sh').spawn();\n"
}

fn phase_three_network_source() -> &'static str {
    "Deno.connect('127.0.0.1', 1);\n"
}

fn assert_phase_three_host_apis_rejected(stderr: &str, expected_members: &[&str]) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert_phase_three_host_apis_rejected_message(stderr, expected_members);
}

fn assert_phase_three_host_apis_rejected_message(messages: &str, expected_members: &[&str]) {
    for expected in expected_members {
        assert!(
            messages.contains(expected),
            "missing {expected} in messages: {messages}"
        );
    }
    assert!(
        messages.contains("subprocess spawning API")
            || messages.contains("socket/listener networking API"),
        "messages: {messages}"
    );
}

fn assert_phase_three_host_apis_rejected_for_source(
    source_name: &str,
    source_text: &str,
    command: &str,
    json_output: bool,
    expected_members: &[&str],
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(source_name);
    fs::write(&source_path, source_text).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path());
    if json_output {
        cli.arg("--output").arg("json");
    }
    let output = cli
        .arg(command)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    if json_output {
        let json: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(!errors.is_empty(), "errors array should not be empty");
        assert!(
            errors.iter().all(|error| error["code"] == "E5506"),
            "unexpected errors: {errors:?}"
        );
        let messages = errors
            .iter()
            .map(|error| error["message"].as_str().expect("error message"))
            .collect::<Vec<_>>()
            .join("\n");
        assert_phase_three_host_apis_rejected_message(&messages, expected_members);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_phase_three_host_apis_rejected(&stderr, expected_members);
    }
    if command == "build" {
        assert!(
            !dir.path().join("main.wasm").exists(),
            "build should not emit an artifact when phase-three host APIs are rejected"
        );
    }
}

#[test]
fn check_rejects_phase_three_host_apis_in_js_input() {
    assert_phase_three_host_apis_rejected_for_source(
        "main.js",
        phase_three_host_api_source(),
        "check",
        false,
        &["Deno.Command", "Deno.connect", "Deno.listen", "Deno.serve"],
    );
}

#[test]
fn check_rejects_phase_three_host_apis_in_jsx_input() {
    assert_phase_three_host_apis_rejected_for_source(
        "main.jsx",
        phase_three_host_api_source(),
        "check",
        false,
        &["Deno.Command", "Deno.connect", "Deno.listen", "Deno.serve"],
    );
}

#[test]
fn check_rejects_phase_three_host_apis_in_tsx_input() {
    assert_phase_three_host_apis_rejected_for_source(
        "main.tsx",
        phase_three_host_api_source(),
        "check",
        false,
        &["Deno.Command", "Deno.connect", "Deno.listen", "Deno.serve"],
    );
}

#[test]
fn build_rejects_phase_three_host_apis_in_js_input() {
    assert_phase_three_host_apis_rejected_for_source(
        "main.js",
        phase_three_host_api_source(),
        "build",
        false,
        &["Deno.Command", "Deno.connect", "Deno.listen", "Deno.serve"],
    );
}

#[test]
fn build_rejects_phase_three_host_apis_in_jsx_input() {
    assert_phase_three_host_apis_rejected_for_source(
        "main.jsx",
        phase_three_host_api_source(),
        "build",
        false,
        &["Deno.Command", "Deno.connect", "Deno.listen", "Deno.serve"],
    );
}

#[test]
fn build_rejects_phase_three_host_apis_in_tsx_input() {
    assert_phase_three_host_apis_rejected_for_source(
        "main.tsx",
        phase_three_host_api_source(),
        "build",
        false,
        &["Deno.Command", "Deno.connect", "Deno.listen", "Deno.serve"],
    );
}

#[test]
fn run_rejects_phase_three_subprocess_api_in_js_and_ts_input() {
    for source_name in ["main.js", "main.ts"] {
        assert_phase_three_host_apis_rejected_for_source(
            source_name,
            phase_three_subprocess_source(),
            "run",
            false,
            &["Deno.Command"],
        );
    }
}

#[test]
fn json_run_rejects_phase_three_subprocess_api_in_js_and_ts_input() {
    for source_name in ["main.js", "main.ts"] {
        assert_phase_three_host_apis_rejected_for_source(
            source_name,
            phase_three_subprocess_source(),
            "run",
            true,
            &["Deno.Command"],
        );
    }
}

#[test]
fn test_rejects_phase_three_network_api_in_js_and_ts_input() {
    for source_name in ["main.js", "main.ts"] {
        assert_phase_three_host_apis_rejected_for_source(
            source_name,
            phase_three_network_source(),
            "test",
            false,
            &["Deno.connect"],
        );
    }
}

#[test]
fn json_test_rejects_phase_three_network_api_in_js_and_ts_input() {
    for source_name in ["main.js", "main.ts"] {
        assert_phase_three_host_apis_rejected_for_source(
            source_name,
            phase_three_network_source(),
            "test",
            true,
            &["Deno.connect"],
        );
    }
}
