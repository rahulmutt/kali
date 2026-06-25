# kali_parser Modularization — Design

**Date:** 2026-06-25
**Status:** Approved (design)
**Scope:** Apply the validated crate-modularization pattern (see
`2026-06-23-kali-crate-modularization-design.md`) to `kali_parser`, the largest
remaining non-deferred monolith.

## Problem

`kali_parser` is two large files:

- `src/lib.rs` — 2,295 lines: one `pub struct Parser` plus a single
  `impl Parser` of 64 functions (the recursive-descent parser), a small
  `TokenStream` cursor type, `ParserOutput`, and one free fn.
- `src/tests.rs` — 2,306 lines: 65 `#[test]` functions sharing one `lex()`
  helper.

A single ~2,180-line `impl` and a flat 2,300-line test file are hard to
navigate, review, and reason about.

## Goal & Hard Constraints

**Pure structural refactor — zero behavior change.**

- The exact same set of tests exists and passes before and after.
- `lib.rs` becomes a thin **facade** (module declarations + `pub use`
  re-exports) so every external path keeps resolving. Public API is tiny —
  `Parser`, `Parser::new`, `Parser::parse`, `ParserOutput`, `TokenStream`; the
  only symbols other crates import are `kali_parser::Parser` /
  `Parser::new`. No public API churn.
- Conform to the repo convention (AGENTS.md): unit tests live in sibling
  `*_tests.rs` files wired via `#[cfg(test)] #[path = "…"] mod`, not inline
  `#[cfg(test)]` modules.

### Proof obligation

Capture a baseline before touching code and compare after:

```
cargo test -p kali_parser -- --list   # snapshot test names → diff after refactor
cargo test -p kali_parser             # must stay green at every commit
```

The `--list` snapshot guards against silently dropping or duplicating tests
during relocation.

## Source Decomposition (impl-split)

`kali_parser` is effectively one `impl Parser`. As with the `kali_types` pilot,
the **struct definition stays in one place** and its **methods split by
responsibility**, each file carrying its own `impl Parser { … }`. No logic is
rewritten. Private methods called across the new module boundaries are widened
to `pub(crate)` (the established "blanket widen for extraction" step).

### Target layout (`kali_parser/src/`)

```
src/
  lib.rs          facade: mod decls + `pub use` (Parser, ParserOutput), crate docs
  token_stream.rs TokenStream struct + cursor impl
                  (current/current_kind/peek_next_kind/eof/advance/advance_if/accept/skip)
  parser.rs       Parser struct + fields + new + parse (entry) + shared helpers
                  (wrap_statement_as_block, push_feature_unavailable,
                   current_token_value_is, skip_class_body)
  statement.rs    parse_statement dispatcher + 15 statement parsers:
                  variable_declaration, block, if, while, for, do_while, switch,
                  break, continue, throw, debugger, try, return, expression_statement
  declaration.rs  parameter_list; function_declaration(_with_async);
                  function_expression(_with_async); class_declaration/expression/body;
                  arrow functions (try_parse_arrow_function_expression(_from),
                  parse_arrow_function_body_expression)
  module.rs       import/export declarations + named/namespace specifiers
                  (parse_import_declaration, parse_export_declaration,
                   parse_export_named_specifiers, parse_import_named_specifiers,
                   parse_import_namespace_specifier)
  expression/
    mod.rs        parse_expression, assignment_expression, assignment_operator,
                  unary_expression, binary_expression, yield_expression, await_expression
    call.rs       parse_call_expression, parse_optional_chain_expression,
                  call_member_access_name, member_access_name, is_object_freeze_call
    primary.rs    parse_primary_expression
    object.rs     parse_object_expression + computed-property-name helpers
                  (computed_object_property_name, unwrap_await_literal_array_expression)
  types.rs        parse_type_reference_text
  literal.rs      string helpers: unquote_string_literal, normalize_string_literal,
                  expression_to_property_name
```

Module groupings are derived from the existing method clusters. The exact
placement of any individual method (e.g. which helper lands in `call.rs` vs
`object.rs` vs `literal.rs`) is settled during implementation; the structure
above is the target shape, not a frozen file-by-file contract.

The `expression/` subdirectory (4 focused files of ~150–250 lines) is preferred
over a single flat ~900-line `expression.rs`, consistent with the `kali_types`
precedent.

## Test Decomposition

The 65 tests are **already meaningfully named** (`test_parse_var_declaration`,
`test_parse_object_literal_*`, `test_parse_named_export_declaration`, …), so
**no renaming is required** (unlike the `kali_types` pilot). Each test is mapped
to the source module it exercises by reading its body and moved into that
module's sibling `*_tests.rs`:

```
statement_tests.rs, declaration_tests.rs, module_tests.rs, types_tests.rs,
expression/mod_tests.rs (or expression_tests.rs), expression/object_tests.rs,
expression/call_tests.rs, expression/primary_tests.rs, …
```

wired as:

```rust
#[cfg(test)]
#[path = "statement_tests.rs"]
mod statement_tests;
```

Assertions are unchanged; only location moves. The `--list` baseline confirms no
test is lost or duplicated.

## Test Infrastructure

The shared `lex(source) -> Vec<Token>` helper is parser-local (wraps
`kali_lexer::Lexer`). It moves into a small `cfg(test)` `test_support` module
shared across the split `*_tests.rs` files. **No change to `kali_test_support`
is needed** — `lex` is specific to this crate's tests.

## Execution & Verification Rhythm

Small, reviewable commits; `cargo test -p kali_parser` green after each:

1. Capture the `cargo test -p kali_parser -- --list` baseline.
2. Widen private items to `pub(crate)` for extraction.
3. Extract source modules one functional cluster at a time, behind the facade
   (token_stream → statement → declaration → module → expression/* → types →
   literal), keeping `lib.rs` a thin facade as items move.
4. Relocate tests into matching sibling `*_tests.rs` files; introduce the shared
   `test_support` `lex` helper.
5. Final check: diff against the `--list` baseline, run `cargo fmt` and
   `cargo clippy`, confirm the full suite is green.

This crate fits the mechanical pattern cleanly and reuses the master design
directly; no new pattern decisions are expected.
