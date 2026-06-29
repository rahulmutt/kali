# kali_types co-located src test-monolith modularization — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split kali_types's seven co-located `src/*_tests.rs` unit-test monoliths (≥750 lines, 332 `#[test]` fns) into a thin facade + per-concern `#[path] mod` submodules via pure verbatim code-motion, with zero behavior change and byte-identical test bodies.

**Architecture:** For each file `F`, the proven series mover (`.superpowers/sdd/move_fns.py`, exact-name-set variant) relocates each `#[test]` fn verbatim into `src/.../F/<group>.rs` (each headed by exactly `use super::*;`), and rewrites the facade `F.rs` to drop the moved fns and append `#[path = "F/<group>.rs"] mod <group>;` decls. The facade keeps its original header `use` lines (children consume them via descendant-visibility through `use super::*;`); the two files with a module-level `assert_*` helper (`array_tests`, `object_tests`) keep that helper in the facade. Byte-identity is proven per file by `.superpowers/sdd/verify.py`.

**Tech Stack:** Rust (cargo, edition-2021 workspace crate `kali_types`); Python 3 mover/verify tools (git-ignored scratch under `.superpowers/sdd/`).

**Spec:** `docs/superpowers/specs/2026-06-29-kali-types-src-test-monoliths-modularization-design.md`

## Global Constraints

- **Pure relocation.** No new product code, no new tests, no renames, no reordering, no reformatting. `#[test]` attr lines + body + one trailing blank relocate byte-for-byte.
- **Submodule header is exactly `use super::*;`** (nothing else). Facade keeps every original `use` line verbatim. No per-submodule extern `use`s.
- **Facade ends with zero `#[test]` fns.** `array_tests`/`object_tests` facades additionally keep exactly their one module-level `assert_*` helper; the other five drain to zero module-level fns. No `include_*!` pins anywhere (verified 0 across the crate).
- **No `pub`/`pub(crate)` widening** — intra-crate child modules reach parent scope via `use super::*`.
- **Do NOT run `cargo fmt`** — the repo `cargo fmt --all --check` gate is already red on baseline; accepted cosmetic minors are not regressions.
- **The `#[path = "F_tests.rs"] mod F_tests;` decls in each production sibling stay unchanged** — they still name the facade file, which now re-exports its children.
- **Baseline (clean, captured at Task 0):** `cargo build -p kali_types --tests` = **0 warnings**; `cargo test -p kali_types --lib` = **372 passed / 0 failed**. Gates are literal here (no env carve-outs): **0 warnings unchanged**, **372 pass / 0 fail unchanged**, per-file `--list` bare-fn multiset preserved.
- **Integration: local-main ff-merge only — NEVER push to origin** (origin/main intentionally lags at `77704c7e7`). Re-verify on merged main, then delete the branch.
- **Branch:** `refactor/kali_types-modularization` (already created off `e0a3416ef`). SDD ledger at `.superpowers/sdd/progress.md` (git-ignored scratch), overwrite per task.
- **Tools are reused as-is** from the kali_optimize entry: `.superpowers/sdd/move_fns.py` (267 lines, exact-name-set grouping) and `.superpowers/sdd/verify.py` (59 lines). Do NOT edit `FN_RE` / `IDENT_CHARS` / `find_close_line`.

All commands below run from the repo root `/workspace`. Let `SCRATCH=/tmp/claude-1000/-workspace/27b67863-d6d8-4443-bcf1-e5b9dc6916c5/scratchpad`.

---

## Per-file split recipe (R1–R7)

Every file-split task (Tasks 1–7) executes these seven steps with the task's parameter row. Parameters: **REL** (facade path), **KEY** (spec/group-file key), **SUBDIR** (submodule dir), **MODPATH** (cargo `--list` module prefix), **NTESTS** (expected count), **HELPER** (retained module-level `assert_*` count: 1 or 0).

- [ ] **R1 — capture this file's pre-split bare-fn multiset.**

```bash
mkdir -p "$SCRATCH/orig" "$SCRATCH/list"
cargo test -p kali_types --lib -- --list 2>/dev/null | grep ': test$' | sed -E 's/: test$//' \
  | grep "MODPATH::" | sed -E 's/^.*:://' | sort > "$SCRATCH/list/before-KEY.txt"
wc -l "$SCRATCH/list/before-KEY.txt"   # MUST equal NTESTS
cp REL "$SCRATCH/orig/KEY.rs"          # original, for byte-identity proof after in-place rewrite
```

- [ ] **R2 — run the verbatim mover.**

```bash
python3 .superpowers/sdd/move_fns.py REL "$(cat .superpowers/sdd/groups/KEY.spec)"
```

Expected stdout: one `group: count` line per group (see the task's expected-counts block); the counts sum to NTESTS. A `WARN: no group for <name>` line (exit 2) means the spec is stale — STOP and reconcile against `cargo test --list`.

- [ ] **R3 — assert facade drained.**

```bash
test "$(grep -c '#\[test\]' REL)" -eq 0 && echo FACADE_TESTS_OK
test "$(grep -cE '^[[:space:]]*fn assert_' REL)" -eq HELPER && echo HELPER_OK
grep -c '^#\[path = ' REL   # == number of non-empty groups for this file
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

Expected: `PROOF OK: all NTESTS #[test] bodies byte-identical, name sets equal`. (verify.py extracts only `#[test]` fns, so the retained `assert_*` helper is correctly ignored.)

- [ ] **R6 — build + test + multiset diff.**

```bash
test "$(cargo build -p kali_types --tests 2>&1 | grep -c '^warning')" -eq 0 && echo WARN0_OK
cargo test -p kali_types --lib 2>&1 | tail -1     # MUST show: 372 passed; 0 failed
cargo test -p kali_types --lib -- --list 2>/dev/null | grep ': test$' | sed -E 's/: test$//' \
  | grep "MODPATH::" | sed -E 's/^.*:://' | sort | diff - "$SCRATCH/list/before-KEY.txt" && echo MULTISET_OK
```

All four must hold: 0 warnings, `372 passed; 0 failed`, empty `diff`, `MULTISET_OK`.

- [ ] **R7 — commit.**

```bash
git add -A
git commit -m "refactor(kali_types): split KEY_tests.rs into per-concern test submodules [refactor]"
```

---

## Task 0: Setup & baseline capture

**Files:** none modified (produces scratch baselines only).

- [ ] **Step 1: Confirm branch and clean tree.**

```bash
git -C /workspace branch --show-current   # refactor/kali_types-modularization
git -C /workspace status --porcelain       # empty
```

- [ ] **Step 2: Confirm the mover/verify tools and the 7 spec files exist.**

```bash
ls -l .superpowers/sdd/move_fns.py .superpowers/sdd/verify.py
ls .superpowers/sdd/groups/   # array.spec object.spec math.spec late_host.spec expression.spec string.spec function.spec
```

If any `groups/*.spec` is missing, regenerate all seven deterministically (the classifier is pinned at `.superpowers/sdd/classify_kali_types.py`; it asserts a clean partition and re-emits `specs.json`, then split it into the per-key `.spec` files). The classifier's grouping rule only needs to *partition*; exact membership is then proven invariant by R5/R6, so the rule itself is not a correctness risk.

- [ ] **Step 3: Capture the global baseline.**

```bash
cargo build -p kali_types --tests 2>&1 | grep -c '^warning'   # MUST print 0
cargo test  -p kali_types --lib   2>&1 | tail -1               # MUST show 372 passed; 0 failed
```

- [ ] **Step 4: Confirm zero `include_*!` macros (no facade pins needed).**

```bash
grep -rn 'include_str!\|include_bytes!\|include!' crates/kali_types/src/ || echo "NONE — no pins"
```

- [ ] **Step 5: Write the SDD ledger header** to `.superpowers/sdd/progress.md` (git-ignored scratch): branch, base `e0a3416ef`, the 7-task order, and the baseline numbers (0 warnings / 372 pass). No commit (scratch is git-ignored).

---

## Task 1: Split `array_tests.rs` (91)

**Files:**
- Modify (facade, rewritten in place): `crates/kali_types/src/static_analysis/array_tests.rs`
- Create: `crates/kali_types/src/static_analysis/array_tests/{set_map_targets,array_from,for_of,for_await,methods}.rs`

**Recipe parameters:**

| param | value |
|---|---|
| REL | `crates/kali_types/src/static_analysis/array_tests.rs` |
| KEY | `array` |
| SUBDIR | `crates/kali_types/src/static_analysis/array_tests` |
| MODPATH | `static_analysis::array::array_tests` |
| NTESTS | `91` |
| HELPER | `1` (`assert_resolution_accepts_frozen_iterator_protocol_edge` stays in facade) |

**Expected R2 mover stdout (sums to 91):**

```
set_map_targets: 7
array_from: 33
for_of: 18
for_await: 12
methods: 21
```

- [ ] Run recipe steps **R1–R7** with the parameters above. R3 expects `FACADE_TESTS_OK`, `HELPER_OK`, and `5` `#[path]` decls.

---

## Task 2: Split `object_tests.rs` (52)

**Files:**
- Modify: `crates/kali_types/src/static_analysis/object_tests.rs`
- Create: `crates/kali_types/src/static_analysis/object_tests/{object_is,has_own_entries,enumeration,freeze_late_model}.rs`

**Recipe parameters:**

| param | value |
|---|---|
| REL | `crates/kali_types/src/static_analysis/object_tests.rs` |
| KEY | `object` |
| SUBDIR | `crates/kali_types/src/static_analysis/object_tests` |
| MODPATH | `static_analysis::object::object_tests` |
| NTESTS | `52` |
| HELPER | `1` (`assert_object_helper_iteration_with_let_binding_in_js_input` stays in facade) |

**Expected R2 mover stdout (sums to 52):**

```
object_is: 15
has_own_entries: 8
enumeration: 10
freeze_late_model: 19
```

- [ ] Run recipe steps **R1–R7**. R3 expects `FACADE_TESTS_OK`, `HELPER_OK`, and `4` `#[path]` decls.

---

## Task 3: Split `math_tests.rs` (65)

**Files:**
- Modify: `crates/kali_types/src/static_analysis/math_tests.rs`
- Create: `crates/kali_types/src/static_analysis/math_tests/{pow,transcendental,rounding,wrappers}.rs`

**Recipe parameters:**

| param | value |
|---|---|
| REL | `crates/kali_types/src/static_analysis/math_tests.rs` |
| KEY | `math` |
| SUBDIR | `crates/kali_types/src/static_analysis/math_tests` |
| MODPATH | `static_analysis::math::math_tests` |
| NTESTS | `65` |
| HELPER | `0` |

**Expected R2 mover stdout (sums to 65):**

```
pow: 11
transcendental: 37
rounding: 10
wrappers: 7
```

- [ ] Run recipe steps **R1–R7**. R3 expects `FACADE_TESTS_OK`, `HELPER_OK` (0 helpers), and `4` `#[path]` decls.

---

## Task 4: Split `late_host_tests.rs` (39)

**Files:**
- Modify: `crates/kali_types/src/late_host_tests.rs`
- Create: `crates/kali_types/src/late_host_tests/{globals,process_env,permissions,intl_imports_kill}.rs`

**Recipe parameters:**

| param | value |
|---|---|
| REL | `crates/kali_types/src/late_host_tests.rs` |
| KEY | `late_host` |
| SUBDIR | `crates/kali_types/src/late_host_tests` |
| MODPATH | `late_host::late_host_tests` |
| NTESTS | `39` |
| HELPER | `0` (the `fn member`/`fn const_descriptor`/`fn permission_query` are nested inside test bodies and move with their parent test) |

**Expected R2 mover stdout (sums to 39):**

```
globals: 10
process_env: 12
permissions: 6
intl_imports_kill: 11
```

- [ ] Run recipe steps **R1–R7**. R3 expects `FACADE_TESTS_OK`, `HELPER_OK` (0 module-level helpers), and `4` `#[path]` decls.

---

## Task 5: Split `expression_tests.rs` (42)

**Files:**
- Modify: `crates/kali_types/src/resolve/expression_tests.rs`
- Create: `crates/kali_types/src/resolve/expression_tests/{exports,operators,dynamic_import}.rs`

**Recipe parameters:**

| param | value |
|---|---|
| REL | `crates/kali_types/src/resolve/expression_tests.rs` |
| KEY | `expression` |
| SUBDIR | `crates/kali_types/src/resolve/expression_tests` |
| MODPATH | `resolve::expression::expression_tests` |
| NTESTS | `42` |
| HELPER | `0` |

**Expected R2 mover stdout (sums to 42):**

```
exports: 16
operators: 8
dynamic_import: 18
```

- [ ] Run recipe steps **R1–R7**. R3 expects `FACADE_TESTS_OK`, `HELPER_OK` (0 helpers), and `3` `#[path]` decls.

---

## Task 6: Split `string_tests.rs` (30)

**Files:**
- Modify: `crates/kali_types/src/static_analysis/string_tests.rs`
- Create: `crates/kali_types/src/static_analysis/string_tests/{iteration,methods}.rs`

**Recipe parameters:**

| param | value |
|---|---|
| REL | `crates/kali_types/src/static_analysis/string_tests.rs` |
| KEY | `string` |
| SUBDIR | `crates/kali_types/src/static_analysis/string_tests` |
| MODPATH | `static_analysis::string::string_tests` |
| NTESTS | `30` |
| HELPER | `0` |

**Expected R2 mover stdout (sums to 30):**

```
iteration: 6
methods: 24
```

- [ ] Run recipe steps **R1–R7**. R3 expects `FACADE_TESTS_OK`, `HELPER_OK` (0 helpers), and `2` `#[path]` decls.

---

## Task 7: Split `function_tests.rs` (13)

**Files:**
- Modify: `crates/kali_types/src/resolve/function_tests.rs`
- Create: `crates/kali_types/src/resolve/function_tests/{generator_functions,class_methods}.rs`

**Recipe parameters:**

| param | value |
|---|---|
| REL | `crates/kali_types/src/resolve/function_tests.rs` |
| KEY | `function` |
| SUBDIR | `crates/kali_types/src/resolve/function_tests` |
| MODPATH | `resolve::function::function_tests` |
| NTESTS | `13` |
| HELPER | `0` |

**Expected R2 mover stdout (sums to 13):**

```
generator_functions: 7
class_methods: 6
```

- [ ] Run recipe steps **R1–R7**. R3 expects `FACADE_TESTS_OK`, `HELPER_OK` (0 helpers), and `2` `#[path]` decls.

---

## Task 8: Finalize — whole-branch review, merge, cleanup

**Files:** none (review + integration).

- [ ] **Step 1: Whole-branch byte-identity re-proof across all 7 files.**

```bash
for KEY in array object math late_host expression string function; do
  case $KEY in
    array)      SUB=crates/kali_types/src/static_analysis/array_tests ;;
    object)     SUB=crates/kali_types/src/static_analysis/object_tests ;;
    math)       SUB=crates/kali_types/src/static_analysis/math_tests ;;
    late_host)  SUB=crates/kali_types/src/late_host_tests ;;
    expression) SUB=crates/kali_types/src/resolve/expression_tests ;;
    string)     SUB=crates/kali_types/src/static_analysis/string_tests ;;
    function)   SUB=crates/kali_types/src/resolve/function_tests ;;
  esac
  python3 .superpowers/sdd/verify.py "$SCRATCH/orig/$KEY.rs" "$SUB/*.rs" || echo "FAIL $KEY"
done
```

Expected: 7 × `PROOF OK` lines, no `FAIL`.

- [ ] **Step 2: Final full build + test on the branch tip.**

```bash
test "$(cargo build -p kali_types --tests 2>&1 | grep -c '^warning')" -eq 0 && echo WARN0_OK
cargo test -p kali_types --lib 2>&1 | tail -1   # 372 passed; 0 failed
```

- [ ] **Step 3: Whole-branch review (opus).** Dispatch a reviewer over the full base→tip diff (`git diff e0a3416ef..HEAD`), checking: only `#[test]` fns moved; bodies byte-identical (line-conservation — every drained line reappears verbatim, only `use super::*;` headers + `#[path] mod` decls added); facades retain their original `use` lines and (array/object only) their one `assert_*` helper; no `pub` widening; no `cargo fmt` reflow. Resolve any finding before merge.

- [ ] **Step 4: ff-merge to local main and delete the branch (NEVER push origin).**

```bash
git -C /workspace checkout main
git -C /workspace merge --ff-only refactor/kali_types-modularization
test "$(cargo build -p kali_types --tests 2>&1 | grep -c '^warning')" -eq 0 && echo WARN0_OK
cargo test -p kali_types --lib 2>&1 | tail -1   # re-verify on merged main: 372 passed; 0 failed
git -C /workspace branch -d refactor/kali_types-modularization
```

- [ ] **Step 5: Update the series memory** (`/home/dev/.claude/projects/-workspace/memory/crate-modularization-series.md`): record kali_types as the 26th entry — 7 files, 332 tests, 5 drain-to-0 + 2 retain-helper facades, exact-name-set grouping reused unchanged, 0 `include_*!` pins, clean literal gates (0 warnings / 372 pass), final local-main commit hash. Note the remaining frontier (kali_runtime, kali_codegen).

---

## Self-Review

- **Spec coverage:** all 7 files in the spec's scope table have a task (1–7); facade-drain model (5 drain / 2 retain-helper) encoded per-task via HELPER; exact-name grouping via the verified `.spec` files; 0-pin fact in Task 0 Step 4; all six spec gates (G1 facade-drained → R3; G2 headers → R4; G3 no-new-warnings → R6; G4 multiset → R6; G5 pass/fail → R6; G6 byte-identity → R5 + Task 8 Step 1); local-main-only integration in Task 8 Step 4.
- **Placeholder scan:** no TBD/TODO; every step has exact commands and expected output; recipe steps spelled out once and parameterized (not "similar to Task N").
- **Type/name consistency:** MODPATH values match the verified `cargo --list` prefixes; group names match the `.spec` file contents and the mover's spec-order stdout; expected per-group counts sum to each NTESTS (7+33+18+12+21=91; 15+8+10+19=52; 11+37+10+7=65; 10+12+6+11=39; 16+8+18=42; 6+24=30; 7+6=13).
</content>
