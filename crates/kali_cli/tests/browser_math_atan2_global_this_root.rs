//! Task 18 batch 3 audit escalation, TRIMMED: this file now holds exactly the
//! one `#[test]` its fixture-introspecting body blocks from migrating, plus the
//! single fixture builder that test reads.
//!
//! It originally had 19 `#[test]` fns. The other 18 -- every `build_emits_*`,
//! `json_build_emits_*`, `run_supports_*` and `test_supports_*` fn, which
//! between them expanded (through their `for filename in [...]` loops) into 69
//! real invocations across
//! `assert_browser_bundle_global_this_math_atan2`,
//! `assert_browser_bundle_global_this_math_atan2_source`,
//! `assert_browser_bundle_global_this_math_atan2_await_wrapped` and
//! `assert_browser_harness_global_this_math_atan2` -- are migrated to
//! `tests/cases/browser/math_atan2_global_this_root.toml` (69 named sibling
//! `[[case]]` entries, no `[matrix]`, audited against the pre-trim source and
//! green). All four of those helpers, the five other fixture builders,
//! `kali_bin()` and the `fs`/`Command`/`Value`/`tempdir` imports went with
//! them; nothing left here is unused.
//!
//! WHAT BLOCKS THE ONE RETAINED TEST.
//! `browser_bundle_global_this_math_atan2_frozen_source_includes_direct_frozen_callable_aliases`
//! (`:71-86`) has no helper: its whole body is three
//! `assert!(source.contains(<needle>))` self-checks (`:73-85`) --
//! one of them itself an OR across two quoting spellings -- run against
//! `browser_bundle_global_this_math_atan2_frozen_source()`'s OWN TEXT
//! (`:52-68`), before any command is built and without ever
//! invoking `kali`. The four blocking literals are
//! `Object.freeze(globalThis.Math.atan2)`,
//! `Object.freeze(globalThis['Math']['atan2'])`, its double-quoted sibling
//! (identical but spelling both bracket keys with `"` instead of `'`) and
//! `Object.freeze(Math.atan2)`.
//!
//! `scripts/audit-case-migration.py` extracts every `.contains(<literal>)`
//! argument as a claim and searches only the fields the case runner turns into
//! assertions; `[source]` is excluded from that search by construction. These
//! four literals are *read*, not *asserted on output*, so no honest migration
//! can put them in an assertion field, and the audit reports them absent
//! regardless of what the migrated `[source]` contains. Verified, not assumed:
//! auditing the PRE-TRIM source against the shipped case file reports
//! `AUDIT FAILED -- 4 claim(s) absent`, listing exactly those four literals and
//! nothing else.
//!
//! Same shape as the Task 18 pilot's `browser_math_pow_exponent_one.rs` and
//! batch 2's `browser_array_from_set_map_bundle.rs`; the controller has ruled
//! the script is NOT extended for it (ruling 4), so this is escalated per rule
//! 3/4 and the affected test is retained hand-written. U4's trim-and-keep
//! applied: this is a partial retention, not a whole-file one, and the trim is
//! done -- this file is now exactly its retained remainder.
//!
//! This file must NOT be deleted by the family-wide sweep after batch 8. See
//! `.superpowers/sdd/2026-07-29-test-binary-consolidation/
//! task-18-batch3-report.md` for the full account.
fn browser_bundle_global_this_math_atan2_frozen_source() -> &'static str {
    r##"// kali-tree-shake: globalThisMathAtan2FrozenCallableAliases
function globalThisMathAtan2FrozenCallableAliases() {
  const zero = 0;
  const one = 1;
  const frozenDotRoot = Object.freeze(globalThis.Math.atan2);
  const frozenBracketedRoot = Object.freeze(globalThis["Math"]["atan2"]);
  const frozenSingleQuotedRoot = Object.freeze(globalThis['Math']['atan2']);
  const frozenDirect = Object.freeze(Math.atan2);
  console.log(frozenDotRoot(zero, one));
  console.log(frozenBracketedRoot(zero, one));
  console.log(frozenSingleQuotedRoot(zero, one));
  console.log(frozenDirect(zero, one));
  return [frozenDotRoot(zero, one), frozenBracketedRoot(zero, one), frozenSingleQuotedRoot(zero, one), frozenDirect(zero, one)];
}
"##
}

#[test]
fn browser_bundle_global_this_math_atan2_frozen_source_includes_direct_frozen_callable_aliases() {
    let source = browser_bundle_global_this_math_atan2_frozen_source();
    assert!(
        source.contains("Object.freeze(globalThis.Math.atan2)"),
        "source: {source}"
    );
    assert!(
        source.contains("Object.freeze(globalThis[\"Math\"][\"atan2\"])")
            || source.contains("Object.freeze(globalThis['Math']['atan2'])"),
        "source: {source}"
    );
    assert!(
        source.contains("Object.freeze(Math.atan2)"),
        "source: {source}"
    );
}
