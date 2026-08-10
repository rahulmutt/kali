//! Task 18 batch 8A audit escalation: kept 100% hand-written, not migrated.
//!
//! GROUND: FIXTURE SELF-INSPECTION (controller ruling 4 / ruling 10).
//! All 8 `#[test]` fns in this file route through
//! `assert_browser_bundle_promise_any` (`:68`), which runs 6
//! `assert!(source.contains(...))` self-checks (`:72-92`) on the JS fixture's
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
//! unconditionally -- 8 of 8 -- so there is no complementary migratable subset
//! to split off. That reach count is not asserted here, it is derived:
//!
//!     $ python3 tools/task-18-browser-pilot/find_fixture_self_inspection.py
//!     browser_promise_any_bundle.rs
//!         6 site(s) in assert_browser_bundle_promise_any; 8 of 8 #[test] fns
//!         reach it -> WHOLE-FILE retention
//!
//! This file is one of the exactly two instances controller ruling 10 recorded
//! as NOT YET ADJUDICATED; this batch adjudicates it. The predicate's `KNOWN`
//! list already carries it, which ruling 10 requires so the selftest cannot
//! silently weaken as the corpus grows.
//!
//! Full reasoning: .superpowers/sdd/2026-07-29-test-binary-consolidation/
//! task-18-batch8a-report.md. No case file exists for this target.//!
//! CONSEQUENCE FOR THE GATES (ruling 9). THIS FILE HAS NO RED-LIST, and that is
//! the finding, not an omission. Ruling 9 addresses a U4 TRIM-and-keep retention,
//! where the on-disk `.rs` is shorter than the source its case file was migrated
//! from and every literal-comparison gate therefore runs against the wrong
//! left-hand side. This is a WHOLE-FILE retention: nothing was trimmed, so there
//! is no pre-trim/post-trim divergence, no `PRE-TRIM REF:` line, and ruling 12's
//! third (migrated-complement) column does not apply either -- there is no
//! migrated complement. There is also no right-hand side: `verify_pair.sh promise_any_bundle`
//! exits 2 with `missing .../cases/browser/promise_any_bundle.toml` before running any gate,
//! so the five gates that take a `.rs`/`.toml` pair cannot run here at all. The
//! exception is `batch5_crosscheck.py`, which needs no case file -- it resolves
//! THIS header's own `:N` citations against this very file. Run it directly:
//! `batch5_crosscheck.py --citations-only promise_any_bundle`. It exits 0 today.
//!
use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

use kali_common::promise_any_browser_body_source;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_promise_any_source() -> String {
    format!(
        "// kali-tree-shake: browserPromiseAny\nasync function browserPromiseAny() {{\n{}\n}}\n",
        promise_any_browser_body_source()
    )
}

fn assert_browser_bundle_promise_any(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = browser_bundle_promise_any_source();
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

    let mut command = Command::new(kali_bin());
    command
        .current_dir(dir.path())
        .arg("build")
        .arg("--bundle")
        .arg("--api")
        .arg("browser");
    if json_output {
        command.arg("--output").arg("json");
    }
    let output = command.arg(&source_path).output().expect("run kali");

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if json_output {
        let envelope: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(envelope["schemaVersion"], 1);
        assert_eq!(envelope["command"], "build");
        assert_eq!(envelope["success"], true);
        assert_eq!(envelope["exitCode"], 0);
        let payload = envelope["payload"].as_object().expect("payload object");
        assert_eq!(payload["artifactKind"], "bundle");
        assert_eq!(payload["bundleFormat"], "esm");
        assert!(envelope["errors"]
            .as_array()
            .expect("errors array")
            .is_empty());
    }

    let bundle_dir = dir.path().join("app");
    let metadata: Value = serde_json::from_str(
        &fs::read_to_string(bundle_dir.join("app.meta.json")).expect("read meta"),
    )
    .expect("parse metadata json");
    assert_eq!(metadata["apiSurface"], "browser");
    assert_eq!(metadata["artifactKind"], "bundle");

    let harness_path = bundle_dir
        .parent()
        .expect("bundle root parent")
        .join("browser-bundle-smoke.mjs");
    let harness = kali_runtime_contract::browser_bundle_harness_script(
        "app",
        false,
        r#"const mod = await import(bundleJs.href);
await mod.browserPromiseAny();
console.log('browser promise any ok');
"#,
    );
    fs::write(&harness_path, harness).expect("write browser bundle harness");

    let mut harness_command = kali_runtime_contract::browser_harness_command_parts_for(
        std::env::var("KALI_BROWSER_BUNDLE_HARNESS_COMMAND")
            .ok()
            .as_deref(),
    );
    let harness_executable = harness_command.remove(0);
    let output = Command::new(&harness_executable)
        .current_dir(&bundle_dir)
        .args(&harness_command)
        .arg(&harness_path)
        .output()
        .expect("run browser bundle harness");

    // Honest re-pin (PR #16 rev2): kali fails closed/loud here;
    // see docs/superpowers/followups/pr16-honest-repin-inventory.md.
    assert!(!output.status.success(), "must fail closed: {output:?}");
}

#[test]
fn build_emits_browser_promise_any_in_js_input() {
    assert_browser_bundle_promise_any("app.js", false);
}

#[test]
fn build_emits_browser_promise_any_in_ts_input() {
    assert_browser_bundle_promise_any("app.ts", false);
}

#[test]
fn build_emits_browser_promise_any_in_jsx_input() {
    assert_browser_bundle_promise_any("app.jsx", false);
}

#[test]
fn build_emits_browser_promise_any_in_tsx_input() {
    assert_browser_bundle_promise_any("app.tsx", false);
}

#[test]
fn json_build_emits_browser_promise_any_in_js_input() {
    assert_browser_bundle_promise_any("app.js", true);
}

#[test]
fn json_build_emits_browser_promise_any_in_ts_input() {
    assert_browser_bundle_promise_any("app.ts", true);
}

#[test]
fn json_build_emits_browser_promise_any_in_jsx_input() {
    assert_browser_bundle_promise_any("app.jsx", true);
}

#[test]
fn json_build_emits_browser_promise_any_in_tsx_input() {
    assert_browser_bundle_promise_any("app.tsx", true);
}
