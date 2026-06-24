# kali_optimize Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Break `kali_optimize` (one 2,872-line `lib.rs` built around a single 71-method `impl Optimizer`, plus a 8,597-line flat `tests.rs` of 110 tests) into seven small, single-purpose modules with co-located sibling test files and a shared `kali_test_support` dev-dependency — with zero behavior change.

**Architecture:** `lib.rs` becomes a thin facade (crate docs, the `use` import surface, the `HOT_FUNCTION_MINIMUM_WEIGHT` const, module declarations, and `pub use` re-exports of every current `pub` item). The one `impl Optimizer` block splits into seven `impl Optimizer` blocks across domain modules (multiple `impl` blocks for one type are legal in one crate). The supporting `pub(crate)` structs/enums and the 13 free functions move into the module whose responsibility they serve. Every method body moves **verbatim** — no function is cracked into helpers. Tests move into sibling `*_tests.rs` files wired with `#[cfg(test)] #[path = "…"] mod`.

**Tech Stack:** Rust 2021, Cargo workspace, `kali_lir`, `kali_mir`, `kali_common`, `kali_error`, `serde`, `serde_json`; dev (newly added): `kali_test_support`.

## Global Constraints

- **Zero behavior change.** Pure structural refactor. The set of tests that pass is identical before and after, **except** for exactly one deliberately-added fixture test (110 → 111 — see Task 10), recorded explicitly. Function bodies are moved verbatim — never rewritten or split.
- **Green at every commit.** `cargo test -p kali_optimize` must pass after every task. Never commit a red tree.
- **Public API preserved.** Every current `pub` item must keep resolving at its existing path. The facade re-exports them all. The full public surface (7 items) is: types `OptimizationLevel`, `OptimizationReport`, `Optimizer`; and the existing `pub use profile::{ProfileData, ProfileSample, ProfileSampleKind, PROFILE_DATA_VERSION}`. Everything else (`MirLayoutClass`, `MirLayoutSignature`, `MirSpecializationPlan`, `SpecializationPlan`, `BindingEnv`, `FunctionSummary`, `ConstantValue`, `SpecializationTracker`, and all 13 free fns) is crate-internal and becomes `pub(crate)`.
- **Text-movement only.** No method/function body is rewritten or cracked. Cross-referenced items are widened to `pub(crate)` in Task 2, after which calls resolve across modules anywhere in the crate.
- **Crate-root re-export rule.** Each extracted module `<m>` is wired into `lib.rs` as:
  ```rust
  mod <m>;
  pub(crate) use <m>::*;        // surfaces the module's pub(crate) items at crate root
  ```
  plus a `pub use <m>::{PublicType, …};` line for any **public** item the module now owns. `use crate::*;` at the top of every module then sees the crate's `use` import block **and** all crate-root re-exports — this is what makes cross-module `pub(crate)` calls (e.g. `optimize_node`, `fold_binary`, `is_hot_function`) resolve after extraction. If a bare `const` is not surfaced by `use crate::*;`, qualify it at the use site as `crate::HOT_FUNCTION_MINIMUM_WEIGHT` (mechanical, no logic change) — the kali_codegen const-fallback lesson.
- **Test convention.** Unit tests live in sibling `*_tests.rs` files wired via `#[cfg(test)] #[path = "…"] mod …;` — never inline `#[cfg(test)] mod tests { … }` blocks. `#[path]` is resolved relative to the directory of the file containing the `mod` declaration; all these modules are flat top-level files, so it's always the bare filename (`#[path = "driver_tests.rs"]`).
- **No new runtime dependencies.** `kali_test_support` is added as a `dev-dependency` only; it already exists in the workspace.
- **Branch.** Create and work on `refactor/kali-optimize-modularization` off `main`. All paths are relative to repo root `/workspace`.
- **`--list` includes module-path prefixes.** Co-location changes the prefix of every moved test, so a raw `diff` of `--list` output is non-empty *by design*. Prove the invariant by comparing **basenames** (strip the `…::` prefix), as Task 12 does.
- **Verification triad per task:** `cargo build -p kali_optimize` → `cargo test -p kali_optimize` → `cargo clippy -p kali_optimize --all-targets -- -D warnings`. Run clippy **every** task (the kali_runtime lesson: `cargo test` does not gate warnings). If a transient `pub(crate)`-could-be-private style lint fires mid-split, it must still be resolved by Task 12; build + test stay green at every commit.

---

### Task 1: Branch + baseline test snapshot

**Files:**
- Create: `docs/superpowers/baselines/kali_optimize-tests-before.txt`

**Interfaces:**
- Produces: `kali_optimize-tests-before.txt` — the authoritative list of test basenames before refactor, diffed against in Task 12.

- [ ] **Step 1: Create and switch to the refactor branch**
```bash
cd /workspace
git checkout -b refactor/kali-optimize-modularization
git branch --show-current
```
Expected: `refactor/kali-optimize-modularization`.

- [ ] **Step 2: Confirm the suite is green before any change**
```bash
cargo test -p kali_optimize 2>&1 | tail -5
```
Expected: `test result: ok.` with 110 passed and no failures.

- [ ] **Step 3: Snapshot the exact set of test basenames**
```bash
mkdir -p docs/superpowers/baselines
cargo test -p kali_optimize -- --list 2>/dev/null | grep ': test$' | sed -E 's/: test$//; s/^.*:://' | sort > docs/superpowers/baselines/kali_optimize-tests-before.txt
wc -l docs/superpowers/baselines/kali_optimize-tests-before.txt
```
Expected: `110`.

- [ ] **Step 4: Commit the baseline**
```bash
git add docs/superpowers/baselines/kali_optimize-tests-before.txt
git commit -m "chore(kali_optimize): snapshot test baseline [refactor]"
```

---

### Task 2: Widen internal visibility to `pub(crate)` (the enabling step)

**Why first:** Once items live in sibling modules, Rust privacy blocks cross-module access to *private* items. Promoting crate-internal items to `pub(crate)` up front turns every later extraction into pure text movement. This task changes only visibility keywords — no code moves, no behavior change.

**Files:**
- Modify: `crates/kali_optimize/src/lib.rs`

**Interfaces:**
- Produces: every private free `fn` (13 of them), every private method in `impl Optimizer` (61 of them; the 10 already-`pub` methods are untouched), and every private struct/enum after the impl block (`MirLayoutClass`, `MirLayoutSignature`, `MirSpecializationPlan`, `SpecializationPlan`, `BindingEnv`, `FunctionSummary`, `ConstantValue`, `SpecializationTracker`) — all become `pub(crate)`. Their fields must also be widened where read cross-module (see Step 3).

- [ ] **Step 1: Promote every private top-level free fn**
```bash
cd /workspace
awk '/^fn /{sub(/^fn /, "pub(crate) fn ")} {print}' crates/kali_optimize/src/lib.rs > /tmp/opt_vis.rs && mv /tmp/opt_vis.rs crates/kali_optimize/src/lib.rs
grep -c "^pub(crate) fn " crates/kali_optimize/src/lib.rs
```
Expected: `13`.

- [ ] **Step 2: Promote every private method inside `impl Optimizer`**
The 4-space-indented private methods all live in the single `impl Optimizer` block (≈65–2358). Promote each private `    fn ` to `    pub(crate) fn ` (the 10 already-`pub fn` are untouched):
```bash
cd /workspace
awk '/^    fn /{sub(/^    fn /, "    pub(crate) fn ")} {print}' crates/kali_optimize/src/lib.rs > /tmp/opt_vis.rs && mv /tmp/opt_vis.rs crates/kali_optimize/src/lib.rs
grep -cE "^    pub\(crate\) fn " crates/kali_optimize/src/lib.rs
```
Expected: `61`.

- [ ] **Step 3: Promote the private structs/enums and their fields**
For each of `MirLayoutClass`, `MirLayoutSignature`, `MirSpecializationPlan`, `SpecializationPlan`, `BindingEnv`, `FunctionSummary`, `ConstantValue`, `SpecializationTracker`: change the leading `struct`/`enum` keyword to `pub(crate) struct`/`pub(crate) enum`, and prefix each of their fields (and enum variants stay public-by-enum) with `pub(crate)` where the field is read or written from what will become a different module. When unsure, widen the field — it is crate-internal regardless. The `Optimizer` struct's fields (`level`, `max_specializations`, `profile_data`, etc.) are read by methods moving to other modules, so widen them to `pub(crate)` too.
```bash
cd /workspace
grep -nE "^(pub\(crate\) )?(struct|enum) (MirLayoutClass|MirLayoutSignature|MirSpecializationPlan|SpecializationPlan|BindingEnv|FunctionSummary|ConstantValue|SpecializationTracker|Optimizer)" crates/kali_optimize/src/lib.rs
```
Expected: all 9 shown as `pub(crate)` (`Optimizer` stays `pub`, but its fields become `pub(crate)`).

- [ ] **Step 4: Build, test, clippy — nothing moved yet, must stay green**
```bash
cargo build -p kali_optimize 2>&1 | tail -3
cargo test -p kali_optimize 2>&1 | tail -5
cargo clippy -p kali_optimize --all-targets -- -D warnings 2>&1 | tail -5
```
Expected: builds; `test result: ok.` 110; no clippy errors. (If clippy flags a now-`pub(crate)` item that is still only used in one place, leave it — it will be used cross-module after extraction; resolve any residual lint in Task 12.)

- [ ] **Step 5: Commit**
```bash
git add crates/kali_optimize/src/lib.rs
git commit -m "refactor(kali_optimize): widen internal visibility to pub(crate) [refactor]"
```

---

### Task 3: Extract `driver.rs` (public surface + recursion core)

**Files:**
- Create: `crates/kali_optimize/src/driver.rs`
- Modify: `crates/kali_optimize/src/lib.rs`

**Interfaces:**
- Produces (re-exported public): `OptimizationLevel`, `OptimizationReport`, `Optimizer`. Moves the three public type definitions and the `impl Optimizer` block holding the public ctors/accessors and the recursive walk. Every other module's `impl Optimizer` methods are reached from here via `self.<method>` and resolve once the modules are wired.

- [ ] **Step 1:** Create `driver.rs` with header `use crate::*;`. Move into it:
  - `pub enum OptimizationLevel` (≈22) and its impls, if any.
  - `pub struct OptimizationReport` (≈36) and its impls, if any.
  - `pub struct Optimizer` (≈59) with its (now `pub(crate)`) fields.
  - An `impl Optimizer { … }` block containing **only**: `new`, `with_max_specializations`, `max_specializations`, `profile_data`, `optimization_report`, `with_profile_data`, `optimize_program`, `optimize_program_with_report`, `optimize_program_with_mir`, `optimize_program_with_mir_and_report`, `optimize_program_internal`, `optimize_node`, `optimize_sequence`, `is_cse_candidate`.
- [ ] **Step 2:** In `lib.rs`, keep the `HOT_FUNCTION_MINIMUM_WEIGHT` const and the crate `use` block; add:
```rust
mod driver;
pub(crate) use driver::*;
pub use driver::{OptimizationLevel, OptimizationReport, Optimizer};
```
- [ ] **Step 3:** Build + test + clippy green, 110.
```bash
cargo build -p kali_optimize 2>&1 | tail -3
cargo test -p kali_optimize 2>&1 | tail -5
cargo clippy -p kali_optimize --all-targets -- -D warnings 2>&1 | tail -5
```
Expected: green, 110, no clippy errors. (If `optimize_node` calls a method that has not moved yet, it still resolves: every method is currently `pub(crate)` on `Optimizer` and reachable via `self`. Extraction order does not matter because all method blocks target the same `Optimizer` type.)
- [ ] **Step 4:** `git add -A && git commit -m "refactor(kali_optimize): extract driver module [refactor]"`

---

### Task 4: Extract `constant_fold.rs` (constant folding + algebraic identities)

**Files:**
- Create: `crates/kali_optimize/src/constant_fold.rs`
- Modify: `crates/kali_optimize/src/lib.rs`

**Interfaces:**
- Produces (re-exported `pub(crate)`): `ConstantValue` and the folding/parsing free fns. Consumed by `driver::optimize_node` and `specialize`/`object_fold` via `self.optimize_constant_expression(…)` and the free fns.

- [ ] **Step 1:** Create `constant_fold.rs` with header `use crate::*;`. Move into it:
  - `pub(crate) enum ConstantValue` (≈2509) and `impl ConstantValue` (≈2553).
  - An `impl Optimizer { … }` block with: `optimize_constant_expression`, `optimize_algebraic_identity`.
  - The free fns: `is_zero_constant`, `is_one_constant`, `literal_value`, `parse_literal_text`, `fold_unary`, `fold_binary`, `literal_text`, `parse_number_literal`, `parse_string_literal`, `parse_regex_literal`.
- [ ] **Step 2:** In `lib.rs`:
```rust
mod constant_fold;
pub(crate) use constant_fold::*;
```
- [ ] **Step 3:** Build + test + clippy green, 110.
- [ ] **Step 4:** `git add -A && git commit -m "refactor(kali_optimize): extract constant_fold module [refactor]"`

---

### Task 5: Extract `specialize.rs` (MIR-layout call-site specialization)

**Files:**
- Create: `crates/kali_optimize/src/specialize.rs`
- Modify: `crates/kali_optimize/src/lib.rs`

**Interfaces:**
- Produces (re-exported `pub(crate)`): `MirLayoutClass`, `MirLayoutSignature`, `MirSpecializationPlan`, `SpecializationPlan`, `SpecializationTracker`, and the specialization methods. Consumed by `driver` (`optimize_program_with_mir*` → `specialize_*`).

- [ ] **Step 1:** Create `specialize.rs` with header `use crate::*;`. Move into it:
  - The structs/enums `pub(crate) enum MirLayoutClass` (≈2358) + both `impl MirLayoutClass` blocks (≈2366, ≈2475), `pub(crate) struct MirLayoutSignature` (≈2379) + `impl` (≈2384), `pub(crate) struct MirSpecializationPlan` (≈2398) + `impl` (≈2403), `pub(crate) struct SpecializationPlan` (≈2488), `pub(crate) struct SpecializationTracker` (≈2524) + `impl` (≈2529).
  - An `impl Optimizer { … }` block with: `specialize_layout_bindings`, `extract_const_binding`, `is_specializable_binding`, `specialize_mir_call_sites`, `specialize_mir_call_site`, `clone_specialized_function`, `specialized_function_name`, `argument_has_concrete_layout`, `argument_has_concrete_shape`, `specialization_signature_with_mir`, `object_literal_signature`, `array_literal_signature`, `object_property_signature`, `build_specialization_plan`, `collect_specialization_plan`, `specialization_signature`, `call_signature`.
- [ ] **Step 2:** In `lib.rs`:
```rust
mod specialize;
pub(crate) use specialize::*;
```
- [ ] **Step 3:** Build + test + clippy green, 110.
- [ ] **Step 4:** `git add -A && git commit -m "refactor(kali_optimize): extract specialize module [refactor]"`

---

### Task 6: Extract `inline.rs` (inlining, dead-code pruning, hotness)

**Files:**
- Create: `crates/kali_optimize/src/inline.rs`
- Modify: `crates/kali_optimize/src/lib.rs`

**Interfaces:**
- Produces (re-exported `pub(crate)`): `FunctionSummary` and the inlining/pruning methods. `is_hot_function`/`inline_threshold_for_function` read `crate::HOT_FUNCTION_MINIMUM_WEIGHT` — qualify as `crate::HOT_FUNCTION_MINIMUM_WEIGHT` if `use crate::*;` does not surface the bare const (kali_codegen const-fallback).

- [ ] **Step 1:** Create `inline.rs` with header `use crate::*;`. Move into it:
  - `pub(crate) struct FunctionSummary` (≈2498).
  - An `impl Optimizer { … }` block with: `optimize_call_site`, `function_summary`, `extract_inline_body`, `count_subtree_nodes`, `contains_call_target`, `collect_call_targets`, `prune_dead_top_level_functions`, `inline_call_site`, `clone_subtree_with_substitution`, `inline_threshold_for_function`, `is_hot_function`, `profile_has_hot_branch_or_layout_hints`.
- [ ] **Step 2:** In `lib.rs`:
```rust
mod inline;
pub(crate) use inline::*;
```
- [ ] **Step 3:** Build + test + clippy green, 110. If the build errors on a bare `HOT_FUNCTION_MINIMUM_WEIGHT`, qualify it at the use site as `crate::HOT_FUNCTION_MINIMUM_WEIGHT` (mechanical, no logic change) and rebuild.
- [ ] **Step 4:** `git add -A && git commit -m "refactor(kali_optimize): extract inline module [refactor]"`

---

### Task 7: Extract `object_fold.rs` (compile-time Object.* folding)

**Files:**
- Create: `crates/kali_optimize/src/object_fold.rs`
- Modify: `crates/kali_optimize/src/lib.rs`

**Interfaces:**
- Produces (re-exported `pub(crate)`): `BindingEnv` and the object-folding methods. Consumed by `driver::optimize_node`.

- [ ] **Step 1:** Create `object_fold.rs` with header `use crate::*;`. Move into it:
  - `pub(crate) struct BindingEnv` (≈2493).
  - An `impl Optimizer { … }` block with: `fold_object_has_own_call`, `fold_object_enumeration_call`, `fold_object_from_entries_call`, `fold_object_enumeration_calls`, `ordered_object_literal_properties`, `resolve_constant_binding`, `is_object_freeze_call`, `collect_constant_bindings`, `collect_constant_bindings_into`.
- [ ] **Step 2:** In `lib.rs`:
```rust
mod object_fold;
pub(crate) use object_fold::*;
```
- [ ] **Step 3:** Build + test + clippy green, 110.
- [ ] **Step 4:** `git add -A && git commit -m "refactor(kali_optimize): extract object_fold module [refactor]"`

---

### Task 8: Extract `layout.rs` and `helpers.rs`; verify the facade is thin

**Files:**
- Create: `crates/kali_optimize/src/layout.rs`
- Create: `crates/kali_optimize/src/helpers.rs`
- Modify: `crates/kali_optimize/src/lib.rs`

**Interfaces:**
- Produces (re-exported `pub(crate)`): the layout-folding methods (`layout.rs`) and the LIR-construction + name/signature helpers (`helpers.rs`). After this task, the `impl Optimizer` block in `lib.rs` is empty and `lib.rs` is a pure facade.

- [ ] **Step 1:** Create `layout.rs` with header `use crate::*;`. Move into it an `impl Optimizer { … }` block with: `fold_layout_member_access`, `object_literal_field`, `array_literal_element`, `array_literal_length`, `constant_array_index`, `is_object_literal`, `is_array_literal`.
- [ ] **Step 2:** Create `helpers.rs` with header `use crate::*;`. Move into it:
  - An `impl Optimizer { … }` block with: `clone_boolean_literal`, `member_access_name`, `normalized_member_access_name`, `canonicalize_optional_chain_member_access_name`, `canonicalize_bracketed_member_access_name`, `constant_property_key`, `clone_string_literal`, `push_array_literal`, `push_object_literal`, `object_property_order_key`.
  - The free fns: `node_signature`, `literal_signature`, `string_literal_signature`.
- [ ] **Step 3:** In `lib.rs`:
```rust
mod layout;
pub(crate) use layout::*;
mod helpers;
pub(crate) use helpers::*;
```
- [ ] **Step 4: Confirm `lib.rs` is now a thin facade.** It should contain only: crate docs, the `use` import block, `const HOT_FUNCTION_MINIMUM_WEIGHT`, `mod profile; pub use profile::{…};`, the seven `mod …; pub(crate) use …::*;` blocks, the one `pub use driver::{OptimizationLevel, OptimizationReport, Optimizer};`, and the `#[cfg(test)] #[path = "tests.rs"] mod tests;` wiring. No `impl Optimizer` block, no struct/enum/fn definitions remain.
```bash
cd /workspace
grep -nE "^(    )?(pub\(crate\) )?(fn|impl Optimizer|struct|enum) " crates/kali_optimize/src/lib.rs
wc -l crates/kali_optimize/src/lib.rs
```
Expected: no `fn`/`impl Optimizer`/`struct`/`enum` definitions listed (only the `pub enum/struct` lines, if any, are gone — they moved to `driver.rs`); `lib.rs` well under ~90 lines.
- [ ] **Step 5:** Build + test + clippy green, 110.
- [ ] **Step 6:** `git add -A && git commit -m "refactor(kali_optimize): extract layout + helpers; lib.rs is facade [refactor]"`

---

### Task 9: `cargo fmt` normalization

**Files:**
- Modify: all `crates/kali_optimize/src/*.rs`

- [ ] **Step 1:** Normalize formatting after the moves:
```bash
cd /workspace
cargo fmt -p kali_optimize
```
- [ ] **Step 2:** Build + test + clippy green, 110.
```bash
cargo test -p kali_optimize 2>&1 | tail -5
cargo clippy -p kali_optimize --all-targets -- -D warnings 2>&1 | tail -5
```
- [ ] **Step 3:** `git add -A && git commit -m "style(kali_optimize): cargo fmt normalization after module split [refactor]"`

---

### Task 10: Add `kali_test_support` dev-dep, crate-local `test_support`, and one fixture test

**Why a fixture test:** `kali_optimize` has **zero** filesystem/tempdir tests, so the new `kali_test_support` dev-dep has nothing to convert and would be flagged "declared-but-unused" by the final review. Replicating the user-approved kali_codegen decision: add **one** new test that exercises `kali_test_support::fixtures`, taking the count **110 → 111**. Separately, the test file already carries a large block of LIR-builder helpers (`literal`, `build_*`, ≈lines 5–646 of `tests.rs`) shared across tests — those move into a crate-local `test_support` module so every co-located `*_tests.rs` can share them.

**Files:**
- Modify: `crates/kali_optimize/Cargo.toml` (add dev-dependency)
- Create: `crates/kali_optimize/src/test_support.rs`
- Create: `crates/kali_optimize/src/fixture_support_tests.rs`
- Modify: `crates/kali_optimize/src/lib.rs` (declare both modules under `cfg(test)`)
- Modify: `crates/kali_optimize/src/tests.rs` (drop moved helpers; import from `test_support`)

**Interfaces:**
- Produces (all `pub(crate)`, available to every `*_tests.rs`): the LIR-builder helpers migrated from `tests.rs` — `literal`, `build_hot_add_program`, `build_short_circuit_program`, `build_object_enumeration_call`, `build_object_string_enumeration_call`, `build_bracketed_global_this_object_string_enumeration_call`, `build_global_this_object_string_enumeration_call`, `build_object_from_entries_call`, `build_global_this_object_from_entries_call`, `build_bracketed_global_this_object_from_entries_call`, `build_bracketed_global_this_object_enumeration_call`, `build_reflect_own_keys_call`, `build_bracketed_reflect_own_keys_call`, `build_global_this_reflect_own_keys_call`, `build_object_freeze_call`, `build_object_has_own_callee`, `build_object_has_own_call`, `build_bracketed_object_has_own_call`, `build_const_bound_object_has_own_call`, `build_const_bound_reflect_own_keys_call`, `build_alias_bound_reflect_own_keys_call`, `build_const_bound_object_enumeration_call`, `build_wrapped_const_bound_object_enumeration_call`, `build_alias_bound_object_enumeration_call` (plus any other non-`#[test]` helper fn currently in `tests.rs`).

- [ ] **Step 1: Add the dev-dependency.** In `crates/kali_optimize/Cargo.toml`, add a `[dev-dependencies]` section (none exists yet):
```toml
[dev-dependencies]
kali_test_support = { workspace = true }
```
- [ ] **Step 2: Enumerate the helper fns to migrate.**
```bash
cd /workspace
awk '/#\[test\]/{skip=1} /^fn /{ if(!skip) print NR": "$0; skip=0 } /^}/{skip=0}' crates/kali_optimize/src/tests.rs | head -60
```
This lists the non-`#[test]` `fn`s (the builder helpers at the top of the file). Verify the set matches the Interfaces list above.
- [ ] **Step 3: Create `test_support.rs`.** Move every non-`#[test]` helper fn here verbatim, each declared `pub(crate)`. Header:
```rust
//! kali_optimize-specific test builders (compiled under cfg(test)).
use crate::*;
use kali_lir::{LirBuilder, LirNodeKind};
```
(Keep only the `use`s the helpers actually reference — `LirBuilder`, `LirNodeKind`, `LirProgram`, `LirNodeId` come via `kali_lir`/`use crate::*;`.)
- [ ] **Step 4: Create `fixture_support_tests.rs`** — the one added test that exercises the dev-dep:
```rust
//! Smoke test that wires kali_test_support's filesystem fixtures so the
//! dev-dependency is genuinely exercised (kali_optimize has no other fs tests).
use kali_test_support::fixtures;

#[test]
fn kali_test_support_fixtures_round_trip_files() {
    // fixtures::tempdir() -> tempfile::TempDir; write_file(dir: &Path, rel, contents) -> PathBuf
    let dir = fixtures::tempdir();
    let path = fixtures::write_file(dir.path(), "profile.json", "{\"version\":1}");
    let contents = std::fs::read_to_string(&path).expect("written fixture file is readable");
    assert_eq!(contents, "{\"version\":1}");
}
```
The signatures above are verified against `crates/kali_test_support/src/lib.rs`: `tempdir() -> tempfile::TempDir`, `write_file(dir: &Path, rel: &str, contents: &str) -> PathBuf` (hence `dir.path()`, since `TempDir` does not deref to `Path`). The goal is one passing test that calls into `kali_test_support::fixtures`.
- [ ] **Step 5: Declare both modules under cfg(test)** in `crates/kali_optimize/src/lib.rs`, just above the existing `tests` wiring:
```rust
#[cfg(test)]
#[path = "test_support.rs"]
mod test_support;

#[cfg(test)]
#[path = "fixture_support_tests.rs"]
mod fixture_support_tests;
```
- [ ] **Step 6: Update `tests.rs`.** Delete the moved helper fns from `tests.rs`; at the top replace `use super::*;` with:
```rust
use crate::*;
use crate::test_support::*;
use kali_lir::{LirBuilder, LirNodeKind};
use std::time::Instant;
```
Leave all `#[test]` fns unchanged (they call the same helper names, now resolved from `test_support`).
- [ ] **Step 7: Build and test.**
```bash
cargo test -p kali_optimize 2>&1 | tail -5
cargo clippy -p kali_optimize --all-targets -- -D warnings 2>&1 | tail -5
```
Expected: `test result: ok.` **111** (110 original + the one fixture test); no clippy errors.
- [ ] **Step 8: Commit.**
```bash
git add crates/kali_optimize/Cargo.toml crates/kali_optimize/src/lib.rs crates/kali_optimize/src/test_support.rs crates/kali_optimize/src/fixture_support_tests.rs crates/kali_optimize/src/tests.rs Cargo.lock
git commit -m "refactor(kali_optimize): add test_support + kali_test_support fixture test [refactor]"
```

---

### Task 11: Split `tests.rs` into sibling `*_tests.rs` per module

**Files:**
- Create: `driver_tests.rs`, `constant_fold_tests.rs`, `specialize_tests.rs`, `inline_tests.rs`, `object_fold_tests.rs`, `layout_tests.rs` (and `helpers_tests.rs` only if any test exercises a helper directly — likely none; helpers are covered transitively).
- Modify: each corresponding source module (add the `#[cfg(test)] #[path] mod` wiring).
- Delete (at end): `crates/kali_optimize/src/tests.rs` and its facade wiring.

**Classification rule:** For each `#[test]` fn, read its body and assign it to the module whose behavior it exercises. Name-prefix guide (verify against the body when ambiguous):

| Destination test file       | Test clusters (by name)                                                                                  |
|-----------------------------|----------------------------------------------------------------------------------------------------------|
| `driver_tests.rs`           | `optimizer_carries_normalized_profile_data`, `optimization_report_distinguishes_profile_usage_states`, `*specialization_cap*` (driver-level budget plumbing) |
| `constant_fold_tests.rs`    | `fast_keeps_binary_expressions_opaque`, `release_constant_folds_*`, `release_advanced_eliminates_algebraic_identities`, `release_advanced_eliminates_division_by_one`, `release_eliminates_constant_branches`, `release_eliminates_duplicate_*` |
| `inline_tests.rs`           | `release_inlines_simple_function_calls`, `release_advanced_prunes_dead_inlined_functions`, `hot_function_profile_data_expands_inlining_budget`, `hot_branch_*`, `hot_layout_*`, `profile_guided_optimization_benchmark_*` |
| `object_fold_tests.rs`      | `*folds_object_keys/entries/values*`, `*folds_object_from_entries*`, `*folds_object_enumeration*`, `*folds_reflect_own_keys*`, `*folds_object_has_own*`, `*object_enumeration_calls_over_*` |
| `layout_tests.rs`           | `release_specializes_const_object_property_access`, `release_specializes_object_literal_property_order_canonicalization`, `release_specializes_const_array_element_access` (these fold member/index access against literals — `layout.rs` domain) |
| `specialize_tests.rs`       | the remaining `release_specializes_*` / `release_*_specializes_*` / `*reuses_generic_specializations*` / `*reexport*` / `release_respects_zero_specialization_budget*` (MIR-layout call-site specialization) |

When a test drives the full pipeline, place it with the **most specific** surface it asserts on. When still ambiguous, place it with the source module whose method the test names most directly.

**Renaming rule:** Keep test names **identical** so the Task 12 basename diff is empty (the one expected addition is `kali_test_support_fixtures_round_trip_files` from Task 10). If a collision arises after moving, rename minimally and record every rename for Task 12.

- [ ] **Step 1: Worklist.** Confirm the count to place:
```bash
cd /workspace
grep -c '#\[test\]' crates/kali_optimize/src/tests.rs
```
Expected: 110 (the fixture test is already in its own file from Task 10).
- [ ] **Step 2:** For each destination module `<m>`, create `<m>_tests.rs` with header:
```rust
use crate::*;
use crate::test_support::*;
use kali_lir::{LirBuilder, LirNodeKind};
```
Add `use std::time::Instant;` only to the file that receives `profile_guided_optimization_benchmark_*` (it uses `Instant`). Trim each file's `use` block to only what its tests reference.
- [ ] **Step 3:** Move the classified tests into their destination files. Wire each into its source module by adding at the **bottom of the source file**, e.g. in `constant_fold.rs`:
```rust
#[cfg(test)]
#[path = "constant_fold_tests.rs"]
mod constant_fold_tests;
```
- [ ] **Step 4:** Move tests in batches by destination module, running after each and committing per module:
```bash
cargo test -p kali_optimize 2>&1 | tail -5
git add -A && git commit -m "test(kali_optimize): co-locate <module> tests [refactor]"
```
Expected after each batch: green, and the running total of co-located + remaining-in-`tests.rs` tests stays at 111.
- [ ] **Step 5:** When `tests.rs` has no `#[test]` fns left, delete it and remove its wiring:
```bash
rm crates/kali_optimize/src/tests.rs
```
Remove the `#[cfg(test)] #[path = "tests.rs"] mod tests;` line from `lib.rs`.
- [ ] **Step 6:** `cargo test -p kali_optimize 2>&1 | tail -5` → green, 111. `cargo clippy -p kali_optimize --all-targets -- -D warnings` → no errors.
- [ ] **Step 7:** `git add -A && git commit -m "test(kali_optimize): remove monolithic tests.rs [refactor]"`

---

### Task 12: Final verification, lint, and baseline diff

**Files:**
- Create: `docs/superpowers/baselines/kali_optimize-tests-after.txt`
- Create: `docs/superpowers/baselines/kali_optimize-tests-renames.md`

- [ ] **Step 1: Regenerate the after-snapshot (basenames, prefix stripped):**
```bash
cd /workspace
cargo test -p kali_optimize -- --list 2>/dev/null | grep ': test$' | sed -E 's/: test$//; s/^.*:://' | sort > docs/superpowers/baselines/kali_optimize-tests-after.txt
```
- [ ] **Step 2: Diff before vs after (basename sets).** The only difference must be the single added fixture test:
```bash
diff docs/superpowers/baselines/kali_optimize-tests-before.txt docs/superpowers/baselines/kali_optimize-tests-after.txt
wc -l docs/superpowers/baselines/kali_optimize-tests-*.txt
```
Expected: a single `>` line `kali_test_support_fixtures_round_trip_files`; before=110, after=111. Record the result in `docs/superpowers/baselines/kali_optimize-tests-renames.md` (note "No renames — basename set identical apart from the one deliberately-added `kali_test_support_fixtures_round_trip_files` fixture test, per the user-approved kali_codegen-style deviation."; if any real renames occurred, list every before→after and why).
- [ ] **Step 3: Format and lint:**
```bash
cargo fmt -p kali_optimize
cargo clippy -p kali_optimize --all-targets -- -D warnings 2>&1 | tail -15
```
Expected: no clippy errors.
- [ ] **Step 4: Full-workspace sanity** (kali_optimize feeds downstream crates):
```bash
cargo build --workspace 2>&1 | tail -5
cargo test -p kali_optimize 2>&1 | tail -5
```
Expected: workspace builds; kali_optimize tests green, 111.
- [ ] **Step 5: Per-file size check** (confirm the monolith is gone):
```bash
find crates/kali_optimize/src -name '*.rs' | xargs wc -l | sort -rn | head -25
```
Expected: no single source file near the old 2,872 lines; `lib.rs` is a thin facade (well under ~90 lines); the largest source files are the domain modules, each holding one `impl Optimizer` block.
- [ ] **Step 6: Commit baselines:**
```bash
git add docs/superpowers/baselines/kali_optimize-tests-after.txt docs/superpowers/baselines/kali_optimize-tests-renames.md
git commit -m "test(kali_optimize): record post-refactor baseline + renames [refactor]"
```
- [ ] **Step 7: STOP for review.** Summarize: per-file line counts before/after, the rename mapping (the one added fixture test), and confirmation that the suite (111) and full workspace are green and clippy-clean. Recommend a whole-branch opus review before merge. Then update the `kali-crate-modularization` memory to mark `kali_optimize` done (4th crate) and note the next candidate crate (`kali_common` / `kali_npm` / `kali_cli`).

---

## Self-Review Notes (for the implementer)

- **All methods move intact.** `kali_optimize` has no kali_runtime-scale mega-function; the largest methods (`optimize_call_site` ≈174 lines, `optimize_algebraic_identity` ≈157, `specialize_mir_call_site` ≈121, `fold_object_enumeration_call` ≈113, `fold_binary` ≈100) are relocated byte-for-byte. Cracking any of them into helpers is a separate, out-of-scope logic refactor (mirrors how kali_codegen deferred cracking `emit_call`).
- **Extraction order is independent.** Because every method targets the same `Optimizer` type and all are `pub(crate)` after Task 2, a method moved in Task 4 can freely call one not moved until Task 8 — it resolves via `self`. So the suite stays green at every commit regardless of module order.
- If a moved item references a name that compiled only because everything lived in one module and you missed it in Task 2, fix it by adding `pub(crate)` to *that* item — never by altering a function body.
- `use crate::*;` is used in **every** extracted module — it surfaces the crate's `use` import block **and** all crate-root re-exports (the `pub(crate) use <m>::*;` lines). That is what makes cross-module free-fn calls (`fold_binary`, `is_zero_constant`, `node_signature`, …) and cross-module method calls resolve after extraction. If a bare `const` (only `HOT_FUNCTION_MINIMUM_WEIGHT` exists) isn't surfaced, qualify it as `crate::HOT_FUNCTION_MINIMUM_WEIGHT` at the use site.
- **Drop `use crate::*;` from any module that references no crate items** (the kali_runtime clippy lesson). Here `test_support.rs` *does* reference crate items (`LirProgram`, `Optimizer`, etc.) so it keeps the header; but verify with clippy and remove the header if it turns out unused.
- If a `pub(crate) use <module>::*;` glob trips an `unused_imports`/clippy warning under `-D warnings` (nothing outside the module references it), replace that glob with an explicit `pub(crate) use <module>::{the, names, used};` list — never drop a re-export a caller depends on. Resolve at Task 9 or Task 12.
- The `#[path]` attribute for a sibling test file is always the bare filename (all modules are flat top-level files): `#[path = "specialize_tests.rs"]`.
- Do not change any `Cargo.toml` dependency versions; the only manifest edit is the `kali_test_support` dev-dependency added in Task 10.
- Keep test names identical through the split (the one intended addition is the Task 10 fixture test); that empty-apart-from-one diff in Task 12 is the strongest proof of zero behavior change.
- Line numbers (≈) are from the pre-refactor `lib.rs`/`tests.rs` and drift as items are removed — locate items by **name**, using the numbers only as a starting hint.
