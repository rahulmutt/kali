# kali_npm src test-monolith modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split kali_npm's three multi-concern co-located `src/*_tests.rs` unit-test monoliths into thin facades + per-concern `#[path] mod` submodules, by pure verbatim code-motion.

**Architecture:** Each monolith `src/<name>_tests.rs` is declared from a product module via `#[cfg(test)] #[path = "<name>_tests.rs"] mod <name>_tests;`. We turn each into a facade that keeps its original `use` line(s) and one `#[path = "<name>_tests/<group>.rs"] mod <group>;` per group, and we move each `#[test]` fn **verbatim** into the matching `src/<name>_tests/<group>.rs`, each of which opens with `use super::*;`. No product code changes; the compiled test set is byte-for-byte identical.

**Tech Stack:** Rust 2021, cargo, kali workspace.

**Reusable tooling (optional, recommended):** `.superpowers/sdd/move_fns.py` automates each split (run from `crates/kali_npm`); it drains the `#[test]` fns into submodules, auto-retains any non-`#[test]` module-level fn in the facade (there are none here), and appends the `#[path] mod` decls. `.superpowers/sdd/verify.py` proves `{name: body}` from the original == from the submodules. Manual code-motion that produces the exact files below is equally acceptable.

## Global Constraints

- **Pure verbatim code-motion, zero behavior change.** No new product code, no new tests, no renamed tests, no reformatting of moved bodies. Move each `#[test]` fn exactly as written.
- **Facades drain to 0 module-level fns.** The only things retained on each facade are its original `use` line(s) and the new `#[path] mod` declarations. (All three files have **0** non-`#[test]` module-level fns.)
- **Submodule header:** every new `src/<name>_tests/<group>.rs` begins with exactly `use super::*;` and nothing else before the first moved fn. (Submodules reach all crate symbols and the facade's retained `use` imports through this glob — proven 0-warning in kali_optimize/kali_types.)
- **Test count is the invariant.** kali_npm's lib test suite is **45 tests** before and after; per-file filters must report: install_tests 15, resolve_tests 7, validate_tests 14. (The three names are not substrings of one another.)
- **No `pub`/`pub(crate)` widening, no `include_*!` pins** — verified 0 of each across all three files; no signature changes needed.
- **Product siblings unchanged:** `install.rs` (decl at line 1106), `resolve.rs` (decl at line 393), `validate.rs` (decl at line 296) keep their existing `#[cfg(test)] #[path = "F_tests.rs"] mod F_tests;` decls. `manifest_tests.rs` and `registry_tests.rs` are out of scope and stay whole.
- **Build gate:** `cargo build -p kali_npm --tests` stays at **0 warnings** (baseline = 0).
- **`cargo fmt --check`** — accept known fmt nits per series convention (do not reformat moved bodies to satisfy it).
- **Commits:** one `refactor(kali_npm): split <name>_tests.rs into per-concern test submodules [refactor]` per task. Local-main ff-merge only; no origin push.

## Before starting (once)

- Branch: work on `refactor/kali_npm-modularization` off main; confirm baseline green (0 warnings, 45 lib tests) before any move.
- **Capture pre-move snapshots** of all three in-scope files into a fixed scratch dir (outside the repo), e.g. `cp crates/kali_npm/src/{install,resolve,validate}_tests.rs <snapshot>/` where `<snapshot>` = `/tmp/claude-1000/-workspace/kali_npm_split_scratch/orig`. The `<snapshot>/<F>_tests.rs` files are the byte-identity baseline used by every `verify.py` step below.

---

### Task 1: Split `install_tests.rs` (15 tests → 4 submodules)

**Files:**
- Create: `crates/kali_npm/src/install_tests/rejections.rs`
- Create: `crates/kali_npm/src/install_tests/reconciliation.rs`
- Create: `crates/kali_npm/src/install_tests/lifecycle.rs`
- Create: `crates/kali_npm/src/install_tests/traversal.rs`
- Modify: `crates/kali_npm/src/install_tests.rs` (reduce to facade)
- Unchanged: `crates/kali_npm/src/install.rs` (`#[cfg(test)] #[path = "install_tests.rs"] mod install_tests;` at line 1106 stays)

**Interfaces:**
- Consumes: nothing from other tasks.
- Produces: nothing other tasks depend on. (Each task is independent.)

- [ ] **Step 1: Confirm baseline count**

Run: `cargo test -p kali_npm install_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 15 passed; ...`

- [ ] **Step 2: Create `rejections.rs`**

Create `crates/kali_npm/src/install_tests/rejections.rs` starting with `use super::*;`, then move these fns **verbatim** from `install_tests.rs` (with their full bodies):
- `install_rejects_allow_scripts_without_effective_npm_work`
- `install_rejects_allow_scripts_for_jsr_targets`
- `install_rejects_allow_scripts_for_raw_url_targets`
- `install_rejects_dev_without_explicit_target`
- `install_rejects_dev_for_raw_url_targets`
- `install_rejects_versioned_registry_targets`

- [ ] **Step 3: Create `reconciliation.rs`**

Create `crates/kali_npm/src/install_tests/reconciliation.rs` starting with `use super::*;`, then move verbatim:
- `install_reconciles_raw_urls_from_source_import_map_rewrites`
- `install_is_idempotent_for_unchanged_raw_url_graph`
- `install_reconciles_semver_style_package_without_allow_scripts`
- `install_reconciles_semver_style_package_with_allow_scripts_noop`

- [ ] **Step 4: Create `lifecycle.rs`**

Create `crates/kali_npm/src/install_tests/lifecycle.rs` starting with `use super::*;`, then move verbatim:
- `lifecycle_hooks_run_in_order_when_allowed`
- `lifecycle_hooks_skip_blank_entries`

- [ ] **Step 5: Create `traversal.rs`**

Create `crates/kali_npm/src/install_tests/traversal.rs` starting with `use super::*;`, then move verbatim:
- `collect_reachable_registry_packages_rejects_install_path_conflicts`
- `install_noops_without_manifest_or_dependencies`
- `install_stops_at_nested_child_project_roots`

- [ ] **Step 6: Reduce the facade**

Replace the entire contents of `crates/kali_npm/src/install_tests.rs` with exactly:

```rust
use crate::*;
use crate::test_support::*;
use crate::LOCK_VERSION;
use std::fs;
use std::sync::atomic::Ordering;

use serde_json::json;

#[path = "install_tests/rejections.rs"]
mod rejections;

#[path = "install_tests/reconciliation.rs"]
mod reconciliation;

#[path = "install_tests/lifecycle.rs"]
mod lifecycle;

#[path = "install_tests/traversal.rs"]
mod traversal;
```

- [ ] **Step 7: Verify count unchanged and tests pass**

Run: `cargo test -p kali_npm install_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 15 passed; 0 failed; ...`

- [ ] **Step 8: Verify whole-crate suite and build**

Run: `cargo test -p kali_npm --lib 2>&1 | grep 'test result'`
Expected: `test result: ok. 45 passed; ...`
Run: `cargo build -p kali_npm --tests 2>&1 | grep -c '^warning'`
Expected: `0`

- [ ] **Step 9: Byte-identity proof**

Run (from repo root, against a pre-move snapshot of `install_tests.rs` captured before Step 2):
`python3 .superpowers/sdd/verify.py <snapshot>/install_tests.rs "crates/kali_npm/src/install_tests/*.rs"`
Expected: exit 0 (15/15 `#[test]` bodies byte-identical).

- [ ] **Step 10: Commit**

```bash
git add crates/kali_npm/src/install_tests.rs crates/kali_npm/src/install_tests/
git commit -m "refactor(kali_npm): split install_tests.rs into per-concern test submodules [refactor]"
```

---

### Task 2: Split `resolve_tests.rs` (7 tests → 3 submodules)

**Files:**
- Create: `crates/kali_npm/src/resolve_tests/bare_import.rs`
- Create: `crates/kali_npm/src/resolve_tests/browser_replacement.rs`
- Create: `crates/kali_npm/src/resolve_tests/exports.rs`
- Modify: `crates/kali_npm/src/resolve_tests.rs` (reduce to facade)
- Unchanged: `crates/kali_npm/src/resolve.rs` (`#[cfg(test)] #[path = "resolve_tests.rs"] mod resolve_tests;` at line 393 stays)

**Interfaces:**
- Consumes: nothing. Produces: nothing other tasks depend on.

- [ ] **Step 1: Confirm baseline count**

Run: `cargo test -p kali_npm resolve_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 7 passed; ...`

- [ ] **Step 2: Create `bare_import.rs`**

Create `crates/kali_npm/src/resolve_tests/bare_import.rs` starting with `use super::*;`, then move verbatim:
- `bare_import_resolves_from_materialized_package`
- `bare_import_resolves_via_types_package_dependency`

- [ ] **Step 3: Create `browser_replacement.rs`**

Create `crates/kali_npm/src/resolve_tests/browser_replacement.rs` starting with `use super::*;`, then move verbatim:
- `browser_replacement_maps_rewrite_selected_root_entries`
- `browser_replacement_maps_rewrite_selected_root_entries_from_explicit_context`
- `browser_replacement_maps_can_block_selected_root_entries`
- `browser_replacement_maps_rewrite_selected_subpaths`

- [ ] **Step 4: Create `exports.rs`**

Create `crates/kali_npm/src/resolve_tests/exports.rs` starting with `use super::*;`, then move verbatim:
- `exports_take_precedence_over_legacy_entry_fields_and_respect_browser_conditions`

- [ ] **Step 5: Reduce the facade**

Replace the entire contents of `crates/kali_npm/src/resolve_tests.rs` with exactly:

```rust
use crate::*;
use std::fs;

#[path = "resolve_tests/bare_import.rs"]
mod bare_import;

#[path = "resolve_tests/browser_replacement.rs"]
mod browser_replacement;

#[path = "resolve_tests/exports.rs"]
mod exports;
```

- [ ] **Step 6: Verify count unchanged and tests pass**

Run: `cargo test -p kali_npm resolve_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 7 passed; 0 failed; ...`

- [ ] **Step 7: Verify whole-crate suite and build**

Run: `cargo test -p kali_npm --lib 2>&1 | grep 'test result'`
Expected: `test result: ok. 45 passed; ...`
Run: `cargo build -p kali_npm --tests 2>&1 | grep -c '^warning'`
Expected: `0`

- [ ] **Step 8: Byte-identity proof**

Run: `python3 .superpowers/sdd/verify.py <snapshot>/resolve_tests.rs "crates/kali_npm/src/resolve_tests/*.rs"`
Expected: exit 0 (7/7 `#[test]` bodies byte-identical).

- [ ] **Step 9: Commit**

```bash
git add crates/kali_npm/src/resolve_tests.rs crates/kali_npm/src/resolve_tests/
git commit -m "refactor(kali_npm): split resolve_tests.rs into per-concern test submodules [refactor]"
```

---

### Task 3: Split `validate_tests.rs` (14 tests → 2 submodules)

**Files:**
- Create: `crates/kali_npm/src/validate_tests/shape.rs`
- Create: `crates/kali_npm/src/validate_tests/host_fit.rs`
- Modify: `crates/kali_npm/src/validate_tests.rs` (reduce to facade)
- Unchanged: `crates/kali_npm/src/validate.rs` (`#[cfg(test)] #[path = "validate_tests.rs"] mod validate_tests;` at line 296 stays)

**Interfaces:**
- Consumes: nothing. Produces: nothing other tasks depend on.

- [ ] **Step 1: Confirm baseline count**

Run: `cargo test -p kali_npm validate_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 14 passed; ...`

- [ ] **Step 2: Create `shape.rs`**

Create `crates/kali_npm/src/validate_tests/shape.rs` starting with `use super::*;`, then move verbatim:
- `validate_package_shape_rejects_install_time_scripts_without_allow_scripts`
- `validate_package_shape_allows_non_install_scripts_without_allow_scripts`
- `validate_package_shape_allows_semver_style_metadata_without_allow_scripts`
- `validate_package_shape_rejects_node_gyp_install_time_scripts`
- `validate_package_shape_rejects_prebuild_install_time_scripts`
- `validate_package_shape_rejects_native_addon_entrypoints`
- `validate_package_shape_rejects_native_exports_entrypoints`
- `validate_package_shape_rejects_native_bin_entrypoints`
- `validate_package_shape_allows_harmless_scripts_when_allowed`

- [ ] **Step 3: Create `host_fit.rs`**

Create `crates/kali_npm/src/validate_tests/host_fit.rs` starting with `use super::*;`, then move verbatim:
- `validate_package_host_fit_rejects_node_builtin_imports`
- `validate_package_host_fit_rejects_node_timers_imports`
- `validate_package_host_fit_rejects_node_timers_promises_imports`
- `validate_package_host_fit_allows_node_builtin_imports_in_node_context`
- `validate_package_host_fit_rejects_node_builtin_requires`

- [ ] **Step 4: Reduce the facade**

Replace the entire contents of `crates/kali_npm/src/validate_tests.rs` with exactly:

```rust
use crate::*;

#[path = "validate_tests/shape.rs"]
mod shape;

#[path = "validate_tests/host_fit.rs"]
mod host_fit;
```

- [ ] **Step 5: Verify count unchanged and tests pass**

Run: `cargo test -p kali_npm validate_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 14 passed; 0 failed; ...`

- [ ] **Step 6: Verify whole-crate suite and build**

Run: `cargo test -p kali_npm --lib 2>&1 | grep 'test result'`
Expected: `test result: ok. 45 passed; ...`
Run: `cargo build -p kali_npm --tests 2>&1 | grep -c '^warning'`
Expected: `0`

- [ ] **Step 7: Byte-identity proof**

Run: `python3 .superpowers/sdd/verify.py <snapshot>/validate_tests.rs "crates/kali_npm/src/validate_tests/*.rs"`
Expected: exit 0 (14/14 `#[test]` bodies byte-identical).

- [ ] **Step 8: Commit**

```bash
git add crates/kali_npm/src/validate_tests.rs crates/kali_npm/src/validate_tests/
git commit -m "refactor(kali_npm): split validate_tests.rs into per-concern test submodules [refactor]"
```

---

## Final verification (after all 3 tasks)

- [ ] **Whole-crate lib suite:** `cargo test -p kali_npm --lib 2>&1 | grep 'test result'` → `45 passed; 0 failed`.
- [ ] **Build gate:** `cargo build -p kali_npm --tests 2>&1 | grep -c '^warning'` → `0`.
- [ ] **Byte-identity proof:** for each split file, `python3 .superpowers/sdd/verify.py <snapshot>/<F>_tests.rs "crates/kali_npm/src/<F>_tests/*.rs"` exits 0 (36/36 `#[test]` bodies byte-identical base→head).
- [ ] **Facade `#[test]` count == 0:** `grep -c '#\[test\]' crates/kali_npm/src/{install,resolve,validate}_tests.rs` → all `0`.
- [ ] **Dependent crate compiles unedited:** `cargo build -p kali_cli` (a kali_npm consumer) builds clean.
- [ ] **Diff is motion-only:** `git diff --stat <base>..HEAD -- crates/kali_npm/` shows only the three `*_tests.rs` facades shrinking + new submodule files; no product-source (`install.rs`, `resolve.rs`, `validate.rs`, `manifest_tests.rs`, `registry_tests.rs`) line changes.
- [ ] **Fmt:** `cargo fmt -p kali_npm --check` — accept known nits per series convention; do not reformat moved bodies.
- [ ] **Integrate:** ff-merge branch into local `main`; re-verify on merged main (`45 passed`, `0 warnings`); delete the branch. **No origin push.**
