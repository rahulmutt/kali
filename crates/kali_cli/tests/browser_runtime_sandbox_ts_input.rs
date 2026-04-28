use std::{fs, path::Path, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn write_valid_policy(path: &Path) {
    fs::write(
        path,
        r#"{
  "schemaVersion": 1,
  "$schema": "https://kali.sh/schemas/policy-v1.json",
  "effects": {
    "fileSystem": { "read": false, "write": false },
    "network": { "fetch": false, "connect": false, "listen": false, "maxConnections": 1 },
    "process": { "spawn": false, "envRead": false, "envWrite": false },
    "timer": { "schedule": false, "maxTimeoutMs": 1000, "maxActiveTimers": 1 },
    "eval": false,
    "random": true,
    "console": true
  },
  "resources": {
    "maxMemoryMB": 256,
    "maxCpuTimeMs": 10000,
    "maxOpenFiles": 8,
    "maxSpawnedProcesses": 0,
    "maxThreads": 0
  }
}"#,
    )
    .expect("write policy");
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

fn assert_browser_runtime_rejection_text(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("standalone browser runtime contract"),
        "stderr: {stderr}"
    );
    assert!(
        stderr.contains("selected host contract: browser-requested"),
        "stderr: {stderr}"
    );
}

fn assert_browser_runtime_rejection_json(json: &Value, expected_origin: &str) {
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty(), "errors array should not be empty");
    let error = &errors[0];
    assert_eq!(error["code"], "E5506");
    let message = error["message"]
        .as_str()
        .expect("browser rejection message");
    assert!(
        message.contains("standalone browser runtime contract"),
        "message: {message}"
    );
    assert!(
        message.contains("selected host contract: browser-requested"),
        "message: {message}"
    );
    assert_eq!(error["context"]["origin"], expected_origin);
    assert_eq!(error["context"]["requestedValue"], "browser");
    assert_eq!(error["context"]["effectiveValue"], "browser");
}

#[test]
fn run_rejects_browser_api_surface_with_sandbox_in_ts_input_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "console.log('browser run');").expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_runtime_rejection_text(&stderr);
}

#[test]
fn json_run_rejects_browser_api_surface_with_sandbox_in_ts_input_when_browser_harness_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "console.log('browser run');").expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_browser_runtime_rejection_json(&json, "cli");
}

#[test]
fn test_rejects_inherited_browser_api_surface_with_sandbox_in_ts_input_when_browser_harness_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "Kali.test('browser', () => {});").expect("write source");
    write_browser_api_surface_manifest(&dir);
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("test")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_runtime_rejection_text(&stderr);
}

#[test]
fn json_test_rejects_inherited_browser_api_surface_with_sandbox_in_ts_input_when_browser_harness_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "Kali.test('browser', () => {});").expect("write source");
    write_browser_api_surface_manifest(&dir);
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_browser_runtime_rejection_json(&json, "config");
}

#[test]
fn run_rejects_inherited_browser_api_surface_with_sandbox_in_ts_input_when_browser_harness_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "console.log('browser run');").expect("write source");
    write_browser_api_surface_manifest(&dir);
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_runtime_rejection_text(&stderr);
}

#[test]
fn json_run_rejects_inherited_browser_api_surface_with_sandbox_in_ts_input_when_browser_harness_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "console.log('browser run');").expect("write source");
    write_browser_api_surface_manifest(&dir);
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_browser_runtime_rejection_json(&json, "config");
}

#[test]
fn test_rejects_browser_api_surface_with_sandbox_in_ts_input_when_browser_harness_is_configured() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "Kali.test('browser', () => {});").expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_runtime_rejection_text(&stderr);
}

#[test]
fn json_test_rejects_browser_api_surface_with_sandbox_in_ts_input_when_browser_harness_is_configured(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "Kali.test('browser', () => {});").expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_browser_runtime_rejection_json(&json, "cli");
}

#[test]
fn run_rejects_browser_api_surface_with_sandbox_in_ts_input_without_browser_harness() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "console.log('browser run');").expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .env_remove(kali_runtime::BROWSER_HARNESS_COMMAND_ENV)
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_runtime_rejection_text(&stderr);
}

#[test]
fn json_run_rejects_browser_api_surface_with_sandbox_in_ts_input_without_browser_harness() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "console.log('browser run');").expect("write source");
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .env_remove(kali_runtime::BROWSER_HARNESS_COMMAND_ENV)
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_browser_runtime_rejection_json(&json, "cli");
}

#[test]
fn test_rejects_inherited_browser_api_surface_with_sandbox_in_ts_input_without_browser_harness() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "Kali.test('browser', () => {});").expect("write source");
    write_browser_api_surface_manifest(&dir);
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .env_remove(kali_runtime::BROWSER_HARNESS_COMMAND_ENV)
        .current_dir(dir.path())
        .arg("test")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_runtime_rejection_text(&stderr);
}

#[test]
fn json_test_rejects_inherited_browser_api_surface_with_sandbox_in_ts_input_without_browser_harness(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.ts");
    let policy_path = dir.path().join("kali.policy.json");
    fs::write(&source_path, "Kali.test('browser', () => {});").expect("write source");
    write_browser_api_surface_manifest(&dir);
    write_valid_policy(&policy_path);

    let output = Command::new(kali_bin())
        .env_remove(kali_runtime::BROWSER_HARNESS_COMMAND_ENV)
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--sandbox")
        .arg(&policy_path)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_browser_runtime_rejection_json(&json, "config");
}
