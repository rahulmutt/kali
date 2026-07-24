use std::{fs, process::Command};

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

fn template_literal_dynamic_import_sequence_test_source(chunk_filename: &str) -> String {
    format!(
        r#"async function main() {{
  const name = "{chunk_filename}";
  const chunk = await import((0, `./${{name}}`));
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
Kali.test('template literal dynamic import sequence', () => {{}});
"#,
    )
}

fn template_literal_dynamic_import_sequence_run_source(chunk_filename: &str) -> String {
    format!(
        r#"async function main() {{
  const name = "{chunk_filename}";
  const chunk = await import((0, `./${{name}}`));
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
Kali.test('template literal dynamic import sequence', () => {{}});
"#,
    )
}

fn object_freeze_literal_dynamic_import_wrappers_source(chunk_filename: &str) -> String {
    format!(
        r#"async function main() {{
  const nullishChunk = await import(Object.freeze((null ?? "./{chunk_filename}")));
  if (typeof nullishChunk.lazyValue !== 'function') {{
    throw new Error('missing lazyValue export');
  }}
  const nullishValue = await nullishChunk.lazyValue();
  if (nullishValue !== 0n) {{
    throw new Error(`unexpected nullish chunk result ${{nullishValue}}`);
  }}
  console.log(String(nullishValue));
  const andChunk = await import(Object.freeze((true && "./{chunk_filename}")));
  if (typeof andChunk.lazyValue !== 'function') {{
    throw new Error('missing lazyValue export');
  }}
  const andValue = await andChunk.lazyValue();
  if (andValue !== 0n) {{
    throw new Error(`unexpected and chunk result ${{andValue}}`);
  }}
  console.log(String(andValue));
  const orChunk = await import(Object.freeze((false || "./{chunk_filename}")));
  if (typeof orChunk.lazyValue !== 'function') {{
    throw new Error('missing lazyValue export');
  }}
  const orValue = await orChunk.lazyValue();
  if (orValue !== 0n) {{
    throw new Error(`unexpected or chunk result ${{orValue}}`);
  }}
  console.log(String(orValue));
  console.log('main loaded');
}}
main();
Kali.test('object.freeze logical literal dynamic import', () => {{}});
"#
    )
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

    // Stage P5 Task 5 reconciliation (was Task A2b fail-closed pin). `String()` is
    // now a real runtime coercion (String de-denylisted + routed through the
    // `emit_as_string` ladder), so the pre-A2b behavior is restored HONESTLY:
    // every fixture routed through this helper does `console.log(String(value))`
    // (the freeze variant does three such lines) where `value` is the imported
    // `0n`. `String(0n)` -> "0" on both kali and node (referee: node v26.5.0), so
    // the browser-harness run/test SUCCEEDS with node-correct stdout. Derive the
    // expected program stdout FROM the fixture — one `0\n` per `console.log(String(`
    // site, then `main loaded\n` — so the assertion is exact and a wrong-reason
    // pass on a stray substring (e.g. a surviving deny that happens to print "0")
    // is not available. In `--output json` mode the program's own stdout is the
    // clean `stdout` JSON field (the TAP `ok 1` test summary stays out of it);
    // for a non-json `test` run the TAP summary trails the program output.
    let string_log_count = source.matches("console.log(String(").count();
    let mut expected_stdout = "0\n".repeat(string_log_count);
    expected_stdout.push_str("main loaded\n");

    assert!(
        output.status.success(),
        "expected success (String() now lowers), got failure. stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if json_output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: serde_json::Value =
            serde_json::from_str(&stdout).expect("parse kali json stdout");
        assert_eq!(json["command"], command, "json: {json}");
        assert_eq!(json["success"], true, "json: {json}");
        assert_eq!(
            json["payload"]["hostContract"], "browser-requested",
            "json: {json}"
        );
        assert_eq!(
            json["payload"]["runtimeBackend"], "browser-harness",
            "json: {json}"
        );
        if expect_test_runner {
            assert_eq!(json["payload"]["passed"], 1, "json: {json}");
            assert_eq!(json["payload"]["failed"], 0, "json: {json}");
        }
        assert_eq!(
            json["stdout"].as_str().expect("json stdout string"),
            expected_stdout,
            "json: {json}"
        );
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        if expect_test_runner {
            assert!(
                stdout.starts_with(&expected_stdout),
                "expected stdout to start with {expected_stdout:?}, got: {stdout:?} (stderr: {})",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(
                stdout.contains("ok 1") && !stdout.contains("not ok"),
                "expected a passing TAP summary, got: {stdout:?}"
            );
        } else {
            assert_eq!(
                stdout,
                expected_stdout,
                "stderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
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

#[test]
fn run_supports_sequence_wrapped_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_requested_template_literal_dynamic_import(
        "run",
        "main.js",
        "lazy.js",
        &template_literal_dynamic_import_sequence_run_source("lazy.js"),
        false,
        false,
    );
}

#[test]
fn test_supports_sequence_wrapped_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_requested_template_literal_dynamic_import(
        "test",
        "smoke.test.js",
        "lazy.js",
        &template_literal_dynamic_import_sequence_test_source("lazy.js"),
        false,
        true,
    );
}

#[test]
fn run_supports_sequence_wrapped_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_ts_jsx_tsx_input(
) {
    for extension in ["ts", "jsx", "tsx"] {
        assert_browser_requested_template_literal_dynamic_import(
            "run",
            &format!("main.{extension}"),
            &format!("lazy.{extension}"),
            &template_literal_dynamic_import_sequence_run_source(&format!("lazy.{extension}")),
            false,
            false,
        );
        assert_browser_requested_template_literal_dynamic_import(
            "run",
            &format!("main.{extension}"),
            &format!("lazy.{extension}"),
            &template_literal_dynamic_import_sequence_run_source(&format!("lazy.{extension}")),
            true,
            false,
        );
    }
}

#[test]
fn test_supports_sequence_wrapped_template_literal_dynamic_import_targets_in_browser_api_surface_with_harness_ts_jsx_tsx_input(
) {
    for extension in ["ts", "jsx", "tsx"] {
        assert_browser_requested_template_literal_dynamic_import(
            "test",
            &format!("smoke.test.{extension}"),
            &format!("lazy.{extension}"),
            &template_literal_dynamic_import_sequence_test_source(&format!("lazy.{extension}")),
            false,
            true,
        );
        assert_browser_requested_template_literal_dynamic_import(
            "test",
            &format!("smoke.test.{extension}"),
            &format!("lazy.{extension}"),
            &template_literal_dynamic_import_sequence_test_source(&format!("lazy.{extension}")),
            true,
            true,
        );
    }
}

#[test]
fn run_supports_object_freeze_wrapped_literal_dynamic_import_targets_with_logical_wrappers_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_requested_template_literal_dynamic_import(
        "run",
        "main.js",
        "lazy.js",
        &object_freeze_literal_dynamic_import_wrappers_source("lazy.js"),
        false,
        false,
    );
}

#[test]
fn json_run_supports_object_freeze_wrapped_literal_dynamic_import_targets_with_logical_wrappers_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_requested_template_literal_dynamic_import(
        "run",
        "main.js",
        "lazy.js",
        &object_freeze_literal_dynamic_import_wrappers_source("lazy.js"),
        true,
        false,
    );
}

#[test]
fn test_supports_object_freeze_wrapped_literal_dynamic_import_targets_with_logical_wrappers_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_requested_template_literal_dynamic_import(
        "test",
        "smoke.test.js",
        "lazy.js",
        &object_freeze_literal_dynamic_import_wrappers_source("lazy.js"),
        false,
        true,
    );
}

#[test]
fn json_test_supports_object_freeze_wrapped_literal_dynamic_import_targets_with_logical_wrappers_in_browser_api_surface_with_harness_js_input(
) {
    assert_browser_requested_template_literal_dynamic_import(
        "test",
        "smoke.test.js",
        "lazy.js",
        &object_freeze_literal_dynamic_import_wrappers_source("lazy.js"),
        true,
        true,
    );
}

#[test]
fn run_supports_object_freeze_wrapped_literal_dynamic_import_targets_with_logical_wrappers_in_browser_api_surface_with_harness_ts_jsx_tsx_input(
) {
    for extension in ["ts", "jsx", "tsx"] {
        let source_filename = format!("main.{extension}");
        let chunk_filename = format!("lazy.{extension}");
        let source = object_freeze_literal_dynamic_import_wrappers_source(&chunk_filename);
        assert_browser_requested_template_literal_dynamic_import(
            "run",
            &source_filename,
            &chunk_filename,
            &source,
            false,
            false,
        );
        assert_browser_requested_template_literal_dynamic_import(
            "run",
            &source_filename,
            &chunk_filename,
            &source,
            true,
            false,
        );
    }
}

#[test]
fn test_supports_object_freeze_wrapped_literal_dynamic_import_targets_with_logical_wrappers_in_browser_api_surface_with_harness_ts_jsx_tsx_input(
) {
    for extension in ["ts", "jsx", "tsx"] {
        let source_filename = format!("smoke.test.{extension}");
        let chunk_filename = format!("lazy.{extension}");
        let source = object_freeze_literal_dynamic_import_wrappers_source(&chunk_filename);
        assert_browser_requested_template_literal_dynamic_import(
            "test",
            &source_filename,
            &chunk_filename,
            &source,
            false,
            true,
        );
        assert_browser_requested_template_literal_dynamic_import(
            "test",
            &source_filename,
            &chunk_filename,
            &source,
            true,
            true,
        );
    }
}
