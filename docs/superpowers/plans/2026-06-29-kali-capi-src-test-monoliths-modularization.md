# kali_capi src test-monolith modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split kali_capi's three multi-concern co-located `src/*_tests.rs` unit-test monoliths into thin facades + per-concern `#[path] mod` submodules, by pure verbatim code-motion.

**Architecture:** Each monolith `src/<name>_tests.rs` is declared from a product module (or `lib.rs`) via `#[cfg(test)] #[path = "<name>_tests.rs"] mod <name>_tests;`. We turn each into a facade that keeps its original `use` line(s) (plus, for `manifest_tests`, its one non-test helper fn) and one `#[path = "<name>_tests/<group>.rs"] mod <group>;` per group, and we move each `#[test]` fn **verbatim** into the matching `src/<name>_tests/<group>.rs` file, each of which opens with `use super::*;`. No product code changes; the compiled test set is byte-for-byte identical.

**Tech Stack:** Rust 2021, cargo, kali workspace.

**Reusable tooling (optional, recommended):** `.superpowers/sdd/move_fns.py` in exact-name-set mode automates each split (`python3 move_fns.py <stem>_tests.rs "g1=name1,name2;g2=..."` run from `crates/kali_capi`); it drains the `#[test]` fns into submodules, **auto-retains any non-`#[test]` module-level fn in the facade**, and appends the `#[path] mod` decls. `.superpowers/sdd/verify.py` proves `{name: body}` from the original == from the submodules (+ facade-retained items). Manual code-motion that produces the exact files below is equally acceptable.

## Global Constraints

- **Pure verbatim code-motion, zero behavior change.** No new product code, no new tests, no renamed tests, no reformatting of moved bodies. Move each `#[test]` fn exactly as written.
- **Facades drain to 0 module-level `#[test]` fns.** The only things retained on a facade are its original `use` line(s), the new `#[path] mod` declarations, and — **only for `manifest_tests`** — its one non-test helper fn `valid_binding_package_manifest`.
- **Submodule header:** every new `src/<name>_tests/<group>.rs` begins with exactly `use super::*;` and nothing else before the first moved fn. (The submodules reach `valid_binding_package_manifest` and all crate symbols through this glob.)
- **Test count is the invariant.** kali_capi's lib test suite is **37 tests** before and after; per-file filters must report: manifest_tests 19, binding_tests 7, metadata_tests 9. (The three name filters each uniquely select — `manifest_tests` / `binding_tests` / `metadata_tests` are not substrings of one another, and the bare `manifest`/`binding`/`metadata` tokens that appear inside other tests' names lack the contiguous `_tests` qualifier.)
- **No `pub`/`pub(crate)` widening, no `include_*!` pins** — verified 0 of each across all three files; no signature changes needed.
- **Product siblings unchanged:** `manifest.rs` (decl at lines 502-503), `metadata.rs` (decl at lines 427-428), and `lib.rs` (decl at lines 24-25) keep their existing `#[cfg(test)] #[path = "F_tests.rs"] mod F_tests;` decls.
- **Build gate:** `cargo build -p kali_capi --tests` stays at **0 warnings** (baseline = 0).
- **`cargo fmt --check`** — accept known fmt nits per series convention (do not reformat moved bodies to satisfy it).
- **Commits:** one `refactor(kali_capi): split <file>_tests.rs into per-concern test submodules [refactor]` per task. Local-main ff-merge only; no origin push.

---

### Task 1: Split `manifest_tests.rs` (19 tests → 4 submodules)

**Files:**
- Create: `crates/kali_capi/src/manifest_tests/parsing.rs`
- Create: `crates/kali_capi/src/manifest_tests/helpers.rs`
- Create: `crates/kali_capi/src/manifest_tests/summary.rs`
- Create: `crates/kali_capi/src/manifest_tests/construction.rs`
- Modify: `crates/kali_capi/src/manifest_tests.rs` (reduce to facade; **keep the `valid_binding_package_manifest` helper**)
- Unchanged: `crates/kali_capi/src/manifest.rs` (`#[cfg(test)] #[path = "manifest_tests.rs"] mod manifest_tests;` at lines 502-503 stays)

**Interfaces:**
- Consumes: nothing from other tasks.
- Produces: nothing other tasks depend on. (Each task is independent.)

- [ ] **Step 1: Confirm baseline count**

Run: `cargo test -p kali_capi manifest_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 19 passed; ...`

- [ ] **Step 2: Create `parsing.rs`**

Create `crates/kali_capi/src/manifest_tests/parsing.rs` starting with `use super::*;`, then move these fns **verbatim** from `manifest_tests.rs` (with their full bodies):
- `binding_package_manifest_parsing_normalizes_string_lists`
- `binding_package_manifest_parsing_rejects_whitespace_padded_string_lists`
- `binding_package_manifest_parsing_rejects_whitespace_padded_artifact_paths`
- `binding_package_manifest_parsing_rejects_non_integer_max_specializations`
- `binding_package_manifest_parsing_rejects_negative_max_specializations`
- `binding_package_manifest_parsing_rejects_unexpected_keys`
- `binding_package_manifest_parsing_rejects_invalid_required_field_types`
- `binding_package_manifest_parsing_rejects_non_string_provenance_fields`

- [ ] **Step 3: Create `helpers.rs`**

Create `crates/kali_capi/src/manifest_tests/helpers.rs` starting with `use super::*;`, then move verbatim (these consume the retained facade helper `valid_binding_package_manifest` via `use super::*;`):
- `binding_package_manifest_helpers_reject_whitespace_padded_module_name`
- `binding_package_manifest_helpers_reject_empty_provenance_fields`
- `binding_package_manifest_helpers_reject_empty_or_whitespace_artifact_paths`
- `binding_package_manifest_helpers_reject_ambiguous_auto_discovery`
- `binding_package_manifest_helpers_load_discover_and_summarize_manifests`

- [ ] **Step 4: Create `summary.rs`**

Create `crates/kali_capi/src/manifest_tests/summary.rs` starting with `use super::*;`, then move verbatim:
- `binding_package_manifest_summary_normalizes_string_lists`
- `binding_package_manifest_summary_rejects_invalid_required_field_types`
- `binding_package_manifest_summary_rejects_non_string_provenance_fields`

- [ ] **Step 5: Create `construction.rs`**

Create `crates/kali_capi/src/manifest_tests/construction.rs` starting with `use super::*;`, then move verbatim:
- `binding_package_manifest_orders_and_deduplicates_glue_deterministically`
- `binding_package_manifest_with_provenance_uses_explicit_contract_labels`
- `binding_package_manifest_rejects_incompatible_host_abi_version_window`

- [ ] **Step 6: Reduce the facade**

Edit `crates/kali_capi/src/manifest_tests.rs`: delete all 19 `#[test]` fns (now moved), **keep** the three `use` lines and the `valid_binding_package_manifest` helper fn in place, and append the four `#[path] mod` decls. The result must be exactly (helper body unchanged, shown abbreviated here — do NOT retype it, leave the existing fn untouched):

```rust
use crate::*;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn valid_binding_package_manifest() -> serde_json::Value {
    // ... existing body unchanged ...
}

#[path = "manifest_tests/parsing.rs"]
mod parsing;

#[path = "manifest_tests/helpers.rs"]
mod helpers;

#[path = "manifest_tests/summary.rs"]
mod summary;

#[path = "manifest_tests/construction.rs"]
mod construction;
```

- [ ] **Step 7: Verify count unchanged and tests pass**

Run: `cargo test -p kali_capi manifest_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 19 passed; 0 failed; ...`

- [ ] **Step 8: Verify whole-crate suite and build**

Run: `cargo test -p kali_capi --lib 2>&1 | grep 'test result'`
Expected: `test result: ok. 37 passed; ...`
Run: `cargo build -p kali_capi --tests 2>&1 | grep -c '^warning'`
Expected: `0`

- [ ] **Step 9: Commit**

```bash
git add crates/kali_capi/src/manifest_tests.rs crates/kali_capi/src/manifest_tests/
git commit -m "refactor(kali_capi): split manifest_tests.rs into per-concern test submodules [refactor]"
```

---

### Task 2: Split `binding_tests.rs` (7 tests → 2 submodules)

**Files:**
- Create: `crates/kali_capi/src/binding_tests/python.rs`
- Create: `crates/kali_capi/src/binding_tests/javascript.rs`
- Modify: `crates/kali_capi/src/binding_tests.rs` (reduce to facade)
- Unchanged: `crates/kali_capi/src/lib.rs` (`#[cfg(test)] #[path = "binding_tests.rs"] mod binding_tests;` at lines 24-25 stays)

**Interfaces:**
- Consumes: nothing. Produces: nothing other tasks depend on.

- [ ] **Step 1: Confirm baseline count**

Run: `cargo test -p kali_capi binding_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 7 passed; ...`

- [ ] **Step 2: Create `python.rs`**

Create `crates/kali_capi/src/binding_tests/python.rs` starting with `use super::*;`, then move verbatim:
- `python_binding_package_metadata_is_present`
- `python_binding_wraps_generated_header_exports`
- `python_binding_auto_discovers_stem_specific_binding_package_manifest`
- `python_binding_rejects_incompatible_host_abi_metadata`
- `python_unittest_smoke_covers_the_binding_helper_package`

- [ ] **Step 3: Create `javascript.rs`**

Create `crates/kali_capi/src/binding_tests/javascript.rs` starting with `use super::*;`, then move verbatim:
- `javascript_binding_package_metadata_is_present`
- `javascript_node_test_smoke_covers_the_binding_helper_package`

- [ ] **Step 4: Reduce the facade**

Replace the entire contents of `crates/kali_capi/src/binding_tests.rs` with exactly:

```rust
use crate::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "binding_tests/python.rs"]
mod python;

#[path = "binding_tests/javascript.rs"]
mod javascript;
```

- [ ] **Step 5: Verify count unchanged and tests pass**

Run: `cargo test -p kali_capi binding_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 7 passed; 0 failed; ...`

- [ ] **Step 6: Verify whole-crate suite and build**

Run: `cargo test -p kali_capi --lib 2>&1 | grep 'test result'`
Expected: `test result: ok. 37 passed; ...`
Run: `cargo build -p kali_capi --tests 2>&1 | grep -c '^warning'`
Expected: `0`

- [ ] **Step 7: Commit**

```bash
git add crates/kali_capi/src/binding_tests.rs crates/kali_capi/src/binding_tests/
git commit -m "refactor(kali_capi): split binding_tests.rs into per-concern test submodules [refactor]"
```

---

### Task 3: Split `metadata_tests.rs` (9 tests → 3 submodules)

**Files:**
- Create: `crates/kali_capi/src/metadata_tests/helpers.rs`
- Create: `crates/kali_capi/src/metadata_tests/generation.rs`
- Create: `crates/kali_capi/src/metadata_tests/parsing.rs`
- Modify: `crates/kali_capi/src/metadata_tests.rs` (reduce to facade)
- Unchanged: `crates/kali_capi/src/metadata.rs` (`#[cfg(test)] #[path = "metadata_tests.rs"] mod metadata_tests;` at lines 427-428 stays)

**Interfaces:**
- Consumes: nothing. Produces: nothing other tasks depend on.

- [ ] **Step 1: Confirm baseline count**

Run: `cargo test -p kali_capi metadata_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 9 passed; ...`

- [ ] **Step 2: Create `helpers.rs`**

Create `crates/kali_capi/src/metadata_tests/helpers.rs` starting with `use super::*;`, then move verbatim:
- `cabi_metadata_helpers_load_and_summarize_generated_payloads`
- `cabi_metadata_helpers_discover_load_and_summarize_root_sidecars`
- `cabi_metadata_helpers_reject_incompatible_host_abi_version_windows`
- `cabi_metadata_helpers_reject_ambiguous_auto_discovery`
- `cabi_metadata_helpers_reject_empty_provenance_fields`

- [ ] **Step 3: Create `generation.rs`**

Create `crates/kali_capi/src/metadata_tests/generation.rs` starting with `use super::*;`, then move verbatim:
- `metadata_generation_includes_expected_artifacts`
- `metadata_generation_with_provenance_keeps_optional_fields_deterministic`

- [ ] **Step 4: Create `parsing.rs`**

Create `crates/kali_capi/src/metadata_tests/parsing.rs` starting with `use super::*;`, then move verbatim:
- `cabi_metadata_parsing_rejects_unexpected_keys`
- `cabi_metadata_parsing_rejects_negative_max_specializations`

- [ ] **Step 5: Reduce the facade**

Replace the entire contents of `crates/kali_capi/src/metadata_tests.rs` with exactly:

```rust
use crate::*;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[path = "metadata_tests/helpers.rs"]
mod helpers;

#[path = "metadata_tests/generation.rs"]
mod generation;

#[path = "metadata_tests/parsing.rs"]
mod parsing;
```

- [ ] **Step 6: Verify count unchanged and tests pass**

Run: `cargo test -p kali_capi metadata_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 9 passed; 0 failed; ...`

- [ ] **Step 7: Verify whole-crate suite and build**

Run: `cargo test -p kali_capi --lib 2>&1 | grep 'test result'`
Expected: `test result: ok. 37 passed; ...`
Run: `cargo build -p kali_capi --tests 2>&1 | grep -c '^warning'`
Expected: `0`

- [ ] **Step 8: Commit**

```bash
git add crates/kali_capi/src/metadata_tests.rs crates/kali_capi/src/metadata_tests/
git commit -m "refactor(kali_capi): split metadata_tests.rs into per-concern test submodules [refactor]"
```

---

## Final verification (after all 3 tasks)

- [ ] **Whole-crate lib suite:** `cargo test -p kali_capi --lib 2>&1 | grep 'test result'` → `37 passed; 0 failed`.
- [ ] **Build gate:** `cargo build -p kali_capi --tests 2>&1 | grep -c '^warning'` → `0`.
- [ ] **Byte-identity proof:** for each split file, `python3 .superpowers/sdd/verify.py <orig_rs> "<submodule_glob>" [facade_glob_for_pins]` exits 0 (35/35 `#[test]` bodies byte-identical base→head). For `manifest_tests`, pass the facade glob so the retained helper is accounted for.
- [ ] **Dependent crate compiles unedited:** `cargo build -p kali_cli` (a kali_capi consumer) builds clean.
- [ ] **Diff is motion-only:** `git diff --stat <base>..HEAD -- crates/kali_capi/` shows only the three `*_tests.rs` facades shrinking + new submodule files; no product-source (`manifest.rs`, `metadata.rs`, `lib.rs`, `binding.rs`, `header.rs`) line changes.
- [ ] **Fmt:** `cargo fmt -p kali_capi --check` — accept known nits per series convention; do not reformat moved bodies.
