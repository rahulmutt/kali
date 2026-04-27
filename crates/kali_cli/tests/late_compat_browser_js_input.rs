use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn late_process_control_source() -> &'static str {
    "globalThis.Deno.cwd; globalThis[\"Deno\"][\"cwd\"]; Deno[\"cwd\"]; globalThis.Deno[\"cwd\"]; Deno.chdir; globalThis.Deno.chdir; globalThis[\"Deno\"][\"chdir\"]; Deno[\"chdir\"]; globalThis.Deno[\"chdir\"]; globalThis.Deno.exit; globalThis[\"Deno\"][\"exit\"]; Deno[\"exit\"]; globalThis.Deno[\"exit\"]; process.pid; globalThis.process.pid; globalThis[\"process\"][\"pid\"]; globalThis.process.cwd; process.chdir; globalThis.process.chdir; process.exit; globalThis[\"process\"][\"cwd\"]; globalThis[\"process\"][\"chdir\"]; globalThis[\"process\"][\"exit\"];"
}

fn late_env_materialization_source() -> &'static str {
    "Deno.env.toObject; globalThis.Deno.env.toObject; Deno.env[\"toObject\"]; Deno[\"env\"][\"toObject\"]; globalThis.Deno[\"env\"][\"toObject\"]; globalThis[\"Deno\"][\"env\"][\"toObject\"]; globalThis.Deno[\"env\"][\"toObject\"];"
}

fn late_object_model_source() -> &'static str {
    "Intl; globalThis.Intl; globalThis[\"Intl\"]; globalThis.Intl.NumberFormat; globalThis.Intl.DateTimeFormat; globalThis[\"Intl\"][\"NumberFormat\"]; globalThis[\"Intl\"][\"DateTimeFormat\"]; Proxy; globalThis.Proxy; globalThis[\"Proxy\"]; Proxy.revocable({}, {}); globalThis.Proxy.revocable({}, {}); globalThis[\"Proxy\"][\"revocable\"]({}, {}); Object.hasOwn({}, \"a\"); globalThis.Object.hasOwn({}, \"a\"); globalThis[\"Object\"][\"hasOwn\"]({}, \"a\"); Object.prototype.hasOwnProperty.call({}, \"a\"); globalThis.Object.prototype.hasOwnProperty.call({}, \"a\"); globalThis[\"Object\"][\"prototype\"][\"hasOwnProperty\"][\"call\"]({}, \"a\"); new WeakMap(); globalThis.WeakMap; globalThis[\"WeakMap\"](); new WeakSet(); globalThis.WeakSet; globalThis[\"WeakSet\"](); globalThis.WeakRef; globalThis[\"WeakRef\"]; new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry; globalThis[\"FinalizationRegistry\"](() => {});"
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

fn late_threaded_runtime_source() -> &'static str {
    "globalThis.SharedArrayBuffer; globalThis.Atomics;"
}

fn assert_browser_late_process_control_rejection(stderr: &str) {
    assert!(stderr.contains("E3100"), "stderr: {stderr}");
    assert!(
        stderr.contains("undefined identifier 'process'"),
        "stderr: {stderr}"
    );
    for expected in [
        "globalThis.Deno.cwd",
        "Deno.chdir",
        "globalThis.Deno.chdir",
        "globalThis.Deno.exit",
        "process.pid",
        "globalThis.process.pid",
        "globalThis.process.cwd",
        "process.chdir",
        "globalThis.process.chdir",
        "process.exit",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

fn assert_browser_late_process_control_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors
            .iter()
            .all(|error| matches!(error["code"].as_str(), Some("E3100") | Some("E5506"))),
        "unexpected errors: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error["code"] == "E3100"),
        "expected at least one E3100 error: {errors:?}"
    );
    for expected in [
        "globalThis.Deno.cwd",
        "Deno.chdir",
        "globalThis.Deno.chdir",
        "globalThis.Deno.exit",
        "process.pid",
        "globalThis.process.pid",
        "globalThis.process.cwd",
        "process.chdir",
        "globalThis.process.chdir",
        "process.exit",
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

fn assert_browser_late_env_materialization_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    for expected in [
        "Deno.env.toObject",
        "globalThis.Deno.env.toObject",
        "Deno[\"env\"][\"toObject\"]",
        "globalThis[\"Deno\"][\"env\"][\"toObject\"]",
        "environment snapshot materialization API",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

fn assert_browser_late_env_materialization_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().all(|error| error["code"] == "E5506"),
        "unexpected errors: {errors:?}"
    );
    for expected in [
        "Deno.env.toObject",
        "globalThis.Deno.env.toObject",
        "Deno[\"env\"][\"toObject\"]",
        "globalThis[\"Deno\"][\"env\"][\"toObject\"]",
        "environment snapshot materialization API",
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

fn assert_browser_late_object_model_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    for expected in [
        "Intl",
        "globalThis.Intl",
        "globalThis.Intl.NumberFormat",
        "globalThis.Intl.DateTimeFormat",
        "Proxy",
        "globalThis.Proxy",
        "Proxy.revocable",
        "globalThis.Proxy.revocable",
        r#"globalThis["Proxy"]["revocable"]"#,
        "Object.hasOwn",
        "globalThis.Object.hasOwn",
        r#"globalThis["Object"]["hasOwn"]"#,
        "Object.prototype.hasOwnProperty.call",
        "globalThis.Object.prototype.hasOwnProperty.call",
        r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]"#,
        "WeakMap",
        r#"globalThis["WeakMap"]"#,
        "WeakSet",
        r#"globalThis["WeakSet"]"#,
        "WeakRef",
        r#"globalThis["WeakRef"]"#,
        "FinalizationRegistry",
        r#"globalThis["FinalizationRegistry"]"#,
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

fn assert_browser_late_object_model_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().all(|error| error["code"] == "E5506"),
        "unexpected errors: {errors:?}"
    );
    let messages = errors
        .iter()
        .map(|error| error["message"].as_str().expect("error message"))
        .collect::<Vec<_>>();
    for expected in [
        "Intl",
        "globalThis.Intl",
        "globalThis.Intl.NumberFormat",
        "globalThis.Intl.DateTimeFormat",
        "Proxy",
        "globalThis.Proxy",
        "Proxy.revocable",
        "globalThis.Proxy.revocable",
        r#"globalThis["Proxy"]["revocable"]"#,
        "Object.hasOwn",
        "globalThis.Object.hasOwn",
        r#"globalThis["Object"]["hasOwn"]"#,
        "Object.prototype.hasOwnProperty.call",
        "globalThis.Object.prototype.hasOwnProperty.call",
        r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]"#,
        "WeakMap",
        r#"globalThis["WeakMap"]"#,
        "WeakSet",
        r#"globalThis["WeakSet"]"#,
        "WeakRef",
        r#"globalThis["WeakRef"]"#,
        "FinalizationRegistry",
        r#"globalThis["FinalizationRegistry"]"#,
    ] {
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "missing {expected} in {messages:?}"
        );
    }
}

fn assert_browser_late_threaded_runtime_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    for expected in [
        "SharedArrayBuffer",
        "globalThis.SharedArrayBuffer",
        "Atomics",
        "globalThis.Atomics",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

fn assert_browser_late_threaded_runtime_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().all(|error| error["code"] == "E5506"),
        "unexpected errors: {errors:?}"
    );
    let messages = errors
        .iter()
        .map(|error| error["message"].as_str().expect("error message"))
        .collect::<Vec<_>>();
    for expected in [
        "SharedArrayBuffer",
        "globalThis.SharedArrayBuffer",
        "Atomics",
        "globalThis.Atomics",
    ] {
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "missing {expected} in {messages:?}"
        );
    }
}

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
}

#[test]
fn browser_late_object_model_source_includes_bracketed_intl_forms() {
    let source = late_object_model_source();
    assert!(source.contains(r#"globalThis["Intl"]"#), "source: {source}");
    assert!(
        source.contains(r#"globalThis["Intl"]["NumberFormat"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Intl"]["DateTimeFormat"]"#),
        "source: {source}"
    );
}

#[test]
fn browser_late_object_model_source_includes_bracketed_proxy_and_finalization_forms() {
    let source = late_object_model_source();
    for expected in [
        r#"globalThis["Proxy"]"#,
        r#"Proxy.revocable"#,
        r#"globalThis.Proxy.revocable"#,
        r#"globalThis["Proxy"]["revocable"]"#,
        r#"globalThis["Object"]["hasOwn"]"#,
        r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]"#,
        r#"globalThis["WeakMap"]"#,
        r#"globalThis["WeakSet"]"#,
        "globalThis.WeakRef",
        r#"globalThis["WeakRef"]"#,
        r#"globalThis["FinalizationRegistry"]"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn browser_late_process_control_source_includes_bracketed_forms() {
    let source = late_process_control_source();
    for expected in [
        r#"globalThis["Deno"]["cwd"]"#,
        r#"Deno["cwd"]"#,
        r#"globalThis.Deno["cwd"]"#,
        r#"globalThis["Deno"]["chdir"]"#,
        r#"Deno["chdir"]"#,
        r#"globalThis.Deno["chdir"]"#,
        r#"globalThis["Deno"]["exit"]"#,
        r#"Deno["exit"]"#,
        r#"globalThis.Deno["exit"]"#,
        r#"globalThis["process"]["pid"]"#,
        r#"globalThis["process"]["cwd"]"#,
        r#"globalThis["process"]["chdir"]"#,
        r#"globalThis["process"]["exit"]"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn browser_late_env_materialization_source_includes_bracketed_forms() {
    let source = late_env_materialization_source();
    for expected in [
        r#"Deno.env["toObject"]"#,
        r#"Deno["env"]["toObject"]"#,
        r#"globalThis.Deno["env"]["toObject"]"#,
        r#"globalThis["Deno"]["env"]["toObject"]"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn check_rejects_late_process_control_members_in_browser_api_surface_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_process_control_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_process_control_rejection(&stderr);
}

#[test]
fn build_rejects_late_process_control_members_in_browser_bundle_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_process_control_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_process_control_rejection(&stderr);
}

#[test]
fn check_rejects_late_process_control_members_in_browser_api_surface_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_process_control_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_process_control_rejection_json(errors);
}

#[test]
fn build_rejects_late_process_control_members_in_browser_bundle_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_process_control_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_process_control_rejection_json(errors);
}

#[test]
fn check_rejects_late_env_materialization_members_in_browser_api_surface_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_env_materialization_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_env_materialization_rejection(&stderr);
}

#[test]
fn check_rejects_late_env_materialization_members_in_browser_api_surface_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_env_materialization_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_env_materialization_rejection_json(errors);
}

#[test]
fn build_rejects_late_env_materialization_members_in_browser_bundle_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_env_materialization_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_env_materialization_rejection(&stderr);
}

#[test]
fn build_rejects_late_env_materialization_members_in_browser_bundle_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_env_materialization_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_env_materialization_rejection_json(errors);
}

#[test]
fn run_rejects_late_process_control_members_in_browser_api_surface_js_input_with_browser_harness() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_process_control_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_process_control_rejection(&stderr);
}

#[test]
fn run_rejects_late_process_control_members_in_browser_api_surface_js_input_with_browser_harness_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_process_control_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
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
    assert_browser_late_process_control_rejection_json(errors);
}

#[test]
fn test_rejects_late_process_control_members_in_browser_api_surface_js_input_with_browser_harness()
{
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_process_control_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_process_control_rejection(&stderr);
}

#[test]
fn test_rejects_late_process_control_members_in_browser_api_surface_js_input_with_browser_harness_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_process_control_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
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
    assert_browser_late_process_control_rejection_json(errors);
}

#[test]
fn check_rejects_late_object_model_members_in_browser_api_surface_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_object_model_rejection(&stderr);
}

#[test]
fn build_rejects_late_object_model_members_in_browser_bundle_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_object_model_rejection(&stderr);
}

#[test]
fn check_rejects_late_object_model_members_in_browser_api_surface_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty());
    assert!(errors.iter().all(|error| error["code"] == "E5506"));
    let messages = errors
        .iter()
        .map(|error| error["message"].as_str().expect("error message"))
        .collect::<Vec<_>>();
    for expected in [
        "Intl",
        "globalThis.Intl",
        "globalThis.Intl.NumberFormat",
        "Proxy",
        "globalThis.Proxy",
        "WeakMap",
        "WeakSet",
        "FinalizationRegistry",
    ] {
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "missing {expected} in {messages:?}"
        );
    }
}

#[test]
fn build_rejects_late_object_model_members_in_browser_bundle_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert!(!errors.is_empty());
    assert!(errors.iter().all(|error| error["code"] == "E5506"));
    let messages = errors
        .iter()
        .map(|error| error["message"].as_str().expect("error message"))
        .collect::<Vec<_>>();
    for expected in [
        "Intl",
        "globalThis.Intl",
        "globalThis.Intl.NumberFormat",
        "Proxy",
        "globalThis.Proxy",
        "WeakMap",
        "WeakSet",
        "FinalizationRegistry",
    ] {
        assert!(
            messages.iter().any(|message| message.contains(expected)),
            "missing {expected} in {messages:?}"
        );
    }
}

#[test]
fn run_rejects_late_object_model_members_in_browser_api_surface_js_input_with_browser_harness() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_object_model_rejection(&stderr);
}

#[test]
fn run_rejects_late_object_model_members_in_browser_api_surface_js_input_with_browser_harness_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
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
    assert_browser_late_object_model_rejection_json(errors);
}

#[test]
fn test_rejects_late_object_model_members_in_browser_api_surface_js_input_with_browser_harness() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_object_model_rejection(&stderr);
}

#[test]
fn test_rejects_late_object_model_members_in_browser_api_surface_js_input_with_browser_harness_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
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
    assert_browser_late_object_model_rejection_json(errors);
}

#[test]
fn run_rejects_late_object_model_members_in_inherited_browser_api_surface_js_input_with_browser_harness(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");
    write_browser_api_surface_manifest(&dir);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_object_model_rejection(&stderr);
}

#[test]
fn run_rejects_late_object_model_members_in_inherited_browser_api_surface_js_input_with_browser_harness_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");
    write_browser_api_surface_manifest(&dir);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("run")
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
    assert_browser_late_object_model_rejection_json(errors);
}

#[test]
fn test_rejects_late_object_model_members_in_inherited_browser_api_surface_js_input_with_browser_harness(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");
    write_browser_api_surface_manifest(&dir);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_object_model_rejection(&stderr);
}

#[test]
fn run_rejects_threaded_runtime_globals_in_browser_api_surface_js_input_with_browser_harness_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_threaded_runtime_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
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
    assert_browser_late_threaded_runtime_rejection_json(errors);
}

#[test]
fn test_rejects_late_object_model_members_in_inherited_browser_api_surface_js_input_with_browser_harness_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_object_model_source()).expect("write source");
    write_browser_api_surface_manifest(&dir);

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
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
    assert_browser_late_object_model_rejection_json(errors);
}

#[test]
fn test_rejects_threaded_runtime_globals_in_browser_api_surface_js_input_with_browser_harness() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_threaded_runtime_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_threaded_runtime_rejection(&stderr);
}

#[test]
fn test_rejects_threaded_runtime_globals_in_browser_api_surface_js_input_with_browser_harness_in_json(
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_threaded_runtime_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
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
    assert_browser_late_threaded_runtime_rejection_json(errors);
}

fn assert_browser_late_threaded_runtime_rejection_for_command(
    command: &str,
    command_args: &[&str],
    with_browser_harness: bool,
    with_explicit_browser_api_surface: bool,
    with_browser_api_surface_manifest: bool,
    source_name: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(source_name);
    fs::write(&source_path, late_threaded_runtime_source()).expect("write source");
    if with_browser_api_surface_manifest {
        write_browser_api_surface_manifest(&dir);
    }

    for json_output in [false, true] {
        let mut output = Command::new(kali_bin());
        output.current_dir(dir.path());
        if with_browser_harness {
            output.env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node");
        }
        if json_output {
            output.arg("--output").arg("json");
        }
        output.arg(command);
        for arg in command_args {
            output.arg(arg);
        }
        if with_explicit_browser_api_surface {
            output.arg("--api").arg("browser");
        }
        output.arg(&source_path);

        let output = output.output().expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(1));
        if json_output {
            let json = parse_json_stdout(&output);
            assert_eq!(json["schemaVersion"], 1);
            assert_eq!(json["command"], command);
            assert_eq!(json["success"], false);
            let errors = json["errors"].as_array().expect("errors array");
            assert_browser_late_threaded_runtime_rejection_json(errors);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert_browser_late_threaded_runtime_rejection(&stderr);
        }
    }
}

#[test]
fn check_rejects_threaded_runtime_globals_in_browser_api_surface_js_input() {
    assert_browser_late_threaded_runtime_rejection_for_command(
        "check",
        &[],
        false,
        true,
        false,
        "main.js",
    );
}

#[test]
fn build_rejects_threaded_runtime_globals_in_browser_bundle_js_input() {
    assert_browser_late_threaded_runtime_rejection_for_command(
        "build",
        &["--bundle"],
        false,
        true,
        false,
        "main.js",
    );
}

#[test]
fn check_rejects_threaded_runtime_globals_in_inherited_browser_api_surface_js_input() {
    assert_browser_late_threaded_runtime_rejection_for_command(
        "check",
        &[],
        false,
        false,
        true,
        "main.js",
    );
}

#[test]
fn build_rejects_threaded_runtime_globals_in_inherited_browser_api_surface_js_input() {
    assert_browser_late_threaded_runtime_rejection_for_command(
        "build",
        &["--bundle"],
        false,
        false,
        true,
        "main.js",
    );
}

fn assert_browser_late_nullish_coalescing_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("nullish coalescing"), "stderr: {stderr}");
}

fn assert_browser_late_nullish_coalescing_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().all(|error| error["code"] == "E5506"),
        "unexpected errors: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error["message"]
            .as_str()
            .expect("error message")
            .contains("nullish coalescing")),
        "missing nullish coalescing in {errors:?}"
    );
}

#[test]
fn check_rejects_nullish_coalescing_in_browser_api_surface_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = null ?? 1;\nconsole.log(value);\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_nullish_coalescing_rejection(&stderr);
}

#[test]
fn build_rejects_nullish_coalescing_in_browser_bundle_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = null ?? 1;\nconsole.log(value);\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_browser_late_nullish_coalescing_rejection(&stderr);
}

#[test]
fn check_rejects_nullish_coalescing_in_browser_api_surface_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = null ?? 1;\nconsole.log(value);\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "check");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_nullish_coalescing_rejection_json(errors);
}

#[test]
fn build_rejects_nullish_coalescing_in_browser_bundle_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(
        &source_path,
        "const value = null ?? 1;\nconsole.log(value);\n",
    )
    .expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json = parse_json_stdout(&output);
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_browser_late_nullish_coalescing_rejection_json(errors);
}

fn late_eval_compatibility_source() -> &'static str {
    "eval('1 + 2'); new Function('return 3')();"
}

fn assert_browser_late_eval_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(
        stderr.contains("compatibility feature 'eval'"),
        "stderr: {stderr}"
    );
}

fn assert_browser_late_eval_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors.iter().all(|error| error["code"] == "E5506"),
        "unexpected errors: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error["message"]
            .as_str()
            .expect("error message")
            .contains("compatibility feature 'eval'")),
        "missing eval compatibility gate in {errors:?}"
    );
}

fn assert_browser_late_eval_rejection_for_command(
    command: &str,
    command_args: &[&str],
    with_browser_harness: bool,
    with_browser_api_surface_manifest: bool,
    source_name: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(source_name);
    fs::write(&source_path, late_eval_compatibility_source()).expect("write source");
    if with_browser_api_surface_manifest {
        write_browser_api_surface_manifest(&dir);
    }

    for json_output in [false, true] {
        let mut output = Command::new(kali_bin());
        output.current_dir(dir.path());
        if with_browser_harness {
            output.env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node");
        }
        if json_output {
            output.arg("--output").arg("json");
        }
        output.arg(command);
        for arg in command_args {
            output.arg(arg);
        }
        output.arg("--api").arg("browser");
        output.arg(&source_path);

        let output = output.output().expect("run kali");

        assert!(!output.status.success());
        assert_eq!(output.status.code(), Some(1));
        if json_output {
            let json = parse_json_stdout(&output);
            assert_eq!(json["schemaVersion"], 1);
            assert_eq!(json["command"], command);
            assert_eq!(json["success"], false);
            let errors = json["errors"].as_array().expect("errors array");
            assert_browser_late_eval_rejection_json(errors);
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert_browser_late_eval_rejection(&stderr);
        }
    }
}

#[test]
fn check_rejects_eval_and_function_constructor_in_browser_api_surface_js_input() {
    assert_browser_late_eval_rejection_for_command("check", &[], false, false, "main.js");
}

#[test]
fn build_rejects_eval_and_function_constructor_in_browser_bundle_js_input() {
    assert_browser_late_eval_rejection_for_command("build", &["--bundle"], false, false, "main.js");
}

#[test]
fn run_rejects_eval_and_function_constructor_in_browser_api_surface_js_input_with_browser_harness()
{
    assert_browser_late_eval_rejection_for_command("run", &[], true, false, "main.js");
}

#[test]
fn test_rejects_eval_and_function_constructor_in_browser_api_surface_js_input_with_browser_harness()
{
    assert_browser_late_eval_rejection_for_command("test", &[], true, false, "smoke.test.js");
}

#[test]
fn check_rejects_eval_and_function_constructor_in_inherited_browser_api_surface_js_input() {
    assert_browser_late_eval_rejection_for_command("check", &[], false, true, "main.js");
}

#[test]
fn build_rejects_eval_and_function_constructor_in_inherited_browser_api_surface_js_input() {
    assert_browser_late_eval_rejection_for_command("build", &["--bundle"], false, true, "main.js");
}

#[test]
fn run_rejects_eval_and_function_constructor_in_inherited_browser_api_surface_js_input_with_browser_harness(
) {
    assert_browser_late_eval_rejection_for_command("run", &[], true, true, "main.js");
}

#[test]
fn test_rejects_eval_and_function_constructor_in_inherited_browser_api_surface_js_input_with_browser_harness(
) {
    assert_browser_late_eval_rejection_for_command("test", &[], true, true, "smoke.test.js");
}
