# kali_npm modularization — design

**Date:** 2026-06-25
**Crate:** `kali_npm` (6th crate in the kali modularization effort)
**Pattern:** reuses the established facade + co-located-tests pattern; closest precedent is `kali_common` (flat free-fn pile) with the fixture handling of `kali_runtime` (abundant fs tests).

## Goal

Break the monolithic `crates/kali_npm/src/lib.rs` (2,892 lines) and `tests.rs` (2,306 lines) into thematic modules behind a thin `lib.rs` facade.

**Hard invariants** (replicated from every prior crate in this effort):

- Zero behavior change.
- Identical public API — exactly **20 items** (8 pub types + 12 pub fns).
- Identical test-name set — **45 tests**, compared by basename (stripping module-path prefixes, since `cargo test -- --list` includes them).
- Green + clippy-clean (`cargo clippy -p kali_npm --all-targets -- -D warnings`) at every commit.
- Verbatim text-movement, **including whitespace** (blank-line separators count).

## Current shape

A mostly-flat free-function pile:

- 12 `pub fn` + 68 private `fn` = 80 free functions.
- 8 public types: `ProjectManifest`, `LockFile`, `LockedPackage`, `RawUrlEntry`, `PackageTarget`, `InstallOptions`, `InstallSummary`, `RegistryPackageAudit`.
- 3 small `impl` blocks: `ProjectManifest`, `LockFile`, `PackageHostFitContext`.
- Crate-level state/consts in `lib.rs`: `MANIFEST_SCHEMA`, `LOCK_VERSION`, `DEFAULT_NPM_REGISTRY`, `NODE_ONLY_HOST_APIS`, and the `REGISTRY_METADATA_CACHE` `OnceLock<Mutex<…>>` static.

**The one structural difference from `kali_common`:** kali_npm's private fns call each other densely *across* the thematic boundaries (install → registry, tarball, validate, resolve; etc.). So unlike kali_common (almost no widening), **most private fns widen to `pub(crate)`** up front to enable pure text-movement extraction. This mirrors the receiver-widening step of the `impl`-based crates, applied to free fns.

## Module decomposition (7 flat sibling modules)

Each module is a sibling `src/<mod>.rs`, populated by pure verbatim text-movement. Types live with their domain.

| Module | Contents (rough) |
|--------|------------------|
| `manifest` | `ProjectManifest`, `LockFile`, `LockedPackage`, `RawUrlEntry` types (+ their impls); `discover_project_root`, `load_manifest`, `save_manifest`, `load_lock`, `save_lock`, `ensure_project_ready`, `project_requires_install`; manifest/lock helpers (`manifest_registry_package_keys`, `split_package_key`, `validate_manifest_registry_collisions`, …) |
| `install` | `install_project` (driver, 236 lines), `install_registry_package` (265), `install_raw_url`, `record_install_path`, reconcile/prune raw-urls, source-file discovery (`discover/collect/is_install_source_file`, `collect_source_module_specifiers`, `resolve_import_map_specifier`), lifecycle hooks, `has_effective_npm_scriptable_install_work`, `InstallOptions`/`InstallSummary` |
| `registry` | `resolve_registry_package`, `resolve_npm_package`, `resolve_jsr_package`, `resolve_npm_like_package`, `fetch_registry_metadata`, registry url builders, `select_registry_version`, `audit_registry_package`, `audit_package_version_metadata`, `RegistryPackageAudit` |
| `target` | `PackageTarget`, `parse_package_target`, `split_package_name_and_version`, `encode_package_name`, `package_key`, `install_name_from_package`, `jsr_compat_name`, `types_package_name`, `split_bare_package_source` |
| `validate` | `PackageJson`, `read_package_json`, `validate_package_shape`, `value_contains_native_addon_path`, `script_uses_native_bootstrap_tool`, `validate_package_host_fit`, `scan_for_node_only_host_api`, `source_mentions_node_only_host_api`, `is_scannable_package_source`, `should_skip_package_scan_dir`, `PackageHostFitContext` (+ impl), `NODE_ONLY_HOST_APIS` |
| `tarball` | `download_bytes`, `verify_tarball_integrity`, `integrity_matches`, `format_sha512`, `sha256_hex`, `extract_tarball`, `copy_tree`, `recursive_copy`, `raw_url_file_name` |
| `resolve` | `resolve_materialized_import`, `resolve_materialized_import_with_browser_context`, `resolve_types_package_import`, `resolve_package_types_entry`, `resolve_package_entry`, `resolve_package_subpath`, `resolve_package_exports`, `resolve_package_exports_target`, `apply_browser_rewrite`, `substitute_export_pattern`, `match_export_pattern`, `resolve_package_file` |

Exact function-to-module assignment for the handful of ambiguous helpers is finalized in the implementation plan; the thematic cut above is approved.

## Facade (`lib.rs`, ~40 lines)

No logic. Contains:

- The crate's import surface (`use base64::Engine;` … `use tar::Archive;`).
- Crate-level `const`s + the `REGISTRY_METADATA_CACHE` static (stay in `lib.rs`).
- `mod <name>;` decls for the 7 modules.
- `pub use <mod>::*;` per module — preserves the flat `kali_npm::<name>` public paths so **zero consumer edits** are needed.
- `#[cfg(test)]` wiring is per-module (see Tests), not in the facade.

**Const fallback** (per kali_codegen/kali_optimize): if `use crate::*;` in a moved module does not surface a bare crate-root `const`, qualify at use site as `crate::<CONST>` — mechanical, no logic change.

**GLOB / no-glob rule** (per kali_optimize/kali_common): a module is re-exported with `pub use <mod>::*;` when it owns public items that must keep flat paths (all 7 do, since they own pub fns/types). Inside each module, `use crate::*;` surfaces crate-root privates and sibling `pub(crate)` items; drop the glob from any module clippy flags as a dead import. Do not `#[allow]` a dead glob — delete it.

## Tests

- 45 tests split into 7 sibling `src/<mod>_tests.rs`, distributed by theme.
- Each wired at the bottom of its source module: `#[cfg(test)] #[path = "<mod>_tests.rs"] mod <mod>_tests;`.
- Test modules use `use crate::*;` (not `use super::*;`). A moved test that reads a module-private item resolves via `use super::<ITEM>;` after `use crate::*;` — no extra visibility widening (per kali_common's private-const precedent).

**Fixture adoption (adopt + convert, hold at 45 — kali_runtime precedent):**

- kali_npm has abundant fs/tempdir tests (29 tempdir refs, 53 fs::write/create refs) and a direct `tempfile` dev-dep.
- Adopt `kali_test_support::fixtures`; convert matching `tempdir` / `write_file` sites to it.
- Leave non-matching sites as-is (`NamedTempFile`, binary tarball `Vec<u8>` writes, any unreadable-dir tricks) — partial adoption is correct.
- The `tempfile` dev-dep stays (non-matching sites still need it). The shared crate is added as a dev-dep.
- Suite **stays at 45** — no added fixture round-trip test (the dep is genuinely exercised by converted sites, so the kali_codegen/kali_optimize "add one test" tension does not recur).

## Giant functions — out of scope to crack

`install_registry_package` (265 lines) and `install_project` (236 lines) move **byte-for-byte intact** into `install`. There is no true mega-function (cf. kali_runtime's 700–800-line monsters). Cracking these into helpers is a separate deferred logic refactor, explicitly out of scope here.

## Process

- subagent-driven-development, one module per task: extract → `cargo test -p kali_npm` green → `cargo clippy -p kali_npm --all-targets -- -D warnings` clean → next.
- **Baseline captured first** (whole-crate `cargo test -p kali_npm -- --list` basename set = 45; public-API probe = 20 items) and committed.
- Verify clippy (not just `cargo test`) after **every** task — the kali_runtime learning: `cargo test` does not gate warnings.
- Prove the identical-name-set invariant by comparing **basenames** (strip module-path prefixes), not a raw `--list` diff (non-empty by design after co-location).
- Final whole-branch opus review → must be 0 Critical / 0 Important → fast-forward merge to main locally (consistent with prior crates; push is a separate explicit step).

## Invariants checklist (verify at the end)

- [ ] Zero behavior change.
- [ ] Public API = exactly 20 items (8 types + 12 fns), flat `kali_npm::<name>` paths preserved.
- [ ] 45-test basename set identical to baseline.
- [ ] Green + clippy `-D warnings` clean at every commit.
- [ ] `lib.rs` is a thin (~40-line) facade with no logic.
- [ ] Verbatim text-movement including whitespace.
