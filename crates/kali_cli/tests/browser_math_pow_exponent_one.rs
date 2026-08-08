//! Task 18 pilot audit escalation residual: the 32 tests this file's
//! fixture-introspecting helper blocks from a clean migration.
//!
//! Every other `#[test]` from the original 48 in this file (the 16
//! `build_emits_*`/`json_build_emits_*` fns, which call
//! `assert_browser_bundle_math_pow_{exponent_one,base_one}_identity` --
//! neither of which reads fixture text) is migrated to
//! `tests/cases/browser/math_pow_exponent_one.toml` (4 cases, matrix-fanned
//! to 16 trials, audited clean). This file keeps exactly the 32 tests that
//! reach `assert_browser_harness_math_pow_exponent_one_identity` (16
//! directly, 16 more via the pass-through wrapper
//! `assert_browser_harness_math_pow_base_one_identity`), because that
//! helper's `expected_stdout` is computed by INSPECTING THE FIXTURE'S OWN
//! TEXT, not by asserting on process output:
//!
//! ```text
//! let expected_value = if source.contains("Math.pow(1, alias)") { "1" } else { "2" };
//! ...
//! if !json_output && source.contains("Kali.test(") { expected_stdout.push_str("\nok 1"); }
//! ```
//!
//! Post-trim, `Kali.test(` appears NOWHERE in the migrated `.toml`'s
//! `[source]` table (0 occurrences: neither of the two remaining fixtures --
//! the `exponent_one`/`base_one` browser-bundle sources -- is a `Kali.test`
//! wrapper at all). `Math.pow(1, alias)` DOES still appear there, 4 times,
//! but coincidentally, not as a trace of the introspecting helper: the
//! `app_base_one.${ext}` bundle fixture (base = 1) happens to construct
//! the exact same substring as an ordinary direct alias call
//! (`console.log(Math.pow(1, alias));`, plus its `globalThis.`-prefixed
//! sibling, each appearing once in the function body and once in the
//! `return [...]` entries) -- unrelated to, and not read by,
//! `assert_browser_harness_math_pow_exponent_one_identity`'s
//! `source.contains(...)` check, which only ever runs against the 32 tests
//! kept here. Either way this is moot for the audit: `[source]` is
//! deliberately excluded from `scripts/audit-case-migration.py`'s search
//! (its own docstring: "`body` and everything under `[source]` are program
//! text, not claims about behavior"), so neither literal's presence or
//! absence there is visible to it regardless. What actually blocks these 32
//! tests from migrating is that their own `.contains(...)` calls inspect
//! the FIXTURE'S OWN TEXT to select an assertion, not process output --
//! the audit script's literal-extraction regex cannot distinguish that from
//! a real output assertion, so it would flag the two literals as "claimed
//! but absent" (from an assertion field) no matter what the migrated
//! `[source]` table happens to contain, since neither literal is ever
//! actually asserted ON anywhere by these 32 tests -- they are read, not
//! checked. This is not a format gap (spec 5.11's usual trigger, e.g.
//! `soundness_abort.rs`'s dual-process byte comparison) -- the case-runner
//! format expresses this file's real assertions (exact `stdout`, exact
//! `json` fields) just fine, and the 16 tests that migrated prove it. It is
//! a tool blind spot specific to a helper that branches on the FIXTURE'S
//! OWN TEXT rather than on process output. Escalated per this task's rule 3
//! ("a claim the tool genuinely cannot see is a tool bug -- escalate, do
//! not disclose-and-ship") rather than silently fabricating a claim or
//! shipping with the audit red. See task-18-pilot-report.md for the full
//! account.
//!
//! This file is kept hand-written and trimmed to just these 32 tests and
//! the helpers they need (the four bundle-source builders and their
//! `assert_browser_bundle_*` helpers were deleted along with the 16
//! `build_emits_*`/`json_build_emits_*` tests that used them).
//! `math_pow_invocation_entries_for_aliases` is no longer imported since it
//! was only used by those now-deleted, now-migrated source builders.

use std::{fs, process::Command};

use kali_common::{
    math_pow_aliases, math_pow_frozen_callable_aliases, math_pow_invocation_lines_for_aliases,
};
use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn browser_harness_math_pow_identity_run_source(exponent_value: &str, pow_base: &str) -> String {
    let frozen_lines = math_pow_invocation_lines_for_aliases(
        &math_pow_frozen_callable_aliases(),
        pow_base,
        "alias",
        "",
    );

    format!(
        "const exponent = {exponent_value}; const alias = exponent; {direct_lines} {frozen_lines}\n",
        exponent_value = exponent_value,
        direct_lines = math_pow_invocation_lines_for_aliases(math_pow_aliases(), pow_base, "alias", ""),
        frozen_lines = frozen_lines,
    )
}

fn browser_harness_math_pow_identity_test_source(
    test_name: &str,
    exponent_value: &str,
    pow_base: &str,
) -> String {
    let frozen_lines = math_pow_invocation_lines_for_aliases(
        &math_pow_frozen_callable_aliases(),
        pow_base,
        "alias",
        "  ",
    );

    format!(
        r#"Kali.test('{test_name}', () => {{
  const exponent = {exponent_value};
  const alias = exponent;
{direct_lines}
{frozen_lines}
}});
"#,
        test_name = test_name,
        exponent_value = exponent_value,
        direct_lines =
            math_pow_invocation_lines_for_aliases(math_pow_aliases(), pow_base, "alias", ""),
        frozen_lines = frozen_lines,
    )
}

fn browser_harness_math_pow_exponent_one_identity_run_source() -> String {
    browser_harness_math_pow_identity_run_source("1", "2")
}

fn browser_harness_math_pow_exponent_one_identity_test_source() -> String {
    browser_harness_math_pow_identity_test_source("math pow exponent one identity", "1", "2")
}

fn browser_harness_math_pow_base_one_identity_run_source() -> String {
    browser_harness_math_pow_identity_run_source("7", "1")
}

fn browser_harness_math_pow_base_one_identity_test_source() -> String {
    browser_harness_math_pow_identity_test_source("math pow base one identity", "7", "1")
}

fn assert_browser_harness_math_pow_exponent_one_identity(
    command: &str,
    filename: &str,
    source: impl AsRef<str>,
    _expected_stdout: &str,
    json_output: bool,
) {
    let source = source.as_ref();
    let expected_value = if source.contains("Math.pow(1, alias)") {
        "1"
    } else {
        "2"
    };
    let mut expected_stdout =
        std::iter::repeat_n(expected_value, source.matches("console.log(").count())
            .collect::<Vec<_>>()
            .join("\n");
    if !json_output && source.contains("Kali.test(") {
        expected_stdout.push_str("\nok 1");
    }

    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut output = Command::new(kali_bin());
    output
        .env(kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV, "node")
        .current_dir(dir.path());
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
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], true);
        assert_eq!(json["payload"]["hostContract"], "browser-requested");
        assert_eq!(json["payload"]["runtimeBackend"], "browser-harness");
        if command == "run" {
            assert_eq!(json["exitCode"], 0);
            assert_eq!(json["payload"]["exitCode"], 0);
        } else {
            let payload = json["payload"].as_object().expect("payload object");
            assert_eq!(payload["total"], 1);
            assert_eq!(payload["passed"], 1);
            assert_eq!(payload["failed"], 0);
            assert_eq!(payload["skipped"], 0);
        }
        let stdout = json["stdout"].as_str().expect("stdout");
        assert!(stdout.contains(&expected_stdout), "json: {json}");
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains(&expected_stdout), "stdout: {stdout}");
    }
}

fn assert_browser_harness_math_pow_base_one_identity(
    command: &str,
    filename: &str,
    source: impl AsRef<str>,
    expected_stdout: &str,
    json_output: bool,
) {
    assert_browser_harness_math_pow_exponent_one_identity(
        command,
        filename,
        source,
        expected_stdout,
        json_output,
    );
}

#[test]
fn run_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_pow_exponent_one_identity(
        "run",
        "main.js",
        browser_harness_math_pow_exponent_one_identity_run_source(),
        "2\n2\n2\n2\n2\n2\n2\n2",
        false,
    );
}

#[test]
fn run_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_pow_exponent_one_identity(
        "run",
        "main.ts",
        browser_harness_math_pow_exponent_one_identity_run_source(),
        "2\n2\n2\n2\n2\n2\n2\n2",
        false,
    );
}

#[test]
fn run_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_math_pow_exponent_one_identity(
        "run",
        "main.jsx",
        browser_harness_math_pow_exponent_one_identity_run_source(),
        "2\n2\n2\n2\n2\n2\n2\n2",
        false,
    );
}

#[test]
fn run_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_math_pow_exponent_one_identity(
        "run",
        "main.tsx",
        browser_harness_math_pow_exponent_one_identity_run_source(),
        "2\n2\n2\n2\n2\n2\n2\n2",
        false,
    );
}

#[test]
fn json_run_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_js_input()
{
    assert_browser_harness_math_pow_exponent_one_identity(
        "run",
        "main.js",
        browser_harness_math_pow_exponent_one_identity_run_source(),
        "2\n2\n2\n2\n2\n2\n2\n2",
        true,
    );
}

#[test]
fn json_run_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_ts_input()
{
    assert_browser_harness_math_pow_exponent_one_identity(
        "run",
        "main.ts",
        browser_harness_math_pow_exponent_one_identity_run_source(),
        "2\n2\n2\n2\n2\n2\n2\n2",
        true,
    );
}

#[test]
fn json_run_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_jsx_input(
) {
    assert_browser_harness_math_pow_exponent_one_identity(
        "run",
        "main.jsx",
        browser_harness_math_pow_exponent_one_identity_run_source(),
        "2\n2\n2\n2\n2\n2\n2\n2",
        true,
    );
}

#[test]
fn json_run_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_tsx_input(
) {
    assert_browser_harness_math_pow_exponent_one_identity(
        "run",
        "main.tsx",
        browser_harness_math_pow_exponent_one_identity_run_source(),
        "2\n2\n2\n2\n2\n2\n2\n2",
        true,
    );
}

#[test]
fn test_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_pow_exponent_one_identity(
        "test",
        "smoke.test.js",
        browser_harness_math_pow_exponent_one_identity_test_source(),
        "2\n2\n2\n2\n2\n2\n2\n2\nok 1",
        false,
    );
}

#[test]
fn test_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_pow_exponent_one_identity(
        "test",
        "smoke.test.ts",
        browser_harness_math_pow_exponent_one_identity_test_source(),
        "2\n2\n2\n2\n2\n2\n2\n2\nok 1",
        false,
    );
}

#[test]
fn test_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_math_pow_exponent_one_identity(
        "test",
        "smoke.test.jsx",
        browser_harness_math_pow_exponent_one_identity_test_source(),
        "2\n2\n2\n2\n2\n2\n2\n2\nok 1",
        false,
    );
}

#[test]
fn test_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_math_pow_exponent_one_identity(
        "test",
        "smoke.test.tsx",
        browser_harness_math_pow_exponent_one_identity_test_source(),
        "2\n2\n2\n2\n2\n2\n2\n2\nok 1",
        false,
    );
}

#[test]
fn json_test_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_js_input(
) {
    assert_browser_harness_math_pow_exponent_one_identity(
        "test",
        "smoke.test.js",
        browser_harness_math_pow_exponent_one_identity_test_source(),
        "2\n2\n2\n2\n2\n2\n2\n2\n",
        true,
    );
}

#[test]
fn json_test_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_ts_input(
) {
    assert_browser_harness_math_pow_exponent_one_identity(
        "test",
        "smoke.test.ts",
        browser_harness_math_pow_exponent_one_identity_test_source(),
        "2\n2\n2\n2\n2\n2\n2\n2\n",
        true,
    );
}

#[test]
fn json_test_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_jsx_input(
) {
    assert_browser_harness_math_pow_exponent_one_identity(
        "test",
        "smoke.test.jsx",
        browser_harness_math_pow_exponent_one_identity_test_source(),
        "2\n2\n2\n2\n2\n2\n2\n2\n",
        true,
    );
}

#[test]
fn json_test_supports_math_pow_exponent_one_identity_when_browser_harness_is_configured_in_tsx_input(
) {
    assert_browser_harness_math_pow_exponent_one_identity(
        "test",
        "smoke.test.tsx",
        browser_harness_math_pow_exponent_one_identity_test_source(),
        "2\n2\n2\n2\n2\n2\n2\n2\n",
        true,
    );
}

#[test]
fn run_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "run",
        "main.js",
        browser_harness_math_pow_base_one_identity_run_source(),
        "1\n1\n1\n1\n1\n1\n1\n1",
        false,
    );
}

#[test]
fn json_run_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "run",
        "main.js",
        browser_harness_math_pow_base_one_identity_run_source(),
        "1\n1\n1\n1\n1\n1\n1\n1",
        true,
    );
}

#[test]
fn run_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "run",
        "main.ts",
        browser_harness_math_pow_base_one_identity_run_source(),
        "1\n1\n1\n1\n1\n1\n1\n1",
        false,
    );
}

#[test]
fn run_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "run",
        "main.jsx",
        browser_harness_math_pow_base_one_identity_run_source(),
        "1\n1\n1\n1\n1\n1\n1\n1",
        false,
    );
}

#[test]
fn run_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "run",
        "main.tsx",
        browser_harness_math_pow_base_one_identity_run_source(),
        "1\n1\n1\n1\n1\n1\n1\n1",
        false,
    );
}

#[test]
fn json_run_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "run",
        "main.ts",
        browser_harness_math_pow_base_one_identity_run_source(),
        "1\n1\n1\n1\n1\n1\n1\n1",
        true,
    );
}

#[test]
fn json_run_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "run",
        "main.jsx",
        browser_harness_math_pow_base_one_identity_run_source(),
        "1\n1\n1\n1\n1\n1\n1\n1",
        true,
    );
}

#[test]
fn json_run_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "run",
        "main.tsx",
        browser_harness_math_pow_base_one_identity_run_source(),
        "1\n1\n1\n1\n1\n1\n1\n1",
        true,
    );
}

#[test]
fn test_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "test",
        "smoke.test.js",
        browser_harness_math_pow_base_one_identity_test_source(),
        "1\n1\n1\n1\n1\n1\n1\n1\nok 1",
        false,
    );
}

#[test]
fn test_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "test",
        "smoke.test.ts",
        browser_harness_math_pow_base_one_identity_test_source(),
        "1\n1\n1\n1\n1\n1\n1\n1\nok 1",
        false,
    );
}

#[test]
fn test_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "test",
        "smoke.test.jsx",
        browser_harness_math_pow_base_one_identity_test_source(),
        "1\n1\n1\n1\n1\n1\n1\n1\nok 1",
        false,
    );
}

#[test]
fn test_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "test",
        "smoke.test.tsx",
        browser_harness_math_pow_base_one_identity_test_source(),
        "1\n1\n1\n1\n1\n1\n1\n1\nok 1",
        false,
    );
}

#[test]
fn json_test_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_js_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "test",
        "smoke.test.js",
        browser_harness_math_pow_base_one_identity_test_source(),
        "1\n1\n1\n1\n1\n1\n1\n1\n",
        true,
    );
}

#[test]
fn json_test_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_ts_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "test",
        "smoke.test.ts",
        browser_harness_math_pow_base_one_identity_test_source(),
        "1\n1\n1\n1\n1\n1\n1\n1\n",
        true,
    );
}

#[test]
fn json_test_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_jsx_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "test",
        "smoke.test.jsx",
        browser_harness_math_pow_base_one_identity_test_source(),
        "1\n1\n1\n1\n1\n1\n1\n1\n",
        true,
    );
}

#[test]
fn json_test_supports_math_pow_base_one_identity_when_browser_harness_is_configured_in_tsx_input() {
    assert_browser_harness_math_pow_base_one_identity(
        "test",
        "smoke.test.tsx",
        browser_harness_math_pow_base_one_identity_test_source(),
        "1\n1\n1\n1\n1\n1\n1\n1\n",
        true,
    );
}
