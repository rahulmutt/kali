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
//! (`:84-93`) has no helper: its whole body is a single
//! `assert!(source.contains(expected))` self-check (`:88-91`) run in a `for`
//! loop over `kali_common::math_floor_trunc_ceil_frozen_callable_aliases()`,
//! against `browser_harness_math_floor_trunc_ceil_run_source()`'s OWN TEXT
//! (`:71-81`), before any command is built and without ever invoking `kali`.
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
