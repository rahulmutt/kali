# kali_capi modularization — design

**Date:** 2026-06-26
**Crate:** `kali_capi` (14th in the kali crate-modularization series)
**Predecessors:** the api trio `kali_api_web` (11th) / `kali_api_node` (12th) / `kali_api_deno` (13th). This spec reuses the series' facade + co-located-tests playbook and records where capi — a new structural shape — diverges.
**Execution:** subagent-driven-development, one module per task.
**Integration:** fast-forward merge to **local main only** (match crates 2–10, 12, 13; only `kali_api_web` was pushed to origin). Re-verify on merged main, delete branch.

## Goal

Decompose the monolithic `crates/kali_capi/src/lib.rs` (1203 lines) into a thin facade plus per-artifact modules, and split `src/tests.rs` (53 KB, 37 tests) into co-located sibling `*_tests.rs` files — with **zero behavior change** and a **preserved public API**.

## What this crate actually is

Despite the name, `kali_capi` is **not** `extern "C"` / FFI code. It is a **deterministic artifact generator**: pure functions that produce, parse, validate, summarize, and load three things consumed by the embedding projection:

1. **C header text** (`generate_header` → a `.h` string),
2. **cabi-metadata** JSON sidecars,
3. **binding-package manifest** JSON sidecars (plus a "bundle" = manifest + metadata combined summary).

There is **no unsafe code, no `#[no_mangle]`, no `repr(C)`, no extern blocks.** The surface is one public struct (`Export`, with one inherent method `Export::new`), one public const (`HOST_ABI_VERSION`), **31** public free functions, and ~10 private helpers (two family-local generate-validators + eight shared JSON field-validators).

## Shape: FLAT FUNCTION-PILE grouped by output artifact

A new shape for the series. Predecessors web/node/deno were INDEPENDENT-OBJECT-PILEs (a struct + its impls per family). `kali_capi` has almost no types — it is free functions that cluster cleanly by **which artifact they touch**. The natural seam is the artifact, not a receiver type. There is no shared mega-struct, so there is **no Task-1 blanket `pub(crate)` receiver-widening**.

Current public surface (flat `kali_capi::Name` paths):

- `HOST_ABI_VERSION` (const), `Export` (struct + `Export::new`)
- header: `arity_from_signature`, `generate_header`, `sanitize_identifier`
- metadata: `generate_metadata`, `generate_metadata_with_provenance`, `parse_metadata`, `cabi_metadata_summary`, `load_metadata`, `load_metadata_summary`, `discover_metadata_path`, `discover_metadata_path_with_name`, `load_metadata_from_root`, `load_metadata_summary_from_root`, `load_metadata_from_root_with_name`, `load_metadata_summary_from_root_with_name`
- manifest: `generate_binding_package_manifest`, `generate_binding_package_manifest_with_provenance`, `parse_binding_package_manifest`, `binding_package_manifest_summary`, `discover_binding_package_manifest_path`, `discover_binding_package_manifest_path_with_name`, `load_binding_package_manifest`, `load_binding_package_manifest_summary`, `load_binding_package_manifest_from_root`, `load_binding_package_manifest_summary_from_root`, `load_binding_package_manifest_from_root_with_name`, `load_binding_package_manifest_summary_from_root_with_name`
- bundle: `binding_package_bundle_summary`, `load_binding_package_bundle_summary`, `load_binding_package_bundle_summary_from_root`, `load_binding_package_bundle_summary_from_root_with_name`

**Sole external consumer:** `crates/kali_cli/src/bin/kali.rs`, importing 7 flat names:
`arity_from_signature, generate_binding_package_manifest_with_provenance, generate_header, generate_metadata_with_provenance as generate_capi_metadata, parse_binding_package_manifest, parse_metadata, Export as CApiExport`. All flat → the glob facade preserves them with zero consumer edits.

## Architecture

`lib.rs` 1203 → **thin facade**: 5 `mod` decls, 4 `pub use <mod>::*;` globs, plus the crate-level const kept at root.

### Module decomposition (4 public families + 1 internal)

| module | facade | contents | notes |
|---|---|---|---|
| `header` | `pub use header::*;` | `Export` (struct + impl), `arity_from_signature`, `generate_header`, `sanitize_identifier` | the only family with a public type |
| `metadata` | `pub use metadata::*;` | `generate_metadata*` (2), `parse_metadata`, `cabi_metadata_summary`, `load_metadata*` (2), `discover_metadata_path*` (2), `load_metadata_*_from_root*` (4); private `validate_generated_cabi_metadata` (family-local, 3 refs, all metadata) | ~12 public fns |
| `manifest` | `pub use manifest::*;` | `generate_binding_package_manifest*` (2), `parse_binding_package_manifest`, `binding_package_manifest_summary`, `discover_binding_package_manifest_path*` (2), `load_binding_package_manifest*` (6); private `validate_generated_binding_package_manifest` (family-local) | ~12 public fns; dir-scan logic duplicated from metadata (not shared — no widening) |
| `bundle` | `pub use bundle::*;` | the 4 `*bundle_summary*` fns | composes manifest + metadata via their **public** surface only |
| `validate` | **no glob** | 8 shared JSON field-validators: `reject_unexpected_keys`, `validate_string_field`, `validate_non_empty_string_field`, `validate_integer_field`, `validate_non_negative_integer_field`, `integer_value`, `validate_host_abi_version_window`, `normalize_string_list_value` — all `pub(crate)` | THE widening; internal only |

### Crate-level const stays in the facade

`HOST_ABI_VERSION` is public and referenced by **both** metadata (4 sites) and manifest (2 sites) generation. It stays declared at the crate root in the facade; modules reference it as `crate::HOST_ABI_VERSION`. Keeping a shared crate-level item at the root (rather than arbitrarily homing it in one family and re-exporting) follows the deno precedent of keeping `deno_api_init()` in the facade. The flat path `kali_capi::HOST_ABI_VERSION` is preserved unchanged.

The glob facade preserves every flat `kali_capi::Name` path → zero consumer edits.

## Widening — one real site (matches prediction)

The 8 JSON field-validators are **private** free fns used by **both** the metadata family (`parse_metadata`, `cabi_metadata_summary`) and the manifest family (`parse_binding_package_manifest`, `binding_package_manifest_summary`). Reference counts in the monolith: `validate_non_empty_string_field` 17, `validate_integer_field` 7, `normalize_string_list_value` 6, `validate_host_abi_version_window` 5, `validate_non_negative_integer_field` 5, `reject_unexpected_keys` 5, `integer_value` 3, `validate_string_field` 2.

**Resolution:** a dedicated internal `validate` module. `mod validate;` holds all 8 as `pub(crate)`. The facade declares `mod validate;` with **no** `pub use` glob — they stay internal, nothing leaks to the public surface. `metadata` and `manifest` both `use crate::validate::{…}`.

This is structurally the **deno `path` precedent**: a private helper set shared by two families, named honestly as an internal `pub(crate)` module with no glob. The coupling is real and small; neither family owns a validator the other reaches into, so a shared module is the honest home.

**Verified clean elsewhere** (no further widening):

- The two `discover_*_path_with_name` fns **duplicate** their directory-scan bodies rather than share a helper → each stays family-local.
- `bundle` calls only **public** fns of metadata/manifest (`cabi_metadata_summary`, `binding_package_manifest_summary`, the `load_*`/`discover_*` entry points) → all glob-exported, no widening.
- `header` uses no validators; `sanitize_identifier` is header-local.
- `validate_generated_cabi_metadata` / `validate_generated_binding_package_manifest` are each used only within their own family → stay private to `metadata` / `manifest` respectively (not in the shared `validate` module).

## Collision check (E0255 / shadowing)

Module names `header`, `metadata`, `manifest`, `bundle`, `validate` do **not** collide with anything `use`d at the crate root. Unlike deno (`mod fs` vs `use std::fs` → E0255), there is no clash: the crate uses `std::fs` and `std::path::{Path, PathBuf}`, none of which share a name with a module. The fs/path-touching modules (`metadata`, `manifest`, `bundle`) each carry their own `use std::fs;` / `use std::path::{Path, PathBuf}` and `use serde_json::{json, Value}` as needed.

## Tests

`src/tests.rs` (37 `#[test]` fns, 53 KB) splits into co-located sibling `*_tests.rs` files. The split does **not** map 1:1 onto the 4 public modules — verified by reading every test:

| test file | count | wired from | tests |
|---|---|---|---|
| `header_tests.rs` | 2 | `header.rs` | `header_generation_*`, `identifier_sanitization_*` |
| `metadata_tests.rs` | 9 | `metadata.rs` | `metadata_generation_*` (2), `cabi_metadata_*` (7) |
| `manifest_tests.rs` | 19 (+1 helper) | `manifest.rs` | `binding_package_manifest_*` (incl. the `*_summary_*` and parsing/rejection tests) + the `valid_binding_package_manifest()` helper fn |
| `binding_tests.rs` | 7 | **facade (`lib.rs`)** | the cross-cutting end-to-end `python_binding_*` (4), `python_unittest_smoke_*`, `javascript_binding_package_metadata_is_present`, `javascript_node_test_smoke_*` |

**`bundle` has no standalone tests.** Its four fns are exercised inside the manifest test `binding_package_manifest_helpers_load_discover_and_summarize_manifests` (which calls `load_binding_package_bundle_summary*`), so bundle coverage rides in `manifest_tests.rs`. This is a divergence from a naive "one test file per module" expectation, recorded honestly.

**`binding_tests.rs` is crate-level.** The 7 end-to-end tests spawn `python3`/`node` subprocesses (returning early if the toolchain is absent), read `bindings/python` and `bindings/node` fixtures, and exercise header+metadata+manifest together. They belong to no single module → declared from the facade with `#[cfg(test)] #[path = "binding_tests.rs"] mod binding_tests;`. (The two `*_package_metadata_is_present` tests call no capi fn at all — they only read `bindings/` files — confirming they are integration, not unit, tests.)

Each module file ends with `#[cfg(test)] #[path = "<name>_tests.rs"] mod <name>_tests;`. The internal `validate` module is tested through the `parse_*`/`*_summary` paths in `metadata_tests.rs` and `manifest_tests.rs`; it gets no standalone test file.

**Self-sufficiency rule (series lesson from deno `net_tests`):** each `*_tests.rs` must `use` everything it needs explicitly. Do not rely on test-glob freeloading through a crate-root glob. Because `cargo build` skips `cfg(test)`, a freeloading test compiles under build but fails under `cargo test` — so verification for each test-split task must run `cargo test -p kali_capi`, not just `cargo build`.

## Task ordering (one module per task)

1. `validate` (internal module first — both metadata and manifest depend on it; extracting it first lets the later families `use crate::validate::*` immediately).
2. `header` (independent; smallest; carries the only public type).
3. `metadata` (depends on `validate` + `crate::HOST_ABI_VERSION`).
4. `manifest` (depends on `validate` + `crate::HOST_ABI_VERSION`).
5. `bundle` (depends on metadata + manifest public surface).
6. Facade finalization: confirm `lib.rs` is `HOST_ABI_VERSION` + 5 `mod` + 4 globs + crate-level `mod binding_tests` wiring; co-locate the five `*_tests.rs` splits (`header`/`metadata`/`manifest` from their modules, `binding_tests` from the facade); delete `tests.rs`.

Each task: extract → `cargo build -p kali_capi` → `cargo test -p kali_capi` → commit.

## Verification (definition of done)

- `cargo build -p kali_capi` clean, **no new warnings** (watch for now-unused imports left in the facade).
- `cargo test -p kali_capi` — all 37 tests pass.
- `cargo build -p kali_cli` / `cargo test -p kali_cli` — the sole consumer compiles and passes **without edits** (proves the flat public API is preserved).
- Whole-workspace `cargo build` + `cargo test` green on merged main.
- **Basename-multiset proof** (series invariant): the multiset of public item names exported from the facade is identical before and after. Concretely: the set of `kali_capi::Name` flat paths is unchanged (1 const + 1 struct + 31 fns = 33 flat names), confirmed by diffing the pre/post exported-symbol list.
- No item added to or removed from the public surface; `validate`'s 8 fns remain `pub(crate)`, never reachable as `kali_capi::…`.

## Non-goals

- No behavior changes, no signature changes, no renames of public items.
- No dedup of the two near-identical `discover_*` dir-scan bodies (that is a real cleanup, but out of scope — modularization is structure-only; flag it for a follow-up if desired).
- No new dependencies; no dependency removals. Note: `kali_common` is declared in `Cargo.toml` but **not actually used** by `lib.rs` or `tests.rs` (a pre-existing unused dependency). Removing it is a real cleanup but **out of scope** here — modularization is structure-only; flag it for a follow-up if desired.
