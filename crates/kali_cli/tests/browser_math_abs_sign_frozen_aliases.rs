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
//! (`:80-98`) has no helper: its whole body is four
//! `assert!(source.contains(<needle>))` self-checks (`:82-97`)
//! run against `browser_bundle_global_this_math_abs_sign_frozen_source()`'s OWN
//! TEXT (`:50-77`), before any command is built and without ever
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
