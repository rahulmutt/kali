# kali_codegen co-located src test-monolith modularization — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split eight of kali_codegen's co-located `src/*_tests.rs` unit-test monoliths (308 `#[test]` fns) into a thin facade + per-concern `#[path] mod` submodules via pure verbatim code-motion, with zero behavior change and byte-identical test bodies.

**Architecture:** For each file `F`, the proven series mover (`.superpowers/sdd/move_fns.py`, exact-name-set variant) relocates each `#[test]` fn verbatim into `src/.../F/<group>.rs` (each headed by exactly `use super::*;`), and rewrites the facade `F.rs` to drop the moved fns and append `#[path = "F/<group>.rs"] mod <group>;` decls. The facade keeps its original header `use` lines (children consume them via descendant-visibility through `use super::*;`). Seven facades drain to **zero** module-level fns; `control_flow_tests` retains exactly one non-`#[test]` helper (`legacy_phase1_baseline`). Byte-identity is proven per file by `.superpowers/sdd/verify.py`.

**Tech Stack:** Rust (cargo, workspace crate `kali_codegen`); Python 3 mover/verify/classify tools (git-ignored scratch under `.superpowers/sdd/`).

**Spec:** `docs/superpowers/specs/2026-06-29-kali-codegen-src-test-monoliths-modularization-design.md`

## Global Constraints

- **Pure relocation.** No new product code, no new tests, no renames, no reordering, no reformatting. `#[test]` attr lines + body + one trailing blank relocate byte-for-byte.
- **Submodule header is exactly `use super::*;`** (nothing else). Facade keeps every original `use` line verbatim. No per-submodule extern `use`s.
- **Seven facades end with zero `#[test]` fns and zero module-level helpers.** `control_flow_tests` ends with zero `#[test]` fns and retains exactly one module-level fn, the non-`#[test]` helper `legacy_phase1_baseline` (the mover leaves non-`#[test]` fns in place automatically; no pin arg needed). No `include_*!` pins anywhere (verified 0 across the in-scope files).
- **No `pub`/`pub(crate)` widening** — intra-crate child modules reach parent scope (including the retained `legacy_phase1_baseline` helper) via `use super::*`.
- **Do NOT run `cargo fmt`** — the repo `cargo fmt --all --check` gate is already red on baseline; accepted cosmetic minors are not regressions.
- **The `#[cfg(test)]` + `#[path = "F_tests.rs"]` + `mod F_tests;` decls in each production sibling stay unchanged** — they still name the facade file, which now re-exports its children.
- **Baseline (clean, captured at Task 0):** `cargo build -p kali_codegen --tests` = **0 warnings**; `cargo test -p kali_codegen --lib` = **325 passed / 0 failed** (308 in-scope tests + 17 in the three out-of-scope files: `number_tests` 3, `literal_tests` 6, `operators_tests` 8). Gates are literal here (no env carve-outs): **0 warnings unchanged**, **325 pass / 0 fail unchanged**, per-file `--list` bare-fn multiset preserved.
- **Integration: local-main ff-merge only — NEVER push to origin.** (origin/main currently equals HEAD `138aaa1a7` from external syncing of prior work, but the standing local-only convention holds.) Re-verify on merged main, then delete the branch.
- **Branch:** `refactor/kali_codegen-modularization` (already created off `138aaa1a7`; the design spec is already committed on it). SDD ledger at `.superpowers/sdd/progress.md` (git-ignored scratch), overwrite per task.
- **Tools are reused as-is** from the kali_runtime/kali_types entries: `.superpowers/sdd/move_fns.py` (exact-name-set grouping) and `.superpowers/sdd/verify.py`. Do NOT edit `FN_RE` / `IDENT_CHARS` / `find_close_line`.

All commands below run from the repo root `/workspace`. Let `SCRATCH=/tmp/claude-1000/-workspace/ac867b03-612e-48bd-afbc-f23854a4bd93/scratchpad`.

---

## Per-file split recipe (R1–R7)

Every file-split task (Tasks 1–8) executes these seven steps with the task's parameter row. Parameters: **REL** (facade path), **REL_CRATE** (REL minus the `crates/kali_codegen/` prefix), **KEY** (spec/group-file key), **SUBDIR** (submodule dir), **MODPATH** (cargo `--list` module prefix), **NTESTS** (expected count), **NGROUPS** (number of non-empty groups = `#[path]` decls), **NMODFNS** (module-level fns remaining in the facade after the split: `0` for all files except `control_flow`, which is `1`).

- [ ] **R1 — capture this file's pre-split bare-fn multiset.**

```bash
mkdir -p "$SCRATCH/orig" "$SCRATCH/list"
cargo test -p kali_codegen --lib -- --list 2>/dev/null | grep ': test$' | sed -E 's/: test$//' \
  | grep "^MODPATH::" | sed -E 's/^.*:://' | sort > "$SCRATCH/list/before-KEY.txt"
wc -l "$SCRATCH/list/before-KEY.txt"   # MUST equal NTESTS
cp REL "$SCRATCH/orig/KEY.rs"          # original, for byte-identity proof after in-place rewrite
```

- [ ] **R2 — run the verbatim mover.**

```bash
( cd crates/kali_codegen && python3 /workspace/.superpowers/sdd/move_fns.py "REL_CRATE" "$(cat /workspace/.superpowers/sdd/groups_codegen/KEY.spec)" )
```

(The mover derives the submodule dir relative to the input file, so it must run from the crate dir.) Expected stdout: one `group: count` line per non-empty group, in spec order (see the task's expected-counts block); the counts sum to NTESTS. A `WARN: no group for <name>` line (exit 2) means the spec is stale — STOP and reconcile against `cargo test --list`.

- [ ] **R3 — assert facade drained.**

```bash
test "$(grep -c '#\[test\]' REL)" -eq 0 && echo FACADE_TESTS_OK
test "$(grep -cE '^[[:space:]]*(pub[[:space:]]+)?(async[[:space:]]+)?fn ' REL)" -eq NMODFNS && echo MODFNS_OK
test "$(grep -c '^#\[path = ' REL)" -eq NGROUPS && echo PATHS_OK
```

`FACADE_TESTS_OK` requires zero `#[test]` in the facade for every file. `MODFNS_OK` requires NMODFNS module-level fns (`0` for seven files; `1` for `control_flow`, the retained `legacy_phase1_baseline`). `PATHS_OK` requires NGROUPS `#[path]` decls.

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

Expected: `PROOF OK: all NTESTS #[test] bodies byte-identical, name sets equal`. (`verify.py` compares only `#[test]` bodies, so `control_flow`'s retained non-`#[test]` helper is excluded from both sides — no facade glob needed.)

- [ ] **R6 — build + test + multiset diff.**

```bash
test "$(cargo build -p kali_codegen --tests 2>&1 | grep -c '^warning')" -eq 0 && echo WARN0_OK
cargo test -p kali_codegen --lib 2>&1 | tail -1     # MUST show: 325 passed; 0 failed
cargo test -p kali_codegen --lib -- --list 2>/dev/null | grep ': test$' | sed -E 's/: test$//' \
  | grep "^MODPATH::" | sed -E 's/^.*:://' | sort | diff - "$SCRATCH/list/before-KEY.txt" && echo MULTISET_OK
```

All four must hold: 0 warnings, `325 passed; 0 failed`, empty `diff`, `MULTISET_OK`.

> Note on the multiset diff: after the split a `--list` line reads `MODPATH::<group>::<fn>`. The `sed -E 's/^.*:://'` strips everything up to the last `::`, leaving the bare fn name, so the post-split multiset matches the pre-split one (which was `MODPATH::<fn>`). The `grep "^MODPATH::"` prefix filter is **anchored with `^`** (kali_runtime hit a suffix-substring over-count when one module path was a substring of another); kali_codegen has no such collision in the in-scope set, but anchor as a precaution.

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
git -C /workspace branch --show-current   # refactor/kali_codegen-modularization
git -C /workspace status --porcelain       # empty (spec already committed)
```

- [ ] **Step 2: Generate the eight group `.spec` files and confirm the tools exist.**

```bash
ls -l .superpowers/sdd/move_fns.py .superpowers/sdd/verify.py .superpowers/sdd/classify_kali_codegen.py
python3 .superpowers/sdd/classify_kali_codegen.py
ls .superpowers/sdd/groups_codegen/   # math.spec call.spec host.spec array.spec control_flow.spec object.spec string.spec collections.spec
```

Expected classifier stdout (the authoritative partition; counts per group must match the per-task expected blocks below) ending with `ALL PARTITIONS CLEAN; total = 308`. The classifier asserts a clean partition (per-group sum + uniqueness + the embedded expected-count table) per file; exact membership is then proven invariant by R5/R6, so the classifier rule itself is not a correctness risk.

- [ ] **Step 3: Capture the global baseline.**

```bash
cargo build -p kali_codegen --tests 2>&1 | grep -c '^warning'   # MUST print 0
cargo test  -p kali_codegen --lib   2>&1 | tail -1               # MUST show 325 passed; 0 failed
```

- [ ] **Step 4: Confirm zero `include_*!` macros in the in-scope files (no facade pins needed).**

```bash
grep -rn 'include_str!\|include_bytes!\|include!' \
  crates/kali_codegen/src/intrinsics/{math,host,array,object,string,collections}_tests.rs \
  crates/kali_codegen/src/emit/{call,control_flow}_tests.rs || echo "NONE — no pins"
```

- [ ] **Step 5: Confirm the one retained helper exists in `control_flow_tests` (and is non-`#[test]`).**

```bash
grep -n 'fn legacy_phase1_baseline' crates/kali_codegen/src/emit/control_flow_tests.rs   # exactly 1 hit, line ~455
grep -cE '^fn ' crates/kali_codegen/src/emit/control_flow_tests.rs                        # 23 (22 #[test] + 1 helper) pre-split
```

- [ ] **Step 6: Write the SDD ledger header** to `.superpowers/sdd/progress.md` (git-ignored scratch): branch, base `138aaa1a7`, the 8-task order, the baseline numbers (0 warnings / 325 pass), and the one-helper-retention note for `control_flow`. No commit (scratch is git-ignored).

---

## Task 1: Split `intrinsics/math_tests.rs` (97)

**Files:**
- Modify (facade, rewritten in place): `crates/kali_codegen/src/intrinsics/math_tests.rs`
- Create: `crates/kali_codegen/src/intrinsics/math_tests/{pow,rounding,integer_ops,transcendental}.rs`

**Recipe parameters:**

| param | value |
|---|---|
| REL | `crates/kali_codegen/src/intrinsics/math_tests.rs` |
| REL_CRATE | `src/intrinsics/math_tests.rs` |
| KEY | `math` |
| SUBDIR | `crates/kali_codegen/src/intrinsics/math_tests` |
| MODPATH | `intrinsics::math::math_tests` |
| NTESTS | `97` |
| NGROUPS | `4` |
| NMODFNS | `0` |
| COMMIT_MSG | `refactor(kali_codegen): split intrinsics/math_tests.rs into per-operation test submodules [refactor]` |

**Expected R2 mover stdout (sums to 97):**

```
pow: 16
rounding: 18
integer_ops: 22
transcendental: 41
```

- [ ] Run recipe steps **R1–R7** with the parameters above. R3 expects `FACADE_TESTS_OK`, `MODFNS_OK` (0), `PATHS_OK` (4 `#[path]` decls).

---

## Task 2: Split `emit/call_tests.rs` (88)

**Files:**
- Modify: `crates/kali_codegen/src/emit/call_tests.rs`
- Create: `crates/kali_codegen/src/emit/call_tests/{reflect_own_keys,diagnostics,array_iteration,object_enumeration}.rs`

**Recipe parameters:**

| param | value |
|---|---|
| REL | `crates/kali_codegen/src/emit/call_tests.rs` |
| REL_CRATE | `src/emit/call_tests.rs` |
| KEY | `call` |
| SUBDIR | `crates/kali_codegen/src/emit/call_tests` |
| MODPATH | `emit::call::call_tests` |
| NTESTS | `88` |
| NGROUPS | `4` |
| NMODFNS | `0` |
| COMMIT_MSG | `refactor(kali_codegen): split emit/call_tests.rs into per-concern test submodules [refactor]` |

**Expected R2 mover stdout (sums to 88):**

```
reflect_own_keys: 9
diagnostics: 4
array_iteration: 44
object_enumeration: 31
```

- [ ] Run recipe steps **R1–R7**. R3 expects `FACADE_TESTS_OK`, `MODFNS_OK` (0), `PATHS_OK` (4 `#[path]` decls).

---

## Task 3: Split `intrinsics/host_tests.rs` (30)

**Files:**
- Modify: `crates/kali_codegen/src/intrinsics/host_tests.rs`
- Create: `crates/kali_codegen/src/intrinsics/host_tests/{env,console,deno,process}.rs`

**Recipe parameters:**

| param | value |
|---|---|
| REL | `crates/kali_codegen/src/intrinsics/host_tests.rs` |
| REL_CRATE | `src/intrinsics/host_tests.rs` |
| KEY | `host` |
| SUBDIR | `crates/kali_codegen/src/intrinsics/host_tests` |
| MODPATH | `intrinsics::host::host_tests` |
| NTESTS | `30` |
| NGROUPS | `4` |
| NMODFNS | `0` |
| COMMIT_MSG | `refactor(kali_codegen): split intrinsics/host_tests.rs into per-surface test submodules [refactor]` |

**Expected R2 mover stdout (sums to 30):**

```
env: 6
console: 2
deno: 7
process: 15
```

- [ ] Run recipe steps **R1–R7**. R3 expects `FACADE_TESTS_OK`, `MODFNS_OK` (0), `PATHS_OK` (4 `#[path]` decls).

---

## Task 4: Split `intrinsics/array_tests.rs` (23)

**Files:**
- Modify: `crates/kali_codegen/src/intrinsics/array_tests.rs`
- Create: `crates/kali_codegen/src/intrinsics/array_tests/{callbacks,static_ops}.rs`

**Recipe parameters:**

| param | value |
|---|---|
| REL | `crates/kali_codegen/src/intrinsics/array_tests.rs` |
| REL_CRATE | `src/intrinsics/array_tests.rs` |
| KEY | `array` |
| SUBDIR | `crates/kali_codegen/src/intrinsics/array_tests` |
| MODPATH | `intrinsics::array::array_tests` |
| NTESTS | `23` |
| NGROUPS | `2` |
| NMODFNS | `0` |
| COMMIT_MSG | `refactor(kali_codegen): split intrinsics/array_tests.rs into per-concern test submodules [refactor]` |

**Expected R2 mover stdout (sums to 23):**

```
callbacks: 12
static_ops: 11
```

- [ ] Run recipe steps **R1–R7**. R3 expects `FACADE_TESTS_OK`, `MODFNS_OK` (0), `PATHS_OK` (2 `#[path]` decls).

---

## Task 5: Split `emit/control_flow_tests.rs` (22, retains 1 helper)

**Files:**
- Modify: `crates/kali_codegen/src/emit/control_flow_tests.rs` (retains the non-`#[test]` helper `legacy_phase1_baseline`)
- Create: `crates/kali_codegen/src/emit/control_flow_tests/{function_plans,unsupported_generators,pipeline_basics}.rs`

**Recipe parameters:**

| param | value |
|---|---|
| REL | `crates/kali_codegen/src/emit/control_flow_tests.rs` |
| REL_CRATE | `src/emit/control_flow_tests.rs` |
| KEY | `control_flow` |
| SUBDIR | `crates/kali_codegen/src/emit/control_flow_tests` |
| MODPATH | `emit::control_flow::control_flow_tests` |
| NTESTS | `22` |
| NGROUPS | `3` |
| NMODFNS | `1` |
| COMMIT_MSG | `refactor(kali_codegen): split emit/control_flow_tests.rs into per-concern test submodules [refactor]` |

**Expected R2 mover stdout (sums to 22):**

```
function_plans: 9
unsupported_generators: 10
pipeline_basics: 3
```

- [ ] Run recipe steps **R1–R7**. **R3 expects `MODFNS_OK` == 1** (the retained `legacy_phase1_baseline` helper) — the only file in this plan where the facade keeps a module-level fn; `FACADE_TESTS_OK` (0 `#[test]`) and `PATHS_OK` (3 `#[path]` decls) hold as usual.
- [ ] **Extra check after R3:** confirm the helper survived and the `pipeline_basics` consumer reaches it:

```bash
grep -c 'fn legacy_phase1_baseline' crates/kali_codegen/src/emit/control_flow_tests.rs   # 1 (in facade)
grep -l 'legacy_phase1_baseline' crates/kali_codegen/src/emit/control_flow_tests/pipeline_basics.rs   # the caller moved here
```

R6's `WARN0_OK` is the decisive proof that the helper resolves through `use super::*;` (descendant-visibility) with no unused-fn / unresolved-name warning.

---

## Task 6: Split `intrinsics/object_tests.rs` (18)

**Files:**
- Modify: `crates/kali_codegen/src/intrinsics/object_tests.rs`
- Create: `crates/kali_codegen/src/intrinsics/object_tests/{is,has_own}.rs`

**Recipe parameters:**

| param | value |
|---|---|
| REL | `crates/kali_codegen/src/intrinsics/object_tests.rs` |
| REL_CRATE | `src/intrinsics/object_tests.rs` |
| KEY | `object` |
| SUBDIR | `crates/kali_codegen/src/intrinsics/object_tests` |
| MODPATH | `intrinsics::object::object_tests` |
| NTESTS | `18` |
| NGROUPS | `2` |
| NMODFNS | `0` |
| COMMIT_MSG | `refactor(kali_codegen): split intrinsics/object_tests.rs into per-operation test submodules [refactor]` |

**Expected R2 mover stdout (sums to 18):**

```
is: 9
has_own: 9
```

- [ ] Run recipe steps **R1–R7**. R3 expects `FACADE_TESTS_OK`, `MODFNS_OK` (0), `PATHS_OK` (2 `#[path]` decls).

---

## Task 7: Split `intrinsics/string_tests.rs` (16)

**Files:**
- Modify: `crates/kali_codegen/src/intrinsics/string_tests.rs`
- Create: `crates/kali_codegen/src/intrinsics/string_tests/{transform,lookup}.rs`

**Recipe parameters:**

| param | value |
|---|---|
| REL | `crates/kali_codegen/src/intrinsics/string_tests.rs` |
| REL_CRATE | `src/intrinsics/string_tests.rs` |
| KEY | `string` |
| SUBDIR | `crates/kali_codegen/src/intrinsics/string_tests` |
| MODPATH | `intrinsics::string::string_tests` |
| NTESTS | `16` |
| NGROUPS | `2` |
| NMODFNS | `0` |
| COMMIT_MSG | `refactor(kali_codegen): split intrinsics/string_tests.rs into per-concern test submodules [refactor]` |

**Expected R2 mover stdout (sums to 16):**

```
transform: 7
lookup: 9
```

- [ ] Run recipe steps **R1–R7**. R3 expects `FACADE_TESTS_OK`, `MODFNS_OK` (0), `PATHS_OK` (2 `#[path]` decls).

---

## Task 8: Split `intrinsics/collections_tests.rs` (14)

**Files:**
- Modify: `crates/kali_codegen/src/intrinsics/collections_tests.rs`
- Create: `crates/kali_codegen/src/intrinsics/collections_tests/{combined,set,map}.rs`

**Recipe parameters:**

| param | value |
|---|---|
| REL | `crates/kali_codegen/src/intrinsics/collections_tests.rs` |
| REL_CRATE | `src/intrinsics/collections_tests.rs` |
| KEY | `collections` |
| SUBDIR | `crates/kali_codegen/src/intrinsics/collections_tests` |
| MODPATH | `intrinsics::collections::collections_tests` |
| NTESTS | `14` |
| NGROUPS | `3` |
| NMODFNS | `0` |
| COMMIT_MSG | `refactor(kali_codegen): split intrinsics/collections_tests.rs into per-constructor test submodules [refactor]` |

**Expected R2 mover stdout (sums to 14):**

```
combined: 3
set: 5
map: 6
```

- [ ] Run recipe steps **R1–R7**. R3 expects `FACADE_TESTS_OK`, `MODFNS_OK` (0), `PATHS_OK` (3 `#[path]` decls).

---

## Task 9: Finalize — whole-branch review, merge, cleanup

**Files:** none (review + integration).

- [ ] **Step 1: Whole-branch byte-identity re-proof across all 8 files.**

```bash
for KEY in math call host array control_flow object string collections; do
  case $KEY in
    math)         SUB=crates/kali_codegen/src/intrinsics/math_tests ;;
    call)         SUB=crates/kali_codegen/src/emit/call_tests ;;
    host)         SUB=crates/kali_codegen/src/intrinsics/host_tests ;;
    array)        SUB=crates/kali_codegen/src/intrinsics/array_tests ;;
    control_flow) SUB=crates/kali_codegen/src/emit/control_flow_tests ;;
    object)       SUB=crates/kali_codegen/src/intrinsics/object_tests ;;
    string)       SUB=crates/kali_codegen/src/intrinsics/string_tests ;;
    collections)  SUB=crates/kali_codegen/src/intrinsics/collections_tests ;;
  esac
  python3 .superpowers/sdd/verify.py "$SCRATCH/orig/$KEY.rs" "$SUB/*.rs" || echo "FAIL $KEY"
done
```

Expected: 8 × `PROOF OK` lines, no `FAIL`.

- [ ] **Step 2: Final full build + test on the branch tip.**

```bash
test "$(cargo build -p kali_codegen --tests 2>&1 | grep -c '^warning')" -eq 0 && echo WARN0_OK
cargo test -p kali_codegen --lib 2>&1 | tail -1   # 325 passed; 0 failed
```

- [ ] **Step 3: Whole-branch review (opus).** Dispatch a reviewer over the full base→tip diff (`git diff 138aaa1a7..HEAD`), checking: only `#[test]` fns moved; bodies byte-identical (line-conservation — every drained line reappears verbatim, only `use super::*;` headers + `#[path] mod` decls added); facades retain their original `use` lines and drain to zero module-level fns **except** `control_flow_tests`, which retains exactly `legacy_phase1_baseline`; no `pub` widening; no `cargo fmt` reflow; the production-sibling `#[path = "F_tests.rs"] mod F_tests;` decls are untouched. Resolve any finding before merge.

- [ ] **Step 4: ff-merge to local main and delete the branch (NEVER push origin).**

```bash
git -C /workspace checkout main
git -C /workspace merge --ff-only refactor/kali_codegen-modularization
test "$(cargo build -p kali_codegen --tests 2>&1 | grep -c '^warning')" -eq 0 && echo WARN0_OK
cargo test -p kali_codegen --lib 2>&1 | tail -1   # re-verify on merged main: 325 passed; 0 failed
git -C /workspace branch -d refactor/kali_codegen-modularization
```

- [ ] **Step 5: Update the series memory** (`/home/dev/.claude/projects/-workspace/memory/crate-modularization-series.md`): record kali_codegen as the 28th entry — 8 files, 308 tests, 24 submodules, seven drain-to-0 facades + one (`control_flow_tests`) retaining the `legacy_phase1_baseline` helper, exact-name-set grouping reused unchanged, 0 `include_*!` pins, clean literal gates (0 warnings / 325 pass), final local-main commit hash. Note that the standing local-only convention held even though origin/main had caught up to HEAD at start. Note the remaining frontier (any other crates' co-located src test monoliths).

---

## Self-Review

- **Spec coverage:** all 8 files in the spec's scope table have a task (1–8); facade-drain model encoded per-task via the R3 `MODFNS_OK` check (0 for seven, 1 for `control_flow`); exact-name grouping via the classifier-generated `.spec` files (Task 0 Step 2); 0-pin fact in Task 0 Step 4; the one retained helper handled in Task 0 Step 5 + Task 5; all six spec gates (G1 facade-drained → R3; G2 headers → R4; G3 no-new-warnings → R6; G4 multiset → R6; G5 pass/fail → R6; G6 byte-identity → R5 + Task 9 Step 1); local-main-only integration in Task 9 Step 4.
- **Placeholder scan:** no TBD/TODO; every step has exact commands and expected output; recipe steps spelled out once and parameterized (not "similar to Task N"). Exact per-group counts came from a real classifier run (`ALL PARTITIONS CLEAN; total = 308`), not estimates.
- **Type/name consistency:** MODPATH values match the verified `cargo --list` prefixes (`intrinsics::math::math_tests`, `emit::call::call_tests`, `intrinsics::host::host_tests`, `intrinsics::array::array_tests`, `emit::control_flow::control_flow_tests`, `intrinsics::object::object_tests`, `intrinsics::string::string_tests`, `intrinsics::collections::collections_tests`); group names match the `.spec` file contents and the mover's spec-order stdout; expected per-group counts sum to each NTESTS (16+18+22+41=97; 9+4+44+31=88; 6+2+7+15=30; 12+11=23; 9+10+3=22; 9+9=18; 7+9=16; 3+5+6=14); total = 308.
