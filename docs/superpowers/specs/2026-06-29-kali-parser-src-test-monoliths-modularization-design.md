# kali_parser src test-monolith modularization — design spec

**Series entry 30.** Continues the crate-by-crate co-located `src/*_tests.rs` unit-test
monolith modularization (kali_optimize 25th, kali_types 26th, kali_runtime 27th,
kali_codegen 28th, kali_common 29th). Same SDD workflow, same reusable mover.

## Goal

Split four multi-concern co-located `src/*_tests.rs` files in `crates/kali_parser` into
thin facades + per-concern `#[path] mod` submodules. **Pure verbatim code-motion** plus
`mod`/`use` wiring only. Zero behavior change; every `#[test]` body byte-identical
base→head; public API unchanged so consumers compile unedited.

## Scope

In scope — split these 4 files (51 `#[test]` total → 12 submodules):

| Facade file | `#[test]` | Submodules (count) |
|---|---|---|
| `src/declaration_tests.rs` | 23 | arrow(7), generator(6), class_method(6), function(4) |
| `src/expression/call_tests.rs` | 8 | member(5), optional_chain(2), dynamic_import(1) |
| `src/expression/mod_tests.rs` | 10 | unary(4), binary(4), type_ops(2) |
| `src/module_tests.rs` | 10 | import(2), export(8) |

Kept whole (single-concern or small, per series precedent — files like kali_common's
array/collections/number that stayed whole):

- `src/expression/object_tests.rs` (7t) — single concern: object-literal computed property names
- `src/statement_tests.rs` (5t), `src/expression/primary_tests.rs` (2t) — small
- The remaining tiny `*_tests.rs` in the crate are not in scope.

## Exact-name-set partition

All four files use the mover's **exact-name-set mode** (full-name set membership `==`, not
leading-prefix `startswith`) — the discriminating token is mid-name in each
(async/generator overlap in declaration; `bracketed`/`dot`/`optional` in call;
`default_import` vs `default_export` in module), which defeats prefix grouping.

### declaration_tests (23 → 4)
- **arrow (7):** `test_parse_parenthesized_arrow_function_expression`,
  `test_parse_single_parameter_arrow_function_expression`,
  `test_parse_async_arrow_function_expression`,
  `test_parse_async_arrow_function_return_type_annotation_with_multiple_params`,
  `test_parse_async_single_parameter_arrow_function_expression`,
  `test_parse_async_arrow_function_return_type_annotation`,
  `test_parse_arrow_function_return_type_annotation`
- **generator (6):** `test_parse_generator_function_declaration`,
  `test_parse_generator_delegating_yield_expression`,
  `test_parse_generator_function_expression`,
  `test_parse_async_generator_function_declaration`,
  `test_parse_async_generator_function_expression`,
  `test_parse_yield_expression_outside_generator_remains_identifier`
- **class_method (6):** `test_parse_generator_class_method_preserves_generator_flag`,
  `test_parse_generator_class_method_delegating_yield_expression`,
  `test_parse_async_generator_class_method_preserves_generator_flags`,
  `test_parse_class_expression_preserves_method_modifiers`,
  `test_parse_default_export_class_expression_preserves_method_modifiers`,
  `test_parse_default_export_class_declaration_preserves_method_modifiers`
- **function (4):** `test_parse_export_async_function_declaration`,
  `test_parse_async_await_expression`,
  `test_parse_function_declaration_stops_before_following_statement`,
  `test_parse_async_function_expression`

### call_tests (8 → 3)
- **member (5):** `test_parse_bracketed_member_expression_chain`,
  `test_parse_fully_bracketed_permission_escalation_member_expression_chain`,
  `test_parse_mixed_bracket_dot_late_object_model_member_expression_chain`,
  `test_parse_dot_delete_member_expression_after_keyword_property`,
  `test_parse_dot_from_member_expression_after_keyword_property`
- **optional_chain (2):** `test_parse_optional_chain_member_expression`,
  `test_parse_optional_chain_index_expression`
- **dynamic_import (1):** `test_parse_dynamic_import_expression`

### mod_tests (10 → 3)
- **unary (4):** `test_parse_prefix_update_expression`,
  `test_parse_void_unary_expression`,
  `test_parse_bitwise_not_unary_expression`,
  `test_parse_postfix_update_expression`
- **binary (4):** `test_parse_nullish_coalescing_expression`,
  `test_parse_exponentiation_expression`,
  `test_parse_modulo_expression`,
  `test_parse_compound_assignment_expression`
- **type_ops (2):** `test_parse_type_assertion_expression`,
  `test_parse_satisfies_expression`

### module_tests (10 → 2)
- **import (2):** `test_parse_side_effect_import_declaration`,
  `test_parse_default_import_declaration`
- **export (8):** `test_parse_named_export_declaration`,
  `test_parse_named_export_declaration_allows_default_aliases`,
  `test_parse_export_all_declaration`,
  `test_parse_default_export_function_declaration`,
  `test_parse_default_export_generator_function_declaration`,
  `test_parse_default_export_async_generator_function_declaration`,
  `test_parse_default_export_anonymous_async_generator_function_declaration`,
  `test_parse_default_export_anonymous_generator_function_declaration`

## Mechanics

- Each facade drains its `#[test]` fns to 0 module-level tests, **keeps its 3 `use` lines**
  (`use crate::test_support::lex;`, `use crate::*;`, `use kali_ast::{...}`), and appends
  `#[path = "<stem>/<mod>.rs"] mod <mod>;` decls. Children consume the retained imports via
  `use super::*;` (Rust descendant-visibility re-exports the facade's private `use` items
  through the child glob — compiles at 0 warnings, no `#[allow]`/`#[cfg(test)]` re-export).
- **declaration_tests retains exactly one non-`#[test]` module-level helper**,
  `assert_parse_class_method_modifiers_are_preserved` (consumed by the class_method tests).
  The mover leaves non-test fns in place automatically (no pin arg) — same as kali_codegen's
  `legacy_phase1_baseline`. The class_method submodule reaches it via `use super::*;`.
- **No `include_*!` pins** — verified 0 across all four files (no 3rd mover arg needed).
- **No `pub`/`pub(crate)` widening** — verified 0 pub fns; all tests are private `fn`.
- The 4 product siblings (`declaration.rs`, `expression/call.rs`, `expression/mod.rs`,
  `module.rs`) keep their existing unchanged `#[cfg(test)] #[path = "F_tests.rs"] mod F_tests;`
  decls — the facade re-exports children.

## Tooling

- **Mover:** `.superpowers/sdd/move_fns.py` in exact-name-set mode. `FN_RE`/`IDENT_CHARS`/
  `find_close_line` stay **byte-identical**; only `main()`'s GROUPS assignment changes per file.
- **Verifier:** `.superpowers/sdd/verify.py` proves `{name: body}` extracted from the original
  equals that from the submodules (+ facade-retained items), exiting non-zero on any
  name-set/body mismatch — the decisive byte-identity gate.

## Gates (clean — no env carve-outs)

- `cargo build -p kali_parser --tests` → **0 warnings** (baseline = 0).
- `cargo test -p kali_parser --lib` → **65 pass / 0 fail** (baseline = 65 pass/0 fail).
- Held at baseline AND on merged main.
- Net code delta = scaffold only: 12× (`use super::*;` + blank) + 12× (`#[path]` + `mod`)
  pairs. No production change, no fmt run (the repo's `cargo fmt --check` gate is
  pre-existing-red; running fmt would violate the verbatim mandate).

## Execution & integration

- 4 SDD tasks, one per facade file. Per task: sonnet implementer → review-package → sonnet
  task review. Opus whole-branch finalize review at the end.
- verify.py re-proves 51/51 `#[test]` bodies byte-identical base→head.
- SDD ledger at `.superpowers/sdd/progress.md` (git-ignored scratch), overwritten this crate.
- **Integration: local-main ff-merge only — NEVER push origin.** Re-verify gates on merged
  main, then delete the branch.
