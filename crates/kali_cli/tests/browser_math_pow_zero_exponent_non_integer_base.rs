use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_math_pow_zero_exponent_non_integer_base_source() -> &'static str {
    r##"// kali-tree-shake: mathPowZeroExponentNonIntegerBase
function mathPowZeroExponentNonIntegerBase() {
  const base = 1.6;
  console.log(Math.pow(base, 0));
  console.log(globalThis.Math.pow(base, 0));
  return [Math.pow(base, 0), globalThis.Math.pow(base, 0)];
}
"##
}

fn assert_browser_bundle_math_pow_zero_exponent_non_integer_base(
    filename: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_math_pow_zero_exponent_non_integer_base_source(),
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
await mod.mathPowZeroExponentNonIntegerBase();
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
    assert!(stdout.contains("1\n1\n"), "stdout: {stdout}");
}

fn browser_harness_math_pow_zero_exponent_non_integer_base_run_source() -> &'static str {
    "const base = 1.6; console.log(Math.pow(base, 0)); console.log(globalThis.Math.pow(base, 0));\n"
}

fn browser_harness_math_pow_zero_exponent_non_integer_base_test_source() -> &'static str {
    r#"Kali.test('math pow zero exponent with non-integer base', () => {
  const base = 1.6;
  console.log(Math.pow(base, 0));
  console.log(globalThis.Math.pow(base, 0));
});
"#
}

fn assert_browser_harness_math_pow_zero_exponent_non_integer_base(
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
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path());
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
fn build_emits_math_pow_zero_exponent_non_integer_base_in_js_input() {
    assert_browser_bundle_math_pow_zero_exponent_non_integer_base("app.js", false);
}

#[test]
fn build_emits_math_pow_zero_exponent_non_integer_base_in_ts_input() {
    assert_browser_bundle_math_pow_zero_exponent_non_integer_base("app.ts", false);
}

#[test]
fn json_build_emits_math_pow_zero_exponent_non_integer_base_in_js_input() {
    assert_browser_bundle_math_pow_zero_exponent_non_integer_base("app.js", true);
}

#[test]
fn json_build_emits_math_pow_zero_exponent_non_integer_base_in_ts_input() {
    assert_browser_bundle_math_pow_zero_exponent_non_integer_base("app.ts", true);
}

#[test]
fn run_supports_math_pow_zero_exponent_non_integer_base_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_harness_math_pow_zero_exponent_non_integer_base(
        "run",
        "main.js",
        browser_harness_math_pow_zero_exponent_non_integer_base_run_source(),
        "1\n1",
    );
}

#[test]
fn run_supports_math_pow_zero_exponent_non_integer_base_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_harness_math_pow_zero_exponent_non_integer_base(
        "run",
        "main.ts",
        browser_harness_math_pow_zero_exponent_non_integer_base_run_source(),
        "1\n1",
    );
}

#[test]
fn test_supports_math_pow_zero_exponent_non_integer_base_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_harness_math_pow_zero_exponent_non_integer_base(
        "test",
        "smoke.test.js",
        browser_harness_math_pow_zero_exponent_non_integer_base_test_source(),
        "1\n1\nok 1",
    );
}

#[test]
fn test_supports_math_pow_zero_exponent_non_integer_base_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_harness_math_pow_zero_exponent_non_integer_base(
        "test",
        "smoke.test.ts",
        browser_harness_math_pow_zero_exponent_non_integer_base_test_source(),
        "1\n1\nok 1",
    );
}

fn browser_bundle_bracketed_global_this_math_pow_zero_exponent_non_integer_base_source(
) -> &'static str {
    r##"// kali-tree-shake: bracketedGlobalThisMathPowZeroExponentNonIntegerBase
function bracketedGlobalThisMathPowZeroExponentNonIntegerBase() {
  const base = 1.6;
  console.log(globalThis["Math"].pow(base, 0));
  console.log(globalThis["Math"]["pow"](base, 0));
  return [globalThis["Math"].pow(base, 0), globalThis["Math"]["pow"](base, 0)];
}
"##
}

fn assert_browser_bundle_bracketed_global_this_math_pow_zero_exponent_non_integer_base(
    filename: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_bracketed_global_this_math_pow_zero_exponent_non_integer_base_source(),
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
await mod.bracketedGlobalThisMathPowZeroExponentNonIntegerBase();
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
    assert!(stdout.contains("1\n1\n"), "stdout: {stdout}");
}

fn browser_harness_bracketed_global_this_math_pow_zero_exponent_non_integer_base_run_source(
) -> &'static str {
    "const base = 1.6; console.log(globalThis[\"Math\"].pow(base, 0)); console.log(globalThis[\"Math\"][\"pow\"](base, 0));\n"
}

fn browser_harness_bracketed_global_this_math_pow_zero_exponent_non_integer_base_test_source(
) -> &'static str {
    r#"Kali.test('bracketed math pow zero exponent with non-integer base', () => {
  const base = 1.6;
  console.log(globalThis["Math"].pow(base, 0));
  console.log(globalThis["Math"]["pow"](base, 0));
});
"#
}

fn assert_browser_harness_bracketed_global_this_math_pow_zero_exponent_non_integer_base(
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
        .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path());
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
fn build_emits_bracketed_global_this_math_pow_zero_exponent_non_integer_base_in_js_input() {
    assert_browser_bundle_bracketed_global_this_math_pow_zero_exponent_non_integer_base(
        "app.js", false,
    );
}

#[test]
fn build_emits_bracketed_global_this_math_pow_zero_exponent_non_integer_base_in_ts_input() {
    assert_browser_bundle_bracketed_global_this_math_pow_zero_exponent_non_integer_base(
        "app.ts", false,
    );
}

#[test]
fn json_build_emits_bracketed_global_this_math_pow_zero_exponent_non_integer_base_in_js_input() {
    assert_browser_bundle_bracketed_global_this_math_pow_zero_exponent_non_integer_base(
        "app.js", true,
    );
}

#[test]
fn json_build_emits_bracketed_global_this_math_pow_zero_exponent_non_integer_base_in_ts_input() {
    assert_browser_bundle_bracketed_global_this_math_pow_zero_exponent_non_integer_base(
        "app.ts", true,
    );
}

#[test]
fn run_supports_bracketed_global_this_math_pow_zero_exponent_non_integer_base_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_harness_bracketed_global_this_math_pow_zero_exponent_non_integer_base(
        "run",
        "main.js",
        browser_harness_bracketed_global_this_math_pow_zero_exponent_non_integer_base_run_source(),
        "1\n1",
    );
}

#[test]
fn run_supports_bracketed_global_this_math_pow_zero_exponent_non_integer_base_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_harness_bracketed_global_this_math_pow_zero_exponent_non_integer_base(
        "run",
        "main.ts",
        browser_harness_bracketed_global_this_math_pow_zero_exponent_non_integer_base_run_source(),
        "1\n1",
    );
}

#[test]
fn test_supports_bracketed_global_this_math_pow_zero_exponent_non_integer_base_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_harness_bracketed_global_this_math_pow_zero_exponent_non_integer_base(
        "test",
        "smoke.test.js",
        browser_harness_bracketed_global_this_math_pow_zero_exponent_non_integer_base_test_source(),
        "1\n1\nok 1",
    );
}

#[test]
fn test_supports_bracketed_global_this_math_pow_zero_exponent_non_integer_base_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_harness_bracketed_global_this_math_pow_zero_exponent_non_integer_base(
        "test",
        "smoke.test.ts",
        browser_harness_bracketed_global_this_math_pow_zero_exponent_non_integer_base_test_source(),
        "1\n1\nok 1",
    );
}
