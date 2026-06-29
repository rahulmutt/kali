# kali_capi co-located src test-monolith modularization — design

**Series:** 31st crate-modularization entry. Seventh entry of the post-kali_cli frontier
(other crates' co-located src unit-test monoliths; kali_optimize was 25th, kali_types 26th,
kali_runtime 27th, kali_codegen 28th, kali_common 29th, kali_parser 30th).
**Date:** 2026-06-29
**Branch base / main HEAD at start:** `d38cd3aa2`

## Goal

Split three of kali_capi's co-located `src/*_tests.rs` unit-test monoliths into a thin facade +
per-concern `#[path] mod` submodules grouped on a **semantic axis**. **Pure verbatim
code-motion, zero behavior change**, identical compiled test set, byte-identical public API (the
crate and its consumers compile unedited).

| file | lines | `#[test]` fns | declared from | facade model |
|---|---|---|---|---|
| `src/manifest_tests.rs` | 771 | 19 | `manifest.rs:502` | drain to 0 `#[test]`; retain 1 non-test helper |
| `src/binding_tests.rs` | 414 | 7 | `lib.rs:24` | drain to 0 |
| `src/metadata_tests.rs` | 328 | 9 | `metadata.rs:427` | drain to 0 |

35 `#[test]` fns total across three files. This is **not** TDD. No new product code, no new
tests, no renames, no reformatting. `binding_tests` and `metadata_tests` drain to **0**
module-level fns (every module-level fn is a `#[test]`). `manifest_tests` retains exactly **one**
non-`#[test]` module-level helper, `valid_binding_package_manifest` (line 5), which the mover
auto-leaves in the facade. Any nested helper travels with its parent test body.

### How the test modules are wired

Each monolith is declared via:

```rust
#[path = "<name>_tests.rs"]
mod <name>_tests;
```

`manifest_tests` and `metadata_tests` are declared at the foot of their product module
(`manifest.rs:502`, `metadata.rs:427`), so `super::` inside those test files resolves to the
**product** module. `binding_tests` is declared from `lib.rs:24`, so its `super::` resolves to the
crate root. In all three the facade keeps its original `use` lines (`use crate::*;` + std
imports); each submodule opens with `use super::*;`, which re-propagates everything the facade
brought into scope. This mirrors the kali_parser (30th) / kali_codegen (28th) structure exactly.

### `include_*!` pins

**Zero** `include_str!`/`include_bytes!`/`include!` in all three files (verified). No
file-relative path resolution breaks on the one-dir-deeper move, so the mover needs **no facade
pin arg**.

### Out-of-scope files (kept whole)

kali_capi's remaining co-located `src/*_tests.rs` file `header_tests.rs` (2 `#[test]`, declared
from `header.rs:95`) is already small and single-concern; it stays as-is this entry (untouched).
This matches the kali_common precedent of keeping small files whole.

## Per-file split (semantic axis)

### `manifest_tests.rs` (19 → 4 submodules) — axis: manifest operation

All tests share the `binding_package_manifest_` prefix; grouped by the **mid-name discriminator**
(exact-name-set, not leading-prefix).

- **`parsing`** (8): `binding_package_manifest_parsing_*` —
  `normalizes_string_lists`, `rejects_whitespace_padded_string_lists`,
  `rejects_whitespace_padded_artifact_paths`, `rejects_non_integer_max_specializations`,
  `rejects_negative_max_specializations`, `rejects_unexpected_keys`,
  `rejects_invalid_required_field_types`, `rejects_non_string_provenance_fields`
- **`helpers`** (5): `binding_package_manifest_helpers_*` —
  `reject_whitespace_padded_module_name`, `reject_empty_provenance_fields`,
  `reject_empty_or_whitespace_artifact_paths`, `reject_ambiguous_auto_discovery`,
  `load_discover_and_summarize_manifests`
- **`summary`** (3): `binding_package_manifest_summary_*` —
  `normalizes_string_lists`, `rejects_invalid_required_field_types`,
  `rejects_non_string_provenance_fields`
- **`construction`** (3): the remaining tests not matching parsing/summary/helpers —
  `binding_package_manifest_orders_and_deduplicates_glue_deterministically`,
  `binding_package_manifest_with_provenance_uses_explicit_contract_labels`,
  `binding_package_manifest_rejects_incompatible_host_abi_version_window`

Non-`#[test]` helper `valid_binding_package_manifest` is auto-retained in the facade.

### `binding_tests.rs` (7 → 2 submodules) — axis: target language

Grouped by **leading prefix** (`python_` / `javascript_`).

- **`python`** (5): `python_binding_package_metadata_is_present`,
  `python_binding_wraps_generated_header_exports`,
  `python_binding_auto_discovers_stem_specific_binding_package_manifest`,
  `python_binding_rejects_incompatible_host_abi_metadata`,
  `python_unittest_smoke_covers_the_binding_helper_package`
- **`javascript`** (2): `javascript_binding_package_metadata_is_present`,
  `javascript_node_test_smoke_covers_the_binding_helper_package`

### `metadata_tests.rs` (9 → 3 submodules) — axis: metadata operation

Grouped by the **mid-name discriminator** (exact-name-set).

- **`helpers`** (5): `cabi_metadata_helpers_*` —
  `load_and_summarize_generated_payloads`, `discover_load_and_summarize_root_sidecars`,
  `reject_incompatible_host_abi_version_windows`, `reject_ambiguous_auto_discovery`,
  `reject_empty_provenance_fields`
- **`generation`** (2): `metadata_generation_*` —
  `includes_expected_artifacts`, `with_provenance_keeps_optional_fields_deterministic`
- **`parsing`** (2): `cabi_metadata_parsing_*` —
  `rejects_unexpected_keys`, `rejects_negative_max_specializations`

**Total: 35 tests → 9 submodules.**

## Mechanical procedure (per file)

1. Create subdir `src/<name>_tests/`.
2. Move each `#[test]` fn **verbatim** into its target group file, prepending a single
   `use super::*;` line (via `move_fns.py`).
3. Reduce the facade to its original `use` line(s) (plus the retained non-test helper for
   `manifest_tests`) and one `#[path = "<name>_tests/<group>.rs"] mod <group>;` declaration per
   group.

## Verification

- `cargo test -p kali_capi --lib` — count identical before and after (37 pass / 0 fail), all pass.
- `cargo build -p kali_capi --tests` — 0 warnings unchanged.
- Per-file `--list` filter count preserved (19 / 7 / 9).
- `verify.py` byte-identity PROOF OK (orig vs submodule glob, + facade glob for the manifest
  retained helper).
- Each facade `#[test]` count → 0.
- One dependent crate compiles unedited (public API byte-identical).
- `cargo fmt --check` — accept known fmt nits per series convention (do **not** run `cargo fmt`).
- `git diff --stat` confirms only test-file code-motion (facades shrink; new submodule files
  added; product `#[path] mod` decls unchanged).

## Commit shape

Mirrors the series:

1. `docs(kali_capi): design spec for co-located src test-monolith modularization [spec]`
2. `docs(kali_capi): implementation plan for src test-monolith modularization [plan]`
3. One `refactor(kali_capi): split <file>_tests.rs into per-concern test submodules [refactor]`
   commit **per file** (3 refactor commits).

Local-main ff-merge only; no origin push.
