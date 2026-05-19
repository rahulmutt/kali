use std::{fs, process::Command};

use kali_common::array_from_frozen_callable_source;
use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn array_from_frozen_loop_lines(loop_header: &str, indentation: &str) -> String {
    array_from_frozen_callable_source()
        .trim_end_matches(';')
        .split("; ")
        .map(|alias| {
            format!(
                "{indentation}{loop_header}{alias}(values) {{\n{indentation}  console.log(value);\n{indentation}}}"
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_for_of_array_iteration_spread(
    command: &str,
    filename: &str,
    source: &str,
    expected: &str,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg(command)
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
    assert!(stdout.contains(expected), "stdout: {stdout}");
}

fn browser_harness_array_from_source(command: &str) -> String {
    let frozen_for_of = array_from_frozen_loop_lines("for (const value of ", "");
    let frozen_for_await = array_from_frozen_loop_lines("for await (const value of ", "");
    let body = format!(
        r#"const values = [1, 2];
for (const value of Array.from(values)) {{
  console.log(value);
}}
for (const value of globalThis.Array.from(values)) {{
  console.log(value);
}}
for (const value of globalThis["Array"].from(values)) {{
  console.log(value);
}}
for (const value of globalThis["Array"]["from"](values)) {{
  console.log(value);
}}
{frozen_for_of}
for await (const value of Array.from(values)) {{
  console.log(value);
}}
for await (const value of globalThis.Array.from(values)) {{
  console.log(value);
}}
for await (const value of globalThis["Array"].from(values)) {{
  console.log(value);
}}
for await (const value of globalThis["Array"]["from"](values)) {{
  console.log(value);
}}
{frozen_for_await}
"#
    );

    match command {
        "test" => format!(
            "Kali.test('browser Array.from wrappers', () => {{
  async function browserArrayFromWrappers() {{
{body}  }}
  return browserArrayFromWrappers();
}});
"
        ),
        _ => body,
    }
}

fn assert_browser_harness_array_from_iteration_spread(
    command: &str,
    filename: &str,
    json_output: bool,
    source: impl AsRef<str>,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source.as_ref()).expect("write source");

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

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        assert_eq!(json["payload"]["threadTopology"]["totalInstances"], 0);
        assert_eq!(json["payload"]["threadTopology"]["terminatedInstances"], 0);
        assert_eq!(
            json["payload"]["threadTopology"]["liveInstances"],
            serde_json::json!([])
        );
        if command == "run" {
            assert_eq!(json["exitCode"], 0);
            assert_eq!(json["payload"]["exitCode"], 0);
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
        }
        let stdout = json["stdout"].as_str().expect("stdout string");
        assert!(stdout.contains("1"), "json: {json}");
        assert!(stdout.contains("2"), "json: {json}");
        assert_eq!(json["stderr"], "");
        assert!(json["errors"].as_array().expect("errors array").is_empty());
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("1"), "stdout: {stdout}");
        assert!(stdout.contains("2"), "stdout: {stdout}");
        if command == "test" {
            assert!(stdout.contains("ok 1"), "stdout: {stdout}");
        }
    }
}

#[test]
fn run_supports_for_of_array_iteration_spread_in_js_input() {
    assert_for_of_array_iteration_spread(
        "run",
        "main.js",
        "const values = [1, 2]; for (const item of [...values]) { console.log(item); }\n",
        "1\n2\n",
    );
}

#[test]
fn run_supports_for_of_array_iteration_spread_in_ts_input() {
    assert_for_of_array_iteration_spread(
        "run",
        "main.ts",
        "const values = [1, 2]; for (const item of [...values]) { console.log(item); }\n",
        "1\n2\n",
    );
}

#[test]
fn test_supports_for_of_array_iteration_spread_in_js_input() {
    assert_for_of_array_iteration_spread(
        "test",
        "smoke.test.js",
        "Kali.test('for-of spread', () => { const values = [1, 2]; for (const item of [...values]) { console.log(item); } });\n",
        "ok 1",
    );
}

#[test]
fn test_supports_for_of_array_iteration_spread_in_ts_input() {
    assert_for_of_array_iteration_spread(
        "test",
        "smoke.test.ts",
        "Kali.test('for-of spread', () => { const values = [1, 2]; for (const item of [...values]) { console.log(item); } });\n",
        "ok 1",
    );
}

#[test]
fn run_supports_for_of_array_iteration_spread_in_jsx_and_tsx_input() {
    for filename in ["main.jsx", "main.tsx"] {
        assert_for_of_array_iteration_spread(
            "run",
            filename,
            "const values = [1, 2]; for (const item of [...values]) { console.log(item); }\n",
            "1\n2\n",
        );
    }
}

#[test]
fn test_supports_for_of_array_iteration_spread_in_jsx_and_tsx_input() {
    for filename in ["smoke.test.jsx", "smoke.test.tsx"] {
        assert_for_of_array_iteration_spread(
            "test",
            filename,
            "Kali.test('for-of spread', () => { const values = [1, 2]; for (const item of [...values]) { console.log(item); } });\n",
            "ok 1",
        );
    }
}

#[test]
fn run_supports_for_of_break_and_continue_in_js_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_for_of_array_iteration_spread(
            "run",
            filename,
            "const values = [0, 1, 1]; for (const value of values) { if (!value) continue; console.log(value); if (value) break; }\n",
            "1\n",
        );
    }
}

#[test]
fn test_supports_for_of_break_and_continue_in_js_input() {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_for_of_array_iteration_spread(
            "test",
            filename,
            "Kali.test('for-of break/continue', () => { const values = [0, 1, 1]; for (const value of values) { if (!value) continue; console.log(value); if (value) break; } });\n",
            "ok 1",
        );
    }
}

#[test]
fn run_supports_for_await_break_and_continue_in_js_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_for_of_array_iteration_spread(
            "run",
            filename,
            "const values = [0, 1, 1]; for await (const value of values) { if (!value) continue; console.log(value); if (value) break; }\n",
            "1\n",
        );
    }
}

#[test]
fn test_supports_for_await_break_and_continue_in_js_input() {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_for_of_array_iteration_spread(
            "test",
            filename,
            "Kali.test('for-await break/continue', () => { const values = [0, 1, 1]; for await (const value of values) { if (!value) continue; console.log(value); if (value) break; } });\n",
            "ok 1",
        );
    }
}

#[test]
fn run_supports_for_of_array_from_iteration_in_js_ts_jsx_and_tsx_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_for_of_array_iteration_spread(
            "run",
            filename,
            "const values = [1, 2]; for (const value of Array.from(values)) { console.log(value); } for (const value of globalThis.Array.from(values)) { console.log(value); } for (const value of Object.freeze(Array.from)(values)) { console.log(value); } for (const value of Object.freeze((Array.from))(values)) { console.log(value); } for (const value of Object.freeze(globalThis.Array.from)(values)) { console.log(value); } for (const value of Object.freeze((globalThis.Array.from))(values)) { console.log(value); } for await (const value of Array.from(values)) { console.log(value); } for await (const value of globalThis.Array.from(values)) { console.log(value); } for await (const value of Object.freeze(Array.from)(values)) { console.log(value); } for await (const value of Object.freeze((Array.from))(values)) { console.log(value); } for await (const value of Object.freeze((globalThis.Array.from))(values)) { console.log(value); }\n",
            "1\n2\n",
        );
    }
}

#[test]
fn test_supports_for_of_array_from_iteration_in_js_ts_jsx_and_tsx_input() {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_for_of_array_iteration_spread(
            "test",
            filename,
            "Kali.test('for-of Array.from', () => { const values = [1, 2]; for (const value of Array.from(values)) { console.log(value); } for (const value of globalThis.Array.from(values)) { console.log(value); } for (const value of Object.freeze(Array.from)(values)) { console.log(value); } for (const value of Object.freeze((Array.from))(values)) { console.log(value); } for (const value of Object.freeze(globalThis.Array.from)(values)) { console.log(value); } for (const value of Object.freeze((globalThis.Array.from))(values)) { console.log(value); } for await (const value of Array.from(values)) { console.log(value); } for await (const value of globalThis.Array.from(values)) { console.log(value); } for await (const value of Object.freeze(Array.from)(values)) { console.log(value); } for await (const value of Object.freeze((Array.from))(values)) { console.log(value); } for await (const value of Object.freeze((globalThis.Array.from))(values)) { console.log(value); } });\n",
            "ok 1",
        );
    }
}

#[test]
fn run_supports_frozen_array_from_iteration_in_js_ts_jsx_and_tsx_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_for_of_array_iteration_spread(
            "run",
            filename,
            "const values = [1, 2]; for (const value of Object.freeze(Array.from)(values)) { console.log(value); }\n",
            "1\n2\n",
        );
    }
}

#[test]
fn test_supports_frozen_array_from_iteration_in_js_ts_jsx_and_tsx_input() {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_for_of_array_iteration_spread(
            "test",
            filename,
            "Kali.test('for-of frozen Array.from', () => { const values = [1, 2]; for (const value of Object.freeze(Array.from)(values)) { console.log(value); } });\n",
            "ok 1",
        );
    }
}

#[test]
fn run_supports_for_await_array_from_iteration_in_js_ts_jsx_and_tsx_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_for_of_array_iteration_spread(
            "run",
            filename,
            "const values = [1, 2]; for await (const value of Array.from(values)) { console.log(value); }\n",
            "1\n2\n",
        );
    }
}

#[test]
fn test_supports_for_await_array_from_iteration_in_js_ts_jsx_and_tsx_input() {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_for_of_array_iteration_spread(
            "test",
            filename,
            "Kali.test('for-await Array.from', () => { const values = [1, 2]; for await (const value of Array.from(values)) { console.log(value); } });\n",
            "ok 1",
        );
    }
}

#[test]
fn run_supports_browser_harness_for_of_array_from_iteration_in_js_input() {
    assert_browser_harness_array_from_iteration_spread(
        "run",
        "main.js",
        false,
        browser_harness_array_from_source("run"),
    );
}

#[test]
fn run_supports_browser_harness_for_of_array_from_iteration_in_ts_input() {
    assert_browser_harness_array_from_iteration_spread(
        "run",
        "main.ts",
        false,
        browser_harness_array_from_source("run"),
    );
}

#[test]
fn run_supports_browser_harness_for_of_array_from_iteration_in_jsx_and_tsx_input() {
    for filename in ["main.jsx", "main.tsx"] {
        assert_browser_harness_array_from_iteration_spread(
            "run",
            filename,
            false,
            browser_harness_array_from_source("run"),
        );
    }
}

#[test]
fn json_run_supports_browser_harness_for_of_array_from_iteration_in_js_ts_jsx_and_tsx_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_browser_harness_array_from_iteration_spread(
            "run",
            filename,
            true,
            browser_harness_array_from_source("run"),
        );
    }
}

#[test]
fn test_supports_browser_harness_for_of_array_from_iteration_in_js_input() {
    assert_browser_harness_array_from_iteration_spread(
        "test",
        "smoke.test.js",
        false,
        browser_harness_array_from_source("test"),
    );
}

#[test]
fn test_supports_browser_harness_for_of_array_from_iteration_in_ts_input() {
    assert_browser_harness_array_from_iteration_spread(
        "test",
        "smoke.test.ts",
        false,
        browser_harness_array_from_source("test"),
    );
}

#[test]
fn test_supports_browser_harness_for_of_array_from_iteration_in_jsx_and_tsx_input() {
    for filename in ["smoke.test.jsx", "smoke.test.tsx"] {
        assert_browser_harness_array_from_iteration_spread(
            "test",
            filename,
            false,
            browser_harness_array_from_source("test"),
        );
    }
}

#[test]
fn json_test_supports_browser_harness_for_of_array_from_iteration_in_js_ts_jsx_and_tsx_input() {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_browser_harness_array_from_iteration_spread(
            "test",
            filename,
            true,
            browser_harness_array_from_source("test"),
        );
    }
}
