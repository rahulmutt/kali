# kali_npm src test-monolith modularization — Design Spec

**Series:** crate-by-crate test-monolith modularization, entry **#32** (kali_npm). Follows the
established pattern (kali_capi #31, kali_parser #30, kali_common #29, kali_codegen #28,
kali_runtime #27, kali_types #26, kali_optimize #25, …).

## Goal

Split kali_npm's three multi-concern co-located `src/*_tests.rs` unit-test monoliths into thin
facades + per-concern `#[path] mod` submodules, by **pure verbatim code-motion**. Zero behavior
change; the compiled test set is byte-for-byte identical.

## Scope

**In scope (3 files, 36 `#[test]` fns):**

| File | Tests | Lines | Product sibling decl (unchanged) |
|------|------:|------:|----------------------------------|
| `src/install_tests.rs` | 15 | 515 | `install.rs:1106` `#[cfg(test)] #[path = "install_tests.rs"] mod install_tests;` |
| `src/resolve_tests.rs` | 7 | 260 | `resolve.rs:393` `#[cfg(test)] #[path = "resolve_tests.rs"] mod resolve_tests;` |
| `src/validate_tests.rs` | 14 | 236 | `validate.rs:296` `#[cfg(test)] #[path = "validate_tests.rs"] mod validate_tests;` |

**Out of scope (kept whole, per series precedent for small/single-concern files):**
`src/manifest_tests.rs` (6 tests, 120 L) and `src/registry_tests.rs` (3 tests, 77 L). These stay
exactly as they are.

**Baseline (captured 2026-06-30):**
- `cargo build -p kali_npm --tests` → **0 warnings**.
- `cargo test -p kali_npm --lib` → **45 passed; 0 failed** (36 in-scope + 6 manifest + 3 registry).
- Per-file filter counts: `install_tests` 15, `resolve_tests` 7, `validate_tests` 14.
- **0** non-`#[test]` (helper) module-level fns in all three files.
- **0** `include_str!`/`include_bytes!`/`include!` occurrences in all three files.
- **0** `pub`/`pub(crate)` widening needed.

## Architecture

Each `src/<F>_tests.rs` is declared from its product sibling via
`#[cfg(test)] #[path = "<F>_tests.rs"] mod <F>_tests;` — **all three decl sites stay unchanged**.

Each monolith becomes a **facade** that retains only:
- its original `use` line(s), verbatim, and
- one `#[path = "<F>_tests/<group>.rs"] mod <group>;` declaration per group.

Every `#[test]` fn moves **verbatim** (full body, no reformatting, no rename) into the matching
`src/<F>_tests/<group>.rs`, each of which begins with exactly `use super::*;` and nothing else
before the first moved fn.

**Why retained facade `use` lines compile at 0 warnings:** when a facade drains to 0 fns, its
retained `use` imports are reached by the submodules through their `use super::*;` glob — Rust
re-exports a facade's private `use` items through descendant `use super::*;` (proven 0-warning in
kali_optimize #25 and kali_types #26, which retained the same `use crate::test_support::*;` shape).
No `#[cfg(test)] use ...` re-export workaround is needed here.

## Per-file concern groupings (36 tests → 9 submodules)

### `install_tests` (15 → 4) — exact-name-set grouping (mixed/mid-name discriminators)

- **`rejections`** (6): `install_rejects_allow_scripts_without_effective_npm_work`,
  `install_rejects_allow_scripts_for_jsr_targets`,
  `install_rejects_allow_scripts_for_raw_url_targets`,
  `install_rejects_dev_without_explicit_target`,
  `install_rejects_dev_for_raw_url_targets`,
  `install_rejects_versioned_registry_targets`
- **`reconciliation`** (4): `install_reconciles_raw_urls_from_source_import_map_rewrites`,
  `install_is_idempotent_for_unchanged_raw_url_graph`,
  `install_reconciles_semver_style_package_without_allow_scripts`,
  `install_reconciles_semver_style_package_with_allow_scripts_noop`
- **`lifecycle`** (2): `lifecycle_hooks_run_in_order_when_allowed`,
  `lifecycle_hooks_skip_blank_entries`
- **`traversal`** (3): `collect_reachable_registry_packages_rejects_install_path_conflicts`,
  `install_noops_without_manifest_or_dependencies`,
  `install_stops_at_nested_child_project_roots`

Facade retains (verbatim, 6 `use` lines):
```rust
use crate::*;
use crate::test_support::*;
use crate::LOCK_VERSION;
use std::fs;
use std::sync::atomic::Ordering;

use serde_json::json;
```

### `resolve_tests` (7 → 3) — leading-prefix grouping

- **`bare_import`** (2): `bare_import_resolves_from_materialized_package`,
  `bare_import_resolves_via_types_package_dependency`
- **`browser_replacement`** (4): `browser_replacement_maps_rewrite_selected_root_entries`,
  `browser_replacement_maps_rewrite_selected_root_entries_from_explicit_context`,
  `browser_replacement_maps_can_block_selected_root_entries`,
  `browser_replacement_maps_rewrite_selected_subpaths`
- **`exports`** (1): `exports_take_precedence_over_legacy_entry_fields_and_respect_browser_conditions`

Facade retains (verbatim, 2 `use` lines):
```rust
use crate::*;
use std::fs;
```

### `validate_tests` (14 → 2) — leading-prefix grouping

- **`shape`** (9): the 9 `validate_package_shape_*` fns
- **`host_fit`** (5): the 5 `validate_package_host_fit_*` fns

Facade retains (verbatim, 1 `use` line):
```rust
use crate::*;
```

## Tooling (reused as-is — do NOT edit `FN_RE` / `IDENT_CHARS` / `find_close_line`)

- **Mover:** `.superpowers/sdd/move_fns.py` run from `crates/kali_npm`. `install_tests` uses
  exact-name-set group specs; `resolve_tests`/`validate_tests` can use leading-prefix specs. The
  mover auto-retains non-`#[test]` module-level fns (there are none here) and appends the
  `#[path] mod` decls. Manual code-motion producing the exact files above is equally acceptable.
- **Verifier:** `.superpowers/sdd/verify.py <orig_rs> "<submodule_glob>"` proves `{name: body}` of
  `#[test]` fns from the original == from the submodules, exiting non-zero on any mismatch. No
  facade-pin glob needed (no retained helpers).

## Gates / invariants (per task and final)

- **Pure verbatim code-motion, zero behavior change.** No new/renamed/reformatted tests.
- **Facades drain to 0 module-level fns** (retain only `use` lines + `#[path] mod` decls).
- **Submodule header** is exactly `use super::*;`.
- **No `pub`/`pub(crate)` widening, no `include_*!` pins.**
- **Build gate:** `cargo build -p kali_npm --tests 2>&1 | grep -c '^warning'` → `0` (baseline 0).
- **Suite gate:** `cargo test -p kali_npm --lib` → `45 passed; 0 failed` (unchanged).
- **Per-file gate:** `install_tests` 15, `resolve_tests` 7, `validate_tests` 14 (unchanged).
- **Byte-identity proof:** `verify.py` exits 0 for each split file (36/36 bodies byte-identical).
- **Product siblings unchanged:** `install.rs`, `resolve.rs`, `validate.rs` decls untouched;
  `manifest_tests.rs`/`registry_tests.rs` untouched.
- **Dependent crate compiles unedited:** a kali_npm consumer (e.g. `cargo build -p kali_cli`) builds clean.
- **Commits:** one `refactor(kali_npm): split <F>_tests.rs into per-concern test submodules [refactor]`
  per file. Local-main ff-merge only; **no origin push**.
- **Fmt:** accept known nits per series convention; do NOT reformat moved bodies.

## Task breakdown

1. **Task 1** — split `install_tests.rs` (15 → `rejections`/`reconciliation`/`lifecycle`/`traversal`).
2. **Task 2** — split `resolve_tests.rs` (7 → `bare_import`/`browser_replacement`/`exports`).
3. **Task 3** — split `validate_tests.rs` (14 → `shape`/`host_fit`).
4. **Final** — whole-crate re-verify, byte-identity proof, dependent-crate build, whole-branch
   review, ff-merge to local main, branch cleanup.
