# kali_common src test-monolith modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split kali_common's five co-located `src/*_tests.rs` unit-test monoliths into thin facades + per-concern `#[path] mod` submodules, by pure verbatim code-motion.

**Architecture:** Each monolith `src/<name>_tests.rs` is declared from its product module via `#[path = "<name>_tests.rs"] mod <name>_tests;`. We turn each into a facade that keeps the original `use` line(s) plus one `#[path = "<name>_tests/<group>.rs"] mod <group>;` per group, and we move each `#[test]` fn **verbatim** into the matching `src/<name>_tests/<group>.rs` file, each of which opens with `use super::*;`. No product code changes; the compiled test set is byte-for-byte identical.

**Tech Stack:** Rust 2021, cargo, kali workspace.

## Global Constraints

- **Pure verbatim code-motion, zero behavior change.** No new product code, no new tests, no renamed tests, no reformatting of moved bodies. Move each `#[test]` fn exactly as written.
- **All facades drain to 0 module-level fns.** No target file contains any non-`#[test]` module-level helper. The only thing retained on a facade is its original `use` line(s) + the new `#[path] mod` declarations.
- **Submodule header:** every new `src/<name>_tests/<group>.rs` begins with exactly `use super::*;` and nothing else before the first moved fn.
- **Test count is the invariant.** kali_common's lib test suite is **102 tests** before and after; per-file filters must report: late_tests 18, math_tests 21, process_kill_tests 21, object_tests 9, promise_tests 4.
- **`cargo fmt --check`** — accept known fmt nits per series convention (do not reformat moved bodies to satisfy it).
- **Commits:** one `refactor(kali_common): split <file>_tests.rs into per-concern test submodules [refactor]` per task. Local-main ff-merge only; no origin push.

---

### Task 1: Split `late_tests.rs` (18 tests → 3 submodules)

**Files:**
- Create: `crates/kali_common/src/late_tests/object_model.rs`
- Create: `crates/kali_common/src/late_tests/capabilities.rs`
- Create: `crates/kali_common/src/late_tests/process_control.rs`
- Modify: `crates/kali_common/src/late_tests.rs` (reduce to facade)
- Unchanged: `crates/kali_common/src/late.rs` (the `#[path = "late_tests.rs"] mod late_tests;` at line 585 stays)

**Interfaces:**
- Consumes: nothing from other tasks.
- Produces: nothing other tasks depend on. (Each task is independent.)

- [ ] **Step 1: Confirm baseline count**

Run: `cargo test -p kali_common late_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 18 passed; ...`

- [ ] **Step 2: Create `object_model.rs`**

Create `crates/kali_common/src/late_tests/object_model.rs` starting with `use super::*;`, then move these fns **verbatim** from `late_tests.rs` (with their full bodies):
- `test_late_object_model_aliases_and_source_are_canonical`
- `test_late_object_model_own_property_aliases_and_source_are_canonical`

- [ ] **Step 3: Create `capabilities.rs`**

Create `crates/kali_common/src/late_tests/capabilities.rs` starting with `use super::*;`, then move these fns verbatim:
- `test_late_threaded_runtime_aliases_and_source_are_canonical`
- `test_late_permission_escalation_source_lists_request_and_revoke_aliases`
- `test_late_env_materialization_source_lists_to_object_aliases`
- `test_late_subprocess_source_lists_command_aliases`
- `test_late_network_source_lists_connect_listen_and_serve_aliases`
- `test_late_compat_object_has_own_source_lists_representative_aliases_in_order`

- [ ] **Step 4: Create `process_control.rs`**

Create `crates/kali_common/src/late_tests/process_control.rs` starting with `use super::*;`, then move the remaining 10 fns verbatim:
- `test_late_process_control_prefix_source_lists_all_prefix_aliases_in_order`
- `test_late_process_control_exit_aliases_are_canonical`
- `test_late_process_control_exit_source_lists_all_aliases_in_order`
- `test_late_process_control_source_reuses_the_shared_zero_probe_inventory_once`
- `test_late_process_control_single_quoted_process_source_reuses_the_shared_zero_probe_inventory_once`
- `test_late_process_control_single_quoted_process_aliases_lists_all_aliases_in_order`
- `test_late_process_control_single_quoted_process_aliases_compose_kill_and_exit_helpers`
- `test_late_process_control_single_quoted_kill_source_lists_all_aliases_in_order`
- `test_late_process_control_single_quoted_exit_source_lists_all_aliases_in_order`
- `test_late_process_env_mutation_source_lists_mixed_quote_process_aliases_and_mixed_delete_aliases`

- [ ] **Step 5: Reduce the facade**

Replace the entire contents of `crates/kali_common/src/late_tests.rs` with exactly:

```rust
use crate::*;
use super::LATE_PROCESS_CONTROL_PREFIX_SEGMENTS;

#[path = "late_tests/object_model.rs"]
mod object_model;

#[path = "late_tests/capabilities.rs"]
mod capabilities;

#[path = "late_tests/process_control.rs"]
mod process_control;
```

- [ ] **Step 6: Verify count unchanged and tests pass**

Run: `cargo test -p kali_common late_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 18 passed; 0 failed; ...`

- [ ] **Step 7: Verify whole-crate suite and build**

Run: `cargo test -p kali_common 2>&1 | grep -E 'test result' | head -1`
Expected: `test result: ok. 102 passed; ...`
Run: `cargo build -p kali_common`
Expected: builds clean (warnings ok).

- [ ] **Step 8: Commit**

```bash
git add crates/kali_common/src/late_tests.rs crates/kali_common/src/late_tests/
git commit -m "refactor(kali_common): split late_tests.rs into per-concern test submodules [refactor]"
```

---

### Task 2: Split `math_tests.rs` (21 tests → 3 submodules)

**Files:**
- Create: `crates/kali_common/src/math_tests/rounding.rs`
- Create: `crates/kali_common/src/math_tests/pow.rs`
- Create: `crates/kali_common/src/math_tests/roots.rs`
- Modify: `crates/kali_common/src/math_tests.rs` (reduce to facade)
- Unchanged: `crates/kali_common/src/math.rs` (`#[path] mod math_tests;` at line 641 stays)

**Interfaces:**
- Consumes: nothing. Produces: nothing other tasks depend on.

- [ ] **Step 1: Confirm baseline count**

Run: `cargo test -p kali_common math_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 21 passed; ...`

- [ ] **Step 2: Create `rounding.rs`**

Create `crates/kali_common/src/math_tests/rounding.rs` starting with `use super::*;`, then move verbatim:
- `test_math_abs_sign_frozen_callable_source_lists_all_aliases_in_order`
- `test_math_abs_sign_frozen_callable_invocation_and_entry_sources_are_canonical`
- `test_math_floor_trunc_ceil_frozen_callable_source_lists_all_aliases_in_order`
- `test_math_floor_trunc_ceil_frozen_callable_invocation_and_entry_sources_are_canonical`
- `test_math_round_frozen_callable_source_lists_all_aliases_in_order`

- [ ] **Step 3: Create `pow.rs`**

Create `crates/kali_common/src/math_tests/pow.rs` starting with `use super::*;`, then move verbatim (these are interleaved with the roots tests in the source — collect all 13 by name):
- `test_math_pow_source_lists_all_aliases_in_order`
- `test_math_pow_alias_inventory_source_reuses_the_shared_helper_sources_once`
- `test_math_pow_browser_alias_inventory_aliases_list_all_aliases_in_order`
- `test_math_pow_browser_alias_inventory_source_is_canonical`
- `test_math_pow_browser_alias_inventory_source_reuses_the_canonical_math_pow_alias_inventory`
- `test_math_pow_browser_alias_inventory_invocation_lines_are_canonical`
- `test_math_pow_browser_alias_inventory_invocation_source_is_canonical`
- `test_math_pow_bracketed_global_this_alias_chain_source_is_canonical`
- `test_math_pow_frozen_callable_source_lists_all_aliases_in_order`
- `test_math_pow_bracketed_frozen_callable_source_lists_all_aliases_in_order`
- `test_math_pow_bracketed_frozen_callable_invocation_lines_are_canonical`
- `test_math_pow_bracketed_frozen_callable_invocation_entries_are_canonical`
- `test_math_pow_invocation_lines_are_canonical`

- [ ] **Step 4: Create `roots.rs`**

Create `crates/kali_common/src/math_tests/roots.rs` starting with `use super::*;`, then move verbatim:
- `test_math_cbrt_frozen_callable_source_lists_all_aliases_in_order`
- `test_math_hypot_frozen_callable_source_lists_all_aliases_in_order`
- `test_math_exp2_frozen_callable_source_lists_all_aliases_in_order`

- [ ] **Step 5: Reduce the facade**

Replace the entire contents of `crates/kali_common/src/math_tests.rs` with exactly:

```rust
use crate::*;

#[path = "math_tests/rounding.rs"]
mod rounding;

#[path = "math_tests/pow.rs"]
mod pow;

#[path = "math_tests/roots.rs"]
mod roots;
```

- [ ] **Step 6: Verify count unchanged and tests pass**

Run: `cargo test -p kali_common math_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 21 passed; 0 failed; ...`

- [ ] **Step 7: Verify whole-crate suite and build**

Run: `cargo test -p kali_common 2>&1 | grep -E 'test result' | head -1`
Expected: `test result: ok. 102 passed; ...`
Run: `cargo build -p kali_common`
Expected: builds clean.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_common/src/math_tests.rs crates/kali_common/src/math_tests/
git commit -m "refactor(kali_common): split math_tests.rs into per-concern test submodules [refactor]"
```

---

### Task 3: Split `process_kill_tests.rs` (21 tests → 3 submodules)

**Files:**
- Create: `crates/kali_common/src/process_kill_tests/inventory.rs`
- Create: `crates/kali_common/src/process_kill_tests/parenthesized_freeze.rs`
- Create: `crates/kali_common/src/process_kill_tests/call_targets.rs`
- Modify: `crates/kali_common/src/process_kill_tests.rs` (reduce to facade)
- Unchanged: `crates/kali_common/src/process_kill.rs` (`#[path] mod process_kill_tests;` at line 391 stays)

**Interfaces:**
- Consumes: nothing. Produces: nothing other tasks depend on.

- [ ] **Step 1: Confirm baseline count**

Run: `cargo test -p kali_common process_kill_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 21 passed; ...`

- [ ] **Step 2: Create `inventory.rs`**

Create `crates/kali_common/src/process_kill_tests/inventory.rs` starting with `use super::*;`, then move verbatim:
- `test_process_kill_zero_probe_source_lists_all_aliases_in_order`
- `test_process_kill_zero_probe_alias_inventory_source_is_prefix_free_and_single_sourced`
- `test_process_kill_zero_probe_unavailable_message_lists_direct_and_wrapped_zero_aliases`
- `test_process_kill_zero_probe_wrapped_zero_aliases_list_all_aliases_in_order`
- `test_process_kill_zero_probe_console_log_source_lists_all_aliases_in_order`
- `test_process_kill_zero_probe_guard_source_lists_all_aliases_in_order`

- [ ] **Step 3: Create `parenthesized_freeze.rs`**

Create `crates/kali_common/src/process_kill_tests/parenthesized_freeze.rs` starting with `use super::*;`, then move verbatim:
- `test_process_kill_zero_probe_parenthesized_frozen_callable_source_lists_all_aliases_in_order`
- `test_process_kill_zero_probe_parenthesized_receiver_freeze_inventory_source_composes_both_helpers`
- `test_process_kill_zero_probe_parenthesized_receiver_freeze_inventory_source_is_prefix_free_and_single_sourced`
- `test_process_kill_zero_probe_parenthesized_receiver_freeze_inventory_source_does_not_include_late_process_control_prefix`
- `test_process_kill_zero_probe_parenthesized_receiver_freeze_bracket_source_lists_all_aliases_in_order`
- `test_process_kill_zero_probe_parenthesized_receiver_freeze_bracket_inventory_source_is_prefix_free_and_single_sourced`
- `test_process_kill_zero_probe_parenthesized_receiver_freeze_inventory_aliases_list_all_aliases_in_order`
- `test_process_kill_zero_probe_parenthesized_receiver_source_lists_all_aliases_in_order`

- [ ] **Step 4: Create `call_targets.rs`**

Create `crates/kali_common/src/process_kill_tests/call_targets.rs` starting with `use super::*;`, then move verbatim:
- `test_process_kill_zero_probe_node_api_surface_sources_are_canonical`
- `test_process_kill_zero_probe_call_target_aliases_list_all_supported_targets_in_order`
- `test_process_kill_zero_probe_typed_wrapper_sources_list_all_call_targets_in_order`
- `test_process_kill_zero_probe_wrapped_call_target_source_reuses_the_shared_inventory`
- `test_process_kill_zero_probe_call_target_aliases_are_in_canonical_order`
- `test_process_kill_zero_probe_direct_call_target_binding_lines_are_canonical`
- `test_process_kill_zero_probe_sequence_call_target_binding_lines_are_canonical`

- [ ] **Step 5: Reduce the facade**

Replace the entire contents of `crates/kali_common/src/process_kill_tests.rs` with exactly:

```rust
use crate::*;

#[path = "process_kill_tests/inventory.rs"]
mod inventory;

#[path = "process_kill_tests/parenthesized_freeze.rs"]
mod parenthesized_freeze;

#[path = "process_kill_tests/call_targets.rs"]
mod call_targets;
```

- [ ] **Step 6: Verify count unchanged and tests pass**

Run: `cargo test -p kali_common process_kill_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 21 passed; 0 failed; ...`

- [ ] **Step 7: Verify whole-crate suite and build**

Run: `cargo test -p kali_common 2>&1 | grep -E 'test result' | head -1`
Expected: `test result: ok. 102 passed; ...`
Run: `cargo build -p kali_common`
Expected: builds clean.

- [ ] **Step 8: Commit**

```bash
git add crates/kali_common/src/process_kill_tests.rs crates/kali_common/src/process_kill_tests/
git commit -m "refactor(kali_common): split process_kill_tests.rs into per-concern test submodules [refactor]"
```

---

### Task 4: Split `object_tests.rs` (9 tests → 2 submodules)

**Files:**
- Create: `crates/kali_common/src/object_tests/reflect.rs`
- Create: `crates/kali_common/src/object_tests/has_own.rs`
- Modify: `crates/kali_common/src/object_tests.rs` (reduce to facade)
- Unchanged: `crates/kali_common/src/object.rs` (`#[path] mod object_tests;` at line 316 stays)

**Interfaces:**
- Consumes: nothing. Produces: nothing other tasks depend on.

- [ ] **Step 1: Confirm baseline count**

Run: `cargo test -p kali_common object_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 9 passed; ...`

- [ ] **Step 2: Create `reflect.rs`**

Create `crates/kali_common/src/object_tests/reflect.rs` starting with `use super::*;`, then move verbatim:
- `test_reflect_own_keys_frozen_callable_aliases_list_all_aliases_in_order`
- `test_reflect_own_keys_frozen_callable_source_lists_all_aliases_in_order`

- [ ] **Step 3: Create `has_own.rs`**

Create `crates/kali_common/src/object_tests/has_own.rs` starting with `use super::*;`, then move verbatim:
- `test_object_has_own_frozen_callable_source_lists_all_aliases_in_order`
- `test_object_enumeration_frozen_callable_source_lists_all_aliases_in_order`
- `test_object_has_own_frozen_callable_condition_source_lists_all_aliases_in_order`
- `test_object_has_own_combined_frozen_callable_condition_source_reuses_both_helpers_once`
- `test_object_has_own_property_call_frozen_callable_source_lists_all_aliases_in_order`
- `test_object_has_own_property_call_frozen_callable_condition_source_lists_all_aliases_in_order`
- `test_object_has_own_property_call_binding_source_is_canonical`

- [ ] **Step 4: Reduce the facade**

Replace the entire contents of `crates/kali_common/src/object_tests.rs` with exactly:

```rust
use crate::*;

#[path = "object_tests/reflect.rs"]
mod reflect;

#[path = "object_tests/has_own.rs"]
mod has_own;
```

- [ ] **Step 5: Verify count unchanged and tests pass**

Run: `cargo test -p kali_common object_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 9 passed; 0 failed; ...`

- [ ] **Step 6: Verify whole-crate suite and build**

Run: `cargo test -p kali_common 2>&1 | grep -E 'test result' | head -1`
Expected: `test result: ok. 102 passed; ...`
Run: `cargo build -p kali_common`
Expected: builds clean.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_common/src/object_tests.rs crates/kali_common/src/object_tests/
git commit -m "refactor(kali_common): split object_tests.rs into per-concern test submodules [refactor]"
```

---

### Task 5: Split `promise_tests.rs` (4 tests → 2 submodules)

**Files:**
- Create: `crates/kali_common/src/promise_tests/aggregate.rs`
- Create: `crates/kali_common/src/promise_tests/select.rs`
- Modify: `crates/kali_common/src/promise_tests.rs` (reduce to facade)
- Unchanged: `crates/kali_common/src/promise.rs` (`#[path] mod promise_tests;` at line 442 stays)

**Interfaces:**
- Consumes: nothing. Produces: nothing other tasks depend on.

- [ ] **Step 1: Confirm baseline count**

Run: `cargo test -p kali_common promise_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 4 passed; ...`

- [ ] **Step 2: Create `aggregate.rs`**

Create `crates/kali_common/src/promise_tests/aggregate.rs` starting with `use super::*;`, then move verbatim:
- `test_promise_all_settled_browser_body_source_includes_the_shared_freeze_wrapper_aliases`
- `test_promise_all_browser_body_source_includes_the_shared_freeze_wrapper_aliases`

- [ ] **Step 3: Create `select.rs`**

Create `crates/kali_common/src/promise_tests/select.rs` starting with `use super::*;`, then move verbatim:
- `test_promise_race_browser_body_source_includes_the_shared_freeze_wrapper_aliases`
- `test_promise_any_browser_body_source_includes_the_shared_freeze_wrapper_aliases`

- [ ] **Step 4: Reduce the facade**

Replace the entire contents of `crates/kali_common/src/promise_tests.rs` with exactly:

```rust
use crate::*;

#[path = "promise_tests/aggregate.rs"]
mod aggregate;

#[path = "promise_tests/select.rs"]
mod select;
```

- [ ] **Step 5: Verify count unchanged and tests pass**

Run: `cargo test -p kali_common promise_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 4 passed; 0 failed; ...`

- [ ] **Step 6: Verify whole-crate suite and build**

Run: `cargo test -p kali_common 2>&1 | grep -E 'test result' | head -1`
Expected: `test result: ok. 102 passed; ...`
Run: `cargo build -p kali_common`
Expected: builds clean.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_common/src/promise_tests.rs crates/kali_common/src/promise_tests/
git commit -m "refactor(kali_common): split promise_tests.rs into per-concern test submodules [refactor]"
```

---

## Final verification (after all 5 tasks)

- [ ] **Whole-crate suite:** `cargo test -p kali_common 2>&1 | grep 'test result'` → `102 passed`.
- [ ] **Dependent crate compiles unedited:** `cargo build -p kali_runtime` (a kali_common consumer) builds clean.
- [ ] **Diff is motion-only:** `git diff --stat 6a9507a0f..HEAD -- crates/kali_common/` shows only `*_tests.rs` facades shrinking + new submodule files; no product-source (`late.rs`, `math.rs`, `process_kill.rs`, `object.rs`, `promise.rs`) line changes.
- [ ] **Fmt:** `cargo fmt -p kali_common --check` — accept known nits per series convention; do not reformat moved bodies.
