//! Task 18 batch 4 audit escalation, TRIMMED: this file now holds exactly the
//! one `#[test]` its fixture-introspecting body blocks from migrating, plus the
//! two fixture builders that test reads.
//!
//! It originally had 17 `#[test]` fns. The other 16 -- every `run_supports_*`,
//! `test_supports_*`, `json_run_supports_*` and `json_test_supports_*` fn, one
//! real invocation each of `assert_browser_harness_math_floor_trunc_ceil` --
//! ARE migrated, to `tests/cases/browser/math_floor_trunc_ceil_aliases.toml`
//! (4 `[[case]]` x a file-wide `ext` `[matrix]` of js/ts/jsx/tsx = 16 trials,
//! audited against the pre-trim source and green). That helper,
//! `browser_harness_math_floor_trunc_ceil_test_source()`, `kali_bin()` and the
//! `fs`/`Command`/`Value`/`tempdir` imports went with them; nothing left here
//! is unused.
//!
//! WHAT BLOCKS THE ONE RETAINED TEST.
//! `browser_harness_math_floor_trunc_ceil_source_includes_full_frozen_callable_inventory`
//! (`:134-134`) has no helper: its whole body is a single
//! `assert!(source.contains(expected))` self-check (`:138-141`) run in a `for`
//! loop over `kali_common::math_floor_trunc_ceil_frozen_callable_aliases()`,
//! against `browser_harness_math_floor_trunc_ceil_run_source()`'s OWN TEXT
//! (`:121-131`), before any command is built and without ever invoking `kali`.
//!
//! It is doubly unmigratable, and the second reason is the sharper one:
//!   1. `scripts/audit-case-migration.py` extracts every `.contains(<literal>)`
//!      argument as a claim and searches only the fields the case runner turns
//!      into assertions; `[source]` is excluded from that search by
//!      construction. A fixture-text read is indistinguishable to it from an
//!      output assertion, so migrating this test would produce a false green.
//!   2. The claim is about `[source]` TEXT, and the format has no step kind
//!      that asserts on it. Every assertion key in design spec 5.4 is about a
//!      process's stdout/stderr/JSON output; this test runs no process at all.
//!      So the claim is not expressible at any strength, which is rule 4's
//!      condition exactly.
//!
//!      CORRECTED, fix round 1 (I4): this paragraph previously said "there is
//!      no literal to migrate at all... a RUNTIME-COMPUTED inventory". That was
//!      FALSE and is worth stating plainly, because a later reader could have
//!      acted on it. `kali_common::math_floor_trunc_ceil_frozen_callable_
//!      aliases()` (crates/kali_common/src/math.rs:87) is a `pub const fn`
//!      returning a compile-time `&[&str]` of 81 raw-string literals, and all
//!      81 are already present verbatim in the migrated case file's `[source]`
//!      bodies (verified by counting, both case files, 81/81). The inventory
//!      is perfectly enumerable. What blocks migration is reason 1 plus the
//!      absence of a `[source]`-text assertion -- not any inability to obtain
//!      the needles.
//!
//! Same shape as the Task 18 pilot's `browser_math_pow_exponent_one.rs`, batch
//! 2's `browser_array_from_set_map_bundle.rs` and batch 3's
//! `browser_math_atan2_global_this_root.rs`; the controller has ruled the
//! script is NOT extended for it (ruling 4), so this is escalated per rule 3/4
//! and the affected test is retained hand-written. U4's trim-and-keep applied:
//! this is a partial retention (1 of 17), not a whole-file one, and the trim is
//! done -- this file is now exactly its retained remainder.
//!
//! CONSEQUENCE FOR THE GATES -- THE COMPLETE RED-LIST (ruling 9). Every gate
//! below was RUN against this post-trim file, not reasoned about. A trimmed
//! retention makes the post-trim `.rs` the WRONG left-hand side: the migrated
//! 16 cases were produced from the file as it stood BEFORE the trim, so any
//! gate that compares case file against source must be given the pre-trim ref.
//!
//!   PRE-TRIM REF:  b44fd6acf9^   (= c934f6ebdd)
//!   git show b44fd6acf9^:crates/kali_cli/tests/browser_math_floor_trunc_ceil_aliases.rs > /tmp/pretrim.rs
//!
//! Against the POST-trim file (i.e. the plain `verify_pair.sh math_floor_trunc_ceil_aliases` run):
//!   RED  comment_coverage.py          exit 1 -- EVERY non-blank line of this
//!        header comes back missing. The checker requires each source comment
//!        line to appear in some case's `rationale`; this header is prose
//!        about the RETAINED test, which by construction has no case, so none
//!        of it is carried anywhere and all of it reports as uncovered.
//!        NO COUNT IS GIVEN, deliberately. Any figure here would be a count of
//!        THIS header's own length, so every edit to this paragraph -- including
//!        the edit that corrects the figure -- silently invalidates it. That is
//!        exactly how it went stale: measured, then the explanatory prose was
//!        written, and the writing changed the measurement. Batch 4 caught this
//!        class three times. Run the gate if you want today's number; the
//!        durable fact is the classification, not the integer.
//!   RED  check_rationale_fn_names.py  exit 1, 2 unexplained -- cites helpers
//!        that left with the migrated cases and no longer exist here.
//!   RED  check_extra_claims.py        exit 1 -- the
//!        migrated cases' claims are absent from the trimmed remainder.
//!        NO COUNT IS GIVEN, a ruling-11 correction applied after this
//!        paragraph shipped with one. `check_extra_claims.py` counts a claim as
//!        justified if the string occurs verbatim ANYWHERE in the `.rs`,
//!        comments included, so this header is part of the gate's own input and
//!        its prose moves the figure -- measured, not supposed. The durable
//!        fact is the classification; run the gate for today's number.
//!   GREEN audit-case-migration.py     exit 0 -- the retained test's claim has
//!        no extractable literal (its needle is a loop variable), so there is
//!        nothing for the audit to report missing. Do not read this green as
//!        the migration being audited; that is the pre-trim run below.
//!   GREEN check_fixtures.py           exit 0.
//!   RED  batch5_crosscheck.py         exit 1 -- the citation gate, wired into
//!        `verify_pair.sh` by batch 6. This row is part of that same wiring
//!        change, which is what ruling 9 requires and what batch 4 failed to do
//!        when it added `check_extra_claims.py`. Every `:N` in the case file is
//!        a PRE-TRIM line number -- this header says so above -- so resolving
//!        them against the trimmed remainder lands them in unrelated code. That
//!        is precisely the artifact the tool's `STEM=PRETRIM.rs` argument
//!        exists for. NO COUNT IS GIVEN: this gate also resolves THIS header's
//!        own `:N` citations, so every edit to this paragraph is an input to
//!        the figure it would report (ruling 11).
//!
//! Against the PRE-TRIM ref, ALL SIX exit 0. That is the run that gates this
//! migration; it is the one to reproduce.
//!
//! Adding a new gate to `verify_pair.sh` includes updating this paragraph, in
//! the same change (ruling 9). `check_extra_claims.py` and the U8 check were
//! shipped in ef0b2cf3f5 and were missing from this list until N1 of that
//! round's re-review caught it -- the fix commit edited this very block
//! without adding the gate it was itself introducing.
//!
//! This file must NOT be deleted by the family-wide sweep after batch 8. See
//! `.superpowers/sdd/2026-07-29-test-binary-consolidation/
//! task-18-batch4-report.md` for the full account.
use std::sync::OnceLock;

fn math_floor_trunc_ceil_frozen_callable_invocations() -> String {
    kali_common::math_floor_trunc_ceil_frozen_callable_invocation_lines("").replace('\n', " ")
}

fn browser_harness_math_floor_trunc_ceil_run_source() -> &'static str {
    static SOURCE: OnceLock<String> = OnceLock::new();
    SOURCE
        .get_or_init(|| {
            format!(
                "const value = 1.6; const alias = value; console.log(Math.floor(alias)); console.log(Math.trunc(alias)); console.log(Math.ceil(alias)); {}\n",
                math_floor_trunc_ceil_frozen_callable_invocations()
            )
        })
        .as_str()
}

#[test]
fn browser_harness_math_floor_trunc_ceil_source_includes_full_frozen_callable_inventory() {
    let source = browser_harness_math_floor_trunc_ceil_run_source();

    for expected in kali_common::math_floor_trunc_ceil_frozen_callable_aliases() {
        assert!(
            source.contains(expected),
            "missing {expected} in source: {source}"
        );
    }
}
