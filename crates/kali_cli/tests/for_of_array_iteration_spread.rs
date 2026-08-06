use std::{fs, process::Command};

use kali_common::{array_from_alias_inventory_source, array_from_loop_lines};
use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
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

fn array_from_iteration_body() -> String {
    let array_from_source = array_from_alias_inventory_source();
    let frozen_for_of = array_from_loop_lines(&array_from_source, "for (const value of ", "");
    let frozen_for_await =
        array_from_loop_lines(&array_from_source, "for await (const value of ", "");
    format!(
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
    )
}

fn browser_harness_array_from_source(command: &str) -> String {
    let body = array_from_iteration_body();

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

fn array_from_set_map_break_continue_body() -> &'static str {
    r#"  const setValues = [0, 1, 1];
  const setItems = [];
  for (const value of Array.from(new Set(setValues))) {
    if (!value) {
      continue;
    }
    setItems.push(value);
    break;
  }
  if (setItems.length !== 1 || setItems[0] !== 1) {
    throw new Error('unexpected Array.from(new Set(...)) break/continue semantics');
  }

  let setReturnFinally = false;
  async function setReturnProbe() {
    try {
      for (const value of Array.from(new Set(setValues))) {
        return value;
      }
      throw new Error('unexpected empty Array.from(new Set(...)) iteration');
    } finally {
      setReturnFinally = true;
    }
  }
  const setReturnValue = await setReturnProbe();
  if (setReturnValue !== 1 || !setReturnFinally) {
    throw new Error('unexpected Array.from(new Set(...)) return/finally semantics');
  }

  let setThrowFinally = false;
  async function setThrowProbe() {
    try {
      for (const value of Array.from(new Set(setValues))) {
        if (value === 1) {
          throw new Error('boom');
        }
      }
      throw new Error('unexpected empty Array.from(new Set(...)) iteration');
    } finally {
      setThrowFinally = true;
    }
  }
  let setThrew = false;
  try {
    await setThrowProbe();
  } catch {
    setThrew = true;
  }
  if (!setThrew || !setThrowFinally) {
    throw new Error('unexpected Array.from(new Set(...)) throw/finally semantics');
  }

  const mapItems = [];
  for await (const entry of Array.from(new Map([[0, 1], [1, 3], [4, 5]]))) {
    if (!entry[0]) {
      continue;
    }
    mapItems.push(entry[0]);
    mapItems.push(entry[1]);
    break;
  }
  if (mapItems.length !== 2 || mapItems[0] !== 1 || mapItems[1] !== 3) {
    throw new Error('unexpected Array.from(new Map(...)) break/continue semantics');
  }

  console.log(1);
  console.log(2);
  console.log('array.from set/map break/continue ok');
"#
}

fn browser_harness_array_from_set_map_break_continue_source(command: &str) -> String {
    let body = array_from_set_map_break_continue_body();
    match command {
        "test" => format!(
            "Kali.test('browser Array.from set/map break/continue', () => {{
  async function browserArrayFromSetMapBreakContinue() {{
{body}  }}
  return browserArrayFromSetMapBreakContinue();
}});
"
        ),
        _ => format!(
            "async function browserArrayFromSetMapBreakContinue() {{
{body}}}

browserArrayFromSetMapBreakContinue();
"
        ),
    }
}

#[test]
fn browser_harness_test_wrapper_reuses_the_shared_array_from_inventory_in_both_loop_sections() {
    let source = browser_harness_array_from_source("test");

    for alias in [
        r#"Object.freeze((Array.from))"#,
        r#"Object.freeze((globalThis.Array.from))"#,
        r#"Object.freeze((globalThis["Array"].from))"#,
        r#"Object.freeze((globalThis["Array"]["from"]))"#,
        r#"Object.freeze((globalThis["Array"])["from"])"#,
        r#"Object.freeze((globalThis['Array']).from)"#,
        r#"Object.freeze((globalThis['Array'])["from"])"#,
        r#"Object.freeze((globalThis.Array).from)"#,
        r#"Object.freeze((globalThis.Array)["from"])"#,
        r#"Object.freeze((globalThis.Array))["from"]"#,
        r#"Object.freeze((globalThis.Array)['from'])"#,
        r#"Object.freeze((null ?? Array.from))"#,
        r#"Object.freeze((true && Array.from))"#,
        r#"Object.freeze((false || Array.from))"#,
        r#"Object.freeze((null ?? globalThis["Array"].from))"#,
        r#"Object.freeze((true && globalThis["Array"].from))"#,
        r#"Object.freeze((false || globalThis["Array"].from))"#,
        r#"Object.freeze((null ?? globalThis["Array"]["from"]))"#,
        r#"Object.freeze((true && globalThis["Array"]["from"]))"#,
        r#"Object.freeze((false || globalThis["Array"]["from"]))"#,
        r#"Object.freeze((null ?? globalThis['Array']['from']))"#,
        r#"Object.freeze((true && globalThis['Array']['from']))"#,
        r#"Object.freeze((false || globalThis['Array']['from']))"#,
    ] {
        assert_eq!(source.matches(alias).count(), 2, "alias {alias}: {source}");
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
    cli.env(kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV, "node")
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

// Honest re-pin (PR #16 rev2): kali fails closed/loud here;
// see docs/superpowers/followups/pr16-honest-repin-inventory.md.
// Batch-local variant of `assert_for_of_array_iteration_spread` (PR #16
// batch 6) — that shared helper still has green out-of-batch callers in
// this file, so it is left untouched; this variant is scoped to the
// array-from-new-Set/new-Map break/continue members only.
fn assert_for_of_array_iteration_spread_fails_closed(command: &str, filename: &str, source: &str) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg(command)
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(!output.status.success(), "must fail closed: {output:?}");
}

// Honest re-pin (PR #16 rev2): kali fails closed/loud here;
// see docs/superpowers/followups/pr16-honest-repin-inventory.md.
// Batch-local variant of `assert_browser_harness_array_from_iteration_spread`
// (PR #16 batch 6) — that shared helper still has green out-of-batch callers
// in this file, so it is left untouched; this variant is scoped to the
// array-from-set/map break/continue members only.
fn assert_browser_harness_array_from_iteration_spread_fails_closed(
    command: &str,
    filename: &str,
    json_output: bool,
    source: impl AsRef<str>,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source.as_ref()).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.env(kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV, "node")
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

    assert!(!output.status.success(), "must fail closed: {output:?}");
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
        let body = array_from_iteration_body();
        assert_for_of_array_iteration_spread("run", filename, &body, "1\n2\n");
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
            &browser_harness_array_from_source("test"),
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

#[test]
fn run_supports_browser_harness_array_from_set_map_break_continue_in_js_input() {
    assert_browser_harness_array_from_iteration_spread_fails_closed(
        "run",
        "main.js",
        false,
        browser_harness_array_from_set_map_break_continue_source("run"),
    );
}

#[test]
fn run_supports_browser_harness_array_from_set_map_break_continue_in_ts_input() {
    assert_browser_harness_array_from_iteration_spread_fails_closed(
        "run",
        "main.ts",
        false,
        browser_harness_array_from_set_map_break_continue_source("run"),
    );
}

#[test]
fn run_supports_browser_harness_array_from_set_map_break_continue_in_jsx_and_tsx_input() {
    for filename in ["main.jsx", "main.tsx"] {
        assert_browser_harness_array_from_iteration_spread_fails_closed(
            "run",
            filename,
            false,
            browser_harness_array_from_set_map_break_continue_source("run"),
        );
    }
}

#[test]
fn json_run_supports_browser_harness_array_from_set_map_break_continue_in_js_ts_jsx_and_tsx_input()
{
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_browser_harness_array_from_iteration_spread_fails_closed(
            "run",
            filename,
            true,
            browser_harness_array_from_set_map_break_continue_source("run"),
        );
    }
}

#[test]
fn test_supports_browser_harness_array_from_set_map_break_continue_in_js_input() {
    assert_browser_harness_array_from_iteration_spread_fails_closed(
        "test",
        "smoke.test.js",
        false,
        browser_harness_array_from_set_map_break_continue_source("test"),
    );
}

#[test]
fn test_supports_browser_harness_array_from_set_map_break_continue_in_ts_input() {
    assert_browser_harness_array_from_iteration_spread_fails_closed(
        "test",
        "smoke.test.ts",
        false,
        browser_harness_array_from_set_map_break_continue_source("test"),
    );
}

#[test]
fn test_supports_browser_harness_array_from_set_map_break_continue_in_jsx_and_tsx_input() {
    for filename in ["smoke.test.jsx", "smoke.test.tsx"] {
        assert_browser_harness_array_from_iteration_spread_fails_closed(
            "test",
            filename,
            false,
            browser_harness_array_from_set_map_break_continue_source("test"),
        );
    }
}

#[test]
fn json_test_supports_browser_harness_array_from_set_map_break_continue_in_js_ts_jsx_and_tsx_input()
{
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_browser_harness_array_from_iteration_spread_fails_closed(
            "test",
            filename,
            true,
            browser_harness_array_from_set_map_break_continue_source("test"),
        );
    }
}

#[test]
fn run_supports_array_from_new_set_and_new_map_break_continue_in_js_input() {
    for filename in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
        assert_for_of_array_iteration_spread_fails_closed(
            "run",
            filename,
            &browser_harness_array_from_set_map_break_continue_source("run"),
        );
    }
}

#[test]
fn test_supports_array_from_new_set_and_new_map_break_continue_in_js_input() {
    for filename in [
        "smoke.test.js",
        "smoke.test.ts",
        "smoke.test.jsx",
        "smoke.test.tsx",
    ] {
        assert_for_of_array_iteration_spread_fails_closed(
            "test",
            filename,
            &browser_harness_array_from_set_map_break_continue_source("test"),
        );
    }
}
