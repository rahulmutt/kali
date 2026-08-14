//! Task 18 batch 8A audit escalation: kept 100% hand-written, not migrated.
//!
//! GROUND: FIXTURE SELF-INSPECTION (controller ruling 4 / ruling 10).
//! All 16 `#[test]` fns in this file route through
//! `assert_browser_requested_promise_any` (`:83`), which runs 6
//! `assert!(source.contains(...))` self-checks (`:91-111`) on the JS fixture's
//! OWN TEXT -- a dev-time invariant check that the fixture still literally
//! embeds every frozen-callable `Promise.any` spelling this file means to
//! exercise -- before the fixture is ever written to disk or `kali` is ever
//! invoked. These are not claims about process output.
//!
//! `audit-case-migration.py` deliberately excludes everything under a migrated
//! case file's `[source]` table from its claim search, so a migration of this
//! file would report those literals MISSING even though they are verbatim
//! present in the `[source]` fixture body. Controller ruling 4 is explicit that
//! the audit script is NOT extended for this shape and that each hit is
//! escalated per rule 3 and retained hand-written with a `//!` header per U3.
//!
//! WHY WHOLE-FILE AND NOT U4 TRIM-AND-KEEP: U4's trim-and-keep is tried first
//! and does not apply, because EVERY test reaches the self-inspecting helper
//! unconditionally -- 16 of 16 -- so there is no complementary migratable
//! subset to split off. That reach count is not asserted here, it is derived:
//!
//!     $ python3 tools/task-18-browser-pilot/find_fixture_self_inspection.py
//!     browser_promise_any_harness.rs
//!         6 site(s) in assert_browser_requested_promise_any; 16 of 16 #[test]
//!         fns reach it -> WHOLE-FILE retention
//!
//! NOTE THE SHAPE, because ruling 10 was CORRECTED over exactly this file: the
//! self-inspecting `assert!` lives INSIDE the helper that also builds the
//! `Command`, so the superseded predicate ("which `#[test]` fns never construct
//! a Command") returns nothing here and misses this file entirely. Do not
//! re-derive the predicate from prose; run the tool.
//!
//! This file is one of the exactly two instances controller ruling 10 recorded
//! as NOT YET ADJUDICATED; this batch adjudicates it. The predicate's `KNOWN`
//! list already carries it, which ruling 10 requires so the selftest cannot
//! silently weaken as the corpus grows.
//!
//! Full reasoning: the batch's own working report -- which was git-ignored scratch and
//! does not ship, so it is deliberately not cited by path. No case file exists for this target.//!
//! CONSEQUENCE FOR THE GATES (ruling 9). THIS FILE HAS NO RED-LIST, and that is
//! the finding, not an omission. Ruling 9 addresses a U4 TRIM-and-keep retention,
//! where the on-disk `.rs` is shorter than the source its case file was migrated
//! from and every literal-comparison gate therefore runs against the wrong
//! left-hand side. This is a WHOLE-FILE retention: nothing was trimmed, so there
//! is no pre-trim/post-trim divergence, no pre-trim ref declaration (the
//! literal marker is deliberately NOT spelled here: several readers grep the
//! whole file for it, so a header quoting it would be read as declaring one),
//! and ruling 12's
//! third (migrated-complement) column does not apply either -- there is no
//! migrated complement. There is also no right-hand side: `verify_pair.sh promise_any_harness`
//! exits 2 with `missing .../cases/browser/promise_any_harness.toml` before running any gate,
//! so the five gates that take a `.rs`/`.toml` pair cannot run here at all. The
//! exception is `batch5_crosscheck.py`, which needs no case file -- it resolves
//! THIS header's own `:N` citations against this very file. Run it directly:
//! `batch5_crosscheck.py --citations-only promise_any_harness`. It exits 0 today.
//!
use std::{fs, process::Command};

use tempfile::tempdir;

use kali_common::promise_any_browser_body_source;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_promise_any_run_source() -> String {
    format!(
        "async function browserPromiseAny() {{\n{}\n}}\n\nasync function main() {{\n  await browserPromiseAny();\n  console.log('browser promise any ok');\n}}\n\nmain();\n",
        promise_any_browser_body_source()
    )
}

fn browser_promise_any_test_source() -> String {
    format!(
        "async function browserPromiseAny() {{\n{}\n}}\n\nKali.test('browser promise any', () => browserPromiseAny());\n",
        promise_any_browser_body_source()
    )
}

fn assert_browser_requested_promise_any(command: &str, filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = if command == "test" {
        browser_promise_any_test_source()
    } else {
        browser_promise_any_run_source()
    };
    assert!(
        source.contains(r#"Object.freeze(globalThis["Promise"]["any"])"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze(globalThis['Promise']['any'])"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis["Promise"]["any"]))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis['Promise']['any']))"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis.Promise)["any"])"#),
        "source: {source}"
    );
    assert!(
        source.contains(r#"Object.freeze((globalThis.Promise)['any'])"#),
        "source: {source}"
    );
    fs::write(&source_path, source).expect("write source");

    let mut command_line = Command::new(kali_bin());
    command_line
        .current_dir(dir.path())
        .env(kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV, "node");
    if json_output {
        command_line.arg("--output").arg("json");
    }
    let output = command_line
        .arg(command)
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!output.status.success(), "must fail closed: {output:?}");
}

#[test]
fn run_supports_browser_promise_any_in_js_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_any("run", "main.js", false);
}

#[test]
fn run_supports_browser_promise_any_in_ts_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_any("run", "main.ts", false);
}

#[test]
fn run_supports_browser_promise_any_in_jsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_any("run", "main.jsx", false);
}

#[test]
fn run_supports_browser_promise_any_in_tsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_any("run", "main.tsx", false);
}

#[test]
fn test_supports_browser_promise_any_in_js_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_any("test", "smoke.test.js", false);
}

#[test]
fn test_supports_browser_promise_any_in_ts_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_any("test", "smoke.test.ts", false);
}

#[test]
fn test_supports_browser_promise_any_in_jsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_any("test", "smoke.test.jsx", false);
}

#[test]
fn test_supports_browser_promise_any_in_tsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_any("test", "smoke.test.tsx", false);
}

#[test]
fn json_run_supports_browser_promise_any_in_js_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_any("run", "main.js", true);
}

#[test]
fn json_run_supports_browser_promise_any_in_ts_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_any("run", "main.ts", true);
}

#[test]
fn json_run_supports_browser_promise_any_in_jsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_any("run", "main.jsx", true);
}

#[test]
fn json_run_supports_browser_promise_any_in_tsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_any("run", "main.tsx", true);
}

#[test]
fn json_test_supports_browser_promise_any_in_js_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_any("test", "smoke.test.js", true);
}

#[test]
fn json_test_supports_browser_promise_any_in_ts_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_any("test", "smoke.test.ts", true);
}

#[test]
fn json_test_supports_browser_promise_any_in_jsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_any("test", "smoke.test.jsx", true);
}

#[test]
fn json_test_supports_browser_promise_any_in_tsx_input_when_browser_harness_is_configured() {
    assert_browser_requested_promise_any("test", "smoke.test.tsx", true);
}
