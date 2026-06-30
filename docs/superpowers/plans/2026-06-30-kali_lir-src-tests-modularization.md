# kali_lir `src/tests.rs` Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the 11-test, 278-line co-located unit-test monolith `crates/kali_lir/src/tests.rs` into a drained facade + three per-concern `#[path] mod` submodules under `src/tests/`, with byte-identical test bodies and zero behavior/API change.

**Architecture:** Pure verbatim code-motion via the series' reusable `.superpowers/sdd/move_fns.py` (leading-prefix grouping mode + a `*` catch-all) in a single mover invocation, followed by the literal build/test/byte-identity gates. The facade keeps its 6 `use` lines + 1 non-test helper fn (`parse_and_lower`); children reach them via `use super::*;`. Integration is local-`main` ff-merge only.

**Tech Stack:** Rust (cargo workspace), Python 3 mover/verifier tools (git-ignored scratch under `.superpowers/sdd/`).

## Global Constraints

- **Verbatim moves only** — no `cargo fmt`, no path rewrites, no `pub`-widening, no `include_*!` edits, no body edits. Nested helper fns travel inside their parent test.
- **Integration: local-`main` ff-merge only — NEVER push to origin.** origin/main intentionally lags.
- **Gates are literal (this crate's baseline is clean):** `cargo build -p kali_lir --tests` → 0 warnings; `cargo test -p kali_lir --lib` → 11 passed / 0 failed.
- **Facade `src/tests.rs` must drain to 0 module-level `#[test]` fns** (1 non-test helper retained: `parse_and_lower`).
- **`lib.rs:16–21` decl (`#[cfg(test)] use kali_mir::MirProgram;` + `#[cfg(test)]` / `#[path = "tests.rs"]` / `mod tests;`) untouched; no production `.rs` file touched.**
- **move_fns.py lexer invariants** (`FN_RE` / `IDENT_CHARS` / `find_close_line`) must stay byte-identical to the version that produced prior entries; only GROUPS/ROOT/docstring/main may differ.
- Groups-spec (two disjoint leading-prefix families + a `*` catch-all, catch-all last):
  `flavor_metadata=test_lir_lowering_preserves_function_flavor_metadata;validation=test_lir_validation_;structure=*`

---

### Task 0: Branch, tooling confirmation, baseline capture

**Files:**
- Create (scratch): `$SCRATCH/lir-tests.rs.orig` (pre-split copy of the monolith), where `$SCRATCH=/tmp/claude-1000/-workspace/25c26801-219a-458b-9956-6330eb9662c1/scratchpad`
- Create (scratch): `.superpowers/sdd/baseline-lir-tests.txt`, `.superpowers/sdd/baseline-lir-warnings.txt`
- Read-only: `.superpowers/sdd/move_fns.py:124-130` (confirm prefix mode + `*` catch-all), `.superpowers/sdd/verify.py`

**Interfaces:**
- Produces: a clean `refactor/kali_lir-srctests-modularization` branch off `main`; a verified-prefix-mode `move_fns.py`; the baseline 11-name test set and the 0-warning baseline, for the Task 1 gates to diff against.

- [ ] **Step 1: Create the work branch off main**

```bash
cd "$(git rev-parse --show-toplevel)"
git checkout main
git checkout -b refactor/kali_lir-srctests-modularization
```

- [ ] **Step 2: Confirm move_fns.py is in leading-prefix mode with `*` catch-all**

Run: `sed -n '124,130p' .superpowers/sdd/move_fns.py`
Expected: `group_for` returns the group when `prefs == "*"` (catch-all) and otherwise matches via `if fn_name.startswith(prefs):` (NOT `==`). If it shows exact-name `==` matching, restore the prefix variant (only `group_for`/`GROUPS`/`main` may change; keep `FN_RE`/`IDENT_CHARS`/`find_close_line` byte-identical).

- [ ] **Step 3: Confirm clean baseline build (0 warnings)**

```bash
cargo build -p kali_lir --tests 2>&1 | grep -c '^warning' | tee .superpowers/sdd/baseline-lir-warnings.txt
```
Expected: `0`

- [ ] **Step 4: Confirm clean baseline test run (11 pass)**

Run: `cargo test -p kali_lir --lib 2>&1 | grep 'test result'`
Expected: `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`

- [ ] **Step 5: Capture the baseline test-name set and a pre-split copy of the monolith**

```bash
cargo test -p kali_lir --lib -- --list 2>/dev/null \
  | grep ': test$' | sed -E 's/: test$//; s/^.*:://' | sort \
  > .superpowers/sdd/baseline-lir-tests.txt
wc -l .superpowers/sdd/baseline-lir-tests.txt   # expect 11
cp crates/kali_lir/src/tests.rs \
  /tmp/claude-1000/-workspace/25c26801-219a-458b-9956-6330eb9662c1/scratchpad/lir-tests.rs.orig
```
Expected: `11 .superpowers/sdd/baseline-lir-tests.txt`. No source changed in this task — nothing to commit; the deliverable is the branch + captured baselines.

---

### Task 1: Split `src/tests.rs` into three submodules (single mover run + gates)

**Files:**
- Modify: `crates/kali_lir/src/tests.rs` (drained to facade: 6 use-lines + 1 helper + 3 `#[path] mod` decls)
- Create: `crates/kali_lir/src/tests/flavor_metadata.rs` (8 tests), `crates/kali_lir/src/tests/validation.rs` (1), `crates/kali_lir/src/tests/structure.rs` (2)

**Interfaces:**
- Consumes: the Task 0 branch, prefix-mode `move_fns.py`, `baseline-lir-tests.txt` (11 names), `lir-tests.rs.orig`.
- Produces: the modularized test tree; facade with 0 `#[test]`. No symbols exported to other tasks (the public API is unchanged).

- [ ] **Step 1: Run the mover (single invocation, all three groups)**

```bash
cd crates/kali_lir
python3 ../../.superpowers/sdd/move_fns.py src/tests.rs \
  "flavor_metadata=test_lir_lowering_preserves_function_flavor_metadata;validation=test_lir_validation_;structure=*"
```
Expected stdout:
```
moved 11 fns into 3 submodules under src/tests/
  flavor_metadata: 8
  validation: 1
  structure: 2
```
If instead it raises `RuntimeError: fn <name> matched no group`, STOP — the `structure=*` catch-all should claim everything not matched by the two earlier prefixes, so this error means the spec was mistyped; do not hand-edit, re-examine the grouping with the plan author.

- [ ] **Step 2: Confirm the facade drained to 0 `#[test]` and the decls were appended**

```bash
grep -c '#\[test\]' src/tests.rs            # expect 0
tail -n 8 src/tests.rs                        # expect the 3 #[path]/mod pairs
grep -n 'fn parse_and_lower' src/tests.rs     # expect the 1 helper retained (line in facade)
```
Expected: `0` test attrs; the three `#[path = "tests/<g>.rs"]` + `mod <g>;` pairs present; the `parse_and_lower` helper still present.

- [ ] **Step 3: Build with the tests — must hold 0 warnings**

```bash
cd "$(git rev-parse --show-toplevel)"
cargo build -p kali_lir --tests 2>&1 | grep -c '^warning'
```
Expected: `0` (matches `baseline-lir-warnings.txt`). A new `unused_imports`/visibility warning here means a `use super::*;`/helper-reach problem — investigate, do not suppress.

- [ ] **Step 4: Run the lib tests — must hold 11 pass / 0 fail**

Run: `cargo test -p kali_lir --lib 2>&1 | grep 'test result'`
Expected: `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`

- [ ] **Step 5: Prove the test-name set is conserved (no test lost/renamed)**

```bash
cargo test -p kali_lir --lib -- --list 2>/dev/null \
  | grep ': test$' | sed -E 's/: test$//; s/^.*:://' | sort \
  | diff - .superpowers/sdd/baseline-lir-tests.txt
```
Expected: empty output (exit 0). Stripping `^.*::` removes the new `flavor_metadata::`/`validation::`/`structure::` module prefix so bare names compare equal.

- [ ] **Step 6: Prove byte-identity of all 11 test bodies (the decisive gate)**

```bash
cd crates/kali_lir
python3 ../../.superpowers/sdd/verify.py \
  /tmp/claude-1000/-workspace/25c26801-219a-458b-9956-6330eb9662c1/scratchpad/lir-tests.rs.orig \
  'src/tests/*.rs'
```
Expected: prints an 11/11 match summary and exits 0. Non-zero exit = a body or name-set mismatch; investigate before committing.

- [ ] **Step 7: Confirm no production file and no lib.rs decl was touched**

```bash
cd "$(git rev-parse --show-toplevel)"
git status --short crates/kali_lir/src
git diff --stat crates/kali_lir/src/lib.rs   # expect empty (no change)
```
Expected: modified `src/tests.rs`, 3 new `src/tests/*.rs`; `lib.rs` unchanged; no other `src/*.rs` touched.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_lir/src/tests.rs crates/kali_lir/src/tests/
git commit -m "refactor(kali_lir): split src/tests.rs into per-concern test submodules [refactor]"
```

---

### Task 2: Finalize — whole-branch review, re-verify on merged main, integrate

**Files:**
- Modify (post-merge, scratch): `.superpowers/sdd/progress-kali_lir-srctests-DONE.md` (ledger)
- Modify (post-merge): `/home/dev/.claude/projects/-workspace/memory/crate-modularization-series.md` + `MEMORY.md` pointer (append 36th entry)

**Interfaces:**
- Consumes: the committed Task 1 branch.
- Produces: the work landed on local `main`; the deleted branch; the updated series memory.

- [ ] **Step 1: Whole-branch diff for review**

```bash
cd "$(git rev-parse --show-toplevel)"
BASE=$(git merge-base main refactor/kali_lir-srctests-modularization)
git diff "$BASE"..refactor/kali_lir-srctests-modularization \
  > .superpowers/sdd/review-lir-srctests.diff
```
Review (opus, whole-branch) must confirm: every removed line reappears verbatim in a submodule; net new lines = scaffold only (3×(`use super::*;`+blank) + 3 `#[path]`/`mod` pairs); 0 production/`pub`/`include`/fmt change; facade `#[test]` count = 0. Expected: **0 findings.**

- [ ] **Step 2: ff-merge to local main**

```bash
git checkout main
git merge --ff-only refactor/kali_lir-srctests-modularization
```
Expected: fast-forward (no merge commit).

- [ ] **Step 3: Re-verify on merged main**

```bash
cargo build -p kali_lir --tests 2>&1 | grep -c '^warning'   # expect 0
cargo test  -p kali_lir --lib    2>&1 | grep 'test result'  # expect 11 passed; 0 failed
```

- [ ] **Step 4: Confirm origin was NOT pushed, then delete the branch**

```bash
git branch -d refactor/kali_lir-srctests-modularization
git log --oneline origin/main -1   # confirm origin still lags (unchanged)
```
Expected: branch deleted; `git status` shows local main ahead of origin/main; **no `git push` was run.**

- [ ] **Step 5: Update the series memory (36th entry)**

Append a `kali_lir (36th)` paragraph to `crate-modularization-series.md` recording: split `src/tests.rs` 11 tests → 3 submodules (flavor_metadata 8 / validation 1 / structure 2 catch-all) via leading-prefix mover + `*` catch-all, facade drained to 0 `#[test]` (1 helper `parse_and_lower` retained), 0 include pins, clean literal gates, merged commit hash, byte-identity proven. Note the frontier: only `kali_embed/src/tests.rs` (20) remains among co-located `tests.rs`-named monoliths.

---

## Notes / risks (from series memory)

- **`use super::*;` cutoff:** the facade's helper (`parse_and_lower`) and its `use` lines (incl. `use super::*;` re-exporting crate-root `pub` items + the `#[cfg(test)] use kali_mir::MirProgram;` from `lib.rs`) reach children through the glob — this is the established pattern and compiles at 0 warnings. If a child fails to see a symbol, the fix is never to widen `pub`; re-examine the facade's retained `use` lines.
- **Multi-line fn signatures:** four `flavor_metadata` tests split their signature across two lines (`fn <name>(` then `) {`). `FN_RE` matches the `fn <name>` line and `find_close_line` brace-counts from there — proven on kali_mir's identical `lower_tests.rs` signatures. No special handling needed.
- **No `include_*!` pins needed** — confirmed 0 in `src/tests.rs`. (If a future re-run finds any, pin via the mover's optional 3rd arg; never rewrite the path.)
- **No env carve-outs** — this crate's baseline is genuinely 0 warnings / fully green, so the literal gates apply as written.
