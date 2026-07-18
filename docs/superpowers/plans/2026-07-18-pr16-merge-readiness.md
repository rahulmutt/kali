# PR #16 Merge Readiness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `soundness-batch1-pra` fully CI-green by honestly re-pinning the ~712 residual red workspace tests (deny-lanes-first for silent miscompiles), then merge PR #16 to `main`.

**Architecture:** Family-batched adjudication waves over a frozen, committed red-set baseline. Each wave either pins an already-fail-closed family's diagnostic or lands a fail-closed deny lane (allowlist at the choke point) and then pins it. Every wave gates on a full-workspace enumeration diffed against the baseline (0 newly-red, monotone drain). Close-out: PR description rewrite, delta-only adversarial review, full CI green, merge commit.

**Tech Stack:** Rust workspace (`cargo test --workspace`), kali crates (`kali_types` + `kali_codegen` twins move in lockstep), `gh` CLI for PR/CI/merge.

**Spec:** `docs/superpowers/specs/2026-07-18-pr16-merge-readiness-design.md` — read it before executing any task.

## Global Constraints

- **No new runtime capability.** A family that looks trivially fixable is recorded in the adjudication table but re-pinned by default; fixing it requires explicit maintainer approval (scope exception).
- **Reject-don't-miscompile:** a silently-miscompiling family MUST get a fail-closed deny lane before its tests are pinned. Never pin a known-wrong value as expected output.
- **Allowlist at the choke point, never a denylist of shapes** (standing repo lesson; 4+ stages re-learned it).
- **`kali_types` ↔ `kali_codegen` twins change in the same commit** for any deny lane (admit/emit desync is a known fail-open class).
- **Never classify a red test from its name alone** — run it and read the actual payload (standing repo lesson).
- **CI-exact verification commands** (softer local variants don't count):
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace -- -D warnings`
  - `cargo test --workspace --verbose`
  - `cargo test -p kali_cli --test runtime_smoke --verbose`
  - `bash scripts/check-determinism.sh`
  - `cargo test -p kali_cli --test browser_cdp_smoke --verbose -- --ignored` (needs Chrome/chromium)
  - `cargo test -p kali_cli --test package_corpus --verbose`
- **Enumeration recipe** (the only accepted red-set measurement; used verbatim everywhere this plan says "enumerate"):

  ```bash
  rm -rf .kali-cache
  cargo build -p kali_cli
  cargo test --workspace --no-fail-fast 2>&1 \
    | grep -E '^test .* \.\.\. FAILED' \
    | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
    | sort -u > <outfile>
  ```

  Always run it TWICE (independent runs, cache cleared each time) and `diff` the two outputs; only a zero-drift pair is a valid measurement. (Parallel-run output interleaving can drop `FAILED` lines — the double run catches it; it cannot fabricate newly-red entries.)
- **Baseline file:** `docs/superpowers/followups/pr16-honest-red-baseline.txt` (committed in Task 2). Newly-red = `comm -13 baseline post`; drain = `comm -23 baseline post`.
- Scratch outputs go in the session scratchpad (`$SCRATCH` below); canonical artifacts go in `docs/superpowers/followups/` and are committed.

---

### Task 1: Mechanical CI fixes (fmt + clippy doc lints)

**Files:**
- Modify: `crates/kali_cli/tests/soundness_abort.rs` (fmt diffs at lines ~344, ~380, and any others `cargo fmt` finds)
- Modify: whichever files carry `doc list item without indentation` lints (enumerated in Step 3; CI log shows 8+)

**Interfaces:**
- Produces: a HEAD where `cargo fmt --all -- --check` and `cargo clippy --workspace -- -D warnings` both exit 0. Later tasks assume fmt/clippy stay clean and re-verify in their gates.

- [ ] **Step 1: Verify the current failures (evidence before fixes)**

Run: `cargo fmt --all -- --check; echo "exit:$?"`
Expected: diffs in `crates/kali_cli/tests/soundness_abort.rs`, nonzero exit.

Run: `cargo clippy --workspace -- -D warnings 2>&1 | grep -c "doc list item"`
Expected: a nonzero count (CI run 29643729332 showed 8+; local clippy 1.97 reports the same lint).

- [ ] **Step 2: Apply fmt**

Run: `cargo fmt --all && cargo fmt --all -- --check && echo FMT-CLEAN`
Expected: `FMT-CLEAN`.

- [ ] **Step 3: Enumerate and fix every doc-list lint**

Run: `cargo clippy --workspace -- -D warnings 2>&1 | grep -B2 "doc list item" | grep -E '^\s*-->' | sort -u`

For each reported `file:line`, indent the doc-comment continuation line(s) so the list item's wrapped lines align under the item text, e.g.:

```rust
// BEFORE (lint):
/// - a callback passed to a scheduling surface must be provably
/// non-capturing or fail closed
// AFTER:
/// - a callback passed to a scheduling surface must be provably
///   non-capturing or fail closed
```

- [ ] **Step 4: Verify clippy clean with the CI-exact command**

Run: `cargo clippy --workspace -- -D warnings && echo CLIPPY-CLEAN`
Expected: `CLIPPY-CLEAN` (no errors of any kind, not just the doc lint).

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "chore(ci): fmt + clippy doc-list-indentation fixes — CI fmt/clippy gates green [pr16-merge]"
```

---

### Task 2: Canonical honest-red baseline (enumerate + commit)

**Files:**
- Create: `docs/superpowers/followups/pr16-honest-red-baseline.txt`

**Interfaces:**
- Produces: the frozen baseline file every later gate diffs against, plus the ratified count N (expected ≈712; whatever the zero-drift pair says is canonical).

- [ ] **Step 1: Double enumeration**

Run the Global-Constraints enumeration recipe twice: outputs `$SCRATCH/pr16-base-run1.txt`, `$SCRATCH/pr16-base-run2.txt`.

- [ ] **Step 2: Zero-drift check**

Run: `diff $SCRATCH/pr16-base-run1.txt $SCRATCH/pr16-base-run2.txt && wc -l $SCRATCH/pr16-base-run1.txt`
Expected: empty diff; count ≈712. If the diff is non-empty, run a third enumeration and take the pair that matches; record the discrepancy in the Task 3 doc.

- [ ] **Step 3: Commit the canonical baseline**

```bash
cp $SCRATCH/pr16-base-run1.txt docs/superpowers/followups/pr16-honest-red-baseline.txt
git add docs/superpowers/followups/pr16-honest-red-baseline.txt
git commit -m "docs(soundness): canonical PR #16 honest-red baseline (N=<count>, double-enumerated zero-drift) [pr16-merge]"
```

---

### Task 3: Evidence-based triage → adjudication table

**Files:**
- Create: `docs/superpowers/followups/pr16-honest-repin-inventory.md`

**Interfaces:**
- Consumes: `pr16-honest-red-baseline.txt` (Task 2).
- Produces: the adjudication table — one row per family: `family-id | test-name pattern | count | representative test | observed evidence | behavior class (A: fail-closed already / B: silent miscompile / C: harness artifact) | action | flip-back condition`. Wave tasks (Task 5 template) are instantiated one-per-row from this table.

- [ ] **Step 1: First-cut clustering by name (hypothesis only, never final)**

Run: `sed -E 's/::.*//' docs/superpowers/followups/pr16-honest-red-baseline.txt | sort | uniq -c | sort -rn > $SCRATCH/pr16-clusters.txt` and also cluster by full-name prefixes. This produces candidate families ONLY — every family classification below requires observed evidence.

- [ ] **Step 2: Per-family representative evidence**

For each candidate family, run at least one member test and read the real payload:

```bash
cargo test --workspace --no-fail-fast <representative_test_name> -- --nocapture 2>&1 | tail -40
```

Where the payload is unclear, extract the fixture's source and run `kali run` / `node` on it directly to observe the divergence class. Classify:
- **A (fail-closed already):** clean diagnostic (E-code / node-shaped trap / exit≠0) — action: pin-reject.
- **B (silent miscompile):** wrong value, exit 0 — action: deny-lane-then-pin. Record the exact construct and its candidate choke point.
- **C (harness artifact):** the product behavior is fine; the test's harness predicate/envelope is wrong — action: fix the test-side predicate honestly (this is a test fix, not product capability; it is in scope).

Note: `await Promise.resolve(v)` is an ADMITTED lane (Stage 3 Task 4, `crates/kali_types/src/static_analysis/promise.rs`) — do not assume the async family's failure mode; measure it.

- [ ] **Step 3: Write the inventory doc**

`pr16-honest-repin-inventory.md` structure (this doc is the single canonical map; §8.6 of `stageD-triage.md` gets a pointer to it, not a copy):

```markdown
# PR #16 honest re-pin inventory (canonical)
## Adjudication table
| family | pattern | count | representative | evidence | class | action | flip-back |
## Per-family notes
### <family-id>
- Evidence transcript (what was run, what it printed)
- Aspiration: what node does
- Flip-back condition: which future stage/feature re-greens the family
## Coverage ledger
- Sum of family counts == baseline N (every baseline entry belongs to exactly one family; list any singletons explicitly)
```

The coverage ledger MUST account for all N baseline entries — no silent remainder.

- [ ] **Step 4: Commit**

```bash
git add docs/superpowers/followups/pr16-honest-repin-inventory.md
git commit -m "docs(soundness): PR #16 adjudication table — <K> families over N=<count>, evidence-classed [pr16-merge]"
```

---

### Task 4: Maintainer checkpoint — ratify wave order and exceptions

**Files:** none (conversation gate).

**Interfaces:**
- Consumes: the adjudication table (Task 3).
- Produces: ratified wave order (largest-first by default), ratified class-B deny-lane list, and explicit approval/denial of any recommended scope exceptions (trivial real fixes).

- [ ] **Step 1: Present the adjudication table to the maintainer** — family sizes, class split (A/B/C), proposed wave order, any scope-exception recommendations. Do not start Task 5 waves until the maintainer ratifies. This is the only blocking human gate before close-out.

---

### Task 5 (template — instantiate once per adjudication-table row, in ratified order): Family wave `<family-id>`

> At execution time, append one instantiated copy of this task per family to this plan file (Task 5.1, 5.2, …), filling in the family id, file list, and representative payloads from the adjudication table. An instantiation is complete only when its wave gate passes.

**Files:**
- Modify: the family's test files under `crates/kali_cli/tests/` (from the baseline entries' test-binary names)
- Class B only — Modify: `crates/kali_types/src/…` + `crates/kali_codegen/src/…` (twins, same commit); Create/Modify: a `crates/kali_cli/tests/soundness_*.rs` suite for fresh deny-lane pins
- Modify: `docs/superpowers/followups/pr16-honest-repin-inventory.md` (mark the family DONE with its gate numbers)

**Interfaces:**
- Consumes: baseline file, inventory doc, the family's ratified action.
- Produces: the family's baseline entries all green (honest pins); inventory row marked DONE; baseline file itself NEVER edited (it stays the frozen Task-2 artifact).

- [ ] **Step 1 (class B only): Write the failing deny-lane pin first (TDD)**

In the family's `soundness_*.rs` suite, write a fresh reproducer pin asserting the reject that does not exist yet, using that suite's existing helpers (`run_kali` / `run_kali_run_expect_error` — each suite defines its own; copy the file-local idiom):

```rust
/// Deny lane (PR #16 merge readiness, family `<family-id>`): kali has no
/// <construct>; a silent placeholder miscompile is closed to E5506.
/// Flip-back: see pr16-honest-repin-inventory.md#<family-id>.
#[test]
fn <construct>_fails_closed_e5506() {
    let stderr = run_kali_run_expect_error("<minimal reproducer source>");
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}
```

Run it, confirm it FAILS (the miscompile currently succeeds silently). Also record the same reproducer's `node` output in the pin's doc comment.

- [ ] **Step 2 (class B only): Land the deny lane**

At the construct's single choke point (resolver/emit site), admit the currently-supported forms by allowlist and default-deny the rest with the family's E-code; `kali_types` and `kali_codegen` twins in the same commit. Re-run the Step-1 pin: PASS. Probe 2–3 adjacent admitted shapes to prove no over-reject (each admitted shape's test stays green).

- [ ] **Step 3: Re-pin the family's baseline tests**

For every family member in the baseline: keep the test name, replace node-parity assertions with honest assertions of the current behavior, using the file's existing helpers, and add the pointer comment:

```rust
// Honest re-pin (PR #16 merge readiness, family `<family-id>`): kali does
// not implement <construct>; behavior is fail-closed <E-code/trap>.
// Aspiration + flip-back: pr16-honest-repin-inventory.md#<family-id>.
```

Class A: assert the observed diagnostic (E-code in stderr / `!out.status.success()` / node-shaped trap text). Class B: assert the Step-2 reject. Class C: fix the harness predicate so the envelope reports honestly, then assert the honest outcome.

- [ ] **Step 4: Wave gate (full enumeration vs frozen baseline)**

Run the enumeration recipe → `$SCRATCH/wave-<family-id>-post.txt`, then:

```bash
comm -13 docs/superpowers/followups/pr16-honest-red-baseline.txt $SCRATCH/wave-<family-id>-post.txt   # newly-red
comm -23 docs/superpowers/followups/pr16-honest-red-baseline.txt $SCRATCH/wave-<family-id>-post.txt | wc -l  # cumulative drain
```

Expected: newly-red EMPTY; drain ≥ (sum of completed families' counts). Any newly-red entry blocks the wave: adjudicate it (a deny lane un-masking a fake-green test feeds it into the inventory as a new/extended family — that is expected behavior, not an exemption). Double-enumerate on any surprise. Spot-check goldens: `cargo test -p kali_cli --test runtime_smoke <one CLBG golden test> -- --nocapture` stays green.

- [ ] **Step 5: Update inventory + commit**

```bash
git add -A
git commit -m "test(soundness): honest re-pin wave <family-id> (<count> pins<, +deny lane E-code if class B>) — gate 0-newly-red, drain <cumulative> [pr16-merge]"
```

---

### Task 6: PR #16 description rewrite

**Files:** none (GitHub PR body via `gh`).

**Interfaces:**
- Consumes: all stage close-out docs (`stageD-triage.md` §8.6, stage memories, spec).
- Produces: a PR body covering the full branch story.

- [ ] **Step 1: Draft the body** in `$SCRATCH/pr16-body.md` with sections: (1) the original 8 soundness closures; (2) throw-fallout Stages 0–5 (the 922-fake-green un-mask and the drain); (3) block-arrows Stages AB/C/D (closures, un-flatten, event surface); (4) structuredClone P2; (5) Abort P3; (6) this merge-readiness effort — the honest-red → honest-pin story, the inventory doc as the canonical aspiration map; (7) verification (final gate numbers, 6 CLBG goldens byte-for-byte, CI green); (8) post-merge roadmap (P4 URL, P5 TextEncoder, async, §8.6 residuals).

- [ ] **Step 2: Update the PR**

Run: `gh pr edit 16 --body-file $SCRATCH/pr16-body.md && gh pr view 16 | head -30`
Expected: the new body renders; title stays or is updated to reflect the full scope (e.g. "Soundness Batch 1 PR-A + throw-fallout drain + block-arrows/P2/P3 + honest re-pin of the residual red set").

---

### Task 7: Delta adversarial review

**Files:** none produced beyond fix commits (if findings).

**Interfaces:**
- Consumes: the merge-readiness delta — every commit from the Task-1 commit through HEAD (deny lanes, re-pin waves, inventory doc).
- Produces: findings fixed and re-gated, or a clean review verdict recorded in the inventory doc's close-out section.

- [ ] **Step 1: Adversarial review of the delta only** (standing whole-stage-review discipline; per repo history this is where CRITICALs surface). Focus list: (a) any re-pin that asserts a wrong VALUE rather than a reject/trap (would bless a miscompile); (b) any class-B deny lane with an admit/emit twin desync; (c) allowlist completeness at each deny choke point (probe sibling shapes); (d) inventory coverage ledger still sums to N; (e) pins that would silently keep passing if the deny lane were deleted (assert the diagnostic, not just failure).

- [ ] **Step 2: Fix findings, re-run the wave gate for any touched family, commit** with `[pr16-merge]` trailer; record the review verdict + rounds in `pr16-honest-repin-inventory.md`.

---

### Task 8: Final gate, CI, merge

**Files:** none (verification + merge mechanics).

**Interfaces:**
- Consumes: everything above.
- Produces: PR #16 merged to `main`; memory updated.

- [ ] **Step 1: Final local gate (all CI-exact commands)**

```bash
cargo fmt --all -- --check && cargo clippy --workspace -- -D warnings
# double enumeration — BOTH runs must produce EMPTY failure sets:
<enumeration recipe> → $SCRATCH/final-run1.txt and final-run2.txt; both zero lines
cargo test -p kali_cli --test runtime_smoke --verbose          # includes the 6 CLBG goldens
bash scripts/check-determinism.sh
cargo test -p kali_cli --test package_corpus --verbose
```

Expected: every command exits 0; both enumeration files are empty.

- [ ] **Step 2: Push and watch CI**

```bash
git push origin soundness-batch1-pra
gh pr checks 16 --watch
```

Expected: all jobs green, including browser-cdp-smoke (not runnable headlessly here unless chromium is present — CI is the gate of record for it; if it reds, debug via `cargo test -p kali_cli --test browser_cdp_smoke --verbose -- --ignored` with chromium installed and fix before proceeding).

- [ ] **Step 3: Mark ready and merge**

```bash
gh pr ready 16
gh pr merge 16 --merge
git fetch origin main && git log --oneline origin/main -3
```

Expected: merge commit on `main` (matching PRs #4–#15 convention), branch head as its second parent.

- [ ] **Step 4: Post-merge bookkeeping** — update the assistant memory ledger (PR #16 merged, honest-red baseline retired, inventory doc is the live aspiration map; P4/P5/async next on fresh branches) and mark the stale "PR #16 held draft" notes superseded.
