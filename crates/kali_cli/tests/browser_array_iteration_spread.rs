//! Deliberately kept hand-written (not migrated to a `.toml` case file).
//!
//! This file originally had 21 `#[test]` fns. 20 of them invoke the real
//! `kali` binary (via `assert_browser_bundle_array_iteration_spread`/
//! `assert_browser_bundle_object_enumeration_spread`) and have been migrated
//! to `crates/kali_cli/tests/cases/browser/array_iteration_spread.toml`.
//!
//! The one remaining here,
//! `browser_bundle_test_reuses_the_shared_array_from_inventory_in_both_loop_sections`,
//! never constructs a `Command` and never invokes `kali` at all: it calls
//! `browser_bundle_array_from_source()` directly and asserts on the
//! *returned string's own line content*
//! (`.lines().filter(|line| line.contains(&format!(...))).count()`). This is
//! a pure Rust unit test of a fixture-construction helper, not a
//! CLI-behavior test -- there is no process output and no JSON file on disk
//! for a case-file `Step` to assert on (`cli` and `browser_bundle_harness`
//! steps assert on a spawned process's output; `file_json` steps assert on a
//! file the binary wrote; none of the three fits "assert on a Rust string
//! value computed in-process"). `audit-case-migration.py` cannot see this
//! shape either way: it deliberately excludes `[source]` fixture text from
//! its search (a claim that exists only inside a fixture body is correctly
//! reported missing, not a bug), and this test's `.contains(&format!(...))`
//! call's argument is not a bare string literal in the first place, so it
//! does not even match the audit script's `CONTAINS` regex.
//!
//! See
//! `/workspace/.superpowers/sdd/2026-07-29-test-binary-consolidation/task-18-pilot-report.md`
//! ("Finding 1" / "File 6", the `browser_math_pow_exponent_one.rs` §5.11
//! trim-and-keep precedent) for the general shape this disposition follows.
//!
//! CONSEQUENCE FOR THE GATES -- THE COMPLETE RED-LIST (ruling 9). Added
//! retroactively by Task 18 batch 5: ruling 9 postdates the commit that trimmed
//! this file, so this pair shipped without one. Every line below was produced by
//! RUNNING the gate against both sides, not by reasoning about it, and it is NOT
//! a copy of batch 4's list -- this pair behaves differently.
//!
//!   PRE-TRIM REF:  f0bfb76d79^   (= 3e083edc5d)
//!   git show f0bfb76d79^:crates/kali_cli/tests/browser_array_iteration_spread.rs > /tmp/pretrim.rs
//!
//! Read the two columns as POST-trim (the plain `verify_pair.sh
//! array_iteration_spread` run, against this file) then PRE-trim.
//!
//!   audit-case-migration.py      GREEN / GREEN. The migration gate proper is
//!        clean on both sides; the one test retained here makes no claim the
//!        script's literal extractor can see, as the paragraph above explains.
//!   comment_coverage.py          RED / RED, but for two different reasons and
//!        only one of them is the trim. Post-trim, every non-blank line of this
//!        header comes back missing, because the header is prose about the
//!        RETAINED test, which has no case. NO COUNT IS GIVEN, deliberately --
//!        any figure would count this header's own length and would be
//!        invalidated by every edit to it. PRE-trim it is red for an unrelated,
//!        pre-existing reason: the source's two-line PR#16 honest-re-pin comment
//!        is carried into 2 of the 8 cases and reported missing from the other
//!        6. That is `comment_coverage.py`'s known absence of per-helper
//!        attribution (U6): the comment belongs to exactly the cases its
//!        producing helper reaches, and copying it into all 8 to green the
//!        checker would be the over-attribution U6 forbids.
//!   check_rationale_fn_names.py  RED / RED -- 35 unexplained post-trim, 10
//!        pre-trim. The pre-trim 10 are `[source]` key stems, `kali_common`
//!        helper names and two JS keywords quoted in backticks; the checker
//!        resolves names only against the `.rs` it is handed, so none of those
//!        can resolve. The post-trim excess is the migrated helpers and fn names
//!        that left with the trim.
//!   check_fixtures.py            GREEN / GREEN.
//!   batch5_crosscheck.py         GREEN / GREEN -- the citation gate, wired into
//!        `verify_pair.sh` by batch 6; this row is part of that same wiring
//!        change, as ruling 9 requires. The post-trim green is INCIDENTAL and
//!        must not be read as a property of retention pairs: it means only that
//!        this file's case-file citations happen to still resolve against the
//!        trimmed remainder. Run it with the pre-trim ref regardless -- that is
//!        the run this migration is gated on, and on the sibling batch-4 pairs
//!        the same gate is red post-trim. NO COUNT IS GIVEN: this gate also
//!        resolves THIS header's own `:N` citations, so every edit to this
//!        paragraph is an input to the figure it would report (ruling 11).
//!   check_extra_claims.py        RED / RED, both sides. This gate and the U8 checker were both shipped in ef0b2cf3f5,
//!        AFTER this pair; the `# EXTRA-OK:` declaration mechanism it reads did
//!        not exist when the case file was written, so its U5-renamed source
//!        keys and its grandfathered exact output pins are undeclared by
//!        construction. NOTE, and it is not a nicety: `check_extra_claims.py`
//!        accepts any claim string that appears verbatim anywhere in the `.rs`,
//!        INCLUDING in a comment, so spelling a claim-shaped literal in this
//!        header would silently green the gate for that claim. None is spelled
//!        here for that reason. (Measured on the sibling retention
//!        `browser_math_abs_sign_frozen_aliases.rs`, where a draft of the same
//!        paragraph did exactly that.)
//!        NO COUNT IS GIVEN, and that is a ruling-11 correction applied after
//!        this paragraph first shipped with one. `check_extra_claims.py`
//!        counts a claim as justified if the string occurs verbatim ANYWHERE
//!        in the `.rs`, comments included, so this header is part of the
//!        gate's own input and its prose moves the figure. Measured, not
//!        supposed: running the gate against this file with and without the
//!        header block gives two different numbers. Ruling 11 forbids a figure
//!        that an edit to the surrounding prose can move, so the durable fact
//!        is the classification. Run the gate for today's number.
//!
//! SO: this pair does NOT go all-green against the pre-trim ref. The audit, the
//! fixture check and the citation gate do; the other three are red on both
//! sides, for three
//! unrelated reasons -- a U6 attribution limit, and two gates that postdate the
//! file. Recording that is the point; a red-list that claimed otherwise would be
//! worse than none.
//!
//! Adding a new gate to `verify_pair.sh` includes updating this paragraph, in
//! the same change (ruling 9). This file must NOT be deleted by the family-wide
//! sweep after batch 8.

use kali_common::{array_from_alias_inventory_source, array_from_loop_lines};

fn browser_bundle_array_from_source() -> String {
    let array_from_source = array_from_alias_inventory_source();
    let frozen_for_of = array_from_loop_lines(&array_from_source, "for (const value of ", "  ");
    let frozen_for_await =
        array_from_loop_lines(&array_from_source, "for await (const value of ", "  ");
    [
        r##"// kali-tree-shake: browserArrayFromWrappers
export async function browserArrayFromWrappers() {
  const values = [1, 2];
"##,
        &frozen_for_of,
        r##"
"##,
        &frozen_for_await,
        r##"}
"##,
    ]
    .join("")
}

#[test]
fn browser_bundle_test_reuses_the_shared_array_from_inventory_in_both_loop_sections() {
    let source = browser_bundle_array_from_source();
    let alias_inventory = array_from_alias_inventory_source();

    for alias in alias_inventory.trim_end_matches(';').split("; ") {
        assert_eq!(
            source
                .lines()
                .filter(|line| line.contains(&format!("for (const value of {alias}(values))")))
                .count(),
            1,
            "browser bundle Array.from source should embed {alias} in the for-of loop section"
        );
        assert_eq!(
            source
                .lines()
                .filter(|line| line.contains(&format!("for await (const value of {alias}(values))")))
                .count(),
            1,
            "browser bundle Array.from source should embed {alias} in the for-await loop section"
        );
    }
}
