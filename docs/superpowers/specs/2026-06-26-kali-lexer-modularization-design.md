# kali_lexer modularization — design (16th in series)

**Date:** 2026-06-26
**Crate:** `crates/kali_lexer`
**Branch:** `refactor/kali-lexer-modularization` (off `main`)
**Series:** crate-by-crate modularization via SDD; ZERO behavior change, byte-identical public API.

## Goal

Decompose the monolithic 556-line `crates/kali_lexer/src/lib.rs` into a thin facade plus
per-concern modules, with **zero behavior change** and a **byte-identical public API**.
Pure code-motion + `mod`/`use` wiring + `pub(crate)` visibility widening; method/type
bodies moved **verbatim**. All consumers must compile unedited.

## Baseline (captured 2026-06-26)

- `cargo test -p kali_lexer`: **13 passed** (12 in `tests.rs`, 1 in `test_and_and.rs`).
- `cargo build -p kali_lexer`: 0 warnings.
- Consumers reference `kali_lexer::{Lexer, Token, TokenType}` (and `LexerResult` flows
  through `Lexer::lex_all`'s return type). 101 `kali_lexer::Lexer::new` call sites across
  the workspace.

## Current shape

SINGLE-STRUCT + METHOD-PILE. One `lib.rs` containing:

- `TokenType` enum (~100 variants) — `pub`
- `Token` struct + `impl Token { new }` — `pub`
- `LexerResult` struct (`tokens`, `diagnostics`) — `pub`
- `Lexer` struct (private fields `source`, `file_id`, `position`, `diagnostics`) + a pile of
  20 methods:
  - **public (4):** `new`, `diagnostics`, `lex_all`, `next_token`
  - **private (16):** `skip_whitespace`, `collect_token`, `lex_identifier`, `lex_number`,
    `lex_string`, `lex_template`, `lex_division_or_comment`, `lex_block_comment`,
    `lex_line_comment`, `lex_punct`, `peek`, `nth`, `is_eof`, `span`, `emit_error`, `slice`
- Test wiring: `#[cfg(test)] #[path = "tests.rs"] mod tests;` and
  `#[cfg(test)] #[path = "test_and_and.rs"] mod test_and_and;`

## Target module layout

`lib.rs` (thin facade) + 9 modules:

| File | Contents |
|---|---|
| `lib.rs` | **thin facade**: crate doc, `mod` declarations, `pub use` of public types, `pub(crate) use` re-exports for sibling imports |
| `token.rs` | `TokenType` enum, `Token` struct + `impl Token { new }`, `LexerResult` struct (pure data) |
| `cursor.rs` | `Lexer` struct (fields → `pub(crate)`), `new`, `diagnostics`, + navigation primitives: `peek`, `nth`, `is_eof`, `span`, `emit_error`, `slice`, `skip_whitespace` |
| `engine.rs` | driver: `lex_all`, `next_token`, `collect_token` (dispatch) |
| `identifier.rs` | `lex_identifier` (keyword table) |
| `number.rs` | `lex_number` |
| `string.rs` | `lex_string` |
| `template.rs` | `lex_template` |
| `comment.rs` | `lex_division_or_comment`, `lex_block_comment`, `lex_line_comment` |
| `punctuation.rs` | `lex_punct` (operators/punctuators) |

Each category module is a verbatim `impl Lexer { … }` block (the kali_lint precedent).
Bodies moved **byte-identical**; only `mod`/`use` wiring and visibility prefixes change.

## Visibility plan

`Lexer` is the public API, so the type stays `pub` — but its fields and cross-module methods
widen to `pub(crate)`.

**`Lexer` struct fields → `pub(crate)`** (4): `source`, `file_id`, `position`, `diagnostics`.
Rule modules in sibling files read/mutate these directly.

**Methods → `pub(crate)`** (cross-module callers):

- Cursor primitives in `cursor.rs`, called by `engine.rs` + every rule module:
  `peek`, `nth`, `is_eof`, `span`, `emit_error`, `slice`, `skip_whitespace`
- Category entry points called by `collect_token` (in `engine.rs`):
  `lex_identifier`, `lex_number`, `lex_string`, `lex_template`, `lex_division_or_comment`,
  `lex_punct`

**Stay `pub`** (consumed API — unchanged): `Token::new`, `Lexer::new`, `diagnostics`,
`lex_all`, `next_token`; and the types `TokenType`, `Token`, `LexerResult`, `Lexer`.

**Stay private** (single-module callers): `lex_block_comment`, `lex_line_comment` — only
called within `comment.rs` by `lex_division_or_comment`.

**Facade re-exports:** `lib.rs` does `pub use token::*;` (and module globs as needed) so the
external surface `kali_lexer::{Lexer, Token, TokenType, LexerResult}` resolves
byte-identically. Rule modules `use crate::Lexer;` / `use crate::token::*;` for their `impl`
blocks. This widening is the minimal set: nothing goes `pub` that wasn't already, and
`pub(crate)` is scoped to exactly what crosses a module boundary.

## Test co-location

Mirrors kali_lint's `tests.rs → engine_tests.rs` move:

- `tests.rs` (12 general tests, full-lexer behavior) → **`engine_tests.rs`**, co-located with
  the driver, wired `#[cfg(test)] #[path = "engine_tests.rs"] mod engine_tests;` from
  `engine.rs`.
- `test_and_and.rs` (1 test, `&&` punctuation via `lex_all`) → **`punctuation_tests.rs`**,
  co-located with `punctuation.rs`, wired from there.

**Test imports:** both files currently rely on `use super::*` picking up `FileId` and `e1`
from `lib.rs`'s top-level imports. After the move, `use super::*` / `use crate::*` brings in
the public types but **not** `kali_common::FileId` or `kali_error::_error_codes::e1`. So each
co-located test file gets explicit imports added (precedent: kali_lint's `engine_tests.rs`
added `use kali_error::_error_codes::w2;`):

- `engine_tests.rs`: `use kali_common::FileId;` + `use kali_error::_error_codes::e1;`
  (uses both — `FileId::new` and `e1::UNTERMINATED_STRING`)
- `punctuation_tests.rs`: `use kali_common::FileId;` only (no `e1` usage)

Test bodies move **verbatim**; only the header `use` lines are adjusted to restore the names
`super::*` no longer transitively provides.

## Data flow (unchanged)

`Lexer::new` → `lex_all`/`next_token` → `collect_token` dispatches on first char → category
`lex_*` method → cursor primitives advance `position` over `Vec<char>` → `Token`. Errors push
into `diagnostics` via `emit_error`. The refactor only relocates *where* each method is
defined; the call graph and execution order are identical.

## Task sequence

SDD via subagent-driven-development. Per-task: implementer (sonnet) → review-package → task
reviewer (sonnet; opus for finalize/whole-branch). Ledger at `.superpowers/sdd/progress.md`
(overwrite for this crate). Each task is byte-identical moves; build+test green after each.

1. `pub(crate)` widening pass on `Lexer` (4 fields + 13 methods) — in place, no moves yet.
2. Extract `token.rs` (data types).
3. Extract `cursor.rs` (struct + navigation primitives).
4. Extract category modules: `identifier`, `number`, `string`, `template`, `comment`,
   `punctuation` (one or grouped per task).
5. Finalize: reduce `lib.rs` to thin facade, extract `engine.rs` (driver), co-locate
   `engine_tests.rs` + `punctuation_tests.rs`, delete `tests.rs` + `test_and_and.rs`.

## Verification gates

- `cargo build -p kali_lexer`: 0 warnings; `cargo test -p kali_lexer`: **13/13**.
- Whole-workspace `cargo build`: 0 warnings — proves consumers compile unedited.
- **Public-API proof:** exactly 4 `pub` types (`TokenType`/`Token`/`LexerResult`/`Lexer`) +
  their pub methods; no rule module exposes `pub`; `Lexer` fields never bare `pub`.
- **Consumer proof:** `git diff` over other crates (esp. the 101 `kali_lexer::Lexer::new`
  call sites) is empty.

## Integration

- Branch `refactor/kali-lexer-modularization` off `main`; baseline green before starting.
- **Local-main ff-merge only — NEVER push to origin** (origin/main intentionally lags).
- Re-verify on merged main, then delete the branch.

## Accepted cosmetic minors (do NOT run `cargo fmt`)

Verbatim moves + the mandated `pub(crate)` prefix may push some signatures >100 cols and
leave stray blank lines. The repo's `cargo fmt --all --check` gate is already red on baseline
(10+ crates), so these are not regressions; running fmt would violate the plan's verbatim
mandate. Same decision as prior crates in the series.
