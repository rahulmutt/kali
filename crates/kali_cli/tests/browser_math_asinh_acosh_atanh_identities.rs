//! Task 18 batch 3 escalation (fix round 1): kept hand-written, not migrated.
//! No case file exists for this target *yet*.
//!
//! ### SUPERSEDED — the adjudication below was REVERSED. This file is now
//! ### migratable, and batch 4 owns migrating it.
//!
//! The "no eleventh assertion key" call recorded further down was overturned
//! by the human partner during the Task 18 batch 4 interlude. Two count keys
//! now exist in the case format (design spec §5.4, which opens "Twelve
//! assertion keys"):
//!
//!     stdout_count = [{ needle = "0\n", at_least = 3 }]
//!     json_count   = [{ path = "stdout", needle = "0\n", at_least = 3 }]
//!
//! `at_least` is non-overlapping `str::matches` counting, exactly the
//! semantics `.matches(needle).count()` has here -- so it carries this file's
//! claim at equal strength, with no weakening (rule 1) and no invented
//! adjacency claim (rule 2). Both of this file's surfaces are covered: `:202`
//! and `:257` count in raw stdout (`stdout_count`); `:253`, inside the
//! `if json_output` branch, counts in `json["stdout"].as_str()`
//! (`json_count`). All three sites are the same `>= 3` bound, so all 24
//! `#[test]` fns are expressible.
//!
//! Nothing below this line has been deleted, because the reasoning is still
//! accurate about *why the format could not carry the claim before the keys
//! existed* -- it is the record that motivated adding them. Read it as
//! history. The one sentence that is now simply wrong is the ADJUDICATED
//! paragraph's "**No eleventh assertion key is being added for it**"; that is
//! the ruling that was reversed. The §5.11 retention this file currently
//! holds stands only until batch 4 migrates it.
//!
//! DO NOT migrate this file from here. Batch 4 owns it, and the keys landed
//! deliberately ahead of any migration so that batch could do it under review.
//!
//! ALL 24 `#[test]` fns in this file make a COUNT claim about stdout, which
//! the case-file format cannot carry. U4's trim-and-keep was applied first
//! and degenerates to whole-file retention: there is no complementary
//! migratable subset to split off. The 8 `build_emits_*` /
//! `json_build_emits_*` fns all reach
//! `assert_browser_bundle_math_inverse_hyperbolic` (`:122-203`), whose only
//! stdout assertion is at `:202`. The other 16 all reach
//! `assert_browser_harness_math_inverse_hyperbolic` (`:205-259`), whose
//! `if json_output` branch asserts the same shape at `:253` and whose `else`
//! branch asserts it at `:257`. Every one of the 24 fns reaches exactly one
//! of those three lines, and each is the same claim:
//!
//!     stdout.matches(<needle>).count() >= 3
//!
//! WHY IT CANNOT BE MIGRATED. `stdout_contains` with the bare needle is
//! satisfied by a single occurrence, so it is a WEAKENING and rule 1 forbids
//! it. An exact `stdout` pin is barred by controller ruling 3, which keeps a
//! substring-shaped source claim as `*_contains` whenever the field has a
//! substring form. The remaining option -- a contiguous three-in-a-row needle,
//! which does imply the count, since `str::matches` counts non-overlapping
//! matches -- substitutes an ADJACENCY claim the source deliberately does not
//! make: "three zeroes anywhere in stdout" is not "three zeroes in a row".
//! Rule 2 forbids inventing a claim the source never made. That contiguous
//! encoding was shipped by batch 3's commit `50061950a4` and the controller
//! reversed it; this retention is the reversal.
//!
//! ADJUDICATED: `.matches(...).count()` is a design-spec 5.11 outlier, in the
//! same class as the `starts_with` / `lines()` sites that 5.4's closing
//! paragraph already places outside the assertion vocabulary. **No eleventh
//! assertion key is being added for it** -- do not reopen this by proposing a
//! `stdout_matches_count` key. This is the same call the human partner made on
//! the universally-quantified JSON-array claims in
//! `browser_generator_default_export_rejection.rs`, which is retained whole on
//! that ground.
//!
//! The distinction from an already-exact count claim matters, so a later
//! reader does not read this as barring exact pins generally: the pilot's
//! `cases/browser/bundle_toplevel_start.toml` legitimately carries an exact
//! `stdout` pin because ITS source asserts `.count() == 1` -- an exact
//! assertion, which ruling 3 maps straight onto an exact pin. `>= 3` is an
//! inequality, and no observed output resolves it without either weakening
//! the claim or inventing a stronger one.
//!
//! `tests/cases/browser/math_asinh_acosh_atanh_identities.toml` was shipped by
//! `50061950a4` and is DELETED in the same commit as this header. The 24
//! trials it contributed are gone from the `cases` target, and these 24 Rust
//! `#[test]` fns are once again the only coverage of this behaviour.
//!
//! This file must NOT be deleted by the family-wide sweep after batch 8. See
//! `.superpowers/sdd/2026-07-29-test-binary-consolidation/
//! task-18-batch3-report.md` for the full account.
use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_bundle_math_inverse_hyperbolic_source() -> &'static str {
    r##"// kali-tree-shake: mathInverseHyperbolicIdentities
function mathInverseHyperbolicIdentities() {
  const zero = 0;
  const one = 1;
  console.log(Math.asinh(zero));
  console.log(Math.acosh(one));
  console.log(Math.atanh(zero));
}
"##
}

fn browser_harness_math_inverse_hyperbolic_run_source() -> &'static str {
    "const zero = 0; const one = 1; console.log(Math.asinh(zero)); console.log(Math.acosh(one)); console.log(Math.atanh(zero));\n"
}

fn browser_harness_math_inverse_hyperbolic_test_source() -> &'static str {
    r#"Kali.test('math inverse hyperbolic identities', () => {
  const zero = 0;
  const one = 1;
  console.log(Math.asinh(zero));
  console.log(Math.acosh(one));
  console.log(Math.atanh(zero));
});
"#
}

fn assert_browser_bundle_math_inverse_hyperbolic(filename: &str, json_output: bool) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(
        &source_path,
        browser_bundle_math_inverse_hyperbolic_source(),
    )
    .expect("write source");

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
await mod.mathInverseHyperbolicIdentities();
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
    assert!(stdout.matches("0\n").count() >= 3, "stdout: {stdout}");
}

fn assert_browser_harness_math_inverse_hyperbolic(
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
        assert!(stdout.matches("0\n").count() >= 3, "json: {json}");
        assert_eq!(json["stderr"], "");
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.matches("0\n").count() >= 3, "stdout: {stdout}");
    }
}

#[test]
fn build_emits_math_inverse_hyperbolic_identity_literals_in_js_input() {
    assert_browser_bundle_math_inverse_hyperbolic("app.js", false);
}

#[test]
fn build_emits_math_inverse_hyperbolic_identity_literals_in_ts_input() {
    assert_browser_bundle_math_inverse_hyperbolic("app.ts", false);
}

#[test]
fn json_build_emits_math_inverse_hyperbolic_identity_literals_in_js_input() {
    assert_browser_bundle_math_inverse_hyperbolic("app.js", true);
}

#[test]
fn json_build_emits_math_inverse_hyperbolic_identity_literals_in_ts_input() {
    assert_browser_bundle_math_inverse_hyperbolic("app.ts", true);
}

#[test]
fn run_supports_math_inverse_hyperbolic_identity_literals_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_harness_math_inverse_hyperbolic(
        "run",
        "main.ts",
        browser_harness_math_inverse_hyperbolic_run_source(),
        false,
    );
}

#[test]
fn run_supports_math_inverse_hyperbolic_identity_literals_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_harness_math_inverse_hyperbolic(
        "run",
        "main.js",
        browser_harness_math_inverse_hyperbolic_run_source(),
        false,
    );
}

#[test]
fn test_supports_math_inverse_hyperbolic_identity_literals_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_harness_math_inverse_hyperbolic(
        "test",
        "smoke.test.ts",
        browser_harness_math_inverse_hyperbolic_test_source(),
        false,
    );
}

#[test]
fn test_supports_math_inverse_hyperbolic_identity_literals_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_harness_math_inverse_hyperbolic(
        "test",
        "smoke.test.js",
        browser_harness_math_inverse_hyperbolic_test_source(),
        false,
    );
}

#[test]
fn run_supports_math_inverse_hyperbolic_identity_literals_when_browser_harness_is_configured_in_json_ts_input(
) {
    assert_browser_harness_math_inverse_hyperbolic(
        "run",
        "main.ts",
        browser_harness_math_inverse_hyperbolic_run_source(),
        true,
    );
}

#[test]
fn run_supports_math_inverse_hyperbolic_identity_literals_when_browser_harness_is_configured_in_json_js_input(
) {
    assert_browser_harness_math_inverse_hyperbolic(
        "run",
        "main.js",
        browser_harness_math_inverse_hyperbolic_run_source(),
        true,
    );
}

#[test]
fn test_supports_math_inverse_hyperbolic_identity_literals_when_browser_harness_is_configured_in_json_ts_input(
) {
    assert_browser_harness_math_inverse_hyperbolic(
        "test",
        "smoke.test.ts",
        browser_harness_math_inverse_hyperbolic_test_source(),
        true,
    );
}

#[test]
fn test_supports_math_inverse_hyperbolic_identity_literals_when_browser_harness_is_configured_in_json_js_input(
) {
    assert_browser_harness_math_inverse_hyperbolic(
        "test",
        "smoke.test.js",
        browser_harness_math_inverse_hyperbolic_test_source(),
        true,
    );
}

#[test]
fn build_emits_math_inverse_hyperbolic_identity_literals_in_jsx_input() {
    assert_browser_bundle_math_inverse_hyperbolic("app.jsx", false);
}

#[test]
fn build_emits_math_inverse_hyperbolic_identity_literals_in_tsx_input() {
    assert_browser_bundle_math_inverse_hyperbolic("app.tsx", false);
}

#[test]
fn json_build_emits_math_inverse_hyperbolic_identity_literals_in_jsx_input() {
    assert_browser_bundle_math_inverse_hyperbolic("app.jsx", true);
}

#[test]
fn json_build_emits_math_inverse_hyperbolic_identity_literals_in_tsx_input() {
    assert_browser_bundle_math_inverse_hyperbolic("app.tsx", true);
}

#[test]
fn run_supports_math_inverse_hyperbolic_identity_literals_when_browser_harness_is_configured_in_jsx_input(
) {
    assert_browser_harness_math_inverse_hyperbolic(
        "run",
        "main.jsx",
        browser_harness_math_inverse_hyperbolic_run_source(),
        false,
    );
}

#[test]
fn run_supports_math_inverse_hyperbolic_identity_literals_when_browser_harness_is_configured_in_tsx_input(
) {
    assert_browser_harness_math_inverse_hyperbolic(
        "run",
        "main.tsx",
        browser_harness_math_inverse_hyperbolic_run_source(),
        false,
    );
}

#[test]
fn test_supports_math_inverse_hyperbolic_identity_literals_when_browser_harness_is_configured_in_jsx_input(
) {
    assert_browser_harness_math_inverse_hyperbolic(
        "test",
        "smoke.test.jsx",
        browser_harness_math_inverse_hyperbolic_test_source(),
        false,
    );
}

#[test]
fn test_supports_math_inverse_hyperbolic_identity_literals_when_browser_harness_is_configured_in_tsx_input(
) {
    assert_browser_harness_math_inverse_hyperbolic(
        "test",
        "smoke.test.tsx",
        browser_harness_math_inverse_hyperbolic_test_source(),
        false,
    );
}

#[test]
fn run_supports_math_inverse_hyperbolic_identity_literals_when_browser_harness_is_configured_in_json_jsx_input(
) {
    assert_browser_harness_math_inverse_hyperbolic(
        "run",
        "main.jsx",
        browser_harness_math_inverse_hyperbolic_run_source(),
        true,
    );
}

#[test]
fn run_supports_math_inverse_hyperbolic_identity_literals_when_browser_harness_is_configured_in_json_tsx_input(
) {
    assert_browser_harness_math_inverse_hyperbolic(
        "run",
        "main.tsx",
        browser_harness_math_inverse_hyperbolic_run_source(),
        true,
    );
}

#[test]
fn test_supports_math_inverse_hyperbolic_identity_literals_when_browser_harness_is_configured_in_json_jsx_input(
) {
    assert_browser_harness_math_inverse_hyperbolic(
        "test",
        "smoke.test.jsx",
        browser_harness_math_inverse_hyperbolic_test_source(),
        true,
    );
}

#[test]
fn test_supports_math_inverse_hyperbolic_identity_literals_when_browser_harness_is_configured_in_json_tsx_input(
) {
    assert_browser_harness_math_inverse_hyperbolic(
        "test",
        "smoke.test.tsx",
        browser_harness_math_inverse_hyperbolic_test_source(),
        true,
    );
}
