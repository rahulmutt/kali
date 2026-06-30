# kali_mir src test-monolith modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split kali_mir's two multi-concern co-located `src/*_tests.rs` unit-test monoliths into thin facades + per-concern `#[path] mod` submodules, by pure verbatim code-motion.

**Architecture:** Each monolith `src/<name>_tests.rs` is declared from a product module via `#[cfg(test)] #[path = "<name>_tests.rs"] mod <name>_tests;`. We turn each into a facade that keeps its original `use` lines and one `#[path = "<name>_tests/<group>.rs"] mod <group>;` per group, and we move each `#[test]` fn **verbatim** into the matching `src/<name>_tests/<group>.rs`, each of which opens with `use super::*;`. No product code changes; the compiled test set is byte-for-byte identical.

**Tech Stack:** Rust 2021, cargo, kali workspace.

**Reusable tooling (optional, recommended):** `.superpowers/sdd/move_fns.py` automates each split (run from `crates/kali_mir`); it drains the `#[test]` fns into submodules by **exact-name grouping** (a fn joins the first group whose member list contains its exact name; `*` = catch-all), auto-retains any non-`#[test]` module-level fn in the facade (there are none here), and appends the `#[path] mod` decls. `.superpowers/sdd/verify.py` proves `{name: body}` from the original == from the submodules. Manual code-motion that produces the exact files below is equally acceptable. **Do not edit `FN_RE` / `IDENT_CHARS` / `find_close_line`.**

## Global Constraints

- **Pure verbatim code-motion, zero behavior change.** No new product code, no new tests, no renamed tests, no reformatting of moved bodies. Move each `#[test]` fn exactly as written, with its full attribute block + body.
- **Facades drain to 0 module-level fns.** The only things retained on each facade are its original `use` lines and the new `#[path] mod` declarations. (Both files have **0** non-`#[test]` module-level fns.)
- **Submodule header:** every new `src/<name>_tests/<group>.rs` begins with exactly `use super::*;` and nothing else before the first moved fn. (Submodules reach all crate symbols and the facade's retained `use` imports through this glob — proven 0-warning across prior series entries.)
- **Test count is the invariant.** kali_mir's lib test suite is **36 tests** before and after; in-scope per-file `--list` counts must stay: `analysis::ownership_analysis_tests` **13**, `lower::lower_tests` **12**.
- **No `pub`/`pub(crate)` widening, no `include_*!` pins** — verified 0 of each across both files; no signature changes needed.
- **Product siblings unchanged:** `src/analysis/mod.rs` (decl at lines 297-298) and `src/lower.rs` (decl at lines 133-134) keep their existing `#[cfg(test)] #[path = "F_tests.rs"] mod F_tests;` decls. The 11 out-of-scope lib tests in small single-concern `*_tests.rs` files stay whole.
- **Build gate:** `cargo build -p kali_mir --tests` stays at **0 warnings** (baseline = 0).
- **`cargo fmt --check`** — accept known fmt nits per series convention (do not reformat moved bodies to satisfy it).
- **Commits:** one `refactor(kali_mir): split <name>_tests.rs into per-concern test submodules [refactor]` per task. Local-main ff-merge only; no origin push.

## Before starting (once)

- Branch: work on `refactor/kali_mir-modularization` off main; confirm baseline green (**0** warnings, **36** lib tests) before any move.
- **Capture pre-move snapshots** of both in-scope files into a fixed scratch dir (outside the repo):
  ```bash
  SNAP=/tmp/claude-1000/-workspace/kali_mir_split_scratch/orig
  mkdir -p "$SNAP"
  cp crates/kali_mir/src/analysis/ownership_analysis_tests.rs "$SNAP/ownership_analysis_tests.rs"
  cp crates/kali_mir/src/lower_tests.rs "$SNAP/lower_tests.rs"
  ```
  The `$SNAP/<F>_tests.rs` files are the byte-identity baseline used by every `verify.py` step below.
- **Capture baseline name-sets** for the count gate (bare fn names, module prefix stripped):
  ```bash
  cargo test -p kali_mir --lib -- --list 2>/dev/null | grep ': test$' \
    | grep 'ownership_analysis_tests::' | sed -E 's/: test$//; s/^.*:://' | sort \
    > "$SNAP/ownership.names"   # 13 lines
  cargo test -p kali_mir --lib -- --list 2>/dev/null | grep ': test$' \
    | grep 'lower_tests::' | sed -E 's/: test$//; s/^.*:://' | sort \
    > "$SNAP/lower.names"       # 12 lines
  ```

---

### Task 1: Split `analysis/ownership_analysis_tests.rs` (13 tests → 4 submodules)

**Files:**
- Create: `crates/kali_mir/src/analysis/ownership_analysis_tests/allocation.rs`
- Create: `crates/kali_mir/src/analysis/ownership_analysis_tests/call_escape.rs`
- Create: `crates/kali_mir/src/analysis/ownership_analysis_tests/alias_precision.rs`
- Create: `crates/kali_mir/src/analysis/ownership_analysis_tests/aggregate_escape.rs`
- Modify: `crates/kali_mir/src/analysis/ownership_analysis_tests.rs` (reduce to facade)
- Unchanged: `crates/kali_mir/src/analysis/mod.rs` (`#[cfg(test)] #[path = "ownership_analysis_tests.rs"] mod ownership_analysis_tests;` at lines 297-298 stays)

**Interfaces:**
- Consumes: nothing from other tasks.
- Produces: nothing other tasks depend on. (Each task is independent.)

- [ ] **Step 1: Confirm baseline count**

Run: `cargo test -p kali_mir ownership_analysis_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 13 passed; ...`

- [ ] **Step 2: Create `allocation.rs`**

Create `crates/kali_mir/src/analysis/ownership_analysis_tests/allocation.rs` starting with `use super::*;`, then move these fns **verbatim** from `ownership_analysis_tests.rs` (full attribute block + body):
- `test_stack_local_bindings_stay_stack_allocated`
- `test_returned_bindings_become_owned_heap`
- `test_captured_bindings_become_shared_heap`
- `test_non_escaping_closure_captures_stay_borrowed`

- [ ] **Step 3: Create `call_escape.rs`**

Create `crates/kali_mir/src/analysis/ownership_analysis_tests/call_escape.rs` starting with `use super::*;`, then move verbatim:
- `test_call_arguments_escape_to_unknown_callees`
- `test_inline_pure_function_calls_do_not_force_argument_escape`
- `test_inline_leaking_function_calls_still_escape_arguments`

- [ ] **Step 4: Create `alias_precision.rs`**

Create `crates/kali_mir/src/analysis/ownership_analysis_tests/alias_precision.rs` starting with `use super::*;`, then move verbatim:
- `test_aliased_function_expressions_preserve_direct_call_precision`
- `test_function_alias_chains_preserve_direct_call_precision`
- `test_aliased_function_expressions_still_track_nested_closure_escapes`

- [ ] **Step 5: Create `aggregate_escape.rs`**

Create `crates/kali_mir/src/analysis/ownership_analysis_tests/aggregate_escape.rs` starting with `use super::*;`, then move verbatim:
- `test_object_literal_values_escape_without_treating_keys_as_identifiers`
- `test_array_element_values_escape_to_heap_storage`
- `test_assignment_into_member_expressions_marks_rhs_escape`

(Steps 2-5 are exactly what `move_fns.py` automates:
```bash
cd crates/kali_mir
python3 ../../.superpowers/sdd/move_fns.py src/analysis/ownership_analysis_tests.rs \
  "allocation=test_stack_local_bindings_stay_stack_allocated,test_returned_bindings_become_owned_heap,test_captured_bindings_become_shared_heap,test_non_escaping_closure_captures_stay_borrowed;call_escape=test_call_arguments_escape_to_unknown_callees,test_inline_pure_function_calls_do_not_force_argument_escape,test_inline_leaking_function_calls_still_escape_arguments;alias_precision=test_aliased_function_expressions_preserve_direct_call_precision,test_function_alias_chains_preserve_direct_call_precision,test_aliased_function_expressions_still_track_nested_closure_escapes;aggregate_escape=test_object_literal_values_escape_without_treating_keys_as_identifiers,test_array_element_values_escape_to_heap_storage,test_assignment_into_member_expressions_marks_rhs_escape"
cd ../..
```
The tool also performs Step 6 (facade rewrite) below. If running it, skip to Step 7 and confirm the facade matches.)

- [ ] **Step 6: Reduce the facade**

Replace the entire contents of `crates/kali_mir/src/analysis/ownership_analysis_tests.rs` with exactly:

```rust
use crate::test_support::*;
use crate::*;
use kali_hir::{HirNode, HirNodeId, HirNodeKind, LoweringResult as HirLoweringResult};

#[path = "ownership_analysis_tests/allocation.rs"]
mod allocation;

#[path = "ownership_analysis_tests/call_escape.rs"]
mod call_escape;

#[path = "ownership_analysis_tests/alias_precision.rs"]
mod alias_precision;

#[path = "ownership_analysis_tests/aggregate_escape.rs"]
mod aggregate_escape;
```

- [ ] **Step 7: Verify count unchanged and tests pass**

Run: `cargo test -p kali_mir ownership_analysis_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 13 passed; 0 failed; ...`

Run (name-set preserved, module prefix stripped):
```bash
cargo test -p kali_mir --lib -- --list 2>/dev/null | grep ': test$' \
  | grep 'ownership_analysis_tests::' | sed -E 's/: test$//; s/^.*:://' | sort \
  | diff - /tmp/claude-1000/-workspace/kali_mir_split_scratch/orig/ownership.names
```
Expected: empty diff (exit 0).

- [ ] **Step 8: Verify whole-crate suite and build**

Run: `cargo test -p kali_mir --lib 2>&1 | grep 'test result'`
Expected: `test result: ok. 36 passed; ...`
Run: `cargo build -p kali_mir --tests 2>&1 | grep -c '^warning'`
Expected: `0`

- [ ] **Step 9: Byte-identity proof**

Run (from repo root, against the pre-move snapshot captured before Step 2):
`python3 .superpowers/sdd/verify.py /tmp/claude-1000/-workspace/kali_mir_split_scratch/orig/ownership_analysis_tests.rs "crates/kali_mir/src/analysis/ownership_analysis_tests/*.rs"`
Expected: exit 0 (13/13 `#[test]` bodies byte-identical).

- [ ] **Step 10: Confirm facade has 0 `#[test]`**

Run: `grep -c '#\[test\]' crates/kali_mir/src/analysis/ownership_analysis_tests.rs`
Expected: `0`

- [ ] **Step 11: Commit**

```bash
git add crates/kali_mir/src/analysis/ownership_analysis_tests.rs crates/kali_mir/src/analysis/ownership_analysis_tests/
git commit -m "refactor(kali_mir): split ownership_analysis_tests.rs into per-concern test submodules [refactor]"
```

---

### Task 2: Split `lower_tests.rs` (12 tests → 2 submodules)

**Files:**
- Create: `crates/kali_mir/src/lower_tests/flavor_metadata.rs`
- Create: `crates/kali_mir/src/lower_tests/structure.rs`
- Modify: `crates/kali_mir/src/lower_tests.rs` (reduce to facade)
- Unchanged: `crates/kali_mir/src/lower.rs` (`#[cfg(test)] #[path = "lower_tests.rs"] mod lower_tests;` at lines 133-134 stays)

**Interfaces:**
- Consumes: nothing. Produces: nothing other tasks depend on.

- [ ] **Step 1: Confirm baseline count**

Run: `cargo test -p kali_mir lower_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 12 passed; ...`

- [ ] **Step 2: Create `flavor_metadata.rs`**

Create `crates/kali_mir/src/lower_tests/flavor_metadata.rs` starting with `use super::*;`, then move these fns **verbatim** from `lower_tests.rs` (full attribute block + body):
- `test_mir_lowering_preserves_function_nodes_with_flavor_metadata`
- `test_mir_lowering_preserves_function_flavor_metadata`
- `test_mir_lowering_preserves_function_flavor_metadata_for_function_expressions`
- `test_mir_lowering_preserves_function_flavor_metadata_for_class_methods`
- `test_mir_lowering_preserves_function_flavor_metadata_for_class_expressions`
- `test_mir_lowering_preserves_function_flavor_metadata_for_default_export_generator_function_declaration`
- `test_mir_lowering_preserves_function_flavor_metadata_for_default_export_anonymous_generator_function_declaration`
- `test_mir_lowering_preserves_function_flavor_metadata_for_default_export_class_expressions`
- `test_mir_lowering_preserves_function_flavor_metadata_for_default_export_class_declarations`

- [ ] **Step 3: Create `structure.rs`**

Create `crates/kali_mir/src/lower_tests/structure.rs` starting with `use super::*;`, then move verbatim:
- `test_mir_lowering_preserves_program_shape`
- `test_call_expressions_lower_to_call_nodes`
- `test_mir_validation_rejects_out_of_bounds_children`

(Steps 2-3 are exactly what `move_fns.py` automates — `structure=*` is the catch-all for the 3 fns not named in `flavor_metadata`:
```bash
cd crates/kali_mir
python3 ../../.superpowers/sdd/move_fns.py src/lower_tests.rs \
  "flavor_metadata=test_mir_lowering_preserves_function_nodes_with_flavor_metadata,test_mir_lowering_preserves_function_flavor_metadata,test_mir_lowering_preserves_function_flavor_metadata_for_function_expressions,test_mir_lowering_preserves_function_flavor_metadata_for_class_methods,test_mir_lowering_preserves_function_flavor_metadata_for_class_expressions,test_mir_lowering_preserves_function_flavor_metadata_for_default_export_generator_function_declaration,test_mir_lowering_preserves_function_flavor_metadata_for_default_export_anonymous_generator_function_declaration,test_mir_lowering_preserves_function_flavor_metadata_for_default_export_class_expressions,test_mir_lowering_preserves_function_flavor_metadata_for_default_export_class_declarations;structure=*"
cd ../..
```
The tool also performs Step 4 (facade rewrite) below. If running it, skip to Step 5 and confirm the facade matches.)

- [ ] **Step 4: Reduce the facade**

Replace the entire contents of `crates/kali_mir/src/lower_tests.rs` with exactly:

```rust
use crate::test_support::*;
use crate::*;
use kali_hir::FunctionFlavor;

#[path = "lower_tests/flavor_metadata.rs"]
mod flavor_metadata;

#[path = "lower_tests/structure.rs"]
mod structure;
```

- [ ] **Step 5: Verify count unchanged and tests pass**

Run: `cargo test -p kali_mir lower_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 12 passed; 0 failed; ...`

Run (name-set preserved, module prefix stripped):
```bash
cargo test -p kali_mir --lib -- --list 2>/dev/null | grep ': test$' \
  | grep 'lower_tests::' | sed -E 's/: test$//; s/^.*:://' | sort \
  | diff - /tmp/claude-1000/-workspace/kali_mir_split_scratch/orig/lower.names
```
Expected: empty diff (exit 0).

- [ ] **Step 6: Verify whole-crate suite and build**

Run: `cargo test -p kali_mir --lib 2>&1 | grep 'test result'`
Expected: `test result: ok. 36 passed; ...`
Run: `cargo build -p kali_mir --tests 2>&1 | grep -c '^warning'`
Expected: `0`

- [ ] **Step 7: Byte-identity proof**

Run: `python3 .superpowers/sdd/verify.py /tmp/claude-1000/-workspace/kali_mir_split_scratch/orig/lower_tests.rs "crates/kali_mir/src/lower_tests/*.rs"`
Expected: exit 0 (12/12 `#[test]` bodies byte-identical).

- [ ] **Step 8: Confirm facade has 0 `#[test]`**

Run: `grep -c '#\[test\]' crates/kali_mir/src/lower_tests.rs`
Expected: `0`

- [ ] **Step 9: Commit**

```bash
git add crates/kali_mir/src/lower_tests.rs crates/kali_mir/src/lower_tests/
git commit -m "refactor(kali_mir): split lower_tests.rs into per-concern test submodules [refactor]"
```

---

## Final verification (after both tasks)

- [ ] **Whole-crate lib suite:** `cargo test -p kali_mir --lib 2>&1 | grep 'test result'` → `36 passed; 0 failed`.
- [ ] **Build gate:** `cargo build -p kali_mir --tests 2>&1 | grep -c '^warning'` → `0`.
- [ ] **Byte-identity proof:** for each split file, `python3 .superpowers/sdd/verify.py /tmp/claude-1000/-workspace/kali_mir_split_scratch/orig/<F>_tests.rs "crates/kali_mir/src/<dir>/<F>_tests/*.rs"` exits 0 (25/25 `#[test]` bodies byte-identical base→head: ownership 13, lower 12).
- [ ] **Facade `#[test]` count == 0:** `grep -c '#\[test\]' crates/kali_mir/src/analysis/ownership_analysis_tests.rs crates/kali_mir/src/lower_tests.rs` → both `0`.
- [ ] **Dependent crate compiles unedited:** `cargo build -p kali_cli` (a kali_mir consumer) builds clean.
- [ ] **Diff is motion-only:** `git diff --stat <base>..HEAD -- crates/kali_mir/` shows only the two `*_tests.rs` facades shrinking + new submodule files; no product-source (`analysis/mod.rs`, `analysis/ownership_analysis.rs`, `lower.rs`) and no out-of-scope `*_tests.rs` line changes.
- [ ] **Fmt:** `cargo fmt -p kali_mir --check` — accept known nits per series convention; do not reformat moved bodies.
- [ ] **Integrate:** ff-merge branch into local `main`; re-verify on merged main (`36 passed`, `0 warnings`); delete the branch. **No origin push.**
