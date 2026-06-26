# kali_lexer Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Decompose the 556-line `crates/kali_lexer/src/lib.rs` into a thin facade plus nine focused modules, with zero behavior change and a byte-identical public API.

**Architecture:** Pure code-motion. The `Lexer` struct stays the public entry point; its 20 methods are split by token-category into sibling modules, each a verbatim `impl Lexer { … }` block. Data types move to `token.rs`, state+navigation to `cursor.rs`, the driver to `engine.rs`. `lib.rs` becomes declarations + re-exports only. The existing 13-test suite is the regression oracle and must stay green after every task.

**Tech Stack:** Rust (edition 2021), Cargo workspace. Dependencies: `kali_common` (`FileId`, `Span`), `kali_error` (`diagnostic::Diagnostic`, `_error_codes::e1`).

## Global Constraints

- **Verbatim moves only.** Method/type bodies are moved byte-identical (cut from `lib.rs`, paste into the new module). Do NOT retype, reformat, reorder, or "improve" any moved code. The only edits permitted are: visibility prefixes (Task 1), `mod`/`use` wiring, and the test-header `use` lines (Tasks 7 & 8).
- **Do NOT run `cargo fmt`.** The repo's `cargo fmt --all --check` gate is already red on baseline (10+ crates). Verbatim moves + the mandated `pub(crate)` prefix may push some lines >100 cols or leave stray blank lines — these are accepted cosmetic minors, not regressions. Running fmt would violate the verbatim mandate.
- **Every task ends green:** `cargo build -p kali_lexer` with **0 warnings** and `cargo test -p kali_lexer` showing **13 passed**. Remove any `use` line that goes unused as code leaves `lib.rs` (the build will flag it).
- **Public surface stays byte-identical:** exactly four `pub` types — `TokenType`, `Token`, `LexerResult`, `Lexer` — plus their existing `pub` methods. No module exposes any other `pub` item. `Lexer` fields are never bare `pub`.
- **Commit message convention:** `refactor(kali_lexer): <description> [refactor]`.
- **Integration:** work on branch `refactor/kali-lexer-modularization` off `main`. Local-main ff-merge only — NEVER push to origin. (Branch is created in Task 1 Step 0, before any commit; the final ff-merge is Task 8 Step 8.)

---

## File Structure (end state)

- `crates/kali_lexer/src/lib.rs` — thin facade: crate doc, 9 `mod` decls, `pub use` re-exports.
- `crates/kali_lexer/src/token.rs` — `TokenType` enum, `Token` + `impl Token { new }`, `LexerResult`.
- `crates/kali_lexer/src/cursor.rs` — `Lexer` struct (fields `pub(crate)`), `new`, `diagnostics`, `peek`, `nth`, `is_eof`, `span`, `emit_error`, `slice`, `skip_whitespace`.
- `crates/kali_lexer/src/engine.rs` — `lex_all`, `next_token`, `collect_token`; wires `engine_tests.rs`.
- `crates/kali_lexer/src/identifier.rs` — `lex_identifier`.
- `crates/kali_lexer/src/number.rs` — `lex_number`.
- `crates/kali_lexer/src/string.rs` — `lex_string`.
- `crates/kali_lexer/src/template.rs` — `lex_template`.
- `crates/kali_lexer/src/comment.rs` — `lex_division_or_comment`, `lex_block_comment`, `lex_line_comment`.
- `crates/kali_lexer/src/punctuation.rs` — `lex_punct`; wires `punctuation_tests.rs`.
- `crates/kali_lexer/src/engine_tests.rs` — the 12 general tests (was `tests.rs`).
- `crates/kali_lexer/src/punctuation_tests.rs` — the 1 `&&` test (was `test_and_and.rs`).

**Source line map** (current `lib.rs`, for verbatim cut/paste):

| Item | Lines |
|---|---|
| `TokenType` enum | 7–116 |
| `Token` struct | 118–123 |
| `impl Token { new }` | 125–129 |
| `LexerResult` | 131–134 |
| `Lexer` struct | 136–141 |
| `new` | 144–151 |
| `diagnostics` | 153–155 |
| `lex_all` | 157–170 |
| `next_token` | 172–178 |
| `skip_whitespace` | 180–188 |
| `collect_token` | 190–209 |
| `lex_identifier` | 211–271 |
| `lex_number` | 273–304 |
| `lex_string` | 306–342 |
| `lex_template` | 344–379 |
| `lex_division_or_comment` | 381–392 |
| `lex_block_comment` | 394–421 |
| `lex_line_comment` | 423–442 |
| `lex_punct` | 444–520 |
| `peek` | 522–524 |
| `nth` | 526–528 |
| `is_eof` | 530–532 |
| `span` | 534–536 |
| `emit_error` | 538–543 |
| `slice` | 545–547 |
| test wiring (`mod tests`, `mod test_and_and`) | 550–556 |

---

### Task 1: Visibility widening pass (in place)

Widen exactly the fields and methods that will be accessed across module boundaries, with no code moved yet. This keeps the diff reviewable in isolation.

**Files:**
- Modify: `crates/kali_lexer/src/lib.rs`

**Interfaces:**
- Produces: `pub(crate)` access to `Lexer.{source, file_id, position, diagnostics}` and to methods `peek`, `nth`, `is_eof`, `span`, `emit_error`, `slice`, `skip_whitespace`, `lex_identifier`, `lex_number`, `lex_string`, `lex_template`, `lex_division_or_comment`, `lex_punct`.
- Stays private (single-module callers only): `collect_token`, `lex_block_comment`, `lex_line_comment`.
- Stays `pub` (public API): `new`, `diagnostics`, `lex_all`, `next_token`.

- [ ] **Step 0: Create the work branch**

Confirm baseline green on `main`, then branch:

```bash
cargo test -p kali_lexer 2>&1 | tail -3   # expect: 13 passed
git checkout -b refactor/kali-lexer-modularization
```

- [ ] **Step 1: Widen the 4 struct fields**

In the `Lexer` struct (lines 136–141), prefix each field with `pub(crate)`:

```rust
pub struct Lexer {
    pub(crate) source: Vec<char>,
    pub(crate) file_id: FileId,
    pub(crate) position: usize,
    pub(crate) diagnostics: Vec<Diagnostic>,
}
```

- [ ] **Step 2: Widen the 13 cross-module methods**

Change each of these from `fn` to `pub(crate) fn` (signatures only; bodies untouched):
`skip_whitespace`, `lex_identifier`, `lex_number`, `lex_string`, `lex_template`, `lex_division_or_comment`, `lex_punct`, `peek`, `nth`, `is_eof`, `span`, `emit_error`, `slice`.

Leave `collect_token`, `lex_block_comment`, `lex_line_comment` as private `fn`. Leave `new`, `diagnostics`, `lex_all`, `next_token` as `pub fn`.

- [ ] **Step 3: Verify build + tests**

Run: `cargo build -p kali_lexer 2>&1 | tail -5 && cargo test -p kali_lexer 2>&1 | tail -3`
Expected: build with 0 warnings; `13 passed`. (Some `pub(crate)` methods may not yet be called cross-module, but private→`pub(crate)` never triggers dead-code warnings for already-used items.)

- [ ] **Step 4: Commit**

```bash
git add crates/kali_lexer/src/lib.rs
git commit -m "refactor(kali_lexer): pub(crate) receiver-widening pass [refactor]"
```

---

### Task 2: Extract `token.rs` (data types)

**Files:**
- Create: `crates/kali_lexer/src/token.rs`
- Modify: `crates/kali_lexer/src/lib.rs`

**Interfaces:**
- Consumes: nothing (leaf data module).
- Produces: `crate::token::{TokenType, Token, LexerResult}`, re-exported at crate root as `crate::{TokenType, Token, LexerResult}`.

- [ ] **Step 1: Create `token.rs` with the moved types**

Header, then the four items moved **byte-identical** from `lib.rs` (TokenType 7–116, Token 118–123, impl Token 125–129, LexerResult 131–134):

```rust
use kali_common::Span;
use kali_error::diagnostic::Diagnostic;

// <TokenType enum — verbatim from lib.rs lines 7–116>
// <Token struct — verbatim from lib.rs lines 118–123>
// <impl Token { new } — verbatim from lib.rs lines 125–129>
// <LexerResult struct — verbatim from lib.rs lines 131–134>
```

- [ ] **Step 2: Remove those types from `lib.rs` and wire the module**

Delete lines 7–134 (the four items) from `lib.rs`. Add near the top, after the crate doc:

```rust
mod token;

pub use token::{LexerResult, Token, TokenType};
```

The remaining `impl Lexer` blocks in `lib.rs` resolve `TokenType`/`Token`/`LexerResult` via this re-export. Keep `lib.rs`'s existing `use kali_common::{FileId, Span};` and `use kali_error::...` lines for now — the `Lexer` struct and driver methods still need them. If the build reports any of them unused, delete the unused one.

- [ ] **Step 3: Verify build + tests**

Run: `cargo build -p kali_lexer 2>&1 | tail -5 && cargo test -p kali_lexer 2>&1 | tail -3`
Expected: 0 warnings; `13 passed`.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_lexer/src/token.rs crates/kali_lexer/src/lib.rs
git commit -m "refactor(kali_lexer): extract token module [refactor]"
```

---

### Task 3: Extract `cursor.rs` (struct + navigation primitives)

**Files:**
- Create: `crates/kali_lexer/src/cursor.rs`
- Modify: `crates/kali_lexer/src/lib.rs`

**Interfaces:**
- Consumes: nothing from sibling modules (uses only `kali_common`/`kali_error`).
- Produces: the `Lexer` type (re-exported as `crate::Lexer`) and `pub(crate)` primitives `peek`, `nth`, `is_eof`, `span`, `emit_error`, `slice`, `skip_whitespace`, plus public `new`/`diagnostics`.

- [ ] **Step 1: Create `cursor.rs`**

Header, then verbatim moves: `Lexer` struct (136–141, now with `pub(crate)` fields from Task 1), and methods `new` (144–151), `diagnostics` (153–155), `skip_whitespace` (180–188), `peek` (522–524), `nth` (526–528), `is_eof` (530–532), `span` (534–536), `emit_error` (538–543), `slice` (545–547). Wrap the methods in a single `impl Lexer { … }` block.

```rust
use kali_common::{FileId, Span};
use kali_error::diagnostic::Diagnostic;

// <Lexer struct — verbatim, pub(crate) fields>

impl Lexer {
    // <new — verbatim>
    // <diagnostics — verbatim>
    // <skip_whitespace — verbatim, pub(crate) fn>
    // <peek — verbatim, pub(crate) fn>
    // <nth — verbatim, pub(crate) fn>
    // <is_eof — verbatim, pub(crate) fn>
    // <span — verbatim, pub(crate) fn>
    // <emit_error — verbatim, pub(crate) fn>
    // <slice — verbatim, pub(crate) fn>
}
```

- [ ] **Step 2: Remove from `lib.rs` and wire the module**

Delete the `Lexer` struct and those 9 methods from `lib.rs`. Add the module decl + re-export alongside the `token` ones:

```rust
mod cursor;

pub use cursor::Lexer;
```

Now `lib.rs` retains only the driver + remaining `lex_*` methods in `impl Lexer` blocks; they reference `Lexer` via the re-export. Remove from `lib.rs` any `use` line that the build now flags as unused (e.g. `FileId` likely moves out of use here; `Span` is still used by no remaining method — confirm via build and delete unused).

- [ ] **Step 3: Verify build + tests**

Run: `cargo build -p kali_lexer 2>&1 | tail -5 && cargo test -p kali_lexer 2>&1 | tail -3`
Expected: 0 warnings; `13 passed`.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_lexer/src/cursor.rs crates/kali_lexer/src/lib.rs
git commit -m "refactor(kali_lexer): extract cursor module [refactor]"
```

---

### Task 4: Extract `identifier.rs` + `number.rs`

**Files:**
- Create: `crates/kali_lexer/src/identifier.rs`, `crates/kali_lexer/src/number.rs`
- Modify: `crates/kali_lexer/src/lib.rs`

**Interfaces:**
- Consumes: `crate::Lexer` + its `pub(crate)` fields/primitives; `crate::token::{Token, TokenType}`.
- Produces: `pub(crate)` methods `lex_identifier`, `lex_number` (already widened in Task 1).

- [ ] **Step 1: Create `identifier.rs`**

```rust
use crate::token::{Token, TokenType};
use crate::Lexer;

impl Lexer {
    // <lex_identifier — verbatim from lib.rs lines 211–271, pub(crate) fn>
}
```

- [ ] **Step 2: Create `number.rs`**

```rust
use crate::token::{Token, TokenType};
use crate::Lexer;

impl Lexer {
    // <lex_number — verbatim from lib.rs lines 273–304, pub(crate) fn>
}
```

- [ ] **Step 3: Remove both methods from `lib.rs`, add module decls**

Delete `lex_identifier` and `lex_number` from `lib.rs`. Add `mod identifier;` and `mod number;` with the other `mod` decls (keep declarations alphabetically grouped for tidiness).

- [ ] **Step 4: Verify build + tests**

Run: `cargo build -p kali_lexer 2>&1 | tail -5 && cargo test -p kali_lexer 2>&1 | tail -3`
Expected: 0 warnings; `13 passed`.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_lexer/src/identifier.rs crates/kali_lexer/src/number.rs crates/kali_lexer/src/lib.rs
git commit -m "refactor(kali_lexer): extract identifier and number modules [refactor]"
```

---

### Task 5: Extract `string.rs` + `template.rs`

**Files:**
- Create: `crates/kali_lexer/src/string.rs`, `crates/kali_lexer/src/template.rs`
- Modify: `crates/kali_lexer/src/lib.rs`

**Interfaces:**
- Consumes: `crate::Lexer`; `crate::token::{Token, TokenType}`; `kali_error::_error_codes::e1`.
- Produces: `pub(crate)` methods `lex_string`, `lex_template`.

- [ ] **Step 1: Create `string.rs`**

```rust
use crate::token::{Token, TokenType};
use crate::Lexer;
use kali_error::_error_codes::e1;

impl Lexer {
    // <lex_string — verbatim from lib.rs lines 306–342, pub(crate) fn>
}
```

- [ ] **Step 2: Create `template.rs`**

```rust
use crate::token::{Token, TokenType};
use crate::Lexer;
use kali_error::_error_codes::e1;

impl Lexer {
    // <lex_template — verbatim from lib.rs lines 344–379, pub(crate) fn>
}
```

- [ ] **Step 3: Remove both methods from `lib.rs`, add module decls**

Delete `lex_string` and `lex_template` from `lib.rs`. Add `mod string;` and `mod template;`. If `e1` is now unused in `lib.rs`, leave it — `lex_block_comment` still uses `e1::ILLEGAL_SYMBOL` until Task 6. Delete only what the build flags.

- [ ] **Step 4: Verify build + tests**

Run: `cargo build -p kali_lexer 2>&1 | tail -5 && cargo test -p kali_lexer 2>&1 | tail -3`
Expected: 0 warnings; `13 passed`.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_lexer/src/string.rs crates/kali_lexer/src/template.rs crates/kali_lexer/src/lib.rs
git commit -m "refactor(kali_lexer): extract string and template modules [refactor]"
```

---

### Task 6: Extract `comment.rs`

**Files:**
- Create: `crates/kali_lexer/src/comment.rs`
- Modify: `crates/kali_lexer/src/lib.rs`

**Interfaces:**
- Consumes: `crate::Lexer`; `crate::token::{Token, TokenType}`; `kali_error::_error_codes::e1`.
- Produces: `pub(crate)` method `lex_division_or_comment`. The two helpers `lex_block_comment` and `lex_line_comment` stay **private** (called only within this module).

- [ ] **Step 1: Create `comment.rs`**

```rust
use crate::token::{Token, TokenType};
use crate::Lexer;
use kali_error::_error_codes::e1;

impl Lexer {
    // <lex_division_or_comment — verbatim from lib.rs lines 381–392, pub(crate) fn>
    // <lex_block_comment — verbatim from lib.rs lines 394–421, private fn>
    // <lex_line_comment — verbatim from lib.rs lines 423–442, private fn>
}
```

- [ ] **Step 2: Remove the three methods from `lib.rs`, add module decl**

Delete `lex_division_or_comment`, `lex_block_comment`, `lex_line_comment` from `lib.rs`. Add `mod comment;`. The `e1` import is now unused in `lib.rs` — delete it if the build flags it (the remaining driver methods don't use `e1`).

- [ ] **Step 3: Verify build + tests**

Run: `cargo build -p kali_lexer 2>&1 | tail -5 && cargo test -p kali_lexer 2>&1 | tail -3`
Expected: 0 warnings; `13 passed`.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_lexer/src/comment.rs crates/kali_lexer/src/lib.rs
git commit -m "refactor(kali_lexer): extract comment module [refactor]"
```

---

### Task 7: Extract `punctuation.rs` + co-locate `punctuation_tests.rs`

**Files:**
- Create: `crates/kali_lexer/src/punctuation.rs`, `crates/kali_lexer/src/punctuation_tests.rs`
- Modify: `crates/kali_lexer/src/lib.rs`
- Delete: `crates/kali_lexer/src/test_and_and.rs`

**Interfaces:**
- Consumes: `crate::Lexer`; `crate::token::{Token, TokenType}`.
- Produces: `pub(crate)` method `lex_punct`; the `&&` regression test co-located with the module it exercises.

- [ ] **Step 1: Create `punctuation.rs`**

```rust
use crate::token::{Token, TokenType};
use crate::Lexer;

impl Lexer {
    // <lex_punct — verbatim from lib.rs lines 444–520, pub(crate) fn>
}

#[cfg(test)]
#[path = "punctuation_tests.rs"]
mod punctuation_tests;
```

- [ ] **Step 2: Create `punctuation_tests.rs` from `test_and_and.rs`**

Move the single test **verbatim**, but replace the header `use super::*;` with explicit imports (after the move, `super` is the `punctuation` module, which does not re-export `FileId`):

```rust
use crate::{Lexer, TokenType};
use kali_common::FileId;

// <test_peek_and_and — verbatim body from test_and_and.rs lines 2–13>
```

- [ ] **Step 3: Remove `lex_punct` from `lib.rs`, drop old test wiring, add module decl**

Delete `lex_punct` from `lib.rs`. Add `mod punctuation;`. Delete the `#[cfg(test)] #[path = "test_and_and.rs"] mod test_and_and;` wiring (lines 554–556) from `lib.rs`. Delete the file `crates/kali_lexer/src/test_and_and.rs`.

- [ ] **Step 4: Verify build + tests**

Run: `cargo build -p kali_lexer 2>&1 | tail -5 && cargo test -p kali_lexer 2>&1 | tail -3`
Expected: 0 warnings; `13 passed` (the `test_peek_and_and` test now runs under the `punctuation::punctuation_tests` path).

- [ ] **Step 5: Commit**

```bash
git add crates/kali_lexer/src/punctuation.rs crates/kali_lexer/src/punctuation_tests.rs crates/kali_lexer/src/lib.rs
git rm crates/kali_lexer/src/test_and_and.rs
git commit -m "refactor(kali_lexer): extract punctuation module, co-locate && test [refactor]"
```

---

### Task 8: Finalize — extract `engine.rs`, thin-facade `lib.rs`, co-locate `engine_tests.rs`, verify + merge

**Files:**
- Create: `crates/kali_lexer/src/engine.rs`, `crates/kali_lexer/src/engine_tests.rs`
- Modify: `crates/kali_lexer/src/lib.rs`
- Delete: `crates/kali_lexer/src/tests.rs`

**Interfaces:**
- Consumes: `crate::Lexer` + its primitives + every `pub(crate)` `lex_*` category method.
- Produces: final facade. Public surface unchanged: `TokenType`, `Token`, `LexerResult`, `Lexer` + their `pub` methods.

- [ ] **Step 1: Create `engine.rs` with the driver + test wiring**

Move `lex_all` (157–170), `next_token` (172–178), `collect_token` (190–209) **verbatim** into a single `impl Lexer { … }` block. `collect_token` stays private; `lex_all`/`next_token` stay `pub`.

```rust
use crate::token::{LexerResult, Token, TokenType};
use crate::Lexer;

impl Lexer {
    // <lex_all — verbatim, pub fn>
    // <next_token — verbatim, pub fn>
    // <collect_token — verbatim, private fn>
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod engine_tests;
```

- [ ] **Step 2: Create `engine_tests.rs` from `tests.rs`**

Move all 12 tests **verbatim**, replacing the header `use super::*;` with explicit imports (after the move, `super` is the `engine` module). The suite uses `FileId::new`, `e1::UNTERMINATED_STRING`, and the public types:

```rust
use crate::{Lexer, TokenType};
use kali_common::FileId;
use kali_error::_error_codes::e1;

// <all 12 tests — verbatim bodies from tests.rs lines 3–118>
```

- [ ] **Step 3: Reduce `lib.rs` to the thin facade**

After removing the driver methods, `lib.rs` should contain only the crate doc, module declarations, re-exports, and the old `#[path = "tests.rs"] mod tests;` wiring — which is now removed. Replace the entire file with:

```rust
//! Tokenizer/lexer for TypeScript and JavaScript.

mod comment;
mod cursor;
mod engine;
mod identifier;
mod number;
mod punctuation;
mod string;
mod template;
mod token;

pub use cursor::Lexer;
pub use token::{LexerResult, Token, TokenType};
```

Delete the file `crates/kali_lexer/src/tests.rs`.

- [ ] **Step 4: Verify build + tests (crate)**

Run: `cargo build -p kali_lexer 2>&1 | tail -5 && cargo test -p kali_lexer 2>&1 | tail -3`
Expected: 0 warnings; `13 passed`.

- [ ] **Step 5: Public-API proof**

Run:
```bash
grep -rn "pub " crates/kali_lexer/src/ | grep -vE "pub\(crate\)" | grep -v "_tests.rs"
```
Expected: only `pub enum TokenType`, `pub struct Token`, `pub struct LexerResult`, `pub struct Lexer`, the `pub use` lines in `lib.rs`, and the existing `pub fn`/`pub` fields on those four types. No `pub` on any `lex_*`/cursor method; no bare `pub` on `Lexer` fields.

- [ ] **Step 6: Consumer + whole-workspace proof**

Run:
```bash
cargo build 2>&1 | tail -5
git diff --stat -- ':!crates/kali_lexer'
```
Expected: workspace build 0 warnings; the `git diff` over all crates except `kali_lexer` is **empty** (consumers, including the 101 `kali_lexer::Lexer::new` call sites, compile unedited).

- [ ] **Step 7: Commit**

```bash
git add crates/kali_lexer/src/engine.rs crates/kali_lexer/src/engine_tests.rs crates/kali_lexer/src/lib.rs
git rm crates/kali_lexer/src/tests.rs
git commit -m "refactor(kali_lexer): finalize facade, extract engine, co-locate tests, delete tests.rs [refactor]"
```

- [ ] **Step 8: Integrate to local main (ff-merge only, no origin push)**

```bash
git checkout main
git merge --ff-only refactor/kali-lexer-modularization
cargo build -p kali_lexer 2>&1 | tail -3 && cargo test -p kali_lexer 2>&1 | tail -3
git branch -d refactor/kali-lexer-modularization
```
Expected: ff-merge succeeds; re-verified 0 warnings + `13 passed` on merged main. Do NOT `git push`.

---

## Self-Review

**Spec coverage:**
- Module layout (10 files) → Tasks 2–8 create all 9 modules + facade. ✓
- Visibility plan (4 fields + 13 methods `pub(crate)`; `lex_block_comment`/`lex_line_comment` private; 4 types stay `pub`) → Task 1 widening + Task 6 keeps the two helpers private + Task 8 Step 5 proof. ✓
- Test co-location (`tests.rs`→`engine_tests.rs`, `test_and_and.rs`→`punctuation_tests.rs`, explicit `FileId`/`e1` imports) → Tasks 7 & 8. ✓
- Verification gates (0 warnings, 13/13, API proof, consumer diff empty) → every task + Task 8 Steps 5–6. ✓
- Integration (branch, local-main ff-merge only, delete branch) → Global Constraints + Task 8 Step 8. ✓
- No `cargo fmt` / verbatim mandate → Global Constraints. ✓

**Placeholder scan:** Code-motion plan; method bodies are intentionally referenced by verbatim source line ranges rather than re-pasted, per the verbatim mandate (re-typing risks transcription drift). New/changed code (headers, struct fields, facade, test imports, wiring) is shown in full. No TBD/TODO. ✓

**Type consistency:** `crate::Lexer` (re-exported from `cursor`), `crate::token::{Token, TokenType, LexerResult}` used consistently across all rule-module headers and the facade. Method names match `lib.rs` exactly. Test imports (`crate::{Lexer, TokenType}`, `kali_common::FileId`, `kali_error::_error_codes::e1`) match the names the moved test bodies reference. ✓
