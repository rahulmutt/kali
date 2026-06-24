# kali_common Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split `kali_common`'s 3,251-line `lib.rs` and 4,083-line `tests.rs` monolith into focused single-purpose modules with co-located tests, with zero behavior change and the public API preserved exactly.

**Architecture:** kali_common is a flat pile of 156 free `pub fn`/`pub const fn` source-snippet generators (consumed as cross-crate test fixtures) plus a small core of real utility types. Extraction is pure verbatim text-movement of free functions into thematic modules. A thin `lib.rs` facade re-exports every module with `pub use <mod>::*;`, preserving the flat public paths consumers use (`kali_common::math_pow_source`, etc.) — so **no consumer crate changes**. Shared private helpers move to a `helpers.rs` module widened to `pub(crate)`.

**Tech Stack:** Rust, cargo workspace, clippy. No new dependencies.

## Global Constraints

- **Zero behavior change.** Every moved function, const, struct, impl, and test is moved **byte-for-byte verbatim** — no reformatting, no renaming, no logic edits.
- **Public API preserved exactly.** Every currently-`pub` item remains reachable at its current flat path (`kali_common::<name>`). Achieved via `pub use <mod>::*;` in the facade.
- **Authoritative test count = 102** (94 in `tests.rs` + 8 in existing `interner_tests`/`span_tests`/`source_map_tests`). This count must never change.
- **Identical `--list` basename set.** The proof compares the whole-crate `cargo test -p kali_common --all-targets -- --list` output with module-path prefixes stripped (basenames only) against the captured baseline. A raw diff is non-empty by design after co-location — compare basenames.
- **Green + clippy-clean at every commit:** `cargo test -p kali_common` passes, `cargo build --workspace` passes, `cargo clippy -p kali_common --all-targets -- -D warnings` is clean.
- **Do NOT add `kali_test_support`** to this crate (foundational crate, no filesystem tests — the dev-dep would be dead weight; decision recorded in the spec).
- **GLOB rule:** a module re-exported into the facade keeps `pub use <mod>::*;` (or `pub(crate) use helpers::*;` for the internal helpers). If clippy flags any glob `unused_imports`, **delete it** — never `#[allow]`. Drop `use crate::*;` from any module that ends up referencing no crate items.
- Branch: `refactor/kali-common-modularization` (already created and checked out; the design spec is already committed on it).

---

## Standard Extraction Procedure

Tasks 3–14 each extract one module and are mechanically identical. The shared procedure (referenced by each task; task-specific item names are listed in the task):

1. **Create `crates/kali_common/src/<mod>.rs`** beginning with `use crate::*;`. Move the task's listed functions (and any listed private consts) out of `lib.rs` into this file **verbatim** (cut from `lib.rs`, paste into `<mod>.rs`). Functions are interleaved with other clusters in `lib.rs` — move **by name**, not by line range. End the file with:
   ```rust
   #[cfg(test)]
   #[path = "<mod>_tests.rs"]
   mod <mod>_tests;
   ```
2. **Create `crates/kali_common/src/<mod>_tests.rs`** beginning with `use crate::*;`. Move the task's listed `#[test] fn`s out of `tests.rs` into this file **verbatim** (with their `#[test]` attribute lines).
3. **Add to the `lib.rs` facade**, in the module-declaration block: `mod <mod>;` and in the re-export block: `pub use <mod>::*;`.
4. **Verify** (run all three; all must pass):
   ```bash
   cargo test -p kali_common
   cargo build --workspace
   cargo clippy -p kali_common --all-targets -- -D warnings
   ```
   Then confirm the basename set still matches the baseline (see Task 1 for the exact command).
5. **Commit** with message `refactor(kali_common): extract <mod> module [refactor]`.

**Cross-module note:** because every fixture function remains reachable at the crate root throughout the refactor (either still defined in `lib.rs`, or re-exported via `pub use <mod>::*;`), the order of Tasks 3–14 does not matter for compilation, and `use crate::*;` in each module/test file resolves all sibling-cluster calls and shared helpers. Only `helpers.rs` (Task 2) must precede the others.

---

### Task 1: Capture baseline and confirm clean starting state

**Files:**
- Create: `docs/superpowers/baselines/2026-06-24-kali-common-baseline.txt`

- [ ] **Step 1: Confirm the crate builds clean before any change**

Run:
```bash
cargo test -p kali_common
cargo clippy -p kali_common --all-targets -- -D warnings
```
Expected: tests pass, clippy clean.

- [ ] **Step 2: Capture the authoritative test basename baseline**

Run:
```bash
cargo test -p kali_common --all-targets -- --list 2>/dev/null \
  | grep ': test$' \
  | sed -E 's/^.*::([a-z0-9_]+): test$/\1/' \
  | sort > docs/superpowers/baselines/2026-06-24-kali-common-baseline.txt
wc -l docs/superpowers/baselines/2026-06-24-kali-common-baseline.txt
```
Expected: `102 docs/superpowers/baselines/2026-06-24-kali-common-baseline.txt`

- [ ] **Step 3: Record the basename-match helper command**

Throughout this plan, "confirm basename match" means this command prints nothing (empty diff):
```bash
diff <(cargo test -p kali_common --all-targets -- --list 2>/dev/null \
  | grep ': test$' | sed -E 's/^.*::([a-z0-9_]+): test$/\1/' | sort) \
  docs/superpowers/baselines/2026-06-24-kali-common-baseline.txt
```
Expected after every task: no output (identical basename set).

- [ ] **Step 4: Commit the baseline**

```bash
git add docs/superpowers/baselines/2026-06-24-kali-common-baseline.txt
git commit -m "test(kali_common): capture pre-refactor baseline [refactor]"
```

---

### Task 2: Extract shared private helpers to `helpers.rs`

**Files:**
- Create: `crates/kali_common/src/helpers.rs`
- Modify: `crates/kali_common/src/lib.rs`

**Interfaces:**
- Produces (all widened from private `fn` to `pub(crate) fn`, signatures otherwise unchanged):
  - `pub(crate) fn join_semicolon_terminated_segments(segments: &[&str]) -> String`
  - `pub(crate) fn join_zero_probe_aliases(aliases: &[&'static str]) -> String`
  - `pub(crate) fn join_const_binding_lines(bindings: &[(&'static str, &'static str)]) -> String`
  - `pub(crate) fn ordered_unique_union(slices: &[&[&'static str]]) -> Vec<&'static str>`
  - These are re-exported at the crate root via `pub(crate) use helpers::*;`, so every module's `use crate::*;` resolves `join_semicolon_terminated_segments(...)` etc. unqualified. `tests.rs` already calls `join_semicolon_terminated_segments` unqualified (line ~3592) — this keeps that working.

- [ ] **Step 1: Create `helpers.rs` with the four helpers, widened to `pub(crate)`**

Move these four functions out of `lib.rs` (currently `crates/kali_common/src/lib.rs:526`, `:532`, `:536`, `:545`) into a new `crates/kali_common/src/helpers.rs`, verbatim except prefixing each `fn` with `pub(crate) `:
- `join_semicolon_terminated_segments`
- `join_zero_probe_aliases`
- `join_const_binding_lines`
- `ordered_unique_union`

The file starts with no `use` line if the bodies reference no crate items (they operate on `&str`/slices/`Vec` only — check; if a body references a crate item, add `use crate::*;`). Do **not** add `#[cfg(test)] #[path]` wiring — these helpers have no dedicated tests.

- [ ] **Step 2: Wire the module into the facade**

In `lib.rs`, add `mod helpers;` to the module-declaration block and `pub(crate) use helpers::*;` to the re-export block. (Internal helpers — `pub(crate)`, **not** `pub`; they must not become public API.)

- [ ] **Step 3: Verify**

Run:
```bash
cargo test -p kali_common
cargo build --workspace
cargo clippy -p kali_common --all-targets -- -D warnings
```
Expected: all pass, clippy clean. Then confirm basename match (Task 1 Step 3) — empty diff.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_common/src/helpers.rs crates/kali_common/src/lib.rs
git commit -m "refactor(kali_common): extract shared helpers module [refactor]"
```

---

### Task 3: Extract `registry.rs` (core source-registry types)

**Files:**
- Create: `crates/kali_common/src/registry.rs`, `crates/kali_common/src/registry_tests.rs`
- Modify: `crates/kali_common/src/lib.rs`, `crates/kali_common/src/tests.rs`

**Interfaces:**
- Produces (all currently `pub`, paths preserved via `pub use registry::*;`): `bytewise_shared_memory_is_lock_free`, statics `GLOBAL_INTERNER` and `SOURCE_REGISTRY`, `SourceRegistry` (+ its impl, incl. the private `canonicalize_path` method), `FileId` (+ impls), `SourceFile` (+ impls), `SourceMap` (+ impl + `Default` impl), `format_file_ref`.
- Note: a **different** `SourceMap` lives in `source_map.rs` and stays there. `source_map.rs` references `use super::{FileId, SourceFile, SourceRegistry};` — these resolve unchanged after the move because `pub use registry::*;` re-exports them at the crate root. Do not touch `source_map.rs`.

- [ ] **Step 1: Create `registry.rs` with the core types**

Move verbatim out of `lib.rs` into `crates/kali_common/src/registry.rs` (currently `lib.rs:27`–`:214`, i.e. through `format_file_ref`):
`bytewise_shared_memory_is_lock_free` (`:27`), `GLOBAL_INTERNER` (`:33`), `SOURCE_REGISTRY` (`:37`), `struct SourceRegistry` + `impl SourceRegistry` (`:42`), `struct FileId` + impls (`:98`), `struct SourceFile` + impls (`:120`), `struct SourceMap` + impl + `impl Default` (`:168`), `fn format_file_ref` (`:211`).

Add the imports these types need at the top of `registry.rs` (move them from `lib.rs:14`–`:17`): `use crate::*;` (surfaces `Interner` for `GLOBAL_INTERNER`), plus:
```rust
use ahash::AHashMap;
use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
```
End the file with the test wiring:
```rust
#[cfg(test)]
#[path = "registry_tests.rs"]
mod registry_tests;
```

- [ ] **Step 2: Create `registry_tests.rs` with the 4 registry tests**

Move verbatim out of `tests.rs` into `crates/kali_common/src/registry_tests.rs` (start file with `use crate::*;`):
`test_file_id_basic`, `test_source_file`, `test_source_registry_interning`, `test_bytewise_shared_memory_lock_free_probe_matches_target_atomic_support`.

- [ ] **Step 3: Wire facade**

In `lib.rs` add `mod registry;` and `pub use registry::*;`. Remove the now-unused `use ahash::AHashMap;` / `use once_cell::sync::Lazy;` / `use std::path::{Path, PathBuf};` / `use std::sync::Mutex;` lines from `lib.rs` (clippy will confirm they are unused there now).

- [ ] **Step 4: Verify** — run the three commands from the Standard Extraction Procedure Step 4 and confirm basename match. Expected: all pass, empty diff.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_common/src/registry.rs crates/kali_common/src/registry_tests.rs crates/kali_common/src/lib.rs crates/kali_common/src/tests.rs
git commit -m "refactor(kali_common): extract registry module [refactor]"
```

---

### Task 4: Extract `messages.rs` (lowering-unavailable messages)

Follow the **Standard Extraction Procedure** for module `messages`.

**Functions to move** (8, all `pub const fn`):
`async_class_method_lowering_unavailable_message`, `generator_class_method_lowering_unavailable_message`, `generator_class_method_yield_lowering_unavailable_message`, `generator_class_method_lowering_unavailable_message_for_flavors`, `generator_class_method_yield_lowering_unavailable_message_for_flavors`, `generator_function_lowering_unavailable_message`, `generator_function_yield_lowering_unavailable_message`, `generator_function_lowering_unavailable_message_for_flavors`.

**Tests to move** (7):
`test_async_class_method_lowering_unavailable_message_is_stable`, `test_generator_class_method_lowering_unavailable_message_lists_async_and_sync_variants`, `test_generator_class_method_lowering_unavailable_message_for_flavors_is_stable`, `test_generator_class_method_yield_lowering_unavailable_message_for_flavors_is_stable`, `test_generator_function_lowering_unavailable_message_lists_async_and_sync_variants`, `test_generator_function_lowering_unavailable_message_for_yield_delegation_is_stable`, `test_generator_function_lowering_unavailable_message_for_flavors_is_stable`.

Commit: `refactor(kali_common): extract messages module [refactor]`

---

### Task 5: Extract `process_kill.rs`

Follow the **Standard Extraction Procedure** for module `process_kill`.

**Functions to move** (all `process_kill_zero_probe_*`, ~33): every `pub fn`/`pub const fn` whose name begins with `process_kill_zero_probe_` — including `process_kill_zero_probe_unavailable_message` (currently far from the others at `lib.rs:2628`). Grep to enumerate:
```bash
grep -oE '^pub (const )?fn (process_kill_zero_probe_[a-z_]+)' crates/kali_common/src/lib.rs
```
Move all matches verbatim.

**Tests to move** (21): every `#[test] fn` whose name begins with `test_process_kill_zero_probe_`. Enumerate with:
```bash
grep -oE '^fn (test_process_kill_zero_probe_[a-z_]+)' crates/kali_common/src/tests.rs
```
(One of these — `..._does_not_include_late_process_control_prefix` — calls a `late_*` function; `use crate::*;` resolves it whether `late` is extracted yet or not.)

Commit: `refactor(kali_common): extract process_kill module [refactor]`

---

### Task 6: Extract `object.rs` (object + reflect frozen-callable fixtures)

Follow the **Standard Extraction Procedure** for module `object`.

**Functions to move** (~13):
`object_has_own_frozen_callable_aliases`, `object_has_own_frozen_callable_source`, `reflect_own_keys_frozen_callable_aliases`, `reflect_own_keys_frozen_callable_source`, `object_enumeration_frozen_callable_aliases`, `object_enumeration_frozen_callable_source`, `object_has_own_frozen_callable_condition_source`, `object_has_own_property_call_frozen_callable_aliases`, `object_has_own_property_call_frozen_callable_source`, `object_has_own_property_call_frozen_callable_condition_source`, `object_has_own_combined_frozen_callable_condition_source`, `object_has_own_property_call_source`, `object_has_own_property_call_binding_source`.
(Note: `late_compat_object_has_own_source` is **not** here — it goes in `late.rs`, Task 13.)

**Tests to move** (9):
`test_reflect_own_keys_frozen_callable_aliases_list_all_aliases_in_order`, `test_reflect_own_keys_frozen_callable_source_lists_all_aliases_in_order`, `test_object_has_own_frozen_callable_source_lists_all_aliases_in_order`, `test_object_enumeration_frozen_callable_source_lists_all_aliases_in_order`, `test_object_has_own_frozen_callable_condition_source_lists_all_aliases_in_order`, `test_object_has_own_combined_frozen_callable_condition_source_reuses_both_helpers_once`, `test_object_has_own_property_call_frozen_callable_source_lists_all_aliases_in_order`, `test_object_has_own_property_call_frozen_callable_condition_source_lists_all_aliases_in_order`, `test_object_has_own_property_call_binding_source_is_canonical`.

Commit: `refactor(kali_common): extract object module [refactor]`

---

### Task 7: Extract `number.rs` (number-predicate fixtures)

Follow the **Standard Extraction Procedure** for module `number`.

**Functions to move** (5):
`number_predicates_preamble_source`, `number_predicates_console_log_body_source`, `number_predicates_runtime_source`, `number_predicates_browser_bundle_source`, `number_predicates_test_source`.

**Tests to move** (1):
`test_number_predicates_source_helpers_are_canonical`.

Commit: `refactor(kali_common): extract number module [refactor]`

---

### Task 8: Extract `math.rs` (Math.* fixtures)

Follow the **Standard Extraction Procedure** for module `math`.

**Functions to move** (~44): every `pub fn`/`pub const fn` whose name begins with `math_` (abs/floor_trunc_ceil/round/pow/cbrt/hypot/exp2 families). These are split across two line ranges in `lib.rs` (the `math_pow_*` family is interrupted by the `promise_*` functions) — move **by name**. Enumerate:
```bash
grep -oE '^pub (const )?fn (math_[a-z0-9_]+)' crates/kali_common/src/lib.rs
```

**Tests to move** (21): every `#[test] fn` whose name begins with `test_math_`. Enumerate:
```bash
grep -oE '^fn (test_math_[a-z0-9_]+)' crates/kali_common/src/tests.rs
```

Commit: `refactor(kali_common): extract math module [refactor]`

---

### Task 9: Extract `promise.rs` (Promise.* browser-body fixtures)

Follow the **Standard Extraction Procedure** for module `promise`.

**Functions to move** (4, all `pub const fn`):
`promise_all_settled_browser_body_source`, `promise_race_browser_body_source`, `promise_any_browser_body_source`, `promise_all_browser_body_source`.

**Tests to move** (4):
`test_promise_all_settled_browser_body_source_includes_the_shared_freeze_wrapper_aliases`, `test_promise_race_browser_body_source_includes_the_shared_freeze_wrapper_aliases`, `test_promise_any_browser_body_source_includes_the_shared_freeze_wrapper_aliases`, `test_promise_all_browser_body_source_includes_the_shared_freeze_wrapper_aliases`.

Commit: `refactor(kali_common): extract promise module [refactor]`

---

### Task 10: Extract `array.rs` (Array.from fixtures)

Follow the **Standard Extraction Procedure** for module `array`.

**Functions to move** (6):
`array_from_aliases`, `array_from_source`, `array_from_frozen_callable_aliases`, `array_from_frozen_callable_source`, `array_from_alias_inventory_source`, `array_from_loop_lines`.

**Tests to move** (4):
`test_array_from_aliases_list_all_supported_aliases_in_order`, `test_array_from_frozen_callable_aliases_contains_representative_supported_aliases_and_source_is_canonical`, `test_array_from_alias_inventory_source_reuses_the_shared_helper_sources_once`, `test_array_from_loop_lines_renders_all_aliases_in_order`.

Commit: `refactor(kali_common): extract array module [refactor]`

---

### Task 11: Extract `template_literal.rs`

Follow the **Standard Extraction Procedure** for module `template_literal`.

**Functions to move** (2, both `pub const fn`):
`template_literal_string_iteration_body_source`, `browser_template_literal_string_iteration_body_source`.
(Distinct from the existing `template.rs` module — do not touch `template.rs`.)

**Tests to move** (2):
`test_template_literal_string_iteration_body_source_is_canonical`, `test_browser_template_literal_string_iteration_body_source_is_canonical`.

Commit: `refactor(kali_common): extract template_literal module [refactor]`

---

### Task 12: Extract `collections.rs` (Set/Map constructor fixtures)

Follow the **Standard Extraction Procedure** for module `collections`.

**Functions to move** (10):
`set_constructor_aliases`, `set_constructor_frozen_callable_aliases`, `set_constructor_source`, `set_constructor_iteration_source`, `set_constructor_frozen_callable_source`, `map_constructor_aliases`, `map_constructor_frozen_callable_aliases`, `map_constructor_source`, `map_constructor_iteration_source`, `map_constructor_frozen_callable_source`.

**Tests to move** (2):
`test_set_constructor_aliases_and_frozen_callable_source_are_canonical`, `test_map_constructor_aliases_and_frozen_callable_source_are_canonical`.

Commit: `refactor(kali_common): extract collections module [refactor]`

---

### Task 13: Extract `late.rs` (late-stage hardening fixtures)

Follow the **Standard Extraction Procedure** for module `late`. This module also moves several private module-level `const` arrays that its functions reference.

**Functions to move** (~28): `late_compat_object_has_own_source` (yes, this `late_compat_*` fn lives here, not in `object`) plus every `pub fn`/`pub const fn` beginning with `late_`. Enumerate:
```bash
grep -oE '^pub (const )?fn (late_[a-z0-9_]+)' crates/kali_common/src/lib.rs
```

**Private consts to move** (verbatim, stay private `const` — referenced only within this cluster):
`LATE_PROCESS_CONTROL_PREFIX_SEGMENTS`, `LATE_PROCESS_CONTROL_EXIT_SEGMENTS`, `LATE_PROCESS_CONTROL_SINGLE_QUOTED_PROCESS_SEGMENTS`, `LATE_PROCESS_ENV_MUTATION_SEGMENTS`, `LATE_OBJECT_MODEL_SEGMENTS`, `LATE_OBJECT_MODEL_OWN_PROPERTY_SEGMENTS`, `LATE_THREADED_RUNTIME_SEGMENTS`, `LATE_PERMISSION_ESCALATION_SEGMENTS`.
(If after the move clippy reports any of these consts is referenced from another module, qualify the use site as `crate::<CONST>` — none are expected to be, since each is prefix-matched to this cluster.)

**Tests to move** (18): `test_late_compat_object_has_own_source_lists_representative_aliases_in_order` plus every `#[test] fn` beginning with `test_late_`. Enumerate:
```bash
grep -oE '^fn (test_late_[a-z0-9_]+)' crates/kali_common/src/tests.rs
```

Commit: `refactor(kali_common): extract late module [refactor]`

---

### Task 14: Extract `intl.rs` (Intl fixtures) and empty `tests.rs`

Follow the **Standard Extraction Procedure** for module `intl`, then delete the now-empty `tests.rs`.

**Functions to move** (2):
`broader_intl_aliases`, `broader_intl_source`.

**Private const to move** (verbatim, stays private):
`BROADER_INTL_SEGMENTS`.

**Tests to move** (1):
`test_broader_intl_aliases_and_source_are_canonical`.

- [ ] **Extra Step: Remove the emptied `tests.rs`**

After moving the intl test, `tests.rs` should contain only its `use super::*;` header and no `#[test]` functions. Confirm:
```bash
grep -c '#\[test\]' crates/kali_common/src/tests.rs
```
Expected: `0`. Then delete the file and remove its wiring from `lib.rs` (the `#[cfg(test)] #[path = "tests.rs"] mod tests;` block at the bottom):
```bash
git rm crates/kali_common/src/tests.rs
```

Verify (three commands + basename match), then commit: `refactor(kali_common): extract intl module and remove monolithic tests.rs [refactor]`

---

### Task 15: Finalize and verify the facade

**Files:**
- Modify: `crates/kali_common/src/lib.rs` (cleanup only)

- [ ] **Step 1: Confirm `lib.rs` is a thin facade**

`lib.rs` should now contain only: the crate-level `//!` doc comment, the `mod`/`pub mod` declarations, the `pub use`/`pub(crate) use` re-export block, and no free functions, structs, statics, or consts. Confirm zero leftover definitions:
```bash
grep -cE '^pub (const )?fn |^(const|static) [A-Z]|^pub struct |^pub enum ' crates/kali_common/src/lib.rs
```
Expected: `0`. If non-zero, the leftover item was missed by an earlier task — move it to its module.

- [ ] **Step 2: Confirm no `use crate::*;` survives in a module that needs no crate items**

Run `cargo clippy -p kali_common --all-targets -- -D warnings`. If clippy flags an unused `use crate::*;` (e.g. in `helpers.rs` or `template_literal.rs` if their bodies reference no crate items), delete that line. Re-run until clean.

- [ ] **Step 3: Full workspace verification**

Run:
```bash
cargo test -p kali_common
cargo build --workspace
cargo clippy -p kali_common --all-targets -- -D warnings
```
Expected: all pass, clippy clean.

- [ ] **Step 4: Final basename-set proof**

Run the basename diff (Task 1 Step 3). Expected: empty output (identical 102-test basename set). Also confirm the count:
```bash
cargo test -p kali_common --all-targets -- --list 2>/dev/null | grep -c ': test$'
```
Expected: `102`.

- [ ] **Step 5: Confirm zero consumer changes**

The only files changed in this branch should be under `crates/kali_common/src/` and `docs/superpowers/`. Confirm no other crate was touched:
```bash
git diff --name-only main... | grep -v '^crates/kali_common/' | grep -v '^docs/'
```
Expected: empty output.

- [ ] **Step 6: Commit any cleanup**

```bash
git add -A
git commit -m "refactor(kali_common): finalize facade and verify identical test set [refactor]"
```
(If Steps 1–2 required no changes, skip the commit.)

---

## Self-Review

- **Spec coverage:** registry + helpers + 11 fixture modules (messages, process_kill, object, number, math, promise, array, template_literal, collections, late, intl) cover all 156 functions and 94 tests; existing `interner`/`span`/`source_map`/`template` left untouched (Tasks 3–14). Facade re-exports preserve flat public paths (Tasks 3–14 Step 3 + Task 15). Identical-set proof + count held at 102 (Task 1, Task 15). `kali_test_support` skipped (Global Constraints). Dual-`SourceMap` preserved (Task 3 Interfaces). GLOB/clippy rule (Global Constraints, Task 15 Step 2). All spec sections map to tasks.
- **Placeholder scan:** no TBD/TODO; every task lists exact function names, test names, and counts, plus exact verification commands. Verbatim-move tasks reference the Standard Extraction Procedure with task-specific item lists inline.
- **Type/name consistency:** module names, the four helper signatures (Task 2 Interfaces), and the registry item list (Task 3 Interfaces) are used consistently across tasks. Per-module test counts sum to 94 (+8 existing = 102), matching the baseline.
