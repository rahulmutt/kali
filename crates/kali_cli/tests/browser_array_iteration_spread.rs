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
