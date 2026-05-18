use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_promise_all_settled_run_source() -> &'static str {
    r#"async function browserPromiseAllSettled() {
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
}

async function main() {
  await browserPromiseAllSettled();
  console.log('browser promise allSettled ok');
}

main();
"#
}

fn browser_promise_all_settled_test_source() -> &'static str {
    r#"async function browserPromiseAllSettled() {
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
}

Kali.test('browser promise allSettled', () => browserPromiseAllSettled());
"#
}
fn assert_browser_requested_promise_all_settled(command: &str, filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        browser_promise_all_settled_test_source()
    } else {
        browser_promise_all_settled_run_source()
    };
    fs::write(&source_path, source).expect("write source");

    let mut command_line = Command::new(kali_bin());
    command_line
        .current_dir(dir.path())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node");
    if json_output {
        command_line.arg("--output").arg("json");
    }
    let output = command_line
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert_eq!(json["exitCode"], 0);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        if command == "run" {
            assert_eq!(json["payload"]["exitCode"], 0);
            assert!(
                json["stdout"]
                    .as_str()
                    .expect("stdout")
                    .contains("browser promise allSettled ok"),
                "json: {json}"
            );
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
            assert_eq!(json["stdout"], "");
        }
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if command == "run" {
        assert!(
            stdout.contains("browser promise allSettled ok"),
            "stdout: {stdout}"
        );
    } else {
        assert!(stdout.contains("ok 1"), "stdout: {stdout}");
    }
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn run_supports_promise_all_settled_in_js_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("run", "main.js", false);
}

#[test]
fn run_supports_promise_all_settled_in_ts_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("run", "main.ts", false);
}

#[test]
fn run_supports_promise_all_settled_in_jsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("run", "main.jsx", false);
}

#[test]
fn run_supports_promise_all_settled_in_tsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("run", "main.tsx", false);
}

#[test]
fn test_supports_promise_all_settled_in_js_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("test", "smoke.test.js", false);
}

#[test]
fn test_supports_promise_all_settled_in_ts_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("test", "smoke.test.ts", false);
}

#[test]
fn test_supports_promise_all_settled_in_jsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("test", "smoke.test.jsx", false);
}

#[test]
fn test_supports_promise_all_settled_in_tsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("test", "smoke.test.tsx", false);
}

#[test]
fn json_run_supports_promise_all_settled_in_js_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("run", "main.js", true);
}

#[test]
fn json_run_supports_promise_all_settled_in_ts_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("run", "main.ts", true);
}

#[test]
fn json_run_supports_promise_all_settled_in_jsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("run", "main.jsx", true);
}

#[test]
fn json_run_supports_promise_all_settled_in_tsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("run", "main.tsx", true);
}

#[test]
fn json_test_supports_promise_all_settled_in_js_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("test", "smoke.test.js", true);
}

#[test]
fn json_test_supports_promise_all_settled_in_ts_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("test", "smoke.test.ts", true);
}

#[test]
fn json_test_supports_promise_all_settled_in_jsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("test", "smoke.test.jsx", true);
}

#[test]
fn json_test_supports_promise_all_settled_in_tsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_all_settled("test", "smoke.test.tsx", true);
}
