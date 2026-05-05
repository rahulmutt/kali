use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn template_literal_dynamic_import_run_source(chunk_filename: &str) -> String {
    format!(
        r#"async function main() {{
  const name = "{chunk_filename}";
  const chunk = await import(`./${{name}}`);
  if (typeof chunk.lazyValue !== 'function') {{
    throw new Error('missing lazyValue export');
  }}
  const value = await chunk.lazyValue();
  if (value !== 0n) {{
    throw new Error(`unexpected chunk result ${{value}}`);
  }}
  console.log(String(value));
  console.log('main loaded');
}}
main();
Kali.test('template literal dynamic import', () => {{}});
"#,
    )
}

fn template_literal_dynamic_import_test_source(chunk_filename: &str) -> String {
    format!(
        r#"async function main() {{
  const name = "{chunk_filename}";
  const chunk = await import(`./${{name}}`);
  if (typeof chunk.lazyValue !== 'function') {{
    throw new Error('missing lazyValue export');
  }}
  const value = await chunk.lazyValue();
  if (value !== 0n) {{
    throw new Error(`unexpected chunk result ${{value}}`);
  }}
  console.log(String(value));
  console.log('main loaded');
}}
main();
Kali.test('template literal dynamic import', () => {{}});
"#,
    )
}

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("valid json stdout")
}

fn assert_browser_requested_template_literal_dynamic_import(
    command: &str,
    source_filename: &str,
    chunk_filename: &str,
    source: &str,
    json_output: bool,
    expect_test_runner: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(source_filename);
    fs::write(
        dir.path().join(chunk_filename),
        "export function lazyValue() { return 0n; }",
    )
    .expect("write chunk");
    fs::write(&source_path, source).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg("--max-threads")
        .arg("0")
        .arg("--max-spawned-processes")
        .arg("0");
    if json_output {
        cli.arg("--output").arg("json");
    }
    let output = cli.arg(&source_path).output().expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(0));

    if json_output {
        let json = parse_json_stdout(&output);
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
        let stdout = json["stdout"].as_str().expect("stdout string");
        assert!(stdout.contains("0"), "json: {json}");
        assert!(stdout.contains("main loaded"), "json: {json}");
        if expect_test_runner {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
        } else {
            assert_eq!(json["exitCode"], 0);
            assert_eq!(json["payload"]["exitCode"], 0);
        }
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("0"), "stdout: {stdout}");
        assert!(stdout.contains("main loaded"), "stdout: {stdout}");
        if expect_test_runner {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn run_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_requested_template_literal_dynamic_import(
        "run",
        "main.js",
        "lazy.js",
        &template_literal_dynamic_import_run_source("lazy.js"),
        false,
        false,
    );
}

#[test]
fn json_run_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_requested_template_literal_dynamic_import(
        "run",
        "main.js",
        "lazy.js",
        &template_literal_dynamic_import_run_source("lazy.js"),
        true,
        false,
    );
}

#[test]
fn run_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_jsx_input(
) {
    assert_browser_requested_template_literal_dynamic_import(
        "run",
        "main.jsx",
        "lazy.jsx",
        &template_literal_dynamic_import_run_source("lazy.jsx"),
        false,
        false,
    );
}

#[test]
fn json_run_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_jsx_input(
) {
    assert_browser_requested_template_literal_dynamic_import(
        "run",
        "main.jsx",
        "lazy.jsx",
        &template_literal_dynamic_import_run_source("lazy.jsx"),
        true,
        false,
    );
}

#[test]
fn test_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_ts_input(
) {
    assert_browser_requested_template_literal_dynamic_import(
        "test",
        "smoke.test.ts",
        "lazy.ts",
        &template_literal_dynamic_import_test_source("lazy.ts"),
        false,
        true,
    );
}

#[test]
fn json_test_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_ts_input(
) {
    assert_browser_requested_template_literal_dynamic_import(
        "test",
        "smoke.test.ts",
        "lazy.ts",
        &template_literal_dynamic_import_test_source("lazy.ts"),
        true,
        true,
    );
}

#[test]
fn test_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_tsx_input(
) {
    assert_browser_requested_template_literal_dynamic_import(
        "test",
        "smoke.test.tsx",
        "lazy.tsx",
        &template_literal_dynamic_import_test_source("lazy.tsx"),
        false,
        true,
    );
}

#[test]
fn json_test_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_tsx_input(
) {
    assert_browser_requested_template_literal_dynamic_import(
        "test",
        "smoke.test.tsx",
        "lazy.tsx",
        &template_literal_dynamic_import_test_source("lazy.tsx"),
        true,
        true,
    );
}

#[test]
fn run_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_ts_input(
) {
    assert_browser_requested_template_literal_dynamic_import(
        "run",
        "main.ts",
        "lazy.ts",
        &template_literal_dynamic_import_run_source("lazy.ts"),
        false,
        false,
    );
}

#[test]
fn json_run_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_ts_input(
) {
    assert_browser_requested_template_literal_dynamic_import(
        "run",
        "main.ts",
        "lazy.ts",
        &template_literal_dynamic_import_run_source("lazy.ts"),
        true,
        false,
    );
}

#[test]
fn run_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_tsx_input(
) {
    assert_browser_requested_template_literal_dynamic_import(
        "run",
        "main.tsx",
        "lazy.tsx",
        &template_literal_dynamic_import_run_source("lazy.tsx"),
        false,
        false,
    );
}

#[test]
fn json_run_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_tsx_input(
) {
    assert_browser_requested_template_literal_dynamic_import(
        "run",
        "main.tsx",
        "lazy.tsx",
        &template_literal_dynamic_import_run_source("lazy.tsx"),
        true,
        false,
    );
}

#[test]
fn test_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_requested_template_literal_dynamic_import(
        "test",
        "smoke.test.js",
        "lazy.js",
        &template_literal_dynamic_import_test_source("lazy.js"),
        false,
        true,
    );
}

#[test]
fn json_test_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_requested_template_literal_dynamic_import(
        "test",
        "smoke.test.js",
        "lazy.js",
        &template_literal_dynamic_import_test_source("lazy.js"),
        true,
        true,
    );
}

#[test]
fn test_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_jsx_input(
) {
    assert_browser_requested_template_literal_dynamic_import(
        "test",
        "smoke.test.jsx",
        "lazy.jsx",
        &template_literal_dynamic_import_test_source("lazy.jsx"),
        false,
        true,
    );
}

#[test]
fn json_test_supports_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_jsx_input(
) {
    assert_browser_requested_template_literal_dynamic_import(
        "test",
        "smoke.test.jsx",
        "lazy.jsx",
        &template_literal_dynamic_import_test_source("lazy.jsx"),
        true,
        true,
    );
}
