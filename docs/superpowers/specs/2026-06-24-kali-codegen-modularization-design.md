# Kali Codegen Modularization — Design

**Date:** 2026-06-24
**Status:** Approved (design)
**Scope:** Apply the `kali_types` pilot pattern to `kali_codegen` (full parity: source split + test co-location + shared/local test support).

## Problem

`kali_codegen` is the largest remaining logic monolith in the workspace:

- `crates/kali_codegen/src/lib.rs` — **10,331 lines**, essentially one
  `impl<'a> FunctionEmitter<'a>` block (lines 276–10331, ~201 methods) plus a
  handful of small support types.
- `crates/kali_codegen/src/tests.rs` — **8,024 lines, 324 `#[test]`s, zero
  submodules** (a flat file), building `LirProgram` trees by hand through a
  verbose local `node()` helper.

This is the same shape `kali_types` had before its (now merged) pilot refactor.
Large files are hard to navigate, review, and hold in context. The goal is to
break the crate into small, single-purpose modules and co-locate tests with the
code they exercise, factoring shared setup into helpers and macros.

## Goal & Hard Constraints

This is a **pure structural refactor — zero behavior change.**

- The exact same set of tests exists and passes before and after.
- `lib.rs` becomes a thin **facade** (module declarations, the crate import
  surface, import-index `const`s, and `pub use` re-exports) so every external
  path (e.g. `kali_codegen::CodegenCtx`) keeps resolving. No public API churn —
  downstream crates (`kali_cli`, `kali_runtime`, …) compile untouched.
- Conform to the established repo convention: unit tests live in **sibling
  `*_tests.rs` files wired via `#[cfg(test)] #[path = "…"] mod …`**, not inline
  `#[cfg(test)]` modules.
- Extraction is **text-movement only**: cross-referenced items/fields/methods
  are widened to `pub(crate)` first, then method bodies are moved verbatim into
  per-domain `impl<'a> FunctionEmitter<'a>` blocks. No method body is rewritten.

### Proof obligation

Capture a baseline before touching code and compare after:

- `cargo test -p kali_codegen --list` yields the **same set of test names**
  (modulo module-path prefix) before and after. Baselines recorded under
  `docs/superpowers/baselines/` (`kali_codegen-tests-before.txt`,
  `kali_codegen-tests-after.txt`, and a `…-renames.md` mapping if any test
  paths shift).
- Green at every commit: `cargo build -p kali_codegen`, `cargo test -p
  kali_codegen`, and `cargo clippy -p kali_codegen -- -D warnings` all pass.
- WASM validity is preserved — tests already gate on `wasmparser::Validator`.
- Final gate: full-workspace `cargo build` + `cargo test` confirm no downstream
  breakage.

## Architecture

The central type `FunctionEmitter<'a>` and the `CodegenCtx` keep their
definitions, fields, and public API unchanged. The ~201 methods are partitioned
into per-domain `impl<'a> FunctionEmitter<'a>` blocks across focused modules.

### Target layout

```
crates/kali_codegen/src/
  lib.rs                  # facade: mod decls, imports, import-index consts,
                          #   pub use re-exports, #[cfg(test)] wiring. No logic.
  ctx.rs                  # CodegenCtx, TargetConfig (+Default), CodegenResult,
                          #   StringPool, static-result enums + their small impls
  emitter.rs              # FunctionEmitter struct def + lifecycle: new, node,
                          #   alloc_scratch_node, control-frame stack;
                          #   FunctionPlan/LoopFrame/EmittedValue/ValueShape
  emit/                   # core emission
    mod.rs                #   emit_function_body, emit_sequence, emit_node,
                          #   emit_value, emit_binary/unary/call/assignment,
                          #   emit_branch, emit_break_or_continue,
                          #   emit_for_of_array_iteration, aggregate literals
  intrinsics/
    string.rs             # resolve_static_string_*, is_string_*_call,
                          #   string_*_call_method (~40 methods)
    array.rs              # resolve_static_array_*, is_array_*, static_array_*,
                          #   callback-iteration helpers
    math.rs              # math_* import indices + constant folding
                          #   (sqrt/cbrt/log/trig/hyperbolic/pow/round/…)
    number.rs            # parse_int/parse_float, global number predicates,
                          #   bigint/uint32 literal eval, numeric-literal checks
    object.rs           # is_object_*, static_object_*, freeze / has_own /
                          #   from_entries / enumeration
    host.rs             # env / cwd / process / deno / console / semver /
                          #   package_json / coverage / kali_test
    collections.rs      # set & map constructor recognition + iteration
```

Exact method-to-module assignment for all ~201 methods is settled in the
implementation-plan phase; this design fixes the **module boundaries and
names**. A method whose domain is ambiguous goes to the module its primary
caller/sibling lives in; when still unclear, defer to plan-phase review rather
than guessing.

### Components & boundaries

- **`ctx.rs`** — owns the codegen context and value/result vocabulary. Depends
  on nothing in the emit/intrinsics layers.
- **`emitter.rs`** — owns the emitter's identity and scratch/control-frame
  bookkeeping. The struct all the `impl` blocks attach to.
- **`emit/`** — drives traversal/emission; calls into `intrinsics/` for static
  recognition + constant folding.
- **`intrinsics/*`** — each module is one JS/host surface area: pure recognizers
  (`is_*`) and constant-folders (`resolve_static_*`). No cross-intrinsic
  dependencies beyond shared helpers on `FunctionEmitter`/`CodegenCtx`.

## Data flow

Unchanged by this refactor. For reference: `LirProgram` → `FunctionEmitter`
walks LIR nodes (`emit/`), consulting `intrinsics/*` to recognize and statically
fold supported call shapes, emitting `wasm_encoder` instructions into a
`CodegenResult`. The refactor only relocates the methods that already implement
this flow.

## Testing

### Co-location

Split the flat `tests.rs` (324 tests) into sibling `*_tests.rs`, one per source
module, each wired at the bottom of its module:

```rust
#[cfg(test)]
#[path = "string_tests.rs"]
mod string_tests;
```

Test-name clusters already map onto the modules:

| Source module            | `*_tests.rs`              | Source cluster (test-name prefix)        |
|--------------------------|---------------------------|------------------------------------------|
| `intrinsics/math.rs`     | `math_tests.rs`           | `math_*`                                 |
| `intrinsics/object.rs`   | `object_tests.rs`         | `object_*`                               |
| `intrinsics/host.rs`     | `host_tests.rs`           | `process_*`, `deno_*`, `console_*`       |
| `intrinsics/collections.rs` | `collections_tests.rs` | `set_*`, `map_*`                         |
| `intrinsics/string.rs`   | `string_tests.rs`         | string intrinsic tests                   |
| `intrinsics/array.rs`    | `array_tests.rs`          | array intrinsic tests                    |
| `intrinsics/number.rs`   | `number_tests.rs`         | `number_*`, parse/global predicate tests |
| `emit/`                  | `emit_tests.rs`           | `supported_*`, `unsupported_*`, `for_*`, update/logical/nullish/bitwise |

Directory-module test files use `use crate::*;` (not `super`). The net
`cargo test --list` name set is identical to the baseline.

### Shared & local support

- **`kali_test_support`** (existing dev-crate): added to `kali_codegen`'s
  `[dev-dependencies]`; adopt its cross-crate fixtures
  (`fixtures::tempdir/write_file/write_manifest`) where codegen tests touch the
  filesystem or manifests.
- **Codegen-local `test_support` module**: introduce a small **LIR builder +
  macro** (the codegen analog of the pilot's per-crate AST builders) to replace
  the repetitive `LirNodeId`/`push` bookkeeping in `node()`-style construction.
  Move shared helpers (`sample_program`, `compile_and_measure`,
  `wasm_instruction_count`, `assert_*_lowers`) here so every `*_tests.rs` shares
  them.
- The builder is **opt-in and behavior-preserving**: tests migrate to it only
  where it reduces noise, and the LIR they produce must be byte-identical
  (assertions unchanged).

## Error handling

No change. Diagnostics (`kali_error`) and placeholder/fallback paths
(`push_placeholder_fallback_diagnostic`, `generator_function_yield_lowering_
unavailable_message`) move with their methods; their behavior is preserved and
asserted by the unchanged tests.

## Sequencing & commit strategy

Incremental, green-at-every-commit, mirroring the pilot's rhythm. Work lands on
a feature branch off `main`. Commit messages follow the existing style
(`refactor(kali_codegen): …`, `test(kali_codegen): … [refactor]`,
`style(kali_codegen): …`).

1. **Baseline** — record `cargo test --list` + counts under
   `docs/superpowers/baselines/`. (no code change)
2. **Widen visibility** — flip cross-referenced items/fields/methods to
   `pub(crate)` while still a single `lib.rs`.
3. **Extract scaffolding** — `ctx.rs`, then `emitter.rs` (text-movement); facade
   `lib.rs` grows its `mod`/`pub use` lines.
4. **Extract `emit/`** core.
5. **Extract `intrinsics/`** one domain per commit:
   `string` → `array` → `math` → `number` → `object` → `host` → `collections`.
6. **`cargo fmt`** normalization — its own commit.
7. **Test support** — introduce codegen-local `test_support` (LIR builder/macro
   + moved helpers); wire `kali_test_support` dev-dep.
8. **Co-locate tests** — split `tests.rs` into `*_tests.rs` per module, one
   commit per domain; delete the monolith last.
9. **Adopt builders/fixtures** in migrated tests where they cut noise.
10. **Post-refactor baseline** + final workspace-wide build/test/clippy.

**Per-step verification:** `cargo build -p kali_codegen` → `cargo test -p
kali_codegen` → `cargo clippy -p kali_codegen -- -D warnings`. Final step also
runs the full-workspace build + test.

## Out of scope

- Any behavior, output, or public-API change.
- Refactoring other crates (`kali_cli`, `kali_optimize`, `kali_runtime`, …) —
  each is its own spec → plan → implementation cycle.
- Rewriting method internals, renaming public items, or restructuring the LIR
  input format.

## References

- Pilot design: `docs/superpowers/specs/2026-06-23-kali-crate-modularization-design.md`
- Pilot plan: `docs/superpowers/plans/2026-06-23-kali-types-modularization.md`
- Pilot baselines: `docs/superpowers/baselines/kali_types-tests-*`
- Memory: `kali-crate-modularization`
