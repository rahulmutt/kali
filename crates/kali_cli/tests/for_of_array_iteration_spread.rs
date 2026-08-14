//! U4 TRIM-AND-KEEP RETENTION (Task 19 batch 5). Thirty-four of this file's
//! thirty-five `#[test]` fns migrated to
//! `cases/misc/for_of_array_iteration_spread.toml`; the one that remains stays
//! hand-written per spec §5.11.
//!
//! PRE-TRIM REF: 47e9b083c61e32c972727189a580d1e9cacb856c
//!
//! THE BLOCKING CONSTRUCT, BY NAME AND LINE. The retained
//! `browser_harness_test_wrapper_reuses_the_shared_array_from_inventory_in_both_loop_sections` (defined at line 131 of this file)
//! asserts against the FIXTURE'S OWN TEXT and never against a process: it binds
//! `browser_harness_array_from_source("test")` and then makes twenty-three
//! `assert_eq!(source.matches(alias).count(), 2, ..)` claims about that string.
//! It runs no `kali` command at all, so it produces no trial a case file could
//! carry.
//!
//! WHY NEITHER THE AUDIT NOR THE FORMAT CAN CARRY IT. The case format's whole
//! vocabulary is claims about a process's exit status, its two text streams and
//! the JSON document on its stdout (design spec §5.4). "This fixture's own text
//! contains each of these twenty-three aliases exactly twice" is not a claim
//! about a process, and there is no key that could express it -- migrating it
//! would mean inventing a trial the source never ran, which rule 2 forbids. It
//! is also invisible to `audit-case-migration.py` for controller ruling 4's
//! reason: the `.contains`/`.matches` receiver is a fixture-builder's return
//! value, read before any command is built, so a literal-coverage tool has
//! nothing to correspond it to.
//!
//! ONLY ONE TEST REACHES IT -- one of thirty-five, which is why this is a TRIM
//! and not a whole-file retention. U4's second clause makes whole-file
//! legitimate only when EVERY test reaches the construct; here thirty-four do
//! not. Derived rather than chosen: the enumerating command and its output are
//! in the case file's header and in the report.
//!
//! FOUND BY READING, NOT BY THE TOOL, AND THE TOOL IS NOW FIXED. Ruling 10's
//! `find_fixture_self_inspection.py` returned 0 hits on this file, because its
//! predicate required a `.contains` and this site spells the same shape as
//! `.matches(..).count()`. That is a live false negative of exactly the class
//! ruling 10 exists to catch, and it is what let a dispatch list this target as
//! migratable-whole. The predicate now admits a terminal `.matches(..).count()`
//! over a fixture receiver, this file is in its `KNOWN` list, and its
//! `--selftest` therefore re-finds this instance on every run.
//!
//! CONSEQUENCE FOR THE GATES -- THREE COLUMNS (rulings 9, 12 and 19). The
//! retained half carries literal claims of its own (the twenty-three aliases),
//! so a literal-coverage gate is red against BOTH the post-trim file and the
//! pre-trim blob, and the correct left-hand side for a FORWARD coverage gate is
//! the MIGRATED COMPLEMENT built mechanically by `migrated_complement.py`. The
//! correct side is per gate, by DIRECTION OF CHECK (ruling 19), not by whether
//! the gate reads prose:
//!
//!   gate                          post-trim  pre-trim  complement  correct side
//!   audit-case-migration.py       RED        GREEN     GREEN       complement
//!   check_fixtures.py             GREEN      GREEN     GREEN       complement
//!   comment_coverage.py           RED        RED       RED         complement
//!   check_extra_claims.py         RED        GREEN     GREEN       pre-trim
//!   check_rationale_fn_names.py   RED        GREEN     GREEN       pre-trim
//!
//! MEASURED, NOT PREDICTED. Every cell above was produced by running the gate,
//! and the table is written by the generator from those runs
//! (`gen_task19_batch5.redlist`), so a header cannot state a cell it did not
//! measure.
//!
//! Reproduce the column that decides the pair:
//!
//!   cd "$(git rev-parse --show-toplevel)"
//!   git show 47e9b083c61e32c972727189a580d1e9cacb856c:crates/kali_cli/tests/for_of_array_iteration_spread.rs > /tmp/pre.rs
//!   python3 tools/task-18-browser-pilot/migrated_complement.py /tmp/pre.rs crates/kali_cli/tests/for_of_array_iteration_spread.rs > /tmp/complement.rs
//!   python3 scripts/audit-case-migration.py /tmp/complement.rs crates/kali_cli/tests/cases/misc/for_of_array_iteration_spread.toml
//!
//! The full derivation is this header plus the reproduction above and the
//! generator that fills the table, tools/migration/gen_task19_batch5.py --
//! all of which ship. The batch's working report lived in git-ignored
//! scratch and is deliberately not cited: a citation that cannot resolve
//! from a clean checkout is worse than no citation.

use kali_common::{array_from_alias_inventory_source, array_from_loop_lines};

fn array_from_iteration_body() -> String {
    let array_from_source = array_from_alias_inventory_source();
    let frozen_for_of = array_from_loop_lines(&array_from_source, "for (const value of ", "");
    let frozen_for_await =
        array_from_loop_lines(&array_from_source, "for await (const value of ", "");
    format!(
        r#"const values = [1, 2];
for (const value of Array.from(values)) {{
  console.log(value);
}}
for (const value of globalThis.Array.from(values)) {{
  console.log(value);
}}
for (const value of globalThis["Array"].from(values)) {{
  console.log(value);
}}
for (const value of globalThis["Array"]["from"](values)) {{
  console.log(value);
}}
{frozen_for_of}
for await (const value of Array.from(values)) {{
  console.log(value);
}}
for await (const value of globalThis.Array.from(values)) {{
  console.log(value);
}}
for await (const value of globalThis["Array"].from(values)) {{
  console.log(value);
}}
for await (const value of globalThis["Array"]["from"](values)) {{
  console.log(value);
}}
{frozen_for_await}
"#
    )
}

fn browser_harness_array_from_source(command: &str) -> String {
    let body = array_from_iteration_body();

    match command {
        "test" => format!(
            "Kali.test('browser Array.from wrappers', () => {{
  async function browserArrayFromWrappers() {{
{body}  }}
  return browserArrayFromWrappers();
}});
"
        ),
        _ => body,
    }
}

#[test]
fn browser_harness_test_wrapper_reuses_the_shared_array_from_inventory_in_both_loop_sections() {
    let source = browser_harness_array_from_source("test");

    for alias in [
        r#"Object.freeze((Array.from))"#,
        r#"Object.freeze((globalThis.Array.from))"#,
        r#"Object.freeze((globalThis["Array"].from))"#,
        r#"Object.freeze((globalThis["Array"]["from"]))"#,
        r#"Object.freeze((globalThis["Array"])["from"])"#,
        r#"Object.freeze((globalThis['Array']).from)"#,
        r#"Object.freeze((globalThis['Array'])["from"])"#,
        r#"Object.freeze((globalThis.Array).from)"#,
        r#"Object.freeze((globalThis.Array)["from"])"#,
        r#"Object.freeze((globalThis.Array))["from"]"#,
        r#"Object.freeze((globalThis.Array)['from'])"#,
        r#"Object.freeze((null ?? Array.from))"#,
        r#"Object.freeze((true && Array.from))"#,
        r#"Object.freeze((false || Array.from))"#,
        r#"Object.freeze((null ?? globalThis["Array"].from))"#,
        r#"Object.freeze((true && globalThis["Array"].from))"#,
        r#"Object.freeze((false || globalThis["Array"].from))"#,
        r#"Object.freeze((null ?? globalThis["Array"]["from"]))"#,
        r#"Object.freeze((true && globalThis["Array"]["from"]))"#,
        r#"Object.freeze((false || globalThis["Array"]["from"]))"#,
        r#"Object.freeze((null ?? globalThis['Array']['from']))"#,
        r#"Object.freeze((true && globalThis['Array']['from']))"#,
        r#"Object.freeze((false || globalThis['Array']['from']))"#,
    ] {
        assert_eq!(source.matches(alias).count(), 2, "alias {alias}: {source}");
    }
}
