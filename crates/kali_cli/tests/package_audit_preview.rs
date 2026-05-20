use std::{path::PathBuf, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> PathBuf {
    std::env::var_os("CARGO_BIN_EXE_kali")
        .map(PathBuf::from)
        .expect("kali binary path")
}

fn run_kali<I, S>(args: I) -> std::process::Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let dir = tempdir().expect("tempdir");
    Command::new(kali_bin())
        .current_dir(dir.path())
        .args(args)
        .output()
        .expect("run kali")
}

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
}

#[test]
fn package_audit_preview_short_circuits_before_malformed_target_validation_in_text_mode() {
    let output = run_kali(["package-audit", "--preview", "npm:lodash"]);

    assert_eq!(output.status.code(), Some(5));
    assert!(output.stdout.is_empty(), "stdout: {:?}", output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("legacy `--preview` compatibility shim is not part of the schema-v1 package-audit command shape"),
        "stderr: {stderr}"
    );
    assert!(
        !stderr.contains("npm:lodash"),
        "preview should short-circuit before malformed-target validation: {stderr}"
    );
}

#[test]
fn package_audit_preview_short_circuits_before_malformed_target_validation_in_json_mode() {
    let output = run_kali([
        "--output",
        "json",
        "package-audit",
        "--preview",
        "npm:lodash",
    ]);

    assert_eq!(output.status.code(), Some(5));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "package-audit");
    assert_eq!(json["success"], false);
    assert!(json["warnings"]
        .as_array()
        .expect("warnings array")
        .is_empty());
    assert!(json["stdout"].is_null());
    assert!(json["stderr"].is_null());
    let errors = json["errors"].as_array().expect("errors array");
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0]["code"], "E5508");
    assert_eq!(
        errors[0]["message"],
        "legacy `--preview` compatibility shim is not part of the schema-v1 package-audit command shape"
    );
    assert_eq!(errors[0]["context"]["flag"], "--preview");
    assert_eq!(errors[0]["context"]["requestedValue"], "true");
    assert_eq!(errors[0]["context"]["effectiveValue"], "true");
}
