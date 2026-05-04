use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
}

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn late_browser_tsx_compatibility_source() -> &'static str {
    "Intl; Deno.permissions.request(); Deno.permissions.revoke(); Deno.env.toObject(); Deno.env.set('KALI_ENV_SET_SMOKE', 'hello'); Deno.env.delete('KALI_ENV_DELETE_SMOKE'); process.pid; process.cwd; process.chdir; process.exit; Proxy.revocable({}, {}); Object.hasOwn({}, 'a'); Object.prototype.hasOwnProperty.call({}, 'a'); new WeakMap(); new WeakSet(); new WeakRef(); new FinalizationRegistry(() => {}); globalThis.SharedArrayBuffer; globalThis.Atomics;"
}

fn assert_browser_late_tsx_compatibility_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("E3100"), "stderr: {stderr}");
    assert!(
        stderr.contains("undefined identifier 'process'"),
        "stderr: {stderr}"
    );
    for expected in [
        "Intl",
        "Deno.permissions.request",
        "Deno.permissions.revoke",
        "Deno.env.toObject",
        "Deno.env.set",
        "Deno.env.delete",
        "process.pid",
        "process.cwd",
        "process.chdir",
        "process.exit",
        "Proxy.revocable",
        "WeakMap",
        "WeakRef",
        "FinalizationRegistry",
        "SharedArrayBuffer",
        "Atomics",
        "object-aggregate lowering",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

fn assert_browser_late_tsx_compatibility_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors
            .iter()
            .all(|error| matches!(error["code"].as_str(), Some("E5506") | Some("E3100"))),
        "unexpected errors: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error["code"] == "E5506"),
        "expected at least one E5506 error: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error["code"] == "E3100"),
        "expected at least one E3100 error: {errors:?}"
    );
    for expected in [
        "Intl",
        "Deno.permissions.request",
        "Deno.permissions.revoke",
        "Deno.env.toObject",
        "Deno.env.set",
        "Deno.env.delete",
        "process.pid",
        "process.cwd",
        "process.chdir",
        "process.exit",
        "Proxy.revocable",
        "WeakMap",
        "WeakRef",
        "FinalizationRegistry",
        "SharedArrayBuffer",
        "Atomics",
        "object-aggregate lowering",
        "undefined identifier 'process'",
    ] {
        assert!(
            errors.iter().any(|error| error["message"]
                .as_str()
                .expect("error message")
                .contains(expected)),
            "missing {expected} in {errors:?}"
        );
    }
}

#[test]
fn run_rejects_late_browser_compatibility_forms_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(&source_path, late_browser_tsx_compatibility_source()).expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_tsx_compatibility_rejection(&stderr);
}

#[test]
fn run_rejects_late_browser_compatibility_forms_in_tsx_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.tsx");
    fs::write(&source_path, late_browser_tsx_compatibility_source()).expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "run");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_tsx_compatibility_rejection_json(errors);
}

#[test]
fn test_rejects_late_browser_compatibility_forms_in_tsx_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.tsx");
    fs::write(
        &source_path,
        format!(
            "Kali.test('late browser tsx compatibility', () => {{ {} }});\n",
            late_browser_tsx_compatibility_source()
        ),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_tsx_compatibility_rejection(&stderr);
}

#[test]
fn test_rejects_late_browser_compatibility_forms_in_tsx_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.tsx");
    fs::write(
        &source_path,
        format!(
            "Kali.test('late browser tsx compatibility', () => {{ {} }});\n",
            late_browser_tsx_compatibility_source()
        ),
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_tsx_compatibility_rejection_json(errors);
}
