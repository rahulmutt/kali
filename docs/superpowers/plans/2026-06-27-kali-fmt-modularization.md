# kali_fmt Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Decompose the 470-line `crates/kali_fmt/src/lib.rs` into a thin facade plus one focused per-concern module, with zero behavior change and a byte-identical public API.

**Architecture:** Pure code-motion. `lib.rs` splits into 1 sibling module (`formatter`); `lib.rs` ends as declarations + re-exports + test wiring. The `formatter` module owns the entire formatting engine: `format_source` (public entry), plus the private `BraceKind`, `Formatter` struct + impl, and `normalize_string`. The existing test suite (`tests.rs`, 2 tests) is the regression oracle and must stay green after every task. No visibility widening is needed — moving `format_source` *with* the engine keeps `Formatter` and friends private.

**Tech Stack:** Rust (edition 2021), Cargo workspace. Dependencies: `kali_common` (`FileId`), `kali_lexer` (`Lexer`, `Token`, `TokenType`).

## Global Constraints

- **Verbatim moves only.** Type/method/fn bodies are moved byte-identical (cut from the source file, paste into the new module). Do NOT retype, reformat, reorder, or "improve" any moved code. The only edits permitted are: `mod`/`use` wiring and re-export lines. No visibility widening is needed in this crate.
- **Do NOT run `cargo fmt`.** The repo's `cargo fmt --all --check` gate is already red on baseline (10+ crates). Verbatim moves may push some lines >100 cols or leave stray blank lines — these are accepted cosmetic minors, not regressions. Running fmt would violate the verbatim mandate.
- **Every task ends green:** `cargo build -p kali_fmt` with **0 warnings** and `cargo test -p kali_fmt` showing all tests passed. Remove any `use` line that goes unused as code leaves the source file (the build will flag it); add any `use` a moved item now needs.
- **Public surface stays byte-identical.** Crate-root `pub`: `format`, `format_files`, `format_source` — unchanged set, signatures, and bodies. `tests.rs` resolves `format_source` via `use super::*` exactly as before.
- **No test-import fix needed.** `tests.rs` references only the public `format_source`; the recurring `use super::*` cutoff gotcha does not trigger here.
- **Commit message convention:** `refactor(kali_fmt): <description> [refactor]`.
- **Integration:** work on branch `refactor/kali-fmt-modularization` off `main`. Local-main ff-merge only — NEVER push to origin. (Branch is created in Task 1 Step 0; the final ff-merge is Task 3.)

---

## File Structure (end state)

- `crates/kali_fmt/src/lib.rs` — thin facade: crate doc, `mod formatter;`, `pub use formatter::format_source;`, the verbatim `format` + `format_files` wrappers, `#[cfg(test)] #[path = "tests.rs"] mod tests;`.
- `crates/kali_fmt/src/formatter.rs` — `use kali_common::FileId; use kali_lexer::{Lexer, Token, TokenType};` (migrated from lib.rs), then `format_source` (pub), `BraceKind` (private), `Formatter` struct + impl (private), `normalize_string` (private).
- `crates/kali_fmt/src/tests.rs` — untouched; stays declared in the `lib.rs` facade with `#[path = "tests.rs"]` and `use super::*`.

**Source line map — `lib.rs`** (current, for verbatim cut/paste):

| Item | Lines |
|---|---|
| crate doc `//!` | 1 |
| `use kali_common::FileId;` + `use kali_lexer::{Lexer, Token, TokenType};` | 3–4 |
| `format` fn | 6–9 |
| `format_files` fn | 11–20 |
| `format_source` fn | 22–31 |
| `BraceKind` enum | 33–37 |
| `Formatter` struct | 39–48 |
| `impl Formatter` (new, run, finish, emit_*) | 50–444 |
| `normalize_string` fn | 446–466 |
| test wiring | 468–470 |

---

### Task 1: Extract `formatter.rs`

**Files:**
- Create: `crates/kali_fmt/src/formatter.rs`
- Modify: `crates/kali_fmt/src/lib.rs`

**Interfaces:**
- Consumes: nothing (leaf module). It imports `kali_common::FileId` and `kali_lexer::{Lexer, Token, TokenType}` directly — these `use` lines migrate here from `lib.rs`.
- Produces: `crate::formatter::format_source`, re-exported at crate root. `BraceKind`, `Formatter`, and `normalize_string` remain module-private.

- [ ] **Step 0: Create the work branch**

Confirm baseline green on `main`, then branch:

```bash
cargo test -p kali_fmt 2>&1 | tail -3
git checkout -b refactor/kali-fmt-modularization
```

Record the baseline test count and HEAD in `.superpowers/sdd/progress.md`. Expected baseline: 2 tests pass, build clean.

- [ ] **Step 1: Create `formatter.rs` with the migrated engine**

Create `crates/kali_fmt/src/formatter.rs` containing, in this order, the two migrated `use` lines followed by the engine items cut **verbatim** from `lib.rs`:

```rust
use kali_common::FileId;
use kali_lexer::{Lexer, Token, TokenType};

/// Format a Kali source snippet into the canonical Phase-1 style.
pub fn format_source(source: &str) -> String {
    let lexer = Lexer::new(FileId::new(0), source.to_string());
    let mut tokens = lexer.lex_all().tokens;
    tokens.retain(|token| token.kind != TokenType::Eof);

    let mut formatter = Formatter::new(tokens);
    formatter.run();
    formatter.finish()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BraceKind {
    Block,
    Object,
}

struct Formatter {
    tokens: Vec<Token>,
    output: String,
    indent: usize,
    line_start: bool,
    paren_depth: usize,
    brace_stack: Vec<BraceKind>,
    block_candidate: bool,
    prev_kind: Option<TokenType>,
}

impl Formatter {
    // <cut lines 50–444 of lib.rs verbatim — new, run, finish, emit_token,
    //  emit_comment, emit_left_brace, emit_right_brace, emit_semicolon,
    //  emit_comma, emit_symbol, emit_left_paren, emit_right_paren,
    //  emit_left_bracket, emit_right_bracket, emit_operator,
    //  emit_prefix_operator, emit_word, emit_string_literal, emit_raw,
    //  needs_space_before_word, space_if_needed, write_indent>
}

fn normalize_string(raw: &str) -> String {
    // <cut lines 446–466 of lib.rs verbatim>
}
```

The two `use` lines (`use kali_common::FileId;` and `use kali_lexer::{Lexer, Token, TokenType};`) are the only additions — they migrate from `lib.rs` (the engine needs them; the facade's wrappers do not). `format_source` stays `pub`; `BraceKind`, `Formatter`, its methods, and `normalize_string` stay private. No `pub(crate)` widening.

- [ ] **Step 2: Remove the engine from `lib.rs` and wire the module**

Delete from `lib.rs`:
- `use kali_common::FileId;` (line 3) — migrated to `formatter.rs`
- `use kali_lexer::{Lexer, Token, TokenType};` (line 4) — migrated to `formatter.rs`
- `format_source` fn (lines 22–31) — moved to `formatter.rs`
- `BraceKind` enum (lines 33–37) — moved to `formatter.rs`
- `Formatter` struct (lines 39–48) — moved to `formatter.rs`
- `impl Formatter` block (lines 50–444) — moved to `formatter.rs`
- `normalize_string` fn (lines 446–466) — moved to `formatter.rs`

The `format` fn (lines 6–9) and `format_files` fn (lines 11–20) STAY — they are the facade wrappers and call `format_source`.

Replace the deleted block with the module declaration. After the crate doc (`//! Code formatter for Kali source files.`) and before the `format` fn, add:

```rust
mod formatter;

pub use formatter::format_source;
```

The resulting `lib.rs` should read, in order: crate doc → `mod formatter;` → `pub use formatter::format_source;` → `format` fn (verbatim) → `format_files` fn (verbatim) → `#[cfg(test)] #[path = "tests.rs"] mod tests;`.

- [ ] **Step 3: Verify build + tests**

```bash
cargo build -p kali_fmt 2>&1 | tail -5 && cargo test -p kali_fmt 2>&1 | tail -3
```

Expected: 0 warnings; all 2 tests pass.

If the build flags an unused `use` in `lib.rs` (e.g., `FileId`/`Lexer`/`Token`/`TokenType` left behind), remove the offending line and re-verify. (Per Step 2 these should already be gone, but verify by the 0-warning gate.)

- [ ] **Step 4: Commit**

```bash
git add crates/kali_fmt/src/formatter.rs crates/kali_fmt/src/lib.rs
git commit -m "refactor(kali_fmt): extract formatter module [refactor]"
```

---

### Task 2: Finalize lib.rs facade + workspace verification

**Files:**
- Modify: `crates/kali_fmt/src/lib.rs` (only if cleanup is needed)

**Interfaces:**
- Consumes: the `formatter` module from Task 1.
- Produces: thin facade — crate doc, `mod` decl, `pub use` re-export, `format` + `format_files` wrappers, test wiring.

- [ ] **Step 1: Verify current state compiles and tests pass**

```bash
cargo build -p kali_fmt 2>&1 | tail -5 && cargo test -p kali_fmt 2>&1 | tail -3
```

Expected: 0 warnings; all 2 tests pass.

If tests fail, the most likely cause is a stale `use` import left in `lib.rs` after the engine moved out — but `tests.rs` references only `format_source` (which stays public via re-export), so no `#[cfg(test)] use` stubs should be needed. If a failure appears, re-check that the `pub use formatter::format_source;` line is present and that `format`/`format_files` bodies are intact and unchanged.

- [ ] **Step 2: Confirm no stray private imports remain in lib.rs**

```bash
cargo build -p kali_fmt 2>&1 | grep -i warning || echo "0 warnings"
```

At this point `lib.rs` should have NO private `use` imports (all of `kali_common`/`kali_lexer` migrated to `formatter.rs`). Verify the facade contains only: crate doc, `mod formatter;`, `pub use formatter::format_source;`, `format` fn, `format_files` fn, test wiring. If any unused-import warning remains, remove the offending line.

- [ ] **Step 3: Final verification — workspace compiles (no consumer breakage)**

```bash
cargo build -p kali_fmt 2>&1 | tail -3
cargo test -p kali_fmt 2>&1 | tail -3
cargo build 2>&1 | tail -3
```

Expected: 0 warnings on all three; all 2 tests pass; workspace compiles (no consumer breakage in `kali_cli` or any other crate that uses `kali_fmt`).

- [ ] **Step 4: Commit (only if any cleanup was needed)**

If Step 1–2 required no edits to `lib.rs`, skip this commit (Task 1's commit already produced the final facade). Otherwise:

```bash
git add crates/kali_fmt/src/lib.rs
git commit -m "refactor(kali_fmt): finalize facade [refactor]"
```

---

### Task 3: FF-merge to main and verify

- [ ] **Step 1: FF-merge to main**

```bash
git checkout main
git merge --ff-only refactor/kali-fmt-modularization
```

- [ ] **Step 2: Final verification on main**

```bash
cargo build -p kali_fmt 2>&1 | tail -3
cargo test -p kali_fmt 2>&1 | tail -3
cargo build 2>&1 | tail -3
```

Expected: 0 warnings; all 2 tests pass; workspace compiles.

- [ ] **Step 3: Delete the feature branch**

```bash
git branch -d refactor/kali-fmt-modularization
```

**Do NOT push to origin.** The series convention is local-main ff-merge only; origin/main intentionally lags.
