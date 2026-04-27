use serde_json::Value;
use std::{path::PathBuf, process::Command};
use tempfile::tempdir;

fn kali_bin() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_kali")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_kali")))
}

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("parse json stdout")
}

#[test]
fn init_reports_application_scaffold_in_json() {
    let dir = tempdir().expect("tempdir");
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("init")
        .current_dir(dir.path())
        .output()
        .expect("run kali init");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "init");
    assert_eq!(json["success"], true);
    assert!(json["errors"].as_array().expect("errors array").is_empty());
    assert!(json["warnings"]
        .as_array()
        .expect("warnings array")
        .is_empty());

    let payload = &json["payload"];
    assert_eq!(payload["root"], serde_json::json!(dir.path()));
    assert_eq!(
        payload["manifestPath"],
        serde_json::json!(dir.path().join("kali.json"))
    );
    assert_eq!(
        payload["sourcePath"],
        serde_json::json!(dir.path().join("main.ts"))
    );
    assert_eq!(payload["library"], false);

    assert!(dir.path().join("kali.json").exists());
    assert!(dir.path().join("main.ts").exists());
}

#[test]
fn init_reports_library_scaffold_in_json() {
    let dir = tempdir().expect("tempdir");
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("init")
        .arg("--lib")
        .current_dir(dir.path())
        .output()
        .expect("run kali init --lib");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "init");
    assert_eq!(json["success"], true);

    let payload = &json["payload"];
    assert_eq!(payload["root"], serde_json::json!(dir.path()));
    assert_eq!(
        payload["manifestPath"],
        serde_json::json!(dir.path().join("kali.json"))
    );
    assert_eq!(
        payload["sourcePath"],
        serde_json::json!(dir.path().join("lib.ts"))
    );
    assert_eq!(payload["library"], true);

    assert!(dir.path().join("kali.json").exists());
    assert!(dir.path().join("lib.ts").exists());
}

#[test]
fn init_reports_json_failure_when_manifest_exists() {
    let dir = tempdir().expect("tempdir");
    std::fs::write(dir.path().join("kali.json"), "{}").expect("write manifest");

    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("init")
        .current_dir(dir.path())
        .output()
        .expect("run kali init");

    assert!(
        !output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "init");
    assert_eq!(json["success"], false);
    assert!(json["payload"].is_null());
    assert_eq!(json["errors"].as_array().expect("errors array").len(), 1);
    assert_eq!(json["errors"][0]["code"], "E5508");
    assert!(json["errors"][0]["message"]
        .as_str()
        .expect("error message string")
        .contains("project scaffold already exists"));
}
