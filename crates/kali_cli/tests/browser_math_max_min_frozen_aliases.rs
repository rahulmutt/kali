use std::{fs, process::Command, sync::OnceLock};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_global_this_math_max_min_frozen_source() -> &'static str {
    static SOURCE: OnceLock<String> = OnceLock::new();
    SOURCE
        .get_or_init(|| {
            r##"// kali-tree-shake: globalThisMathMaxMinFrozenAliases
function globalThisMathMaxMinFrozenAliases() {
  const value = 2;
  const alias = value;
  console.log(globalThis.Math.max(1, alias, 3));
  console.log(globalThis.Math.min(3, alias, 1));
  console.log(Object.freeze(Math.max)(1, alias, 3));
  console.log(Object.freeze(Math.min)(3, alias, 1));
  console.log(Object.freeze(globalThis.Math["max"])(1, alias, 3));
  console.log(Object.freeze(globalThis.Math["min"])(3, alias, 1));
  console.log(Object.freeze(globalThis.Math['max'])(1, alias, 3));
  console.log(Object.freeze(globalThis.Math['min'])(3, alias, 1));
  console.log(Object.freeze(globalThis["Math"]["max"])(1, alias, 3));
  console.log(Object.freeze(globalThis["Math"]["min"])(3, alias, 1));
  console.log(Object.freeze(globalThis["Math"]['max'])(1, alias, 3));
  console.log(Object.freeze(globalThis["Math"]['min'])(3, alias, 1));
  console.log(Object.freeze(globalThis['Math']['max'])(1, alias, 3));
  console.log(Object.freeze(globalThis['Math']['min'])(3, alias, 1));
  console.log(Object.freeze(globalThis['Math'].max)(1, alias, 3));
  console.log(Object.freeze(globalThis['Math'].min)(3, alias, 1));
  console.log(Object.freeze(Math["max"])(1, alias, 3));
  console.log(Object.freeze(Math["min"])(3, alias, 1));
  console.log(Object.freeze(Math['max'])(1, alias, 3));
  console.log(Object.freeze(Math['min'])(3, alias, 1));
  return [
    globalThis.Math.max(1, alias, 3),
    globalThis.Math.min(3, alias, 1),
    Object.freeze(Math.max)(1, alias, 3),
    Object.freeze(Math.min)(3, alias, 1),
    Object.freeze(globalThis.Math["max"])(1, alias, 3),
    Object.freeze(globalThis.Math["min"])(3, alias, 1),
    Object.freeze(globalThis.Math['max'])(1, alias, 3),
    Object.freeze(globalThis.Math['min'])(3, alias, 1),
    Object.freeze(globalThis["Math"]["max"])(1, alias, 3),
    Object.freeze(globalThis["Math"]["min"])(3, alias, 1),
    Object.freeze(globalThis["Math"]['max'])(1, alias, 3),
    Object.freeze(globalThis["Math"]['min'])(3, alias, 1),
    Object.freeze(globalThis['Math']['max'])(1, alias, 3),
    Object.freeze(globalThis['Math']['min'])(3, alias, 1),
    Object.freeze(globalThis['Math'].max)(1, alias, 3),
    Object.freeze(globalThis['Math'].min)(3, alias, 1),
    Object.freeze(Math["max"])(1, alias, 3),
    Object.freeze(Math["min"])(3, alias, 1),
    Object.freeze(Math['max'])(1, alias, 3),
    Object.freeze(Math['min'])(3, alias, 1),
  ];
}
"##
            .to_string()
        })
        .as_str()
}

fn browser_harness_global_this_math_max_min_run_source() -> &'static str {
    static SOURCE: OnceLock<String> = OnceLock::new();
    SOURCE
        .get_or_init(|| {
            "const value = 2; const alias = value; console.log(globalThis.Math.max(1, alias, 3)); console.log(globalThis.Math.min(3, alias, 1)); console.log(Object.freeze(Math.max)(1, alias, 3)); console.log(Object.freeze(Math.min)(3, alias, 1)); console.log(Object.freeze(globalThis.Math[\"max\"])(1, alias, 3)); console.log(Object.freeze(globalThis.Math[\"min\"])(3, alias, 1)); console.log(Object.freeze(globalThis.Math['max'])(1, alias, 3)); console.log(Object.freeze(globalThis.Math['min'])(3, alias, 1)); console.log(Object.freeze(globalThis[\"Math\"][\"max\"])(1, alias, 3)); console.log(Object.freeze(globalThis[\"Math\"][\"min\"])(3, alias, 1)); console.log(Object.freeze(globalThis[\"Math\"]['max'])(1, alias, 3)); console.log(Object.freeze(globalThis[\"Math\"]['min'])(3, alias, 1)); console.log(Object.freeze(globalThis['Math']['max'])(1, alias, 3)); console.log(Object.freeze(globalThis['Math']['min'])(3, alias, 1)); console.log(Object.freeze(globalThis['Math'].max)(1, alias, 3)); console.log(Object.freeze(globalThis['Math'].min)(3, alias, 1)); console.log(Object.freeze(Math[\"max\"])(1, alias, 3)); console.log(Object.freeze(Math[\"min\"])(3, alias, 1)); console.log(Object.freeze(Math['max'])(1, alias, 3)); console.log(Object.freeze(Math['min'])(3, alias, 1));\n".to_string()
        })
        .as_str()
}

fn browser_harness_global_this_math_max_min_test_source() -> &'static str {
    static SOURCE: OnceLock<String> = OnceLock::new();
    SOURCE
        .get_or_init(|| {
            r#"Kali.test('globalThis.Math max min frozen aliases', () => {
  const value = 2;
  const alias = value;
  console.log(globalThis.Math.max(1, alias, 3));
  console.log(globalThis.Math.min(3, alias, 1));
  console.log(Object.freeze(Math.max)(1, alias, 3));
  console.log(Object.freeze(Math.min)(3, alias, 1));
  console.log(Object.freeze(globalThis.Math["max"])(1, alias, 3));
  console.log(Object.freeze(globalThis.Math["min"])(3, alias, 1));
  console.log(Object.freeze(globalThis.Math['max'])(1, alias, 3));
  console.log(Object.freeze(globalThis.Math['min'])(3, alias, 1));
  console.log(Object.freeze(globalThis["Math"]["max"])(1, alias, 3));
  console.log(Object.freeze(globalThis["Math"]["min"])(3, alias, 1));
  console.log(Object.freeze(globalThis["Math"]['max'])(1, alias, 3));
  console.log(Object.freeze(globalThis["Math"]['min'])(3, alias, 1));
  console.log(Object.freeze(globalThis['Math']['max'])(1, alias, 3));
  console.log(Object.freeze(globalThis['Math']['min'])(3, alias, 1));
  console.log(Object.freeze(globalThis['Math'].max)(1, alias, 3));
  console.log(Object.freeze(globalThis['Math'].min)(3, alias, 1));
  console.log(Object.freeze(Math["max"])(1, alias, 3));
  console.log(Object.freeze(Math["min"])(3, alias, 1));
  console.log(Object.freeze(Math['max'])(1, alias, 3));
  console.log(Object.freeze(Math['min'])(3, alias, 1));
});
"#
            .to_string()
        })
        .as_str()
}

fn assert_browser_bundle_global_this_math_max_min_frozen(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_global_this_math_max_min_frozen_source(),
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
    let harness = kali_runtime_contract::browser_bundle_harness_script(
        "app",
        false,
        r#"const mod = await import(bundleJs.href);
await mod.globalThisMathMaxMinFrozenAliases();
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

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("3\n"), "stdout: {stdout}");
    assert!(stdout.contains("1\n"), "stdout: {stdout}");
}

fn assert_browser_harness_global_this_math_max_min_frozen(
    command: &str,
    filename: &str,
    source: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut output = Command::new(kali_bin());
    output
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node");
    if json_output {
        output.arg("--output").arg("json");
    }
    let output = output
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
        let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        if command == "run" {
            assert_eq!(json["exitCode"], 0);
            assert_eq!(json["payload"]["exitCode"], 0);
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
        }
        assert!(
            json["stdout"]
                .as_str()
                .expect("stdout string")
                .contains("3\n"),
            "json: {json}"
        );
        assert!(
            json["stdout"]
                .as_str()
                .expect("stdout string")
                .contains("1\n"),
            "json: {json}"
        );
        assert_eq!(json["stderr"], "");
        assert_eq!(json["errors"], serde_json::Value::Array(vec![]));
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("3\n"), "stdout: {stdout}");
        assert!(stdout.contains("1\n"), "stdout: {stdout}");
    }
}

#[test]
fn browser_bundle_global_this_math_max_min_frozen_source_includes_direct_frozen_math_aliases() {
    let source = browser_bundle_global_this_math_max_min_frozen_source();
    assert!(
        source.contains("Object.freeze(Math.max)"),
        "source: {source}"
    );
    assert!(
        source.contains("Object.freeze(Math.min)"),
        "source: {source}"
    );
    assert!(
        source.contains("Object.freeze(globalThis.Math[\"max\"])")
            && source.contains("Object.freeze(globalThis.Math[\"min\"])"),
        "source: {source}"
    );
    assert!(
        source.contains("Object.freeze(globalThis[\"Math\"][\"max\"])")
            && source.contains("Object.freeze(globalThis[\"Math\"][\"min\"])"),
        "source: {source}"
    );
    assert!(
        source.contains("Object.freeze(Math[\"max\"])")
            && source.contains("Object.freeze(Math[\"min\"])"),
        "source: {source}"
    );
}

#[test]
fn build_emits_global_this_math_max_min_frozen_aliases_in_js_input() {
    assert_browser_bundle_global_this_math_max_min_frozen("app.js", false);
}

#[test]
fn build_emits_global_this_math_max_min_frozen_aliases_in_ts_input() {
    assert_browser_bundle_global_this_math_max_min_frozen("app.ts", false);
}

#[test]
fn json_build_emits_global_this_math_max_min_frozen_aliases_in_js_input() {
    assert_browser_bundle_global_this_math_max_min_frozen("app.js", true);
}

#[test]
fn json_build_emits_global_this_math_max_min_frozen_aliases_in_ts_input() {
    assert_browser_bundle_global_this_math_max_min_frozen("app.ts", true);
}

#[test]
fn run_supports_global_this_math_max_min_frozen_aliases_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_harness_global_this_math_max_min_frozen(
        "run",
        "main.js",
        browser_harness_global_this_math_max_min_run_source(),
        false,
    );
}

#[test]
fn run_supports_global_this_math_max_min_frozen_aliases_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_harness_global_this_math_max_min_frozen(
        "run",
        "main.ts",
        browser_harness_global_this_math_max_min_run_source(),
        false,
    );
}

#[test]
fn test_supports_global_this_math_max_min_frozen_aliases_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_harness_global_this_math_max_min_frozen(
        "test",
        "smoke.test.js",
        browser_harness_global_this_math_max_min_test_source(),
        false,
    );
}

#[test]
fn test_supports_global_this_math_max_min_frozen_aliases_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_harness_global_this_math_max_min_frozen(
        "test",
        "smoke.test.ts",
        browser_harness_global_this_math_max_min_test_source(),
        false,
    );
}

#[test]
fn json_run_supports_global_this_math_max_min_frozen_aliases_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_harness_global_this_math_max_min_frozen(
        "run",
        "main.js",
        browser_harness_global_this_math_max_min_run_source(),
        true,
    );
}

#[test]
fn json_run_supports_global_this_math_max_min_frozen_aliases_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_harness_global_this_math_max_min_frozen(
        "run",
        "main.ts",
        browser_harness_global_this_math_max_min_run_source(),
        true,
    );
}

#[test]
fn json_test_supports_global_this_math_max_min_frozen_aliases_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_harness_global_this_math_max_min_frozen(
        "test",
        "smoke.test.js",
        browser_harness_global_this_math_max_min_test_source(),
        true,
    );
}

#[test]
fn json_test_supports_global_this_math_max_min_frozen_aliases_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_harness_global_this_math_max_min_frozen(
        "test",
        "smoke.test.ts",
        browser_harness_global_this_math_max_min_test_source(),
        true,
    );
}
