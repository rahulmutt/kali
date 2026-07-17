use crate::test_support::*;
use crate::*;
use std::fs;

// --- Browser import-list mirror-sync guard (throw-fallout Stage 3, bucket H) ---
//
// The browser lane hand-mirrors its `kali:rt` importObject across four JS
// templates: this harness module's bundle-runtime script (List A) and
// self-contained runtime script (List B), plus `kali_cli`'s generated ESM
// (List C) and CJS (List D) bundle glue in `cmd_build.rs`. There is no single
// source of truth for the member set (see memory
// `kali-browser-harness-import-sync`), so any host-wired conditional import
// the guest may emit must be added to all four by hand or the browser lane
// LinkErrors (`WebAssembly.instantiate` rejects with "function import
// requires a callable").
//
// This test does not single-source the four lists' *text* into one template;
// it single-sources each list's text at its own emission site (the harness
// functions below are the same functions `execute.rs` calls to build the
// scripts that actually run) and cross-checks that every REQUIRED member
// name appears in all four.

/// Raw source of the `kali_cli` bundle-glue generator, pulled in at compile
/// time so this crate (which `kali_cli` depends on, not the reverse) can
/// scan List C/D's text without introducing a reverse crate dependency.
const CMD_BUILD_SRC: &str = include_str!("../../../kali_cli/src/bin/cmd_build.rs");

/// Split `cmd_build.rs`'s source into the ESM (List C) and CJS (List D)
/// bundle-glue `format!` template bodies, keyed on the two `BundleFormat`
/// match-arm markers that introduce each template. If either template is
/// restructured such that these markers move or disappear, this fails loudly
/// (`.expect`) rather than silently scanning the wrong text.
///
/// Both slices are bounded on both ends: the ESM slice runs from its marker
/// up to (not including) the CJS marker, and the CJS slice runs from its
/// marker up to its own raw-string closing delimiter (`"#`) followed by the
/// `format!` call's closing `),` and the enclosing `match`'s closing `};`).
/// Without the CJS upper bound, the slice would run to end-of-file and could
/// false-pass a `REQUIRED`-member check on unrelated code appearing later in
/// `cmd_build.rs` that happens to contain the same text, rather than on
/// content actually inside List D.
fn cmd_build_bundle_sources() -> [(&'static str, String); 2] {
    let esm_marker = "BundleFormat::Esm => format!(";
    let cjs_marker = "BundleFormat::Cjs => format!(";
    // The exact byte sequence that closes the CJS template's raw string
    // literal, the `format!(...)` call, and the enclosing `match format { ... }`
    // arm (verified against the literal source: `"#` alone on its own line,
    // then the `),` closing the `format!` call, then the `};` closing the
    // `match`).
    let cjs_template_close = "\n\"#\n        ),\n    };";
    let esm_start = CMD_BUILD_SRC
        .find(esm_marker)
        .expect("cmd_build.rs must declare the ESM bundle-glue importObject template");
    let cjs_start = CMD_BUILD_SRC
        .find(cjs_marker)
        .expect("cmd_build.rs must declare the CJS bundle-glue importObject template");
    assert!(
        esm_start < cjs_start,
        "expected the ESM bundle-glue template to precede the CJS template in cmd_build.rs"
    );
    let cjs_close_rel = CMD_BUILD_SRC[cjs_start..].find(cjs_template_close).expect(
        "cmd_build.rs CJS bundle-glue template must have a matching closing delimiter \
             (raw string `\"#` + format! `),` + match-arm `};`); if the template body changed \
             shape, update `cjs_template_close` to match rather than widening the slice",
    );
    let cjs_end = cjs_start + cjs_close_rel + cjs_template_close.len();
    let esm_text = CMD_BUILD_SRC[esm_start..cjs_start].to_string();
    let cjs_text = CMD_BUILD_SRC[cjs_start..cjs_end].to_string();
    [("cmd_build.esm", esm_text), ("cmd_build.cjs", cjs_text)]
}

/// The four hand-mirrored browser `kali:rt` importObject sources, labeled for
/// assertion failure messages.
///
/// NOTE (Tasks 5-7 extension point): List A/B below are RENDERED JS — the
/// harness functions have already run their `format!` substitutions, so
/// e.g. a `{{` in the Rust template source appears here as a plain `{`.
/// List C/D (`cmd_build_bundle_sources`) are RAW Rust template *source*
/// text pulled in via `include_str!`, still containing the doubled
/// `{{`/`}}` `format!` escapes verbatim. A `REQUIRED` member check added
/// here must use a substring test that matches both shapes (e.g. checking
/// for `"{member}("` works for both, since neither shape doubles the
/// member-name-plus-paren text itself) — do not assert on exact rendered-JS
/// text, since that would only ever match List A/B.
fn browser_import_list_sources() -> Vec<(&'static str, String)> {
    let mut sources = vec![
        (
            "harness.A",
            browser_bundle_runtime_harness_module_script("bundle-dir", true, &[], false),
        ),
        ("harness.B", browser_runtime_harness_script(&[], &[], false)),
    ];
    sources.extend(cmd_build_bundle_sources());
    sources
}

#[test]
fn browser_import_lists_declare_all_host_wired_kalirt_members() {
    // Members every browser importObject must expose (conditional imports the guest may emit).
    const REQUIRED: &[&str] = &[
        "coverage_hit",
        "performance_now",
        "crypto_get_random_values",
        "crypto_random_uuid",
        "crypto_subtle_digest",
        // Stage D Task 6 (D2): the deferred-callback lane's five host imports
        // (Tasks 4-5 codegen). Missing any of these LinkErrors any browser
        // module that emits a `queueMicrotask`/`setTimeout`/`setInterval` call.
        "queueMicrotask",
        "setTimeout",
        "setInterval",
        "clearTimeout",
        "clearInterval",
    ];
    for (label, src) in browser_import_list_sources() {
        for member in REQUIRED {
            assert!(
                src.contains(&format!("{member}(")) || src.contains(&format!("{member} (")),
                "browser import list {label} is missing kali:rt member `{member}`"
            );
        }
    }
}

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
    let script_path =
        kali_test_support::fixtures::write_file(tempdir.path(), "browser-runtime.mjs", &script);

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
    let script_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-tests.mjs",
        &script,
    );

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
    let script_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-tests-failed.mjs",
        &script,
    );

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
    let script_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-empty-tests.mjs",
        &script,
    );

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
    let script_path = kali_test_support::fixtures::write_file(
        tempdir.path(),
        "browser-runtime-summary-file.mjs",
        &script,
    );
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

#[test]
fn browser_bundle_harness_page_is_browser_native() {
    let body = "const mod = await import(bundleJs.href);\nawait mod.start();\n";
    let page = browser_bundle_harness_page("app", body);
    assert!(page.starts_with("<!doctype html>"), "page: {page}");
    assert!(page.contains("<script type=\"module\">"), "page: {page}");
    assert!(
        page.contains("const bundleJs = new URL('./app/app.js', import.meta.url);"),
        "page: {page}"
    );
    assert!(page.contains(body), "page: {page}");
    assert!(
        page.contains("globalThis.__kaliHarnessDone('')"),
        "page must signal the completion binding with one string arg: {page}"
    );
    assert!(page.contains(BROWSER_HARNESS_DONE_BINDING), "page: {page}");
    assert!(
        !page.contains("node:"),
        "a browser-native page must not import node builtins: {page}"
    );
}

#[test]
fn browser_bundle_harness_page_shares_the_node_script_body_contract() {
    let body = "const mod = await import(bundleJs.href);\nawait mod.start();\n";
    let node_script = browser_bundle_harness_script("app", false, body);
    let page = browser_bundle_harness_page("app", body);
    for artifact in [node_script.as_str(), page.as_str()] {
        assert!(
            artifact.contains("const bundleJs = new URL('./app/app.js', import.meta.url);"),
            "artifact: {artifact}"
        );
        assert!(artifact.contains(body), "artifact: {artifact}");
    }
}
