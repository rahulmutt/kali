# Kali Crate Modularization — Design

**Date:** 2026-06-23
**Status:** Approved (design)
**Scope:** Pilot on `kali_types`, then roll the validated pattern out to the remaining crates.

## Problem

The Kali workspace (a TypeScript/JS → WASM AOT compiler, 22 crates, ~324K lines of
Rust) concentrates enormous amounts of code in single files, with equally large
test files:

- `kali_codegen/src/lib.rs` — 10.3K lines, 237 fns
- `kali_types/src/lib.rs` — 7.9K lines (one ~7,000-line `impl TypeContext`)
- `kali_cli/tests/runtime_smoke.rs` — **73K lines / 1,816 tests in one file**
- `kali_types/src/tests.rs` — 15.3K lines, 372 tests
- `kali_cli/src/build_tests.rs` — 15.5K lines, 716 tests
- Many crates are just `lib.rs` + `tests.rs`.

Large files are hard to navigate, review, and reason about. The goal is to break
each crate into small, isolated, single-purpose modules and co-locate tests with
the code they exercise, factoring shared setup into helpers and (pragmatically)
macros.

## Goal & Hard Constraints

This is a **pure structural refactor — zero behavior change.**

- The exact same set of tests must exist and pass before and after.
- `lib.rs` becomes a thin **facade** (module declarations + `pub use`
  re-exports) so every external path (e.g. `kali_types::TypeContext`) keeps
  resolving. No public API churn.
- Conform to the established repo convention (AGENTS.md): unit tests live in
  **sibling `*tests.rs` files wired via `#[path = "…"] mod`**, not inline
  `#[cfg(test)]` modules.

### Proof obligation

Capture a baseline before touching code and compare after:

```
cargo test -p kali_types -- --list   # snapshot test names → diff after refactor
cargo test -p kali_types             # must stay green at every commit
```

The `--list` snapshot guards against silently dropping or duplicating tests
while they are relocated.

## Sequencing

**Pilot one crate (`kali_types`), get sign-off, then roll out crate-by-crate.**
The pattern is validated on real code before scaling. `kali_cli` (250 integration
files incl. `runtime_smoke.rs`) is deliberately deferred to last because its
integration-test surface is atypical.

## Source Decomposition Strategy

`kali_types` is effectively one ~7,000-line `impl TypeContext`. Rust allows a
single `impl` block to span many files within a crate, so we keep the **struct
definition in one place** and **split its methods by responsibility**, each file
carrying its own `impl TypeContext { … }`. No logic is rewritten.

Approaches considered:

- **(A) impl-split into a functional directory tree — CHOSEN.** Honest module
  boundaries, no logic rewriting, lowest risk.
- (B) flat sibling modules under `src/` — rejected: a dozen peer files with no
  grouping.
- (C) deeper refactor splitting `TypeContext` into sub-structs — rejected:
  changes internal interfaces and risks behavior drift; incompatible with a
  "no behavior change" refactor.

### Target layout (`kali_types/src/`)

```
src/
  lib.rs            facade: mod decls + pub use, crate docs
  scope.rs          ScopeType, Scope, ScopeRef, bind helpers          (+ scope_tests.rs)
  context.rs        TypeContext struct + fields + Default + ctors/config
  resolve/
    mod.rs          resolve_statements/statement/block/loop/switch/var-decl
    expression.rs   resolve_expression, identifier, update, optional-chain, template
    call.rs         resolve_call_expression + callable-name helpers
    member.rs       resolve_member_expression + member_access_name* helpers
    function.rs     function/arrow/class/import/export resolution
    jsx.rs          resolve_jsx_*
  static_analysis/
    mod.rs
    array.rs        is_static_array_*, array-callback predicates
    string.rs       static string + string member-call resolution
    object.rs       static object identity/model/keys/from_entries
    math.rs         resolve_math_*
    number.rs       number predicates, parse_int/float
    promise.rs      resolve_promise_member_call
  late_host.rs      resolve_late_* (host/subprocess/network/env/permission/intl)
  typecheck.rs      check_node/typecheck/TypeChecker + annotation-parse helpers
  package.rs        package-root + native-addon rejection
  builtins.rs       builtin_globals, node_builtin_*, StaticObjectIdentityValue
```

Module groupings are derived from the existing method clusters in the current
`impl TypeContext`. The exact placement of any individual method is settled
during implementation; the structure above is the target shape, not a frozen
file-by-file contract.

Each source file that has tests gets a **sibling `*_tests.rs`** wired as:

```rust
#[cfg(test)]
#[path = "foo_tests.rs"]
mod foo_tests;
```

## Test Decomposition

The 372 tests are a flat list (354 named `test_resolution_*`), so the names carry
no thematic information. Each test is mapped to the source module it exercises by
**reading its body**, then moved into that module's `*_tests.rs`. Tests are
**renamed to meaningful names** during relocation (the old `test_resolution_NNN`
names convey nothing). The assertions are unchanged; only location and name move.
The baseline `--list` snapshot is used to confirm no test is lost or duplicated
(accounting for deliberate renames).

## Test Infrastructure

Hybrid helper placement plus pragmatic macros:

- **New `crates/kali_test_support` crate** (dev-dependency): cross-crate helpers
  reusable during rollout — tempdir/fixture writers, manifest builders, process
  setup, common assertions.
- **Per-crate `src/test_support/` module** (compiled under `cfg(test)`):
  crate-specific helpers — for `kali_types`, the AST-node **builders** that
  replace boilerplate like `sequence_expression` and
  `optional_chain_global_this_math`.
- **Pragmatic macros** where they cut the most boilerplate — e.g. an
  `assert_resolution!` macro and a small AST-builder DSL — and plain builder
  functions everywhere a macro would not add clarity.

## Execution & Verification Rhythm

Small, reviewable commits; `cargo test -p kali_types` green after each:

1. Capture the `cargo test -p kali_types -- --list` baseline.
2. Scaffold `kali_test_support` + per-crate `test_support` builders/macros.
3. Split source modules one functional cluster at a time, behind the facade.
4. Relocate tests into the matching sibling `*_tests.rs` files, renaming them.
5. Final check: diff against the `--list` baseline, run `cargo fmt` and
   `cargo clippy`, and confirm the full suite is green.

Then **pause for review** before generalizing the pattern to the next crate.

## Rollout (after pilot sign-off)

Apply the same shape to the remaining crates, ordered roughly worst-offender
first among the simpler crates, with `kali_cli` (and `runtime_smoke.rs`) last:

- `kali_codegen`, `kali_runtime`, `kali_optimize`, `kali_common`, `kali_parser`,
  `kali_ast`, `kali_error`, `kali_sandbox`, `kali_api_*`, `kali_capi`,
  `kali_npm`, `kali_fmt`, `kali_lint`, `kali_embed`, `kali_hir`, `kali_mir`,
  `kali_lir`.
- `kali_cli` last: split `build.rs`/`output.rs`/`init.rs` and shard the giant
  `tests/runtime_smoke.rs` into themed integration files (mirroring the existing
  `browser_*`, `object_*`, `array_*`, `math_*` naming already present under
  `tests/`).

Each crate is its own spec → plan → implementation cycle if it turns out to need
more than the mechanical pattern; crates that fit the pattern cleanly can reuse
this design directly.
