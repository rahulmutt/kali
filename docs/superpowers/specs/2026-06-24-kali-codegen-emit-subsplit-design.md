# Kali Codegen `emit/` Sub-split — Design

**Date:** 2026-06-24
**Status:** Approved (design)
**Scope:** Complete the deferred follow-up from the `kali_codegen` modularization: split the one remaining large file, `crates/kali_codegen/src/emit/mod.rs`, into focused method-modules with co-located tests.

## Problem

The `kali_codegen` modularization (merged) left one outstanding large file. The
plan deliberately grouped all core-emission methods into a single
`impl<'a> FunctionEmitter<'a>` block in `emit/mod.rs`:

- `crates/kali_codegen/src/emit/mod.rs` — **4,030 lines**, one `impl` block of
  ~25 methods (`use crate::*;` at top, `#[path] mod emit_tests;` at bottom).
- `crates/kali_codegen/src/emit/emit_tests.rs` — **2,969 lines, 124 `#[test]`s**,
  a flat sibling file.

`emit_call` alone spans lines **1167–3326 (~2,160 lines)** — a single method that
dispatches over every supported builtin/method call family. It is the dominant
mass of the file.

This is the same shape every other file in the crate has already been reduced
from. The goal is to finish the job: break `emit/mod.rs` into small,
single-purpose method-modules and co-locate their tests, following the exact
pattern used for the rest of the crate.

## Goal & Hard Constraints

This is a **pure structural refactor — zero behavior change.**

- The exact same set of tests exists and passes before and after.
- `emit/mod.rs` becomes a thin **facade** for the `emit/` directory module
  (`use crate::*;`, `mod` declarations for the new files, and the `#[cfg(test)]`
  test wiring). No `impl` logic remains in it.
- Each new file carries its own `impl<'a> FunctionEmitter<'a>` block — legal
  within one crate (the same method-only-directory-module technique used for
  `intrinsics/*`). Directory-module files use `use crate::*;` (not `super`).
- Extraction is **text-movement only**: method bodies move verbatim. No method
  body is rewritten, no helper is extracted, no call graph changes.
- Public/`pub(crate)` surface is untouched: `emit_call` and siblings stay
  `pub(crate)`; `FunctionEmitter` and the 5 `Static*` enums stay `pub(crate)`.

### Explicit non-goal: `emit_call` stays intact

`emit_call` remains a single ~2,160-line method, moved verbatim into `call.rs`.
`call.rs` will therefore stay large (~2,300 lines). Cracking `emit_call` into
per-family helper methods would shrink it, but that is a **behavior-preserving
logic refactor (changes the call graph), not text movement** — it breaks this
effort's core invariant and carries real regression risk. It is deferred as a
separate, optional future pass with its own spec, not smuggled into this one.

### Proof obligation

Capture a baseline before touching code and compare after:

- `cargo test -p kali_codegen --list` yields the **same set of test names**
  (modulo module-path prefix) before and after. Baselines recorded under
  `docs/superpowers/baselines/` (`kali_codegen-emit-tests-before.txt`,
  `kali_codegen-emit-tests-after.txt`, and a `…-renames.md` mapping for the test
  paths that shift from `emit_tests::` to the new module prefixes).
- Green at every commit: `cargo build -p kali_codegen`, `cargo test -p
  kali_codegen`, and `cargo clippy -p kali_codegen -- -D warnings` all pass.
- WASM validity is preserved — tests already gate on `wasmparser::Validator`.
- Final gate: full-workspace `cargo build` + `cargo test` confirm no downstream
  breakage.

## Architecture

`FunctionEmitter<'a>` keeps its definition, fields, and API unchanged. The ~25
methods currently in `emit/mod.rs`'s single `impl` block are partitioned into
four per-domain `impl<'a> FunctionEmitter<'a>` blocks across focused files.

### Target layout

```
crates/kali_codegen/src/emit/
  mod.rs            # facade: use crate::*; mod call/operators/literal/control_flow;
                    #   #[cfg(test)] #[path] mod wiring. No impl logic.
  control_flow.rs   # node dispatch + structural emission:
                    #   emit_node, emit_value, emit_sequence, emit_function_body,
                    #   emit_break_or_continue, emit_branch,
                    #   for_of_binding_name, for_of_binding_name_from_node
  operators.rs      # operator / arithmetic emission:
                    #   emit_unary, emit_binary, emit_update_expression,
                    #   emit_exponentiation_expression, perfect_square_root_i128
  literal.rs        # literal + assignment emission:
                    #   emit_aggregate_literal, resolve_literal_aggregate,
                    #   assignment_target_name, emit_assignment
  call.rs           # call emission (dominated by emit_call):
                    #   emit_call, resolve_static_index_member, static_member_index,
                    #   resolve_static_reference_root_name, unwrap_transparent_value_node,
                    #   is_supported_callable_reference, resolve_bound_node,
                    #   resolve_bound_member_callable_node, resolve_transparent_callable_node
```

Approximate sizes after the split: `control_flow.rs` ~640, `operators.rs` ~560,
`literal.rs` ~360, `call.rs` ~2,300, `mod.rs` ~15 (facade). A method whose domain
is ambiguous goes to the module its primary caller/sibling lives in; when still
unclear, defer to plan-phase review rather than guessing.

### Components & boundaries

- **`mod.rs`** — directory-module facade only: declares the four method-files and
  wires their test siblings. Holds no `impl` block.
- **`control_flow.rs`** — the traversal hub: `emit_node` dispatches by LIR node
  kind into the other files' methods; owns sequence/body/branch/break-continue
  and for-of binding resolution.
- **`operators.rs`** — unary/binary/update/exponentiation emission and the
  `perfect_square_root_i128` arithmetic helper they rely on.
- **`literal.rs`** — aggregate-literal emission, literal-aggregate resolution, and
  assignment emission/target-name resolution.
- **`call.rs`** — `emit_call` plus the callable/static-reference resolution
  helpers it alone consumes.

These boundaries match how `emit_node` already fans out, so no new cross-file
coupling is introduced beyond the shared `FunctionEmitter`/`CodegenCtx` surface
that already exists.

## Data flow

Unchanged by this refactor. `emit_node` (in `control_flow.rs`) dispatches each
LIR node to the appropriate emission method (`operators.rs` / `literal.rs` /
`call.rs`), which emit `wasm_encoder` instructions into the `CodegenResult`. The
refactor only relocates methods that already implement this flow.

## Testing

### Co-location

Split the flat `emit_tests.rs` (2,969 lines, 124 tests) into four sibling
`*_tests.rs`, one per new source file, each wired at the bottom of its module:

```rust
#[cfg(test)]
#[path = "call_tests.rs"]
mod call_tests;
```

Test-name clusters map onto the modules by what they exercise:

| Source file        | `*_tests.rs`            | Test cluster                                                |
|--------------------|-------------------------|-------------------------------------------------------------|
| `control_flow.rs`  | `control_flow_tests.rs` | `function_plans_*`, `for_of_*`, `for_await_*`, branch/sequence |
| `operators.rs`     | `operators_tests.rs`    | `bitwise_*`, unary/binary, update, exponentiation           |
| `literal.rs`       | `literal_tests.rs`      | aggregate-literal / assignment tests                        |
| `call.rs`          | `call_tests.rs`         | call-emission / intrinsic-dispatch tests                    |

Directory-module test files use `use crate::*;`. Each test moves verbatim to the
file whose method it exercises; assertions are unchanged. The net
`cargo test --list` name set is identical to the baseline (only module-path
prefixes change, captured in the renames mapping).

### Support

No new test support is introduced. The crate-local `test_support` (LIR builder +
macros) and the `kali_test_support` dev-dep already exist from the prior refactor;
the moved tests keep using them via `use crate::*;`. Shared helpers stay where
they are.

## Error handling

No change. Diagnostic/fallback paths move with their methods; behavior is
preserved and asserted by the unchanged tests.

## Sequencing & commit strategy

Incremental, green-at-every-commit, on a feature branch off `main`. Commit
messages follow the existing style (`refactor(kali_codegen): … [refactor]`,
`test(kali_codegen): … [refactor]`, `style(kali_codegen): … [refactor]`).

1. **Baseline** — record `cargo test -p kali_codegen --list` + counts under
   `docs/superpowers/baselines/` (`…-emit-tests-before.txt`). (no code change)
2. **Extract `control_flow.rs`** — move its methods verbatim out of
   `emit/mod.rs` into a new file with its own `impl` block; add the `mod` line.
3. **Extract `operators.rs`.**
4. **Extract `literal.rs`.**
5. **Extract `call.rs`** — move `emit_call` + resolvers; `emit/mod.rs` is now a
   facade (no `impl` block left).
6. **`cargo fmt`** normalization — its own commit.
7. **Co-locate tests** — split `emit_tests.rs` into the four `*_tests.rs`, one
   commit per file; delete the monolith last and re-wire `#[path]` lines.
8. **Post-refactor baseline** (`…-emit-tests-after.txt` + renames mapping) +
   final workspace-wide build/test/clippy.

**Per-step verification:** `cargo build -p kali_codegen` → `cargo test -p
kali_codegen` → `cargo clippy -p kali_codegen -- -D warnings`. Final step also
runs the full-workspace build + test.

## Out of scope

- Any behavior, output, or public-API change.
- Cracking `emit_call` into per-family helper methods (separate future pass).
- Refactoring other crates — each is its own spec → plan → implementation cycle.
- Rewriting method internals, renaming public items, restructuring LIR input.

## References

- Parent design: `docs/superpowers/specs/2026-06-24-kali-codegen-modularization-design.md`
- Parent plan: `docs/superpowers/plans/2026-06-24-kali-codegen-modularization.md`
- Pilot design: `docs/superpowers/specs/2026-06-23-kali-crate-modularization-design.md`
- Memory: `kali-crate-modularization`
