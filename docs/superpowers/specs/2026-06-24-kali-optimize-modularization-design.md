# Kali Optimize Modularization — Design

**Date:** 2026-06-24
**Status:** Approved (design)
**Scope:** Apply the established crate-modularization pattern (facade + co-located
tests + shared/local test support) to `kali_optimize`. Pure text-movement, zero
behavior change.

## Problem

`kali_optimize` is the next-largest remaining monolith in the workspace (the 4th
crate in the modularization effort, after `kali_types`, `kali_codegen`, and
`kali_runtime`):

- `crates/kali_optimize/src/lib.rs` — **2,872 lines / ~101 KB**. One central
  `impl Optimizer` block (lines ~65–2358, **71 methods**) holding the entire
  optimization pipeline — constant folding, algebraic identities, MIR-driven
  layout specialization, inlining/dead-code pruning, and object folding — followed
  by a cluster of `pub(crate)` structs/enums (`MirLayoutClass`,
  `MirLayoutSignature`, `MirSpecializationPlan`, `SpecializationPlan`,
  `BindingEnv`, `FunctionSummary`, `ConstantValue`, `SpecializationTracker`) and
  13 free fold/parse/literal functions.
- `crates/kali_optimize/src/tests.rs` — **8,597 lines, 110 `#[test]`s, flat (no
  submodules)**.
- `crates/kali_optimize/src/profile.rs` — already split out (kept as-is).

Unlike `kali_runtime`, this crate has **no single mega-function**: the largest
methods are ~150–175 lines (`optimize_call_site`, `optimize_algebraic_identity`),
all comfortably moved intact. It also has **zero filesystem/tempdir tests** — the
same shape as `kali_codegen`.

This is the same monolith shape the three completed crates had before their (now
merged) refactors. Large single files are hard to navigate, review, and hold in
context. The goal: break the central `impl Optimizer` into small, single-purpose
modules and co-locate tests with the code they exercise.

## Goal & Hard Constraints

This is a **pure structural refactor — zero behavior change.**

- The same set of tests passes before and after (see the one deliberate +1
  deviation under Testing).
- `lib.rs` becomes a thin **facade** (module declarations, the crate import
  surface, the `HOT_FUNCTION_MINIMUM_WEIGHT` const, and `pub use` re-exports) so
  every external path (e.g. `kali_optimize::Optimizer`) keeps resolving. No public
  API churn — downstream crates compile untouched.
- Unit tests live in **sibling `*_tests.rs` files wired via
  `#[cfg(test)] #[path = "…"] mod …`**, not inline `#[cfg(test)]` modules.
- Extraction is **text-movement only**: cross-referenced
  items/fields/methods/functions are widened to `pub(crate)` first, then bodies
  are moved verbatim. All methods move intact (no function cracking — see Out of
  scope).

### Public API — preserved exactly

Public surface stays exactly: `OptimizationLevel`, `OptimizationReport`,
`Optimizer`, plus the existing `pub use profile::{ProfileData, ProfileSample,
ProfileSampleKind, PROFILE_DATA_VERSION}`. Everything else (all the structs/enums
and free fns listed above) stays `pub(crate)`, widened up front from private to
enable pure text-movement across module boundaries.

### Proof obligation

Capture a baseline before touching code and compare after:

- `cargo test -p kali_optimize -- --list` yields the **same set of test
  basenames** before and after (modulo the one added fixture test — see Testing).
  Note: `--list` *includes* module-path prefixes, so co-location makes a raw diff
  non-empty by design — prove the invariant by stripping prefixes and comparing
  **basenames**. Baselines recorded under `docs/superpowers/baselines/`
  (`kali_optimize-tests-before.txt`, `kali_optimize-tests-after.txt`, and a
  `…-renames.md` mapping).
- Green at every commit: `cargo build -p kali_optimize`, `cargo test -p
  kali_optimize`, and `cargo clippy -p kali_optimize --all-targets -- -D warnings`
  all pass.
- Final gate: full-workspace `cargo build` + `cargo test` confirm no downstream
  breakage.

## Architecture

`Optimizer`, `OptimizationLevel`, `OptimizationReport`, and all current items keep
their definitions, fields, and (public) API unchanged. The one `impl Optimizer`
block splits into separate `impl Optimizer` blocks across modules (legal within
one crate, same as `kali_codegen`/`kali_runtime`); free functions and the
supporting structs/enums move as `pub(crate)` items into the module whose
responsibility they serve.

### Target layout

```
crates/kali_optimize/src/
  lib.rs            # facade: mod decls, imports, HOT_FUNCTION_MINIMUM_WEIGHT const,
                    #   pub use re-exports, #[cfg(test)] wiring. No logic.
  profile.rs        # unchanged (already split)
  driver.rs         # impl Optimizer: public ctors/accessors (new, with_*,
                    #   max_specializations, profile_data, optimization_report,
                    #   optimize_program*, optimize_program_internal) + the
                    #   recursion core (optimize_node, optimize_sequence,
                    #   is_cse_candidate); OptimizationLevel, OptimizationReport
  constant_fold.rs  # impl Optimizer: optimize_constant_expression,
                    #   optimize_algebraic_identity; ConstantValue (+impl) and the
                    #   free fold/parse/literal fns (fold_unary, fold_binary,
                    #   is_zero/one_constant, literal_value, parse_*_literal,
                    #   literal_text, *_signature helpers)
  specialize.rs     # impl Optimizer: specialize_layout_bindings,
                    #   specialize_mir_call_site(s), clone_specialized_function,
                    #   argument_has_concrete_*, *_specialization_plan,
                    #   specialization_signature*; MirLayoutClass,
                    #   MirLayoutSignature, MirSpecializationPlan,
                    #   SpecializationPlan, SpecializationTracker
  inline.rs         # impl Optimizer: inline_call_site, clone_subtree_with_*,
                    #   prune_dead_top_level_functions, function_summary,
                    #   extract_inline_body, count_subtree_nodes,
                    #   contains/collect_call_target(s), inline_threshold_for_*,
                    #   is_hot_function, profile_has_hot_branch_or_layout_hints;
                    #   FunctionSummary
  object_fold.rs    # impl Optimizer: fold_object_* (has_own / enumeration /
                    #   from_entries / enumeration_calls),
                    #   ordered_object_literal_properties, resolve_constant_binding,
                    #   is_object_freeze_call, collect_constant_bindings(_into);
                    #   BindingEnv
  layout.rs         # impl Optimizer: fold_layout_member_access,
                    #   object_literal_field, array_literal_element/length,
                    #   constant_array_index, is_object/array_literal
  helpers.rs        # impl Optimizer: literal/clone/push helpers
                    #   (clone_boolean_literal, clone_string_literal,
                    #   push_array/object_literal), member_access_name +
                    #   normalize/canonicalize variants, constant_property_key,
                    #   object_property_order_key, call_signature, node_signature
```

Exact method-to-module assignment is finalized in the implementation-plan phase;
this design fixes the **module boundaries and names**. A method whose domain is
ambiguous goes to the module its primary caller/sibling lives in; when still
unclear, defer to plan-phase review rather than guessing. The `layout.rs` /
`helpers.rs` boundary in particular is finalized during the split — the
responsibility split (layout-aware folding vs. generic node/literal construction)
is what matters.

### Components & boundaries

- **`driver.rs`** — the public entry surface and the recursive walk that dispatches
  to every other optimization module. Owns `OptimizationLevel` /
  `OptimizationReport`.
- **`constant_fold.rs`** — constant expression evaluation + algebraic identities,
  plus the `ConstantValue` representation and its parse/fold primitives.
- **`specialize.rs`** — MIR-layout-driven call-site specialization and its plan
  vocabulary.
- **`inline.rs`** — inlining decisions, subtree cloning/substitution, and
  dead-function pruning, including profile-driven hotness thresholds.
- **`object_fold.rs`** — compile-time folding of `Object.*` operations and the
  constant-binding environment they consult.
- **`layout.rs`** — folding member/index access against known object/array
  literals and layouts.
- **`helpers.rs`** — shared LIR-node construction and name/signature utilities used
  across the optimization modules.

Any directory-module submodules (none currently planned — the split is flat
files) would use `use crate::*;` (not `super`). Per the `kali_runtime` lesson,
drop `use crate::*;` from any module (e.g. a `test_support` module) that
references no crate items, to keep `clippy -D warnings` clean.

## Data flow

Unchanged by this refactor. For reference: a caller builds an `Optimizer`
(`new`/`with_*`), optionally attaches `ProfileData`, and calls
`optimize_program*`; `optimize_program_internal` drives a recursive walk
(`optimize_node`) that dispatches to constant folding, algebraic simplification,
object/layout folding, inlining, and (with MIR) layout specialization, producing
an `OptimizationReport`. The refactor only relocates the code that already
implements this flow.

## Testing

### Co-location

Split the flat `tests.rs` (110 tests) into sibling `*_tests.rs`, one per source
module, each wired at the bottom of its module:

```rust
#[cfg(test)]
#[path = "constant_fold_tests.rs"]
mod constant_fold_tests;
```

Tests are grouped by the module they exercise (`driver_tests.rs`,
`constant_fold_tests.rs`, `specialize_tests.rs`, `inline_tests.rs`,
`object_fold_tests.rs`, `layout_tests.rs`, `helpers_tests.rs`). The net `cargo
test -- --list` **basename** set equals the baseline plus the one added fixture
test below.

### Shared & local support

- **`kali_test_support`** (existing dev-crate): added to `kali_optimize`'s
  `[dev-dependencies]`. `kali_optimize` has **no** filesystem/tempdir tests, so —
  exactly like `kali_codegen` — there is nothing to convert and the dep would
  otherwise be flagged "declared-but-unused" by the final review. Resolution
  (replicating the `kali_codegen` decision): add **one** new fixture test that
  exercises `kali_test_support::fixtures`, taking the count **110 → 111**. This is
  a deliberate, user-approved deviation from the identical-count invariant, scoped
  to exactly one test, recorded in the `…-renames.md` baseline note.
- **Crate-local `test_support` module**: extracted **only if** the split test
  files share `LirProgram`-builder boilerplate worth centralizing (decided during
  the split). If extracted, it must not carry an unused `use crate::*;` header
  (the `kali_runtime` clippy lesson).

## Error handling

No change. Diagnostics and fallback paths move with their methods; behavior is
preserved and asserted by the unchanged tests.

## Sequencing & commit strategy

Incremental, green-at-every-commit, mirroring the prior refactors. Work lands on
branch `refactor/kali-optimize-modularization` off `main`. Commit messages follow
the existing style (`refactor(kali_optimize): …`, `test(kali_optimize): …
[refactor]`, `style(kali_optimize): …`).

1. **Baseline** — record `cargo test -- --list` + counts under
   `docs/superpowers/baselines/`. (no code change)
2. **Widen visibility** — flip cross-referenced items/fields/methods/functions to
   `pub(crate)` while still a single `lib.rs`.
3. **Extract scaffolding** — `driver.rs` (public surface + recursion core), with
   the facade `lib.rs` growing its `mod`/`pub use` lines.
4. **Extract optimization modules** — `constant_fold` → `specialize` → `inline` →
   `object_fold` → `layout` → `helpers`, one commit per module (each its own
   `impl Optimizer` block; supporting structs/enums move with their module).
5. **`cargo fmt`** normalization — its own commit.
6. **Test support** — wire `kali_test_support` dev-dep; introduce crate-local
   `test_support` if warranted; add the one fixture test.
7. **Co-locate tests** — split `tests.rs` into `*_tests.rs` per module, one commit
   per module group; delete the monolith last.
8. **Post-refactor baseline** + final workspace-wide build/test/clippy.

**Per-step verification:** `cargo build -p kali_optimize` → `cargo test -p
kali_optimize` → `cargo clippy -p kali_optimize --all-targets -- -D warnings`
(clippy gated **every** task, per the `kali_runtime` lesson). Final step also runs
the full-workspace build + test. A whole-branch opus review precedes merge.

## Out of scope

- Any behavior, output, or public-API change.
- **Cracking large methods** (e.g. `optimize_call_site`,
  `optimize_algebraic_identity`) into smaller helpers — a separate logic refactor
  with its own spec → plan cycle, deferred exactly as `kali_codegen` deferred
  cracking `emit_call`.
- Refactoring other crates (`kali_common`, `kali_npm`, `kali_cli`, …) — each is
  its own spec → plan → implementation cycle.
- Renaming public items or restructuring the optimization pipeline's semantics.

## References

- Pilot design: `docs/superpowers/specs/2026-06-23-kali-crate-modularization-design.md`
- Codegen design: `docs/superpowers/specs/2026-06-24-kali-codegen-modularization-design.md`
- Codegen emit sub-split: `docs/superpowers/specs/2026-06-24-kali-codegen-emit-subsplit-design.md`
- Runtime design: `docs/superpowers/specs/2026-06-24-kali-runtime-modularization-design.md`
- Prior baselines: `docs/superpowers/baselines/kali_{types,codegen,runtime}-tests-*`
- Memory: `kali-crate-modularization`
