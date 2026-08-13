//! Task 18 batch 8C design-spec 5.11 retention: kept 100% hand-written, not
//! migrated. No case file exists for this target.
//!
//! WHY IT SURVIVES. The design spec's own 5.11 table names this target by name
//! -- `browser_harness_failing_test_propagates_failure`, "asserts harness
//! failure propagation" -- as one of the two `browser_*` entries in that table,
//! alongside `browser_cdp_smoke`. Enumerating command (ruling 13), run before
//! the family deletion:
//!
//!     awk '/^### 5.11/,/^## 6/' \
//!       docs/superpowers/specs/2026-07-29-test-binary-consolidation-design.md \
//!       | grep -c '^| `browser_'
//!
//! The implementation plan repeats the keep-list twice more, in
//! docs/superpowers/plans/2026-07-29-test-binary-consolidation.md at lines
//! 3546-3547 ("Four `browser_*` targets are **not** migrated and must remain
//! untouched") and again at line 3671. No migration batch ever had this file in
//! scope and none touched it.
//!
//! WHY IT NEEDED SAYING, WHICH IS THE POINT OF U3. Until batch 8C wrote this
//! header, nothing in the tree recorded any of the above. Its keep-list partner
//! `browser_cdp_smoke.rs` has carried a `//!` header throughout; this file had
//! none, no same-stem case file, and no case file claiming it -- which is
//! byte-for-byte the signature of an OVERLOOKED MIGRATION TARGET rather than an
//! adjudicated retention. U3 exists for exactly this: a retention whose
//! reasoning lives only in a plan is indistinguishable from a skipped file, and
//! after the family deletion this would have been the sole unexplained `.rs`
//! left in the directory.
//!
//! THE DISTINCTION THAT SAVED IT. At batch 8C's BASE, nine `browser_*.rs`
//! carried no `//!` header AND had no same-stem case file. Eight of the nine
//! are U2 splits -- one source migrated into two case files, each naming it in
//! its own `Migrated from` line -- and were deleted as fully migrated. This
//! file is the ninth, and it is claimed by ZERO case files:
//!
//!     grep -l "Migrated from tests/browser_harness_failing_test_propagates_failure\.rs" \
//!       crates/kali_cli/tests/cases/browser/*.toml
//!
//! printed nothing, while the same command run for each of the other eight
//! printed two paths apiece. A classifier keyed on "no same-stem case file"
//! keeps all nine and deletes nothing; one keyed on "no header and no case
//! file" deletes all nine, this one included. Neither predicate is a
//! classifier: the `Migrated from` resolution is what separates the two groups,
//! and `citation_tiers.resolve_case_stem` is the implementation of record.
//!
//! WHOLE-FILE, BY ARITHMETIC (U4). The file holds exactly one `#[test]` fn,
//! `browser_harness_failing_registered_test_reports_failure_and_nonzero_exit`
//! (`:108`), so U4's trim-and-keep degenerates to whole-file retention:
//! there is no complementary migratable subset to split off, and the "does
//! every test reach the construct" question U4 asks answers itself.
//!
//! WHAT IT ASSERTS. A registered `Kali.test` whose body throws
//! (`failing_browser_test_source`, `:97`) runs under `kali --output json
//! test --api browser` with `KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node`. The
//! trap escapes the JS harness's per-callback try/catch and kills the process
//! before the summary is written, so the Rust crash-lane synthesises the
//! result. The test pins that the process exits non-zero
//! (`!output.status.success()`, `:126`), the JSON envelope's
//! command/success/exitCode, the payload's total/passed/failed counts, and that
//! the failure is carried purely by the failed-test count with an EMPTY errors
//! array (`json["errors"]`, `:142`) -- which is what distinguishes the
//! trap-swallow class from a compile diagnostic.
//!
//! NO FORMAT-BLOCKER IS CLAIMED, AND THAT IS DELIBERATE. This family's other
//! 5.11 retentions each name a construct the twelve assertion keys cannot carry
//! -- a `stdout.lines()` count, an `errors.iter().all(...)` quantifier. This
//! retention does not rest on such a finding and batch 8C did not manufacture
//! one so the header would resemble its neighbours. The ground is the spec
//! table and the plan keep-list, which outrank any re-adjudication a migration
//! batch could make on its own. Rule 2's spirit binds retention prose as much
//! as it binds assertions: do not state what you did not establish.
//!
//! CONSEQUENCE FOR THE GATES (ruling 9): THIS FILE HAS NO RED-LIST, and that is
//! the finding rather than an omission. Ruling 9 addresses a U4 trim-and-keep
//! retention, whose on-disk `.rs` is shorter than the source its case file was
//! migrated from. Nothing was trimmed here and there is no case file, so there
//! is no pre-trim/post-trim divergence and no pair for
//! `audit-case-migration.py`, `comment_coverage.py`, the U8 check or
//! `check_extra_claims.py` to be red against. Ruling 12's third column, the
//! migrated complement, is not applicable either: the complement of a
//! whole-file retention is empty. In `citation_sweep.sh` this file joins the
//! RETENTION population, where the arm that runs is the header-citation check.
use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

/// A registered test whose self-check fails via `throw`. Under the print-then-
/// trap `throw` lowering, the test's body executes inline during `_start` and
/// the trap escapes the JS harness's per-callback try/catch, killing the process
/// before the summary is written. The Rust crash-lane counts this as a failed
/// test (no compile/trap diagnostic) — the exact trap-swallow class.
fn failing_browser_test_source() -> &'static str {
    r#"Kali.test('self-check throw propagates as a failure', () => {
  const actual = 1;
  if (actual !== 2) {
    throw 'expected 2';
  }
});
"#
}

#[test]
fn browser_harness_failing_registered_test_reports_failure_and_nonzero_exit() {
    let dir = tempdir().expect("tempdir");
    let source_path = dir.path().join("failing.test.js");
    fs::write(&source_path, failing_browser_test_source()).expect("write source");

    let output = Command::new(kali_bin())
        .current_dir(dir.path())
        .env("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", "node")
        .arg("--output")
        .arg("json")
        .arg("test")
        .arg("--api")
        .arg("browser")
        .arg(&source_path)
        .output()
        .expect("run kali");

    assert!(
        !output.status.success(),
        "process should exit non-zero for a failing registered test\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let json: Value = serde_json::from_slice(&output.stdout).expect("json stdout");
    assert_eq!(json["command"], "test");
    assert_eq!(json["success"], false, "json: {json}");
    assert_eq!(json["exitCode"], 1, "json: {json}");
    assert_eq!(json["payload"]["total"], 1, "json: {json}");
    assert_eq!(json["payload"]["passed"], 0, "json: {json}");
    assert_eq!(json["payload"]["failed"], 1, "json: {json}");
    // The failure is carried purely by the failed-test count, not a diagnostic:
    // this is what distinguishes the trap-swallow class from a compile error.
    assert!(
        json["errors"].as_array().expect("errors array").is_empty(),
        "expected no compile/trap diagnostics, only a failed test count; json: {json}"
    );
}
