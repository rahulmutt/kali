# kali_hir Modularization — Design

**Date:** 2026-06-25
**Status:** Approved (design)
**Scope:** Apply the validated crate-modularization pattern (see
`2026-06-23-kali-crate-modularization-design.md`) to `kali_hir`, the 8th crate
in the effort and the next core-pipeline monolith after `kali_parser`.

## Problem

`kali_hir` is two large files:

- `src/lib.rs` — 1,248 lines: the HIR node types (`HirNodeKind`, `HirNode`,
  `HirNodeId`, `HirBuilder`, `FunctionFlavor`, `LoweringResult`), one
  `pub struct HirLowerer` plus a single ~900-line `impl HirLowerer` of ~30
  AST-to-HIR lowering methods (`lower_statement` alone is ~356 lines,
  `lower_expression` ~192 lines), a `push_child!` macro, and 5 free helper fns.
- `src/tests.rs` — 638 lines: 20 `#[test]` functions sharing one `parse()`
  helper.

A single ~900-line `impl` mixing statement, expression, function, object, and
import/export lowering — plus a flat 638-line test file — is hard to navigate,
review, and reason about.

## Goal & Hard Constraints

**Pure structural refactor — zero behavior change.** Only items are relocated
and visibility widened; no logic is rewritten.

- The exact same set of tests exists and passes before and after.
- `cargo test -p kali_hir` is **green after every commit** (20 unit tests; no
  integration-test directory exists for this crate).
- `lib.rs` becomes a thin **facade** (crate docs + module declarations +
  `pub use` re-exports + `cfg(test)` test wiring) so every external path keeps
  resolving. The public API is preserved byte-for-byte:
  `kali_hir::{HirLowerer, LoweringResult, FunctionFlavor, HirBuilder, HirNode,
  HirNodeId, HirNodeKind}` (and their public methods/fields) remain at their
  flat paths. No public API churn.
- Conform to the repo convention (AGENTS.md §5): unit tests live in sibling
  `*_tests.rs` files wired via `#[cfg(test)] #[path = "…"] mod`, **not** inline
  `#[cfg(test)]` modules.
- Verbatim text-movement: method/fn/type bodies move byte-for-byte, including
  blank-line separators and the original's exact qualification style (do **not**
  convert inline `kali_ast::Foo` refs to imported short names or vice versa).
  Prior crates in this effort caught dropped blank lines and silent
  requalification — watch for both.

### Proof obligation

Capture a baseline before touching code and compare after. As established by
the `kali_parser` correction and the `kali_npm` precedent, the durable check is
the **basename multiset** of test names (module-path prefixes change as tests
relocate, so a raw full-name diff would falsely fail):

```
cargo test -p kali_hir -- --list 2>/dev/null | grep ': test$' \
  | sed -E 's/^.*:://; s/: test$//' | sort
```

Compare before vs after with `diff` → must be **empty**. Use `sort` **without**
`-u` so any duplicate basename is preserved (a real basename collision, if one
exists among the 20 tests, must be confirmed empirically at baseline and the
count held constant). `cargo test -p kali_hir` must also stay green at every
commit. The basename-multiset diff guards against silently dropping,
duplicating, or renaming a test during relocation.

## Source Decomposition (impl-split)

The HIR node **types stay together by concern**, and the `HirLowerer` methods
**split by responsibility**, each sibling file carrying its own
`impl HirLowerer { … }`. No logic is rewritten. Private items called across the
new module boundaries are widened to `pub(crate)` (the established "blanket
widen for extraction" step).

### Target layout (`kali_hir/src/`)

```
src/
  lib.rs           facade: crate docs + mod decls + `pub use` re-exports + cfg(test) test wiring
  node.rs          HirNodeKind (enum) + HirNode (struct + impl) + HirNodeId (struct + impl)
  builder.rs       HirBuilder (struct + impl + impl Default)
  result.rs        FunctionFlavor (enum + impl) + LoweringResult (struct + impl) + validate_tree
                   (free fn — kept here as it is private to LoweringResult::validate)
  helpers.rs       free fns: lower_literal_value, logical_op_text, update_op_text,
                   assignment_op_text, object_property_kind_text
  lowering/
    mod.rs         HirLowerer struct + new + impl Default + push_child! macro;
                   pub entry API: lower_statements, lower_program_from_ast, lower_node,
                   diagnostics, clear_diagnostics;
                   shared helpers: push_child, record_function_flavor, next_synthetic_function_name
    statement.rs   lower_statement (dispatcher) + lower_block, lower_class_body,
                   lower_method_definition, lower_variable_declarator
    expression.rs  lower_expression (dispatcher) + lower_template_literal,
                   lower_update_expression, lower_assignment_expression, lower_optional_chain
    function.rs    lower_function_expression, lower_arrow_function_expression, lower_class_expression
    object.rs      lower_object_property, lower_property_name
    module.rs      lower_import_specifier, lower_export_specifier, lower_export_default
```

Module groupings are derived from the existing method clusters. The exact
placement of any individual method (e.g. whether `lower_block` lands in
`statement.rs`, or `lower_optional_chain` in `expression.rs`) is settled during
implementation; the structure above is the target shape, not a frozen
file-by-file contract. As long as it compiles and the suite stays green, minor
placement shifts are acceptable.

The `lowering/` subdirectory (one core module + 5 focused pass modules) is
preferred over a single flat ~900-line `lowering.rs`, consistent with the
`expression/` subtree precedent from `kali_parser`.

## Two crate-specific wrinkles (no direct kali_parser precedent)

These are the only elements not covered verbatim by the prior six impl/free-fn
splits; both have an established repo idiom.

### 1. The `push_child!` macro

`push_child!` (`macro_rules!`, lib.rs:247) is invoked by nearly every lowering
method to evaluate a child expression and then call `self.push_child(parent,
child)` — sidestepping a double-mutable-borrow. When the lowering methods split
across sibling modules, the macro must be visible to all of them.

Resolution (repo's established `kali_types/test_support.rs` idiom): define the
`macro_rules! push_child` in `lowering/mod.rs` immediately followed by
`pub(crate) use push_child;`. Each sibling pass module consumes it via
`use crate::lowering::push_child;`. The macro's target method `push_child`
widens to `pub(crate)`. No `#[macro_export]` / `#[macro_use]` is used — the
path-import form matches existing crate style.

### 2. Direct field access across the new boundary

Two fields are read directly (not only through methods) from code that will
live in a different module after the split:

- `HirLowerer.builder` — accessed as `self.builder.alloc(...)` /
  `self.builder.alloc_text(...)` in essentially every lowering method (all of
  which leave `mod.rs`). Widens to `pub(crate)`.
- `HirBuilder.nodes` — read directly as `self.builder.nodes.clone()` in
  `lower_statements` (lib.rs:290), which stays in `lowering/mod.rs` while
  `HirBuilder` moves to `builder.rs`. Widens to `pub(crate)`.

Per the blanket-widen step, all 4 `HirLowerer` fields (`builder`,
`diagnostics`, `function_flavors`, `synthetic_function_counter`) widen to
`pub(crate)` for uniformity, alongside all private `HirLowerer` lowering
methods and the 5 free helper fns in `helpers.rs`. `validate_tree` stays
private (its only caller, `LoweringResult::validate`, is co-located in
`result.rs`). The already-`pub` items (`HirBuilder::{new, alloc, alloc_text,
node_mut}`, the entry API, the type constructors) keep their visibility.

## Test Decomposition

The 20 tests are already meaningfully named
(`test_lower_statements_records_function_flavor_metadata_for_class_methods`,
`test_numeric_object_property_names_lower_as_string_literals`,
`test_hir_validation_rejects_out_of_bounds_children`, …), so **no renaming is
required**. Each test is mapped to the source module it exercises by reading its
body and moved into that module's sibling `*_tests.rs`, e.g.:

```
builder_tests.rs            (test_hir_builder)
lowering/statement_tests.rs (lower_statements_to_hir, lower_program_from_ast,
                             export_all node tests)
lowering/function_tests.rs  (the function-flavor-metadata family)
lowering/object_tests.rs    (object-literal + numeric-property-name tests)
lowering/expression_tests.rs(update-expression form test)
result_tests.rs             (hir_validation_rejects_out_of_bounds_children)
```

wired as:

```rust
#[cfg(test)]
#[path = "statement_tests.rs"]
mod statement_tests;
```

Assertions are unchanged; only location moves. The basename-multiset baseline
confirms no test is lost, duplicated, or renamed. Exact per-test placement is
settled during implementation by reading each test body; the mapping above is
the expected shape.

## Test Infrastructure

The shared `parse(source) -> Vec<Statement>` helper wraps `kali_parser::Parser`
+ `kali_lexer::Lexer` (`kali_parser` and `kali_lexer` are already dev-deps of
this crate — unlike `kali_parser`'s self-contained `lex`, this crate lowers
real parsed ASTs). It moves into a small `cfg(test)` `test_support` module
exposing `pub(crate) fn parse`, shared across the split `*_tests.rs` files. **No
change to `kali_test_support` is needed** — `parse` is specific to this crate's
tests.

## Incidental cleanup

The monolith carries `#[allow(unused_imports)]` on its large `use kali_ast::{…}`
block (a flat catch-all import for the whole file). After the split, each module
imports precisely the `kali_ast` names it uses, so the `allow` disappears
naturally — consistent with the per-module precise-import precedent from the
prior crates. This is a faithful consequence of the split, not a separate
refactor.

## Execution & Verification Rhythm

Small, reviewable commits; `cargo test -p kali_hir` green after each:

1. Capture the basename-multiset baseline (command above).
2. Widen private items to `pub(crate)` for extraction (fields, lowering
   methods, free helper fns, the `push_child` method).
3. Extract type modules behind the facade (`node` → `builder` → `result` →
   `helpers`), keeping `lib.rs` a thin facade as items move.
4. Extract the lowering subtree: `lowering/mod.rs` (struct + entry + shared
   helpers + `push_child!` macro) → `statement` → `expression` → `function` →
   `object` → `module`, one functional cluster per commit.
5. Reduce `lib.rs` to the final facade.
6. Relocate tests into matching sibling `*_tests.rs` files; introduce the shared
   `test_support::parse` helper.
7. Final check: basename-multiset diff against baseline → empty; run
   `cargo fmt -p kali_hir` and `cargo clippy -p kali_hir --all-targets` clean;
   `cargo build` (workspace) clean so downstream consumers compile against the
   facade; confirm the full suite is green.

This crate reuses the validated impl-split pattern directly. The only new
decisions — macro relocation and cross-module field widening — are resolved
above with established repo idioms; no new pattern decisions are expected.
