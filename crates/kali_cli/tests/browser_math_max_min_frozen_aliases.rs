//! Task 18 batch 5 audit escalation, TRIMMED: this file now holds exactly the
//! one `#[test]` its fixture-introspecting body blocks from migrating, plus the
//! single fixture builder that test reads.
//!
//! It originally had 13 `#[test]` fns. The other 12 -- the 4
//! `build_emits_*`/`json_build_emits_*` fns that called
//! `assert_browser_bundle_global_this_math_max_min_frozen`, and the 8
//! `run_supports_*`/`test_supports_*`/`json_*` fns that called
//! `assert_browser_harness_global_this_math_max_min_frozen` -- are migrated to
//! `tests/cases/browser/math_max_min_frozen_aliases.toml` (6 `[[case]]`
//! entries, an `ext` `[matrix]` of js/ts -- two values, because this source
//! never runs jsx or tsx -- fanned to 12 trials). Both of those helpers, the
//! two harness fixture builders, `kali_bin()` and the `fs`/`Command`/`Value`/
//! `tempdir` imports went with them; nothing left here is unused.
//!
//! WHAT BLOCKS THE ONE RETAINED TEST.
//! `browser_bundle_global_this_math_max_min_frozen_source_includes_direct_frozen_math_aliases`
//! (`:175`) has no helper: its whole body is five self-checks (`:177-195`) run
//! against `browser_bundle_global_this_math_max_min_frozen_source()`'s OWN TEXT
//! (`:117`), before any command is built and without ever invoking `kali`.
//! Between them they name 8 distinct frozen-alias spellings.
//!
//! `scripts/audit-case-migration.py` extracts each of those 8 arguments as a
//! claim and searches only the fields the case runner turns into assertions;
//! `[source]` is excluded from that search by construction (its module
//! docstring: "`body` and everything under `[source]` are program text, not
//! claims about behavior"). The 8 are *read*, not *asserted on output*, so no
//! honest migration can put them in an assertion field -- doing so would invent
//! a claim the source never made -- and the audit reports them absent no matter
//! what the migrated `[source]` contains.
//!
//! Same shape as the Task 18 pilot's `browser_math_pow_exponent_one.rs`, batch
//! 2's `browser_array_from_set_map_bundle.rs` and batch 3's
//! `browser_math_abs_sign_frozen_aliases.rs`. The controller has ruled that
//! `audit-case-migration.py` is NOT extended for it (ruling 4), so this is
//! escalated per rule 3/4 and the affected test is retained hand-written. U4's
//! trim-and-keep applied: this is a partial retention, not a whole-file one,
//! and the trim is done -- this file is now exactly its retained remainder.
//!
//! CONSEQUENCE FOR THE GATES -- THE COMPLETE RED-LIST (ruling 9). Every line
//! below was produced by RUNNING the gate against both sides, not by reasoning
//! about it, and it is NOT copied from another retention's list.
//!
//!   PRE-TRIM REF:  f712bdbf4b   (the commit before batch 5's migration commit)
//!   git show f712bdbf4b:crates/kali_cli/tests/browser_math_max_min_frozen_aliases.rs > /tmp/pretrim.rs
//!
//! A trimmed retention makes the post-trim `.rs` the WRONG left-hand side: the
//! migrated cases were produced from the file as it stood BEFORE the trim, so
//! any gate that compares case file against source must be given the pre-trim
//! ref. Read the two columns as POST-trim (the plain
//! `verify_pair.sh math_max_min_frozen_aliases --allow-empty` run) then PRE-trim.
//!
//!   audit-case-migration.py      RED / RED, and BYTE-IDENTICAL both ways -- the
//!        same 8 claims absent, the 8 needles the retained test reads out of the
//!        fixture builder's own text. This is the escalation itself, not a trim
//!        artifact, and the pre-trim ref does NOT rescue this gate: the needles
//!        are read, never asserted on output, so no case file can carry them at
//!        any strength without inventing a claim (rule 2). Controller ruling 4
//!        is explicit that the script is not extended for this shape, so it is
//!        escalated per rule 3/4 and shipped red, documented, rather than
//!        greened by moving fixture text onto an assertion key.
//!   comment_coverage.py          RED / green. Post-trim, every non-blank line of
//!        this header comes back missing: the checker requires each source
//!        comment line to appear in some case's rationale, and this header is
//!        prose about the RETAINED test, which by construction has no case. NO
//!        COUNT IS GIVEN, deliberately -- any figure would count this header's
//!        own length and would be invalidated by every edit to it, including
//!        the edit that corrected it. Pre-trim the source carries no Rust
//!        comment at all, so the run is the vacuous green `--allow-empty`
//!        acknowledges.
//!   check_rationale_fn_names.py  RED / green -- 6 unexplained post-trim, 0
//!        pre-trim. Every one is a helper or parameter name that left with the
//!        migrated cases; the checker resolves names only against the `.rs` it
//!        is handed.
//!   check_fixtures.py            green / green.
//!   check_extra_claims.py        RED / green. Post-trim the migrated cases'
//!        claim strings are absent from the trimmed remainder, so they all
//!        report as unexplained extras; pre-trim every one of them resolves.
//!        NO COUNT IS GIVEN, deliberately, and the reason is specific rather
//!        than cautious: this gate accepts any claim string that appears
//!        verbatim ANYWHERE in the `.rs`, comments included, so the prose of
//!        this very header supplies justification for some of them and lowers
//!        the figure. Measured, not supposed -- running the gate against this
//!        file with and without the header block differs by exactly the claims
//!        `build`, `command`, `json` and `run` -- ordinary English words
//!        that happen to also be
//!        argv tokens or envelope keys. Any integer written here would
//!        therefore be a number describing this header, invalidated by every
//!        edit to it. Run the gate for today's figure; the durable fact is the
//!        classification.
//!
//! Against the PRE-TRIM ref, four of the five gates exit 0. The fifth,
//! `audit-case-migration.py`, stays red for the reason given above, and that
//! red IS the escalation -- not a defect in the migration. That is the run that
//! gates this
//! migration; it is the one to reproduce.
//!
//! NOTE FOR WHOEVER EDITS THIS BLOCK, and it is the reason the extra-claims
//! line carries no integer: `check_extra_claims.py` treats a claim string as
//! justified if it occurs anywhere in the `.rs`, INCLUDING inside a comment.
//! A retention header is therefore part of that gate's input, and prose edits
//! move its result. This was first measured during batch 5's retroactive sweep,
//! on `browser_math_abs_sign_frozen_aliases.rs`, where a draft paragraph that
//! spelled a JSON leaf's dotted path dropped that file's unexplained count by
//! one. Do not add a claim-shaped LITERAL here -- the ordinary English words
//! already present are unavoidable and accounted for above, but a quoted
//! needle or dotted path would silently green a real claim.
//!
//! Adding a new gate to `verify_pair.sh` includes updating this paragraph, in
//! the same change (ruling 9).
//!
//! This file must NOT be deleted by the family-wide sweep after batch 8. See
//! `.superpowers/sdd/2026-07-29-test-binary-consolidation/
//! task-18-batch5-report.md` for the full account.
use std::sync::OnceLock;

fn browser_bundle_global_this_math_max_min_frozen_source() -> &'static str {
    static SOURCE: OnceLock<String> = OnceLock::new();
    SOURCE
        .get_or_init(|| {
            r##"// kali-tree-shake: globalThisMathMaxMinFrozenAliases
function globalThisMathMaxMinFrozenAliases() {
  const value = 2;
  const alias = value;
  console.log(globalThis.Math.max(1, alias, 3));
  console.log(globalThis.Math.min(3, alias, 1));
  console.log(Object.freeze(Math.max)(1, alias, 3));
  console.log(Object.freeze(Math.min)(3, alias, 1));
  console.log(Object.freeze(globalThis.Math["max"])(1, alias, 3));
  console.log(Object.freeze(globalThis.Math["min"])(3, alias, 1));
  console.log(Object.freeze(globalThis.Math['max'])(1, alias, 3));
  console.log(Object.freeze(globalThis.Math['min'])(3, alias, 1));
  console.log(Object.freeze(globalThis["Math"]["max"])(1, alias, 3));
  console.log(Object.freeze(globalThis["Math"]["min"])(3, alias, 1));
  console.log(Object.freeze(globalThis["Math"]['max'])(1, alias, 3));
  console.log(Object.freeze(globalThis["Math"]['min'])(3, alias, 1));
  console.log(Object.freeze(globalThis['Math']['max'])(1, alias, 3));
  console.log(Object.freeze(globalThis['Math']['min'])(3, alias, 1));
  console.log(Object.freeze(globalThis['Math'].max)(1, alias, 3));
  console.log(Object.freeze(globalThis['Math'].min)(3, alias, 1));
  console.log(Object.freeze(Math["max"])(1, alias, 3));
  console.log(Object.freeze(Math["min"])(3, alias, 1));
  console.log(Object.freeze(Math['max'])(1, alias, 3));
  console.log(Object.freeze(Math['min'])(3, alias, 1));
  return [
    globalThis.Math.max(1, alias, 3),
    globalThis.Math.min(3, alias, 1),
    Object.freeze(Math.max)(1, alias, 3),
    Object.freeze(Math.min)(3, alias, 1),
    Object.freeze(globalThis.Math["max"])(1, alias, 3),
    Object.freeze(globalThis.Math["min"])(3, alias, 1),
    Object.freeze(globalThis.Math['max'])(1, alias, 3),
    Object.freeze(globalThis.Math['min'])(3, alias, 1),
    Object.freeze(globalThis["Math"]["max"])(1, alias, 3),
    Object.freeze(globalThis["Math"]["min"])(3, alias, 1),
    Object.freeze(globalThis["Math"]['max'])(1, alias, 3),
    Object.freeze(globalThis["Math"]['min'])(3, alias, 1),
    Object.freeze(globalThis['Math']['max'])(1, alias, 3),
    Object.freeze(globalThis['Math']['min'])(3, alias, 1),
    Object.freeze(globalThis['Math'].max)(1, alias, 3),
    Object.freeze(globalThis['Math'].min)(3, alias, 1),
    Object.freeze(Math["max"])(1, alias, 3),
    Object.freeze(Math["min"])(3, alias, 1),
    Object.freeze(Math['max'])(1, alias, 3),
    Object.freeze(Math['min'])(3, alias, 1),
  ];
}
"##
            .to_string()
        })
        .as_str()
}

#[test]
fn browser_bundle_global_this_math_max_min_frozen_source_includes_direct_frozen_math_aliases() {
    let source = browser_bundle_global_this_math_max_min_frozen_source();
    assert!(
        source.contains("Object.freeze(Math.max)"),
        "source: {source}"
    );
    assert!(
        source.contains("Object.freeze(Math.min)"),
        "source: {source}"
    );
    assert!(
        source.contains("Object.freeze(globalThis.Math[\"max\"])")
            && source.contains("Object.freeze(globalThis.Math[\"min\"])"),
        "source: {source}"
    );
    assert!(
        source.contains("Object.freeze(globalThis[\"Math\"][\"max\"])")
            && source.contains("Object.freeze(globalThis[\"Math\"][\"min\"])"),
        "source: {source}"
    );
    assert!(
        source.contains("Object.freeze(Math[\"max\"])")
            && source.contains("Object.freeze(Math[\"min\"])"),
        "source: {source}"
    );
}
