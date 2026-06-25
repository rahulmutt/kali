# kali_mir Modularization — Design

**Date:** 2026-06-25
**Status:** Approved (design)
**Scope:** Apply the validated crate-modularization pattern (see
`2026-06-23-kali-crate-modularization-design.md`) to `kali_mir`, the 9th crate
in the effort and the next core-pipeline monolith after `kali_hir`.

## Problem

`kali_mir` is two large files:

- `src/lib.rs` — 1,569 lines: 18 public MIR data types (ownership/threading
  types, `LayoutDescriptor`, binding types, function types, node/arena types,
  `MirProgram`), two free helper fns (`validate_tree`, `function_scope_name`),
  a small `pub struct MirLowerer` (HIR→MIR structural lowering) plus its free
  helper `map_kind`, and a large private `OwnershipAnalyzer` engine — one
  struct and a single ~670-line `impl` of ~26 memory/ownership-analysis methods
  (`walk_scope_node` alone is ~250 lines) backed by ~230 lines of support types
  and free fns (`UseContext`, `BindingState`, `ScopeState`, `default_ownership`,
  `parameter_escape_flags`, `function_binding_escapes`, `finalise_binding`).
- `src/tests.rs` — 1,043 lines: 36 `#[test]` functions sharing a parse/lower
  helper.

A single file mixing the data model, two distinct engines (structural lowering
and ownership analysis), and a ~900-line analysis subsystem — plus a flat
1,043-line test file — is hard to navigate, review, and reason about.

## Goal & Hard Constraints

**Pure structural refactor — zero behavior change.** Only items are relocated
and visibility widened; no logic is rewritten.

- The exact same set of tests exists and passes before and after.
- `cargo test -p kali_mir` is **green after every commit** (36 unit tests; no
  integration-test directory exists for this crate).
- `lib.rs` becomes a thin **facade** (crate docs + module declarations +
  `pub use` re-exports + `cfg(test)` test wiring, ~30 lines) so every external
  path keeps resolving. The public API is preserved byte-for-byte — all 18
  public types remain at their flat paths and keep their public fields and
  methods:
  `kali_mir::{OwnershipClass, ThreadBoundaryDisposition, ThreadBoundaryBinding,
  ThreadBoundaryProfile, LayoutDescriptor, MirBindingKind, MirBinding,
  BorrowedLifetime, MirFunctionKind, MirFunction, MirNodeKind, MirNodeId,
  PlaceRef, PlaceValue, MirNode, MirBuilder, MirProgram, MirLowerer}`.
  No public API churn. (Downstream consumers: `kali_codegen`, `kali_optimize`,
  `kali_lir`, `kali_cli`.)
- Conform to the repo convention (AGENTS.md §5): unit tests live in sibling
  `*_tests.rs` files wired via `#[cfg(test)] #[path = "…"] mod`, **not** inline
  `#[cfg(test)]` modules. The shared parse/lower helper lives in a
  `test_support.rs` wired as a plain `#[cfg(test)] mod test_support;`.
- **Visibility widened minimally:** items needing cross-module access become
  `pub(crate)`, never bare `pub`. Items whose sole caller is co-located stay
  private (e.g. `validate_tree`/`function_scope_name` move with `MirProgram`;
  `map_kind` moves with `MirLowerer`; the entire `OwnershipAnalyzer` engine and
  its support types stay `pub(crate)`/private — none is part of the public API).
- Verbatim text-movement: method/fn/type bodies move byte-for-byte, including
  blank-line separators and the original's exact qualification style (do **not**
  convert inline `kali_hir::Foo` refs to imported short names or vice versa).
  Prior crates in this effort caught dropped blank lines and silent
  requalification — watch for both.

### Proof obligation

Capture a baseline before touching code and compare after. As established by
the `kali_parser` correction and the `kali_npm`/`kali_hir` precedent, the
durable check is the **basename multiset** of test names (module-path prefixes
change as tests relocate, so a raw full-name diff would falsely fail):

```
cargo test -p kali_mir -- --list 2>/dev/null | grep ': test$' \
  | sed -E 's/^.*:://; s/: test$//' | sort
```

Compare before vs after with `diff` → must be **empty**. Use `sort` **without**
`-u` (preserve duplicates). Baseline captured pre-flight: **36** sorted
basenames, **no duplicates**.

## Architecture: target module layout

`src/lib.rs` becomes a facade re-exporting from these modules. Largest
resulting source file ≈ 250 lines (`analysis/walk.rs`).

### Data-type modules (flat siblings)

| module | contents |
|---|---|
| `ownership.rs` | `OwnershipClass`, `ThreadBoundaryDisposition`, `ThreadBoundaryBinding`, `ThreadBoundaryProfile` + their impls |
| `layout.rs` | `LayoutDescriptor` + impl |
| `binding.rs` | `MirBindingKind`, `MirBinding`, `BorrowedLifetime` + impl |
| `function.rs` | `MirFunctionKind`, `MirFunction` + impl |
| `node.rs` | `MirNodeKind`, `MirNodeId`, `PlaceRef`, `PlaceValue`, `MirNode`, `MirBuilder` + impls (the node/arena primitives) |
| `program.rs` | `MirProgram` + impl, plus free fns `validate_tree` and `function_scope_name` (co-located with their sole caller `MirProgram`; stay private) |

### Structural-lowering module (flat)

| module | contents |
|---|---|
| `lower.rs` | `pub struct MirLowerer` + impl, plus free fn `map_kind` (co-located with its sole caller; stays private). Calls `OwnershipAnalyzer::new(...).analyze_program(...)`. |

### Ownership-analysis subtree (impl-split by concern)

The ~900-line `OwnershipAnalyzer` engine splits into an `analysis/` subtree,
mirroring how `kali_hir` split its ~900-line `impl HirLowerer` into a
`lowering/` subtree. The struct is defined once in `analysis/mod.rs`; each
sibling carries its own `impl OwnershipAnalyzer` block. Method placement is a
target grouping, not a frozen contract — borderline shifts are fine as long as
it compiles and the suite stays green.

| module | contents |
|---|---|
| `analysis/mod.rs` | `struct OwnershipAnalyzer` + entry methods (`new`, `analyze_program`, `function_flavor`); support types `UseContext`, `BindingState` (+impl), `ScopeState` (+impl); free fns `default_ownership`, `parameter_escape_flags`, `function_binding_escapes`, `finalise_binding`; `mod`/`use` wiring for the siblings |
| `analysis/scope.rs` | `impl OwnershipAnalyzer`: `push_scope`, `pop_scope_and_record`, `current_scope_label`, `current_scope_index`, `current_scope_mut`, `precollect_scope_bindings`, `define_binding`, `collect_import_bindings` |
| `analysis/walk.rs` | `impl OwnershipAnalyzer`: `walk_scope_node` (~250-line dispatcher), `resolve_use`, `resolve_binding`, `is_heap_store_target` |
| `analysis/infer.rs` | `impl OwnershipAnalyzer`: `infer_layout`, `infer_binary_layout`, `infer_unary_layout`, `resolve_binding_layout`, `layout_field_name`, `object_property_order_key` |
| `analysis/resolve.rs` | `impl OwnershipAnalyzer`: `function_parameter_escape_flags`, `resolve_function_target`, `function_target_from_node`, `function_name_from_recent_functions`, `next_function_name` |

### Facade (`lib.rs`, ~30 lines)

Crate-level docs + `mod` declarations (alphabetical) + `pub use` re-exports of
the 18 public types at their flat paths + `cfg(test)` test wiring. No
functions, structs, enums, impls, or macros remain.

## Data flow (unchanged)

`MirLowerer::lower_hir_result` builds the MIR node arena via `MirBuilder`
(`map_kind` translates each `HirNodeKind`), then runs
`OwnershipAnalyzer::new(&hir.nodes, &hir.function_flavors).analyze_program(root)`
to produce the analyzed `Vec<MirFunction>`, assembling a `MirProgram`.
`MirProgram`'s query/summary methods (`module_scope`, `validate`,
`borrowed_lifetimes*`, `thread_boundary_profile*`) use the co-located free fns
`validate_tree` and `function_scope_name`. Relocation does not change any of
these call paths — only the files the items live in.

## Cross-module visibility (what gets widened)

- `OwnershipAnalyzer`'s struct fields and its ~26 methods → `pub(crate)` (the
  impl is split across `analysis/` siblings, so fields/methods accessed across
  those files must be crate-visible). The struct itself is `pub(crate)`.
- Support types `UseContext`, `BindingState`, `ScopeState` and their methods,
  and free fns `default_ownership`/`parameter_escape_flags`/
  `function_binding_escapes`/`finalise_binding` → `pub(crate)` where referenced
  by sibling `impl` blocks; otherwise remain private within `analysis/mod.rs`.
- `MirBuilder` fields read across module boundaries (e.g. `nodes` read by
  `MirLowerer`) → `pub(crate)`.
- Items whose only caller is co-located stay **private**: `validate_tree`,
  `function_scope_name` (with `MirProgram`), `map_kind` (with `MirLowerer`).
- Nothing is widened to bare `pub` beyond what is already public API.

## Testing

The 36 existing tests relocate into sibling `*_tests.rs` files wired via
`#[cfg(test)] #[path = "…"] mod`, grouped to match the module they exercise
(e.g. ownership/threading-type tests, layout tests, binding/borrowed-lifetime
and program-summary tests, lowering tests, and the ownership-analysis escape
tests). The shared parse/lower helper moves to `test_support.rs`
(`#[cfg(test)] mod test_support;`). The implementer maps each test to its
home by reading bodies; the **basename-multiset diff is the authoritative gate**
— exact per-file placement is a target, not a frozen contract. No new tests,
no changed assertions.

## Out of scope

- No logic changes, no API changes, no renames of public items.
- No new dependencies.
- No reformatting beyond a final `cargo fmt` (behavior-neutral) as the last
  commit.
- No changes to other crates (the facade keeps every `kali_mir::*` path stable).

## Commit convention

`refactor(kali_mir): <summary> [refactor]` (or `test`/`style`/`docs` as the
prefix fits), one commit per task, suite green at each.
