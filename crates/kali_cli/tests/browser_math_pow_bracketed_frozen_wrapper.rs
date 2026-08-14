//! Task 18 batch 5 audit escalation, TRIMMED: this file now holds exactly the
//! one `#[test]` its fixture-introspecting body blocks from migrating, plus the
//! single fixture builder that test reads.
//!
//! It originally had 9 `#[test]` fns. The other 8 -- every `build_emits_*` and
//! `json_build_emits_*` fn, one real invocation each of
//! `assert_browser_bundle_bracketed_global_this_math_pow_frozen` -- are
//! migrated to `tests/cases/browser/math_pow_bracketed_frozen_wrapper.toml`
//! (2 `[[case]]` x a file-wide `ext` `[matrix]` of js/ts/jsx/tsx = 8 trials).
//! That helper, `kali_bin()` and the `fs`/`Command`/`Value`/`tempdir` imports
//! went with it; nothing left here is unused.
//!
//! WHAT BLOCKS THE ONE RETAINED TEST.
//! `browser_bundle_bracketed_global_this_math_pow_frozen_source_includes_parenthesized_bracketed_aliases`
//! (`:148`) has no helper: its whole body is a single `assert!(...)` self-check
//! (`:153`) run in a `for` loop over
//! `kali_common::math_pow_bracketed_frozen_callable_aliases()`, against
//! `browser_bundle_bracketed_global_this_math_pow_frozen_source`
//! (`:130`)'s OWN TEXT, before any command is built and without ever invoking
//! `kali`.
//!
//! It is doubly unmigratable, and the second reason is the sharper one:
//!   1. `scripts/audit-case-migration.py` extracts a `.contains` argument as a
//!      claim and searches only the fields the case runner turns into
//!      assertions; `[source]` is excluded from that search by construction. A
//!      fixture-text read is indistinguishable to it from an output assertion,
//!      so migrating this test would produce a false green.
//!   2. The claim is about `[source]` TEXT, and the format has no step kind
//!      that asserts on it. Every assertion key in design spec 5.4 is about a
//!      process's stdout/stderr/JSON output; this test runs no process at all.
//!      So the claim is not expressible at any strength, which is rule 4's
//!      condition exactly. The needles themselves are perfectly enumerable --
//!      `kali_common`'s alias list is a compile-time slice of literals, and
//!      every one already appears verbatim in the migrated case file's
//!      `[source]` bodies -- so what blocks migration is reason 1 plus the
//!      absence of a `[source]`-text assertion, not any inability to obtain
//!      them.
//!
//! Same shape as the Task 18 pilot's `browser_math_pow_exponent_one.rs` and
//! batch 4's `browser_math_floor_trunc_ceil_aliases.rs`; the controller has
//! ruled the script is NOT extended for it (ruling 4), so this is escalated per
//! rule 3/4 and the affected test is retained hand-written. U4's trim-and-keep
//! applied: this is a partial retention (1 of 9), not a whole-file one, and the
//! trim is done -- this file is now exactly its retained remainder.
//!
//! CONSEQUENCE FOR THE GATES -- THE COMPLETE RED-LIST (ruling 9). Every line
//! below was produced by RUNNING the gate against both sides, not by reasoning
//! about it, and it is NOT copied from another retention's list.
//!
//!   PRE-TRIM REF:  f712bdbf4b   (the commit before batch 5's migration commit)
//!   git show f712bdbf4b:crates/kali_cli/tests/browser_math_pow_bracketed_frozen_wrapper.rs > /tmp/pretrim.rs
//!
//! A trimmed retention makes the post-trim `.rs` the WRONG left-hand side: the
//! migrated cases were produced from the file as it stood BEFORE the trim, so
//! any gate that compares case file against source must be given the pre-trim
//! ref. Read the two columns as POST-trim (the plain
//! `verify_pair.sh math_pow_bracketed_frozen_wrapper --allow-empty` run) then PRE-trim.
//!
//!   audit-case-migration.py      INAPPLICABLE (exit 3) / green (exit 0). Both
//!        re-measured; the post-trim column read "green" until the audit gained
//!        its third verdict. Post-trim the retained test's needle is a LOOP
//!        VARIABLE, so 0 claims are demanded either way and the run decides
//!        nothing -- it never audited this migration; the pre-trim run below
//!        does. Same shape as batch 4's `browser_math_floor_trunc_ceil_aliases.rs`.
//!   comment_coverage.py          RED / green. Post-trim, every non-blank line of
//!        this header comes back missing: the checker requires each source
//!        comment line to appear in some case's rationale, and this header is
//!        prose about the RETAINED test, which by construction has no case. NO
//!        COUNT IS GIVEN, deliberately -- any figure would count this header's
//!        own length and would be invalidated by every edit to it, including
//!        the edit that corrected it. Pre-trim the source carries no Rust
//!        comment at all, so the run is the vacuous green `--allow-empty`
//!        acknowledges.
//!   check_rationale_fn_names.py  RED / green -- 3 unexplained post-trim, 0
//!        pre-trim. Every one is a helper or parameter name that left with the
//!        migrated cases; the checker resolves names only against the `.rs` it
//!        is handed.
//!   check_fixtures.py            green / green.
//!   batch5_crosscheck.py         RED / green -- the citation gate, wired into
//!        `verify_pair.sh` by batch 6; this row is part of that same wiring
//!        change, which is what ruling 9 requires and what batch 4 failed to do
//!        when it added `check_extra_claims.py`. Every `:N` in the case file is
//!        a PRE-TRIM line number -- this header says so above -- so resolving
//!        them against the trimmed remainder lands them in unrelated code. That
//!        is precisely the artifact the tool's `STEM=PRETRIM.rs` argument
//!        exists for. NO COUNT IS GIVEN: this gate also resolves THIS header's
//!        own `:N` citations, so every edit to this paragraph is an input to
//!        the figure it would report (ruling 11).
//!   check_extra_claims.py        RED / green. Post-trim the migrated cases'
//!        claim strings are absent from the trimmed remainder, so they all
//!        report as unexplained extras; pre-trim every one of them resolves.
//!        NO COUNT IS GIVEN, deliberately, and the reason is specific rather
//!        than cautious: this gate accepts any claim string that appears
//!        verbatim ANYWHERE in the `.rs`, comments included, so the prose of
//!        this very header supplies justification for some of them and lowers
//!        the figure. Measured, not supposed -- running the gate against this
//!        file with and without the header block differs by exactly the claims
//!        `build`, `command` and `json`, which are ordinary English words that happen to also be
//!        argv tokens or envelope keys. Any integer written here would
//!        therefore be a number describing this header, invalidated by every
//!        edit to it. Run the gate for today's figure; the durable fact is the
//!        classification.
//!
//! Against the PRE-TRIM ref, ALL SIX gates exit 0. That is the run that
//! gates this migration; it is the one to reproduce.
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
//! the batch's own working report -- which was git-ignored scratch and
//! does not ship, so it is deliberately not cited by path.
use kali_common::{
    math_pow_bracketed_frozen_callable_aliases,
    math_pow_bracketed_frozen_callable_invocation_entries,
    math_pow_bracketed_frozen_callable_invocation_lines,
};

fn browser_bundle_bracketed_global_this_math_pow_frozen_source() -> String {
    format!(
        r##"// kali-tree-shake: bracketedGlobalThisMathPowFrozenWrapper
function bracketedGlobalThisMathPowFrozenWrapper() {{
  const exponent = 3;
  const alias = exponent;
  {}
  return [
{}
  ];
}}
"##,
        math_pow_bracketed_frozen_callable_invocation_lines("  "),
        math_pow_bracketed_frozen_callable_invocation_entries("    "),
    )
}

#[test]
fn browser_bundle_bracketed_global_this_math_pow_frozen_source_includes_parenthesized_bracketed_aliases(
) {
    let source = browser_bundle_bracketed_global_this_math_pow_frozen_source();

    for expected in math_pow_bracketed_frozen_callable_aliases() {
        assert!(source.contains(expected), "source: {source}");
    }
}
