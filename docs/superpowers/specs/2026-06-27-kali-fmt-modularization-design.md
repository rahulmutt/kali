# kali_fmt modularization — design (19th in series)

Date: 2026-06-27
Status: approved
Crate: `kali_fmt` (19th crate in the kali workspace modularization series; kali_embed was 18th)

## Goal & invariant

Pure code-motion. Decompose the single monolith `src/lib.rs` (470 lines) into a thin facade plus one per-concern module with **zero behavior change** and a **byte-identical public API**. External consumers MUST compile unedited.

Allowed changes only: `mod` declarations, `use` wiring, and (if unavoidable) `pub(crate)` visibility widening. Item bodies are moved **verbatim**. Do **not** run `cargo fmt` (verbatim moves may push some lines over 100 columns or leave stray blank lines; the repo's `cargo fmt --all --check` gate is already red on baseline, so these are not regressions).

**This crate needs no visibility widening at all** (see "Visibility" below) — the cleanest outcome in the series so far.

## Baseline (branch base)

`cargo test -p kali_fmt`: 2 tests pass, `cargo build -p kali_fmt` clean. Record exact branch-base HEAD and test counts in the SDD ledger before starting.

## Current shape

- `src/lib.rs` (470 lines): three public fns (`format`, `format_files`, `format_source`) plus the private formatting engine — `BraceKind` enum, `Formatter` struct + 395-line `impl` (the token-emission state machine), and the `normalize_string` free helper. All engine items are currently private.
- `src/tests.rs`: co-located, declared in `lib.rs` via `#[path = "tests.rs"]`, uses `use super::*`. Tests reference **only** the public `format_source` (3 uses across 2 tests) — no private symbols.

## Approach

Extract the entire formatting engine — `format_source` **plus** `BraceKind`, `Formatter`, and `normalize_string` — into a single sibling module `formatter.rs`. The facade retains the two trivial wrappers `format` and `format_files`.

Moving `format_source` *with* the engine is the load-bearing decision: `format_source` is the sole caller of `Formatter::new` / `run` / `finish`, so once it lives in the same module as `Formatter`, those methods remain private. No `pub(crate)` widening is required; `Formatter`, `BraceKind`, and `normalize_string` stay fully encapsulated as module-internal implementation details.

## Target layout

### `formatter.rs` (~450 lines) — leaf module

```rust
use kali_common::FileId;
use kali_lexer::{Lexer, Token, TokenType};

/// Format a Kali source snippet into the canonical Phase-1 style.
pub fn format_source(source: &str) -> String {
    // <verbatim from lib.rs lines 23–31>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BraceKind { /* <verbatim 33–37> */ }   // stays private

struct Formatter { /* <verbatim 39–48> */ } // stays private

impl Formatter {
    // <verbatim 50–444> — new/run/finish stay private (only format_source calls them)
}

fn normalize_string(raw: &str) -> String {
    // <verbatim 446–466> — stays private
}
```

The two `use` lines migrate from `lib.rs` (the engine needs `FileId`, `Lexer`, `Token`, `TokenType`; the facade's wrappers need none of them).

### `lib.rs` facade (~18 lines)

```rust
//! Code formatter for Kali source files.

mod formatter;

pub use formatter::format_source;

/// Format a source file.
pub fn format(source: &str) -> Option<String> {
    Some(format_source(source))
}

/// Format multiple source snippets.
///
/// This helper is primarily used by higher-level tooling; each input string is
/// treated as source text and formatted independently.
pub fn format_files(files: &[String]) -> Vec<Result<String, ()>> {
    files
        .iter()
        .map(|source| Ok(format_source(source)))
        .collect()
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
```

`format` and `format_files` stay verbatim; they call `format_source`, which the `pub use` brings into scope at the crate root. All `kali_common` / `kali_lexer` imports leave the facade.

## Public surface (byte-identical)

Crate-root `pub` after the move: `format`, `format_files`, `format_source` — unchanged set, unchanged signatures, unchanged bodies. Consumers and `tests.rs` resolve `format_source` exactly as before.

## Visibility

Zero widening. `Formatter`, `BraceKind`, `normalize_string`, and every `Formatter` method remain private to `formatter.rs`. (Contrast with Approach B, which would have kept `format_source` in the facade and forced `Formatter` + `new`/`run`/`finish` to `pub(crate)` — rejected as unnecessary exposure of an implementation detail.)

## Cross-module dependencies (within crate)

```
formatter → kali_common::FileId, kali_lexer::{Lexer, Token, TokenType}   (leaf; no intra-crate deps)
facade    → formatter::format_source (via pub use)
```

`formatter.rs` is a pure leaf — no dependencies on other modules within the crate.

## Test import fix

**None needed.** `tests.rs` references only `format_source`, which remains public at the crate root via re-export. The recurring `use super::*` cutoff gotcha does not trigger.

## Build verification

- `cargo build -p kali_fmt` — green, 0 warnings
- `cargo test -p kali_fmt` — 2 tests pass
- `cargo build` — workspace compiles (no consumer breakage)
- `cargo clippy -p kali_fmt` — 0 new warnings

## Constraints (series conventions)

- Work on branch `refactor/kali-fmt-modularization` off `main`; baseline green before starting.
- Local-main **ff-merge only — never push to origin** (origin/main intentionally lags).
- Verbatim moves only; do **not** run `cargo fmt`.
- Every task ends green: `cargo build -p kali_fmt` at 0 warnings + `cargo test -p kali_fmt` passing.
- Commit messages: `refactor(kali_fmt): <description> [refactor]`.
- SDD ledger at `.superpowers/sdd/progress.md` (git-ignored scratch); overwrite per crate.
