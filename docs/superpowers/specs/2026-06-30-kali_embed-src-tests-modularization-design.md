# kali_embed `src/tests.rs` Unit-Test Modularization (Series Entry 37)

**Date:** 2026-06-30
**Crate:** `kali_embed`
**Branch:** `refactor/kali_embed-src-tests-modularization` (off local `main`)
**Integration:** local-`main` ff-merge only — **never push to origin** (origin/main intentionally lags).

## Goal

Decompose the co-located unit-test monolith `crates/kali_embed/src/tests.rs`
(20 `#[test]` fns, 571 lines) into a thin **drained facade** plus four
per-concern `#[path] mod` submodules under `src/tests/`. Zero behavior change,
byte-identical test bodies, public API untouched, consumers compile unedited.

This is the 37th entry in the crate-modularization series and follows the exact
verbatim-code-motion recipe used for the prior entries (kali_optimize,
kali_types, kali_runtime, kali_codegen, kali_common, kali_mir, kali_api_web,
kali_sandbox, kali_lir, …). It uses a co-located `tests.rs`-named file declared
from `lib.rs` with a drained facade that **retains one non-test helper** — the
same shape as the 35th entry (kali_sandbox). **This is the last co-located
`tests.rs`-named monolith on the series frontier.**

## Baseline (verified)

- Declared from `lib.rs:24–26`: `#[cfg(test)]` / `#[path = "tests.rs"]` / `mod tests;` — **untouched** by this work.
- `cargo build -p kali_embed --tests` → **0 warnings**.
- `cargo test -p kali_embed --lib` → **20 passed; 0 failed** (all 20 lib tests live in `tests.rs`; no other `#[test]` in the crate).
- **0 `include_*!` pins** in `tests.rs` (no facade pinning needed).
- 1 module-level helper fn is **not** `#[test]` and stays in the facade:
  `permissive_policy` (lines 9–48, returns `kali_sandbox::SandboxPolicy`). It is
  used only by 4 of the `embedding_predicates_*` tests. The mover leaves
  non-`#[test]` fns in place automatically; children reach it via `use super::*;`
  (Rust descendant-visibility re-exports the facade's private items through the
  child glob — proven at 0 warnings across prior entries).
- Facade currently retains these `use` lines (kept verbatim): `use super::*;`,
  `use crate::compiler::temporary_source_path;`,
  `use std::{fs, sync::{Arc, Mutex}};`, `use tempfile::tempdir;`.

## Architecture

```
crates/kali_embed/src/
  lib.rs                  # lines 24–26 decl UNCHANGED
  tests.rs                # FACADE: 4 use-lines + 1 helper + 4 `#[path] mod` decls, 0 #[test]
  tests/
    compiler.rs           # use super::*; + 5 verbatim #[test] fns
    runtime_profiles.rs   # use super::*; + 3 verbatim #[test] fns
    context.rs            # use super::*; + 6 verbatim #[test] fns
    predicates.rs         # use super::*; + 6 verbatim #[test] fns
```

The facade drains to **0 module-level `#[test]` fns** (helper `permissive_policy`
retained) and appends:

```rust
#[path = "tests/compiler.rs"]
mod compiler;
#[path = "tests/runtime_profiles.rs"]
mod runtime_profiles;
#[path = "tests/context.rs"]
mod context;
#[path = "tests/predicates.rs"]
mod predicates;
```

## Grouping (four leading-prefix families, no catch-all)

The mover's native `name.startswith(prefix-tuple)` mode handles all four families;
every one of the 20 tests is covered by an explicit prefix (no `*` catch-all).

**move_fns.py groups-spec:**
```
compiler=compiles_,compile_lib_,temporary_source_paths_;runtime_profiles=compiler_rejects_;context=embedding_context_,embedding_layer_,embedding_operation_context_;predicates=embedding_predicates_,embedding_predicate_registration_
```

| submodule | n | prefixes |
|-----------|--:|----------|
| `compiler` | 5 | `compiles_`, `compile_lib_`, `temporary_source_paths_` |
| `runtime_profiles` | 3 | `compiler_rejects_` |
| `context` | 6 | `embedding_context_`, `embedding_layer_`, `embedding_operation_context_` |
| `predicates` | 6 | `embedding_predicates_`, `embedding_predicate_registration_` |

Exact membership (the decisive multiset; 5+3+6+6 = 20):

- **compiler** (5) — `compiles_standalone_artifacts_in_memory`, `compiles_library_artifacts_with_wit_sidecars`, `compile_lib_reports_missing_export_surfaces`, `compile_lib_from_raw_source_uses_a_stable_module_name`, `temporary_source_paths_are_unique_across_calls`
- **runtime_profiles** (3) — `compiler_rejects_threaded_runtime_profiles_in_the_current_phase`, `compiler_rejects_duplicate_runtime_profiles_before_phase_gating`, `compiler_rejects_unknown_runtime_profiles_before_phase_gating`
- **context** (6) — `embedding_context_uses_the_stable_compiler_api` (`embedding_context_`, 1), `embedding_layer_reexports_the_host_predicate_context` (`embedding_layer_`, 1), and `embedding_operation_context_{uses_process_spawn_resource_alias_and_details, carries_file_network_and_env_details, carries_remaining_host_specific_details, uses_the_resource_alias_and_details_for_threads}` (`embedding_operation_context_`, 4)
- **predicates** (6) — `embedding_predicates_{can_deny_with_a_host_specific_reason, do_not_override_declarative_denials, can_inspect_thread_budget_context_details, run_in_registration_order}` (`embedding_predicates_`, 4), `embedding_predicate_registration_{availability_can_be_queried, rejects_when_disabled}` (`embedding_predicate_registration_`, 2)

Disjointness notes (leading-prefix partition is unambiguous):
- `embedding_context_` (1 test) is **not** a prefix of `embedding_operation_context_`
  (the latter's 9th char is `o`/`p`, not `c`) — distinct families.
- `embedding_predicates_` (4 tests) is **not** a prefix of
  `embedding_predicate_registration_` (after `predicate` the former has `s`, the
  latter `_`) — distinct families.
- No `compiler_rejects_*` name starts with `compile_lib_` or `compiles_`.

## Method

Pure verbatim code-motion via the series' reusable tools (git-ignored scratch
under `.superpowers/sdd/`):

- **Task 0** establishes the toolchain for this entry: confirm `move_fns.py` is in
  leading-prefix mode (`group_for` uses `fn_name.startswith(prefs)`; re-install the
  prefix variant if a prior crate left the exact-name variant) and the matching
  `verify.py`; keep `FN_RE` / `IDENT_CHARS` / `find_close_line` byte-identical.
  Capture the baseline `--list` test-name set.
- **Single mover invocation** for all four groups, then build+test gate.
  Implementer = haiku/sonnet (pure command-transcription); review = sonnet.
- Bodies move **verbatim** — no `cargo fmt`, no path rewrites, no `pub`-widening,
  no `include_*!` changes. Nested helper fns inside a test body travel with their
  parent test.

## Invariants / gates (literal — this crate's baseline is clean)

1. `cargo build -p kali_embed --tests` → **0 warnings** (held at baseline and on merged main).
2. `cargo test -p kali_embed --lib` → **20 passed; 0 failed** (held throughout).
3. Facade `src/tests.rs` drains to **0 module-level `#[test]` fns** (1 helper `permissive_policy` retained).
4. `verify.py` proves `{name: body}` extracted from `src/tests.rs@base` ==
   union of the four submodules — 20/20 bodies byte-identical, disjoint namespaces,
   0 collisions, net new lines = scaffold only (`use super::*;` + blank per file +
   4 `#[path]`/`mod` pairs).
5. `lib.rs:24–26` decl unchanged; no production `.rs` file touched; `kali_embed`
   consumers (e.g. anything depending on the crate) compile unedited.
6. `cargo fmt -p kali_embed --check` clean (baseline is clean; verbatim moves
   preserve formatting).

## Out of scope

- Any production-`src` refactor of `kali_embed` (this entry is unit-tests only).
- Other crates' co-located `src/*_tests.rs` monoliths are already done; the tiny
  single-concern `tests.rs` files (kali_error 2, kali_fmt 2, kali_cli 8) are kept
  whole. With `kali_embed` done, the co-located `tests.rs`-named monolith frontier
  is **exhausted**.

## Completion

Per-task review + opus whole-branch review → 0 findings; re-verify on merged
`main`; ff-merge to local `main`; delete branch; **do not push origin**. Update the
`crate-modularization-series` memory with the 37th entry.
