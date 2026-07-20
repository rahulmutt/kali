# PR #16 merge readiness — revision 2 (Strategy B: re-pin class-A, ignore class-B)

**Date:** 2026-07-20
**Branch:** `soundness-batch1-pra` (PR #16, draft since 2026-07-10)
**Supersedes:** `2026-07-18-pr16-merge-readiness-design.md` (same goal; strategy revised
after the silent-miscompile register landed and Group-1 fixes completed).
**Decision owner:** maintainer (choices ratified in the 2026-07-20 brainstorm).

## 0. Why a revision

The 2026-07-18 design planned to make the branch CI-green by re-pinning ~694 honest-red
workspace tests, deny-lane-first for silent miscompiles. Executing it exposed a deeper
problem, now recorded in `docs/superpowers/followups/kali-silent-miscompile-register.md`:
~33 silent-miscompile defects **corrupt the evidence** the re-pin classification relied on
(a default param truncates the module, `console` eats arguments, `const` re-evaluates its
initializer, aliasing defeats guards). Pinning a test as "kali has no X" when the real
reason it fails is a truncated fixture would **commit a falsehood to `main`**.

The register mandated a fix ordering. Since 2026-07-18, ~19 `[pr16-merge]` commits worked it:

- **Group 1 (evidence-corrupting) — DONE.** R-01 param truncation (`e276ffa7a`), R-04
  console args (`86de4271d`), R-07 `const` binding (`f8c60482c`).
- **Group 2 (contained) — partial.** G6 unimplemented-builtins fail-closed (`30715f5d2`),
  plus `&&`/`||` short-circuit, `===`-by-type, boolean concat rendering, first-class-fn
  fail-closed.
- **Groups 3 (guard-holes) and 4 (architectural) — not started.**

With Group 1 complete, the compiler's observable behavior can once again be trusted as
evidence, so re-derivation is now sound. This revision also adopts the maintainer's
faster, still-honest adjudication: **`#[ignore]`+track** the silent-miscompile tests
instead of deny-laning and re-pinning every one of them.

## 1. Current measured state (authoritative)

- **708 full-workspace reds** at HEAD `b379f7ee6`, double-enumeration basis
  (`cargo test --workspace --no-fail-fast`, `rm -rf .kali-cache` first). The frozen
  694 baseline is stale (Group-1/Group-2 fixes drained some fake-greens and un-masked
  others); **the 708 set is the new baseline**.
- CI's `build` job runs `cargo test --workspace` and **fail-fasts** — its visible "5–68
  failures" is the first failing binary only, not the true count. The full 708 is the
  merge gate.
- `main` (`origin/main`, `40c7cb71e`) is fully CI-green and is a proper ancestor of the
  branch (branch = `origin/main` + 277 commits, 0 behind).

**Honesty framing (this is what resolves the register's concern).** All 708 tests are
*fake-green on `main`* — they pass there only because kali's self-checks were silent
no-ops. The underlying miscompiles already ship on `main` today, hidden by those
fake-greens. Therefore `#[ignore = "R-NN, tracked #issue"]` is **strictly more honest
than the `main` status quo**: it does not degrade `main`; it makes the suite tell the
truth about pre-existing gaps.

## 2. Ratified decisions (2026-07-20)

1. **Strategy B** — adjudicate each red test into one of three actions:
   - **Class A (kali fails closed):** clean diagnostic — `E5506` / node-shaped trap /
     nonzero exit with a diagnostic. → **re-pin** to assert that diagnostic. A true,
     durable regression guard.
   - **Class B (silent miscompile):** wrong value at exit 0. → **`#[ignore]`+track**
     (see decision 3 for the deny-lane exception).
   - **Class C (harness/predicate artifact):** product behavior is correct, the test's
     envelope/predicate is wrong. → fix the test predicate honestly (in scope; a test
     fix, not a capability change).
2. **No new runtime capability.** Class-B fixes are deferred post-merge. The only code
   changes admitted here are (a) class-C predicate fixes and (b) the cheap class-B deny
   lanes of decision 3.
3. **Cheap class-B gets a deny lane; architectural class-B gets `#[ignore]`.** For a
   class-B family with a small, known-shape fix (register **Group 2**: e.g. R-11 bitwise
   compound-assign, R-24 `Object.freeze`, R-09 `continue`, R-16 string-repr arms, R-26
   unary `+`, R-27 comma, R-22 `==`), **land the fail-closed deny lane** (allowlist at the
   single choke point; `kali_types`↔`kali_codegen` twins in one commit) so **no silent
   miscompile ships**, then re-pin the reject (it becomes class A). For architectural
   class-B (**Groups 3/4**: R-05/R-02 first-class functions, R-06 composite initializers,
   R-08/R-21 no-null/undefined-value, R-10 block shadowing, R-14 returned arrays, R-23
   `typeof`), **`#[ignore]`+track**.
4. **One GitHub issue per deferred class-B family**, linked from every `#[ignore]` reason
   string in that family and from the inventory row. Register R-NN referenced in the issue.
5. **Allowlist at the choke point, never a denylist of shapes** (standing repo lesson) for
   every deny lane.
6. **Merge review = delta only.** The 10+ per-stage adversarial reviews stand; this effort
   is reviewed as its own "stage" (the triage + waves + deny lanes + inventory), plus a
   mechanical full-gate.

## 3. Definition of done

PR #16 merges to `main` via a merge commit (matching PRs #4–#15), with full CI green on
the PR head:

- `cargo test --workspace` — **0 failures**, double-enumerated with zero drift
  (`--no-fail-fast`, cache cleared, two identical sorted empty failure sets).
- `cargo fmt --all -- --check` clean; `cargo clippy --workspace -- -D warnings` clean
  (CI-exact commands).
- The 6 CLBG goldens (nbody, fannkuch, spectral-norm, mandelbrot, binary-trees, fasta)
  byte-for-byte.
- Remaining CI jobs green: `phase1-evidence` (runtime_smoke), determinism (both OSes),
  browser-cdp-smoke, package-corpus, build matrix, proof-check.
- PR #16 description rewritten to cover the whole branch.
- One tracking GitHub issue filed per deferred class-B family; every `#[ignore]` reason
  references its issue + register R-NN.
- Memory/ledger updated post-merge.

**Non-goals:** any new runtime capability; fixing architectural class-B miscompiles. Those
are the post-merge roadmap (P4 `URL`/`URLSearchParams`, P5 `TextEncoder`/`TextDecoder`,
Group-3 guard-hole closures, Group-4 architectural stages).

## 4. Plan of record

### Step 0 — refreeze baseline + mechanical CI

- Re-verify `cargo fmt --all -- --check` and `cargo clippy --workspace -- -D warnings`
  are clean at HEAD (the `9ba68f460` fix landed post the 2026-07-18 red state; confirm it
  still holds). Fix any residue; commit.
- Commit the canonical baseline `docs/superpowers/followups/pr16-honest-red-baseline.txt`
  = the 708-line double-enumerated zero-drift set at HEAD. (Overwrites the stale 694 file;
  record the 694→708 provenance in the commit message.)

### Step 1 — automated A/B/C triage → inventory revision 2

Name-based classification is void (register §0). But the **A/B split is mechanically
detectable from each failure's payload**:

- Capture every red test's output (`cargo test <name> -- --nocapture`, or a batched
  harness that records stdout/stderr/exit per test).
- **Class A** ⇒ stderr contains an `E####` diagnostic, or a node-shaped trap, or the
  process exits nonzero with a diagnostic.
- **Class B** ⇒ the process exits 0 and the failure is a value/output mismatch.
- **Class C** ⇒ the divergence is in the harness envelope (schema/predicate), not the
  program's own output. Flag for manual confirmation.
- **Ambiguous** (a warning *and* a wrong value; exit 0 but with a `[warn]`): manual audit.

Rebuild `docs/superpowers/followups/pr16-honest-repin-inventory.md` as **revision 2**:
one row per family — `family | pattern | count | representative | evidence (fixed-binary
transcript) | class A/B/C | action (re-pin / deny-lane+pin / ignore+issue / predicate-fix)
| flip-back condition | issue #`. Coverage ledger sums to 708, no silent remainder. The
old revision's rows are a hypothesis only; every class call is re-observed on the current
binary.

### Step 2 — maintainer checkpoint

Present the revision-2 adjudication table: family sizes, A/B/C split, which class-B
families get a Group-2 deny lane vs `#[ignore]`, proposed wave order (largest family
first). Ratify before waves start. This is the only blocking human gate before close-out.

### Step 3 — adjudication waves (subagent-parallel by family)

Per family wave:

1. **Class A:** re-pin every member to assert the observed diagnostic (`E####` in stderr /
   `!status.success()` / node-shaped trap text), each pin commented with a pointer to the
   inventory row. Use the file-local test helpers.
2. **Class B, cheap (Group 2):** TDD the deny-lane reject pin first (it fails — the
   miscompile currently succeeds silently); land the allowlist-at-choke-point deny lane
   (types+codegen twins, one commit); re-pin the reject; probe 2–3 adjacent admitted
   shapes to prove no over-reject.
3. **Class B, architectural (Groups 3/4):** file the family's tracking GitHub issue;
   `#[ignore = "R-NN: <one-line reason>; tracked <issue-url>; see inventory#<family>"]`
   every member. Assert nothing about correctness.
4. **Class C:** fix the harness predicate/envelope so it reports honestly; assert the
   honest outcome.
5. **Wave gate:** full re-enumeration diffed against the frozen 708 baseline —
   **newly-red empty** (`comm -13`), **drain monotone** (`comm -23` only shrinks), goldens
   spot-check green. A deny lane that newly-reds a currently-green (fake-passing) test
   feeds that test into the same adjudication, not an exemption. Update the inventory row
   to DONE with gate numbers; commit with `[pr16-merge]`.

### Step 4 — close-out and merge

- Rewrite the PR #16 description: original 8 closures + throw-fallout Stages 0–5 +
  block-arrows AB/C/D + P2 + P3 + G6 + this rev-2 merge-readiness effort, with the
  honest-red → re-pin/ignore story, the inventory as the canonical aspiration map, and the
  post-merge roadmap.
- Delta-only adversarial review (standing whole-stage discipline). Focus: (a) any re-pin
  asserting a wrong VALUE rather than a reject/trap; (b) class-B deny-lane admit/emit twin
  desync; (c) allowlist completeness at each choke (probe siblings); (d) inventory ledger
  sums to 708; (e) `#[ignore]` reasons all carry an issue + R-NN; (f) pins that would
  still pass if the deny lane were deleted (assert the diagnostic, not just failure).
- Final gate per §3; push; all CI jobs green on the PR.
- `gh pr ready 16 && gh pr merge 16 --merge`; update memory/ledger; mark the stale
  "PR #16 held draft" notes superseded.

## 5. Risks

1. **Deny lanes newly-red fake-green tests.** Expected; the per-wave gate surfaces them and
   they join the adjudication.
2. **browser-cdp-smoke lane.** Currently passing on the branch CI; not runnable headlessly
   here without chromium. CI is the gate of record; debug locally with
   `cargo test -p kali_cli --test browser_cdp_smoke -- --ignored` if it reds.
3. **Volume.** ~708 adjudications is multi-session, subagent-driven; per-wave gates + the
   committed baseline make it resumable and verifiable at each checkpoint.
4. **Toolchain drift.** CI escalates clippy warnings (`-D warnings`); verify with CI-exact
   command lines, not softer local variants.
5. **`#[ignore]` coverage cliff.** Mass-ignoring class-B removes a large slice of the suite
   from the run. Mitigated by: (a) it is more honest than the fake-green status quo; (b)
   every ignore is issue-tracked with a flip-back condition; (c) cheap class-B gets a
   deny lane instead of an ignore, shrinking the ignored set.

## 6. Post-merge roadmap (not in scope here)

Tracked per class-B family issue, sequenced on fresh branches after merge: Group-3
guard-hole closures (R-03/R-12/R-13/R-18/`??`, one allowlist-first project), Group-4
architectural (R-08+R-21 value-repr axis; R-02+R-05 first-class functions; R-10 block
shadowing; R-06/R-14 storage/escape; R-23 `typeof`), and the parity stages P4
(`URL`/`URLSearchParams`) and P5 (`TextEncoder`/`TextDecoder`).
