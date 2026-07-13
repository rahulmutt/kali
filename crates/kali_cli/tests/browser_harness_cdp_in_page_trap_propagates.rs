//! Stage-0 residual regression pin (throw-fallout Stage 3, Task 9): the
//! Chromium/HTML CDP lane must SURFACE an in-page guest trap, never swallow it
//! into a clean pass.
//!
//! Stage 0 closed a trap-swallow bug where a *node* browser-harness crash was
//! reported as `passed:1 success:true`. The Rust crash lane
//! (`kali_runtime::browser_tests_failed`) is lane-agnostic — it counts a
//! non-success harness exit with zero reported failures as one failure — but it
//! only works if the driver propagates an in-page guest trap. Stage 0's
//! reproducer only exercised the node `.mjs` lane and explicitly flagged the
//! residual: "a CDP driver exiting 0 on a caught in-page trap would still
//! swallow — the host-wiring stage must confirm."
//!
//! This test confirms the Chromium/HTML lane. It drives the *production*
//! embedded-wasm browser harness page — the exact page
//! `kali_runtime::browser_runtime_execute_checked` writes when a Chromium-named
//! harness command routes `kali test --api browser` through the HTML entrypoint
//! (`browser_harness_uses_html_entrypoint`) — under a real headless Chromium via
//! CDP, with a registered `Kali.test` whose body traps at runtime
//! (`throw` lowers to a WASM `unreachable`, i.e. `RuntimeError: unreachable`).
//!
//! Outcome: the production page ALREADY propagates. The guest trap surfaces to
//! the driver as an uncaught `RuntimeError: unreachable` exception and the page
//! never signals a clean completion, so no consumer can read the crashed run as
//! a pass. This test pins that behavior: it fails if the page is ever changed to
//! swallow the trap (e.g. catch it and signal a clean "done", or emit a
//! zero-failure summary despite the crash).
//!
//! Gated on real Chromium (`#[ignore]` + a runtime presence check), like the
//! sibling `browser_cdp_smoke.rs`, so it skips cleanly where no browser exists.
mod cdp_driver;

use std::fs;
use std::process::Command;
use std::time::Duration;

use serde_json::Value;
use tempfile::tempdir;

use cdp_driver::{CdpBrowser, CdpPageOutcome};

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

/// The first available real browser executable, or `None` to skip.
fn chromium() -> Option<String> {
    ["chromium", "chromium-browser", "google-chrome", "chrome"]
        .into_iter()
        .find(|&exe| cdp_driver::chromium_available(exe))
        .map(str::to_owned)
}

/// A registered test whose body traps at runtime: under the print-then-trap
/// `throw` lowering the body runs inline during `_start` and traps
/// (`RuntimeError: unreachable`) — the same trapping construct Stage 0's
/// node-lane fixture used, now exercised through the HTML/CDP entrypoint.
fn trapping_registered_test_source() -> &'static str {
    "Kali.test('self-check throw traps at runtime', () => {\n\
  const actual = 1;\n\
  if (actual !== 2) {\n\
    throw 'expected 2';\n\
  }\n\
});\n"
}

/// Parse a console line as the browser-harness runtime summary — a JSON object
/// with `runtimeBackend: "browser-harness"` — returning it when it matches.
fn parse_harness_summary(text: &str) -> Option<Value> {
    let value: Value = serde_json::from_str(text).ok()?;
    (value.get("runtimeBackend").and_then(Value::as_str) == Some("browser-harness"))
        .then_some(value)
}

#[test]
#[ignore = "requires a real Chromium; run with `-- --ignored`"]
fn browser_harness_cdp_in_page_trap_surfaces_and_is_not_swallowed() {
    let Some(chromium_exe) = chromium() else {
        eprintln!("skipping: no Chromium available");
        return;
    };

    // 1. Build a browser bundle whose registered test traps at runtime.
    let dir = tempdir().expect("tempdir");
    let source = dir.path().join("failing.test.js");
    fs::write(&source, trapping_registered_test_source()).expect("write source");
    let build = Command::new(kali_bin())
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser")
        .arg(&source)
        .output()
        .expect("run kali build");
    assert!(
        build.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    let wasm = fs::read(dir.path().join("failing.test").join("failing.test.wasm"))
        .expect("read bundle wasm");

    // 2. Write the PRODUCTION embedded-wasm HTML harness page (the exact page
    //    the Chromium/HTML `kali test --api browser` entrypoint loads) with
    //    registered-test execution enabled, then load it via a file:// URL —
    //    the wasm is base64-embedded, so no HTTP fetch is needed.
    let page = kali_runtime::browser_runtime_harness_page(&wasm, &[], true);
    let page_path = dir.path().join("browser-runtime.html");
    fs::write(&page_path, page).expect("write harness page");
    let url = url::Url::from_file_path(&page_path)
        .expect("file url")
        .to_string();

    // 3. Drive it through a real headless Chromium via CDP.
    let mut browser =
        CdpBrowser::launch(&chromium_exe, Duration::from_secs(20)).expect("launch chromium");
    let outcome: CdpPageOutcome = browser
        .run_page(&url, Duration::from_secs(20))
        .expect("run page");
    browser.close().expect("close");

    // 4a. A crashed run must NEVER signal a clean completion. The harness page
    //     installs no completion binding on a trap; a swallow that caught the
    //     trap and signalled "done" would flip this to `true`.
    assert!(
        !outcome.completed,
        "a trapping registered test must not report a clean completion; console: {:?}",
        outcome.console
    );

    // 4b. The guest trap must be SURFACED to the driver as an uncaught
    //     exception, not silently discarded.
    let surfaced_trap = outcome.console.iter().any(|line| {
        line.kind == "exception"
            && line.text.contains("RuntimeError")
            && line.text.contains("unreachable")
    });
    assert!(
        surfaced_trap,
        "expected an uncaught `RuntimeError: unreachable` from the in-page guest trap; console: {:?}",
        outcome.console
    );

    // 4c. The crash must not be masked as a zero-failure pass: if any harness
    //     summary was emitted, it must not claim `testsFailed: 0`.
    for line in &outcome.console {
        if let Some(summary) = parse_harness_summary(&line.text) {
            let tests_failed = summary.get("testsFailed").and_then(Value::as_u64);
            assert_ne!(
                tests_failed,
                Some(0),
                "harness summary masked the crash as zero failures: {summary}"
            );
        }
    }
}
