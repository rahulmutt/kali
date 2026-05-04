use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_bracketed_global_this_math_core_suite_source() -> &'static str {
    r##"// kali-tree-shake: bracketedGlobalThisMathCoreSuite
function bracketedGlobalThisMathCoreSuite() {
  console.log(globalThis["Math"].max(1, 2, 3));
  console.log(globalThis["Math"].min(3, 2, 1));
  console.log(globalThis["Math"].abs(3 - 6));
  console.log(globalThis["Math"].sign(3 - 6));
  console.log(globalThis["Math"].imul(2147483647, 2));
  console.log(globalThis["Math"].clz32(1));
  return [
    globalThis["Math"].max(1, 2, 3),
    globalThis["Math"].min(3, 2, 1),
    globalThis["Math"].abs(3 - 6),
    globalThis["Math"].sign(3 - 6),
    globalThis["Math"].imul(2147483647, 2),
    globalThis["Math"].clz32(1),
  ];
}
"##
}

fn assert_browser_bundle_bracketed_global_this_math_core_suite(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_bracketed_global_this_math_core_suite_source(),
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
await mod.bracketedGlobalThisMathCoreSuite();
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
    assert!(stdout.contains("3\n"), "stdout: {stdout}");
    assert!(stdout.contains("1\n"), "stdout: {stdout}");
    assert!(stdout.contains("-2\n"), "stdout: {stdout}");
    assert!(stdout.contains("31\n"), "stdout: {stdout}");
    assert!(stdout.contains("-1\n"), "stdout: {stdout}");
}

#[test]
fn build_emits_bracketed_global_this_math_core_suite_in_js_input() {
    assert_browser_bundle_bracketed_global_this_math_core_suite("app.js", false);
}

#[test]
fn build_emits_bracketed_global_this_math_core_suite_in_ts_input() {
    assert_browser_bundle_bracketed_global_this_math_core_suite("app.ts", false);
}

#[test]
fn build_emits_bracketed_global_this_math_core_suite_in_jsx_input() {
    assert_browser_bundle_bracketed_global_this_math_core_suite("app.jsx", false);
}

#[test]
fn build_emits_bracketed_global_this_math_core_suite_in_tsx_input() {
    assert_browser_bundle_bracketed_global_this_math_core_suite("app.tsx", false);
}

#[test]
fn json_build_emits_bracketed_global_this_math_core_suite_in_js_input() {
    assert_browser_bundle_bracketed_global_this_math_core_suite("app.js", true);
}

#[test]
fn json_build_emits_bracketed_global_this_math_core_suite_in_ts_input() {
    assert_browser_bundle_bracketed_global_this_math_core_suite("app.ts", true);
}

#[test]
fn json_build_emits_bracketed_global_this_math_core_suite_in_jsx_input() {
    assert_browser_bundle_bracketed_global_this_math_core_suite("app.jsx", true);
}

#[test]
fn json_build_emits_bracketed_global_this_math_core_suite_in_tsx_input() {
    assert_browser_bundle_bracketed_global_this_math_core_suite("app.tsx", true);
}

#[test]
fn run_and_test_supports_bracketed_global_this_math_core_suite_when_browser_harness_is_configured_in_js_and_ts_input(
) {
    for (command, source_name, source, expected_stdout) in [
        (
            "run",
            "main.js",
            "console.log(globalThis[\"Math\"].max(1, 2, 3));\nconsole.log(globalThis[\"Math\"].min(3, 2, 1));\nconsole.log(globalThis[\"Math\"].abs(3 - 6));\nconsole.log(globalThis[\"Math\"].sign(3 - 6));\nconsole.log(globalThis[\"Math\"].imul(2147483647, 2));\nconsole.log(globalThis[\"Math\"].clz32(1));\n",
            "3\n1\n3\n-1\n-2\n31",
        ),
        (
            "test",
            "smoke.test.js",
            "Kali.test('bracketed math core suite', () => { console.log(globalThis[\"Math\"].max(1, 2, 3)); console.log(globalThis[\"Math\"].min(3, 2, 1)); console.log(globalThis[\"Math\"].abs(3 - 6)); console.log(globalThis[\"Math\"].sign(3 - 6)); console.log(globalThis[\"Math\"].imul(2147483647, 2)); console.log(globalThis[\"Math\"].clz32(1)); });\n",
            "3\n1\n3\n-1\n-2\n31",
        ),
        (
            "run",
            "main.ts",
            "console.log(globalThis[\"Math\"].max(1, 2, 3));\nconsole.log(globalThis[\"Math\"].min(3, 2, 1));\nconsole.log(globalThis[\"Math\"].abs(3 - 6));\nconsole.log(globalThis[\"Math\"].sign(3 - 6));\nconsole.log(globalThis[\"Math\"].imul(2147483647, 2));\nconsole.log(globalThis[\"Math\"].clz32(1));\n",
            "3\n1\n3\n-1\n-2\n31",
        ),
        (
            "test",
            "smoke.test.ts",
            "Kali.test('bracketed math core suite', () => { console.log(globalThis[\"Math\"].max(1, 2, 3)); console.log(globalThis[\"Math\"].min(3, 2, 1)); console.log(globalThis[\"Math\"].abs(3 - 6)); console.log(globalThis[\"Math\"].sign(3 - 6)); console.log(globalThis[\"Math\"].imul(2147483647, 2)); console.log(globalThis[\"Math\"].clz32(1)); });\n",
            "3\n1\n3\n-1\n-2\n31",
        ),
        (
            "run",
            "main.jsx",
            "console.log(globalThis[\"Math\"].max(1, 2, 3));\nconsole.log(globalThis[\"Math\"].min(3, 2, 1));\nconsole.log(globalThis[\"Math\"].abs(3 - 6));\nconsole.log(globalThis[\"Math\"].sign(3 - 6));\nconsole.log(globalThis[\"Math\"].imul(2147483647, 2));\nconsole.log(globalThis[\"Math\"].clz32(1));\n",
            "3\n1\n3\n-1\n-2\n31",
        ),
        (
            "test",
            "smoke.test.jsx",
            "Kali.test('bracketed math core suite', () => { console.log(globalThis[\"Math\"].max(1, 2, 3)); console.log(globalThis[\"Math\"].min(3, 2, 1)); console.log(globalThis[\"Math\"].abs(3 - 6)); console.log(globalThis[\"Math\"].sign(3 - 6)); console.log(globalThis[\"Math\"].imul(2147483647, 2)); console.log(globalThis[\"Math\"].clz32(1)); });\n",
            "3\n1\n3\n-1\n-2\n31",
        ),
        (
            "run",
            "main.tsx",
            "console.log(globalThis[\"Math\"].max(1, 2, 3));\nconsole.log(globalThis[\"Math\"].min(3, 2, 1));\nconsole.log(globalThis[\"Math\"].abs(3 - 6));\nconsole.log(globalThis[\"Math\"].sign(3 - 6));\nconsole.log(globalThis[\"Math\"].imul(2147483647, 2));\nconsole.log(globalThis[\"Math\"].clz32(1));\n",
            "3\n1\n3\n-1\n-2\n31",
        ),
        (
            "test",
            "smoke.test.tsx",
            "Kali.test('bracketed math core suite', () => { console.log(globalThis[\"Math\"].max(1, 2, 3)); console.log(globalThis[\"Math\"].min(3, 2, 1)); console.log(globalThis[\"Math\"].abs(3 - 6)); console.log(globalThis[\"Math\"].sign(3 - 6)); console.log(globalThis[\"Math\"].imul(2147483647, 2)); console.log(globalThis[\"Math\"].clz32(1)); });\n",
            "3\n1\n3\n-1\n-2\n31",
        ),
    ] {
        for output_json in [false, true] {
            let dir = tempdir().expect("tempdir");
            let source_path = dir.path().join(source_name);
            fs::write(&source_path, source).expect("write source");

            let mut output = Command::new(kali_bin());
            output
                .env(kali_runtime::BROWSER_HARNESS_COMMAND_ENV, "node")
                .current_dir(dir.path());
            if output_json {
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
            if output_json {
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
                    json["stdout"].as_str().expect("stdout").contains(expected_stdout),
                    "json: {json}"
                );
                assert_eq!(json["stderr"], "");
            } else {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains(expected_stdout), "stdout: {stdout}");
            }
        }
    }
}
