use crate::*;
use crate::test_support::*;
use std::{fs};


#[test]
fn browser_runtime_harness_page_wraps_the_module_body_for_real_browser_hosts() {
    let page = browser_runtime_harness_page(
        &[0x00, 0x61, 0x73, 0x6d],
        &["alpha".to_string(), "beta".to_string()],
        true,
    );

    assert!(page.starts_with("<!doctype html>"), "page: {page}");
    assert!(page.contains("<script type=\"module\">"), "page: {page}");
    assert!(
        page.contains("const runtimeArgs = [\"alpha\",\"beta\"]"),
        "page: {page}"
    );
    assert!(
        page.contains("const runRegisteredTests = true;"),
        "page: {page}"
    );
    assert!(page.contains("decodeBase64(\""), "page: {page}");
}


#[test]
fn browser_harness_uses_html_entrypoint_for_browser_executables() {
    assert!(browser_harness_uses_html_entrypoint("chrome"));
    assert!(browser_harness_uses_html_entrypoint(
        "chrome-headless-shell"
    ));
    assert!(browser_harness_uses_html_entrypoint("chromium"));
    assert!(browser_harness_uses_html_entrypoint("chromium-browser"));
    assert!(browser_harness_uses_html_entrypoint("chromium-for-testing"));
    assert!(browser_harness_uses_html_entrypoint("chromium for testing"));
    assert!(browser_harness_uses_html_entrypoint(
        "/usr/bin/google-chrome-stable"
    ));
    assert!(browser_harness_uses_html_entrypoint("google chrome beta"));
    assert!(browser_harness_uses_html_entrypoint("google chrome canary"));
    assert!(browser_harness_uses_html_entrypoint("google chrome dev"));
    assert!(browser_harness_uses_html_entrypoint(
        "google chrome for testing"
    ));
    assert!(browser_harness_uses_html_entrypoint("google chrome stable"));
    assert!(browser_harness_uses_html_entrypoint(
        "google chrome unstable"
    ));
    assert!(browser_harness_uses_html_entrypoint("google-chrome-stable"));
    assert!(browser_harness_uses_html_entrypoint(
        "google-chrome-headless-shell"
    ));
    assert!(browser_harness_uses_html_entrypoint("msedge.exe"));
    assert!(browser_harness_uses_html_entrypoint("msedge-beta"));
    assert!(browser_harness_uses_html_entrypoint("msedge-canary"));
    assert!(browser_harness_uses_html_entrypoint("msedge-dev"));
    assert!(browser_harness_uses_html_entrypoint("msedge-insider"));
    assert!(browser_harness_uses_html_entrypoint("msedge-stable"));
    assert!(browser_harness_uses_html_entrypoint("edge-beta"));
    assert!(browser_harness_uses_html_entrypoint("edge-canary"));
    assert!(browser_harness_uses_html_entrypoint("edge-dev"));
    assert!(browser_harness_uses_html_entrypoint("edge-insider"));
    assert!(browser_harness_uses_html_entrypoint("edge-stable"));
    assert!(browser_harness_uses_html_entrypoint(
        "microsoft-edge-stable"
    ));
    assert!(browser_harness_uses_html_entrypoint(
        "microsoft edge stable"
    ));
    assert!(browser_harness_uses_html_entrypoint("brave-browser.exe"));
    assert!(browser_harness_uses_html_entrypoint(
        "brave-browser-stable.exe"
    ));
    assert!(browser_harness_uses_html_entrypoint("brave browser stable"));
    assert!(browser_harness_uses_html_entrypoint("chrome.cmd"));
    assert!(browser_harness_uses_html_entrypoint(
        "google-chrome.desktop"
    ));
    assert!(browser_harness_uses_html_entrypoint("Google Chrome.app"));
    assert!(browser_harness_uses_html_entrypoint(
        "Google Chrome.command"
    ));
    assert!(browser_harness_uses_html_entrypoint("Google Chrome.lnk"));
    assert!(browser_harness_uses_html_entrypoint(
        "Google Chrome.lnk.exe"
    ));
    assert!(browser_harness_uses_html_entrypoint(
        "Google Chrome.app.exe"
    ));
    assert!(browser_harness_uses_html_entrypoint(
        "C:/Program Files/Google/Chrome/Application/google-chrome.desktop"
    ));
    assert!(browser_harness_uses_html_entrypoint(
        "microsoft-edge.desktop.cmd"
    ));
    assert!(browser_harness_uses_html_entrypoint("chrome.ps1"));
    assert!(browser_harness_uses_html_entrypoint("Google Chrome.url"));
    assert!(browser_harness_uses_html_entrypoint(
        "google-chrome.url.exe"
    ));
    assert!(browser_harness_uses_html_entrypoint(
        "microsoft-edge.url.cmd"
    ));
    assert!(browser_harness_uses_html_entrypoint("google-chrome-dev"));
    assert!(browser_harness_uses_html_entrypoint("brave-browser-dev"));
    assert!(browser_harness_uses_html_entrypoint(
        "brave-browser-nightly"
    ));
    assert!(browser_harness_uses_html_entrypoint(
        "microsoft-edge-insider"
    ));
    assert!(browser_harness_uses_html_entrypoint("firefox-beta"));
    assert!(browser_harness_uses_html_entrypoint("firefox-esr"));
    assert!(browser_harness_uses_html_entrypoint("opera-stable"));
    assert!(browser_harness_uses_html_entrypoint("vivaldi-stable"));
    assert!(browser_harness_uses_html_entrypoint("vivaldi-snapshot"));
    assert!(browser_harness_uses_html_entrypoint(
        "C:/Program Files/Google/Chrome/Application/chrome.bat"
    ));
    assert!(browser_harness_uses_html_entrypoint("firefox"));
    assert!(browser_harness_uses_html_entrypoint("firefox-nightly"));
    assert!(browser_harness_uses_html_entrypoint(
        "firefox-developer-edition"
    ));
    assert!(browser_harness_uses_html_entrypoint("librewolf"));
    assert!(browser_harness_uses_html_entrypoint("waterfox"));
    assert!(browser_harness_uses_html_entrypoint("mullvad-browser"));
    assert!(browser_harness_uses_html_entrypoint("mullvad browser"));
    assert!(browser_harness_uses_html_entrypoint("privacy-browser"));
    assert!(browser_harness_uses_html_entrypoint("privacy browser"));
    assert!(browser_harness_uses_html_entrypoint("opera"));
    assert!(browser_harness_uses_html_entrypoint("vivaldi"));
    assert!(browser_harness_uses_html_entrypoint("Mullvad Browser.app"));
    assert!(browser_harness_uses_html_entrypoint("zen-browser"));
    assert!(browser_harness_uses_html_entrypoint("zen browser"));
    assert!(browser_harness_uses_html_entrypoint("thorium-browser"));
    assert!(browser_harness_uses_html_entrypoint("thorium browser"));
    assert!(!browser_harness_uses_html_entrypoint("node"));
    assert!(!browser_harness_uses_html_entrypoint("bun"));
}


#[test]
fn browser_bundle_harness_script_reuses_the_shared_fetch_prelude() {
    let script = browser_bundle_harness_script(
        "browser-app",
        false,
        "const mod = await import(bundleJs.href);\nconsole.log(typeof mod);\n",
    );
    assert!(script.contains("const bundleJs = new URL('./browser-app/browser-app.js'"));
    assert!(script.contains("const wasmUrl = new URL('./browser-app/browser-app.wasm'"));
    assert!(script.contains("console.log(typeof mod);"));
    assert!(script.contains("globalThis.fetch = async (input) => {"));
}


#[test]
fn browser_runtime_harness_script_executes_wasm_and_bridges_console_output() {
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "args_len" (func $args_len (result i32)))
                (import "kali:rt" "console_log" (func $console_log (param i64)))
                (import "kali:rt" "console_error" (func $console_error (param i64)))
                (func (export "_start")
                    call $args_len
                    i64.extend_i32_s
                    call $console_log
                    call $args_len
                    i64.extend_i32_s
                    call $console_error))
        "#,
    );
    let script =
        browser_runtime_harness_script(&wasm, &["alpha".to_string(), "beta".to_string()], false);
    let tempdir = kali_test_support::fixtures::tempdir();
    let script_path = kali_test_support::fixtures::write_file(tempdir.path(), "browser-runtime.mjs", &script);

    let outcome = browser_harness_run_checked(Some("node"), &script_path, &[], tempdir.path())
        .expect("launch browser runtime harness");

    assert_eq!(outcome.command[0], "node");
    assert_eq!(outcome.status.code(), Some(0));
    assert!(outcome.stdout.contains('2'), "stdout: {}", outcome.stdout);
    assert!(outcome.stderr.contains('2'), "stderr: {}", outcome.stderr);
}


#[test]
fn browser_runtime_harness_script_executes_registered_callbacks_and_reports_zero_failures() {
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "test_register" (func $test_register (param i64)))
                (import "kali:rt" "console_log" (func $console_log (param i64)))
                (func (export "__kali_callback_7")
                    i64.const 11
                    call $console_log)
                (func (export "_start")
                    i64.const 7
                    call $test_register))
        "#,
    );
    let script = browser_runtime_harness_script(&wasm, &["gamma".to_string()], true);
    let tempdir = kali_test_support::fixtures::tempdir();
    let script_path = kali_test_support::fixtures::write_file(tempdir.path(), "browser-runtime-tests.mjs", &script);

    let outcome = browser_harness_run_checked(Some("node"), &script_path, &[], tempdir.path())
        .expect("launch browser runtime harness");

    assert_eq!(outcome.status.code(), Some(0));
    assert!(
        outcome.stdout.contains("\"tests\":[\"7\"]"),
        "stdout: {}",
        outcome.stdout
    );
    assert!(
        outcome.stdout.contains("\"testsFailed\":0"),
        "stdout: {}",
        outcome.stdout
    );
    assert!(outcome.stdout.contains("11"), "stdout: {}", outcome.stdout);
    assert!(
        outcome.stdout.contains("\"args\":[\"gamma\"]"),
        "stdout: {}",
        outcome.stdout
    );
}


#[test]
fn browser_runtime_harness_script_reports_failed_callbacks_and_nonzero_exit() {
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "test_register" (func $test_register (param i64)))
                (func (export "__kali_callback_7")
                    unreachable)
                (func (export "_start")
                    i64.const 7
                    call $test_register))
        "#,
    );
    let script = browser_runtime_harness_script(&wasm, &["delta".to_string()], true);
    let tempdir = kali_test_support::fixtures::tempdir();
    let script_path = kali_test_support::fixtures::write_file(tempdir.path(), "browser-runtime-tests-failed.mjs", &script);

    let outcome = browser_harness_run_checked(Some("node"), &script_path, &[], tempdir.path())
        .expect("launch browser runtime harness");

    assert_ne!(outcome.status.code(), Some(0));
    assert!(
        outcome.stdout.contains("\"tests\":[\"7\"]"),
        "stdout: {}",
        outcome.stdout
    );
    assert!(
        outcome.stdout.contains("\"testsFailed\":1"),
        "stdout: {}",
        outcome.stdout
    );
    assert!(
        outcome.stderr.contains("browser runtime test failures: 1"),
        "stderr: {}",
        outcome.stderr
    );
}


#[test]
fn browser_runtime_harness_script_reports_an_empty_test_summary_when_no_callbacks_register() {
    let wasm = compile_wat(
        r#"
            (module
                (func (export "_start")))
        "#,
    );
    let script = browser_runtime_harness_script(&wasm, &["epsilon".to_string()], true);
    let tempdir = kali_test_support::fixtures::tempdir();
    let script_path = kali_test_support::fixtures::write_file(tempdir.path(), "browser-runtime-empty-tests.mjs", &script);

    let outcome = browser_harness_run_checked(Some("node"), &script_path, &[], tempdir.path())
        .expect("launch browser runtime harness");

    assert_eq!(outcome.status.code(), Some(0));
    assert!(
        outcome.stdout.contains("\"tests\":[]"),
        "stdout: {}",
        outcome.stdout
    );
    assert!(
        outcome.stdout.contains("\"testsFailed\":0"),
        "stdout: {}",
        outcome.stdout
    );
    assert!(
        outcome.stdout.contains("\"args\":[\"epsilon\"]"),
        "stdout: {}",
        outcome.stdout
    );
}


#[test]
fn browser_runtime_harness_summary_file_capture_is_deterministic() {
    let wasm = compile_wat(
        r#"
            (module
                (import "kali:rt" "test_register" (func $test_register (param i64)))
                (import "kali:rt" "console_log" (func $console_log (param i64)))
                (func (export "__kali_callback_7")
                    i64.const 11
                    call $console_log)
                (func (export "_start")
                    i64.const 7
                    call $test_register))
        "#,
    );
    let script = browser_runtime_harness_script(&wasm, &["zeta".to_string()], true);
    let tempdir = kali_test_support::fixtures::tempdir();
    let script_path = kali_test_support::fixtures::write_file(tempdir.path(), "browser-runtime-summary-file.mjs", &script);
    let summary_path = tempdir.path().join("browser-runtime-summary.json");

    let outcome = browser_harness_run_checked_with_env(
        Some("node"),
        &script_path,
        &[],
        tempdir.path(),
        &[(
            crate::BROWSER_HARNESS_SUMMARY_FILE_ENV,
            summary_path.as_os_str(),
        )],
    )
    .expect("launch browser runtime harness");

    assert_eq!(outcome.status.code(), Some(0));
    assert!(
        !outcome.stdout.contains("\"tests\":"),
        "stdout: {}",
        outcome.stdout
    );
    let summary = fs::read_to_string(&summary_path).expect("summary file");
    assert!(
        summary.contains("\"args\":[\"zeta\"]"),
        "summary: {}",
        summary
    );
    assert!(
        summary.contains("\"tests\":[\"7\"]"),
        "summary: {}",
        summary
    );
    assert!(
        summary.contains("\"testsFailed\":0"),
        "summary: {}",
        summary
    );
    assert!(
        summary.contains("\"hostContract\":\"browser-requested\""),
        "summary: {}",
        summary
    );
    assert!(
        summary.contains("\"runtimeBackend\":\"browser-harness\""),
        "summary: {}",
        summary
    );
}


#[test]
fn browser_bundle_runtime_harness_page_wraps_the_module_body_for_real_browser_hosts() {
    let page = browser_bundle_runtime_harness_page(
        "browser-app",
        false,
        &["alpha".to_string(), "beta".to_string()],
        true,
    );

    assert!(page.starts_with("<!doctype html>"), "page: {page}");
    assert!(page.contains("<script type=\"module\">"), "page: {page}");
    assert!(
        page.contains("const runtimeArgs = [\"alpha\",\"beta\"]"),
        "page: {page}"
    );
    assert!(
        page.contains("const runRegisteredTests = true;"),
        "page: {page}"
    );
    assert!(page.contains("browser-app/browser-app.js"), "page: {page}");
}
