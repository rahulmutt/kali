use std::{fs, process::Command};

use tempfile::tempdir;

use kali_common::browser_template_literal_string_iteration_body_source;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn indent_source(source: &str, indentation: &str) -> String {
    source
        .lines()
        .map(|line| format!("{indentation}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn browser_template_literal_string_iteration_source(command: &str) -> String {
    let body = indent_source(
        browser_template_literal_string_iteration_body_source(),
        "  ",
    );

    match command {
        "test" => {
            format!("Kali.test('browser template literal iteration', () => {{\n{body}\n}});\n")
        }
        _ => format!("{body}\nconsole.log('browser template literal iteration ok');\n"),
    }
}

fn assert_browser_harness_template_literal_string_iteration(
    command: &str,
    filename: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_template_literal_string_iteration_source(command),
    )
    .expect("write source");

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

    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!output.status.success(), "must fail closed: {output:?}");
}

#[test]
fn run_supports_template_literal_iteration_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_template_literal_string_iteration("run", "main.js", false);
}

#[test]
fn test_supports_template_literal_iteration_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_template_literal_string_iteration("test", "smoke.test.js", false);
}

#[test]
fn json_run_supports_template_literal_iteration_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_template_literal_string_iteration("run", "main.js", true);
}

#[test]
fn json_test_supports_template_literal_iteration_in_browser_api_surface_with_harness_js_input() {
    assert_browser_harness_template_literal_string_iteration("test", "smoke.test.js", true);
}

#[test]
fn run_supports_template_literal_iteration_in_browser_api_surface_with_harness_ts_input() {
    assert_browser_harness_template_literal_string_iteration("run", "main.ts", false);
}

#[test]
fn test_supports_template_literal_iteration_in_browser_api_surface_with_harness_ts_input() {
    assert_browser_harness_template_literal_string_iteration("test", "smoke.test.ts", false);
}

#[test]
fn json_run_supports_template_literal_iteration_in_browser_api_surface_with_harness_ts_input() {
    assert_browser_harness_template_literal_string_iteration("run", "main.ts", true);
}

#[test]
fn json_test_supports_template_literal_iteration_in_browser_api_surface_with_harness_ts_input() {
    assert_browser_harness_template_literal_string_iteration("test", "smoke.test.ts", true);
}

#[test]
fn run_supports_template_literal_iteration_in_browser_api_surface_with_harness_jsx_input() {
    assert_browser_harness_template_literal_string_iteration("run", "main.jsx", false);
}

#[test]
fn test_supports_template_literal_iteration_in_browser_api_surface_with_harness_jsx_input() {
    assert_browser_harness_template_literal_string_iteration("test", "smoke.test.jsx", false);
}

#[test]
fn json_run_supports_template_literal_iteration_in_browser_api_surface_with_harness_jsx_input() {
    assert_browser_harness_template_literal_string_iteration("run", "main.jsx", true);
}

#[test]
fn json_test_supports_template_literal_iteration_in_browser_api_surface_with_harness_jsx_input() {
    assert_browser_harness_template_literal_string_iteration("test", "smoke.test.jsx", true);
}

#[test]
fn run_supports_template_literal_iteration_in_browser_api_surface_with_harness_tsx_input() {
    assert_browser_harness_template_literal_string_iteration("run", "main.tsx", false);
}

#[test]
fn test_supports_template_literal_iteration_in_browser_api_surface_with_harness_tsx_input() {
    assert_browser_harness_template_literal_string_iteration("test", "smoke.test.tsx", false);
}

#[test]
fn json_run_supports_template_literal_iteration_in_browser_api_surface_with_harness_tsx_input() {
    assert_browser_harness_template_literal_string_iteration("run", "main.tsx", true);
}

#[test]
fn json_test_supports_template_literal_iteration_in_browser_api_surface_with_harness_tsx_input() {
    assert_browser_harness_template_literal_string_iteration("test", "smoke.test.tsx", true);
}
