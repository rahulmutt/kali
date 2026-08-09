//! Task 18 batch 6A design-spec 5.11 retention, TRIMMED: this file now holds
//! exactly the three `#[test]` fns whose JSON `errors` quantifier blocks them from
//! migrating, plus the one assert helper and the two fixture builders they read.
//!
//! It originally had 6 `#[test]` fns. The other 3 -- every `*_supports_math_sqrt_*`
//! fn, 20 real invocations of `assert_browser_harness_math_sqrt_success` -- are
//! migrated to
//! `tests/cases/browser/math_unsupported_member_calls_harness_jsx_tsx.toml`
//! (20 named `[[case]]` entries and no `[matrix]`; the three migrated fns cover
//! different extension sets, which the case file's header records). That helper,
//! its `///` doc, and the two non-atan2 fixture builders went with them; nothing
//! left here is unused.
//!
//! WHAT BLOCKS THE THREE RETAINED TESTS. Each of
//! `run_rejects_broader_math_atan2_member_calls_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input`
//! (`:225`),
//! `test_rejects_broader_math_atan2_member_calls_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input`
//! (`:246`) and
//! `build_rejects_broader_math_atan2_member_calls_in_browser_api_surface_with_harness_jsx_and_tsx_input`
//! (`:267`) routes through
//! `assert_browser_harness_unsupported_math_rejection` (`:154`), and each
//! calls it twice per extension inside its own `for extension in [...]` loop, once
//! with the JSON-output flag false and once true. The true call is unconditional,
//! so 3 of 3 retained tests reach the blocking construct on every iteration.
//!
//! The blocking construct is a QUANTIFIER over the JSON `errors` array,
//! `errors.iter().all(...)` (`:194`). The case-file format offers only closed
//! dotted-path indexing into JSON -- design spec 5.4 is explicit that there are
//! "no slices, no wildcards, no negative-from-end indexing, no filters" -- so a
//! dotted path can pin the FIRST array element and nothing more. Narrowing "every
//! error has this code" to "error 0 has this code" is a weakening (a second,
//! differently-coded diagnostic would satisfy the migration and fail the source),
//! and rule 1 forbids weakening. Nothing in the twelve assertion keys expresses it.
//! The neighbouring `assert!(!errors.is_empty(), ...)` (`:192`) is not the
//! problem and would migrate on its own; the quantifier is.
//!
//! ADJUDICATED, NOT PROPOSED. The human partner has ruled this quantifier a design
//! spec 5.11 outlier, in the same class as the position-anchored and line-oriented
//! sites 5.4's closing paragraph already places outside the vocabulary: **no
//! assertion key is being added for it.** The identical shape is recorded in
//! batch 5's `browser_math_pow_optional_chain_harness.rs` and in batch 6A's
//! `browser_non_literal_dynamic_import_harness_jsx_tsx.rs`. Do not reopen this by
//! proposing a wildcard dotted path or a quantified-array key.
//!
//! The non-JSON half of these three tests WOULD have migrated cleanly -- its arm is
//! `stderr.contains("E5506")` (`:199`) plus a three-way OR that rule 11
//! resolves against the real binary -- but a source `#[test]` fn cannot be split
//! across the two halves: each fn's own loop body runs both, on every iteration.
//! U4's trim-and-keep unit is the `#[test]` fn, so the split is 3 migrated / 3
//! retained and not finer.
//!
//! CONSEQUENCE FOR THE GATES -- THE COMPLETE RED-LIST (ruling 9). Every line below
//! was produced by RUNNING the gate, on every side named, not by reasoning about
//! it, and it is NOT copied from another retention's list -- batch 5 proved that
//! copying would have been wrong on all four pairs it checked.
//!
//!   PRE-TRIM REF:  fe6a403411   (the commit before batch 6A's migration commit)
//!   git show fe6a403411:crates/kali_cli/tests/browser_math_unsupported_member_calls_harness_jsx_tsx.rs > /tmp/pretrim.rs
//!
//! THIS RETENTION NEEDS A THIRD LEFT-HAND SIDE, AND SO DOES EVERY TRIM ALREADY IN
//! THE TREE. Ruling 9's pre-trim rule assumes the pre-trim blob is the right
//! comparison for every gate. It is right for citations and for comment coverage.
//! It is NOT right for `audit-case-migration.py` or `check_fixtures.py` when the
//! RETAINED tests carry literal claims of their own, as this file's do -- `E5506`,
//! `Math.sqrt`, `Math.atan2`, `unsupported math`, and the JSON key `code`, which no
//! migrated case may claim. Against the post-trim file the case file's claims are
//! compared with a source stripped of the half that makes them; against the
//! pre-trim blob the retained half's claims are compared with a case file that
//! carries only the migrated half's. Both red, for opposite reasons.
//!
//! AN EARLIER VERSION OF THIS PARAGRAPH CALLED THAT NEW, AND IT IS NOT. It said
//! batch 5's trims were green on both sides "only because their retained tests'
//! needles were loop variables". Measured, one command per file, each against the
//! ref in its own header: `browser_math_max_min_frozen_aliases.rs` is red pre-trim
//! with 8 missing claims, `browser_math_abs_sign_frozen_aliases.rs` with 4,
//! `browser_math_atan2_global_this_root.rs` with 4, and
//! `browser_math_pow_exponent_one.rs` with 14 -- and all four go green against the
//! complement described below. Those four headers currently describe their audit
//! red as the escalation itself rather than as an artifact of the trim; correcting
//! them is scoped to BATCH 7, following the precedent of batch 5's retroactive
//! ruling-9 sweep, and is not done here. The four retentions themselves stand
//! unchanged: every one is adjudicated on the FIXTURE SELF-INSPECTION ground and
//! every one is in `find_fixture_self_inspection.py`'s `KNOWN` list. The audit red
//! was never their escalation ground, and it is not this file's either -- the
//! quantifier above is.
//!
//! The right left-hand side for those two gates is the DIFFERENCE of the two blobs
//! -- the migrated half -- reconstructed mechanically by
//! `tools/task-18-browser-pilot/migrated_complement.py`:
//!
//!   python3 tools/task-18-browser-pilot/migrated_complement.py \
//!       /tmp/pretrim.rs \
//!       crates/kali_cli/tests/browser_math_unsupported_member_calls_harness_jsx_tsx.rs \
//!       > /tmp/migrated_part.rs
//!
//! Read the columns as POST-trim / PRE-trim / MIGRATED-PART. The migrated part is a
//! GATE INPUT, not a compilable file: `kali_bin` is used by both halves, so it stays
//! here and is absent from the complement by construction.
//!
//!   audit-case-migration.py      RED / RED / green, and the two reds have DIFFERENT
//!        causes, which is why both are named. PRE-trim it fails forward with
//!        exactly five claims absent from the case file -- `E5506`, `Math.atan2`,
//!        `Math.sqrt`, `unsupported math` and the JSON key `code` -- every one of
//!        them made by a retained test and by nothing else. POST-trim it fails in
//!        the REVERSE direction instead: the case file's `stdout_count`/`json_count`
//!        claims no longer correspond to any `.matches(...).count()` in a source
//!        that no longer contains the helper making them. Green against the migrated
//!        half, which is the run that actually audits this migration.
//!   check_fixtures.py            RED / RED / green. Same cause as the pre-trim
//!        audit red: `browser_harness_run_atan2_source` is retained-half program
//!        text and appears in no case file, correctly. (Its `test` sibling passes
//!        the gate's `format!`-segment arm, so the report names one fixture, not
//!        two.)
//!   comment_coverage.py          RED / green / green. Post-trim, every non-blank
//!        line of this header comes back missing: the checker requires each source
//!        comment line to appear in some case's rationale, and this header is prose
//!        about the RETAINED tests, which by construction have no case. NO COUNT IS
//!        GIVEN, deliberately -- any figure would count this header's own length and
//!        would be invalidated by every edit to it, including the edit that
//!        corrected it. On both other sides the only comment block is the `///` doc
//!        on the migrated helper, which every migrated case carries.
//!   check_rationale_fn_names.py  RED / green / RED, and the two reds are mirror
//!        images. Post-trim, six cited names left with the migration
//!        (`assert_browser_harness_math_sqrt_success`, the two non-atan2 fixture
//!        builders, and the three migrated `#[test]` fns). Against the migrated half
//!        the other two go unresolved instead
//!        (`assert_browser_harness_unsupported_math_rejection` and `kali_bin`),
//!        because the case file legitimately names the helper it did NOT migrate
//!        when it explains the split. Only the PRE-trim run has both halves present,
//!        and it is green.
//!   check_extra_claims.py        RED / green / green. Post-trim the case file's
//!        claims are compared against a source stripped of the half that makes them.
//!   batch5_crosscheck.py         green / green / n-a, with the case file's citations
//!        resolved against the pre-trim ref (`verify_pair.sh --pretrim`) and THIS
//!        header's own citations always resolved against the shipped file.
//!   cargo test -p kali_cli --test cases -- browser/math_unsupported_member_calls
//!        green (20 trials), and `cargo test -p kali_cli --test
//!        browser_math_unsupported_member_calls_harness_jsx_tsx` is green too: the
//!        three retained tests still compile and pass as their own target.
//!
//! Escalated per rule 3/4 rather than shipped with a false green or a fabricated
//! claim. This file must NOT be deleted by the family-wide sweep after batch 8.
//! See `.superpowers/sdd/2026-07-29-test-binary-consolidation/
//! task-18-batch6a-report.md` for the full account.
use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

fn assert_browser_harness_unsupported_math_rejection(
    command: &str,
    filename: &str,
    source: &str,
    bundle: bool,
    json_output: bool,
) {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join(filename);
    fs::write(&source_path, source).expect("write source");

    let mut cli = Command::new(kali_bin());
    cli.current_dir(dir.path())
        .env(kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV, "node");
    if json_output {
        cli.arg("--output").arg("json");
    }
    cli.arg(command);
    if bundle {
        cli.arg("--bundle");
    }
    cli.arg("--api").arg("browser").arg(&source_path);

    let output = cli.output().expect("run kali");
    assert!(
        !output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(1));

    if json_output {
        let json: Value = serde_json::from_slice(&output.stdout).expect("valid json stdout");
        assert_eq!(json["schemaVersion"], 1);
        assert_eq!(json["command"], command);
        assert_eq!(json["success"], false);
        let errors = json["errors"].as_array().expect("errors array");
        assert!(!errors.is_empty(), "errors array should not be empty");
        assert!(
            errors.iter().all(|error| error["code"] == "E5506"),
            "unexpected errors: {errors:?}"
        );
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("E5506"), "stderr: {stderr}");
        assert!(
            stderr.contains("Math.sqrt")
                || stderr.contains("Math.atan2")
                || stderr.contains("unsupported math"),
            "stderr: {stderr}"
        );
    }
}

fn browser_harness_run_atan2_source() -> &'static str {
    "console.log(Math.atan2(1, 1));\nconsole.log(Math[\"atan2\"](1, 1));\nconsole.log(globalThis.Math[\"atan2\"](1, 1));\nconsole.log(globalThis[\"Math\"].atan2(1, 1));\nconsole.log(globalThis[\"Math\"][\"atan2\"](1, 1));\nconsole.log(globalThis['Math']['atan2'](1, 1));\n"
}

fn browser_harness_test_atan2_source() -> &'static str {
    r#"Kali.test('unsupported math member', () => {
  console.log(Math.atan2(1, 1));
  console.log(Math["atan2"](1, 1));
  console.log(globalThis.Math["atan2"](1, 1));
  console.log(globalThis["Math"].atan2(1, 1));
  console.log(globalThis["Math"]["atan2"](1, 1));
  console.log(globalThis['Math']['atan2'](1, 1));
});
"#
}
#[test]
fn run_rejects_broader_math_atan2_member_calls_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_browser_harness_unsupported_math_rejection(
            "run",
            &format!("main.{extension}"),
            browser_harness_run_atan2_source(),
            false,
            false,
        );
        assert_browser_harness_unsupported_math_rejection(
            "run",
            &format!("main.{extension}"),
            browser_harness_run_atan2_source(),
            false,
            true,
        );
    }
}

#[test]
fn test_rejects_broader_math_atan2_member_calls_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input(
) {
    for extension in ["js", "ts", "jsx", "tsx"] {
        assert_browser_harness_unsupported_math_rejection(
            "test",
            &format!("smoke.test.{extension}"),
            browser_harness_test_atan2_source(),
            false,
            false,
        );
        assert_browser_harness_unsupported_math_rejection(
            "test",
            &format!("smoke.test.{extension}"),
            browser_harness_test_atan2_source(),
            false,
            true,
        );
    }
}

#[test]
fn build_rejects_broader_math_atan2_member_calls_in_browser_api_surface_with_harness_jsx_and_tsx_input(
) {
    for extension in ["jsx", "tsx"] {
        assert_browser_harness_unsupported_math_rejection(
            "build",
            &format!("main.{extension}"),
            browser_harness_run_atan2_source(),
            true,
            false,
        );
        assert_browser_harness_unsupported_math_rejection(
            "build",
            &format!("main.{extension}"),
            browser_harness_run_atan2_source(),
            true,
            true,
        );
    }
}
