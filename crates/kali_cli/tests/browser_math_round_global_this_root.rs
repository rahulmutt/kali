//! Task 18 batch 5 design-spec 5.11 retention: kept 100% hand-written, not
//! migrated. No case file exists for this target.
//!
//! WHAT BLOCKS IT. All 9 `#[test]` fns in this file route through one of two
//! helpers -- `assert_browser_bundle_global_this_math_round` (`:191`), which
//! 8 of them call, and `assert_browser_harness_global_this_math_round`
//! (`:316`), which the ninth calls 16 times from an inlined loop -- and
//! BOTH helpers end in a line-oriented count -- `stdout.lines()` (`:302`) in the
//! bundle helper, and `stdout.lines()` (`:384`, JSON branch) and
//! `stdout.lines()` (`:432`, text branch) in the harness helper. The
//! harness helper's two sites sit on opposite arms of its output-shape `if`, so
//! every call reaches exactly one of them. 9 of 9 tests reach the construct
//! unconditionally, so U4's trim-and-keep degenerates to whole-file retention:
//! there is no complementary migratable subset to split off.
//!
//! MIGRATION NOTE (controller ruling 8): the retained looping fn
//! `run_and_test_supports_global_this_math_round_identity_when_browser_harness_is_configured_in_js_and_ts_input`
//! has a name that misdescribes its own body -- it says `js_and_ts`, but its
//! literal table covers js, ts, jsx AND tsx, so it makes 16 invocations rather
//! than the 8 the name implies. The source is NOT corrected: a fn name is not a
//! comment so U7 does not literally apply, and editing a source invalidates
//! every audit run against its blob. This file is a WHOLE-FILE retention with no
//! case file, so there is no `rationale` for the note to live in -- ruling 8's
//! usual home -- and it is recorded here instead, which is the only place a
//! later reader will look.
//!
//! The construct splits the captured output into lines, keeps the lines equal to
//! a literal, and pins how many there are. Design spec 5.4's closing paragraph
//! already places the line-oriented outliers outside the assertion vocabulary,
//! and none of the twelve keys reaches this shape. In particular the occurrence-
//! count keys added in batch 4 do NOT: they count NON-OVERLAPPING SUBSTRING
//! occurrences, as Rust's substring-match iterator does, whereas this construct
//! counts WHOLE LINES that equal the literal exactly. The two differ on real
//! output -- a line holding the literal plus anything else is counted by the
//! substring form and rejected by the line form -- so migrating onto a count key
//! would be a silent weakening, not a transcription. Rule 1 forbids that.
//!
//! ADJUDICATED, NOT PROPOSED. The human partner has ruled the line-oriented
//! sites design-spec 5.11 outliers: **no assertion key is being added for them.**
//! Do not reopen this by proposing a line-equality or line-count key.
//!
//! CONSEQUENCE FOR THE GATES (ruling 9): THIS FILE HAS NO RED-LIST, and that is
//! the finding, not an omission. Ruling 9 addresses a U4 trim-and-keep retention,
//! where the on-disk `.rs` is shorter than the source its case file was migrated
//! from. Nothing was trimmed here, so there is no pre-trim/post-trim divergence
//! and no pre-trim ref to run anything against; and there is no right-hand side,
//! since `verify_pair.sh math_round_global_this_root` exits 2 with a missing case
//! gate. FIVE of the six gates take a `.rs`/`.toml` pair and therefore cannot
//! run here at all. The SIXTH is the exception, and it changes this paragraph:
//! `batch5_crosscheck.py`, the citation gate that batch 6 wired into
//! `verify_pair.sh`, needs no case file -- it resolves THIS header's own `:N`
//! citations against this very file. So a whole-file retention is no longer
//! ungated: run it directly, as
//! `batch5_crosscheck.py --citations-only math_round_global_this_root`, because `verify_pair.sh`
//! still exits 2 before reaching it. It exits 0 today. Ruling 11 exempts `:N` from the
//! no-moving-numbers rule only because it is mechanically gated, and this is
//! where that gating applies to a file with no pair. Verified by running it, not assumed.
//!
//! Escalated per rule 3/4 rather than shipped with a false green or a fabricated
//! claim. This file must NOT be deleted by the family-wide sweep after batch 8.
//! See `.superpowers/sdd/2026-07-29-test-binary-consolidation/
//! task-18-batch5-report.md` for the full account.
use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_global_this_math_round_source() -> &'static str {
    r##"// kali-tree-shake: globalThisMathRoundIdentity
function globalThisMathRoundIdentity() {
  const value = 1.6;
  const frozenValue = Object.freeze(value);
  console.log(globalThis.Math.round(value));
  console.log(globalThis.Math["round"](value));
  console.log(globalThis["Math"]["round"](value));
  console.log("optional-chain", globalThis?.Math.round(value));
  console.log("frozen-optional-chain", Object.freeze(globalThis?.Math.round)(value));
  console.log("frozen-nullish-direct", Object.freeze((null ?? Math.round))(value));
  console.log("frozen-logical-and-root", Object.freeze((true && globalThis.Math.round))(value));
  console.log("frozen-logical-or-bracket", Object.freeze((false || globalThis["Math"]["round"]))(value));
  const frozenParenthesizedOptionalChain = Object.freeze((globalThis?.Math.round))(value);
  if (frozenParenthesizedOptionalChain !== 2) {
    throw new Error("unexpected frozen parenthesized optional-chain identity");
  }
  console.log("frozen-parenthesized-mixed-bracket", Object.freeze((globalThis.Math["round"]))(value));
  const frozenParenthesizedBracketedDotRoot = Object.freeze((globalThis["Math"].round))(value);
  if (frozenParenthesizedBracketedDotRoot !== 2) {
    throw new Error("unexpected frozen parenthesized bracketed-dot root identity");
  }
  console.log("frozen-parenthesized-bracketed-dot-root", frozenParenthesizedBracketedDotRoot);
  console.log(Math.round(frozenValue));
  console.log(Object.freeze(globalThis["Math"]["round"])(value));
  console.log(Object.freeze((globalThis["Math"])["round"])(value));
  console.log(Object.freeze((globalThis['Math'])['round'])(value));
  console.log(Object.freeze((globalThis['Math'])["round"])(value));
  console.log("frozen-mixed-bracket-root", Object.freeze(globalThis.Math["round"])(value));
  console.log("frozen-bracketed-dot-root", Object.freeze(globalThis["Math"].round)(value));
  console.log("frozen-parenthesized-bracket-root", Object.freeze((globalThis["Math"]).round)(value));
  console.log("frozen-parenthesized-single-quoted-bracket-root", Object.freeze((globalThis['Math']).round)(value));
  console.log(Object.freeze(globalThis.Math.round)(value));
  console.log(Object.freeze(globalThis.Math['round'])(value));
  console.log(Object.freeze(globalThis["Math"]['round'])(value));
  console.log(Object.freeze(globalThis['Math'].round)(value));
  console.log(Object.freeze(Math.round)(value));
  console.log("frozen-parenthesized-direct", Object.freeze((Math.round))(value));
  if (Object.freeze((Math.round))(value) !== 2) {
    throw new Error("unexpected frozen parenthesized direct identity");
  }
  console.log(Object.freeze(globalThis['Math']['round'])(value));
  console.log("frozen-parenthesized-mixed-quoted-bracketed-root", Object.freeze((globalThis["Math"]['round']))(value));
  console.log("frozen-parenthesized-single-quoted-bracketed-dot-root", Object.freeze((globalThis['Math'].round))(value));
  return [
    globalThis.Math.round(value),
    globalThis.Math["round"](value),
    globalThis["Math"]["round"](value),
    globalThis?.Math.round(value),
    Object.freeze(globalThis?.Math.round)(value),
    frozenParenthesizedOptionalChain,
    frozenParenthesizedBracketedDotRoot,
    Object.freeze((globalThis.Math["round"]))(value),
    Math.round(frozenValue),
    Object.freeze(globalThis["Math"]["round"])(value),
    Object.freeze((globalThis["Math"])["round"])(value),
    Object.freeze((globalThis['Math'])['round'])(value),
    Object.freeze((globalThis['Math'])["round"])(value),
    Object.freeze(globalThis.Math["round"])(value),
    Object.freeze(globalThis["Math"].round)(value),
    Object.freeze((globalThis["Math"]).round)(value),
    Object.freeze((globalThis['Math']).round)(value),
    Object.freeze(globalThis.Math.round)(value),
    Object.freeze(Math.round)(value),
    Object.freeze(globalThis['Math']['round'])(value),
    Object.freeze((globalThis["Math"]['round']))(value),
    Object.freeze((globalThis['Math'].round))(value),
  ];
}
"##
}
fn browser_harness_global_this_math_round_run_source() -> &'static str {
    r#"const value = 1.6; const frozenValue = Object.freeze(value); console.log(globalThis.Math.round(value)); console.log(globalThis.Math["round"](value)); console.log(globalThis["Math"]["round"](value)); console.log("optional-chain", globalThis?.Math.round(value)); console.log("frozen-optional-chain", Object.freeze(globalThis?.Math.round)(value)); console.log("frozen-nullish-direct", Object.freeze((null ?? Math.round))(value)); console.log("frozen-logical-and-root", Object.freeze((true && globalThis.Math.round))(value)); console.log("frozen-logical-or-bracket", Object.freeze((false || globalThis["Math"]["round"]))(value)); const frozenParenthesizedOptionalChain = Object.freeze((globalThis?.Math.round))(value); if (frozenParenthesizedOptionalChain !== 2) { throw new Error("unexpected frozen parenthesized optional-chain identity"); } console.log("frozen-parenthesized-mixed-bracket", Object.freeze((globalThis.Math["round"]))(value)); const frozenParenthesizedBracketedDotRoot = Object.freeze((globalThis["Math"].round))(value); if (frozenParenthesizedBracketedDotRoot !== 2) { throw new Error("unexpected frozen parenthesized bracketed-dot root identity"); } console.log("frozen-parenthesized-bracketed-dot-root", frozenParenthesizedBracketedDotRoot); console.log(Math.round(frozenValue)); console.log(Object.freeze(globalThis["Math"]["round"])(value)); console.log(Object.freeze((globalThis["Math"])["round"])(value)); console.log(Object.freeze((globalThis['Math'])['round'])(value)); console.log(Object.freeze((globalThis['Math'])["round"])(value)); console.log("frozen-mixed-bracket-root", Object.freeze(globalThis.Math["round"])(value)); console.log("frozen-bracketed-dot-root", Object.freeze(globalThis["Math"].round)(value)); console.log("frozen-parenthesized-bracket-root", Object.freeze((globalThis["Math"]).round)(value)); console.log("frozen-parenthesized-single-quoted-bracket-root", Object.freeze((globalThis['Math']).round)(value)); console.log(Object.freeze(globalThis.Math.round)(value)); console.log(Object.freeze(globalThis.Math['round'])(value)); console.log(Object.freeze(globalThis["Math"]['round'])(value)); console.log(Object.freeze(globalThis['Math'].round)(value)); console.log(Object.freeze(Math.round)(value)); console.log("frozen-parenthesized-direct", Object.freeze((Math.round))(value)); console.log(Object.freeze(globalThis['Math']['round'])(value)); console.log("frozen-parenthesized-mixed-quoted-bracketed-root", Object.freeze((globalThis["Math"]['round']))(value)); console.log("frozen-parenthesized-single-quoted-bracketed-dot-root", Object.freeze((globalThis['Math'].round))(value));
"#
}
fn browser_harness_global_this_math_round_test_source() -> &'static str {
    r#"Kali.test('globalThis.Math round identity', () => {
  const value = 1.6;
  const frozenValue = Object.freeze(value);
  console.log(globalThis.Math.round(value));
  console.log(globalThis.Math["round"](value));
  console.log(globalThis["Math"]["round"](value));
  console.log("optional-chain", globalThis?.Math.round(value));
  console.log("frozen-optional-chain", Object.freeze(globalThis?.Math.round)(value));
  console.log("frozen-nullish-direct", Object.freeze((null ?? Math.round))(value));
  console.log("frozen-logical-and-root", Object.freeze((true && globalThis.Math.round))(value));
  console.log("frozen-logical-or-bracket", Object.freeze((false || globalThis["Math"]["round"]))(value));
  const frozenParenthesizedOptionalChain = Object.freeze((globalThis?.Math.round))(value);
  if (frozenParenthesizedOptionalChain !== 2) {
    throw new Error("unexpected frozen parenthesized optional-chain identity");
  }
  console.log("frozen-parenthesized-mixed-bracket", Object.freeze((globalThis.Math["round"]))(value));
  const frozenParenthesizedBracketedDotRoot = Object.freeze((globalThis["Math"].round))(value);
  if (frozenParenthesizedBracketedDotRoot !== 2) {
    throw new Error("unexpected frozen parenthesized bracketed-dot root identity");
  }
  console.log("frozen-parenthesized-bracketed-dot-root", frozenParenthesizedBracketedDotRoot);
  console.log(Math.round(frozenValue));
  console.log(Object.freeze(globalThis["Math"]["round"])(value));
  console.log(Object.freeze((globalThis["Math"])["round"])(value));
  console.log(Object.freeze((globalThis['Math'])['round'])(value));
  console.log(Object.freeze((globalThis['Math'])["round"])(value));
  console.log("frozen-mixed-bracket-root", Object.freeze(globalThis.Math["round"])(value));
  console.log("frozen-bracketed-dot-root", Object.freeze(globalThis["Math"].round)(value));
  console.log(Object.freeze(globalThis.Math.round)(value));
  console.log(Object.freeze(globalThis.Math['round'])(value));
  console.log(Object.freeze(globalThis["Math"]['round'])(value));
  console.log(Object.freeze(globalThis['Math'].round)(value));
  console.log(Object.freeze(Math.round)(value));
  console.log("frozen-parenthesized-direct", Object.freeze((Math.round))(value));
  if (Object.freeze((Math.round))(value) !== 2) {
    throw new Error("unexpected frozen parenthesized direct identity");
  }
  console.log(Object.freeze(globalThis['Math']['round'])(value));
  console.log("frozen-parenthesized-mixed-quoted-bracketed-root", Object.freeze((globalThis["Math"]['round']))(value));
  console.log("frozen-parenthesized-single-quoted-bracketed-dot-root", Object.freeze((globalThis['Math'].round))(value));
});
"#
}
fn assert_browser_bundle_global_this_math_round(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, browser_bundle_global_this_math_round_source()).expect("write source");

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
        assert_eq!(envelope["errors"], serde_json::Value::Array(vec![]));
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
await mod.globalThisMathRoundIdentity();
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

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("frozen-mixed-bracket-root"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("frozen-parenthesized-bracketed-dot-root"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("frozen-parenthesized-mixed-quoted-bracketed-root"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("frozen-parenthesized-single-quoted-bracketed-dot-root"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("frozen-bracketed-dot-root"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("frozen-parenthesized-bracket-root"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("frozen-parenthesized-single-quoted-bracket-root"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("frozen-parenthesized-direct"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains("frozen-nullish-direct"), "stdout: {stdout}");
    assert!(
        stdout.contains("frozen-logical-and-root"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("frozen-logical-or-bracket"),
        "stdout: {stdout}"
    );
    assert_eq!(
        stdout.lines().filter(|line| *line == "2").count(),
        14,
        "stdout: {stdout}"
    );
}

fn assert_browser_harness_global_this_math_round(
    command: &str,
    filename: &str,
    source: &str,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut output = Command::new(kali_bin());
    output
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node");
    if json_output {
        output.arg("--output").arg("json");
    }
    let output = output
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
        if command == "run" {
            assert_eq!(json["exitCode"], 0);
            assert_eq!(json["payload"]["exitCode"], 0);
        } else {
            assert_eq!(json["payload"]["total"], 1);
            assert_eq!(json["payload"]["passed"], 1);
            assert_eq!(json["payload"]["failed"], 0);
        }
        let stdout = json["stdout"].as_str().expect("stdout string");
        assert!(stdout.contains("optional-chain"), "json: {json}");
        assert!(stdout.contains("frozen-optional-chain"), "json: {json}");
        assert!(
            stdout.contains("frozen-parenthesized-mixed-bracket"),
            "json: {json}"
        );
        assert!(stdout.contains("frozen-mixed-bracket-root"), "json: {json}");
        assert!(
            stdout.contains("frozen-parenthesized-bracketed-dot-root"),
            "json: {json}"
        );
        assert!(
            stdout.contains("frozen-parenthesized-mixed-quoted-bracketed-root"),
            "json: {json}"
        );
        assert!(
            stdout.contains("frozen-parenthesized-single-quoted-bracketed-dot-root"),
            "json: {json}"
        );
        assert!(stdout.contains("frozen-bracketed-dot-root"), "json: {json}");
        assert!(stdout.contains("frozen-nullish-direct"), "json: {json}");
        assert!(stdout.contains("frozen-logical-and-root"), "json: {json}");
        assert!(stdout.contains("frozen-logical-or-bracket"), "json: {json}");
        assert_eq!(
            stdout.lines().filter(|line| *line == "2").count(),
            14,
            "json: {json}"
        );
        assert_eq!(json["stderr"], "");
        assert_eq!(json["errors"], serde_json::Value::Array(vec![]));
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("optional-chain"), "stdout: {stdout}");
        assert!(stdout.contains("frozen-optional-chain"), "stdout: {stdout}");
        assert!(
            stdout.contains("frozen-parenthesized-mixed-bracket"),
            "stdout: {stdout}"
        );
        assert!(
            stdout.contains("frozen-mixed-bracket-root"),
            "stdout: {stdout}"
        );
        assert!(
            stdout.contains("frozen-parenthesized-bracketed-dot-root"),
            "stdout: {stdout}"
        );
        assert!(
            stdout.contains("frozen-parenthesized-mixed-quoted-bracketed-root"),
            "stdout: {stdout}"
        );
        assert!(
            stdout.contains("frozen-parenthesized-single-quoted-bracketed-dot-root"),
            "stdout: {stdout}"
        );
        assert!(
            stdout.contains("frozen-bracketed-dot-root"),
            "stdout: {stdout}"
        );
        assert!(
            stdout.contains("frozen-parenthesized-direct"),
            "stdout: {stdout}"
        );
        assert!(stdout.contains("frozen-nullish-direct"), "stdout: {stdout}");
        assert!(
            stdout.contains("frozen-logical-and-root"),
            "stdout: {stdout}"
        );
        assert!(
            stdout.contains("frozen-logical-or-bracket"),
            "stdout: {stdout}"
        );
        assert_eq!(
            stdout.lines().filter(|line| *line == "2").count(),
            14,
            "stdout: {stdout}"
        );
    }
}

#[test]
fn build_emits_global_this_math_round_identity_literals_in_js_input() {
    assert_browser_bundle_global_this_math_round("app.js", false);
}

#[test]
fn build_emits_global_this_math_round_identity_literals_in_ts_input() {
    assert_browser_bundle_global_this_math_round("app.ts", false);
}

#[test]
fn build_emits_global_this_math_round_identity_literals_in_jsx_input() {
    assert_browser_bundle_global_this_math_round("app.jsx", false);
}

#[test]
fn build_emits_global_this_math_round_identity_literals_in_tsx_input() {
    assert_browser_bundle_global_this_math_round("app.tsx", false);
}

#[test]
fn json_build_emits_global_this_math_round_identity_literals_in_js_input() {
    assert_browser_bundle_global_this_math_round("app.js", true);
}

#[test]
fn json_build_emits_global_this_math_round_identity_literals_in_ts_input() {
    assert_browser_bundle_global_this_math_round("app.ts", true);
}

#[test]
fn json_build_emits_global_this_math_round_identity_literals_in_jsx_input() {
    assert_browser_bundle_global_this_math_round("app.jsx", true);
}

#[test]
fn json_build_emits_global_this_math_round_identity_literals_in_tsx_input() {
    assert_browser_bundle_global_this_math_round("app.tsx", true);
}

#[test]
fn run_and_test_supports_global_this_math_round_identity_when_browser_harness_is_configured_in_js_and_ts_input(
) {
    for (command, source_name, source) in [
        (
            "run",
            "main.js",
            browser_harness_global_this_math_round_run_source(),
        ),
        (
            "test",
            "smoke.test.js",
            browser_harness_global_this_math_round_test_source(),
        ),
        (
            "run",
            "main.ts",
            browser_harness_global_this_math_round_run_source(),
        ),
        (
            "test",
            "smoke.test.ts",
            browser_harness_global_this_math_round_test_source(),
        ),
        (
            "run",
            "main.jsx",
            browser_harness_global_this_math_round_run_source(),
        ),
        (
            "test",
            "smoke.test.jsx",
            browser_harness_global_this_math_round_test_source(),
        ),
        (
            "run",
            "main.tsx",
            browser_harness_global_this_math_round_run_source(),
        ),
        (
            "test",
            "smoke.test.tsx",
            browser_harness_global_this_math_round_test_source(),
        ),
    ] {
        for output_json in [false, true] {
            assert_browser_harness_global_this_math_round(
                command,
                source_name,
                source,
                output_json,
            );
        }
    }
}
