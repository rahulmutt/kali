# kali_lint modularization — design

**Date:** 2026-06-26
**Crate:** `kali_lint` (15th in the kali crate-modularization series)
**Predecessors:** the api trio `kali_api_web`/`node`/`deno` (11th–13th) and `kali_capi` (14th). This spec reuses the series' facade + co-located-tests playbook and records where `kali_lint` diverges.
**Execution:** subagent-driven-development, one module per task.
**Integration:** fast-forward merge to **local main only** (series default). Re-verify on merged main, delete branch.

## Goal

Decompose the monolithic `crates/kali_lint/src/lib.rs` (879 lines) into a thin facade plus 6 sibling modules, and relocate `src/tests.rs` (2 end-to-end tests) into a co-located `engine_tests.rs` — with **zero behavior change** and a **preserved public API**.

## What this crate actually is

`kali_lint` is a linter for Kali source. The pipeline is: lex → parse → run a set of rule checks over the token stream + AST → collect `Diagnostic`s → optionally apply safe fixes and re-format. The bulk is private machinery; the public surface is tiny.

## Shape: ORCHESTRATOR + RULE-METHOD-PILE on a shared `Analyzer`

The web/node/deno precedent (an object with impls), **not** capi's flat function-pile. A single private `Analyzer` mega-struct holds the shared state (`tokens`, `statements`, `diagnostics`, `fix_plan`); its `run()` method drives ten `check_*` methods; a `FixPlan` struct accumulates fix sites. The natural seam is the **lint concern** (style / variables / control-flow / scope), with the driver and fix-application as their own units.

Because rule methods live in different modules than `run()` and all share `Analyzer`'s fields, this crate **does** need the Task-1 blanket `pub(crate)` receiver-widening (the deno/node/web precedent; capi had none because it had no mega-struct).

## Public-API invariant

Exactly **3 flat `kali_lint::Name` paths**, unchanged before and after:

- `lint(source: &str) -> Vec<Diagnostic>` (fn)
- `lint_with_options(source: &str, fix: bool) -> LintResult` (fn)
- `LintResult` (struct, with pub fields `diagnostics: Vec<Diagnostic>`, `fixed_source: Option<String>`)

All three move into the `engine` module and are restored at the crate root via `pub use engine::*;`. **Basename-multiset proof** (series invariant): the set of `kali_lint::Name` flat paths is identical before/after — 2 fns + 1 struct = 3 flat names.

**Sole external consumer:** `crates/kali_cli/src/bin/kali.rs`, line 31 — `use kali_lint::lint_with_options;`. The glob facade preserves the path with **zero consumer edits**. (`kali_error/src/lib.rs` only mentions `kali_lint` in a comment; not a code dependency.)

## Architecture

`lib.rs` 879 → **thin facade**: `mod` declarations, `pub use engine::*;`, and the crate-level `//!` doc comment. All logic moves into modules.

### Module map (cut by item name — never by absolute line range)

Functions are interleaved in the source; after each move the line numbers shift. Re-locate the next item with `grep -n 'fn <name>' src/lib.rs`.

| Module | Public-to-crate items | ~lines |
|---|---|---|
| `engine` | `lint`, `lint_with_options`, `LintResult`, `Analyzer` (struct + `new`, `run`, `collect_declared_names`, `count_identifier_tokens`), `FixPlan` (struct) | ~140 |
| `style` | `impl Analyzer`: `check_explicit_any`, `check_no_console`, `check_debugger`, `check_eqeqeq` | ~80 |
| `variables` | `impl Analyzer`: `check_no_var_and_prefer_const`; free fns: `walk_statement_for_var_rules`, `check_variable_declaration_kind` | ~180 |
| `control_flow` | `impl Analyzer`: `check_no_empty_and_unreachable`; free fns: `check_statement_for_empty_and_unreachable`, `check_block_for_unreachable`, `is_terminating_statement` | ~130 |
| `scope` | `impl Analyzer`: `check_no_unused_vars`, `check_no_unused_imports`, `check_no_undef`; free fns: `collect_statements_declarations`, `collect_statement_declarations`, `collect_block_declarations`, `collect_import_ranges`, `builtin_globals` | ~230 |
| `fixes` | `apply_fixes` | ~45 |

The `check_*` methods stay `impl Analyzer` blocks (one block per rule module). Standalone helpers move as free functions into the module that uses them.

### Item → owning module (full inventory)

- **engine:** `lint`, `lint_with_options`, `LintResult`, `Analyzer`, `Analyzer::new`, `Analyzer::run`, `Analyzer::collect_declared_names`, `Analyzer::count_identifier_tokens`, `FixPlan`
- **style:** `Analyzer::check_explicit_any`, `Analyzer::check_no_console`, `Analyzer::check_debugger`, `Analyzer::check_eqeqeq`
- **variables:** `Analyzer::check_no_var_and_prefer_const`, `walk_statement_for_var_rules`, `check_variable_declaration_kind`
- **control_flow:** `Analyzer::check_no_empty_and_unreachable`, `check_statement_for_empty_and_unreachable`, `check_block_for_unreachable`, `is_terminating_statement`
- **scope:** `Analyzer::check_no_unused_vars`, `Analyzer::check_no_unused_imports`, `Analyzer::check_no_undef`, `collect_statements_declarations`, `collect_statement_declarations`, `collect_block_declarations`, `collect_import_ranges`, `builtin_globals`
- **fixes:** `apply_fixes`

## Visibility changes (the only semantic edits — no signature or body changes)

1. **Receiver-widening (Task 1):** `Analyzer`'s fields (`tokens`, `statements`, `diagnostics`, `fix_plan`) and **all** `FixPlan` fields (`var_tokens`, `let_to_const_tokens`, `eqeqeq_tokens`, `debugger_tokens`, `unused_import_ranges`) become `pub(crate)`, so `impl Analyzer` blocks in rule modules and free fns in `variables` can read/mutate them. The `struct Analyzer`/`struct FixPlan` type definitions themselves stay crate-private (no `pub`).
2. **Check methods → `pub(crate) fn`:** every `check_*` method that `run()` (in `engine`) invokes must be `pub(crate)`, because an inherent method defaults to private-to-the-module-of-its-impl-block and `run` lives in a different module than the rule impls.
3. **Cross-module free fns → `pub(crate)`:** any free fn called outside its defining module (notably `apply_fixes`, called by `lint_with_options` in `engine`). Helpers used only within their own module (e.g. `collect_import_ranges`, `builtin_globals`, `check_variable_declaration_kind`, the `collect_*` recursion) stay private unless a cross-module call requires otherwise.

No function signatures change. No bodies change. The only edits are `mod`/`use` wiring and `pub(crate)` visibility.

## Test layout

`src/tests.rs` holds 2 end-to-end tests, both driving the full public pipeline:

| test file | wired from | count | tests |
|---|---|---|---|
| `engine_tests.rs` | `engine.rs` | 2 | `reports_basic_lint_issues`, `fix_mode_applies_basic_safe_rewrites` |

Both exercise `lint` / `lint_with_options` end to end (var, prefer-const, debugger, eqeqeq, and fix mode). They are cross-cutting integration tests, so they live with the `engine` driver rather than being scattered per rule (capi precedent: cross-cutting smoke tests wired from one place). Per-rule modules get **no** standalone tests in this pass.

**Wiring:** `engine.rs` ends with
```rust
#[cfg(test)]
#[path = "engine_tests.rs"]
mod engine_tests;
```
**Self-sufficiency rule (series lesson):** `engine_tests.rs` begins with `use crate::*;` (the facade glob-exports `lint`, `lint_with_options`, `LintResult`) **plus** every explicit import its bodies reference — here `use kali_error::_error_codes::w2;` (the tests assert on `w2::*` diagnostic codes). Because `cargo build` skips `cfg(test)`, a missing test import compiles under build but fails under `cargo test`. **Every test-touching task verifies with `cargo test -p kali_lint`, not just `cargo build`.**

## Task order

| # | Task | Constraint |
|---|---|---|
| 1 | Receiver-widening: `Analyzer`/`FixPlan` fields → `pub(crate)` (no moves) | Must precede all rule-module extractions; stays green |
| 2 | Extract `fixes` (`apply_fixes`) | Leaf; `apply_fixes` → `pub(crate)` |
| 3 | Extract `style` (4 token-scan checks) | Independent |
| 4 | Extract `control_flow` | Independent |
| 5 | Extract `scope` | Independent |
| 6 | Extract `variables` | Independent |
| 7 | Finalize: reduce `lib.rs` to facade (`pub use engine::*;`), create `engine.rs`, relocate tests → `engine_tests.rs`, trim now-unused imports | Last |

Rule-module order (3–6) is interchangeable; each is independent once Task 1 has widened the receiver. The `engine` module crystallizes in the finalize task (what remains after every rule has been extracted).

## Verification

- **Per task:** `cargo build -p kali_lint` **and** `cargo test -p kali_lint` green, then commit.
- **Baseline (pre-work, green):** `cargo test -p kali_lint` → 2 passed.
- **Final:** basename-multiset proof (3 flat names identical before/after); `cargo build --workspace` + `cargo test --workspace` to confirm `kali_cli` compiles and passes **unedited**.
- Mid-plan unused-import warnings on the crate-root `use` block are acceptable; that block is trimmed in the finalize task once empty.

## Constraints (series invariants)

- **Zero behavior change, preserved public API.** Same 3 flat `kali_lint::Name` paths. The only visibility changes are the receiver-widening + `pub(crate)` on cross-module items — none of which are reachable as `kali_lint::…`.
- **No consumer edits.** `kali_cli/src/bin/kali.rs` must compile and pass without changes.
- **Facade stays logic-free** — only `mod` decls, `pub use engine::*;`, and the crate doc-comment.
- **Cut by item name, never by line range** — re-`grep` after each move.

## Tech stack

Rust 2021, cargo workspace. Deps (all used): `kali_ast`, `kali_common`, `kali_error`, `kali_fmt`, `kali_lexer`, `kali_parser`. std only beyond those.
