# kali_codegen `emit/` Sub-split Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the one remaining large file from the `kali_codegen` modularization — `crates/kali_codegen/src/emit/mod.rs` (4,030 lines, one `impl<'a> FunctionEmitter<'a>` block of 26 methods) and its flat sibling `emit/emit_tests.rs` (2,969 lines, 124 tests) — into four focused, single-purpose method-modules with co-located sibling test files, with zero behavior change.

**Architecture:** `emit/mod.rs` becomes a thin directory-module facade (`mod` declarations only). The 26 methods are partitioned by cohesion into four new files — `control_flow.rs`, `operators.rs`, `literal.rs`, `call.rs` — each carrying its own `impl<'a> FunctionEmitter<'a> { … }` block (legal within one crate). `emit_call` (the ~2,160-line dispatch method) moves verbatim into `call.rs` and stays intact. `emit_tests.rs` splits into four sibling `*_tests.rs` wired with `#[cfg(test)] #[path = "…"] mod`.

**Tech Stack:** Rust 2021, Cargo workspace, `wasm-encoder`, `wasmparser`; dev: the existing crate-local `test_support` (LIR builder + macros) and `kali_test_support` dev-dependency (already wired into the crate).

## Global Constraints

- **Zero behavior change.** Pure structural refactor. The set of tests that exist and pass is identical before and after.
- **Green at every commit.** `cargo test -p kali_codegen` must pass after every task — currently **325 tests**. Never commit a red tree.
- **Identical test-name set.** `cargo test -p kali_codegen -- --list` omits module-path prefixes, so moving a test between files does **not** change the name set. No test fn is renamed; the Task 8 baseline diff must be empty and the count must stay **325**.
- **Public/`pub(crate)` surface preserved.** `FunctionEmitter` and the `Static*` enums stay crate-internal (never `pub`); all 26 methods stay `pub(crate)`. No re-exports change. `emit_call` is **not** broken into helpers (explicit non-goal — see spec).
- **Text-movement only.** Method bodies and test bodies move verbatim. No body is rewritten. Every method is already `pub(crate)` and attaches to the same `FunctionEmitter`, so `self.foo()` already resolves across `impl` blocks anywhere in the crate — **no visibility-widening step is required**.
- **Test convention.** Tests live in sibling `*_tests.rs` wired via `#[cfg(test)] #[path = "…"] mod …;`. Directory-module files use `use crate::*;` (not `super`).
- **No dependency changes.** `test_support` and `kali_test_support` are already wired; nothing added to `Cargo.toml`.
- **Verification triad per task:** `cargo build -p kali_codegen` → `cargo test -p kali_codegen` → `cargo clippy -p kali_codegen --all-targets -- -D warnings` (the clippy gate may be deferred to Task 6/8 if a transient unused-import lint fires mid-split; build + test stay green every commit). All paths are relative to repo root `/workspace`.

---

## Task 1: Create branch and snapshot the test baseline

**Files:**
- Create: `docs/superpowers/baselines/kali_codegen-emit-tests-before.txt`

- [ ] **Step 1: Branch off `main`.**
```bash
cd /workspace
git switch -c refactor/kali-codegen-emit-subsplit
```
Expected: "Switched to a new branch 'refactor/kali-codegen-emit-subsplit'".

- [ ] **Step 2: Confirm the suite is green before any change.**
```bash
cargo test -p kali_codegen 2>&1 | tail -5
```
Expected: all pass; `325 passed`.

- [ ] **Step 3: Snapshot the exact set of test names.**
```bash
cargo test -p kali_codegen -- --list 2>/dev/null | grep ': test$' | sed 's/: test$//' | sort > docs/superpowers/baselines/kali_codegen-emit-tests-before.txt
wc -l docs/superpowers/baselines/kali_codegen-emit-tests-before.txt
```
Expected: 325 lines.

- [ ] **Step 4: Commit the baseline.**
```bash
git add docs/superpowers/baselines/kali_codegen-emit-tests-before.txt
git commit -m "test(kali_codegen): snapshot emit/ subsplit baseline [refactor]"
```

---

## Source-extraction tasks (Tasks 2–5)

Each extraction task follows the identical mechanical recipe:

1. Create the new file in `crates/kali_codegen/src/emit/` with this exact header and one impl block:
   ```rust
   use crate::*;

   impl<'a> FunctionEmitter<'a> {
       // moved methods go here, verbatim
   }
   ```
2. **Cut** the listed methods (full signature through closing `}`) out of the `impl<'a> FunctionEmitter<'a>` block in `emit/mod.rs` and **paste** them, unchanged, into the new file's impl block.
3. Add the `mod <name>;` line to `emit/mod.rs` (keep them alphabetically grouped under the existing `use crate::*;`).
4. Run the verification triad; the remaining methods stay in `mod.rs`'s impl block until their own task.

Method line ranges below are from the **current** `emit/mod.rs` (before any cut) for locating each method; identify each by its `pub(crate) fn <name>` signature, not by absolute line number (numbers shift as you cut).

### Task 2: Extract `control_flow.rs` (node dispatch + structural emission)

**Files:**
- Create: `crates/kali_codegen/src/emit/control_flow.rs`
- Modify: `crates/kali_codegen/src/emit/mod.rs`

**Interfaces:**
- Produces: `impl<'a> FunctionEmitter<'a>` methods `emit_node`, `emit_value`, `emit_sequence`, `emit_function_body`, `emit_break_or_continue`, `emit_branch`, `for_of_binding_name`, `for_of_binding_name_from_node` — all `pub(crate)`, signatures unchanged.

- [ ] **Step 1:** Create `emit/control_flow.rs` with the header/impl skeleton above. Move these eight methods verbatim (current ranges):
  - `emit_break_or_continue` (≈6–63)
  - `emit_function_body` (≈64–79)
  - `emit_sequence` (≈80–110)
  - `emit_node` (≈111–186)
  - `emit_value` (≈187–313)
  - `for_of_binding_name` (≈3929–3933)
  - `for_of_binding_name_from_node` (≈3934–3956)
  - `emit_branch` (≈3957–4028)

- [ ] **Step 2:** In `emit/mod.rs`, add `mod control_flow;` under `use crate::*;`.

- [ ] **Step 3:** Verify.
```bash
cargo build -p kali_codegen 2>&1 | tail -5
cargo test -p kali_codegen 2>&1 | tail -5
```
Expected: builds; `325 passed`.

- [ ] **Step 4: Commit.**
```bash
git add -A && git commit -m "refactor(kali_codegen): extract emit/control_flow [refactor]"
```

---

### Task 3: Extract `operators.rs` (unary / binary / update / exponentiation)

**Files:**
- Create: `crates/kali_codegen/src/emit/operators.rs`
- Modify: `crates/kali_codegen/src/emit/mod.rs`

**Interfaces:**
- Produces: `pub(crate)` methods `emit_unary`, `emit_binary`, `emit_update_expression`, `emit_exponentiation_expression`, `perfect_square_root_i128` — signatures unchanged.

- [ ] **Step 1:** Create `emit/operators.rs` with the header/impl skeleton. Move these five methods verbatim (current ranges):
  - `emit_update_expression` (≈314–376)
  - `emit_unary` (≈377–660)
  - `emit_binary` (≈1011–1166)
  - `emit_exponentiation_expression` (≈3456–3678)
  - `perfect_square_root_i128` (≈3679–3701)

- [ ] **Step 2:** In `emit/mod.rs`, add `mod operators;`.

- [ ] **Step 3:** Verify.
```bash
cargo build -p kali_codegen 2>&1 | tail -5
cargo test -p kali_codegen 2>&1 | tail -5
```
Expected: builds; `325 passed`.

- [ ] **Step 4: Commit.**
```bash
git add -A && git commit -m "refactor(kali_codegen): extract emit/operators [refactor]"
```

---

### Task 4: Extract `literal.rs` (aggregate literals + assignment)

**Files:**
- Create: `crates/kali_codegen/src/emit/literal.rs`
- Modify: `crates/kali_codegen/src/emit/mod.rs`

**Interfaces:**
- Produces: `pub(crate)` methods `emit_aggregate_literal`, `resolve_literal_aggregate`, `assignment_target_name`, `emit_assignment` — signatures unchanged.

- [ ] **Step 1:** Create `emit/literal.rs` with the header/impl skeleton. Move these four methods verbatim (current ranges):
  - `emit_aggregate_literal` (≈661–694)
  - `resolve_literal_aggregate` (≈695–809)
  - `assignment_target_name` (≈810–829)
  - `emit_assignment` (≈830–1010)

- [ ] **Step 2:** In `emit/mod.rs`, add `mod literal;`.

- [ ] **Step 3:** Verify.
```bash
cargo build -p kali_codegen 2>&1 | tail -5
cargo test -p kali_codegen 2>&1 | tail -5
```
Expected: builds; `325 passed`.

- [ ] **Step 4: Commit.**
```bash
git add -A && git commit -m "refactor(kali_codegen): extract emit/literal [refactor]"
```

---

### Task 5: Extract `call.rs` and reduce `emit/mod.rs` to a facade

**Files:**
- Create: `crates/kali_codegen/src/emit/call.rs`
- Modify: `crates/kali_codegen/src/emit/mod.rs`

**Interfaces:**
- Produces: `pub(crate)` methods `emit_call`, `resolve_static_index_member`, `static_member_index`, `resolve_static_reference_root_name`, `unwrap_transparent_value_node`, `is_supported_callable_reference`, `resolve_bound_node`, `resolve_bound_member_callable_node`, `resolve_transparent_callable_node` — signatures unchanged. `emit_call` moves **intact** (~2,160 lines, no helper extraction).

- [ ] **Step 1:** Create `emit/call.rs` with the header/impl skeleton. Move these nine methods verbatim (current ranges) — every method remaining in `mod.rs`'s impl block after Tasks 2–4:
  - `emit_call` (≈1167–3326)
  - `resolve_static_index_member` (≈3327–3367)
  - `static_member_index` (≈3368–3372)
  - `resolve_static_reference_root_name` (≈3373–3455)
  - `unwrap_transparent_value_node` (≈3702–3716)
  - `is_supported_callable_reference` (≈3717–3730)
  - `resolve_bound_node` (≈3731–3752)
  - `resolve_bound_member_callable_node` (≈3753–3829)
  - `resolve_transparent_callable_node` (≈3830–3928)

- [ ] **Step 2:** In `emit/mod.rs`, add `mod call;`. The `impl<'a> FunctionEmitter<'a>` block in `mod.rs` is now empty — delete the empty `impl<'a> FunctionEmitter<'a> {}` block. `mod.rs` should now contain only the four `mod` declarations plus the still-present test wiring at the bottom (`#[cfg(test)] #[path = "emit_tests.rs"] mod emit_tests;`, untouched until Task 7). If the leading `use crate::*;` now triggers an unused-import warning, remove it.

- [ ] **Step 3:** Confirm `emit/mod.rs` is a thin facade — no `impl` block, no `fn`:
```bash
grep -nE '^\s*(impl|pub\(crate\) fn|fn )' crates/kali_codegen/src/emit/mod.rs
```
Expected: no output (only `mod` decls and the `#[path]` test wiring remain).

- [ ] **Step 4:** Verify.
```bash
cargo build -p kali_codegen 2>&1 | tail -5
cargo test -p kali_codegen 2>&1 | tail -5
```
Expected: builds; `325 passed`.

- [ ] **Step 5: Commit.**
```bash
git add -A && git commit -m "refactor(kali_codegen): extract emit/call; reduce emit/mod.rs to facade [refactor]"
```

---

## Task 6: `cargo fmt` normalization

**Files:**
- Modify: the five `emit/*.rs` source files

- [ ] **Step 1:** Normalize formatting after the split:
```bash
cd /workspace
cargo fmt -p kali_codegen
```

- [ ] **Step 2:** Confirm still green and clippy-clean:
```bash
cargo test -p kali_codegen 2>&1 | tail -5
cargo clippy -p kali_codegen --all-targets -- -D warnings 2>&1 | tail -15
```
Expected: `325 passed`; no clippy errors.

- [ ] **Step 3: Commit.**
```bash
git add -A && git commit -m "style(kali_codegen): cargo fmt after emit/ split [refactor]"
```

---

## Task 7: Split `emit_tests.rs` into sibling `*_tests.rs` per module

**Files:**
- Create: `crates/kali_codegen/src/emit/control_flow_tests.rs`, `operators_tests.rs`, `literal_tests.rs`, `call_tests.rs`
- Modify: each new source module (add `#[cfg(test)] #[path] mod` wiring at its bottom); `emit/mod.rs` (remove the `emit_tests` wiring)
- Delete (at end): `crates/kali_codegen/src/emit/emit_tests.rs`

**Classification rule.** Each of the 124 `#[test]` fns moves verbatim to the test file matching the method it exercises. Because `--list` omits module paths, a borderline test landing in a neighboring file does **not** affect the identical-name-set proof — keep files cohesive; when unsure between two, read the body and place it with the method it most directly asserts on. Counts below total 124.

| Destination test file       | Test clusters (by name; verify against body)                                                                                   | Count |
|-----------------------------|--------------------------------------------------------------------------------------------------------------------------------|-------|
| `operators_tests.rs`        | `bitwise_*`, `update_expression_*`, `supported_exponentiation_*`, `unsupported_exponentiation_*`, `supported_remainder_*`, `static_identity_*`, `nullish_coalescing_*` | 8     |
| `literal_tests.rs`          | `mutable_local_*`, `compound_assignment_*`, `nullish_assignment_*`, `logical_assignment_*`                                      | 6     |
| `control_flow_tests.rs`     | `generates_valid_wasm_*`, `function_plans_*`, `boolean_branches_*`, `unsupported_*generator_*`, `mixed_generator_*`, `generator_function_without_yield_*`, `legacy_phase1_baseline`, `mir_backed_pipeline_*` | 24    |
| `call_tests.rs`             | all `for_of_*` / `for_await_*` / `supported_for_*` iteration tests, `unsupported_array_callback_iteration_*`, `object_enumeration_helper_*`, `unresolved_*`, `duplicate_unresolved_*`, `source_path_in_temp_dir_*` | 86    |

(Rationale for the large `call_tests.rs`: the for-of/for-await iteration and unresolved-target tests stress the callable/static-reference resolvers — `unwrap_transparent_value_node`, `resolve_bound_node`, `is_supported_callable_reference`, `emit_call` — that live in `call.rs`. This mirrors `call.rs` being the largest source file.)

- [ ] **Step 1: Worklist — confirm the count to place.**
```bash
cd /workspace
grep -cE '^\s*fn [a-z0-9_]+\(' crates/kali_codegen/src/emit/emit_tests.rs
```
Expected: 124.

- [ ] **Step 2:** For each of the four destination files, create it with this header (copied from the current `emit_tests.rs`), then delete the import lines whose symbols the moved tests in that file do **not** use (the build's unused-import warnings tell you which):
```rust
use crate::lower::collect_functions;
use crate::test_support::*;
use crate::*;
use kali_test_support::fixtures::{tempdir, write_file};
use wasmparser::Validator;
```

- [ ] **Step 3:** Wire each test file into its source module by adding at the **bottom of the source file** (e.g. in `emit/operators.rs`):
```rust
#[cfg(test)]
#[path = "operators_tests.rs"]
mod operators_tests;
```
The `#[path]` is relative to the source file's directory, so `emit/operators.rs` → `operators_tests.rs` resolves to `emit/operators_tests.rs`. Repeat for `control_flow`, `literal`, `call`.

- [ ] **Step 4:** Move the classified tests into their destination files, one destination at a time, running after each and committing per file. Keep the remaining tests in `emit_tests.rs` until their batch:
```bash
cargo test -p kali_codegen 2>&1 | tail -5
git add -A && git commit -m "test(kali_codegen): co-locate emit/<module> tests [refactor]"
```
Expected after each batch: green; the running total (moved + still-in-`emit_tests.rs`) stays at **325**.

- [ ] **Step 5:** When `emit_tests.rs` has no `#[test]` fns left, delete it and remove its wiring:
```bash
rm crates/kali_codegen/src/emit/emit_tests.rs
```
Remove the `#[cfg(test)] #[path = "emit_tests.rs"] mod emit_tests;` lines from `emit/mod.rs`.

- [ ] **Step 6:** Verify.
```bash
cargo test -p kali_codegen 2>&1 | tail -5
```
Expected: `325 passed`.

- [ ] **Step 7: Commit.**
```bash
git add -A && git commit -m "test(kali_codegen): remove monolithic emit_tests.rs [refactor]"
```

---

## Task 8: Final verification, lint, and baseline diff

**Files:**
- Create: `docs/superpowers/baselines/kali_codegen-emit-tests-after.txt`
- Create: `docs/superpowers/baselines/kali_codegen-emit-tests-renames.md`

- [ ] **Step 1: Regenerate the after-snapshot.**
```bash
cd /workspace
cargo test -p kali_codegen -- --list 2>/dev/null | grep ': test$' | sed 's/: test$//' | sort > docs/superpowers/baselines/kali_codegen-emit-tests-after.txt
```

- [ ] **Step 2: Diff before vs after — must be empty.**
```bash
diff docs/superpowers/baselines/kali_codegen-emit-tests-before.txt docs/superpowers/baselines/kali_codegen-emit-tests-after.txt && echo "IDENTICAL"
wc -l docs/superpowers/baselines/kali_codegen-emit-tests-*.txt
```
Expected: prints `IDENTICAL`; both files 325 lines. Write `kali_codegen-emit-tests-renames.md` containing "No renames — test name set identical (module-path prefixes are not included by `--list`)." (If the diff is somehow non-empty, a test was dropped or renamed — fix before proceeding.)

- [ ] **Step 3: Per-file size check** — confirm the monolith is gone:
```bash
wc -l crates/kali_codegen/src/emit/*.rs | sort -n
```
Expected: no source file except `call.rs` (~2,300, dominated by the intact `emit_call`) exceeds ~700 lines; `mod.rs` is a small facade.

- [ ] **Step 4: Format, lint, and full-workspace sanity.**
```bash
cargo fmt -p kali_codegen
cargo clippy -p kali_codegen --all-targets -- -D warnings 2>&1 | tail -15
cargo build --workspace 2>&1 | tail -5
cargo test -p kali_codegen 2>&1 | tail -5
```
Expected: no clippy errors; workspace builds; `325 passed`.

- [ ] **Step 5: Commit the after-baseline.**
```bash
git add docs/superpowers/baselines/kali_codegen-emit-tests-after.txt docs/superpowers/baselines/kali_codegen-emit-tests-renames.md
git commit -m "test(kali_codegen): record emit/ subsplit after-baseline [refactor]"
```

---

## Out of scope

- Cracking `emit_call` into per-family helper methods (separate future pass — its own spec).
- Any behavior, output, or public-API change; renaming public items; restructuring LIR input.
- Refactoring other crates.

## References

- Design spec: `docs/superpowers/specs/2026-06-24-kali-codegen-emit-subsplit-design.md`
- Parent plan (same pattern): `docs/superpowers/plans/2026-06-24-kali-codegen-modularization.md`
- Memory: `kali-crate-modularization`
