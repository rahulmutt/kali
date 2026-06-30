# kali_lir `src/tests.rs` Unit-Test Modularization (Series Entry 36)

**Date:** 2026-06-30
**Crate:** `kali_lir`
**Branch:** `refactor/kali_lir-srctests-modularization` (off local `main`)
**Integration:** local-`main` ff-merge only — **never push to origin** (origin/main intentionally lags).

## Goal

Decompose the co-located unit-test monolith `crates/kali_lir/src/tests.rs`
(11 `#[test]` fns, 278 lines) into a thin **drained facade** plus three
per-concern `#[path] mod` submodules under `src/tests/`. Zero behavior change,
byte-identical test bodies, public API untouched, consumers compile unedited.

This is the 36th entry in the crate-modularization series and follows the exact
verbatim-code-motion recipe used for the prior entries (kali_optimize,
kali_types, kali_runtime, kali_codegen, kali_common, kali_mir, kali_api_web,
kali_sandbox, …). It mirrors kali_mir's `lower_tests.rs` split (a
`flavor_metadata` family + a `structure` catch-all), applied to a co-located
`tests.rs`-named file declared from `lib.rs` (as in the 35th entry,
kali_sandbox).

## Baseline (verified)

- Declared from `lib.rs:19–21`: `#[cfg(test)]` / `#[path = "tests.rs"]` / `mod tests;` — **untouched** by this work.
- `cargo build -p kali_lir --tests` → **0 warnings**.
- `cargo test -p kali_lir --lib` → **11 passed; 0 failed** (all 11 lib tests live in `tests.rs`; no other `#[test]` in the crate).
- **0 `include_*!` pins** in `tests.rs` (no facade pinning needed).
- 1 module-level helper fn is **not** `#[test]` and stays in the facade:
  `parse_and_lower` (lines 8–16, returns `MirProgram`). The mover leaves
  non-`#[test]` fns in place automatically; children reach it via `use super::*;`
  (Rust descendant-visibility re-exports the facade's private items through the
  child glob — proven at 0 warnings across prior entries).
- Facade currently retains these `use` lines (kept verbatim): `use super::*;`,
  `use kali_common::FileId;`, `use kali_hir::HirLowerer;`, `use kali_lexer::Lexer;`,
  `use kali_mir::MirLowerer;`, `use kali_parser::Parser;`.
- `lib.rs:16–17` (`#[cfg(test)] use kali_mir::MirProgram;`) — required by the
  retained helper's return type, reached via the facade's `use super::*;` —
  **untouched**.

## Architecture

```
crates/kali_lir/src/
  lib.rs                         # lines 16–21 decl UNCHANGED
  tests.rs                       # FACADE: 6 use-lines + 1 helper + 3 `#[path] mod` decls, 0 #[test]
  tests/
    flavor_metadata.rs           # use super::*; + 8 verbatim #[test] fns
    validation.rs                # use super::*; + 1 verbatim #[test] fn
    structure.rs                 # use super::*; + 2 verbatim #[test] fns
```

The facade drains to **0 module-level `#[test]` fns** and appends:

```rust
#[path = "tests/flavor_metadata.rs"]
mod flavor_metadata;
#[path = "tests/validation.rs"]
mod validation;
#[path = "tests/structure.rs"]
mod structure;
```

## Grouping (two leading-prefix families + a catch-all)

The mover's native `name.startswith(prefix-tuple)` mode handles the two specific
families; `structure` is the `*` catch-all (must be last) capturing the two
remaining lowering tests. This mirrors kali_mir's `lower_tests.rs` split
(`flavor_metadata` family + `structure=*`).

**move_fns.py groups-spec:**
```
flavor_metadata=test_lir_lowering_preserves_function_flavor_metadata;validation=test_lir_validation_;structure=*
```

| submodule | n | prefixes |
|-----------|--:|----------|
| `flavor_metadata` | 8 | `test_lir_lowering_preserves_function_flavor_metadata` |
| `validation` | 1 | `test_lir_validation_` |
| `structure` | 2 | `*` (catch-all) |

Exact membership (the decisive multiset; 8+1+2 = 11):

- **flavor_metadata** — `test_lir_lowering_preserves_function_flavor_metadata`, `test_lir_lowering_preserves_function_flavor_metadata_for_default_export_generator_function_declaration`, `test_lir_lowering_preserves_function_flavor_metadata_for_default_export_anonymous_generator_function_declaration`, `test_lir_lowering_preserves_function_flavor_metadata_for_default_export_async_generator_function_declaration`, `test_lir_lowering_preserves_function_flavor_metadata_for_class_methods`, `test_lir_lowering_preserves_function_flavor_metadata_for_class_expressions`, `test_lir_lowering_preserves_function_flavor_metadata_for_default_export_class_expressions`, `test_lir_lowering_preserves_function_flavor_metadata_for_default_export_class_declarations`
- **validation** — `test_lir_validation_rejects_out_of_bounds_children`
- **structure** — `test_lir_lowering_preserves_root`, `test_lir_lowering_preserves_child_order_and_text_payloads`

Disjointness note: the `flavor_metadata` prefix does not match the two
`structure` names (`..._preserves_root` / `..._preserves_child_order...`), and
`validation` is matched before the catch-all, so the partition is unambiguous.

## Method

Pure verbatim code-motion via the series' reusable tools (git-ignored scratch
under `.superpowers/sdd/`):

- **Task 0** establishes the toolchain for this entry: confirm `move_fns.py` is in
  leading-prefix mode (`group_for` uses `fn_name.startswith(prefs)`) and the
  matching `verify.py`; keep `FN_RE` / `IDENT_CHARS` / `find_close_line`
  byte-identical. Capture the baseline `--list` test-name set.
- **Single mover invocation** for all three groups, then build+test gate.
  Implementer = haiku/sonnet (pure command-transcription); review = sonnet.
- Bodies move **verbatim** — no `cargo fmt`, no path rewrites, no `pub`-widening,
  no `include_*!` changes. Nested helper fns inside a test body travel with their
  parent test.

## Invariants / gates (literal — this crate's baseline is clean)

1. `cargo build -p kali_lir --tests` → **0 warnings** (held at baseline and on merged main).
2. `cargo test -p kali_lir --lib` → **11 passed; 0 failed** (held throughout).
3. Facade `src/tests.rs` drains to **0 module-level `#[test]` fns** (1 helper retained).
4. `verify.py` proves `{name: body}` extracted from `src/tests.rs@base` ==
   union of the three submodules — 11/11 bodies byte-identical, disjoint namespaces,
   0 collisions, net new lines = scaffold only (`use super::*;` + blank per file +
   3 `#[path]`/`mod` pairs).
5. `lib.rs:16–21` decl unchanged; no production `.rs` file touched; `kali_lir`
   consumers compile unedited.

## Out of scope

- `kali_embed/src/tests.rs` (20 tests) — the remaining co-located `tests.rs`
  monolith, a future series entry.
- Any production-`src` refactor (kali_lir production was the 20th entry).

## Completion

Per-task review + opus whole-branch review → 0 findings; re-verify on merged
`main`; ff-merge to local `main`; delete branch; **do not push origin**. Update the
`crate-modularization-series` memory with the 36th entry.
