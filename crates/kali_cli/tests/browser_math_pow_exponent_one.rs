use std::{fs, process::Command};

use kali_common::math_pow_frozen_callable_aliases;
use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_math_pow_exponent_one_source() -> String {
    let frozen_lines = math_pow_frozen_callable_aliases()
        .iter()
        .map(|alias| format!("  console.log({alias}(2, alias));"))
        .collect::<Vec<_>>()
        .join("\n");
    let frozen_entries = math_pow_frozen_callable_aliases()
        .iter()
        .map(|alias| format!("    {alias}(2, alias),"))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r##"// kali-tree-shake: mathPowExponentOneIdentity
function mathPowExponentOneIdentity() {{
  const exponent = 1;
  const alias = exponent;
  console.log(Math.pow(2, alias));
  console.log(globalThis.Math.pow(2, alias));
  console.log(globalThis["Math"]["pow"](2, alias));
  console.log(globalThis.Math["pow"](2, alias));
{frozen_lines}
  return [
    Math.pow(2, alias),
    globalThis.Math.pow(2, alias),
    globalThis["Math"]["pow"](2, alias),
    globalThis.Math["pow"](2, alias),
{frozen_entries}
  ];
}}
"##,
        frozen_lines = frozen_lines,
        frozen_entries = frozen_entries,
    )
}

fn assert_browser_bundle_math_pow_exponent_one_identity(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, browser_bundle_math_pow_exponent_one_source()).expect("write source");

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
await mod.mathPowExponentOneIdentity();
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
    let expected_stdout = format!(
        "{}\n",
        std::iter::repeat("2")
            .take(4 + math_pow_frozen_callable_aliases().len())
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(stdout.contains(&expected_stdout), "stdout: {stdout}");
}

fn browser_harness_math_pow_exponent_one_identity_run_source() -> &'static str {
    "const exponent = 1; const alias = exponent; console.log(Math.pow(2, alias)); console.log(globalThis.Math.pow(2, alias)); console.log(globalThis[\"Math\"][\"pow\"](2, alias)); console.log(globalThis.Math[\"pow\"](2, alias)); console.log(Object.freeze(globalThis.Math[\"pow\"])(2, alias)); console.log(Object.freeze((globalThis.Math[\"pow\"]))(2, alias)); console.log(Object.freeze(globalThis[\"Math\"][\"pow\"])(2, alias)); console.log(Object.freeze((globalThis[\"Math\"][\"pow\"]))(2, alias)); console.log(Object.freeze(globalThis.Math.pow)(2, alias)); console.log(Object.freeze((globalThis.Math.pow))(2, alias)); console.log(Object.freeze(globalThis[\"Math\"].pow)(2, alias)); console.log(Object.freeze((globalThis[\"Math\"].pow))(2, alias));\n"
}

fn browser_harness_math_pow_exponent_one_identity_test_source() -> &'static str {
    r#"Kali.test('math pow exponent one identity', () => {
  const exponent = 1;
  const alias = exponent;
  console.log(Math.pow(2, alias));
  console.log(globalThis.Math.pow(2, alias));
  console.log(globalThis["Math"]["pow"](2, alias));
  console.log(globalThis.Math["pow"](2, alias));
  console.log(Object.freeze(globalThis.Math["pow"])(2, alias));
  console.log(Object.freeze((globalThis.Math["pow"]))(2, alias));
  console.log(Object.freeze(globalThis["Math"]["pow"])(2, alias));
  console.log(Object.freeze((globalThis["Math"]["pow"]))(2, alias));
  console.log(Object.freeze(globalThis.Math.pow)(2, alias));
  console.log(Object.freeze((globalThis.Math.pow))(2, alias));
  console.log(Object.freeze(globalThis["Math"].pow)(2, alias));
  console.log(Object.freeze((globalThis["Math"].pow))(2, alias));
});
"#
}

fn browser_bundle_math_pow_base_one_identity_source() -> &'static str {
    r##"// kali-tree-shake: mathPowBaseOneIdentity
function mathPowBaseOneIdentity() {
  const exponent = 7;
  const alias = exponent;
  console.log(Math.pow(1, alias));
  console.log(globalThis.Math.pow(1, alias));
  console.log(globalThis["Math"]["pow"](1, alias));
  console.log(globalThis.Math["pow"](1, alias));
  console.log(Object.freeze(globalThis.Math["pow"])(1, alias));
  console.log(Object.freeze((globalThis.Math["pow"]))(1, alias));
  console.log(Object.freeze(globalThis["Math"]["pow"])(1, alias));
  console.log(Object.freeze((globalThis["Math"]["pow"]))(1, alias));
  console.log(Object.freeze(globalThis.Math.pow)(1, alias));
  console.log(Object.freeze((globalThis.Math.pow))(1, alias));
  console.log(Object.freeze(globalThis["Math"].pow)(1, alias));
  console.log(Object.freeze((globalThis["Math"].pow))(1, alias));
  return [
    Math.pow(1, alias),
    globalThis.Math.pow(1, alias),
    globalThis["Math"]["pow"](1, alias),
    globalThis.Math["pow"](1, alias),
    Object.freeze(globalThis.Math["pow"])(1, alias),
    Object.freeze((globalThis.Math["pow"]))(1, alias),
    Object.freeze(globalThis["Math"]["pow"])(1, alias),
    Object.freeze((globalThis["Math"]["pow"]))(1, alias),
    Object.freeze(globalThis.Math.pow)(1, alias),
    Object.freeze((globalThis.Math.pow))(1, alias),
    Object.freeze(globalThis["Math"].pow)(1, alias),
    Object.freeze((globalThis["Math"].pow))(1, alias),
  ];
}
"##
}

fn assert_browser_bundle_math_pow_base_one_identity(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_math_pow_base_one_identity_source(),
    )
    .expect("write source");

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
await mod.mathPowBaseOneIdentity();
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
        stdout.contains("1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n1\n"),
        "stdout: {stdout}"
    );
}

fn browser_harness_math_pow_base_one_identity_run_source() -> &'static str {
    "const exponent = 7; const alias = exponent; console.log(Math.pow(1, alias)); console.log(globalThis.Math.pow(1, alias)); console.log(globalThis[\"Math\"][\"pow\"](1, alias)); console.log(globalThis.Math[\"pow\"](1, alias)); console.log(Object.freeze(globalThis.Math[\"pow\"])(1, alias)); console.log(Object.freeze((globalThis.Math[\"pow\"]))(1, alias)); console.log(Object.freeze(globalThis[\"Math\"][\"pow\"])(1, alias)); console.log(Object.freeze((globalThis[\"Math\"][\"pow\"]))(1, alias)); console.log(Object.freeze(globalThis.Math.pow)(1, alias)); console.log(Object.freeze((globalThis.Math.pow))(1, alias)); console.log(Object.freeze(globalThis[\"Math\"].pow)(1, alias)); console.log(Object.freeze((globalThis[\"Math\"].pow))(1, alias));\n"
}

fn browser_harness_math_pow_base_one_identity_test_source() -> &'static str {
    r#"Kali.test('math pow base one identity', () => {
  const exponent = 7;
  const alias = exponent;
  console.log(Math.pow(1, alias));
  console.log(globalThis.Math.pow(1, alias));
  console.log(globalThis["Math"]["pow"](1, alias));
  console.log(globalThis.Math["pow"](1, alias));
  console.log(Object.freeze(globalThis.Math["pow"])(1, alias));
  console.log(Object.freeze((globalThis.Math["pow"]))(1, alias));
  console.log(Object.freeze(globalThis["Math"]["pow"])(1, alias));
  console.log(Object.freeze((globalThis["Math"]["pow"]))(1, alias));
  console.log(Object.freeze(globalThis.Math.pow)(1, alias));
  console.log(Object.freeze((globalThis.Math.pow))(1, alias));
  console.log(Object.freeze(globalThis["Math"].pow)(1, alias));
  console.log(Object.freeze((globalThis["Math"].pow))(1, alias));
});
"#
}

fn assert_browser_harness_math_pow_exponent_one_identity(
    command: &str,
    filename: &str,
    source: &str,
    expected_stdout: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut output = Command::new(kali_bin());
    output
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path());
    if json_output {
        output.arg("--output").arg("json");
    }
    let output = output
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

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        if command == "run" {
            assert_eq!(json["exitCode"], 0);
            assert_eq!(json["payload"]["exitCode"], 0);
        } else {
            let payload = json["payload"].as_object().expect("payload object");
            assert_eq!(payload["total"], 1);
            assert_eq!(payload["passed"], 1);
            assert_eq!(payload["failed"], 0);
            assert_eq!(payload["skipped"], 0);
        }
        let stdout = json["stdout"].as_str().expect("stdout");
        assert!(stdout.contains(expected_stdout), "json: {json}");
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
    }
}

fn assert_browser_harness_math_pow_base_one_identity(
    command: &str,
    filename: &str,
    source: &str,
    expected_stdout: &str,
    json_output: bool,
) {
    assert_browser_harness_math_pow_exponent_one_identity(
        command,
        filename,
        source,
        expected_stdout,
        json_output,
    );
}

#[test]
fn build_emits_math_pow_exponent_one_identity_in_js_input() {
    assert_browser_bundle_math_pow_exponent_one_identity("app.js", false);
}

#[test]
fn build_emits_math_pow_exponent_one_identity_in_ts_input() {
    assert_browser_bundle_math_pow_exponent_one_identity("app.ts", false);
}

#[test]
fn build_emits_math_pow_exponent_one_identity_in_jsx_input() {
    assert_browser_bundle_math_pow_exponent_one_identity("app.jsx", false);
}

#[test]
fn build_emits_math_pow_exponent_one_identity_in_tsx_input() {
    assert_browser_bundle_math_pow_exponent_one_identity("app.tsx", false);
}

#[test]
fn json_build_emits_math_pow_exponent_one_identity_in_js_input() {
    assert_browser_bundle_math_pow_exponent_one_identity("app.js", true);
}

#[test]
fn json_build_emits_math_pow_exponent_one_identity_in_ts_input() {
    assert_browser_bundle_math_pow_exponent_one_identity("app.ts", true);
}

#[test]
fn json_build_emits_math_pow_exponent_one_identity_in_jsx_input() {
    assert_browser_bundle_math_pow_exponent_one_identity("app.jsx", true);
}

#[test]
fn json_build_emits_math_pow_exponent_one_identity_in_tsx_input() {
    assert_browser_bundle_math_pow_exponent_one_identity("app.tsx", true);
}

#[test]
fn run_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_pow_exponent_one_identity(
        "run",
        "main.js",
        browser_harness_math_pow_exponent_one_identity_run_source(),
        "2\n2\n2\n2\n2\n2\n2\n2",
        false,
    );
}

#[test]
fn run_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_pow_exponent_one_identity(
        "run",
        "main.ts",
        browser_harness_math_pow_exponent_one_identity_run_source(),
        "2\n2\n2\n2\n2\n2\n2\n2",
        false,
    );
}

#[test]
fn run_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_math_pow_exponent_one_identity(
        "run",
        "main.jsx",
        browser_harness_math_pow_exponent_one_identity_run_source(),
        "2\n2\n2\n2\n2\n2\n2\n2",
        false,
    );
}

#[test]
fn run_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_math_pow_exponent_one_identity(
        "run",
        "main.tsx",
        browser_harness_math_pow_exponent_one_identity_run_source(),
        "2\n2\n2\n2\n2\n2\n2\n2",
        false,
    );
}

#[test]
fn json_run_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_js_input()
{
    assert_browser_harness_math_pow_exponent_one_identity(
        "run",
        "main.js",
        browser_harness_math_pow_exponent_one_identity_run_source(),
        "2\n2\n2\n2\n2\n2\n2\n2",
        true,
    );
}

#[test]
fn json_run_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_ts_input()
{
    assert_browser_harness_math_pow_exponent_one_identity(
        "run",
        "main.ts",
        browser_harness_math_pow_exponent_one_identity_run_source(),
        "2\n2\n2\n2\n2\n2\n2\n2",
        true,
    );
}

#[test]
fn json_run_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_jsx_input(
) {
    assert_browser_harness_math_pow_exponent_one_identity(
        "run",
        "main.jsx",
        browser_harness_math_pow_exponent_one_identity_run_source(),
        "2\n2\n2\n2\n2\n2\n2\n2",
        true,
    );
}

#[test]
fn json_run_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_tsx_input(
) {
    assert_browser_harness_math_pow_exponent_one_identity(
        "run",
        "main.tsx",
        browser_harness_math_pow_exponent_one_identity_run_source(),
        "2\n2\n2\n2\n2\n2\n2\n2",
        true,
    );
}

#[test]
fn test_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_pow_exponent_one_identity(
        "test",
        "smoke.test.js",
        browser_harness_math_pow_exponent_one_identity_test_source(),
        "2\n2\n2\n2\n2\n2\n2\n2\nok 1",
        false,
    );
}

#[test]
fn test_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_pow_exponent_one_identity(
        "test",
        "smoke.test.ts",
        browser_harness_math_pow_exponent_one_identity_test_source(),
        "2\n2\n2\n2\n2\n2\n2\n2\nok 1",
        false,
    );
}

#[test]
fn test_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_math_pow_exponent_one_identity(
        "test",
        "smoke.test.jsx",
        browser_harness_math_pow_exponent_one_identity_test_source(),
        "2\n2\n2\n2\n2\n2\n2\n2\nok 1",
        false,
    );
}

#[test]
fn test_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_math_pow_exponent_one_identity(
        "test",
        "smoke.test.tsx",
        browser_harness_math_pow_exponent_one_identity_test_source(),
        "2\n2\n2\n2\n2\n2\n2\n2\nok 1",
        false,
    );
}

#[test]
fn json_test_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_harness_math_pow_exponent_one_identity(
        "test",
        "smoke.test.js",
        browser_harness_math_pow_exponent_one_identity_test_source(),
        "2\n2\n2\n2\n2\n2\n2\n2\n",
        true,
    );
}

#[test]
fn json_test_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_harness_math_pow_exponent_one_identity(
        "test",
        "smoke.test.ts",
        browser_harness_math_pow_exponent_one_identity_test_source(),
        "2\n2\n2\n2\n2\n2\n2\n2\n",
        true,
    );
}

#[test]
fn json_test_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_jsx_input(
) {
    assert_browser_harness_math_pow_exponent_one_identity(
        "test",
        "smoke.test.jsx",
        browser_harness_math_pow_exponent_one_identity_test_source(),
        "2\n2\n2\n2\n2\n2\n2\n2\n",
        true,
    );
}

#[test]
fn json_test_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_tsx_input(
) {
    assert_browser_harness_math_pow_exponent_one_identity(
        "test",
        "smoke.test.tsx",
        browser_harness_math_pow_exponent_one_identity_test_source(),
        "2\n2\n2\n2\n2\n2\n2\n2\n",
        true,
    );
}

#[test]
fn build_emits_math_pow_base_one_identity_in_js_input() {
    assert_browser_bundle_math_pow_base_one_identity("app.js", false);
}

#[test]
fn build_emits_math_pow_base_one_identity_in_ts_input() {
    assert_browser_bundle_math_pow_base_one_identity("app.ts", false);
}

#[test]
fn build_emits_math_pow_base_one_identity_in_jsx_input() {
    assert_browser_bundle_math_pow_base_one_identity("app.jsx", false);
}

#[test]
fn build_emits_math_pow_base_one_identity_in_tsx_input() {
    assert_browser_bundle_math_pow_base_one_identity("app.tsx", false);
}

#[test]
fn json_build_emits_math_pow_base_one_identity_in_js_input() {
    assert_browser_bundle_math_pow_base_one_identity("app.js", true);
}

#[test]
fn json_build_emits_math_pow_base_one_identity_in_ts_input() {
    assert_browser_bundle_math_pow_base_one_identity("app.ts", true);
}

#[test]
fn json_build_emits_math_pow_base_one_identity_in_jsx_input() {
    assert_browser_bundle_math_pow_base_one_identity("app.jsx", true);
}

#[test]
fn json_build_emits_math_pow_base_one_identity_in_tsx_input() {
    assert_browser_bundle_math_pow_base_one_identity("app.tsx", true);
}

#[test]
fn run_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "run",
        "main.js",
        browser_harness_math_pow_base_one_identity_run_source(),
        "1\n1\n1\n1\n1\n1\n1\n1",
        false,
    );
}

#[test]
fn json_run_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "run",
        "main.js",
        browser_harness_math_pow_base_one_identity_run_source(),
        "1\n1\n1\n1\n1\n1\n1\n1",
        true,
    );
}

#[test]
fn run_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "run",
        "main.ts",
        browser_harness_math_pow_base_one_identity_run_source(),
        "1\n1\n1\n1\n1\n1\n1\n1",
        false,
    );
}

#[test]
fn run_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "run",
        "main.jsx",
        browser_harness_math_pow_base_one_identity_run_source(),
        "1\n1\n1\n1\n1\n1\n1\n1",
        false,
    );
}

#[test]
fn run_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "run",
        "main.tsx",
        browser_harness_math_pow_base_one_identity_run_source(),
        "1\n1\n1\n1\n1\n1\n1\n1",
        false,
    );
}

#[test]
fn json_run_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "run",
        "main.ts",
        browser_harness_math_pow_base_one_identity_run_source(),
        "1\n1\n1\n1\n1\n1\n1\n1",
        true,
    );
}

#[test]
fn json_run_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "run",
        "main.jsx",
        browser_harness_math_pow_base_one_identity_run_source(),
        "1\n1\n1\n1\n1\n1\n1\n1",
        true,
    );
}

#[test]
fn json_run_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "run",
        "main.tsx",
        browser_harness_math_pow_base_one_identity_run_source(),
        "1\n1\n1\n1\n1\n1\n1\n1",
        true,
    );
}

#[test]
fn test_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "test",
        "smoke.test.js",
        browser_harness_math_pow_base_one_identity_test_source(),
        "1\n1\n1\n1\n1\n1\n1\n1\nok 1",
        false,
    );
}

#[test]
fn test_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "test",
        "smoke.test.ts",
        browser_harness_math_pow_base_one_identity_test_source(),
        "1\n1\n1\n1\n1\n1\n1\n1\nok 1",
        false,
    );
}

#[test]
fn test_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "test",
        "smoke.test.jsx",
        browser_harness_math_pow_base_one_identity_test_source(),
        "1\n1\n1\n1\n1\n1\n1\n1\nok 1",
        false,
    );
}

#[test]
fn test_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "test",
        "smoke.test.tsx",
        browser_harness_math_pow_base_one_identity_test_source(),
        "1\n1\n1\n1\n1\n1\n1\n1\nok 1",
        false,
    );
}

#[test]
fn json_test_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "test",
        "smoke.test.js",
        browser_harness_math_pow_base_one_identity_test_source(),
        "1\n1\n1\n1\n1\n1\n1\n1\n",
        true,
    );
}

#[test]
fn json_test_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "test",
        "smoke.test.ts",
        browser_harness_math_pow_base_one_identity_test_source(),
        "1\n1\n1\n1\n1\n1\n1\n1\n",
        true,
    );
}

#[test]
fn json_test_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "test",
        "smoke.test.jsx",
        browser_harness_math_pow_base_one_identity_test_source(),
        "1\n1\n1\n1\n1\n1\n1\n1\n",
        true,
    );
}

#[test]
fn json_test_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "test",
        "smoke.test.tsx",
        browser_harness_math_pow_base_one_identity_test_source(),
        "1\n1\n1\n1\n1\n1\n1\n1\n",
        true,
    );
}
