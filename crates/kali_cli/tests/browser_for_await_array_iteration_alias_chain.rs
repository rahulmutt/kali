use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn for_await_const_alias_chain_source() -> &'static str {
    r##"// kali-tree-shake: forAwaitArrayIterationConstAliasChainWrapper
async function forAwaitArrayIterationConstAliasChainWrapper() {
  const values = [1, 2];
  const alias = values;
  for await (const value of alias) {
    console.log(value);
  }
}
"##
}

fn assert_browser_bundle_for_await_alias_chain(
    filename: &str,
    json_output: bool,
    source: &str,
    harness_function: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

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
        &format!(
            r#"const mod = await import(bundleJs.href);
await mod.{harness_function}();
"#
        ),
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
    assert!(stdout.contains("1\n"), "stdout: {stdout}");
    assert!(stdout.contains("2\n"), "stdout: {stdout}");
}

#[test]
fn build_emits_for_await_const_alias_chain_in_js_input() {
    assert_browser_bundle_for_await_alias_chain(
        "app.js",
        false,
        for_await_const_alias_chain_source(),
        "forAwaitArrayIterationConstAliasChainWrapper",
    );
}

#[test]
fn json_build_emits_for_await_const_alias_chain_in_js_input() {
    assert_browser_bundle_for_await_alias_chain(
        "app.js",
        true,
        for_await_const_alias_chain_source(),
        "forAwaitArrayIterationConstAliasChainWrapper",
    );
}

#[test]
fn build_emits_for_await_const_alias_chain_in_ts_input() {
    assert_browser_bundle_for_await_alias_chain(
        "app.ts",
        false,
        for_await_const_alias_chain_source(),
        "forAwaitArrayIterationConstAliasChainWrapper",
    );
}

#[test]
fn json_build_emits_for_await_const_alias_chain_in_ts_input() {
    assert_browser_bundle_for_await_alias_chain(
        "app.ts",
        true,
        for_await_const_alias_chain_source(),
        "forAwaitArrayIterationConstAliasChainWrapper",
    );
}

fn browser_harness_for_await_const_alias_chain_source() -> &'static str {
    "const values = [1, 2]; const alias = values; for await (const value of alias) { console.log(value); }\n"
}

fn assert_browser_harness_for_await_alias_chain(command: &str, filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_harness_for_await_const_alias_chain_source(),
    )
    .expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path());
    if json_output {
        cli.arg("--output").arg("json");
    }
    let output = cli
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg("--max-threads")
        .arg("0")
        .arg("--max-spawned-processes")
        .arg("0")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        assert!(
            json["stdout"]
                .as_str()
                .expect("stdout string")
                .contains("1"),
            "json: {json}"
        );
        assert!(
            json["stdout"]
                .as_str()
                .expect("stdout string")
                .contains("2"),
            "json: {json}"
        );
        assert_eq!(json["stderr"], "");
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("1"), "stdout: {stdout}");
        assert!(stdout.contains("2"), "stdout: {stdout}");
    }
}

#[test]
fn run_supports_for_await_const_alias_chain_in_browser_api_surface_with_harness_ts_input() {
    assert_browser_harness_for_await_alias_chain("run", "main.ts", false);
}

#[test]
fn test_supports_for_await_const_alias_chain_in_browser_api_surface_with_harness_ts_input() {
    assert_browser_harness_for_await_alias_chain("test", "smoke.test.ts", false);
}

#[test]
fn json_run_supports_for_await_const_alias_chain_in_browser_api_surface_with_harness_ts_input() {
    assert_browser_harness_for_await_alias_chain("run", "main.ts", true);
}

#[test]
fn json_test_supports_for_await_const_alias_chain_in_browser_api_surface_with_harness_ts_input() {
    assert_browser_harness_for_await_alias_chain("test", "smoke.test.ts", true);
}

#[test]
fn run_supports_for_await_const_alias_chain_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_for_await_alias_chain("run", "main.js", false);
}

#[test]
fn test_supports_for_await_const_alias_chain_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_for_await_alias_chain("test", "smoke.test.js", false);
}

#[test]
fn json_run_supports_for_await_const_alias_chain_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_for_await_alias_chain("run", "main.js", true);
}

#[test]
fn json_test_supports_for_await_const_alias_chain_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_for_await_alias_chain("test", "smoke.test.js", true);
}
