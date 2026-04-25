use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

fn kali_bin() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_kali")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_kali")))
}

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("parse json stdout")
}

#[test]
fn doctor_reports_env_selected_browser_harness_in_json() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("doctor")
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node --test")
        .output()
        .expect("run kali doctor");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "doctor");
    assert_eq!(json["success"], true);
    let harness = &json["payload"]["browserHarness"];
    assert_eq!(harness["envVar"], "KALI_BROWSER_BUNDLE_HARNESS_COMMAND");
    assert_eq!(harness["source"], "env");
    assert_eq!(harness["override"], "node --test");
    assert_eq!(harness["command"], serde_json::json!(["node", "--test"]));
    assert_eq!(harness["executable"], "node");
    assert_eq!(harness["args"], serde_json::json!(["--test"]));
}

#[test]
fn doctor_reports_malformed_browser_harness_override_in_json() {
    let output = Command::new(kali_bin())
        .arg("--output")
        .arg("json")
        .arg("doctor")
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "   ")
        .output()
        .expect("run kali doctor");

    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["command"], "doctor");
    assert_eq!(json["success"], false);
    assert_eq!(json["errors"][0]["code"], "E5508");
    assert!(json["errors"][0]["message"]
        .as_str()
        .expect("error message")
        .contains("KALI_BROWSER_BUNDLE_HARNESS_COMMAND"));
}
