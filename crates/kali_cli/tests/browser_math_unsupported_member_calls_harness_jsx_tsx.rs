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
//! (`:267`),
//! `test_rejects_broader_math_atan2_member_calls_in_browser_api_surface_with_harness_js_ts_jsx_and_tsx_input`
//! (`:288`) and
//! `build_rejects_broader_math_atan2_member_calls_in_browser_api_surface_with_harness_jsx_and_tsx_input`
//! (`:309`) routes through
//! `assert_browser_harness_unsupported_math_rejection` (`:196`), and each
//! calls it twice per extension inside its own `for extension in [...]` loop, once
//! with the JSON-output flag false and once true. The true call is unconditional,
//! so 3 of 3 retained tests reach the blocking construct on every iteration.
//!
//! The blocking construct is a QUANTIFIER over the JSON `errors` array,
//! `errors.iter().all(...)` (`:236`). The case-file format offers only closed
//! dotted-path indexing into JSON -- design spec 5.4 is explicit that there are
//! "no slices, no wildcards, no negative-from-end indexing, no filters" -- so a
//! dotted path can pin the FIRST array element and nothing more. Narrowing "every
//! error has this code" to "error 0 has this code" is a weakening (a second,
//! differently-coded diagnostic would satisfy the migration and fail the source),
//! and rule 1 forbids weakening. Nothing in the twelve assertion keys expresses it.
//! The neighbouring `assert!(!errors.is_empty(), ...)` (`:234`) is not the
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
//! `stderr.contains("E5506")` (`:241`) plus a three-way OR that rule 11
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
//! THIS RETENTION NEEDS A THIRD LEFT-HAND SIDE, AND THE CONDITION FOR THAT IS NOT
//! "being a trim". Ruling 9's pre-trim rule assumes the pre-trim blob is the right
//! comparison for every gate. It is right for citations and for comment coverage.
//! It is NOT right for `audit-case-migration.py` or `check_fixtures.py` WHEN THE
//! RETAINED TESTS CARRY LITERAL CLAIMS OF THEIR OWN -- as this file's do: `E5506`,
//! `Math.sqrt`, `Math.atan2`, `unsupported math`, and the JSON key `code`, none of
//! which any migrated case may claim.
//!
//! The two sides fail differently, and the post-trim one fails BOTH WAYS AT ONCE
//! (measured; batch 7 corrected an earlier "in the REVERSE direction instead" in
//! the red-list below, which said the post-trim run replaced one direction with
//! the other). Post-trim, the retained half's literal claims are absent from the
//! case file AND the case file's count claims no longer correspond to a source
//! that no longer holds the helper making them -- forward and reverse in one run.
//! Pre-trim it is red in the forward direction only, because the blob holds both
//! halves while the case file carries one.
//!
//! NO INTEGER IS GIVEN FOR THE FORWARD DIRECTION, and that is a ruling-11
//! correction batch 7 applied after this header shipped with one. The forward
//! figure is HEADER-MOVABLE: `audit-case-migration.py`'s `.contains` arm reads
//! `//!` prose, so a single extra `//!` line carrying a quoted construct adds a
//! source claim no case file can satisfy and the figure goes up. Measured, not
//! supposed -- inserting one such line moves the forward count by exactly one
//! while leaving the reverse count untouched. The reverse figure is NOT movable
//! that way (its input is the case file's count claims), so it is stated below.
//!
//! TWO EARLIER VERSIONS OF THIS PARAGRAPH GOT THE SCOPE WRONG, IN OPPOSITE
//! DIRECTIONS. The first said batch 5's trims were green on both sides "only
//! because their retained tests' needles were loop variables" and that this file
//! was the first where they are not -- too narrow. Its replacement said "and so
//! does every trim already in the tree" -- too broad. Measured over every stem in
//! the family carrying a `PRE-TRIM REF:` and a case file, TEN of them, each against
//! the ref in its OWN header: FIVE are green on both pre-trim gates and need no
//! third side at all (`browser_array_iteration_spread`,
//! `browser_math_floor_trunc_ceil_aliases`, `browser_math_floor_trunc_ceil_bundle`,
//! `browser_math_pow_bracketed_frozen_wrapper_harness`,
//! `browser_math_pow_bracketed_frozen_wrapper`), and FIVE need it
//! (`browser_math_abs_sign_frozen_aliases` and
//! `browser_math_atan2_global_this_root` and `browser_math_max_min_frozen_aliases`,
//! red on the audit alone; `browser_math_pow_exponent_one` and this file, red on
//! the audit and on `check_fixtures.py`). All five go green against the complement
//! below. The discriminator is the condition in the first paragraph, not the fact
//! of being a trim.
//!
//! SCOPE FOR BATCH 7, STATED EXACTLY, because a wrong scope in a handoff sentence
//! is how both earlier versions went wrong. The retroactive header sweep is FOUR
//! files -- `browser_math_max_min_frozen_aliases.rs`,
//! `browser_math_abs_sign_frozen_aliases.rs`,
//! `browser_math_atan2_global_this_root.rs` and
//! `browser_math_pow_exponent_one.rs` -- because those four, and only those, carry
//! a sentence calling their audit red "the escalation itself, not a trim artifact".
//! The five green trims say no such thing and need no edit. The four retentions
//! themselves stand unchanged: every one is adjudicated on the FIXTURE
//! SELF-INSPECTION ground and every one is in
//! `find_fixture_self_inspection.py`'s `KNOWN` list. The audit red was never their
//! escalation ground, and it is not this file's either -- the quantifier above is.
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
//!        causes, which is why both are named. PRE-trim it fails FORWARD ONLY: the
//!        absent claims are the retained tests' own literals -- the E5506 code, the
//!        two unsupported member-call spellings, the diagnostic phrase and the JSON
//!        error-code key -- every one made by a retained test and by nothing else.
//!        NO INTEGER, per the ruling-11 note above: that figure is header-movable.
//!        POST-trim it fails in BOTH DIRECTIONS AT ONCE, not in the reverse
//!        direction INSTEAD (corrected by batch 7; the old word claimed the
//!        post-trim run swapped one direction for the other, and measured it adds
//!        the reverse to the forward). The reverse half is 24 gate failure
//!        ENTRIES, and 24 is not the number of count claims in the case file --
//!        that is 16, which the gate prints on its own line. The 24 entries fall
//!        in two classes, which the earlier wording collapsed into one:
//!          * 16 of `stdout_count`/`json_count` needles that correspond to no
//!            `.matches(...).count()` in a source that no longer contains the
//!            helper making them -- one per count claim;
//!          * 8 of `path segment 'stdout' is not a JSON key the source ever
//!            indexed`, raised for the `json_count` claims only, so the eight
//!            JSON-mode cases are reported twice, once in each class.
//!        Green against the migrated half, which is the run that actually audits
//!        this migration.
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
//! See the batch's own working report -- which was git-ignored scratch and
//! does not ship, so it is deliberately not cited by path.
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
