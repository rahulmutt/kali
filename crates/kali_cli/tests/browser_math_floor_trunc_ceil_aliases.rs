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
//! (`:73-82`) has no helper: its whole body is a single
//! `assert!(source.contains(expected))` self-check (`:77-80`) run in a `for`
//! loop over `kali_common::math_floor_trunc_ceil_frozen_callable_aliases()`,
//! against `browser_harness_math_floor_trunc_ceil_run_source()`'s OWN TEXT
//! (`:60-70`), before any command is built and without ever invoking `kali`.
//!
//! It is doubly unmigratable, and the second reason is the sharper one:
//!   1. `scripts/audit-case-migration.py` extracts every `.contains(<literal>)`
//!      argument as a claim and searches only the fields the case runner turns
//!      into assertions; `[source]` is excluded from that search by
//!      construction. A fixture-text read is indistinguishable to it from an
//!      output assertion, so migrating this test would produce a false green.
//!   2. There is no literal to migrate at all. The needle is `expected`, a
//!      loop variable bound to a RUNTIME-COMPUTED inventory (81 alias
//!      spellings today) returned by a `kali_common` function. The case format
//!      has no step kind that asserts about `[source]` text, and no assertion
//!      key whose needles come from another crate's function. The claim is not
//!      expressible at any strength, which is rule 4's condition exactly.
//!
//! Same shape as the Task 18 pilot's `browser_math_pow_exponent_one.rs`, batch
//! 2's `browser_array_from_set_map_bundle.rs` and batch 3's
//! `browser_math_atan2_global_this_root.rs`; the controller has ruled the
//! script is NOT extended for it (ruling 4), so this is escalated per rule 3/4
//! and the affected test is retained hand-written. U4's trim-and-keep applied:
//! this is a partial retention (1 of 17), not a whole-file one, and the trim is
//! done -- this file is now exactly its retained remainder.
//!
//! CONSEQUENCE FOR THE GATES, measured rather than assumed. Auditing this
//! POST-trim file against the shipped case file reports its claims absent, and
//! `comment_coverage.py` reports this header's own lines as missing from every
//! `rationale`. Both are expected for a trimmed retention: the header describes
//! the RETAINED test, which by construction has no case. Audit the migrated 16
//! against the PRE-TRIM source (git history), where the audit exits 0.
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
