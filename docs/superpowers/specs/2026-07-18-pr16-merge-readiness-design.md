# PR #16 merge readiness — honest re-pin of the residual red set

**Date:** 2026-07-18
**Branch:** `soundness-batch1-pra` (PR #16, draft since 2026-07-10)
**Decision owner:** maintainer (choices ratified in the 2026-07-18 brainstorm)

## 1. Problem

PR #16 is 241 commits ahead of `main` (0 behind, head current) and cannot
merge because:

1. **712 honest-red workspace tests.** CI's `cargo test --workspace` fails on
   the branch's deliberately carried honest-red baseline — tests that are
   fake-green on `main` (their self-check `throw`s were silent no-ops there)
   and honestly red here now that `throw` is sound. The Stage-D triage doc
   marks this baseline "never merge to main". Known composition (from the
   throw-fallout triage, to be re-confirmed): the async/await/Promise
   placeholder lane (~200), browser-harness variants, web-baseline corpus
   items waiting on Stage P4 (`URL`/`URLSearchParams`) and P5
   (`TextEncoder`/`TextDecoder`), and smaller families. No current
   enumeration exists — the 712 list lived in a dead session scratchpad.
2. **CI fmt is red at HEAD** — real diffs in
   `crates/kali_cli/tests/soundness_abort.rs` (the P3 close-out's "fmt clean"
   claim does not hold at HEAD).
3. **CI clippy is red** — "doc list item without indentation" lints, escalated
   to errors by CI's `-D warnings` (they appear as warnings locally).
4. **The PR description is stale** — it describes only the original 8
   soundness closures; the branch has since absorbed throw-fallout Stages
   0–5, block-arrows Stages AB/C/D, structuredClone (P2), and Abort (P3).

## 2. Ratified decisions

- **Adjudication policy: mass honest re-pin.** No new feature work in this
  effort. Every remaining red test is re-pinned to assert kali's current
  honest behavior, with a canonical inventory so aspiration is not lost.
  P4, P5, async, and all §8.6 residuals continue post-merge on new branches.
- **Silently-miscompiling families get a deny lane first.** Where the current
  behavior is a silent wrong value (e.g. `await Promise.resolve(7)` → 0),
  land a fail-closed reject (allowlist at the choke point, never a shape
  denylist) and then pin the reject. Merged `main` must not ship a known
  silent miscompile that tests bless (reject-don't-miscompile invariant).
- **Merge review = delta only.** The 10+ per-stage adversarial reviews stand;
  the merge gate adversarially reviews only the merge-readiness delta (deny
  lanes + re-pin waves + inventory), plus mechanical full-gate verification.
- **Execution shape: family-batched waves** (one root-cause family per wave,
  each wave fully gated), not a single mega-sweep.

## 3. Definition of done

PR #16 merges to `main` via the usual merge commit, with full CI green on the
PR head:

- `cargo test --workspace` — **0 failures**, double-enumerated with zero
  drift (`--no-fail-fast`, cache cleared, two independent runs, identical
  sorted failure sets — here, empty).
- `cargo fmt --all --check` clean and
  `cargo clippy --workspace --all-targets -- -D warnings` clean (CI's exact
  commands).
- The 6 CLBG goldens (nbody, fannkuch, spectral-norm, mandelbrot,
  binary-trees, fasta) byte-for-byte.
- The remaining CI jobs green: browser-cdp-smoke, determinism (both OSes),
  package-corpus, build matrix.
- PR description rewritten to cover the whole branch.
- Memory/ledger updated post-merge.

**Non-goals:** any new runtime capability. A family that turns out to be a
trivial product fix is *recorded* as such in the adjudication table but still
re-pinned by default; fixing it requires explicit maintainer approval as a
scope exception.

## 4. Plan of record

### Step 0 — mechanical CI fixes

- `cargo fmt --all`; commit.
- Fix all "doc list item without indentation" doc comments; verify with
  `cargo clippy --workspace --all-targets -- -D warnings`; commit.
- Audit `.github/workflows/ci.yml` job-by-job and list every command CI runs,
  so no other gate is a late surprise (e.g. `runtime_smoke`-only job,
  `package_corpus`, `browser_cdp_smoke -- --ignored`).

### Step 1 — fresh enumeration + evidence-based triage

- Fresh build, then two independent full enumerations:
  `rm -rf .kali-cache && cargo test --workspace --no-fail-fast`, extracting
  `^test .* \.\.\. FAILED` names, sorted unique. Zero-drift check
  (`diff` empty). **Commit the canonical red-set list** to
  `docs/superpowers/followups/pr16-honest-red-baseline.txt` — it must not
  live only in a scratchpad again.
- Bucket every failure by root-cause family with **observed evidence**: run at
  least one representative test per family with `--nocapture`, read the
  actual failure payload (diagnostic code, trap, wrong value, harness
  envelope). Never classify from the test name alone.
- Output: an adjudication table in
  `docs/superpowers/followups/pr16-honest-repin-inventory.md` —
  family → representative evidence → current behavior class
  (fail-closed already / silent miscompile / harness artifact) → action
  (pin-reject / deny-lane-then-pin / harness fix) → flip-back condition.

### Step 2 — per-family adjudication waves

Ordered by family size (largest first). Per wave:

1. If the family already fails closed → re-pin the batch to assert the
   current diagnostic shape (E5506 / node-shaped trap / exit≠0), each pin
   commented with a pointer to the inventory entry.
2. If the family silently miscompiles → land the deny lane at the single
   choke point first (allowlist admitted forms; default deny), prove the
   reject with fresh reproducers, then pin.
3. Wave gate: full-workspace enumeration diffed against the frozen Step-1
   baseline — **0 newly-red** (`comm -13`), **drain monotone**
   (`comm -23` only shrinks the red set), goldens spot-check. A deny lane
   that newly-reds a currently-green (fake-passing) test feeds that test
   into the same adjudication, not an exemption.

The inventory doc is the single canonical map: every re-pinned test →
family → aspiration (what node does) → flip-back condition (which future
stage/feature re-greens it). §8.6 of the Stage-D triage doc gets a pointer,
not a copy.

### Step 3 — close-out and merge

- Rewrite the PR #16 description: the 8 original closures + throw-fallout
  Stages 0–5 + block-arrows AB/C/D + P2 + P3 + this merge-readiness effort,
  with the honest-red → honest-pin story and the post-merge roadmap.
- Adversarial review of the merge-readiness delta (deny lanes, re-pin waves,
  inventory doc) — the standing whole-stage-review discipline applies to
  this delta as its own "stage".
- Final gate per §3; push; wait for all CI jobs green on the PR.
- Merge (merge commit, matching PRs #4–#15); update memory/ledger.

## 5. Risks

1. **Deny lanes newly-red fake-green tests.** Expected and handled: the
   per-wave gate surfaces them immediately and they join the adjudication.
2. **browser-cdp-smoke lane.** Known residual risk (throw-fallout Stage 0);
   it has not run green on this branch recently. Checked in CI at Step 3;
   local `cargo test -p kali_cli --test browser_cdp_smoke -- --ignored`
   available for debugging if it reds.
3. **Volume.** ~712 re-pins across many families is multi-session,
   subagent-driven work; per-wave gates + the committed baseline make it
   resumable and verifiable at every checkpoint.
4. **Toolchain drift.** CI escalates clippy warnings; always verify with the
   CI-exact command lines listed in Step 0, not softer local variants.
