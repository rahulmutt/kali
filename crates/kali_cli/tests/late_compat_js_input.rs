use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
}

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn late_js_compatibility_source() -> &'static str {
    "Intl; globalThis.Intl; globalThis[\"Intl\"]; globalThis.Intl.NumberFormat; globalThis.Intl.DateTimeFormat; globalThis.Intl.PluralRules; globalThis.Intl.RelativeTimeFormat; globalThis.Intl.Collator; globalThis.Intl.DisplayNames; globalThis.Intl.Locale; globalThis[\"Intl\"][\"NumberFormat\"]; globalThis[\"Intl\"][\"DateTimeFormat\"]; globalThis[\"Intl\"][\"PluralRules\"]; globalThis[\"Intl\"][\"RelativeTimeFormat\"]; globalThis[\"Intl\"][\"Collator\"]; globalThis[\"Intl\"][\"DisplayNames\"]; globalThis[\"Intl\"][\"Locale\"]; Intl.NumberFormat; Intl.DateTimeFormat; Intl.PluralRules; Intl.RelativeTimeFormat; Intl.Collator; Intl.DisplayNames; Intl.Locale; globalThis[\"Deno\"][\"cwd\"]; Deno[\"cwd\"]; globalThis.Deno[\"cwd\"]; Deno.chdir; globalThis.Deno.chdir; globalThis[\"Deno\"][\"chdir\"]; Deno[\"chdir\"]; globalThis.Deno[\"chdir\"]; globalThis.Deno.exit; globalThis[\"Deno\"][\"exit\"]; Deno[\"exit\"]; globalThis.Deno[\"exit\"]; Deno.permissions[\"request\"](); Deno.permissions[\"revoke\"](); globalThis.Deno.permissions[\"request\"](); globalThis.Deno.permissions[\"revoke\"](); globalThis[\"Deno\"][\"permissions\"][\"request\"](); globalThis[\"Deno\"][\"permissions\"][\"revoke\"](); Deno.env.toObject; globalThis.Deno.env.toObject; Deno.env[\"toObject\"]; Deno[\"env\"][\"toObject\"]; globalThis.Deno[\"env\"][\"toObject\"]; globalThis[\"Deno\"][\"env\"][\"toObject\"]; globalThis.Deno[\"env\"][\"toObject\"]; process.pid; globalThis.process.pid; globalThis[\"process\"][\"pid\"]; process[\"pid\"]; globalThis.process[\"pid\"]; globalThis.process.cwd; process.chdir; globalThis.process.chdir; process[\"cwd\"]; globalThis.process[\"cwd\"]; process[\"chdir\"]; globalThis.process[\"chdir\"]; process.exit; globalThis[\"process\"][\"cwd\"]; globalThis[\"process\"][\"chdir\"]; globalThis[\"process\"][\"exit\"]; process[\"exit\"]; globalThis.process[\"exit\"]; Proxy; globalThis.Proxy; globalThis[\"Proxy\"]; Proxy.revocable({}, {}); globalThis.Proxy.revocable({}, {}); globalThis[\"Proxy\"][\"revocable\"]({}, {}); Object.hasOwn({}, \"a\"); globalThis.Object.hasOwn({}, \"a\"); globalThis[\"Object\"][\"hasOwn\"]({}, \"a\"); Object.prototype.hasOwnProperty.call({}, \"a\"); globalThis.Object.prototype.hasOwnProperty.call({}, \"a\"); globalThis[\"Object\"][\"prototype\"][\"hasOwnProperty\"][\"call\"]({}, \"a\"); new WeakMap(); globalThis.WeakMap; globalThis[\"WeakMap\"](); new WeakSet(); globalThis.WeakSet; globalThis[\"WeakSet\"](); globalThis.WeakRef; globalThis[\"WeakRef\"]; new FinalizationRegistry(() => {}); globalThis.FinalizationRegistry; globalThis[\"FinalizationRegistry\"](() => {}); globalThis.SharedArrayBuffer; globalThis[\"SharedArrayBuffer\"]; globalThis.Atomics; globalThis[\"Atomics\"]; null ?? 1;"
}

fn assert_late_js_compatibility_rejection(stderr: &str) {
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
    assert!(stderr.contains("E3100"), "stderr: {stderr}");
    assert!(
        stderr.contains("undefined identifier 'process'"),
        "stderr: {stderr}"
    );
    assert!(
        stderr.matches("Deno.permissions.request").count() >= 2,
        "stderr: {stderr}"
    );
    assert!(
        stderr.matches("Deno.permissions.revoke").count() >= 2,
        "stderr: {stderr}"
    );
    for expected in [
        "Intl",
        "globalThis.Intl",
        "globalThis.Intl.NumberFormat",
        "globalThis.Intl.DateTimeFormat",
        "globalThis.Intl.RelativeTimeFormat",
        "globalThis.Intl.PluralRules",
        "globalThis.Intl.Collator",
        "globalThis.Intl.DisplayNames",
        "globalThis.Intl.Locale",
        "Intl.NumberFormat",
        "Intl.DateTimeFormat",
        "Intl.RelativeTimeFormat",
        "Intl.PluralRules",
        "Intl.Collator",
        "Intl.DisplayNames",
        "Intl.Locale",
        "globalThis[\"Intl\"][\"DisplayNames\"]",
        "globalThis[\"Intl\"][\"Locale\"]",
        "Deno.env.toObject",
        "globalThis.Deno.env.toObject",
        "Deno[\"env\"][\"toObject\"]",
        "globalThis[\"Deno\"][\"env\"][\"toObject\"]",
        "globalThis.Deno.cwd",
        "Deno.chdir",
        "globalThis.Deno.chdir",
        "Deno.exit",
        "globalThis.Deno.exit",
        "Deno.permissions.request",
        "Deno.permissions.revoke",
        "process.pid",
        "globalThis.process.pid",
        "globalThis.process.cwd",
        "process.chdir",
        "globalThis.process.chdir",
        "process.exit",
        "Proxy",
        "globalThis.Proxy",
        r#"globalThis["Proxy"]["revocable"]"#,
        "Object.hasOwn",
        r#"globalThis["Object"]["hasOwn"]"#,
        "Object.prototype.hasOwnProperty.call",
        r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]"#,
        "WeakMap",
        "globalThis.WeakMap",
        r#"globalThis["WeakMap"]"#,
        "WeakSet",
        "globalThis.WeakSet",
        r#"globalThis["WeakSet"]"#,
        "WeakRef",
        "globalThis.WeakRef",
        r#"globalThis["WeakRef"]"#,
        "FinalizationRegistry",
        "globalThis.FinalizationRegistry",
        r#"globalThis["FinalizationRegistry"]"#,
        "SharedArrayBuffer",
        "Atomics",
        "nullish coalescing",
    ] {
        assert!(
            stderr.contains(expected),
            "missing {expected} in stderr: {stderr}"
        );
    }
}

fn assert_late_js_compatibility_rejection_json(errors: &[Value]) {
    assert!(!errors.is_empty(), "errors array should not be empty");
    assert!(
        errors
            .iter()
            .all(|error| matches!(error["code"].as_str(), Some("E5506") | Some("E3100"))),
        "unexpected errors: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error["code"] == "E3100"),
        "expected at least one E3100 error: {errors:?}"
    );
    assert!(
        errors
            .iter()
            .filter(|error| error["message"]
                .as_str()
                .expect("error message")
                .contains("Deno.permissions.request"))
            .count()
            >= 2,
        "missing bracketed request coverage in {errors:?}"
    );
    assert!(
        errors
            .iter()
            .filter(|error| error["message"]
                .as_str()
                .expect("error message")
                .contains("Deno.permissions.revoke"))
            .count()
            >= 2,
        "missing bracketed revoke coverage in {errors:?}"
    );
    for expected in [
        "Intl",
        "globalThis.Intl",
        "globalThis.Intl.NumberFormat",
        "globalThis.Intl.DateTimeFormat",
        "globalThis.Intl.RelativeTimeFormat",
        "globalThis.Intl.PluralRules",
        "globalThis.Intl.Collator",
        "globalThis.Intl.DisplayNames",
        "globalThis.Intl.Locale",
        "Intl.NumberFormat",
        "Intl.DateTimeFormat",
        "Intl.RelativeTimeFormat",
        "Intl.PluralRules",
        "Intl.Collator",
        "Intl.DisplayNames",
        "Intl.Locale",
        "globalThis[\"Intl\"][\"DisplayNames\"]",
        "globalThis[\"Intl\"][\"Locale\"]",
        "Deno.env.toObject",
        "globalThis.Deno.env.toObject",
        "Deno[\"env\"][\"toObject\"]",
        "globalThis[\"Deno\"][\"env\"][\"toObject\"]",
        "globalThis.Deno.cwd",
        "Deno.chdir",
        "globalThis.Deno.chdir",
        "Deno.exit",
        "globalThis.Deno.exit",
        "process.pid",
        "globalThis.process.pid",
        "globalThis.process.cwd",
        "process.chdir",
        "globalThis.process.chdir",
        "process.exit",
        "Proxy",
        "globalThis.Proxy",
        r#"globalThis["Proxy"]["revocable"]"#,
        "Object.hasOwn",
        r#"globalThis["Object"]["hasOwn"]"#,
        "Object.prototype.hasOwnProperty.call",
        r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]"#,
        "WeakMap",
        "globalThis.WeakMap",
        r#"globalThis["WeakMap"]"#,
        "WeakSet",
        "globalThis.WeakSet",
        r#"globalThis["WeakSet"]"#,
        "WeakRef",
        "globalThis.WeakRef",
        r#"globalThis["WeakRef"]"#,
        "FinalizationRegistry",
        "globalThis.FinalizationRegistry",
        r#"globalThis["FinalizationRegistry"]"#,
        "SharedArrayBuffer",
        "Atomics",
        "nullish coalescing",
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
fn late_js_compatibility_source_includes_bracketed_intl_forms() {
    let source = late_js_compatibility_source();
    assert!(source.contains(r#"globalThis["Intl"]"#), "source: {source}");
    assert!(
        source.contains(r#"globalThis["Intl"]["NumberFormat"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Intl"]["DateTimeFormat"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Intl"]["RelativeTimeFormat"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Intl"]["PluralRules"]"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"globalThis["Intl"]["Collator"]"#),
        "source: {source}"
    );
    assert!(source.contains(r#"globalThis["Intl"]["DisplayNames"]"#),);
    assert!(
        source.contains(r#"globalThis["Intl"]["Locale"]"#),
        "source: {source}"
    );
}

#[test]
fn late_js_compatibility_source_includes_bracketed_process_object_and_env_forms() {
    let source = late_js_compatibility_source();
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
        r#"Deno["env"]["toObject"]"#,
        r#"globalThis.Deno["env"]["toObject"]"#,
        r#"globalThis["Deno"]["env"]["toObject"]"#,
        r#"globalThis["process"]["pid"]"#,
        r#"process["pid"]"#,
        r#"globalThis.process["pid"]"#,
        r#"globalThis["process"]["cwd"]"#,
        r#"process["cwd"]"#,
        r#"globalThis.process["cwd"]"#,
        r#"globalThis["process"]["chdir"]"#,
        r#"process["chdir"]"#,
        r#"globalThis.process["chdir"]"#,
        r#"globalThis["process"]["exit"]"#,
        r#"process["exit"]"#,
        r#"globalThis.process["exit"]"#,
        r#"globalThis["Proxy"]"#,
        r#"Proxy.revocable"#,
        r#"globalThis.Proxy.revocable"#,
        r#"globalThis["Proxy"]["revocable"]"#,
        r#"globalThis["Object"]["hasOwn"]"#,
        r#"globalThis["Object"]["prototype"]["hasOwnProperty"]["call"]"#,
        "globalThis.WeakMap",
        r#"globalThis["WeakMap"]"#,
        "globalThis.WeakSet",
        r#"globalThis["WeakSet"]"#,
        "globalThis.WeakRef",
        r#"globalThis["WeakRef"]"#,
        "globalThis.FinalizationRegistry",
        r#"globalThis["FinalizationRegistry"]"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn late_js_compatibility_source_includes_bracketed_permission_escalation_forms() {
    let source = late_js_compatibility_source();
    for expected in [
        r#"Deno.permissions["request"]()"#,
        r#"Deno.permissions["revoke"]()"#,
        r#"globalThis.Deno.permissions["request"]()"#,
        r#"globalThis.Deno.permissions["revoke"]()"#,
        r#"globalThis["Deno"]["permissions"]["request"]()"#,
        r#"globalThis["Deno"]["permissions"]["revoke"]()"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn late_js_compatibility_source_includes_bracketed_globalthis_deno_env_and_permission_forms() {
    let source = format!(
        "{} globalThis[\"Deno\"].permissions[\"request\"](); globalThis[\"Deno\"].permissions[\"revoke\"](); globalThis[\"Deno\"].env[\"toObject\"];",
        late_js_compatibility_source()
    );
    for expected in [
        r#"globalThis["Deno"]["permissions"]["request"]()"#,
        r#"globalThis["Deno"]["permissions"]["revoke"]()"#,
        r#"globalThis["Deno"]["env"]["toObject"]"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn late_js_compatibility_source_includes_bracketed_threaded_runtime_forms() {
    let source = late_js_compatibility_source();
    for expected in [
        r#"globalThis["SharedArrayBuffer"]"#,
        r#"globalThis["Atomics"]"#,
    ] {
        assert!(source.contains(expected), "source: {source}");
    }
}

#[test]
fn check_rejects_late_compatibility_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_js_compatibility_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("check")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_late_js_compatibility_rejection(&stderr);
}

#[test]
fn check_rejects_late_compatibility_members_in_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_js_compatibility_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("check")
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
    assert_late_js_compatibility_rejection_json(errors);
}

#[test]
fn run_rejects_late_compatibility_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_js_compatibility_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("run")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_late_js_compatibility_rejection(&stderr);
}

#[test]
fn build_rejects_late_compatibility_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_js_compatibility_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_late_js_compatibility_rejection(&stderr);
}

#[test]
fn build_rejects_late_compatibility_members_in_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_js_compatibility_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("--output")
        .arg("json")
        .arg("build")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
    assert_eq!(json["schemaVersion"], 1);
    assert_eq!(json["command"], "build");
    assert_eq!(json["success"], false);
    let errors = json["errors"].as_array().expect("errors array");
    assert_late_js_compatibility_rejection_json(errors);
}

#[test]
fn run_rejects_late_compatibility_members_in_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("main.js");
    fs::write(&source_path, late_js_compatibility_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
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
    assert_late_js_compatibility_rejection_json(errors);
}

#[test]
fn test_rejects_late_compatibility_members_in_js_input() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_js_compatibility_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("test")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_late_js_compatibility_rejection(&stderr);
}

#[test]
fn test_rejects_late_compatibility_members_in_js_input_in_json() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("smoke.test.js");
    fs::write(&source_path, late_js_compatibility_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
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
    assert_late_js_compatibility_rejection_json(errors);
}
