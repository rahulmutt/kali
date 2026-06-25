# kali_parser Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split `kali_parser`'s monolithic `lib.rs` (one `impl Parser`, 64 fns) and `tests.rs` (65 tests) into small, single-responsibility modules behind a thin facade, with zero behavior change.

**Architecture:** Impl-split. The `Parser` struct stays in one file (`parser.rs`); its methods move into sibling modules by responsibility, each carrying its own `impl Parser { … }`. `lib.rs` becomes a facade (`mod` decls + `pub use`). Tests co-locate into sibling `*_tests.rs` files wired via `#[cfg(test)] #[path] mod`, sharing a `cfg(test)` `lex()` helper.

**Tech Stack:** Rust (edition 2021), Cargo workspace, `kali_lexer`/`kali_ast`/`kali_error`/`kali_common` deps.

## Global Constraints

- **Pure structural refactor — zero behavior change.** No logic rewritten; only items relocated and visibility widened.
- **Exact same tests before and after.** `cargo test -p kali_parser -- --list` must be identical (no test added, dropped, renamed, or duplicated).
- `cargo test -p kali_parser` must be green after **every** commit.
- `lib.rs` ends as a thin facade: module declarations + `pub use` re-exports + crate-level `#![allow(dead_code)]`. Public paths (`kali_parser::Parser`, `Parser::new`, `Parser::parse`, `kali_parser::ParserOutput`, `kali_parser::TokenStream`) keep resolving unchanged.
- Unit tests live in sibling `*_tests.rs` files wired via `#[cfg(test)] #[path = "…"] mod`, per AGENTS.md — **not** inline `#[cfg(test)]` modules.
- Final commit only after `cargo fmt`, `cargo clippy -p kali_parser` clean, and `--list` baseline diff empty.

---

## File Structure

Source layout when complete (`crates/kali_parser/src/`):

```
lib.rs            facade: mod decls + pub use, crate docs/attrs
token_stream.rs   TokenStream struct + cursor impl
parser.rs         Parser struct + fields + new + parse + ParserOutput + shared helpers
statement.rs      parse_statement dispatcher + 15 statement parsers
declaration.rs    params, functions, classes, arrow functions
module.rs         import/export declarations + specifiers
expression/
  mod.rs          parse_expression, assignment, operator, unary, binary, yield, await
  call.rs         call expr, optional chain, member-access-name helpers
  primary.rs      parse_primary_expression
  object.rs       object expr + computed-property-name helpers
types.rs          parse_type_reference_text
literal.rs        unquote/normalize string + expression_to_property_name
```

Test layout when complete:

```
test_support.rs        cfg(test) shared `lex()` helper
statement_tests.rs
declaration_tests.rs
module_tests.rs
types_tests.rs
expression/
  mod_tests.rs         (expression operators/unary/binary/etc.)
  call_tests.rs
  primary_tests.rs
  object_tests.rs
```

**Method → module map** (current `lib.rs` line numbers, for relocation):

| Module | Items (current lines) |
|---|---|
| `token_stream.rs` | `TokenStream` struct (21–24) + impl: `new` 27, `current` 34, `current_kind` 38, `peek_next_kind` 42, `eof` 46, `advance` 50, `advance_if` 60, `accept` 69, `skip` 73 |
| `parser.rs` | `Parser` struct (80–87), `new` 90, `parse` 162, shared helpers `wrap_statement_as_block` 101, `push_feature_unavailable` 108, `current_token_value_is` 1225, `skip_class_body` 115; `ParserOutput` struct (2268–2272) |
| `statement.rs` | `parse_statement` 181, `parse_variable_declaration` 247, `parse_block_statement` 275, `parse_if_statement` 447, `parse_while_statement` 477, `parse_for_statement` 493, `parse_do_while_statement` 685, `parse_switch_statement` 701, `parse_break_statement` 777, `parse_continue_statement` 791, `parse_throw_statement` 805, `parse_debugger_statement` 813, `parse_try_statement` 820, `parse_return_statement` 873, `parse_expression_statement` 890 |
| `declaration.rs` | `parse_parameter_list` 138, `parse_function_declaration` 296, `parse_function_declaration_with_async` 300, `parse_class_body` 359, `parse_class_declaration` 420, `parse_class_expression` 432, `parse_function_expression` 1807, `parse_function_expression_with_async` 1811, `try_parse_arrow_function_expression` 1692, `try_parse_arrow_function_expression_from` 1696, `parse_arrow_function_body_expression` 1778 |
| `module.rs` | `parse_import_declaration` 899, `parse_export_declaration` 975, `parse_export_named_specifiers` 1072, `parse_import_named_specifiers` 1119, `parse_import_namespace_specifier` 1168 |
| `expression/mod.rs` | `parse_expression` 1189, `parse_assignment_expression` 1193, `parse_assignment_operator` 1209, `parse_unary_expression` 1231, `parse_binary_expression` 1305, `parse_yield_expression` 1394, `parse_await_expression` 1410 |
| `expression/call.rs` | `parse_call_expression` 1416, `parse_optional_chain_expression` 1658, `is_object_freeze_call` 2035, `call_member_access_name` 2051, `member_access_name` 2080 |
| `expression/primary.rs` | `parse_primary_expression` 2085 |
| `expression/object.rs` | `parse_object_expression` 1863, `unwrap_await_literal_array_expression` 1944, `computed_object_property_name` 1974 |
| `types.rs` | `parse_type_reference_text` 1592 |
| `literal.rs` | `expression_to_property_name` 1537, `normalize_string_literal` 1577, `unquote_string_literal` (free fn) 2274 |

Exact placement of a borderline helper (e.g. `is_object_freeze_call` in `call.rs` vs `object.rs`) may shift during implementation as long as it compiles and the suite stays green; the table is the target, not a frozen contract.

---

### Task 1: Baseline + widen visibility for extraction

**Files:**
- Create: `crates/kali_parser/baseline_tests.txt` (temporary, git-ignored scratch — store under the scratchpad, not the repo)
- Modify: `crates/kali_parser/src/lib.rs` (visibility only)

**Interfaces:**
- Consumes: nothing.
- Produces: a `pub(crate)` surface on `Parser` fields, `TokenStream` fields + methods, and all `impl Parser`/`impl TokenStream` methods, so sibling-module `impl` blocks compile.

- [ ] **Step 1: Capture the test-list baseline**

```bash
cargo test -p kali_parser -- --list > /tmp/claude-1000/-workspace/<session>/scratchpad/kali_parser_baseline.txt
wc -l /tmp/claude-1000/-workspace/<session>/scratchpad/kali_parser_baseline.txt
```
Expected: a non-empty list ending in a summary; 65 `: test` lines. Keep this file for the final diff.

- [ ] **Step 2: Confirm a green starting point**

Run: `cargo test -p kali_parser`
Expected: PASS, `test result: ok. 65 passed`.

- [ ] **Step 3: Widen `Parser` and `TokenStream` field visibility to `pub(crate)`**

In `crates/kali_parser/src/lib.rs`, change the struct field declarations (these fields are read/written by methods that will move to sibling modules):

```rust
pub struct TokenStream {
    pub(crate) tokens: Vec<Token>,
    pub(crate) position: usize,
}

pub struct Parser {
    pub(crate) file_id: FileId,
    pub(crate) stream: TokenStream,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) jsx_mode: bool,
    pub(crate) in_generator_function: bool,
    pub(crate) in_async_function: bool,
}
```

- [ ] **Step 4: Widen all `impl TokenStream` and `impl Parser` methods to `pub(crate)`**

For every method in both impl blocks **except** the already-`pub` `Parser::new` and `Parser::parse`, prefix the existing `fn` with `pub(crate) ` (e.g. `fn current(&self)` → `pub(crate) fn current(&self)`; `fn parse_statement(&mut self)` → `pub(crate) fn parse_statement(&mut self)`). Also widen the free fn `fn unquote_string_literal` → `pub(crate) fn unquote_string_literal`. Leave method bodies untouched. The associated (non-`self`) fns like `wrap_statement_as_block`, `is_object_freeze_call`, `member_access_name`, `call_member_access_name`, `expression_to_property_name`, `normalize_string_literal` also become `pub(crate)`.

- [ ] **Step 5: Verify still green (no relocation yet)**

Run: `cargo test -p kali_parser`
Expected: PASS, 65 passed. (`#![allow(dead_code)]` already suppresses unused-visibility warnings.)

- [ ] **Step 6: Commit**

```bash
git add crates/kali_parser/src/lib.rs
git commit -m "refactor(kali_parser): widen private items to pub(crate) for extraction [refactor]"
```

---

### Task 2: Extract `token_stream.rs`

**Files:**
- Create: `crates/kali_parser/src/token_stream.rs`
- Modify: `crates/kali_parser/src/lib.rs`

**Interfaces:**
- Consumes: `kali_lexer::{Token, TokenType}`.
- Produces: `pub struct TokenStream` and its `pub(crate)` cursor methods, re-exported from the crate root via the facade.

- [ ] **Step 1: Create `token_stream.rs` with the `TokenStream` struct + impl**

Move the `TokenStream` struct (current lib.rs 21–24) and its entire `impl TokenStream` block (26–78) into a new file. Header:

```rust
//! Token cursor over the lexer output.

use kali_lexer::{Token, TokenType};

pub struct TokenStream {
    pub(crate) tokens: Vec<Token>,
    pub(crate) position: usize,
}

impl TokenStream {
    // … moved methods verbatim (new, current, current_kind, peek_next_kind,
    //    eof, advance, advance_if, accept, skip) …
}
```

- [ ] **Step 2: Wire the module into the facade**

In `lib.rs`, delete the moved `TokenStream` struct + impl, and add near the top (after the crate attrs):

```rust
mod token_stream;
pub use token_stream::TokenStream;
```

Keep the existing `use kali_lexer::{Token, TokenType};` in `lib.rs` (still used by `Parser`).

- [ ] **Step 3: Run tests**

Run: `cargo test -p kali_parser`
Expected: PASS, 65 passed.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_parser/src/token_stream.rs crates/kali_parser/src/lib.rs
git commit -m "refactor(kali_parser): extract token_stream module [refactor]"
```

---

### Task 3: Extract `statement.rs`

**Files:**
- Create: `crates/kali_parser/src/statement.rs`
- Modify: `crates/kali_parser/src/lib.rs`

**Interfaces:**
- Consumes: `crate::Parser`, `crate::TokenStream`, AST statement types, `Token`/`TokenType`.
- Produces: `impl Parser` statement-parsing methods (still `pub(crate)`), reachable from `parse` and the expression modules unchanged.

- [ ] **Step 1: Create `statement.rs` with an `impl Parser` block**

Move the 15 statement methods listed in the map (`parse_statement` and the per-keyword parsers) into a new file. Header pattern (import only what the moved bodies reference — let the compiler errors in Step 2 drive the exact `use` list):

```rust
//! Statement parsing (`parse_statement` dispatcher + per-keyword parsers).

use crate::Parser;
use kali_ast::{ /* statement types used by moved bodies */ };
use kali_lexer::TokenType;

impl Parser {
    // … moved statement methods verbatim …
}
```

- [ ] **Step 2: Remove moved methods from `lib.rs`, add `mod statement;`**

In `lib.rs`, delete the moved methods from `impl Parser`, and add `mod statement;` with the other `mod` decls. No `pub use` needed (methods, not types). Resolve any `unused import` warnings by trimming `lib.rs`'s `kali_ast` import list for names now only used in `statement.rs`.

- [ ] **Step 3: Run tests**

Run: `cargo test -p kali_parser`
Expected: PASS, 65 passed.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_parser/src/statement.rs crates/kali_parser/src/lib.rs
git commit -m "refactor(kali_parser): extract statement module [refactor]"
```

---

### Task 4: Extract `declaration.rs`

**Files:**
- Create: `crates/kali_parser/src/declaration.rs`
- Modify: `crates/kali_parser/src/lib.rs`

**Interfaces:**
- Consumes: `crate::Parser`, AST function/class/param types.
- Produces: `impl Parser` declaration methods (functions, classes, params, arrows).

- [ ] **Step 1: Create `declaration.rs`**

Move the declaration methods from the map (params, `parse_function_declaration*`, `parse_function_expression*`, `parse_class_*`, arrow-function methods) into a new file using the same `impl Parser` scaffold and a `use crate::Parser;` + the AST types the moved bodies reference.

- [ ] **Step 2: Remove from `lib.rs`, add `mod declaration;`**

- [ ] **Step 3: Run tests**

Run: `cargo test -p kali_parser`
Expected: PASS, 65 passed.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_parser/src/declaration.rs crates/kali_parser/src/lib.rs
git commit -m "refactor(kali_parser): extract declaration module [refactor]"
```

---

### Task 5: Extract `module.rs`

**Files:**
- Create: `crates/kali_parser/src/module.rs`
- Modify: `crates/kali_parser/src/lib.rs`

**Interfaces:**
- Consumes: `crate::Parser`, AST import/export types (`ImportDeclaration`, `ExportNamedDeclaration`, `ExportSpecifier`, `ImportNamedSpecifier`, `ImportSpecifier`, …).
- Produces: `impl Parser` import/export methods.

- [ ] **Step 1: Create `module.rs`**

Move `parse_import_declaration`, `parse_export_declaration`, `parse_export_named_specifiers`, `parse_import_named_specifiers`, `parse_import_namespace_specifier` into a new file with the `impl Parser` scaffold and the import/export AST `use` list.

- [ ] **Step 2: Remove from `lib.rs`, add `mod module;`**

- [ ] **Step 3: Run tests**

Run: `cargo test -p kali_parser`
Expected: PASS, 65 passed.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_parser/src/module.rs crates/kali_parser/src/lib.rs
git commit -m "refactor(kali_parser): extract module (import/export) module [refactor]"
```

---

### Task 6: Extract `literal.rs` and `types.rs`

**Files:**
- Create: `crates/kali_parser/src/literal.rs`, `crates/kali_parser/src/types.rs`
- Modify: `crates/kali_parser/src/lib.rs`

**Interfaces:**
- Consumes: `crate::Parser` (for `parse_type_reference_text` and the `&self` string helpers), AST `Expression`/`PropertyName`.
- Produces: `pub(crate)` string helpers (`unquote_string_literal`, `normalize_string_literal`, `expression_to_property_name`) and `parse_type_reference_text`, used by expression/object/primary modules.

- [ ] **Step 1: Create `literal.rs`**

Move `expression_to_property_name`, `normalize_string_literal`, and the free fn `unquote_string_literal`. The free fn becomes a module-level `pub(crate) fn`; the two associated fns stay in an `impl Parser` block (or become free `pub(crate) fn`s if they take no `self` — match their current signatures). Header:

```rust
//! String-literal normalization and property-name helpers.

use crate::Parser; // only if any moved fn takes `&self`
use kali_ast::{Expression, PropertyName};

pub(crate) fn unquote_string_literal(value: &str) -> String {
    // … verbatim …
}

impl Parser {
    // … expression_to_property_name, normalize_string_literal (if &self) …
}
```
Update call sites that referenced the crate-root free fn `unquote_string_literal` to `crate::literal::unquote_string_literal` (or add `use crate::literal::unquote_string_literal;` where used).

- [ ] **Step 2: Create `types.rs`**

Move `parse_type_reference_text` into `types.rs` with the `impl Parser` scaffold.

- [ ] **Step 3: Remove both from `lib.rs`, add `mod literal;` and `mod types;`**

- [ ] **Step 4: Run tests**

Run: `cargo test -p kali_parser`
Expected: PASS, 65 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_parser/src/literal.rs crates/kali_parser/src/types.rs crates/kali_parser/src/lib.rs
git commit -m "refactor(kali_parser): extract literal and types modules [refactor]"
```

---

### Task 7: Extract the `expression/` subtree

**Files:**
- Create: `crates/kali_parser/src/expression/mod.rs`, `expression/call.rs`, `expression/primary.rs`, `expression/object.rs`
- Modify: `crates/kali_parser/src/lib.rs`

**Interfaces:**
- Consumes: `crate::Parser`, AST expression types, `crate::literal` helpers.
- Produces: `impl Parser` expression methods. `expression/mod.rs` declares the submodules: `mod call; mod primary; mod object;` (sibling `impl Parser` blocks need no re-export — they extend the same type).

- [ ] **Step 1: Create `expression/mod.rs`**

Move the operator/precedence methods (`parse_expression`, `parse_assignment_expression`, `parse_assignment_operator`, `parse_unary_expression`, `parse_binary_expression`, `parse_yield_expression`, `parse_await_expression`). At the top of `expression/mod.rs` declare the submodules and an `impl Parser` block:

```rust
//! Expression parsing: operator precedence + sub-parsers.

mod call;
mod object;
mod primary;

use crate::Parser;
use kali_ast::{ /* expression types used here */ };

impl Parser {
    // … moved operator methods verbatim …
}
```

- [ ] **Step 2: Create `expression/call.rs`, `expression/primary.rs`, `expression/object.rs`**

Move per the map: call/optional-chain + member-access helpers → `call.rs`; `parse_primary_expression` → `primary.rs`; object expression + computed-property-name helpers → `object.rs`. Each file: `use crate::Parser;` + its AST `use` list + one `impl Parser { … }`. Where a method calls `parse_type_reference_text` or `unquote_string_literal`, they resolve via `self.` (same type) or `crate::literal::…` respectively — adjust the one free-fn call site as in Task 6.

- [ ] **Step 3: Remove all moved methods from `lib.rs`, add `mod expression;`**

- [ ] **Step 4: Run tests**

Run: `cargo test -p kali_parser`
Expected: PASS, 65 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_parser/src/expression crates/kali_parser/src/lib.rs
git commit -m "refactor(kali_parser): extract expression subtree [refactor]"
```

---

### Task 8: Extract `parser.rs`; reduce `lib.rs` to a facade

**Files:**
- Create: `crates/kali_parser/src/parser.rs`
- Modify: `crates/kali_parser/src/lib.rs`

**Interfaces:**
- Consumes: every sibling module's `impl Parser` extensions, `crate::TokenStream`.
- Produces: `pub struct Parser`, `pub struct ParserOutput`, `Parser::new`, `Parser::parse`, and the remaining shared helpers — re-exported from the crate root.

- [ ] **Step 1: Create `parser.rs`**

Move the `Parser` struct (with its `pub(crate)` fields), `ParserOutput` struct, `Parser::new`, `Parser::parse`, and the remaining shared helpers still in `lib.rs` (`wrap_statement_as_block`, `push_feature_unavailable`, `current_token_value_is`, `skip_class_body`). Header:

```rust
//! Parser entry point: struct, constructor, top-level `parse`, shared helpers.

use crate::TokenStream;
use kali_ast::{ /* types used by new/parse/helpers */ };
use kali_common::FileId;
use kali_error::{_error_codes::e5, diagnostic::Diagnostic};
use kali_lexer::{Token, TokenType};

pub struct Parser { /* … pub(crate) fields … */ }
pub struct ParserOutput { /* … pub fields … */ }

impl Parser {
    pub fn new(file_id: FileId, tokens: Vec<Token>) -> Self { /* … */ }
    pub fn parse(&mut self, _path: Option<String>) -> ParserOutput { /* … */ }
    // … shared pub(crate) helpers …
}
```

- [ ] **Step 2: Reduce `lib.rs` to a pure facade**

`lib.rs` should now contain only crate attrs, module declarations, and re-exports:

```rust
#![allow(dead_code)]

mod declaration;
mod expression;
mod literal;
mod module;
mod parser;
mod statement;
mod token_stream;
mod types;

pub use parser::{Parser, ParserOutput};
pub use token_stream::TokenStream;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
```
Remove the now-unused top-level `use` imports from `lib.rs` (they live in the modules that need them).

- [ ] **Step 3: Run tests + clippy**

Run: `cargo test -p kali_parser && cargo clippy -p kali_parser`
Expected: PASS, 65 passed; clippy clean (no new warnings).

- [ ] **Step 4: Commit**

```bash
git add crates/kali_parser/src/parser.rs crates/kali_parser/src/lib.rs
git commit -m "refactor(kali_parser): extract parser module, reduce lib.rs to facade [refactor]"
```

---

### Task 9: Add shared `test_support` and relocate tests

**Files:**
- Create: `crates/kali_parser/src/test_support.rs`, `statement_tests.rs`, `declaration_tests.rs`, `module_tests.rs`, `types_tests.rs`, `expression/mod_tests.rs`, `expression/call_tests.rs`, `expression/primary_tests.rs`, `expression/object_tests.rs`
- Modify: each corresponding source module (`#[cfg(test)] #[path] mod` wiring); delete `crates/kali_parser/src/tests.rs`

**Interfaces:**
- Consumes: the public `Parser`/`ParserOutput` API and `kali_lexer::Lexer`.
- Produces: `pub(crate) fn lex(source: &str) -> Vec<kali_lexer::Token>` for all test modules.

- [ ] **Step 1: Create the shared test helper**

```rust
// crates/kali_parser/src/test_support.rs
//! Shared test helpers for the parser test modules.
use kali_common::FileId;
use kali_lexer::{Lexer, Token};

pub(crate) fn lex(source: &str) -> Vec<Token> {
    let lexer = Lexer::new(FileId::new(0), source.to_string());
    lexer.lex_all().tokens
}
```
Wire it in `lib.rs` under cfg(test):

```rust
#[cfg(test)]
mod test_support;
```

- [ ] **Step 2: Split `tests.rs` by cluster into sibling `*_tests.rs` files**

Read each of the 65 tests and move it (verbatim, **same name**) into the `*_tests.rs` file matching the source module it exercises (statement parsers → `statement_tests.rs`, object-literal tests → `expression/object_tests.rs`, import/export tests → `module_tests.rs`, etc.). Each `*_tests.rs` starts with:

```rust
use crate::test_support::lex;
use crate::*;            // Parser, ParserOutput, etc.
use kali_ast::{ /* types referenced by the tests in this file */ };
```
Replace the old in-file `fn lex` with the shared import. Do **not** rename any test.

- [ ] **Step 3: Wire each `*_tests.rs` into its source module**

At the bottom of each source module add, e.g. in `statement.rs`:

```rust
#[cfg(test)]
#[path = "statement_tests.rs"]
mod statement_tests;
```
For the expression submodules, the `#[path]` is relative to the submodule file (e.g. `object.rs` → `#[path = "object_tests.rs"]`). Delete the old `#[cfg(test)] #[path = "tests.rs"] mod tests;` and the file `src/tests.rs`.

- [ ] **Step 4: Run tests**

Run: `cargo test -p kali_parser`
Expected: PASS, 65 passed.

- [ ] **Step 5: Diff the test-list baseline**

```bash
cargo test -p kali_parser -- --list | sort > /tmp/.../after.txt
sort /tmp/.../kali_parser_baseline.txt > /tmp/.../before.txt
diff /tmp/.../before.txt /tmp/.../after.txt
```
Expected: **empty diff** (no test added, dropped, renamed, or duplicated). If non-empty, a test was misplaced/duplicated — fix before committing.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_parser/src
git commit -m "test(kali_parser): relocate tests into sibling modules + shared test_support [refactor]"
```

---

### Task 10: Final verification

**Files:** none (verification only).

- [ ] **Step 1: Format**

Run: `cargo fmt -p kali_parser` then `git diff --stat`
If fmt changed files, review and commit: `git commit -am "style(kali_parser): cargo fmt [refactor]"`.

- [ ] **Step 2: Clippy**

Run: `cargo clippy -p kali_parser --all-targets`
Expected: clean (no new warnings vs. baseline).

- [ ] **Step 3: Full suite + workspace build**

Run: `cargo test -p kali_parser && cargo build`
Expected: 65 passed; workspace builds (confirms no downstream crate broke on the facade).

- [ ] **Step 4: Confirm `lib.rs` is a thin facade**

Run: `wc -l crates/kali_parser/src/lib.rs`
Expected: a small file (~20 lines) of mod decls + `pub use` + cfg(test) wiring only.

---

## Self-Review

- **Spec coverage:** Source impl-split (Tasks 2–8), `expression/` subdir (Task 7), facade (Task 8), test co-location with no renames + shared `lex` (Task 9), `--list` proof obligation (Task 1 capture, Task 9 diff), fmt/clippy/green finish (Task 10), `pub(crate)` widen prep (Task 1). All spec sections mapped.
- **Placeholder scan:** No "TBD"/"handle edge cases"/"similar to". Relocation steps name exact methods + line numbers; helper/test code shown in full where created.
- **Type consistency:** `Parser`, `ParserOutput`, `TokenStream`, `lex(&str) -> Vec<Token>` used consistently across tasks; facade re-exports match struct names.
