# PR #16 Merge Readiness (rev2, Strategy B) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `soundness-batch1-pra` fully CI-green and merge PR #16 by adjudicating all 708 honest-red workspace tests via Strategy B — re-pin class-A (fail-closed) tests to assert their diagnostic, deny-lane the *cheap* class-B families, `#[ignore]`+issue the architectural class-B families, and fix class-C harness predicates — without blessing any silent miscompile into `main`.

**Architecture:** A triage-then-waves meta-plan over a frozen, committed 708-test baseline. Task 2 classifies every red test by *observed payload* (never by name) into A/B/C on the current Group-1-fixed binary and writes a canonical inventory. Each later wave (Task 4 template, instantiated one-per-family after the Task 3 checkpoint) applies that family's ratified action and gates on a full re-enumeration diffed against the baseline: **0 newly-red, monotone drain**. Close-out rewrites the PR body, adversarially reviews the delta, gates on full CI green, and merges.

**Tech Stack:** Rust workspace (`cargo test --workspace`), kali crates (`kali_types`↔`kali_codegen` twins move in lockstep), `gh` CLI for PR/CI/issues/merge, Node v26 as the parity oracle.

**Spec:** `docs/superpowers/specs/2026-07-20-pr16-merge-readiness-rev2-design.md` — read it before executing any task.

## Global Constraints

- **No new runtime capability.** The only code changes admitted are (a) class-C harness-predicate fixes and (b) cheap class-B deny lanes (register Group 2). Architectural class-B miscompiles are deferred post-merge (`#[ignore]`+issue). Fixing one anyway requires explicit maintainer approval (scope exception).
- **Honest-framing invariant.** All 708 tests are *fake-green on `main`* — they pass there only because kali's self-checks were silent no-ops; the miscompiles already ship on `main`. `#[ignore = "R-NN, tracked #issue"]` therefore does NOT degrade `main`; it makes the suite honest. Never re-pin a class-B test to assert a *wrong value* — that WOULD write a falsehood into `main` (reject-don't-bless).
- **Never classify a red test from its name alone** — run it and read the actual payload (standing repo lesson; the whole reason rev1 was voided).
- **Allowlist at the choke point, never a denylist of shapes** for every deny lane (standing repo lesson; 4+ stages re-learned it).
- **`kali_types` ↔ `kali_codegen` twins change in the same commit** for any deny lane (admit/emit desync is a known fail-open class).
- **CI-exact verification commands** (softer local variants do not count):
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace -- -D warnings`
  - `cargo test --workspace --verbose`
  - `cargo test -p kali_cli --test runtime_smoke --verbose`
  - `bash scripts/check-determinism.sh`
  - `cargo test -p kali_cli --test browser_cdp_smoke --verbose -- --ignored` (needs Chrome/chromium)
  - `cargo test -p kali_cli --test package_corpus --verbose`
- **Enumeration recipe** (the ONLY accepted red-set measurement; used verbatim everywhere this plan says "enumerate"):

  ```bash
  rm -rf .kali-cache
  cargo build -p kali_cli
  cargo test --workspace --no-fail-fast 2>&1 \
    | grep -E '^test .* \.\.\. FAILED' \
    | sed -E 's/^test (.*) \.\.\. FAILED$/\1/' \
    | sort -u > <outfile>
  ```

  Always run it TWICE (independent runs, cache cleared each time) and `diff` the two outputs; only a zero-drift pair is a valid measurement. (Parallel-run output interleaving can drop `FAILED` lines — the double run catches it; it cannot fabricate newly-red entries.)
- **Baseline file:** `docs/superpowers/followups/pr16-honest-red-baseline.txt` (refrozen at N=708 in Task 1). Newly-red = `comm -13 baseline post`; drain = `comm -23 baseline post`.
- **Verbatim test idioms** (copied from live suites — use these exact forms):
  - Class-A re-pin (fail-closed diagnostic): `assert!(stderr.contains("E5506"), "stderr: {stderr}");` or, when only the exit matters, `assert!(!output.status.success(), "must fail closed: {output:?}");`
  - `#[ignore]` (architectural class-B): a doc comment + `#[ignore = "R-NN: <one-line>; tracked <issue-url>; see pr16-honest-repin-inventory.md#<family>"]` immediately above the `#[test]` line (mirrors `crates/kali_cli/tests/soundness_block_arrows.rs:265-268`).
  - New deny-lane pin suite: mirror `crates/kali_cli/tests/soundness_array_spread.rs` (module doc block naming the R-NN, `run_source`/`assert_fails_closed` local helpers, every expected value captured from node v26).
- Scratch outputs go in the session scratchpad (`$SCRATCH`); canonical artifacts go in `docs/superpowers/followups/` and are committed.

---

### Task 1: Refreeze the N=708 baseline + confirm mechanical CI clean

**Files:**
- Overwrite: `docs/superpowers/followups/pr16-honest-red-baseline.txt` (was 694; refreeze at 708)

**Interfaces:**
- Produces: the frozen 708-line baseline every later wave gate diffs against. `fmt`/`clippy` confirmed green so later gates can assume they stay green.

- [ ] **Step 1: Confirm fmt + clippy are clean (CI-exact)**

Run: `cargo fmt --all -- --check && echo FMT-CLEAN`
Expected: `FMT-CLEAN` (confirmed clean at HEAD `b379f7ee6` during planning; re-verify).

Run: `cargo clippy --workspace -- -D warnings && echo CLIPPY-CLEAN`
Expected: `CLIPPY-CLEAN`. If either reds, fix minimally and commit `chore(ci): …` before proceeding.

- [ ] **Step 2: Double-enumerate the current red set**

Run the Global-Constraints enumeration recipe twice → `$SCRATCH/base1.txt`, `$SCRATCH/base2.txt`.

Run: `diff $SCRATCH/base1.txt $SCRATCH/base2.txt && wc -l $SCRATCH/base1.txt`
Expected: empty diff; count `708` (±drift from any Step-1 fix; whatever the zero-drift pair says is canonical). If the diff is non-empty, run a third enumeration and take the matching pair.

- [ ] **Step 3: Commit the refrozen baseline**

```bash
cp $SCRATCH/base1.txt docs/superpowers/followups/pr16-honest-red-baseline.txt
git add docs/superpowers/followups/pr16-honest-red-baseline.txt
git commit -m "docs(soundness): refreeze PR #16 honest-red baseline (N=708, was 694; Group-1/2 fixes re-drained) — double-enumerated zero-drift [pr16-merge]"
```

---

### Task 2: Automated A/B/C triage → inventory revision 2

**Files:**
- Create: `$SCRATCH/pr16-triage.sh` (throwaway classifier script)
- Overwrite: `docs/superpowers/followups/pr16-honest-repin-inventory.md` (revision 2, evidence re-gathered on the fixed binary)

**Interfaces:**
- Consumes: `pr16-honest-red-baseline.txt` (Task 1).
- Produces: the revision-2 adjudication table — one row per family: `family | pattern | count | representative | evidence (fixed-binary transcript) | class (A/B/C) | action (re-pin / deny-lane+pin / ignore+issue / predicate-fix) | flip-back condition | issue#`. Coverage ledger sums to 708. Task 4 instantiations are drawn one-per-row from this table.

- [ ] **Step 1: Capture one full failing run and write the block classifier**

Baseline lines are bare `module::fn` (e.g. `run::json_run_supports_x`) with NO binary name — so per-test re-runs can't be reconstructed from a line, and dual-binary names would misroute. Instead capture ONE full run and classify each test's `---- <name> stdout ----` block (the robust parser; cargo emits one block per failing test with its captured stdout/stderr, which includes the panic message and the kali output the failing `assert!` embedded). **Do NOT pass `--nocapture`** — it streams output inline and produces ZERO replay blocks (the classifier then matches nothing):

```bash
rm -rf .kali-cache
cargo test --workspace --no-fail-fast 2>&1 \
  | sed 's/\x1b\[[0-9;]*m//g' > "$SCRATCH/pr16-fullrun.txt"
# sanity: block count must be > 0 (0 means --nocapture leaked in)
grep -c '^---- .* stdout ----' "$SCRATCH/pr16-fullrun.txt"
```

Then `$SCRATCH/pr16-triage.sh` splits that capture into per-test blocks and classifies each:

```bash
#!/usr/bin/env bash
# Classify each failing test's captured block. Output TSV: name<TAB>class<TAB>evidence
set -u
awk '
  /^---- .* stdout ----$/ { if (name) emit(); name=$2; blk=""; next }
  name { blk = blk $0 "\n" }
  END { if (name) emit() }
  function emit() {
    cls="?"; ev=""
    if (blk ~ /error\[E[0-9][0-9][0-9][0-9]\]/) { cls="A"; match(blk, /error\[E[0-9]+\][^\n]*/); ev=substr(blk,RSTART,RLENGTH) }
    else if (blk ~ /RuntimeError: unreachable|error\[E4000\]|Uncaught /) { cls="A"; match(blk,/(RuntimeError: unreachable|Uncaught [^\n]*)/); ev=substr(blk,RSTART,RLENGTH) }
    else if (blk ~ /assertion .*failed|left == right|left: |stdout: /) { cls="B"; match(blk,/(left: [^\n]*|stdout: [^\n]*)/); ev=substr(blk,RSTART,RLENGTH) }
    printf "%s\t%s\t%s\n", name, cls, ev
  }
' "$SCRATCH/pr16-fullrun.txt" | sort -u > "$SCRATCH/pr16-triage.tsv"
```

Note: `E4000`/`RuntimeError: unreachable` is a *loud trap* — class A (fails closed, never a wrong value) — but a program whose OWN self-check printed a wrong value and then trapped is class B; the `stdout:`/`left:` branch catches most, and every `?` plus a sample of each class is manually audited in Step 3.

- [ ] **Step 2: Run the classifier and reconcile the count**

Run: `bash $SCRATCH/pr16-triage.sh && cut -f2 $SCRATCH/pr16-triage.tsv | sort | uniq -c && wc -l $SCRATCH/pr16-triage.tsv`
Expected: a split across `A`/`B`/`?`; line count within a few of 708 (a full `--nocapture` run may interleave — cross-check `cut -f1 $SCRATCH/pr16-triage.tsv | sort -u | comm -13 - docs/superpowers/followups/pr16-honest-red-baseline.txt` to list any baseline names the capture missed, and re-capture if the miss set is non-trivial). The `?` rows and any surprising `B` rows are the manual-audit set.

- [ ] **Step 3: Manually audit every `?` and a sample of each cluster**

For each `?` row and ≥1 representative per name-cluster, extract the fixture source and run `kali run <fixture>` and `node <fixture>` directly, comparing outputs. Confirm the class:
- **A** — kali emits a diagnostic / traps loud / exits nonzero without producing a wrong value.
- **B** — kali exits 0 (or its self-check trap fires only *after* printing a wrong value) with a value diverging from node.
- **C** — kali's program output matches node; only the test's harness envelope/predicate diverges.

Record the transcript (command run, both outputs) for the inventory.

- [ ] **Step 4: Cluster into families and write inventory revision 2**

Cluster by root-cause family (name-prefix is a hypothesis; the audited evidence is authoritative). Write `docs/superpowers/followups/pr16-honest-repin-inventory.md`:

```markdown
# PR #16 honest re-pin inventory — REVISION 2 (Strategy B, N=708)
Evidence gathered on HEAD <sha> against target/debug/kali (Group-1-fixed), vs Node 26.5.0.

## Adjudication table
| family | pattern | count | representative | evidence | class | action | flip-back | issue |
|--------|---------|-------|----------------|----------|-------|--------|-----------|-------|
...

## Per-family notes
### <family-id>
- Evidence transcript
- Aspiration (what node does)
- Class-B only: cheap (Group-2 deny lane) or architectural (ignore+issue)? Choke point if cheap.
- Flip-back condition

## Coverage ledger
- Sum of family counts == 708 (list singletons explicitly; no silent remainder)
```

For each class-B family, tag it **cheap** (register Group 2: R-11, R-24, R-09, R-16, R-26, R-27, R-22 — bounded, known-shape) or **architectural** (register Group 3/4: R-02/R-05, R-06, R-08/R-21, R-10, R-14, R-23).

- [ ] **Step 5: Commit**

```bash
git add docs/superpowers/followups/pr16-honest-repin-inventory.md
git commit -m "docs(soundness): PR #16 inventory rev2 — A/B/C re-derived on the Group-1-fixed binary, <K> families over N=708 [pr16-merge]"
```

---

### Task 3: Maintainer checkpoint — ratify wave order, deny-lane list, issue set

**Files:** none (conversation gate).

**Interfaces:**
- Consumes: the revision-2 adjudication table (Task 2).
- Produces: ratified wave order (largest family first), the ratified **cheap class-B deny-lane list**, and the list of **class-B families to `#[ignore]`+issue**.

- [ ] **Step 1: Present the rev2 table to the maintainer** — family sizes, A/B/C split, which class-B families are cheap (deny-lane) vs architectural (ignore+issue), proposed wave order, and the per-family issue list to be filed in Task 4. Do not start Task 4 waves until ratified. This is the only blocking human gate before close-out.

---

### Task 4 (TEMPLATE — instantiate once per adjudication-table row, in ratified order): Family wave `<family-id>`

> At execution time, append one instantiated copy per family (Task 4.1, 4.2, …), filling in the family id, member test list, class, and action from the inventory. An instantiation is complete only when its wave gate (Step 6) passes. Run the four action branches (Steps 2a–2d) that apply to the family's class; every family runs Steps 1, 5, 6.

**Files:**
- Modify: the family's member test files under `crates/kali_cli/tests/` (located from baseline entries matching the row's pattern; watch dual-binary names that appear in both `run.rs`/`test.rs` and their `json_` twins)
- Class-B cheap only — Create: a `crates/kali_cli/tests/soundness_<family>.rs` deny-lane pin suite; Modify: `crates/kali_types/src/…` + `crates/kali_codegen/src/…` (twins, same commit)
- Class-B architectural only — GitHub issue via `gh issue create`
- Modify: `docs/superpowers/followups/pr16-honest-repin-inventory.md` (mark the family DONE with gate numbers + issue#)

**Interfaces:**
- Consumes: baseline file, inventory row, ratified action.
- Produces: the family's baseline entries all green (via re-pin / deny-lane+pin / ignore / predicate-fix); inventory row DONE; the baseline file itself NEVER edited (frozen Task-1 artifact).

- [ ] **Step 1: Locate the family's member tests**

Run: `grep -f <(grep '<family-pattern>' docs/superpowers/followups/pr16-honest-red-baseline.txt | sed -E 's/.* //') -rl crates/kali_cli/tests/`
Confirm the count matches the inventory row. Note whether members are inline bodies or shared-helper wrappers (`assert_*` calls) — `#[ignore]` always goes on the `#[test]` wrapper; re-pins prefer per-test assertions over editing a shared helper that green tests also call.

- [ ] **Step 2a (class A — re-pin the diagnostic):**

For each member, replace the node-parity assertion with an assertion of the observed fail-closed behavior, and add a pointer comment. Verbatim idiom:

```rust
// Honest re-pin (PR #16 merge readiness, family `<family-id>`): kali fails
// closed on <construct>; behavior is <E-code / node-shaped trap>.
// Aspiration + flip-back: pr16-honest-repin-inventory.md#<family-id>.
let stderr = String::from_utf8_lossy(&output.stderr);
assert!(stderr.contains("E5506"), "stderr: {stderr}");
```

When only the nonzero exit is guaranteed (trap without a stable E-code), use:

```rust
assert!(!output.status.success(), "must fail closed: {output:?}");
```

- [ ] **Step 2b (class B cheap — TDD the deny lane, then pin):**

First write the failing reject pin in a new `crates/kali_cli/tests/soundness_<family>.rs` (mirror `soundness_array_spread.rs`):

```rust
//! Deny lane (PR #16 merge readiness, family `<family-id>`, register R-NN):
//! kali has no <construct> lowering; the silent placeholder miscompile is
//! closed to E5506. node v26 output recorded per pin. Flip-back:
//! pr16-honest-repin-inventory.md#<family-id>.
#[test]
fn <construct>_fails_closed_e5506() {
    let out = run_source("<minimal reproducer>");
    assert!(!out.status.success(), "{out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("E5506"), "stderr: {stderr}");
}
```

Run it, confirm it FAILS (the miscompile currently succeeds silently). Then land the deny lane at the construct's single choke point (allowlist admitted forms, default-deny the rest with `E5506`; `kali_types` + `kali_codegen` twins in the same commit). Re-run: PASS. Probe 2–3 adjacent admitted shapes to prove no over-reject. Then re-pin the family's baseline members per Step 2a (they now fail closed).

- [ ] **Step 2c (class B architectural — file issue, then ignore):**

```bash
gh issue create --title "kali: <family> silently miscompiles (register R-NN)" \
  --body "Deferred from PR #16 merge readiness. <one-paragraph: what node does, what kali does, register R-NN link, flip-back condition>." \
  --label soundness
```

Capture the issue URL. For each member, add above its `#[test]`:

```rust
/// Deferred class-B (PR #16 merge readiness): kali silently miscompiles
/// <construct> (register R-NN). Not re-pinned — asserting kali's wrong value
/// would bless a falsehood; asserting the reject would be false (kali does not
/// reject). Ignored until the tracked fix lands.
#[ignore = "R-NN: <one-line>; tracked <issue-url>; see pr16-honest-repin-inventory.md#<family-id>"]
```

Leave the test body's assertions intact (they document the aspiration and re-activate on `-- --ignored`).

- [ ] **Step 2d (class C — fix the harness predicate):**

Correct the test-side envelope/predicate so it reports the honest outcome (this is a test fix, not a capability change), then assert the honest outcome. Add a `// Harness-predicate fix (PR #16): …` comment.

- [ ] **Step 5: Local family check**

Run the family's binary for the touched names, e.g. `cargo test -p kali_cli --test runtime_smoke -- <family-pattern>`
Expected: all members green (passing pins / ignored / predicate-fixed). For a class-B-cheap wave, also run the new `soundness_<family>` suite: all green.

- [ ] **Step 6: Wave gate (full enumeration vs frozen baseline) + commit**

Run the enumeration recipe → `$SCRATCH/wave-<family>-post.txt`, then:

```bash
comm -13 docs/superpowers/followups/pr16-honest-red-baseline.txt $SCRATCH/wave-<family>-post.txt          # newly-red — MUST be empty
comm -23 docs/superpowers/followups/pr16-honest-red-baseline.txt $SCRATCH/wave-<family>-post.txt | wc -l  # cumulative drain
```

Expected: newly-red EMPTY; drain ≥ sum of completed families' counts. Any newly-red blocks the wave — a deny lane that un-masks a fake-green test feeds that test into the inventory as a new/extended family (expected, not an exemption). Double-enumerate on any surprise. Spot-check a CLBG golden: `cargo test -p kali_cli --test runtime_smoke <one golden> -- --nocapture` stays green. Then:

```bash
git add -A
git commit -m "test(soundness): PR #16 wave <family-id> (<n> re-pin/ignore<, +deny lane E5506 if cheap-B>) — gate 0-newly-red, drain <cum> [pr16-merge]"
```

---

### Task 5: PR #16 description rewrite

**Files:** none (GitHub PR body via `gh`).

**Interfaces:**
- Consumes: all stage close-out docs, the rev2 inventory.
- Produces: a PR body covering the whole branch.

- [ ] **Step 1: Draft the body** in `$SCRATCH/pr16-body.md` with sections: (1) original 8 soundness closures; (2) throw-fallout Stages 0–5 (the 922-fake-green un-mask + drain); (3) block-arrows AB/C/D (closures, un-flatten, event surface); (4) structuredClone P2; (5) Abort P3; (6) G6 unimplemented-builtin fail-closed; (7) this rev2 merge-readiness effort — the honest-red → re-pin/ignore story, the inventory as the canonical aspiration map, one tracking issue per deferred class-B family; (8) verification (final gate numbers, 6 CLBG goldens byte-for-byte, CI green); (9) post-merge roadmap (Group-3 guard-holes, Group-4 architectural, P4 URL, P5 TextEncoder).

- [ ] **Step 2: Update the PR**

Run: `gh pr edit 16 --title "Soundness Batch 1 PR-A + throw-fallout + block-arrows/P2/P3/G6 + honest re-pin of the residual red set" --body-file $SCRATCH/pr16-body.md && gh pr view 16 | head -30`
Expected: the new body renders.

---

### Task 6: Delta adversarial review

**Files:** none beyond fix commits (if findings).

**Interfaces:**
- Consumes: the merge-readiness delta — every commit from Task 1 through HEAD.
- Produces: findings fixed and re-gated, or a clean verdict recorded in the inventory close-out.

- [ ] **Step 1: Adversarial review of the delta** (standing whole-stage discipline — per repo history this is where CRITICALs surface). Focus: (a) any re-pin asserting a wrong VALUE rather than a reject/trap (blesses a miscompile); (b) any `#[ignore]` missing its issue URL or R-NN; (c) class-B-cheap deny-lane admit/emit twin desync; (d) allowlist completeness at each deny choke (probe sibling shapes); (e) inventory coverage ledger still sums to 708; (f) re-pins that would still pass if the deny lane were deleted (assert the diagnostic, not just failure); (g) any class-C predicate fix that hides a real divergence.

- [ ] **Step 2: Fix findings, re-run the touched family's wave gate, commit** with `[pr16-merge]`; record the verdict + rounds in `pr16-honest-repin-inventory.md`.

---

### Task 7: Final gate, CI, merge

**Files:** none (verification + merge mechanics).

- [ ] **Step 1: Final local gate (all CI-exact commands)**

```bash
cargo fmt --all -- --check && cargo clippy --workspace -- -D warnings
# double enumeration — BOTH runs must be EMPTY:
<enumeration recipe> → $SCRATCH/final1.txt and final2.txt; both 0 lines; diff empty
cargo test -p kali_cli --test runtime_smoke --verbose        # includes the 6 CLBG goldens
bash scripts/check-determinism.sh
cargo test -p kali_cli --test package_corpus --verbose
```

Expected: every command exits 0; both enumeration files empty.

- [ ] **Step 2: Push and watch CI**

```bash
git push origin soundness-batch1-pra
gh pr checks 16 --watch
```

Expected: all jobs green, including browser-cdp-smoke (CI is the gate of record; if it reds, debug with `cargo test -p kali_cli --test browser_cdp_smoke --verbose -- --ignored` under chromium and fix before proceeding).

- [ ] **Step 3: Mark ready and merge**

```bash
gh pr ready 16
gh pr merge 16 --merge
git fetch origin main && git log --oneline origin/main -3
```

Expected: merge commit on `main` (matching PRs #4–#15), branch head as its second parent.

- [ ] **Step 4: Post-merge bookkeeping** — update the assistant memory ledger (PR #16 merged; honest-red baseline retired; inventory rev2 is the live aspiration map; deferred class-B issues are the post-merge queue; P4/P5/Group-3/Group-4 next on fresh branches) and mark the stale "PR #16 held draft" notes superseded.

---

## Task 4 instantiations (appended after the Task 3 checkpoint)

> Seed hypothesis from the rev1 inventory's 13 families (object-enum ~319, promise ~128, string-iter ~94, mapset ~33, object-hasown ~28, for-await ~24, microtask ~22, reflect ~16, corpus ~15, deno ~4, bool-bundle ~4, crypto-bundle ~4, await-wrapped ~3) — but Task 2 RE-DERIVES class and count on the fixed binary; several rev1 class-B calls are expected to flip to A (the sweep found 310/694 are class A). Instantiate from the rev2 table, largest family first. Do NOT pre-write these before Task 2/3.
