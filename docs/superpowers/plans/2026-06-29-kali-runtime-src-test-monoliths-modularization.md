# kali_runtime co-located src test-monolith modularization — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split six of kali_runtime's co-located `src/*_tests.rs` unit-test monoliths (141 `#[test]` fns) into a thin facade + per-concern `#[path] mod` submodules via pure verbatim code-motion, with zero behavior change and byte-identical test bodies.

**Architecture:** For each file `F`, the proven series mover (`.superpowers/sdd/move_fns.py`, exact-name-set variant) relocates each `#[test]` fn verbatim into `src/.../F/<group>.rs` (each headed by exactly `use super::*;`), and rewrites the facade `F.rs` to drop the moved fns and append `#[path = "F/<group>.rs"] mod <group>;` decls. The facade keeps its original header `use` lines (children consume them via descendant-visibility through `use super::*;`). All six facades drain to **zero** module-level fns. Byte-identity is proven per file by `.superpowers/sdd/verify.py`.

**Tech Stack:** Rust (cargo, workspace crate `kali_runtime`); Python 3 mover/verify tools (git-ignored scratch under `.superpowers/sdd/`).

**Spec:** `docs/superpowers/specs/2026-06-29-kali-runtime-src-test-monoliths-modularization-design.md`

## Global Constraints

- **Pure relocation.** No new product code, no new tests, no renames, no reordering, no reformatting. `#[test]` attr lines + body + one trailing blank relocate byte-for-byte.
- **Submodule header is exactly `use super::*;`** (nothing else). Facade keeps every original `use` line verbatim. No per-submodule extern `use`s.
- **Every facade ends with zero `#[test]` fns and zero module-level helpers** (all six are fully drained — verified: every module-level fn in these files carries `#[test]`). No `include_*!` pins anywhere (verified 0 across the crate).
- **No `pub`/`pub(crate)` widening** — intra-crate child modules reach parent scope via `use super::*`.
- **Do NOT run `cargo fmt`** — the repo `cargo fmt --all --check` gate is already red on baseline; accepted cosmetic minors are not regressions.
- **The `#[cfg(test)]` + `#[path = "F_tests.rs"]` + `mod F_tests;` decls in each production sibling stay unchanged** — they still name the facade file, which now re-exports its children.
- **Baseline (clean, captured at Task 0):** `cargo build -p kali_runtime --tests` = **0 warnings**; `cargo test -p kali_runtime --lib` = **158 passed / 0 failed**. Gates are literal here (no env carve-outs): **0 warnings unchanged**, **158 pass / 0 fail unchanged**, per-file `--list` bare-fn multiset preserved.
- **Integration: local-main ff-merge only — NEVER push to origin** (origin/main intentionally lags). Re-verify on merged main, then delete the branch.
- **Branch:** `refactor/kali_runtime-modularization` (already created off `318279f1a`; the design spec is already committed on it). SDD ledger at `.superpowers/sdd/progress.md` (git-ignored scratch), overwrite per task.
- **Tools are reused as-is** from the kali_types entry: `.superpowers/sdd/move_fns.py` (exact-name-set grouping) and `.superpowers/sdd/verify.py`. Do NOT edit `FN_RE` / `IDENT_CHARS` / `find_close_line`.

All commands below run from the repo root `/workspace`. Let `SCRATCH=/tmp/claude-1000/-workspace/1da720a8-e3e9-4c6c-a0e0-eca134729bcd/scratchpad`.

---

## Per-file split recipe (R1–R7)

Every file-split task (Tasks 1–6) executes these seven steps with the task's parameter row. Parameters: **REL** (facade path), **KEY** (spec/group-file key), **SUBDIR** (submodule dir), **MODPATH** (cargo `--list` module prefix), **NTESTS** (expected count), **NGROUPS** (number of non-empty groups = `#[path]` decls).

- [ ] **R1 — capture this file's pre-split bare-fn multiset.**

```bash
mkdir -p "$SCRATCH/orig" "$SCRATCH/list"
cargo test -p kali_runtime --lib -- --list 2>/dev/null | grep ': test$' | sed -E 's/: test$//' \
  | grep "MODPATH::" | sed -E 's/^.*:://' | sort > "$SCRATCH/list/before-KEY.txt"
wc -l "$SCRATCH/list/before-KEY.txt"   # MUST equal NTESTS
cp REL "$SCRATCH/orig/KEY.rs"          # original, for byte-identity proof after in-place rewrite
```

- [ ] **R2 — run the verbatim mover.**

```bash
( cd crates/kali_runtime && python3 /workspace/.superpowers/sdd/move_fns.py "REL_CRATE" "$(cat /workspace/.superpowers/sdd/groups_runtime/KEY.spec)" )
```

(`REL_CRATE` is REL with the leading `crates/kali_runtime/` stripped — the mover derives the submodule dir relative to the input file, so it must run from the crate dir.) Expected stdout: one `group: count` line per group (see the task's expected-counts block); the counts sum to NTESTS. A `WARN: no group for <name>` line (exit 2) means the spec is stale — STOP and reconcile against `cargo test --list`.

- [ ] **R3 — assert facade drained.**

```bash
test "$(grep -c '#\[test\]' REL)" -eq 0 && echo FACADE_TESTS_OK
test "$(grep -cE '^[[:space:]]*(pub[[:space:]]+)?(async[[:space:]]+)?fn ' REL)" -eq 0 && echo NO_MODULE_FNS_OK
test "$(grep -c '^#\[path = ' REL)" -eq NGROUPS && echo PATHS_OK
```

- [ ] **R4 — assert every submodule header is exactly `use super::*;`.**

```bash
for f in SUBDIR/*.rs; do
  [ "$(head -1 "$f")" = "use super::*;" ] || echo "BAD HEADER: $f"
done; echo R4_DONE
```

- [ ] **R5 — prove byte-identity (decisive gate).**

```bash
python3 .superpowers/sdd/verify.py "$SCRATCH/orig/KEY.rs" "SUBDIR/*.rs"
```

Expected: `PROOF OK: all NTESTS #[test] bodies byte-identical, name sets equal`.

- [ ] **R6 — build + test + multiset diff.**

```bash
test "$(cargo build -p kali_runtime --tests 2>&1 | grep -c '^warning')" -eq 0 && echo WARN0_OK
cargo test -p kali_runtime --lib 2>&1 | tail -1     # MUST show: 158 passed; 0 failed
cargo test -p kali_runtime --lib -- --list 2>/dev/null | grep ': test$' | sed -E 's/: test$//' \
  | grep "MODPATH::" | sed -E 's/^.*:://' | sort | diff - "$SCRATCH/list/before-KEY.txt" && echo MULTISET_OK
```

All four must hold: 0 warnings, `158 passed; 0 failed`, empty `diff`, `MULTISET_OK`.

> Note on the multiset diff: after the split a `--list` line reads `MODPATH::<group>::<fn>`. The `sed -E 's/^.*:://'` strips everything up to the last `::`, leaving the bare fn name, so the post-split multiset matches the pre-split one (which was `MODPATH::<fn>`). The `grep "MODPATH::"` prefix filter still matches because `MODPATH::<group>::` begins with `MODPATH::`.

- [ ] **R7 — commit.**

```bash
git add -A
git commit -m "COMMIT_MSG"
```

(`COMMIT_MSG` is the per-task commit subject given in each task's parameter table.)

---

## Task 0: Setup & baseline capture

**Files:** none modified (produces scratch baselines + spec files only).

- [ ] **Step 1: Confirm branch and clean tree.**

```bash
git -C /workspace branch --show-current   # refactor/kali_runtime-modularization
git -C /workspace status --porcelain       # empty (spec already committed)
```

- [ ] **Step 2: Generate the six group `.spec` files and confirm the tools exist.**

```bash
ls -l .superpowers/sdd/move_fns.py .superpowers/sdd/verify.py .superpowers/sdd/classify_kali_runtime.py
python3 .superpowers/sdd/classify_kali_runtime.py
ls .superpowers/sdd/groups_runtime/   # summary.spec execute.spec state.spec profiles.spec browser_execute.spec browser_command.spec
```

Expected classifier stdout (the authoritative partition; counts per group must match the per-task expected blocks below) ending with `ALL PARTITIONS CLEAN; total = 141`. The classifier asserts a clean partition (sum + uniqueness) per file; exact membership is then proven invariant by R5/R6, so the classifier rule itself is not a correctness risk.

- [ ] **Step 3: Capture the global baseline.**

```bash
cargo build -p kali_runtime --tests 2>&1 | grep -c '^warning'   # MUST print 0
cargo test  -p kali_runtime --lib   2>&1 | tail -1               # MUST show 158 passed; 0 failed
```

- [ ] **Step 4: Confirm zero `include_*!` macros (no facade pins needed).**

```bash
grep -rn 'include_str!\|include_bytes!\|include!' crates/kali_runtime/src/ || echo "NONE — no pins"
```

- [ ] **Step 5: Write the SDD ledger header** to `.superpowers/sdd/progress.md` (git-ignored scratch): branch, base `318279f1a`, the 6-task order, and the baseline numbers (0 warnings / 158 pass). No commit (scratch is git-ignored).

---

## Task 1: Split `browser/summary_tests.rs` (60)

**Files:**
- Modify (facade, rewritten in place): `crates/kali_runtime/src/browser/summary_tests.rs`
- Create: `crates/kali_runtime/src/browser/summary_tests/{runtime_summary,bundle,requested}.rs`

**Recipe parameters:**

| param | value |
|---|---|
| REL | `crates/kali_runtime/src/browser/summary_tests.rs` |
| REL_CRATE | `src/browser/summary_tests.rs` |
| KEY | `summary` |
| SUBDIR | `crates/kali_runtime/src/browser/summary_tests` |
| MODPATH | `browser::summary::summary_tests` |
| NTESTS | `60` |
| NGROUPS | `3` |
| COMMIT_MSG | `refactor(kali_runtime): split browser/summary_tests.rs into per-fixture test submodules [refactor]` |

**Expected R2 mover stdout (sums to 60):**

```
runtime_summary: 24
bundle: 13
requested: 23
```

- [ ] Run recipe steps **R1–R7** with the parameters above. R3 expects `FACADE_TESTS_OK`, `NO_MODULE_FNS_OK`, `PATHS_OK` (3 `#[path]` decls).

---

## Task 2: Split `execute_tests.rs` (35)

**Files:**
- Modify: `crates/kali_runtime/src/execute_tests.rs`
- Create: `crates/kali_runtime/src/execute_tests/{node_imports,timers,crypto_random,test_runner,host_env}.rs`

**Recipe parameters:**

| param | value |
|---|---|
| REL | `crates/kali_runtime/src/execute_tests.rs` |
| REL_CRATE | `src/execute_tests.rs` |
| KEY | `execute` |
| SUBDIR | `crates/kali_runtime/src/execute_tests` |
| MODPATH | `execute::execute_tests` |
| NTESTS | `35` |
| NGROUPS | `5` |
| COMMIT_MSG | `refactor(kali_runtime): split execute_tests.rs into per-concern test submodules [refactor]` |

**Expected R2 mover stdout (sums to 35):**

```
node_imports: 12
timers: 6
crypto_random: 3
test_runner: 2
host_env: 12
```

- [ ] Run recipe steps **R1–R7**. R3 expects `FACADE_TESTS_OK`, `NO_MODULE_FNS_OK`, `PATHS_OK` (5 `#[path]` decls).

---

## Task 3: Split `browser/execute_tests.rs` (11)

**Files:**
- Modify: `crates/kali_runtime/src/browser/execute_tests.rs`
- Create: `crates/kali_runtime/src/browser/execute_tests/{execution,harness,diagnostic}.rs`

**Recipe parameters:**

| param | value |
|---|---|
| REL | `crates/kali_runtime/src/browser/execute_tests.rs` |
| REL_CRATE | `src/browser/execute_tests.rs` |
| KEY | `browser_execute` |
| SUBDIR | `crates/kali_runtime/src/browser/execute_tests` |
| MODPATH | `browser::execute::execute_tests` |
| NTESTS | `11` |
| NGROUPS | `3` |
| COMMIT_MSG | `refactor(kali_runtime): split browser/execute_tests.rs into per-concern test submodules [refactor]` |

**Expected R2 mover stdout (sums to 11):**

```
execution: 6
harness: 4
diagnostic: 1
```

- [ ] Run recipe steps **R1–R7**. R3 expects `FACADE_TESTS_OK`, `NO_MODULE_FNS_OK`, `PATHS_OK` (3 `#[path]` decls).

---

## Task 4: Split `browser/command_tests.rs` (10)

**Files:**
- Modify: `crates/kali_runtime/src/browser/command_tests.rs`
- Create: `crates/kali_runtime/src/browser/command_tests/{command_parts,split_command,harness_misc}.rs`

**Recipe parameters:**

| param | value |
|---|---|
| REL | `crates/kali_runtime/src/browser/command_tests.rs` |
| REL_CRATE | `src/browser/command_tests.rs` |
| KEY | `browser_command` |
| SUBDIR | `crates/kali_runtime/src/browser/command_tests` |
| MODPATH | `browser::command::command_tests` |
| NTESTS | `10` |
| NGROUPS | `3` |
| COMMIT_MSG | `refactor(kali_runtime): split browser/command_tests.rs into per-concern test submodules [refactor]` |

**Expected R2 mover stdout (sums to 10):**

```
command_parts: 5
split_command: 2
harness_misc: 3
```

- [ ] Run recipe steps **R1–R7**. R3 expects `FACADE_TESTS_OK`, `NO_MODULE_FNS_OK`, `PATHS_OK` (3 `#[path]` decls).

---

## Task 5: Split `state_tests.rs` (13)

**Files:**
- Modify: `crates/kali_runtime/src/state_tests.rs`
- Create: `crates/kali_runtime/src/state_tests/{host_state,summary_parser,thread_exec}.rs`

**Recipe parameters:**

| param | value |
|---|---|
| REL | `crates/kali_runtime/src/state_tests.rs` |
| REL_CRATE | `src/state_tests.rs` |
| KEY | `state` |
| SUBDIR | `crates/kali_runtime/src/state_tests` |
| MODPATH | `state::state_tests` |
| NTESTS | `13` |
| NGROUPS | `3` |
| COMMIT_MSG | `refactor(kali_runtime): split state_tests.rs into per-concern test submodules [refactor]` |

**Expected R2 mover stdout (sums to 13):**

```
host_state: 6
summary_parser: 2
thread_exec: 5
```

- [ ] Run recipe steps **R1–R7**. R3 expects `FACADE_TESTS_OK`, `NO_MODULE_FNS_OK`, `PATHS_OK` (3 `#[path]` decls).

---

## Task 6: Split `profiles_tests.rs` (12)

**Files:**
- Modify: `crates/kali_runtime/src/profiles_tests.rs`
- Create: `crates/kali_runtime/src/profiles_tests/{browser_surface,thread_budget,normalization}.rs`

**Recipe parameters:**

| param | value |
|---|---|
| REL | `crates/kali_runtime/src/profiles_tests.rs` |
| REL_CRATE | `src/profiles_tests.rs` |
| KEY | `profiles` |
| SUBDIR | `crates/kali_runtime/src/profiles_tests` |
| MODPATH | `profiles::profiles_tests` |
| NTESTS | `12` |
| NGROUPS | `3` |
| COMMIT_MSG | `refactor(kali_runtime): split profiles_tests.rs into per-concern test submodules [refactor]` |

**Expected R2 mover stdout (sums to 12):**

```
browser_surface: 3
thread_budget: 3
normalization: 6
```

- [ ] Run recipe steps **R1–R7**. R3 expects `FACADE_TESTS_OK`, `NO_MODULE_FNS_OK`, `PATHS_OK` (3 `#[path]` decls).

---

## Task 7: Finalize — whole-branch review, merge, cleanup

**Files:** none (review + integration).

- [ ] **Step 1: Whole-branch byte-identity re-proof across all 6 files.**

```bash
for KEY in summary execute browser_execute browser_command state profiles; do
  case $KEY in
    summary)         SUB=crates/kali_runtime/src/browser/summary_tests ;;
    execute)         SUB=crates/kali_runtime/src/execute_tests ;;
    browser_execute) SUB=crates/kali_runtime/src/browser/execute_tests ;;
    browser_command) SUB=crates/kali_runtime/src/browser/command_tests ;;
    state)           SUB=crates/kali_runtime/src/state_tests ;;
    profiles)        SUB=crates/kali_runtime/src/profiles_tests ;;
  esac
  python3 .superpowers/sdd/verify.py "$SCRATCH/orig/$KEY.rs" "$SUB/*.rs" || echo "FAIL $KEY"
done
```

Expected: 6 × `PROOF OK` lines, no `FAIL`.

- [ ] **Step 2: Final full build + test on the branch tip.**

```bash
test "$(cargo build -p kali_runtime --tests 2>&1 | grep -c '^warning')" -eq 0 && echo WARN0_OK
cargo test -p kali_runtime --lib 2>&1 | tail -1   # 158 passed; 0 failed
```

- [ ] **Step 3: Whole-branch review (opus).** Dispatch a reviewer over the full base→tip diff (`git diff 318279f1a..HEAD`), checking: only `#[test]` fns moved; bodies byte-identical (line-conservation — every drained line reappears verbatim, only `use super::*;` headers + `#[path] mod` decls added); facades retain their original `use` lines and drain to zero module-level fns; no `pub` widening; no `cargo fmt` reflow; the production-sibling `#[path = "F_tests.rs"] mod F_tests;` decls are untouched. Resolve any finding before merge.

- [ ] **Step 4: ff-merge to local main and delete the branch (NEVER push origin).**

```bash
git -C /workspace checkout main
git -C /workspace merge --ff-only refactor/kali_runtime-modularization
test "$(cargo build -p kali_runtime --tests 2>&1 | grep -c '^warning')" -eq 0 && echo WARN0_OK
cargo test -p kali_runtime --lib 2>&1 | tail -1   # re-verify on merged main: 158 passed; 0 failed
git -C /workspace branch -d refactor/kali_runtime-modularization
```

- [ ] **Step 5: Update the series memory** (`/home/dev/.claude/projects/-workspace/memory/crate-modularization-series.md`): record kali_runtime as the 27th entry — 6 files, 141 tests, all-six drain-to-0 facades, exact-name-set grouping reused unchanged (`summary_tests` partitioned by leading prefix but emitted as exact names), 0 `include_*!` pins, clean literal gates (0 warnings / 158 pass), final local-main commit hash. Note the remaining frontier (kali_codegen).

---

## Self-Review

- **Spec coverage:** all 6 files in the spec's scope table have a task (1–6); facade-drain-to-0 model encoded per-task via the R3 `NO_MODULE_FNS_OK` check; exact-name grouping via the classifier-generated `.spec` files (Task 0 Step 2); 0-pin fact in Task 0 Step 4; all six spec gates (G1 facade-drained → R3; G2 headers → R4; G3 no-new-warnings → R6; G4 multiset → R6; G5 pass/fail → R6; G6 byte-identity → R5 + Task 7 Step 1); local-main-only integration in Task 7 Step 4.
- **Placeholder scan:** no TBD/TODO; every step has exact commands and expected output; recipe steps spelled out once and parameterized (not "similar to Task N"). Exact per-group counts came from a real mover dry-run, not estimates.
- **Type/name consistency:** MODPATH values match the verified `cargo --list` prefixes (`browser::summary::summary_tests`, `execute::execute_tests`, `browser::execute::execute_tests`, `browser::command::command_tests`, `state::state_tests`, `profiles::profiles_tests`); group names match the `.spec` file contents and the mover's spec-order stdout; expected per-group counts sum to each NTESTS (24+13+23=60; 12+6+3+2+12=35; 6+4+1=11; 5+2+3=10; 6+2+5=13; 3+3+6=12); total = 141.
