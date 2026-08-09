//! Task 18 batch 2 audit escalation: kept 100% hand-written, not migrated.
//!
//! All 5 `#[test]` fns in this file route through
//! `assert_browser_bundle_array_from_set_map` (`:164`), which runs 22
//! `assert!(source.contains(...))` self-checks (`:168-189`) on the JS
//! fixture's OWN TEXT -- a dev-time invariant check that the fixture still
//! literally embeds every `Array.from`/bracket-notation/logical-operator
//! variant this file means to exercise -- before the fixture is ever
//! written to disk or `kali` is ever invoked. These are not claims about
//! process output.
//!
//! `audit-case-migration.py` deliberately excludes everything under a
//! migrated case file's `[source]` table from its claim search (see that
//! script's module docstring: "`body` and everything under `[source]` are
//! program text, not claims about behavior"). A full draft migration of
//! this file was built and verified against the real `kali` binary (8/8
//! trials passing, `ext(4) x json_output(2)` matrix), then audited with
//! the real `audit-case-migration.py` -- AUDIT FAILED, all 22 of the
//! literals above reported MISSING, despite being genuinely, verbatim
//! present in the migrated `[source]` fixture body (confirmed by
//! construction: that body is a byte-for-byte copy of this file's own
//! `browser_bundle_array_from_set_map_source()`). This is the same shape
//! as the Task 18 pilot's `browser_math_pow_exponent_one.rs` finding (see
//! `/workspace/.superpowers/sdd/2026-07-29-test-binary-consolidation/
//! task-18-pilot-report.md`), except here EVERY `#[test]` fn (not a
//! subset) reaches the flagged helper unconditionally, so the pilot's
//! §5.11 "trim-and-keep" disposition degenerates to whole-file retention --
//! there is no complementary migratable subset to split off. The draft
//! `.toml` was deleted rather than shipped; no case file exists for this
//! target.
//!
//! CONSEQUENCE FOR THE GATES (ruling 9), added retroactively by Task 18 batch 5:
//! THIS FILE HAS NO RED-LIST, and that is the finding, not an omission. Ruling 9
//! addresses a U4 trim-and-keep retention, where the on-disk `.rs` is shorter
//! than the source its case file was migrated from and every literal-comparison
//! gate therefore goes red against the wrong left-hand side. This is a
//! WHOLE-FILE retention: nothing was trimmed, so there is no pre-trim/post-trim
//! divergence and no pre-trim ref to run anything against. There is also no
//! right-hand side -- `verify_pair.sh array_from_set_map_bundle` exits 2 with
//! `missing .../cases/browser/array_from_set_map_bundle.toml` before running any
//! gate. FIVE of the six gates take a `.rs`/`.toml` pair and therefore cannot
//! run here at all. The SIXTH is the exception, and it changes this paragraph:
//! `batch5_crosscheck.py`, the citation gate that batch 6 wired into
//! `verify_pair.sh`, needs no case file -- it resolves THIS header's own `:N`
//! citations against this very file. So a whole-file retention is no longer
//! ungated: run it directly, as
//! `batch5_crosscheck.py --citations-only array_from_set_map_bundle`, because `verify_pair.sh`
//! still exits 2 before reaching it. It exits 0 today. Ruling 11 exempts `:N` from the
//! no-moving-numbers rule only because it is mechanically gated, and this is
//! where that gating applies to a file with no pair. Verified by
//! running it, not assumed. The batch-8 family gate's carve-out for this file is
//! the "must NOT be deleted" line above, not a gate red-list.
use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_array_from_set_map_source() -> &'static str {
    r##"// kali-tree-shake: browserArrayFromSetMapWrappers
export async function browserArrayFromSetMapWrappers() {
  const setValues = [1, 2, 1];
  const mapValues = [[1, 2], [1, 3], [4, 5]];
  for (const value of Array.from(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze(globalThis["Array"].from)(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis["Array"])["from"])(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis["Array"]).from)(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis['Array']).from)(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis['Array'])["from"])(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis['Array'])['from'])(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((null ?? globalThis["Array"].from))(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((true && globalThis["Array"].from))(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((false || globalThis["Array"].from))(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((null ?? globalThis["Array"]["from"]))(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((true && globalThis["Array"]["from"]))(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((false || globalThis["Array"]["from"]))(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze(globalThis['Array']['from'])(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze(globalThis.Array['from'])(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze(globalThis["Array"]['from'])(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((null ?? globalThis['Array']['from']))(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis["Array"]))["from"](new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis['Array']))["from"](new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis['Array']))['from'](new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis.Array).from)(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis.Array)["from"])(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis.Array)['from'])(new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis.Array))["from"](new Set(setValues))) {
    console.log(value);
  }
  for (const value of Object.freeze((globalThis.Array))['from'](new Set(setValues))) {
    console.log(value);
  }
  for await (const value of Array.from(new Set(setValues))) {
    console.log(value);
  }
  for (const entry of Array.from(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for await (const entry of Array.from(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
  for (const value of globalThis["Array"]["from"](new Set(setValues))) {
    console.log(value);
  }
  for await (const entry of Object.freeze((globalThis["Array"])["from"])(new Map(mapValues))) {
    console.log(entry[0]);
    console.log(entry[1]);
  }
}
"##
}

fn assert_browser_bundle_array_from_set_map(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    let source = browser_bundle_array_from_set_map_source();
    assert!(source.contains(r#"Object.freeze((globalThis["Array"])["from"])"#));
    assert!(source.contains(r#"Object.freeze((globalThis['Array']).from)"#));
    assert!(source.contains(r#"Object.freeze((globalThis['Array'])["from"])"#));
    assert!(source.contains(r#"Object.freeze((globalThis['Array'])['from'])"#));
    assert!(source.contains(r#"Object.freeze((globalThis.Array).from)"#));
    assert!(source.contains(r#"Object.freeze((globalThis.Array)["from"])"#));
    assert!(source.contains(r#"Object.freeze((globalThis.Array)['from'])"#));
    assert!(source.contains(r#"Object.freeze((globalThis["Array"]))["from"]"#));
    assert!(source.contains(r#"Object.freeze((globalThis['Array']))["from"]"#));
    assert!(source.contains(r#"Object.freeze((globalThis['Array']))['from']"#));
    assert!(source.contains(r#"Object.freeze((globalThis.Array))["from"]"#));
    assert!(source.contains(r#"Object.freeze((globalThis.Array))['from']"#));
    assert!(source.contains(r#"Object.freeze(globalThis["Array"]['from'])"#));
    assert!(source.contains(r#"Object.freeze((null ?? globalThis['Array']['from']))"#));
    assert!(source.contains(r#"Object.freeze((globalThis["Array"])["from"])"#));
    assert!(source.contains(r#"Object.freeze((globalThis["Array"]).from)"#));
    assert!(source.contains(r#"Object.freeze((null ?? globalThis["Array"].from))"#));
    assert!(source.contains(r#"Object.freeze((true && globalThis["Array"].from))"#));
    assert!(source.contains(r#"Object.freeze((false || globalThis["Array"].from))"#));
    assert!(source.contains(r#"Object.freeze((null ?? globalThis["Array"]["from"]))"#));
    assert!(source.contains(r#"Object.freeze((true && globalThis["Array"]["from"]))"#));
    assert!(source.contains(r#"Object.freeze((false || globalThis["Array"]["from"]))"#));
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
        "const mod = await import(bundleJs.href);\nawait mod.browserArrayFromSetMapWrappers();\n",
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

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("1\n2\n1\n2\n1\n2\n1\n2\n1\n2\n1\n3\n4\n5\n1\n3\n4\n5\n"),
        "stdout: {stdout}"
    );
}

#[test]
fn build_emits_array_from_new_set_and_new_map_in_js_input() {
    assert_browser_bundle_array_from_set_map("app.js", false);
}

#[test]
fn json_build_emits_array_from_new_set_and_new_map_in_js_input() {
    assert_browser_bundle_array_from_set_map("app.js", true);
}

#[test]
fn build_emits_array_from_new_set_and_new_map_in_ts_input() {
    assert_browser_bundle_array_from_set_map("app.ts", false);
}

#[test]
fn json_build_emits_array_from_new_set_and_new_map_in_ts_input() {
    assert_browser_bundle_array_from_set_map("app.ts", true);
}

#[test]
fn build_emits_array_from_new_set_and_new_map_in_jsx_and_tsx_input() {
    for filename in ["app.jsx", "app.tsx"] {
        assert_browser_bundle_array_from_set_map(filename, false);
        assert_browser_bundle_array_from_set_map(filename, true);
    }
}
