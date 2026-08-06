use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

use kali_common::promise_race_browser_body_source;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_promise_race_source() -> String {
    format!(
        "// kali-tree-shake: browserPromiseRace\nasync function browserPromiseRace() {{\n{}\n}}\n",
        promise_race_browser_body_source()
    )
}

fn assert_browser_bundle_promise_race(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, browser_bundle_promise_race_source()).expect("write source");

    let mut command = Command::new(kali_bin());
    command
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser");
    if json_output {
        command.arg("--output").arg("json");
    }
    let output = command.arg(&source_path).output().expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if json_output {
        let envelope: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(envelope["schemaVersion"], 1);
        assert_eq!(envelope["command"], "build");
        assert_eq!(envelope["success"], true);
        assert_eq!(envelope["exitCode"], 0);
        let payload = envelope["payload"].as_object().expect("payload object");
        assert_eq!(payload["artifactKind"], "bundle");
        assert_eq!(payload["bundleFormat"], "esm");
        assert!(envelope["errors"]
            .as_array()
            .expect("errors array")
            .is_empty());
    }

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_eq!(metadata["apiSurface"], "browser");
    assert_eq!(metadata["artifactKind"], "bundle");

    let harness_path = bundle_dir
        .parent()
        .expect("bundle root parent")
        .join("browser-bundle-smoke.mjs");
    let harness = kali_runtime_contract::browser_bundle_harness_script(
        "app",
        false,
        r#"const mod = await import(bundleJs.href);
await mod.browserPromiseRace();
console.log('browser promise race ok');
"#,
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = kali_runtime_contract::browser_harness_command_parts_for(
        std::env::var("KALI_BROWSER_BUNDLE_HARNESS_COMMAND")
            .ok()
            .as_deref(),
    );
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(&bundle_dir)
        .args(&harness_command)
        .arg(&harness_path)
        .output()
        .expect("run browser bundle harness");

    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!output.status.success(), "must fail closed: {output:?}");
}

#[test]
fn build_emits_promise_race_in_js_input() {
    assert_browser_bundle_promise_race("app.js", false);
}

#[test]
fn build_emits_promise_race_in_ts_input() {
    assert_browser_bundle_promise_race("app.ts", false);
}

#[test]
fn json_build_emits_promise_race_in_js_input() {
    assert_browser_bundle_promise_race("app.js", true);
}

#[test]
fn json_build_emits_promise_race_in_ts_input() {
    assert_browser_bundle_promise_race("app.ts", true);
}

#[test]
fn build_emits_promise_race_in_jsx_input() {
    assert_browser_bundle_promise_race("app.jsx", false);
}

#[test]
fn build_emits_promise_race_in_tsx_input() {
    assert_browser_bundle_promise_race("app.tsx", false);
}

#[test]
fn json_build_emits_promise_race_in_jsx_input() {
    assert_browser_bundle_promise_race("app.jsx", true);
}

#[test]
fn json_build_emits_promise_race_in_tsx_input() {
    assert_browser_bundle_promise_race("app.tsx", true);
}
