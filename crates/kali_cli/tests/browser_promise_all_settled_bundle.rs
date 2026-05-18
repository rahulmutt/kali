use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_promise_all_settled_source() -> &'static str {
    r##"// kali-tree-shake: browserPromiseAllSettled
async function browserPromiseAllSettled() {
  const settled = await Promise.allSettled([Promise.resolve(1), Promise.reject('boom')]);
  const mixedSettled = await Promise["allSettled"]([Promise.resolve(1), Promise.reject('boom')]);
  const dottedSettled = await globalThis.Promise.allSettled([Promise.resolve(1), Promise.reject('boom')]);
  const mixedDottedSettled = await globalThis.Promise["allSettled"]([Promise.resolve(1), Promise.reject('boom')]);
  const mixedBracketedSettled = await globalThis["Promise"].allSettled([Promise.resolve(1), Promise.reject('boom')]);
  const bracketedSettled = await globalThis["Promise"]["allSettled"]([Promise.resolve(1), Promise.reject('boom')]);
  const frozenBracketedSettled = await Object.freeze(globalThis["Promise"]["allSettled"])([Promise.resolve(1), Promise.reject('boom')]);
  if (
    settled.length !== 2 ||
    settled[0].status !== 'fulfilled' ||
    settled[0].value !== 1 ||
    settled[1].status !== 'rejected' ||
    settled[1].reason !== 'boom' ||
    mixedSettled.length !== 2 ||
    mixedSettled[0].status !== 'fulfilled' ||
    mixedSettled[0].value !== 1 ||
    mixedSettled[1].status !== 'rejected' ||
    mixedSettled[1].reason !== 'boom' ||
    dottedSettled.length !== 2 ||
    dottedSettled[0].status !== 'fulfilled' ||
    dottedSettled[0].value !== 1 ||
    dottedSettled[1].status !== 'rejected' ||
    dottedSettled[1].reason !== 'boom' ||
    mixedDottedSettled.length !== 2 ||
    mixedDottedSettled[0].status !== 'fulfilled' ||
    mixedDottedSettled[0].value !== 1 ||
    mixedDottedSettled[1].status !== 'rejected' ||
    mixedDottedSettled[1].reason !== 'boom' ||
    mixedBracketedSettled.length !== 2 ||
    mixedBracketedSettled[0].status !== 'fulfilled' ||
    mixedBracketedSettled[0].value !== 1 ||
    mixedBracketedSettled[1].status !== 'rejected' ||
    mixedBracketedSettled[1].reason !== 'boom' ||
    bracketedSettled.length !== 2 ||
    bracketedSettled[0].status !== 'fulfilled' ||
    bracketedSettled[0].value !== 1 ||
    bracketedSettled[1].status !== 'rejected' ||
    bracketedSettled[1].reason !== 'boom' ||
    frozenBracketedSettled.length !== 2 ||
    frozenBracketedSettled[0].status !== 'fulfilled' ||
    frozenBracketedSettled[0].value !== 1 ||
    frozenBracketedSettled[1].status !== 'rejected' ||
    frozenBracketedSettled[1].reason !== 'boom'
  ) {
    throw new Error('unexpected Promise.allSettled semantics');
  }
  console.log('browser promise allSettled ok');
}
"##
}

fn assert_browser_bundle_promise_all_settled(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, browser_bundle_promise_all_settled_source()).expect("write source");

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
    let harness = kali_runtime::browser_bundle_harness_script(
        "app",
        false,
        r#"const mod = await import(bundleJs.href);
await mod.browserPromiseAllSettled();
"#,
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = kali_runtime::browser_harness_command_parts_for(
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

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("browser promise allSettled ok"),
        "stdout: {stdout}"
    );
}

#[test]
fn build_emits_promise_all_settled_in_js_input() {
    assert_browser_bundle_promise_all_settled("app.js", false);
}

#[test]
fn build_emits_promise_all_settled_in_ts_input() {
    assert_browser_bundle_promise_all_settled("app.ts", false);
}

#[test]
fn build_emits_promise_all_settled_in_jsx_input() {
    assert_browser_bundle_promise_all_settled("app.jsx", false);
}

#[test]
fn build_emits_promise_all_settled_in_tsx_input() {
    assert_browser_bundle_promise_all_settled("app.tsx", false);
}

#[test]
fn json_build_emits_promise_all_settled_in_js_input() {
    assert_browser_bundle_promise_all_settled("app.js", true);
}

#[test]
fn json_build_emits_promise_all_settled_in_ts_input() {
    assert_browser_bundle_promise_all_settled("app.ts", true);
}

#[test]
fn json_build_emits_promise_all_settled_in_jsx_input() {
    assert_browser_bundle_promise_all_settled("app.jsx", true);
}

#[test]
fn json_build_emits_promise_all_settled_in_tsx_input() {
    assert_browser_bundle_promise_all_settled("app.tsx", true);
}
