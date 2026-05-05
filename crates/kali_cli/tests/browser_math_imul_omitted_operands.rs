use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_math_imul_omitted_operands_source() -> &'static str {
    r##"// kali-tree-shake: mathImulOmittedOperands
function mathImulOmittedOperands() {
  console.log(Math.imul());
  console.log(globalThis.Math.imul());
  console.log(globalThis.Math["imul"]());
  console.log(globalThis["Math"].imul());
  console.log(globalThis["Math"]["imul"]());
  return [
    Math.imul(),
    globalThis.Math.imul(),
    globalThis.Math["imul"](),
    globalThis["Math"].imul(),
    globalThis["Math"]["imul"](),
  ];
}
"##
}

fn browser_harness_math_imul_omitted_operands_run_source() -> &'static str {
    "console.log(Math.imul()); console.log(globalThis.Math.imul()); console.log(globalThis.Math[\"imul\"]()); console.log(globalThis[\"Math\"].imul()); console.log(globalThis[\"Math\"][\"imul\"]());\n"
}

fn browser_harness_math_imul_omitted_operands_test_source() -> &'static str {
    r#"Kali.test('math imul omitted operands', () => {
  console.log(Math.imul());
  console.log(globalThis.Math.imul());
  console.log(globalThis.Math["imul"]());
  console.log(globalThis["Math"].imul());
  console.log(globalThis["Math"]["imul"]());
});
"#
}

fn assert_browser_bundle_math_imul_omitted_operands(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_math_imul_omitted_operands_source(),
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
await mod.mathImulOmittedOperands();
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
    assert!(stdout.contains("0\n0\n0\n0\n0\n"), "stdout: {stdout}");
}

fn assert_browser_harness_math_imul_omitted_operands(
    command: &str,
    filename: &str,
    source: &str,
    expected_stdout: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut output = Command::new(kali_bin());
    output
        .current_dir(dir.path())
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node");
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

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
}

#[test]
fn build_emits_math_imul_omitted_operands_in_js_input() {
    assert_browser_bundle_math_imul_omitted_operands("app.js", false);
}

#[test]
fn build_emits_math_imul_omitted_operands_in_ts_input() {
    assert_browser_bundle_math_imul_omitted_operands("app.ts", false);
}

#[test]
fn build_emits_math_imul_omitted_operands_in_jsx_input() {
    assert_browser_bundle_math_imul_omitted_operands("app.jsx", false);
}

#[test]
fn build_emits_math_imul_omitted_operands_in_tsx_input() {
    assert_browser_bundle_math_imul_omitted_operands("app.tsx", false);
}

#[test]
fn json_build_emits_math_imul_omitted_operands_in_js_input() {
    assert_browser_bundle_math_imul_omitted_operands("app.js", true);
}

#[test]
fn json_build_emits_math_imul_omitted_operands_in_ts_input() {
    assert_browser_bundle_math_imul_omitted_operands("app.ts", true);
}

#[test]
fn json_build_emits_math_imul_omitted_operands_in_jsx_input() {
    assert_browser_bundle_math_imul_omitted_operands("app.jsx", true);
}

#[test]
fn json_build_emits_math_imul_omitted_operands_in_tsx_input() {
    assert_browser_bundle_math_imul_omitted_operands("app.tsx", true);
}

#[test]
fn run_supports_math_imul_omitted_operands_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_imul_omitted_operands(
        "run",
        "main.js",
        browser_harness_math_imul_omitted_operands_run_source(),
        "0\n0\n0\n0\n0",
    );
}

#[test]
fn run_supports_math_imul_omitted_operands_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_imul_omitted_operands(
        "run",
        "main.ts",
        browser_harness_math_imul_omitted_operands_run_source(),
        "0\n0\n0\n0\n0",
    );
}

#[test]
fn test_supports_math_imul_omitted_operands_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_imul_omitted_operands(
        "test",
        "smoke.test.js",
        browser_harness_math_imul_omitted_operands_test_source(),
        "0\n0\n0\n0\n0\nok 1",
    );
}

#[test]
fn test_supports_math_imul_omitted_operands_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_imul_omitted_operands(
        "test",
        "smoke.test.ts",
        browser_harness_math_imul_omitted_operands_test_source(),
        "0\n0\n0\n0\n0\nok 1",
    );
}

#[test]
fn run_supports_math_imul_omitted_operands_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_math_imul_omitted_operands(
        "run",
        "main.jsx",
        browser_harness_math_imul_omitted_operands_run_source(),
        "0\n0\n0\n0\n0",
    );
}

#[test]
fn run_supports_math_imul_omitted_operands_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_math_imul_omitted_operands(
        "run",
        "main.tsx",
        browser_harness_math_imul_omitted_operands_run_source(),
        "0\n0\n0\n0\n0",
    );
}

#[test]
fn test_supports_math_imul_omitted_operands_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_math_imul_omitted_operands(
        "test",
        "smoke.test.jsx",
        browser_harness_math_imul_omitted_operands_test_source(),
        "0\n0\n0\n0\n0\nok 1",
    );
}

#[test]
fn test_supports_math_imul_omitted_operands_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_math_imul_omitted_operands(
        "test",
        "smoke.test.tsx",
        browser_harness_math_imul_omitted_operands_test_source(),
        "0\n0\n0\n0\n0\nok 1",
    );
}

#[test]
fn json_run_supports_math_imul_omitted_operands_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_imul_omitted_operands(
        "run",
        "main.js",
        browser_harness_math_imul_omitted_operands_run_source(),
        "0\n0\n0\n0\n0",
    );
}

#[test]
fn json_run_supports_math_imul_omitted_operands_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_imul_omitted_operands(
        "run",
        "main.ts",
        browser_harness_math_imul_omitted_operands_run_source(),
        "0\n0\n0\n0\n0",
    );
}

#[test]
fn json_test_supports_math_imul_omitted_operands_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_imul_omitted_operands(
        "test",
        "smoke.test.js",
        browser_harness_math_imul_omitted_operands_test_source(),
        "0\n0\n0\n0\n0\nok 1",
    );
}

#[test]
fn json_test_supports_math_imul_omitted_operands_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_imul_omitted_operands(
        "test",
        "smoke.test.ts",
        browser_harness_math_imul_omitted_operands_test_source(),
        "0\n0\n0\n0\n0\nok 1",
    );
}

#[test]
fn json_run_supports_math_imul_omitted_operands_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_math_imul_omitted_operands(
        "run",
        "main.jsx",
        browser_harness_math_imul_omitted_operands_run_source(),
        "0\n0\n0\n0\n0",
    );
}

#[test]
fn json_run_supports_math_imul_omitted_operands_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_math_imul_omitted_operands(
        "run",
        "main.tsx",
        browser_harness_math_imul_omitted_operands_run_source(),
        "0\n0\n0\n0\n0",
    );
}

#[test]
fn json_test_supports_math_imul_omitted_operands_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_math_imul_omitted_operands(
        "test",
        "smoke.test.jsx",
        browser_harness_math_imul_omitted_operands_test_source(),
        "0\n0\n0\n0\n0\nok 1",
    );
}

#[test]
fn json_test_supports_math_imul_omitted_operands_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_math_imul_omitted_operands(
        "test",
        "smoke.test.tsx",
        browser_harness_math_imul_omitted_operands_test_source(),
        "0\n0\n0\n0\n0\nok 1",
    );
}
