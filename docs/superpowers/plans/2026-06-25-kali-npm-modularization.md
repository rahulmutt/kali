# kali_npm Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the monolithic `crates/kali_npm/src/lib.rs` (2,892 lines) and `tests.rs` (2,306 lines) into 7 thematic modules behind a thin facade, with zero behavior change and an identical public API and test set.

**Architecture:** Pure verbatim text-movement of free functions + types into 7 sibling modules (`manifest`, `target`, `tarball`, `validate`, `registry`, `resolve`, `install`), each re-exported flat from a ~40-line `lib.rs` facade via `pub use <mod>::*;`. Tests are co-located into sibling `<mod>_tests.rs` files; shared test infrastructure (mock servers, tarball builder, registry lock) moves into a `#[cfg(test)] test_support` module. Cross-module references are enabled by blanket-widening private free functions/types to `pub(crate)` up front.

**Tech Stack:** Rust (workspace crate), `cargo test`, `cargo clippy`, `kali_test_support` dev-crate fixtures.

## Global Constraints

Copied verbatim from `docs/superpowers/specs/2026-06-25-kali-npm-modularization-design.md`. Every task's requirements implicitly include these:

- **Zero behavior change.** Movement is byte-for-byte verbatim, **including whitespace** (blank-line separators between items count — do not drop or add any).
- **Public API = exactly 20 items**, flat `kali_npm::<name>` paths preserved: 8 pub types (`ProjectManifest`, `LockFile`, `LockedPackage`, `RawUrlEntry`, `PackageTarget`, `InstallOptions`, `InstallSummary`, `RegistryPackageAudit`) + 12 pub fns (`discover_project_root`, `load_manifest`, `save_manifest`, `load_lock`, `save_lock`, `ensure_project_ready`, `install_project`, `audit_registry_package`, `source_mentions_node_only_host_api`, `resolve_materialized_import`, `resolve_materialized_import_with_browser_context`, `project_requires_install`).
- **Test set = exactly 45 tests**, name set identical to baseline. Prove by comparing **basenames** (strip module-path prefixes from `cargo test -- --list`), never a raw diff.
- **Green + clippy-clean at every commit:** `cargo test -p kali_npm` passes (45) AND `cargo clippy -p kali_npm --all-targets -- -D warnings` is clean. Run **both** after every task — `cargo test` does not gate warnings.
- **Consts/static stay in `lib.rs`:** `MANIFEST_SCHEMA`, `LOCK_VERSION`, `DEFAULT_NPM_REGISTRY`, `NODE_ONLY_HOST_APIS`, and the `REGISTRY_METADATA_CACHE` static remain crate-root privates; modules reach them via `use crate::*;`, falling back to `crate::<NAME>` qualification at the use site if the glob doesn't surface a bare reference.
- **GLOB / no-glob rule:** each module gets `mod <m>;` + `pub use <m>::*;`. If clippy `-D warnings` flags a `pub use` glob as unused, **delete** that glob (do not `#[allow]` it). Inside every module use `use crate::*;`; drop that header from any module clippy proves references no crate items.
- **Commit message convention:** `refactor(kali_npm): <summary> [refactor]` (or `test(...)`/`docs(...)` as appropriate), matching prior crates.
- **Branch:** `refactor/kali-npm-modularization` (already created; design doc committed at `32ea9f883`).

---

## File Structure

**Source modules** (each `crates/kali_npm/src/<name>.rs`):

| Module | Types (def'd here) | Functions | Notes |
|--------|--------------------|-----------|-------|
| `manifest` | `ProjectManifest`(+impl), `LockFile`(+impl), `LockedPackage`, `RawUrlEntry` | `discover_project_root`, `load_manifest`, `save_manifest`, `load_lock`, `save_lock`, `ensure_project_ready`, `project_requires_install`, `manifest_registry_package_keys`, `split_package_key`, `validate_manifest_registry_collisions` (10) | uses `MANIFEST_SCHEMA`, `LOCK_VERSION` |
| `target` | `PackageTarget` | `parse_package_target`, `split_package_name_and_version`, `encode_package_name`, `package_key`, `install_name_from_package`, `jsr_compat_name`, `types_package_name`, `split_bare_package_source` (8) | leaf, no co-located tests |
| `tarball` | — | `download_bytes`, `verify_tarball_integrity`, `integrity_matches`, `format_sha512`, `sha256_hex`, `extract_tarball`, `copy_tree`, `recursive_copy`, `raw_url_file_name` (9) | leaf, no co-located tests |
| `validate` | `PackageHostFitContext`(+impl), `PackageJson` | `source_mentions_node_only_host_api`, `package_host_fit_context_for_manifest`, `read_package_json`, `value_contains_native_addon_path`, `validate_package_shape`, `script_uses_native_bootstrap_tool`, `validate_package_host_fit`, `scan_for_node_only_host_api`, `is_scannable_package_source`, `should_skip_package_scan_dir` (10) | uses `NODE_ONLY_HOST_APIS`; `PackageJson` is `pub(crate)` (also used by `resolve`) |
| `registry` | `ResolvedRegistryPackage`, `RegistryPackageAudit` | `audit_registry_package`, `resolve_registry_package`, `resolve_npm_package`, `npm_registry_base_url`, `npm_registry_metadata_url`, `jsr_registry_metadata_url`, `fetch_registry_metadata`, `resolve_jsr_package`, `resolve_npm_like_package`, `audit_package_version_metadata`, `select_registry_version` (11) | uses `DEFAULT_NPM_REGISTRY`, `REGISTRY_METADATA_CACHE` |
| `resolve` | `PackageResolutionOutcome` | `resolve_materialized_import`, `resolve_materialized_import_with_browser_context`, `resolve_types_package_import`, `resolve_package_types_entry`, `resolve_package_entry`, `resolve_package_subpath`, `apply_browser_rewrite`, `resolve_package_exports`, `resolve_package_exports_target`, `substitute_export_pattern`, `match_export_pattern`, `resolve_package_file` (12) | uses `crate::PackageJson` |
| `install` | `InstallOptions`, `InstallSummary` | `install_project`, `ensure_lock_install_name_unique`, `collect_reachable_registry_packages`, `prune_unreachable_registry_packages`, `discover_install_source_files`, `collect_install_source_files`, `is_install_source_file`, `collect_source_module_specifiers`, `resolve_import_map_specifier`, `is_raw_url`, `discover_install_time_raw_urls`, `prune_unreachable_raw_urls`, `remove_cached_raw_url_entry`, `reconcile_raw_urls`, `has_effective_npm_scriptable_install_work`, `record_install_path`, `install_registry_package`, `install_raw_url`, `run_package_lifecycle_hooks`, `run_package_lifecycle_hook` (20) | largest; `install_project` (236 lines) + `install_registry_package` (265 lines) move INTACT |

Total: 80 functions (12 pub + 68 private) + 12 types (8 pub + 4 private) + 3 impl blocks. **Sum check:** 10+8+9+10+11+12+20 = 80 fns; 4+1+0+2+2+1+2 = 12 types. ✓

**Test files** (each `crates/kali_npm/src/<name>_tests.rs`), 45 tests total:

| File | Count | Tests |
|------|------:|-------|
| `manifest_tests.rs` | 6 | `manifest_round_trip_is_deterministic`, `lock_round_trip_is_deterministic`, `manifest_registry_collisions_are_rejected_before_install`, `manifest_registry_collisions_allow_identical_identity_spelling`, `ensure_project_ready_rejects_stale_lock_entries`, `ensure_project_ready_rejects_missing_raw_url_cache` |
| `validate_tests.rs` | 14 | the 9 `validate_package_shape_*` + the 5 `validate_package_host_fit_*` |
| `registry_tests.rs` | 3 | `requested_version_ranges_select_highest_matching_release`, `registry_metadata_is_cached_within_a_process`, `audit_package_version_metadata_rejects_native_exports_entrypoints` |
| `resolve_tests.rs` | 7 | `bare_import_resolves_from_materialized_package`, `bare_import_resolves_via_types_package_dependency`, the 4 `browser_replacement_maps_*`, `exports_take_precedence_over_legacy_entry_fields_and_respect_browser_conditions` |
| `install_tests.rs` | 15 | `lifecycle_hooks_run_in_order_when_allowed`, `lifecycle_hooks_skip_blank_entries`, `collect_reachable_registry_packages_rejects_install_path_conflicts`, `install_reconciles_raw_urls_from_source_import_map_rewrites`, `install_is_idempotent_for_unchanged_raw_url_graph`, `install_noops_without_manifest_or_dependencies`, `install_stops_at_nested_child_project_roots`, the 6 `install_rejects_*`, `install_reconciles_semver_style_package_without_allow_scripts`, `install_reconciles_semver_style_package_with_allow_scripts_noop` |

`target_tests.rs` and `tarball_tests.rs` are **not** created (no tests target those modules directly; they are exercised indirectly through `install`/`registry` integration tests).

**Shared test infrastructure** → `crates/kali_npm/src/test_support.rs` (`#[cfg(test)] mod test_support`): `append_marker_command`, `start_raw_url_server`, `start_response_server`, `build_package_tarball`, `kali_registry_lock` (+ its inner `static LOCK`), `start_metadata_server`. Consumed by `install_tests.rs` and `registry_tests.rs` via `use crate::test_support::*;`.

**Facade** (`crates/kali_npm/src/lib.rs`, ~40 lines): crate doc comment + `use` imports + 4 consts + 1 static + 7 `mod`/`pub use` pairs + `#[cfg(test)] mod test_support;`.

---

## Task 1: Capture baseline

**Files:**
- Create: `docs/superpowers/baselines/2026-06-25-kali-npm-tests.txt`
- Create: `docs/superpowers/baselines/2026-06-25-kali-npm-api.txt`

- [ ] **Step 1: Capture the sorted test basename set**

Run:
```bash
cargo test -p kali_npm -- --list 2>/dev/null \
  | grep -E ': test$' \
  | sed -E 's/.*:://; s/: test$//' \
  | sort -u > docs/superpowers/baselines/2026-06-25-kali-npm-tests.txt
wc -l docs/superpowers/baselines/2026-06-25-kali-npm-tests.txt
```
Expected: `45 docs/superpowers/baselines/2026-06-25-kali-npm-tests.txt`

- [ ] **Step 2: Record the public API surface**

Write the 20 public paths (from Global Constraints) into the api baseline file, one per line, sorted:
```bash
cat > docs/superpowers/baselines/2026-06-25-kali-npm-api.txt <<'EOF'
kali_npm::InstallOptions
kali_npm::InstallSummary
kali_npm::LockFile
kali_npm::LockedPackage
kali_npm::PackageTarget
kali_npm::ProjectManifest
kali_npm::RawUrlEntry
kali_npm::RegistryPackageAudit
kali_npm::audit_registry_package
kali_npm::discover_project_root
kali_npm::ensure_project_ready
kali_npm::install_project
kali_npm::load_lock
kali_npm::load_manifest
kali_npm::project_requires_install
kali_npm::resolve_materialized_import
kali_npm::resolve_materialized_import_with_browser_context
kali_npm::save_lock
kali_npm::save_manifest
kali_npm::source_mentions_node_only_host_api
EOF
wc -l docs/superpowers/baselines/2026-06-25-kali-npm-api.txt
```
Expected: `20 docs/superpowers/baselines/2026-06-25-kali-npm-api.txt`

- [ ] **Step 3: Confirm green + clippy-clean starting point**

Run:
```bash
cargo test -p kali_npm 2>&1 | tail -3
cargo clippy -p kali_npm --all-targets -- -D warnings 2>&1 | tail -3
```
Expected: test run reports `test result: ok. 45 passed`; clippy finishes with no warnings/errors.

- [ ] **Step 4: Commit**

```bash
git add docs/superpowers/baselines/2026-06-25-kali-npm-tests.txt docs/superpowers/baselines/2026-06-25-kali-npm-api.txt
git commit -m "test(kali_npm): capture pre-refactor baseline [refactor]"
```

---

## Task 2: Widen private items to `pub(crate)` (extraction prep)

This makes every later extraction a pure cut-paste with no per-move visibility edits. `pub(crate)` keeps items crate-private (no public-API change) and is clippy-clean for items used somewhere in-crate (which all of these are). `unreachable_pub` is allow-by-default and not part of `-D warnings`, so blanket widening does not trip clippy.

**Files:**
- Modify: `crates/kali_npm/src/lib.rs`

**Interfaces:**
- Produces: every currently-private free `fn` and the 4 private types (`PackageHostFitContext`, `ResolvedRegistryPackage`, `PackageJson`, `PackageResolutionOutcome`) become `pub(crate)`. Consts/static stay private.

- [ ] **Step 1: Widen all private free functions**

In `lib.rs`, prefix every top-level `fn ` (the 68 private free functions — NOT the `pub fn` ones, NOT methods inside `impl` blocks, NOT test fns) with `pub(crate) `. Mechanically: a line beginning at column 0 with `fn ` becomes `pub(crate) fn `.

```bash
# in crates/kali_npm/src
perl -i -pe 's/^fn /pub(crate) fn /' lib.rs
grep -cE '^pub\(crate\) fn ' lib.rs   # expect 68
grep -cE '^fn ' lib.rs                # expect 0
```
Expected: `68` then `0`.

- [ ] **Step 2: Widen the 4 private types**

Edit each of these four declarations in `lib.rs` to add `pub(crate) ` (keep `enum`/`struct` keyword):
- `enum PackageHostFitContext {` → `pub(crate) enum PackageHostFitContext {`
- `struct ResolvedRegistryPackage {` → `pub(crate) struct ResolvedRegistryPackage {`
- `struct PackageJson {` → `pub(crate) struct PackageJson {`
- `enum PackageResolutionOutcome {` → `pub(crate) enum PackageResolutionOutcome {`

Leave their fields as-is unless a later compile error requires field widening (these types stay within their owning module except `PackageJson`, whose fields are already accessed by `resolve`; if a field access fails to compile in Task 9, widen that specific field to `pub(crate)` then — note it in the commit).

- [ ] **Step 3: Verify green + clippy-clean**

Run:
```bash
cargo test -p kali_npm 2>&1 | tail -2
cargo clippy -p kali_npm --all-targets -- -D warnings 2>&1 | tail -2
```
Expected: `test result: ok. 45 passed`; clippy clean.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_npm/src/lib.rs
git commit -m "refactor(kali_npm): widen private items to pub(crate) for extraction [refactor]"
```

---

## Task 3: Extract shared test infrastructure to `test_support`

Move the shared test helpers out of `tests.rs` (still the monolith) into a dedicated module, so later per-module test splits can reference them uniformly. `tests.rs` keeps working via `use crate::test_support::*;`.

**Files:**
- Create: `crates/kali_npm/src/test_support.rs`
- Modify: `crates/kali_npm/src/tests.rs` (remove the moved helpers, add an import)
- Modify: `crates/kali_npm/src/lib.rs` (declare the module)

**Interfaces:**
- Produces: `pub(crate) fn append_marker_command`, `pub(crate) fn start_raw_url_server`, `pub(crate) fn start_response_server`, `pub(crate) fn build_package_tarball`, `pub(crate) fn kali_registry_lock`, `pub(crate) fn start_metadata_server` — all reachable as `crate::test_support::<name>`.

- [ ] **Step 1: Create `test_support.rs` with the moved helpers**

Cut these items **verbatim** from `tests.rs` and paste them into a new `crates/kali_npm/src/test_support.rs`, adding `pub(crate)` to each helper `fn` (the inner `static LOCK` inside `kali_registry_lock` stays as-is). At the top of the file add the imports those helpers need (move the relevant `use` lines, e.g. `use std::sync::{Mutex, OnceLock};`, plus any `std::net`/`std::thread`/`std::io` used by the servers). The moved items:
- `append_marker_command`
- `start_raw_url_server`
- `start_response_server`
- `build_package_tarball`
- `kali_registry_lock` (with its inner `static LOCK: OnceLock<Mutex<()>>`)
- `start_metadata_server`

File header:
```rust
//! Shared test infrastructure for kali_npm integration tests.

use crate::*;
```
(If clippy later flags `use crate::*;` here as unused, drop it per the GLOB rule.)

- [ ] **Step 2: Wire the module in `lib.rs`**

At the bottom of `lib.rs`, just above the existing `#[cfg(test)] #[path = "tests.rs"] mod tests;`, add:
```rust
#[cfg(test)]
mod test_support;
```

- [ ] **Step 3: Point `tests.rs` at the moved helpers**

At the top of `tests.rs` (after its existing `use` lines), add:
```rust
use crate::test_support::*;
```
Remove the now-moved helper definitions from `tests.rs` (they live in `test_support.rs` now).

- [ ] **Step 4: Verify green + clippy-clean**

Run:
```bash
cargo test -p kali_npm 2>&1 | tail -2
cargo clippy -p kali_npm --all-targets -- -D warnings 2>&1 | tail -2
```
Expected: `test result: ok. 45 passed`; clippy clean.

- [ ] **Step 5: Verify the basename set is unchanged**

```bash
cargo test -p kali_npm -- --list 2>/dev/null | grep -E ': test$' \
  | sed -E 's/.*:://; s/: test$//' | sort -u \
  | diff - docs/superpowers/baselines/2026-06-25-kali-npm-tests.txt && echo "BASENAMES MATCH"
```
Expected: `BASENAMES MATCH`.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_npm/src/test_support.rs crates/kali_npm/src/tests.rs crates/kali_npm/src/lib.rs
git commit -m "test(kali_npm): extract shared test_support module [refactor]"
```

---

## Tasks 4–10: Module extractions (shared recipe)

Each of the next seven tasks extracts ONE module. They share an identical recipe; only the item list, module name, and test split differ. The recipe is given once here; per-task sections below supply the specifics.

**Per-extraction recipe:**

1. **Create `src/<m>.rs`.** Header:
   ```rust
   use crate::*;
   ```
   Cut the module's functions, types, and impl blocks **verbatim** (whitespace included) from `lib.rs` and paste them into `<m>.rs`, preserving their relative order. They are already `pub`/`pub(crate)` from Task 2 — do not change visibility.
2. **Wire in `lib.rs`.** Add, grouped with the other module decls:
   ```rust
   mod <m>;
   pub use <m>::*;
   ```
3. **Split tests (if the module has a test file).** Create `src/<m>_tests.rs` with header `use crate::*;` (and `use crate::test_support::*;` if it uses shared infra). Cut the listed test fns **verbatim** from `tests.rs` into it. At the bottom of `src/<m>.rs` add:
   ```rust
   #[cfg(test)]
   #[path = "<m>_tests.rs"]
   mod <m>_tests;
   ```
4. **Const/import fallback.** If compilation fails because a moved item references a crate-root const/static not surfaced by `use crate::*;`, qualify it at the use site as `crate::<NAME>` (mechanical, no logic change). If a moved test reads a module-private item, add `use super::<ITEM>;` after `use crate::*;` in the test file.
5. **Verify** (all four must pass):
   ```bash
   cargo test -p kali_npm 2>&1 | tail -2          # test result: ok. 45 passed
   cargo clippy -p kali_npm --all-targets -- -D warnings 2>&1 | tail -2   # clean
   cargo test -p kali_npm -- --list 2>/dev/null | grep -E ': test$' \
     | sed -E 's/.*:://; s/: test$//' | sort -u \
     | diff - docs/superpowers/baselines/2026-06-25-kali-npm-tests.txt && echo "BASENAMES MATCH"
   ```
6. **GLOB check.** If clippy flags the new `pub use <m>::*;` as unused, delete that line (the module's public items are re-exported elsewhere or the module has none beyond `pub(crate)` used via `use crate::*;`). If clippy flags the module's own `use crate::*;` as unused, delete it.
7. **Commit:** `git add crates/kali_npm/src/ && git commit -m "refactor(kali_npm): extract <m> module [refactor]"`.

Extraction order (leaf/low-dependency first, `install` last):

---

### Task 4: Extract `target` module

**Files:**
- Create: `crates/kali_npm/src/target.rs`
- Modify: `crates/kali_npm/src/lib.rs`

**Interfaces:**
- Produces: `pub enum PackageTarget`; `pub fn` n/a; `pub(crate) fn parse_package_target, split_package_name_and_version, encode_package_name, package_key, install_name_from_package, jsr_compat_name, types_package_name, split_bare_package_source`.

- [ ] **Step 1:** Apply the recipe. Move `PackageTarget` (enum) + the 8 functions listed for `target` in the File Structure table. No test file (skip recipe step 3).
- [ ] **Step 2:** Run recipe step 5 verification. Expected: 45 passed, clippy clean, `BASENAMES MATCH`.
- [ ] **Step 3:** Run recipe step 6 GLOB check.
- [ ] **Step 4:** Commit: `refactor(kali_npm): extract target module [refactor]`.

---

### Task 5: Extract `tarball` module

**Files:**
- Create: `crates/kali_npm/src/tarball.rs`
- Modify: `crates/kali_npm/src/lib.rs`

**Interfaces:**
- Produces: `pub(crate) fn download_bytes, verify_tarball_integrity, integrity_matches, format_sha512, sha256_hex, extract_tarball, copy_tree, recursive_copy, raw_url_file_name`. No types.

- [ ] **Step 1:** Apply the recipe. Move the 9 `tarball` functions. No test file.
- [ ] **Step 2:** Run recipe step 5 verification. Expected: 45 passed, clippy clean, `BASENAMES MATCH`.
- [ ] **Step 3:** Run recipe step 6 GLOB check. (Note: this module has only `pub(crate)` items — its `pub use tarball::*;` will likely be clippy-flagged as unused; if so, replace `mod tarball; pub use tarball::*;` with just `mod tarball;`. Callers reach these via `use crate::*;` only if re-exported, so KEEP the glob if removing it breaks the build; trust clippy + the compiler.)
- [ ] **Step 4:** Commit: `refactor(kali_npm): extract tarball module [refactor]`.

---

### Task 6: Extract `validate` module

**Files:**
- Create: `crates/kali_npm/src/validate.rs`, `crates/kali_npm/src/validate_tests.rs`
- Modify: `crates/kali_npm/src/lib.rs`, `crates/kali_npm/src/tests.rs`

**Interfaces:**
- Produces: `pub fn source_mentions_node_only_host_api`; `pub(crate) enum PackageHostFitContext` (+impl); `pub(crate) struct PackageJson`; `pub(crate) fn package_host_fit_context_for_manifest, read_package_json, value_contains_native_addon_path, validate_package_shape, script_uses_native_bootstrap_tool, validate_package_host_fit, scan_for_node_only_host_api, is_scannable_package_source, should_skip_package_scan_dir`.

- [ ] **Step 1:** Apply the recipe. Move the 10 `validate` functions + `PackageHostFitContext` (enum + its impl) + `PackageJson` (struct). `validate.rs` references `NODE_ONLY_HOST_APIS` (crate-root const) — rely on `use crate::*;`; if unresolved, qualify as `crate::NODE_ONLY_HOST_APIS`.
- [ ] **Step 2:** Create `validate_tests.rs` (header `use crate::*;`) and move the **14** tests: the 9 `validate_package_shape_*` and the 5 `validate_package_host_fit_*` (names in the File Structure test table). Wire `#[cfg(test)] #[path = "validate_tests.rs"] mod validate_tests;` at the bottom of `validate.rs`.
- [ ] **Step 3:** Run recipe step 5 verification. Expected: 45 passed, clippy clean, `BASENAMES MATCH`.
- [ ] **Step 4:** Run recipe step 6 GLOB check.
- [ ] **Step 5:** Commit: `refactor(kali_npm): extract validate module [refactor]`.

---

### Task 7: Extract `registry` module

**Files:**
- Create: `crates/kali_npm/src/registry.rs`, `crates/kali_npm/src/registry_tests.rs`
- Modify: `crates/kali_npm/src/lib.rs`, `crates/kali_npm/src/tests.rs`

**Interfaces:**
- Produces: `pub fn audit_registry_package`; `pub struct RegistryPackageAudit`; `pub(crate) struct ResolvedRegistryPackage`; `pub(crate) fn resolve_registry_package, resolve_npm_package, npm_registry_base_url, npm_registry_metadata_url, jsr_registry_metadata_url, fetch_registry_metadata, resolve_jsr_package, resolve_npm_like_package, audit_package_version_metadata, select_registry_version`.

- [ ] **Step 1:** Apply the recipe. Move the 11 `registry` functions + `RegistryPackageAudit` (pub struct) + `ResolvedRegistryPackage` (pub(crate) struct). References `DEFAULT_NPM_REGISTRY` and `REGISTRY_METADATA_CACHE` (crate-root) — rely on `use crate::*;`, falling back to `crate::DEFAULT_NPM_REGISTRY` / `crate::REGISTRY_METADATA_CACHE` if needed.
- [ ] **Step 2:** Create `registry_tests.rs` (header `use crate::*;` plus `use crate::test_support::*;` — these tests use `start_metadata_server`/`kali_registry_lock`). Move the **3** tests: `requested_version_ranges_select_highest_matching_release`, `registry_metadata_is_cached_within_a_process`, `audit_package_version_metadata_rejects_native_exports_entrypoints`. Wire `mod registry_tests;` at the bottom of `registry.rs`.
- [ ] **Step 3:** Run recipe step 5 verification. Expected: 45 passed, clippy clean, `BASENAMES MATCH`.
- [ ] **Step 4:** Run recipe step 6 GLOB check.
- [ ] **Step 5:** Commit: `refactor(kali_npm): extract registry module [refactor]`.

---

### Task 8: Extract `resolve` module

**Files:**
- Create: `crates/kali_npm/src/resolve.rs`, `crates/kali_npm/src/resolve_tests.rs`
- Modify: `crates/kali_npm/src/lib.rs`, `crates/kali_npm/src/tests.rs`

**Interfaces:**
- Produces: `pub fn resolve_materialized_import, resolve_materialized_import_with_browser_context`; `pub(crate) enum PackageResolutionOutcome`; `pub(crate) fn resolve_types_package_import, resolve_package_types_entry, resolve_package_entry, resolve_package_subpath, apply_browser_rewrite, resolve_package_exports, resolve_package_exports_target, substitute_export_pattern, match_export_pattern, resolve_package_file`. Consumes `crate::PackageJson` (from `validate`).

- [ ] **Step 1:** Apply the recipe. Move the 12 `resolve` functions + `PackageResolutionOutcome` (enum). `resolve_package_types_entry` takes `&PackageJson` — resolved via `use crate::*;` (re-exported from `validate`). If a `PackageJson` field access fails to compile, widen that field to `pub(crate)` in `validate.rs` (record in commit body).
- [ ] **Step 2:** Create `resolve_tests.rs` (header `use crate::*;`). Move the **7** tests: `bare_import_resolves_from_materialized_package`, `bare_import_resolves_via_types_package_dependency`, the 4 `browser_replacement_maps_*`, `exports_take_precedence_over_legacy_entry_fields_and_respect_browser_conditions`. Wire `mod resolve_tests;` at the bottom of `resolve.rs`.
- [ ] **Step 3:** Run recipe step 5 verification. Expected: 45 passed, clippy clean, `BASENAMES MATCH`.
- [ ] **Step 4:** Run recipe step 6 GLOB check.
- [ ] **Step 5:** Commit: `refactor(kali_npm): extract resolve module [refactor]`.

---

### Task 9: Extract `manifest` module

**Files:**
- Create: `crates/kali_npm/src/manifest.rs`, `crates/kali_npm/src/manifest_tests.rs`
- Modify: `crates/kali_npm/src/lib.rs`, `crates/kali_npm/src/tests.rs`

**Interfaces:**
- Produces: `pub struct ProjectManifest`(+impl), `pub struct LockFile`(+impl), `pub struct LockedPackage`, `pub struct RawUrlEntry`; `pub fn discover_project_root, load_manifest, save_manifest, load_lock, save_lock, ensure_project_ready, project_requires_install`; `pub(crate) fn manifest_registry_package_keys, split_package_key, validate_manifest_registry_collisions`.

- [ ] **Step 1:** Apply the recipe. Move the 10 `manifest` functions + the 4 types (`ProjectManifest`, `LockFile`, `LockedPackage`, `RawUrlEntry`) + the 2 impl blocks (`impl ProjectManifest`, `impl LockFile`). References `MANIFEST_SCHEMA`, `LOCK_VERSION` — rely on `use crate::*;`, fallback `crate::MANIFEST_SCHEMA` / `crate::LOCK_VERSION`.
- [ ] **Step 2:** Create `manifest_tests.rs` (header `use crate::*;`). Move the **6** tests: `manifest_round_trip_is_deterministic`, `lock_round_trip_is_deterministic`, `manifest_registry_collisions_are_rejected_before_install`, `manifest_registry_collisions_allow_identical_identity_spelling`, `ensure_project_ready_rejects_stale_lock_entries`, `ensure_project_ready_rejects_missing_raw_url_cache`. Wire `mod manifest_tests;` at the bottom of `manifest.rs`.
- [ ] **Step 3:** Run recipe step 5 verification. Expected: 45 passed, clippy clean, `BASENAMES MATCH`.
- [ ] **Step 4:** Run recipe step 6 GLOB check.
- [ ] **Step 5:** Commit: `refactor(kali_npm): extract manifest module [refactor]`.

---

### Task 10: Extract `install` module (and remove the monolithic `tests.rs`)

This is the largest module and runs last; after it, `tests.rs` holds no `#[test]` fns and is deleted.

**Files:**
- Create: `crates/kali_npm/src/install.rs`, `crates/kali_npm/src/install_tests.rs`
- Delete: `crates/kali_npm/src/tests.rs`
- Modify: `crates/kali_npm/src/lib.rs`

**Interfaces:**
- Produces: `pub fn install_project`; `pub struct InstallOptions`, `pub struct InstallSummary`; `pub(crate) fn ensure_lock_install_name_unique, collect_reachable_registry_packages, prune_unreachable_registry_packages, discover_install_source_files, collect_install_source_files, is_install_source_file, collect_source_module_specifiers, resolve_import_map_specifier, is_raw_url, discover_install_time_raw_urls, prune_unreachable_raw_urls, remove_cached_raw_url_entry, reconcile_raw_urls, has_effective_npm_scriptable_install_work, record_install_path, install_registry_package, install_raw_url, run_package_lifecycle_hooks, run_package_lifecycle_hook`.

- [ ] **Step 1:** Apply the recipe. Move the 20 `install` functions + `InstallOptions` + `InstallSummary`. `install_project` (236 lines) and `install_registry_package` (265 lines) move **byte-for-byte intact** — do NOT crack them into helpers (separate deferred refactor, out of scope).
- [ ] **Step 2:** Create `install_tests.rs` (header `use crate::*;` plus `use crate::test_support::*;`). Move the **15** `install`/`lifecycle`/`collect_reachable` tests (names in the File Structure test table). Wire `mod install_tests;` at the bottom of `install.rs`.
- [ ] **Step 3: Remove the empty monolithic `tests.rs`.** It now contains no `#[test]` fns (all moved). Delete the file and remove its wiring from `lib.rs`:
  ```bash
  # confirm no tests remain in it:
  grep -c '#\[test\]' crates/kali_npm/src/tests.rs   # expect 0
  git rm crates/kali_npm/src/tests.rs
  ```
  In `lib.rs`, delete the `#[cfg(test)] #[path = "tests.rs"] mod tests;` line. (If any non-test residue remains in `tests.rs` — e.g. a stray helper — move it to `test_support.rs` first.)
- [ ] **Step 4:** Run recipe step 5 verification. Expected: 45 passed, clippy clean, `BASENAMES MATCH`.
- [ ] **Step 5:** Run recipe step 6 GLOB check.
- [ ] **Step 6:** Commit: `refactor(kali_npm): extract install module and remove monolithic tests.rs [refactor]`.

---

## Task 11: Adopt `kali_test_support::fixtures`

kali_npm has abundant fs/tempdir tests. Adopt the shared fixtures crate, convert matching sites, hold the count at 45 (the kali_runtime precedent). Non-matching sites stay on the direct `tempfile` dep.

**Files:**
- Modify: `crates/kali_npm/Cargo.toml`
- Modify: the `*_tests.rs` files (and `test_support.rs`) that use `tempdir()` / `fs::write` for simple file setup

**Interfaces:**
- Consumes: `kali_test_support::fixtures::{tempdir, write_file}` (and `write_manifest` if a site writes a manifest).

- [ ] **Step 1: Add the dev-dependency**

In `crates/kali_npm/Cargo.toml` under `[dev-dependencies]`, add (keep the existing `tempfile` line — non-matching sites still need it):
```toml
kali_test_support = { workspace = true }
```
Verify it resolves: `cargo build -p kali_npm --tests 2>&1 | tail -2` (expect no error). If `kali_test_support` is not yet a workspace member dependency alias, mirror how a prior adopting crate (e.g. `kali_runtime`) declares it — check `crates/kali_runtime/Cargo.toml`.

- [ ] **Step 2: Inventory candidate sites**

```bash
grep -rnE 'tempfile::tempdir|TempDir::new|fs::write' crates/kali_npm/src/*_tests.rs crates/kali_npm/src/test_support.rs
```
For each **matching** site — a plain temp-dir creation or a UTF-8 file write used purely as test setup — convert to `kali_test_support::fixtures::tempdir()` / `fixtures::write_file(dir, rel, contents)`. **Leave non-matching sites unchanged:** `NamedTempFile`, binary tarball writes (`fs::write(path, bytes)` where bytes are `Vec<u8>` from `build_package_tarball`), `fs::create_dir*` used for unreadable-dir tricks, and any write whose exact path/permissions the test asserts on.

- [ ] **Step 3: Convert matching sites**

Replace each matching site with the fixtures call (exact substitution depends on the site; preserve behavior — same dir, same file contents). Example shape:
```rust
// before
let dir = tempfile::tempdir().unwrap();
std::fs::write(dir.path().join("kali.json"), contents).unwrap();
// after
let dir = kali_test_support::fixtures::tempdir();
kali_test_support::fixtures::write_file(dir.path(), "kali.json", contents);
```
(Confirm the exact `fixtures` signatures by reading `crates/kali_test_support/src/`; adapt the call to match.)

- [ ] **Step 4: Verify green + clippy + count held**

```bash
cargo test -p kali_npm 2>&1 | tail -2                                    # 45 passed
cargo clippy -p kali_npm --all-targets -- -D warnings 2>&1 | tail -2     # clean
cargo test -p kali_npm -- --list 2>/dev/null | grep -E ': test$' \
  | sed -E 's/.*:://; s/: test$//' | sort -u \
  | diff - docs/superpowers/baselines/2026-06-25-kali-npm-tests.txt && echo "BASENAMES MATCH"
```
Expected: 45 passed, clippy clean, `BASENAMES MATCH`. **Do not add a fixture round-trip test** — converted sites already exercise the dep, so the count stays at 45.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_npm/Cargo.toml crates/kali_npm/src/
git commit -m "test(kali_npm): adopt kali_test_support::fixtures [refactor]"
```

---

## Task 12: Final facade verification & whole-crate review prep

**Files:**
- Modify: `crates/kali_npm/src/lib.rs` (only if cleanup is needed)

- [ ] **Step 1: Confirm the facade is thin and logic-free**

```bash
wc -l crates/kali_npm/src/lib.rs   # expect ~40 (no fn bodies, no impls, no types)
grep -nE '^\s*(pub )?(fn|impl|struct|enum) ' crates/kali_npm/src/lib.rs   # expect: no output
```
`lib.rs` should contain only: doc comment, `use` imports, the 4 consts, the 1 static, seven `mod`/`pub use` pairs, and the `#[cfg(test)] mod test_support;` line. If any function/type/impl remains, it was missed by an extraction — move it to its module now and re-run verification before continuing.

- [ ] **Step 2: Confirm the public API is exactly the 20 baseline items**

Build a throwaway probe in the scratchpad that names every public path (do NOT commit it):
```bash
PROBE=/tmp/claude-1000/-workspace/621b6bf6-13ea-433c-bc1d-ec718fbcf6e3/scratchpad/api_probe.rs
sed 's/^kali_npm::/use kali_npm::/; s/$/;/' docs/superpowers/baselines/2026-06-25-kali-npm-api.txt > "$PROBE"
echo 'fn main() {}' >> "$PROBE"
# compile against the built rlib (adjust the --extern path to the actual artifact):
cargo build -p kali_npm 2>&1 | tail -1
RLIB=$(find target/debug/deps -name 'libkali_npm-*.rlib' | head -1)
rustc --edition 2021 --crate-type bin -L target/debug/deps --extern kali_npm="$RLIB" "$PROBE" -o /dev/null 2>&1 | tail -5 && echo "API PROBE OK"
```
Expected: `API PROBE OK` with no `unresolved import` errors (every one of the 20 paths resolves). If any path fails, the corresponding `pub use` glob was dropped too aggressively — restore the flat re-export for that item.

- [ ] **Step 3: Full workspace build + final crate gate**

```bash
cargo build --workspace 2>&1 | tail -3
cargo test -p kali_npm 2>&1 | tail -2
cargo clippy -p kali_npm --all-targets -- -D warnings 2>&1 | tail -2
cargo test -p kali_npm -- --list 2>/dev/null | grep -E ': test$' \
  | sed -E 's/.*:://; s/: test$//' | sort -u \
  | diff - docs/superpowers/baselines/2026-06-25-kali-npm-tests.txt && echo "BASENAMES MATCH"
```
Expected: workspace builds; 45 passed; clippy clean; `BASENAMES MATCH`.

- [ ] **Step 4: Verbatim spot-check (whitespace discipline)**

Confirm no logic drift crept in during movement — compare the sorted set of non-blank source lines before/after across the split files vs. the original `lib.rs`+`tests.rs` at the pre-refactor commit:
```bash
git show 32ea9f883~1:crates/kali_npm/src/lib.rs > /tmp/old_lib.rs 2>/dev/null || \
  git show HEAD~11:crates/kali_npm/src/lib.rs > /tmp/old_lib.rs
# (use the pre-Task-2 commit hash for the original lib.rs)
```
Then have the reviewer confirm each module's moved bodies are byte-identical to their origin (the per-task commits make this a focused diff). This is a manual reviewer gate, not an automated assert.

- [ ] **Step 5: Commit any cleanup**

If Steps 1–4 required edits:
```bash
git add crates/kali_npm/src/lib.rs
git commit -m "refactor(kali_npm): finalize facade [refactor]"
```
If no edits were needed, skip the commit.

---

## Self-Review (completed by plan author)

**Spec coverage:**
- 7-module thematic cut → Tasks 4–10 (one per module). ✓
- Thin facade with `pub use` re-exports → Tasks 4–10 wiring + Task 12 Step 1. ✓
- Co-located `*_tests.rs` → Tasks 6–10. ✓
- Shared `test_support` → Task 3. ✓
- `pub(crate)` widening for dense cross-module calls → Task 2. ✓
- Consts/static stay in lib.rs + const-fallback rule → Global Constraints + recipe step 4. ✓
- GLOB/no-glob rule → Global Constraints + recipe step 6. ✓
- Fixture adoption (adopt+convert, hold at 45) → Task 11. ✓
- Giant fns move intact, cracking out of scope → Task 10 Step 1. ✓
- Invariants (zero behavior change, 20-item API, 45-test basename set, green+clippy every commit, whitespace-verbatim) → Global Constraints + per-task verification + Task 12. ✓
- Baselines captured first → Task 1. ✓

**Placeholder scan:** No "TBD"/"TODO"/"handle edge cases". The one deferral (cracking `install_project`/`install_registry_package`) is explicitly out of scope per spec, not a gap. ✓

**Type consistency:** All 12 types and 80 functions are assigned to exactly one module; names in the "Produces" interface blocks match the File Structure table and the spec's 20-item public list. Function-count sum (80) and type-count sum (12) verified against the source inventory. ✓
