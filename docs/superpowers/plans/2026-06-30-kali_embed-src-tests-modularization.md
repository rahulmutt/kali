# kali_embed `src/tests.rs` Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the 20-test, 571-line co-located unit-test monolith `crates/kali_embed/src/tests.rs` into a drained facade + four per-concern `#[path] mod` submodules under `src/tests/`, with byte-identical test bodies and zero behavior/API change.

**Architecture:** Pure verbatim code-motion via the series' reusable `.superpowers/sdd/move_fns.py` (leading-prefix grouping mode, all 20 tests covered by explicit prefixes — no `*` catch-all) in a single mover invocation, followed by the literal build/test/byte-identity gates. The facade keeps its 4 `use` lines + 1 non-test helper fn (`permissive_policy`); children reach them via `use super::*;`. Integration is local-`main` ff-merge only.

**Tech Stack:** Rust (cargo workspace), Python 3 mover/verifier tools (git-ignored scratch under `.superpowers/sdd/`).

## Global Constraints

- **Verbatim moves only** — no `cargo fmt`, no path rewrites, no `pub`-widening, no `include_*!` edits, no body edits. Nested helper fns travel inside their parent test.
- **Integration: local-`main` ff-merge only — NEVER push to origin.** origin/main intentionally lags.
- **Gates are literal (this crate's baseline is clean):** `cargo build -p kali_embed --tests` → 0 warnings; `cargo test -p kali_embed --lib` → 20 passed / 0 failed.
- **Facade `src/tests.rs` must drain to 0 module-level `#[test]` fns** (1 non-test helper retained: `permissive_policy`).
- **`lib.rs:24–26` decl (`#[cfg(test)]` / `#[path = "tests.rs"]` / `mod tests;`) untouched; no production `.rs` file touched.**
- **move_fns.py lexer invariants** (`FN_RE` / `IDENT_CHARS` / `find_close_line`) must stay byte-identical to the version that produced prior entries; only GROUPS/ROOT/docstring/main may differ.
- Groups-spec (four disjoint leading-prefix families, no catch-all — every one of the 20 tests is claimed by an explicit prefix):
  `compiler=compiles_,compile_lib_,temporary_source_paths_;runtime_profiles=compiler_rejects_;context=embedding_context_,embedding_layer_,embedding_operation_context_;predicates=embedding_predicates_,embedding_predicate_registration_`

---

### Task 0: Branch, tooling confirmation, baseline capture

**Files:**
- Create (scratch): `$SCRATCH/embed-tests.rs.orig` (pre-split copy of the monolith), where `$SCRATCH=/tmp/claude-1000/-workspace/39d69a85-9cee-4c27-8eda-da22f1c9546d/scratchpad`
- Create (scratch): `.superpowers/sdd/baseline-embed-tests.txt`, `.superpowers/sdd/baseline-embed-warnings.txt`
- Read-only: `.superpowers/sdd/move_fns.py` (confirm prefix mode), `.superpowers/sdd/verify.py`

**Interfaces:**
- Produces: the `refactor/kali_embed-src-tests-modularization` branch off `main` (already created and holding the `[spec]` + `[plan]` commits); a verified-prefix-mode `move_fns.py`; the baseline 20-name test set and the 0-warning baseline, for the Task 1 gates to diff against.

- [ ] **Step 1: Confirm we are on the work branch (already created off main)**

```bash
cd "$(git rev-parse --show-toplevel)"
git branch --show-current
```
Expected: `refactor/kali_embed-src-tests-modularization`. (The branch was created off `main` during brainstorming and already carries the spec + this plan. If for any reason it is missing, recreate with `git checkout main && git checkout -b refactor/kali_embed-src-tests-modularization`.)

- [ ] **Step 2: Confirm move_fns.py is in leading-prefix mode**

Run: `sed -n '120,135p' .superpowers/sdd/move_fns.py`
Expected: `group_for` matches via `if fn_name.startswith(prefs):` (leading-prefix), NOT exact-name `==`. If it shows exact-name `==` matching (a prior crate left the exact-name variant), restore the prefix variant (only `group_for`/`GROUPS`/`ROOT`/`main`/docstring may change; keep `FN_RE`/`IDENT_CHARS`/`find_close_line` byte-identical). A `*` catch-all branch may or may not be present — it is unused here (all groups have explicit prefixes), so its presence/absence does not matter.

- [ ] **Step 3: Confirm clean baseline build (0 warnings)**

```bash
cargo build -p kali_embed --tests 2>&1 | grep -c '^warning' | tee .superpowers/sdd/baseline-embed-warnings.txt
```
Expected: `0`

- [ ] **Step 4: Confirm clean baseline test run (20 pass)**

Run: `cargo test -p kali_embed --lib 2>&1 | grep 'test result'`
Expected: `test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`

- [ ] **Step 5: Capture the baseline test-name set and a pre-split copy of the monolith**

```bash
cargo test -p kali_embed --lib -- --list 2>/dev/null \
  | grep ': test$' | sed -E 's/: test$//; s/^.*:://' | sort \
  > .superpowers/sdd/baseline-embed-tests.txt
wc -l .superpowers/sdd/baseline-embed-tests.txt   # expect 20
cp crates/kali_embed/src/tests.rs \
  /tmp/claude-1000/-workspace/39d69a85-9cee-4c27-8eda-da22f1c9546d/scratchpad/embed-tests.rs.orig
```
Expected: `20 .superpowers/sdd/baseline-embed-tests.txt`. No source changed in this task — nothing to commit; the deliverable is the branch + captured baselines.

---

### Task 1: Split `src/tests.rs` into four submodules (single mover run + gates)

**Files:**
- Modify: `crates/kali_embed/src/tests.rs` (drained to facade: 4 use-lines + 1 helper `permissive_policy` + 4 `#[path] mod` decls)
- Create: `crates/kali_embed/src/tests/compiler.rs` (5 tests), `crates/kali_embed/src/tests/runtime_profiles.rs` (3), `crates/kali_embed/src/tests/context.rs` (6), `crates/kali_embed/src/tests/predicates.rs` (6)

**Interfaces:**
- Consumes: the Task 0 branch, prefix-mode `move_fns.py`, `baseline-embed-tests.txt` (20 names), `embed-tests.rs.orig`.
- Produces: the modularized test tree; facade with 0 `#[test]`. No symbols exported to other tasks (the public API is unchanged).

- [ ] **Step 1: Run the mover (single invocation, all four groups)**

```bash
cd crates/kali_embed
python3 ../../.superpowers/sdd/move_fns.py src/tests.rs \
  "compiler=compiles_,compile_lib_,temporary_source_paths_;runtime_profiles=compiler_rejects_;context=embedding_context_,embedding_layer_,embedding_operation_context_;predicates=embedding_predicates_,embedding_predicate_registration_"
```
Expected stdout:
```
moved 20 fns into 4 submodules under src/tests/
  compiler: 5
  runtime_profiles: 3
  context: 6
  predicates: 6
```
If instead it raises `RuntimeError: fn <name> matched no group`, STOP — there is no `*` catch-all here, so an unmatched name means a `#[test]` fn whose prefix is not in the spec (or a typo in the spec). Do not hand-edit the source; re-examine the grouping with the plan author. (The non-test helper `permissive_policy` has no `#[test]` attribute and is correctly left in the facade by the mover, NOT flagged as unmatched.)

- [ ] **Step 2: Confirm the facade drained to 0 `#[test]`, kept the helper, and got the decls**

```bash
grep -c '#\[test\]' src/tests.rs              # expect 0
tail -n 8 src/tests.rs                          # expect the 4 #[path]/mod pairs
grep -n 'fn permissive_policy' src/tests.rs     # expect the 1 helper retained
```
Expected: `0` test attrs; the four `#[path = "tests/<g>.rs"]` + `mod <g>;` pairs present; the `permissive_policy` helper still present in the facade.

- [ ] **Step 3: Build with the tests — must hold 0 warnings**

```bash
cd "$(git rev-parse --show-toplevel)"
cargo build -p kali_embed --tests 2>&1 | grep -c '^warning'
```
Expected: `0` (matches `baseline-embed-warnings.txt`). A new `unused_imports`/visibility warning here means a `use super::*;`/helper-reach problem (e.g. the `predicates` submodule not seeing `permissive_policy`) — investigate, do not suppress, do not widen `pub`.

- [ ] **Step 4: Run the lib tests — must hold 20 pass / 0 fail**

Run: `cargo test -p kali_embed --lib 2>&1 | grep 'test result'`
Expected: `test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`

- [ ] **Step 5: Prove the test-name set is conserved (no test lost/renamed)**

```bash
cargo test -p kali_embed --lib -- --list 2>/dev/null \
  | grep ': test$' | sed -E 's/: test$//; s/^.*:://' | sort \
  | diff - .superpowers/sdd/baseline-embed-tests.txt
```
Expected: empty output (exit 0). Stripping `^.*::` removes the new `compiler::`/`runtime_profiles::`/`context::`/`predicates::` module prefix so bare names compare equal.

- [ ] **Step 6: Prove byte-identity of all 20 test bodies (the decisive gate)**

```bash
cd crates/kali_embed
python3 ../../.superpowers/sdd/verify.py \
  /tmp/claude-1000/-workspace/39d69a85-9cee-4c27-8eda-da22f1c9546d/scratchpad/embed-tests.rs.orig \
  'src/tests/*.rs'
```
Expected: prints a 20/20 match summary and exits 0. Non-zero exit = a body or name-set mismatch; investigate before committing. (No facade-pin glob arg is needed — there are 0 `include_*!` pins.)

- [ ] **Step 7: Confirm no production file and no lib.rs decl was touched**

```bash
cd "$(git rev-parse --show-toplevel)"
git status --short crates/kali_embed/src
git diff --stat crates/kali_embed/src/lib.rs   # expect empty (no change)
```
Expected: modified `src/tests.rs`, 4 new `src/tests/*.rs`; `lib.rs` unchanged; no other `src/*.rs` (`artifact.rs`/`compiler.rs`/`context.rs`/`error.rs`) touched.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_embed/src/tests.rs crates/kali_embed/src/tests/
git commit -m "refactor(kali_embed): split src/tests.rs into per-concern test submodules [refactor]"
```

---

### Task 2: Finalize — whole-branch review, re-verify on merged main, integrate

**Files:**
- Modify (post-merge, scratch): `.superpowers/sdd/progress-kali_embed-srctests-DONE.md` (ledger)
- Modify (post-merge): `/home/dev/.claude/projects/-workspace/memory/crate-modularization-series.md` + `MEMORY.md` pointer (append 37th entry)

**Interfaces:**
- Consumes: the committed Task 1 branch.
- Produces: the work landed on local `main`; the deleted branch; the updated series memory.

- [ ] **Step 1: Whole-branch diff for review**

```bash
cd "$(git rev-parse --show-toplevel)"
BASE=$(git merge-base main refactor/kali_embed-src-tests-modularization)
git diff "$BASE"..refactor/kali_embed-src-tests-modularization \
  -- crates/kali_embed/src \
  > .superpowers/sdd/review-embed-srctests.diff
```
Review (opus, whole-branch) must confirm: every removed line reappears verbatim in a submodule; net new lines = scaffold only (4×(`use super::*;`+blank) + 4 `#[path]`/`mod` pairs); 0 production/`pub`/`include`/fmt change; facade `#[test]` count = 0; helper `permissive_policy` retained. Expected: **0 findings.**

- [ ] **Step 2: ff-merge to local main**

```bash
git checkout main
git merge --ff-only refactor/kali_embed-src-tests-modularization
```
Expected: fast-forward (no merge commit).

- [ ] **Step 3: Re-verify on merged main**

```bash
cargo build -p kali_embed --tests 2>&1 | grep -c '^warning'   # expect 0
cargo test  -p kali_embed --lib    2>&1 | grep 'test result'  # expect 20 passed; 0 failed
```

- [ ] **Step 4: Confirm origin was NOT pushed, then delete the branch**

```bash
git branch -d refactor/kali_embed-src-tests-modularization
git log --oneline origin/main -1   # confirm origin still lags (unchanged)
```
Expected: branch deleted; `git status` shows local main ahead of origin/main; **no `git push` was run.**

- [ ] **Step 5: Update the series memory (37th entry)**

Append a `kali_embed (37th)` paragraph to `crate-modularization-series.md` recording: split `src/tests.rs` 20 tests → 4 submodules (compiler 5 / runtime_profiles 3 / context 6 / predicates 6) via leading-prefix mover (no catch-all), facade drained to 0 `#[test]` (1 helper `permissive_policy` retained), 0 include pins, clean literal gates, merged commit hash, byte-identity proven (20/20). Note the frontier shift: **the co-located `tests.rs`-named monolith frontier is now exhausted** (remaining `tests.rs` files — kali_error 2, kali_fmt 2, kali_cli 8 — are small single-concern and kept whole). Add the `MEMORY.md` one-line pointer if not already present.

---

## Notes / risks (from series memory)

- **`use super::*;` cutoff:** the facade's helper (`permissive_policy`) and its `use` lines (incl. `use super::*;` re-exporting crate-root `pub` items + `use crate::compiler::temporary_source_path;`, `use std::{fs, sync::{Arc, Mutex}};`, `use tempfile::tempdir;`) reach children through the glob — this is the established pattern and compiles at 0 warnings. If a child fails to see a symbol, the fix is never to widen `pub`; re-examine the facade's retained `use` lines.
- **Helper only used by one submodule:** `permissive_policy` is consumed only by 4 `embedding_predicates_*` tests (now in `predicates.rs`). It still stays in the facade (the mover leaves non-`#[test]` fns in place automatically) and `predicates.rs` reaches it via `use super::*;`. Do NOT move the helper into `predicates.rs` — that would be a non-verbatim restructuring.
- **Disjoint-prefix partition (no catch-all):** the four families are mutually non-prefixing — `embedding_context_` is not a prefix of `embedding_operation_context_`; `embedding_predicates_` is not a prefix of `embedding_predicate_registration_`; `compiler_rejects_` shares no prefix with `compiles_`/`compile_lib_`. So group order does not affect assignment, and every one of the 20 tests matches exactly one prefix. A `RuntimeError: matched no group` therefore signals a real surprise (renamed/new test), not an expected catch-all gap.
- **No `include_*!` pins needed** — confirmed 0 in `src/tests.rs`. (If a future re-run finds any, pin via the mover's optional 3rd arg; never rewrite the path.)
- **No env carve-outs** — this crate's baseline is genuinely 0 warnings / fully green, so the literal gates apply as written.
