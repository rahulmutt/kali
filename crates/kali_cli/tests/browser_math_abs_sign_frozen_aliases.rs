//! Task 18 batch 3 audit escalation, TRIMMED: this file now holds exactly the
//! one `#[test]` its fixture-introspecting body blocks from migrating, plus the
//! single fixture builder that test reads.
//!
//! It originally had 25 `#[test]` fns. The other 24 -- the 8
//! `build_emits_*`/`json_build_emits_*` fns that called
//! `assert_browser_bundle_global_this_math_abs_sign_frozen`, and the 16
//! `run_supports_*`/`test_supports_*`/`json_*` fns that called
//! `assert_browser_harness_global_this_math_abs_sign_frozen` -- are migrated to
//! `tests/cases/browser/math_abs_sign_frozen_aliases.toml` (6 `[[case]]`
//! entries, `ext = [js, ts, jsx, tsx]` matrix-fanned to 24 trials, audited
//! against the pre-trim source and green). Both of those helpers, their two
//! harness fixture builders, `kali_bin()` and the `fs`/`Command`/`Value`/
//! `tempdir` imports went with them; nothing left here is unused.
//!
//! WHAT BLOCKS THE ONE RETAINED TEST.
//! `browser_bundle_global_this_math_abs_sign_frozen_source_includes_direct_frozen_math_aliases`
//! (`:196-196`) has no helper: its whole body is four
//! `assert!(source.contains(<needle>))` self-checks (`:198-210`)
//! run against `browser_bundle_global_this_math_abs_sign_frozen_source()`'s OWN
//! TEXT (`:166-193`), before any command is built and without ever
//! invoking `kali`. The four blocking literals are
//! `Object.freeze(globalThis.Math.abs)`, `Object.freeze(globalThis.Math.sign)`,
//! `Object.freeze(Math.abs)` and `Object.freeze(Math.sign)`.
//!
//! `scripts/audit-case-migration.py` extracts every `.contains(<literal>)`
//! argument as a claim and searches only the fields the case runner turns into
//! assertions; `[source]` is excluded from that search by construction (its
//! module docstring: "`body` and everything under `[source]` are program text,
//! not claims about behavior"). These four literals are *read*, not *asserted
//! on output*, so no honest migration can put them in an assertion field --
//! doing so would invent a claim the source never made -- and the audit reports
//! them absent no matter what the migrated `[source]` contains. Verified, not
//! assumed: auditing the PRE-TRIM source against the shipped case file reports
//! `AUDIT FAILED -- 4 claim(s) absent`, listing exactly those four literals and
//! nothing else.
//!
//! Same shape as the Task 18 pilot's `browser_math_pow_exponent_one.rs` and
//! batch 2's `browser_array_from_set_map_bundle.rs`. The controller has ruled
//! that `audit-case-migration.py` is NOT extended for it (ruling 4), so this is
//! escalated per rule 3/4 and the affected test is retained hand-written. U4's
//! trim-and-keep applied: this is a partial retention, not a whole-file one,
//! and the trim is done -- this file is now exactly its retained remainder.
//!
//! CONSEQUENCE FOR THE GATES -- THE COMPLETE RED-LIST (ruling 9). Added
//! retroactively by Task 18 batch 5: ruling 9 postdates the commit that trimmed
//! this file, so this pair shipped without one. Every line below was produced by
//! RUNNING the gate against both sides, not by reasoning about it, and it is NOT
//! a copy of batch 4's list -- this pair behaves differently.
//!
//!   PRE-TRIM REF:  1db95b469f^   (= 50061950a4)
//!   git show 1db95b469f^:crates/kali_cli/tests/browser_math_abs_sign_frozen_aliases.rs > /tmp/pretrim.rs
//!
//! THIRD COLUMN ADDED BY BATCH 7 UNDER RULING 12. Ruling 9's pre-trim rule
//! assumes the pre-trim blob is the right comparison for every gate. It is right
//! for citations and for comment coverage. It is NOT right for
//! `audit-case-migration.py` or `check_fixtures.py` when the RETAINED tests
//! carry literal claims of their own -- as this file's retained test does. Those
//! literals are in the pre-trim blob too, so the audit is red against BOTH older
//! sides and the red looks permanent. The correct left-hand side is the part
//! that was actually migrated: the complement of the retained half, built
//! mechanically by
//!
//!   python3 tools/task-18-browser-pilot/migrated_complement.py \
//!       /tmp/pretrim.rs \
//!       crates/kali_cli/tests/browser_math_abs_sign_frozen_aliases.rs \
//!       > /tmp/migrated_part.rs
//!
//! Read the three columns as POST-trim (the plain `verify_pair.sh
//! math_abs_sign_frozen_aliases` run, against this file) / PRE-trim /
//! MIGRATED-PART. The migrated part is a GATE INPUT, not a compilable file.
//! Every cell below was produced by RUNNING the gate on that side.
//!
//!   audit-case-migration.py      RED / RED / GREEN. The two reds are
//!        BYTE-IDENTICAL -- the same 4 claims absent, the same 4
//!        fixture-self-inspection literals named in the paragraph above.
//!        CORRECTED BY BATCH 7 (ruling 12). This paragraph used to say that red
//!        "is the escalation itself, not a trim artifact". That is FALSE, and
//!        the third column is the proof: against the migrated complement -- the
//!        half the case file was actually produced from -- the audit exits 0
//!        with nothing absent. The red was always a CONSEQUENCE of the trim,
//!        appearing on both older sides because both contain the retained half
//!        whose literals no case may carry. THE RETENTION ITSELF IS UNAFFECTED
//!        AND STANDS: its ground is FIXTURE SELF-INSPECTION -- the literals are
//!        read by the retained test, never asserted on output, so no case file
//!        can carry them at any strength -- and this file is in
//!        `find_fixture_self_inspection.py`'s `KNOWN` list on that ground. What
//!        changed is the DESCRIPTION of a gate result, not the adjudication.
//!   comment_coverage.py          RED / RED / RED. Post-trim, every non-blank line of
//!        this header comes back missing: the header is prose about the RETAINED
//!        test, which by construction has no case. NO COUNT IS GIVEN,
//!        deliberately -- any figure would count this header's own length and
//!        would be invalidated by every edit to it, including the edit that
//!        corrected it. PRE-trim it is red for the same reason, because the
//!        migration commit added this header before the trim commit removed the
//!        migrated tests; the pre-trim ref therefore already carries a `//!`
//!        block no rationale has any reason to reproduce. Against the migrated
//!        part it is red for the same reason as pre-trim: the complement carries
//!        the `//!` block, which no rationale reproduces.
//!   check_rationale_fn_names.py  RED / GREEN / RED, and the two reds are mirror
//!        images. Post-trim the two `assert_browser_*` helpers that left with the
//!        migrated cases go unresolved; against the migrated part the RETAINED
//!        half's names go unresolved instead, because the case file legitimately
//!        names the helper it did NOT migrate when it explains the split. Only
//!        the PRE-trim run has both halves present, and it is the green one.
//!   check_fixtures.py            GREEN / GREEN / GREEN. The retained half
//!        carries no program text of its own that a case file would have to
//!        reproduce, which is why this row is unaffected by the third column.
//!   batch5_crosscheck.py         GREEN / GREEN / n-a -- the citation gate, wired into
//!        `verify_pair.sh` by batch 6; this row is part of that same wiring
//!        change, as ruling 9 requires. The post-trim green is INCIDENTAL and
//!        must not be read as a property of retention pairs: it means only that
//!        this file's case-file citations happen to still resolve against the
//!        trimmed remainder. Run it with the pre-trim ref regardless -- that is
//!        the run this migration is gated on, and on the sibling batch-4 pairs
//!        the same gate is red post-trim. NO COUNT IS GIVEN: this gate also
//!        resolves THIS header's own `:N` citations, so every edit to this
//!        paragraph is an input to the figure it would report (ruling 11).
//!   check_extra_claims.py        RED / RED / RED, all three sides. The pre-trim side reduces
//!        to a single entry, which is a live-captured exact pin on a
//!        JSON string leaf, which is exactly what the `# EXTRA-OK:` mechanism
//!        exists to declare -- but that mechanism, and this gate, shipped in
//!        ef0b2cf3f5, AFTER this pair, so the declaration is absent by
//!        construction rather than by oversight.
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
//!        THE HAZARD THIS RECORDS, now that the figure itself is gone: an
//!        earlier draft of this paragraph spelled a JSON leaf's dotted path,
//!        which supplied the missing justification for one claim and moved
//!        the gate's result. The figure was corrected, then removed under
//!        ruling 11 -- correcting a number that prose can move only postpones
//!        it going stale. No claim-shaped literal is spelled here.
//!
//! SO: this pair does NOT go all-green against any single side, and batch 7's
//! third column changes which red is which rather than removing them. Against
//! the PRE-TRIM ref only the U8 checker flips. Against the MIGRATED COMPLEMENT
//! the AUDIT flips -- and that is the correction ruling 12 requires here,
//! because the audit red was a trim artifact and not, as this block used to say,
//! the escalation. The comment-coverage and extra-claims reds stay on all three
//! sides, for two unrelated reasons -- a retention header no case can carry, and
//! a gate that postdates the file. The fixture check and the citation gate are
//! green. Recording that is the point; a red-list that claimed otherwise would
//! be worse than none.
//!
//! The RETENTION is not in question here and no part of this correction reopens
//! it. Ruling 12 changes which blob a gate is measured against; it does not
//! change why the retained test cannot be migrated, which is the fixture
//! self-inspection ground recorded above.
//!
//! Adding a new gate to `verify_pair.sh` includes updating this paragraph, in
//! the same change (ruling 9).
//!
//! This file must NOT be deleted by the family-wide sweep after batch 8. See
//! `.superpowers/sdd/2026-07-29-test-binary-consolidation/
//! task-18-batch3-report.md` for the full account.
use std::sync::OnceLock;

fn browser_bundle_global_this_math_abs_sign_frozen_source() -> &'static str {
    static SOURCE: OnceLock<String> = OnceLock::new();
    SOURCE
        .get_or_init(|| {
            r##"// kali-tree-shake: globalThisMathAbsSignFrozenAliases
function globalThisMathAbsSignFrozenAliases() {
  const value = -3;
  const alias = value;
  console.log(globalThis.Math.abs(value));
  console.log(globalThis.Math.sign(value));
  console.log(Object.freeze(globalThis.Math.abs)(alias));
  console.log(Object.freeze(globalThis.Math.sign)(alias));
  console.log(Object.freeze(Math.abs)(alias));
  console.log(Object.freeze(Math.sign)(alias));
  return [
    globalThis.Math.abs(value),
    globalThis.Math.sign(value),
    Object.freeze(globalThis.Math.abs)(alias),
    Object.freeze(globalThis.Math.sign)(alias),
    Object.freeze(Math.abs)(alias),
    Object.freeze(Math.sign)(alias),
  ];
}
"##
            .to_string()
        })
        .as_str()
}

#[test]
fn browser_bundle_global_this_math_abs_sign_frozen_source_includes_direct_frozen_math_aliases() {
    let source = browser_bundle_global_this_math_abs_sign_frozen_source();
    assert!(
        source.contains("Object.freeze(globalThis.Math.abs)"),
        "source: {source}"
    );
    assert!(
        source.contains("Object.freeze(globalThis.Math.sign)"),
        "source: {source}"
    );
    assert!(
        source.contains("Object.freeze(Math.abs)"),
        "source: {source}"
    );
    assert!(
        source.contains("Object.freeze(Math.sign)"),
        "source: {source}"
    );
}
