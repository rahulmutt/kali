# kali_sandbox `src/tests.rs` Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the 41-test, 1303-line co-located unit-test monolith `crates/kali_sandbox/src/tests.rs` into a drained facade + four per-concern `#[path] mod` submodules under `src/tests/`, with byte-identical test bodies and zero behavior/API change.

**Architecture:** Pure verbatim code-motion via the series' reusable `.superpowers/sdd/move_fns.py` (leading-prefix grouping mode) in a single mover invocation, followed by the literal build/test/byte-identity gates. The facade keeps its `use` lines + 3 non-test helper fns; children reach them via `use super::*;`. Integration is local-`main` ff-merge only.

**Tech Stack:** Rust (cargo workspace), Python 3 mover/verifier tools (git-ignored scratch under `.superpowers/sdd/`).

## Global Constraints

- **Verbatim moves only** — no `cargo fmt`, no path rewrites, no `pub`-widening, no `include_*!` edits, no body edits. Nested helper fns travel inside their parent test.
- **Integration: local-`main` ff-merge only — NEVER push to origin.** origin/main intentionally lags.
- **Gates are literal (this crate's baseline is clean):** `cargo build -p kali_sandbox --tests` → 0 warnings; `cargo test -p kali_sandbox --lib` → 41 passed / 0 failed.
- **Facade `src/tests.rs` must drain to 0 module-level `#[test]` fns** (3 non-test helpers retained: `write_source_fixture`, `write_source_fixture_with_extension`, `valid_policy`).
- **`lib.rs:36–38` decl (`#[cfg(test)]` / `#[path = "tests.rs"]` / `mod tests;`) untouched; no production `.rs` file touched.**
- **move_fns.py lexer invariants** (`FN_RE` / `IDENT_CHARS` / `find_close_line`) must stay byte-identical to the version that produced prior entries; only GROUPS/ROOT/docstring/main may differ.
- Groups-spec (all four groups partition by disjoint leading prefix):
  `policy=policy_;predicates=predicate_,registered_,declarative_,late_,access_;effect_analysis=effect_analysis_;effect_reports=effect_reports_,effect_inference_`

---

### Task 0: Branch, tooling confirmation, baseline capture

**Files:**
- Create (scratch): `$SCRATCH/sandbox-tests.rs.orig` (pre-split copy of the monolith), where `$SCRATCH=/tmp/claude-1000/-workspace/c04e0a5d-eaaf-41d0-91e4-5d63a8f64eb8/scratchpad`
- Create (scratch): `.superpowers/sdd/baseline-sandbox-tests.txt`, `.superpowers/sdd/baseline-sandbox-warnings.txt`
- Read-only: `.superpowers/sdd/move_fns.py:124-131` (confirm prefix mode), `.superpowers/sdd/verify.py`

**Interfaces:**
- Produces: a clean `refactor/kali_sandbox-srctests-modularization` branch off `main`; a verified-prefix-mode `move_fns.py`; the baseline 41-name test set and the 0-warning baseline, for the Task 1 gates to diff against.

- [ ] **Step 1: Create the work branch off main**

```bash
cd "$(git rev-parse --show-toplevel)"
git checkout main
git checkout -b refactor/kali_sandbox-srctests-modularization
```

- [ ] **Step 2: Confirm move_fns.py is in leading-prefix mode**

Run: `sed -n '124,131p' .superpowers/sdd/move_fns.py`
Expected: `group_for` matches via `if fn_name.startswith(prefs):` (NOT `==`). If it shows exact-name `==` matching, restore the prefix variant (only `group_for`/`GROUPS`/`main` may change; keep `FN_RE`/`IDENT_CHARS`/`find_close_line` byte-identical).

- [ ] **Step 3: Confirm clean baseline build (0 warnings)**

```bash
cargo build -p kali_sandbox --tests 2>&1 | grep -c '^warning' | tee .superpowers/sdd/baseline-sandbox-warnings.txt
```
Expected: `0`

- [ ] **Step 4: Confirm clean baseline test run (41 pass)**

Run: `cargo test -p kali_sandbox --lib 2>&1 | grep 'test result'`
Expected: `test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`

- [ ] **Step 5: Capture the baseline test-name set and a pre-split copy of the monolith**

```bash
cargo test -p kali_sandbox --lib -- --list 2>/dev/null \
  | grep ': test$' | sed -E 's/: test$//; s/^.*:://' | sort \
  > .superpowers/sdd/baseline-sandbox-tests.txt
wc -l .superpowers/sdd/baseline-sandbox-tests.txt   # expect 41
cp crates/kali_sandbox/src/tests.rs \
  /tmp/claude-1000/-workspace/c04e0a5d-eaaf-41d0-91e4-5d63a8f64eb8/scratchpad/sandbox-tests.rs.orig
```
Expected: `41 .superpowers/sdd/baseline-sandbox-tests.txt`. No source changed in this task — nothing to commit; the deliverable is the branch + captured baselines.

---

### Task 1: Split `src/tests.rs` into four submodules (single mover run + gates)

**Files:**
- Modify: `crates/kali_sandbox/src/tests.rs` (drained to facade: use-lines + 3 helpers + 4 `#[path] mod` decls)
- Create: `crates/kali_sandbox/src/tests/policy.rs` (10 tests), `crates/kali_sandbox/src/tests/predicates.rs` (12), `crates/kali_sandbox/src/tests/effect_analysis.rs` (10), `crates/kali_sandbox/src/tests/effect_reports.rs` (9)

**Interfaces:**
- Consumes: the Task 0 branch, prefix-mode `move_fns.py`, `baseline-sandbox-tests.txt` (41 names), `sandbox-tests.rs.orig`.
- Produces: the modularized test tree; facade with 0 `#[test]`. No symbols exported to other tasks (the public API is unchanged).

- [ ] **Step 1: Run the mover (single invocation, all four groups)**

```bash
cd crates/kali_sandbox
python3 ../../.superpowers/sdd/move_fns.py src/tests.rs \
  "policy=policy_;predicates=predicate_,registered_,declarative_,late_,access_;effect_analysis=effect_analysis_;effect_reports=effect_reports_,effect_inference_"
```
Expected stdout:
```
moved 41 fns into 4 submodules under src/tests/
  policy: 10
  predicates: 12
  effect_analysis: 10
  effect_reports: 9
```
If instead it raises `RuntimeError: fn <name> matched no group`, STOP — a test name fell outside the four prefix-sets; do not hand-edit, re-examine the grouping with the plan author.

- [ ] **Step 2: Confirm the facade drained to 0 `#[test]` and the decls were appended**

```bash
grep -c '#\[test\]' src/tests.rs            # expect 0
tail -n 11 src/tests.rs                       # expect the 4 #[path]/mod pairs
grep -n 'fn write_source_fixture\|fn write_source_fixture_with_extension\|fn valid_policy' src/tests.rs  # expect the 3 helpers retained
```
Expected: `0` test attrs; the four `#[path = "tests/<g>.rs"]` + `mod <g>;` pairs present; all 3 helper fns still present.

- [ ] **Step 3: Build with the tests — must hold 0 warnings**

```bash
cd "$(git rev-parse --show-toplevel)"
cargo build -p kali_sandbox --tests 2>&1 | grep -c '^warning'
```
Expected: `0` (matches `baseline-sandbox-warnings.txt`). A new `unused_imports`/visibility warning here means a `use super::*;`/helper-reach problem — investigate, do not suppress.

- [ ] **Step 4: Run the lib tests — must hold 41 pass / 0 fail**

Run: `cargo test -p kali_sandbox --lib 2>&1 | grep 'test result'`
Expected: `test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`

- [ ] **Step 5: Prove the test-name set is conserved (no test lost/renamed)**

```bash
cargo test -p kali_sandbox --lib -- --list 2>/dev/null \
  | grep ': test$' | sed -E 's/: test$//; s/^.*:://' | sort \
  | diff - .superpowers/sdd/baseline-sandbox-tests.txt
```
Expected: empty output (exit 0). Stripping `^.*::` removes the new `policy::`/`predicates::`/… module prefix so bare names compare equal.

- [ ] **Step 6: Prove byte-identity of all 41 test bodies (the decisive gate)**

```bash
cd crates/kali_sandbox
python3 ../../.superpowers/sdd/verify.py \
  /tmp/claude-1000/-workspace/c04e0a5d-eaaf-41d0-91e4-5d63a8f64eb8/scratchpad/sandbox-tests.rs.orig \
  'src/tests/*.rs'
```
Expected: prints a 41/41 match summary and exits 0. Non-zero exit = a body or name-set mismatch; investigate before committing.

- [ ] **Step 7: Confirm no production file and no lib.rs decl was touched**

```bash
cd "$(git rev-parse --show-toplevel)"
git status --short crates/kali_sandbox/src
git diff --stat crates/kali_sandbox/src/lib.rs   # expect empty (no change)
```
Expected: modified `src/tests.rs`, 4 new `src/tests/*.rs`; `lib.rs` unchanged; no other `src/*.rs` touched.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_sandbox/src/tests.rs crates/kali_sandbox/src/tests/
git commit -m "refactor(kali_sandbox): split src/tests.rs into per-concern test submodules [refactor]"
```

---

### Task 2: Finalize — whole-branch review, re-verify on merged main, integrate

**Files:**
- Modify (post-merge, scratch): `.superpowers/sdd/progress-kali_sandbox-srctests-DONE.md` (ledger)
- Modify (post-merge): `/home/dev/.claude/projects/-workspace/memory/crate-modularization-series.md` + `MEMORY.md` pointer (append 35th entry)

**Interfaces:**
- Consumes: the committed Task 1 branch.
- Produces: the work landed on local `main`; the deleted branch; the updated series memory.

- [ ] **Step 1: Whole-branch diff for review**

```bash
cd "$(git rev-parse --show-toplevel)"
BASE=$(git merge-base main refactor/kali_sandbox-srctests-modularization)
git diff "$BASE"..refactor/kali_sandbox-srctests-modularization \
  > .superpowers/sdd/review-sandbox-srctests.diff
```
Review (opus, whole-branch) must confirm: every removed line reappears verbatim in a submodule; net new lines = scaffold only (4×(`use super::*;`+blank) + 4 `#[path]`/`mod` pairs); 0 production/`pub`/`include`/fmt change; facade `#[test]` count = 0. Expected: **0 findings.**

- [ ] **Step 2: ff-merge to local main**

```bash
git checkout main
git merge --ff-only refactor/kali_sandbox-srctests-modularization
```
Expected: fast-forward (no merge commit).

- [ ] **Step 3: Re-verify on merged main**

```bash
cargo build -p kali_sandbox --tests 2>&1 | grep -c '^warning'   # expect 0
cargo test  -p kali_sandbox --lib    2>&1 | grep 'test result'  # expect 41 passed; 0 failed
```

- [ ] **Step 4: Confirm origin was NOT pushed, then delete the branch**

```bash
git branch -d refactor/kali_sandbox-srctests-modularization
git log --oneline origin/main -1   # confirm origin still lags (unchanged)
```
Expected: branch deleted; `git status` shows local main ahead of origin/main; **no `git push` was run.**

- [ ] **Step 5: Update the series memory (35th entry)**

Append a `kali_sandbox (35th)` paragraph to `crate-modularization-series.md` recording: split `src/tests.rs` 41 tests → 4 submodules (policy 10 / predicates 12 / effect_analysis 10 / effect_reports 9) via leading-prefix mover, facade drained to 0 `#[test]` (3 helpers retained), 0 include pins, clean literal gates, merged commit hash, byte-identity proven. Note the frontier shift: `src/*_tests.rs`-named monoliths are exhausted; remaining `tests.rs`-named monoliths are kali_embed (20) and kali_lir (11).

---

## Notes / risks (from series memory)

- **`use super::*;` cutoff:** the facade's helpers and `use std::{…}` reach children through the glob — this is the established pattern and compiles at 0 warnings. If a child fails to see a helper, the fix is never to widen `pub`; re-examine the facade's retained `use` lines.
- **No `include_*!` pins needed** — confirmed 0 in `src/tests.rs`. (If a future re-run finds any, pin via the mover's optional 3rd arg; never rewrite the path.)
- **No env carve-outs** — unlike kali_cli, this crate's baseline is genuinely 0 warnings / fully green, so the literal gates apply as written.
