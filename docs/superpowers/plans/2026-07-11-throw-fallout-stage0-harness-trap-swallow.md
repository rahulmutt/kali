# throw-fallout Stage 0 — Harness Trap-Swallow — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `kali test` report failure (`success:false`, exit 1) whenever a registered test fails — including browser-harness callback traps that today increment the failure count but are reported as success.

**Architecture:** The `kali test` command (`crates/kali_cli/src/bin/cmd_test.rs`) computes run success from `diagnostics.is_empty()` at three decision sites (JSON envelope `success`/`exitCode`, the human-readable `ok`/`FAILED` print, and the function's `Ok/Err` return). A browser test callback that traps is caught by the JS harness, counted into the summary's `testsFailed`, and surfaces as `outcome.tests_failed` → `failed += …`, but pushes **no** diagnostic — so `diagnostics.is_empty()` stays true and the run is falsely reported as success while the JSON payload already honestly shows `failed:1`. The fix introduces one pure predicate, `test_run_succeeded(diagnostics_empty, failed)`, requiring **both** no diagnostics **and** zero failed tests, and applies it at all three sites.

**Tech Stack:** Rust (workspace crate `kali_cli`, binary target `kali`), `cargo test`, `serde_json`; browser integration tests drive the compiled `kali` binary with `--api browser` against a `node` harness.

## Global Constraints

These apply to every task in this plan (and every stage of the throw-fallout program). Copied from `docs/superpowers/specs/2026-07-11-throw-fallout-design.md`:

- **Fix, never flip.** Real implementation matching node's observable behavior; no construct is rejected/trapped to pass a test.
- **The gate is `cargo test --workspace`** — the exact CI command, whole workspace, never a subset.
- **Diff the failing set against a `main` worktree** (built at merge-base), never a mid-branch red baseline. Stand up one persistent `main` worktree at `../kali-main` and diff every checkpoint against it.
- **Stage 0 is the one stage that raises red before lowering it.** Honest new failures (browser self-check tests that were fake-green) are the *expected* result of this fix, not a regression. A newly-red test is a regression **only** if it is not a self-check/trap propagation case.
- **No re-masking.** A fix that re-silences a self-check `throw` is a defect even if a test goes green.
- **Node parity** is byte-for-byte on the same fixture where applicable.

---

## Task 1: Unified test-run success predicate in `cmd_test.rs`

**Files:**
- Modify: `crates/kali_cli/src/bin/cmd_test.rs` (add helper near the other free helpers ~line 451; edit the three decision sites at lines 397, 419, 444; add a `#[cfg(test)] mod tests` at end of file)
- Test: same file, embedded `#[cfg(test)] mod tests` (the `kali` bin's unit tests)

**Interfaces:**
- Produces: `fn test_run_succeeded(diagnostics_empty: bool, failed: usize) -> bool` — a private free function in the `cmd_test` module. Returns `true` iff `diagnostics_empty && failed == 0`. Consumed by the three decision sites in the same file and by Task 2's integration reproducer (behaviorally, via the envelope).

- [ ] **Step 1: Write the failing unit tests**

Add at the very end of `crates/kali_cli/src/bin/cmd_test.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::test_run_succeeded;

    #[test]
    fn clean_run_with_no_failures_succeeds() {
        assert!(test_run_succeeded(true, 0));
    }

    #[test]
    fn diagnostics_present_is_a_failure() {
        assert!(!test_run_succeeded(false, 0));
    }

    #[test]
    fn test_failures_without_diagnostics_are_a_failure() {
        // The trap-swallow class: a browser callback trap increments `failed`
        // through the run summary without pushing a diagnostic.
        assert!(!test_run_succeeded(true, 1));
    }

    #[test]
    fn both_diagnostics_and_failures_is_a_failure() {
        assert!(!test_run_succeeded(false, 2));
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p kali_cli --bin kali test_run_succeeded`
Expected: FAIL — compile error `cannot find function 'test_run_succeeded' in this scope` (the helper does not exist yet).

- [ ] **Step 3: Add the helper**

Insert this function into `crates/kali_cli/src/bin/cmd_test.rs` immediately above `fn coverage_function_count_from_wasm` (currently line 451):

```rust
/// A `kali test` run succeeds only when it produced no diagnostics AND every
/// registered test passed. Test failures reported purely through the run
/// summary — e.g. a browser-harness callback trap the JS harness catches and
/// counts into `testsFailed` — increment `failed` without pushing a
/// diagnostic, so `diagnostics.is_empty()` alone must not decide success.
/// (throw-fallout Stage 0: harness trap-swallow.)
fn test_run_succeeded(diagnostics_empty: bool, failed: usize) -> bool {
    diagnostics_empty && failed == 0
}
```

- [ ] **Step 4: Run the unit tests to verify they pass**

Run: `cargo test -p kali_cli --bin kali test_run_succeeded`
Expected: PASS — 4 tests pass.

- [ ] **Step 5: Apply the predicate at decision site 1 (JSON envelope success + exit code)**

In `crates/kali_cli/src/bin/cmd_test.rs`, replace line 397:

```rust
        let success = diagnostics.is_empty();
```

with:

```rust
        let success = test_run_succeeded(diagnostics.is_empty(), failed);
```

(The `exitCode` on line 409, `if success { 0 } else { 1 }`, now follows automatically.)

- [ ] **Step 6: Apply the predicate at decision site 2 (human-readable ok/FAILED print)**

In the same file, replace line 419:

```rust
        if diagnostics.is_empty() {
```

with:

```rust
        if test_run_succeeded(diagnostics.is_empty(), failed) {
```

(The `else` branch already prints `FAILED {failed}`, which is now reached when a test failed without a diagnostic.)

- [ ] **Step 7: Apply the predicate at decision site 3 (command return / process exit)**

In the same file, replace the tail block at lines 444–448:

```rust
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(1)
    }
```

with:

```rust
    if test_run_succeeded(diagnostics.is_empty(), failed) {
        Ok(())
    } else {
        Err(1)
    }
```

- [ ] **Step 8: Verify the crate still builds and its unit tests pass**

Run: `cargo test -p kali_cli --bin kali`
Expected: PASS — the bin's unit tests (including the 4 new ones) pass; no compile errors.

- [ ] **Step 9: Commit**

```bash
git add crates/kali_cli/src/bin/cmd_test.rs
git commit -m "fix(cli): kali test success requires zero failed tests, not just no diagnostics (throw-fallout Stage 0)"
```

---

## Task 2: Browser integration reproducer — a throwing `Kali.test` fails the run

**Files:**
- Create: `crates/kali_cli/tests/browser_harness_failing_test_propagates_failure.rs`
- Consumes: the compiled `kali` binary (`CARGO_BIN_EXE_kali`); the `--api browser` path with a `node` harness (`KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node`); the envelope behavior fixed in Task 1.

**Interfaces:**
- Consumes: Task 1's fix — with it, a browser test callback that traps yields `success:false`, `exitCode:1`, `payload.failed:1`, and empty `errors` (the failure comes purely through `failed`, with no compile/trap diagnostic).

- [ ] **Step 1: Write the failing integration test**

Create `crates/kali_cli/tests/browser_harness_failing_test_propagates_failure.rs`:

```rust
use std::{fs, process::Command};

use serde_json::Value;
use tempfile::tempdir;

fn kali_bin() -> String {
    std::env::var("CARGO_BIN_EXE_kali").expect("kali binary path")
}

/// A registered test whose self-check fails via `throw`. Under the print-then-
/// trap `throw` lowering this traps in the guest; the browser JS harness
/// catches the trap and counts it into the summary's `testsFailed` without
/// producing a compile/trap diagnostic — the exact trap-swallow class.
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
```

- [ ] **Step 2: Run the integration test on the pre-Task-1 binary to confirm the swallow (diagnostic check)**

> Only run this step if you are validating the reproducer against an unfixed tree (e.g. a `main` worktree). On the current branch Task 1 is already applied, so skip to Step 3.

Run (in a `main` worktree): `cargo test -p kali_cli --test browser_harness_failing_test_propagates_failure -- --nocapture`
Expected: FAIL on the `success`/`exitCode` assertions — the pre-fix envelope reports `"success": true`, `"exitCode": 0` even though `"payload":{"failed":1}`. This is the trap-swallow reproduced.

- [ ] **Step 3: Run the integration test to verify it passes with the fix**

Run: `cargo test -p kali_cli --test browser_harness_failing_test_propagates_failure`
Expected: PASS — `success:false`, `exitCode:1`, `payload.failed:1`, empty `errors`.

Note: this test requires `node` on `PATH` (the browser harness runner), consistent with the existing `browser_*` integration tests in `crates/kali_cli/tests/`.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_cli/tests/browser_harness_failing_test_propagates_failure.rs
git commit -m "test(cli): browser harness propagates a throwing Kali.test as failure (throw-fallout Stage 0)"
```

---

## Task 3: Stage-0 gate — establish the true denominator and re-pin any tests that asserted the swallow

**Files:**
- Modify (as needed): any existing test that asserted the swallowed behavior (`success:true`/exit 0 while a registered test failed). Identified in Step 2.
- Create: `docs/superpowers/followups/throw-fallout-stage0-denominator.md` (the snapshot of the new true failing set)

**Interfaces:**
- Consumes: Tasks 1 and 2 landed.
- Produces: a recorded, triaged failing-set snapshot that later stages drain against.

- [ ] **Step 1: Stand up the `main` worktree gate (once for the whole program)**

Run:
```bash
git worktree add ../kali-main main
( cd ../kali-main && cargo test --workspace 2>&1 | tee /tmp/kali-main-tests.log | tail -5 )
```
Expected: the `main` worktree's `cargo test --workspace` reports **0 failures** (the clean baseline the branch is diffed against). If it is not 0, stop — the baseline assumption is wrong and must be investigated before proceeding.

- [ ] **Step 2: Find any existing tests that assert the swallow, and re-pin them**

Run:
```bash
grep -rln --include=*.rs -E 'success.*true|"exitCode".*0|status\(\).success\(\)' crates/kali_cli/tests | xargs grep -ln -E 'Kali\.test|testsFailed|failed' 2>/dev/null
```
For each hit, read the test. If it constructs a **failing/throwing** registered test yet asserts `success:true`/exit 0/`failed:0`, it encoded the swallow — re-pin it to assert the honest outcome (`success:false`, `exitCode:1`, `failed >= 1`), exactly as Task 2's reproducer does. A test that only exercises **passing** registered tests is correct as-is; leave it untouched. Record each re-pinned test in the commit message.

(If the grep returns no swallow-asserting tests, note that explicitly in Step 4's snapshot and skip the re-pin commit.)

- [ ] **Step 3: Run the full CI gate on the branch and diff against `main`**

Run:
```bash
cargo test --workspace 2>&1 | tee /tmp/kali-branch-tests.log | tail -5
```
Then extract the failing test names from both logs and diff:
```bash
grep -E '^test .* \.\.\. FAILED' /tmp/kali-branch-tests.log | sort > /tmp/branch-fail.txt
grep -E '^test .* \.\.\. FAILED' /tmp/kali-main-tests.log  | sort > /tmp/main-fail.txt
comm -23 /tmp/branch-fail.txt /tmp/main-fail.txt
```
Expected: the branch failing set is the previously-inventoried throw-fallout classes **plus** any browser self-check tests this stage newly un-masked. Every name in the `comm -23` output (branch-only failures) must be classifiable as a self-check/trap-propagation un-masking — the expected, honest consequence of Stage 0 per the Global Constraints. **Any branch-only failure that is NOT a self-check/trap un-masking is a real regression from Task 1** and must be fixed before Stage 0 closes (most likely a passing-test assertion that broke because the predicate is wrong — re-check Task 1's edits).

- [ ] **Step 4: Snapshot the true denominator**

Create `docs/superpowers/followups/throw-fallout-stage0-denominator.md` recording: the branch total failing count after Stage 0, the count that is browser self-check un-masking newly exposed by this stage, and the `comm -23` list bucketed by the design doc's classes (#1–#10). This is the denominator later stages drain against; it supersedes the raw 922 as the working target.

- [ ] **Step 5: Commit**

```bash
git add docs/superpowers/followups/throw-fallout-stage0-denominator.md
# plus any re-pinned tests from Step 2
git commit -m "docs+test(cli): Stage 0 true-denominator snapshot; re-pin swallow-asserting tests (throw-fallout)"
```

---

## Self-Review

**Spec coverage (against `2026-07-11-throw-fallout-design.md`, Stage 0):**
- "Any trap in a test callback is a failure — non-zero exit, `success:false`" → Task 1 (all three decision sites) + Task 2 (integration proof). ✅
- "We need the true denominator before counting green" / "this stage is expected to increase red" → Task 3 (gate diff vs `main`, denominator snapshot, honest-red acceptance). ✅
- Gate discipline (whole-workspace, vs `main` worktree) → Task 3 Steps 1 and 3. ✅
- Fix-never-flip / no-re-masking → Global Constraints; nothing here rejects a construct. ✅

**Placeholder scan:** No TBD/TODO. Every code step shows complete code; every run step shows the exact command and expected result. The only conditional is Task 3 Step 2's re-pin (explicitly handles the "no hits" case) and Task 2 Step 2 (explicitly optional, for a `main` worktree). ✅

**Type consistency:** `test_run_succeeded(diagnostics_empty: bool, failed: usize) -> bool` is defined in Task 1 Step 3 and called with `(diagnostics.is_empty(), failed)` at all three sites (Steps 5–7) and imported as `super::test_run_succeeded` in the unit tests (Step 1). `failed` is the existing `usize` accumulator declared at `cmd_test.rs:243`. Envelope field names (`success`, `exitCode`, `payload.total/passed/failed`, `errors`) match the existing envelope shape asserted by `browser_math_expm1_log1p_identities.rs`. ✅
