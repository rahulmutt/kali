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

    // Task A2b fail-closed flip. Every fixture routed through this helper does
    // `console.log(String(value))`. Pre-A2b `String()` silently lowered to 0, so
    // these assertions accepted a stdout containing "0" (the fake-green
    // placeholder, NOT a real coercion) and the package looked deployable.
    // `String` is now in the terminal deny-set, so kali fails the build closed
    // (E5506) rather than miscompiling. node: `String(0n)` -> "0" via a real
    // coercion, but kali has no `String` lowering. In `--output json` mode the
    // diagnostic is emitted as a JSON error on stdout; otherwise on stderr — so
    // check the combined stream.
    let _ = expect_test_runner;
    assert!(
        !output.status.success(),
        "expected fail-closed on String(), got success. stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains("E5506") && combined.contains("String"),
        "expected E5506 for 'String' (json_output={json_output}), got: {combined}"
    );
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
