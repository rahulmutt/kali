# kali_mir co-located src test-monolith modularization — design spec

**Series entry:** #33 (kali_mir) of the crate-by-crate co-located **src** unit-test-monolith
modularization series. Predecessors: kali_optimize, kali_types, kali_runtime, kali_codegen,
kali_common, kali_cli, kali_capi, kali_parser, kali_npm.

**Date:** 2026-06-30
**Branch:** `refactor/kali_mir-modularization` (off local `main`)
**Integration:** local-main ff-merge **only — never push to origin**.

## Goal

Decompose kali_mir's two multi-concern co-located `src/*_tests.rs` unit-test monoliths into a
thin facade + per-concern `#[path] mod` submodules, with **zero behavior change** and a
**byte-identical** set of `#[test]` bodies. Pure verbatim code-motion + `mod`/`use` wiring; no
production-source edits, no public-surface changes, no `cargo fmt`.

This is the established series pattern; this entry is mechanical.

## Scope

In-scope: the two multi-concern monoliths only — **25 of kali_mir's 36 lib tests**.

| File | Tests | Lines | Module path |
|------|-------|-------|-------------|
| `src/analysis/ownership_analysis_tests.rs` | 13 | 390 | `analysis::ownership_analysis_tests` |
| `src/lower_tests.rs` | 12 | 282 | `lower::lower_tests` |

**Out of scope (kept whole):** the remaining 11 lib tests live in small, already single-concern
co-located `*_tests.rs` files. Splitting them would not improve clarity (series convention:
small/single-concern files stay whole).

Both in-scope files:
- have **0** non-`#[test]` module-level fns (facades drain fully to 0).
- have **0** `include_str!` / `include_bytes!` / `include!` macros (no facade pins needed).
- are declared from their **product sibling** via an existing `#[cfg(test)] #[path = "…"] mod …;`
  pair that is left **untouched** (it now points at the facade):
  - `src/analysis/mod.rs:297-298` → `ownership_analysis_tests`
  - `src/lower.rs:133-134` → `lower_tests`

## Target structure

### File 1 — `src/analysis/ownership_analysis_tests.rs` → facade + 4 submodules

Submodules written to `src/analysis/ownership_analysis_tests/<mod>.rs`, each headed by exactly
`use super::*;`. Final namespace: `analysis::ownership_analysis_tests::<mod>::<test>`.

| Submodule | N | Concern | Tests |
|-----------|---|---------|-------|
| `allocation` | 4 | stack/heap ownership classification from JS source via `analyze()` | `test_stack_local_bindings_stay_stack_allocated`, `test_returned_bindings_become_owned_heap`, `test_captured_bindings_become_shared_heap`, `test_non_escaping_closure_captures_stay_borrowed` |
| `call_escape` | 3 | call-argument escape semantics | `test_call_arguments_escape_to_unknown_callees`, `test_inline_pure_function_calls_do_not_force_argument_escape`, `test_inline_leaking_function_calls_still_escape_arguments` |
| `alias_precision` | 3 | function-alias direct-call precision | `test_aliased_function_expressions_preserve_direct_call_precision`, `test_function_alias_chains_preserve_direct_call_precision`, `test_aliased_function_expressions_still_track_nested_closure_escapes` |
| `aggregate_escape` | 3 | hand-built-HIR object/array/member-assignment escapes | `test_object_literal_values_escape_without_treating_keys_as_identifiers`, `test_array_element_values_escape_to_heap_storage`, `test_assignment_into_member_expressions_marks_rhs_escape` |

Facade retains, verbatim, its 3 `use` lines and nothing else (0 `#[test]`):
```
use crate::test_support::*;
use crate::*;
use kali_hir::{HirNode, HirNodeId, HirNodeKind, LoweringResult as HirLoweringResult};
```
Children reach these symbols via `use super::*;` (Rust descendant-visibility re-exports the
facade's private `use` items through the child glob — verified clean at 0 warnings across prior
series entries).

### File 2 — `src/lower_tests.rs` → facade + 2 submodules

Submodules written to `src/lower_tests/<mod>.rs`, each headed by exactly `use super::*;`. Final
namespace: `lower::lower_tests::<mod>::<test>`.

| Submodule | N | Concern | Tests |
|-----------|---|---------|-------|
| `flavor_metadata` | 9 | `FunctionFlavor` preservation through MIR lowering | `test_mir_lowering_preserves_function_nodes_with_flavor_metadata`, `test_mir_lowering_preserves_function_flavor_metadata`, `test_mir_lowering_preserves_function_flavor_metadata_for_function_expressions`, `test_mir_lowering_preserves_function_flavor_metadata_for_class_methods`, `test_mir_lowering_preserves_function_flavor_metadata_for_class_expressions`, `test_mir_lowering_preserves_function_flavor_metadata_for_default_export_generator_function_declaration`, `test_mir_lowering_preserves_function_flavor_metadata_for_default_export_anonymous_generator_function_declaration`, `test_mir_lowering_preserves_function_flavor_metadata_for_default_export_class_expressions`, `test_mir_lowering_preserves_function_flavor_metadata_for_default_export_class_declarations` |
| `structure` | 3 | structural lowering + validation | `test_mir_lowering_preserves_program_shape`, `test_call_expressions_lower_to_call_nodes`, `test_mir_validation_rejects_out_of_bounds_children` |

Facade retains, verbatim, its 3 `use` lines and nothing else (0 `#[test]`):
```
use crate::test_support::*;
use crate::*;
use kali_hir::FunctionFlavor;
```

## Mechanics

- **Mover:** `.superpowers/sdd/move_fns.py` (run from `crates/kali_mir`), **exact-name grouping**
  (current tool behavior: a `#[test]` fn joins the first group whose member list contains its
  exact name; `*` = catch-all). Both files use mid-name semantic discriminators, so groups are
  expressed as explicit exact-name lists (file 2 may use `structure=*` as the catch-all after the
  9 `flavor_metadata` names). **No edits to `FN_RE` / `IDENT_CHARS` / `find_close_line`.**
- **Verifier:** `.superpowers/sdd/verify.py` proves `{name: body}` of `#[test]` fns is byte-identical
  between the pre-move snapshot and the submodule glob, per file.
- **Pre-move snapshots:** copy both files to a fixed out-of-repo scratch dir
  (`/tmp/claude-1000/-workspace/kali_mir_split_scratch/orig/`) before any move, for verify.py.
- Product siblings (`analysis/mod.rs`, `lower.rs`) and all out-of-scope `*_tests.rs` files are
  **untouched** (diff must be empty for them).

## Gates (literal — no env carve-outs; kali_mir baseline is clean)

Baseline captured 2026-06-30:
- `cargo build -p kali_mir --tests` → **0** warnings
- `cargo test -p kali_mir --lib` → **36 passed; 0 failed**
- per-file `--list` counts: `analysis::ownership_analysis_tests` **13**, `lower::lower_tests` **12**

Post-split, all must hold:
1. `cargo build -p kali_mir --tests` → **0** warnings (unchanged).
2. `cargo test -p kali_mir --lib` → **36 pass / 0 fail** (unchanged).
3. Per-file `--list` count preserved (13 / 12), comparing name-sets with the new module prefix
   stripped (`sed -E 's/::test_/\x00/; …'` → strip to bare fn name) → empty diff vs baseline.
4. `verify.py` byte-identity **PROOF OK** for each file (orig snapshot vs submodule glob).
5. Each facade `#[test]` count == **0**.
6. Changed paths = exactly **2 facades + 6 submodules** (8 files); no production/`pub`-widen/
   `include`/fmt changes.
7. Dependent crate (`kali_cli` or any kali_mir consumer) compiles **unedited**.

**fmt:** do **not** run `cargo fmt`. The repo's `cargo fmt --all --check` gate is already red on
baseline across many crates; verbatim moves may leave minor nits in the moved/facade lines — these
are accepted per series convention and are not regressions.

## Process

- SDD ledger at `.superpowers/sdd/progress.md` (git-ignored scratch), overwritten for this entry.
- 2 implementation tasks (one per file): sonnet implementer → review-package → sonnet task reviewer.
- Final: opus whole-branch review (line-conservation + byte-identity reproof) → ff-merge to local
  `main` → re-verify on merged main → delete branch. **No origin push.**

## Non-goals

- No splitting of the 11 out-of-scope small/single-concern lib-test files.
- No production-source refactoring, no public-API changes, no path rewrites, no `cargo fmt`.
- No origin push (origin/main intentionally lags).
