# kali_lint Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Decompose the monolithic `crates/kali_lint/src/lib.rs` (879 lines) into a thin facade plus an `engine` driver module and 5 per-concern rule/fix modules (`style`, `variables`, `control_flow`, `scope`, `fixes`), and relocate `src/tests.rs` (2 end-to-end tests) into a co-located `engine_tests.rs` — zero behavior change, preserved public API.

**Architecture:** ORCHESTRATOR + RULE-METHOD-PILE on a shared `Analyzer` (the web/node/deno precedent). A private `Analyzer` mega-struct holds shared state; its `run()` drives the `check_*` rule methods, which split by lint concern into sibling modules as `impl Analyzer` blocks. Because rule methods live in different modules than `run()` and share `Analyzer`'s fields, this crate **does** need the Task-1 blanket `pub(crate)` receiver-widening. The facade re-exports the public family via `pub use engine::*;` so the 3 flat `kali_lint::Name` paths are preserved → zero consumer edits. `Analyzer`/`FixPlan` stay `pub(crate)` (internal) and are kept reachable at the crate root via a `pub(crate) use engine::{Analyzer, FixPlan};` re-export so rule modules' `crate::Analyzer` / `crate::FixPlan` imports never change.

**Tech Stack:** Rust 2021, cargo workspace. Deps (all used): `kali_ast`, `kali_common`, `kali_error`, `kali_fmt`, `kali_lexer`, `kali_parser`. std only beyond those.

## Global Constraints

- **Zero behavior change, preserved public API.** The public surface is exactly **3** flat `kali_lint::Name` paths: `lint` (fn), `lint_with_options` (fn), `LintResult` (struct, with pub fields `diagnostics: Vec<Diagnostic>`, `fixed_source: Option<String>`). No other change to what is reachable as `kali_lint::…`.
- **No changes to any consumer.** Sole external consumer: `crates/kali_cli/src/bin/kali.rs:31` — `use kali_lint::lint_with_options;`. It must compile and pass **without edits**. (`kali_error/src/lib.rs` only references `kali_lint` in a code comment — not a dependency.)
- **Facade stays logic-free** — after finalize, `lib.rs` is only the `//!` crate doc, the `mod` declarations, `pub use engine::*;`, and `pub(crate) use engine::{Analyzer, FixPlan};`.
- **Functions are interleaved in the source — cut by item name, never by absolute line range.** After each task the line numbers shift; re-locate the next item with `grep -n 'fn <name>' src/lib.rs`.
- **Visibility is the only semantic edit.** No function signatures change, no bodies change. The only edits are `mod`/`use` wiring and `pub(crate)` visibility markers.
- **`Analyzer`/`FixPlan` paths are `crate::Analyzer` / `crate::FixPlan` throughout.** During Tasks 2–6 these types physically live in `lib.rs` (crate root), so the path resolves directly. After Task 7 moves them into `engine`, the `pub(crate) use engine::{Analyzer, FixPlan};` re-export keeps the same paths valid — so **no rule-module import ever changes**.
- **Cross-module free fns / methods need `pub(crate)`:** the 9 `check_*` methods `run()` invokes, plus `collect_statements_declarations` (called by `engine::Analyzer::collect_declared_names`) and `apply_fixes` (called by `engine::lint_with_options`). All other helpers stay private to their module. (Pre-widened in Task 1 so each extraction is pure code-motion.)
- **Per-task verification:** every task ends with `cargo build -p kali_lint` **and** `cargo test -p kali_lint` green, then a commit. Mid-plan unused-import warnings on the crate-root `use` block are acceptable; that block is removed in the finalize task.
- **Test self-sufficiency:** `engine_tests.rs` begins with `use crate::*;` (re-exports `lint`/`lint_with_options`/`LintResult`) **plus** `use kali_error::_error_codes::w2;` (the tests assert on `w2::*` codes). Because `cargo build` skips `cfg(test)`, a missing test import compiles under build but fails under `cargo test` — the finalize task MUST run `cargo test -p kali_lint`.

### Item → module map (cut by name)

| module | `impl Analyzer` methods (→ `pub(crate)`) | free fns / types moved in | facade exposure |
|---|---|---|---|
| `engine` | `new`, `run`, `collect_declared_names`, `count_identifier_tokens` (stay private — same module as `run`) | `lint` (pub), `lint_with_options` (pub), `LintResult` (pub), `Analyzer` (pub(crate)), `FixPlan` (pub(crate)) | `pub use engine::*;` + `pub(crate) use engine::{Analyzer, FixPlan};` |
| `style` | `check_explicit_any`, `check_no_console`, `check_debugger`, `check_eqeqeq` | — | `mod style;` (no re-export) |
| `variables` | `check_no_var_and_prefer_const` | `walk_statement_for_var_rules` (priv), `check_variable_declaration_kind` (priv) | `mod variables;` |
| `control_flow` | `check_no_empty_and_unreachable` | `check_statement_for_empty_and_unreachable` (priv), `check_block_for_unreachable` (priv), `is_terminating_statement` (priv) | `mod control_flow;` |
| `scope` | `check_no_unused_vars`, `check_no_unused_imports`, `check_no_undef` | `collect_statements_declarations` (**pub(crate)** — called by engine), `collect_statement_declarations` (priv), `collect_block_declarations` (priv), `collect_import_ranges` (priv), `builtin_globals` (priv) | `mod scope;` |
| `fixes` | — | `apply_fixes` (**pub(crate)** — called by engine) | `mod fixes;` |

### Test → file map (2 tests)

| test file | wired from | count | tests |
|---|---|---|---|
| `engine_tests.rs` | `engine.rs` | 2 | `reports_basic_lint_issues`, `fix_mode_applies_basic_safe_rewrites` |

Both are end-to-end (`lint` / `lint_with_options`), so they co-locate with the `engine` driver. Per-rule modules get no standalone tests in this pass.

---

### Task 1: Receiver-widening + cross-module visibility pass (no moves)

The shared precondition for every rule-module extraction. Nothing moves files yet — this is a pure visibility edit inside `lib.rs`, so it stays green.

**Files:**
- Modify: `crates/kali_lint/src/lib.rs`

**Interfaces:**
- Produces (still in `lib.rs` crate root for now, reachable as `crate::…`): `pub(crate) struct Analyzer` (fields `pub(crate)`), `pub(crate) struct FixPlan` (fields `pub(crate)`), the 9 `pub(crate)` check methods, `pub(crate) fn collect_statements_declarations`, `pub(crate) fn apply_fixes`.

- [ ] **Step 1: Widen `Analyzer` and `FixPlan` (structs + all fields) to `pub(crate)`**

```rust
#[derive(Default)]
pub(crate) struct FixPlan {
    pub(crate) var_tokens: HashSet<usize>,
    pub(crate) let_to_const_tokens: HashSet<usize>,
    pub(crate) eqeqeq_tokens: HashMap<usize, &'static str>,
    pub(crate) debugger_tokens: HashSet<usize>,
    pub(crate) unused_import_ranges: Vec<(usize, usize)>,
}

pub(crate) struct Analyzer {
    pub(crate) tokens: Vec<Token>,
    pub(crate) statements: Vec<Statement>,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) fix_plan: FixPlan,
}
```

- [ ] **Step 2: Widen the 9 `check_*` methods that `run()` calls to `pub(crate)`**

In the `impl Analyzer` block, prefix these method signatures with `pub(crate)` (leave `new`, `run`, `collect_declared_names`, `count_identifier_tokens` unchanged — they stay in `engine`):

```text
pub(crate) fn check_no_var_and_prefer_const(&mut self)
pub(crate) fn check_explicit_any(&mut self)
pub(crate) fn check_no_console(&mut self)
pub(crate) fn check_no_empty_and_unreachable(&mut self)
pub(crate) fn check_debugger(&mut self)
pub(crate) fn check_eqeqeq(&mut self)
pub(crate) fn check_no_unused_vars(&mut self, declared: &HashMap<String, usize>, identifier_counts: &HashMap<String, usize>)
pub(crate) fn check_no_unused_imports(&mut self, identifier_counts: &HashMap<String, usize>)
pub(crate) fn check_no_undef(&mut self, declared: &HashMap<String, usize>)
```

- [ ] **Step 3: Widen the two cross-module free fns to `pub(crate)`**

```text
pub(crate) fn collect_statements_declarations(statements: &[Statement], counts: &mut HashMap<String, usize>)
pub(crate) fn apply_fixes(source: &str, plan: &FixPlan) -> String
```

Leave all other free fns (`collect_statement_declarations`, `collect_block_declarations`, `check_statement_for_empty_and_unreachable`, `check_block_for_unreachable`, `is_terminating_statement`, `walk_statement_for_var_rules`, `check_variable_declaration_kind`, `collect_import_ranges`, `builtin_globals`) private.

- [ ] **Step 4: Build**

Run: `cargo build -p kali_lint`
Expected: PASS, no warnings (everything is still used in-crate).

- [ ] **Step 5: Test**

Run: `cargo test -p kali_lint`
Expected: PASS — 2 tests.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_lint/src/lib.rs
git commit -m "refactor(kali_lint): pub(crate) receiver-widening pass [refactor]"
```

---

### Task 2: `fixes` module (leaf — depends only on `FixPlan`)

**Files:**
- Create: `crates/kali_lint/src/fixes.rs`
- Modify: `crates/kali_lint/src/lib.rs`

**Interfaces:**
- Consumes: `crate::FixPlan` (its `pub(crate)` fields), `kali_fmt::format_source`.
- Produces: `pub(crate) fn apply_fixes(source: &str, plan: &FixPlan) -> String` (called by `lint_with_options`, which stays in `lib.rs`/`engine`).

- [ ] **Step 1: Create `fixes.rs` and move `apply_fixes` verbatim**

Cut `apply_fixes` out of `lib.rs` by name. New file:

```rust
//! Application of the accumulated safe-fix plan to source text.

use kali_fmt::format_source;

use crate::FixPlan;

pub(crate) fn apply_fixes(source: &str, plan: &FixPlan) -> String {
    // ... body moved verbatim from lib.rs (reads plan.unused_import_ranges,
    //     plan.debugger_tokens, plan.var_tokens, plan.let_to_const_tokens,
    //     plan.eqeqeq_tokens; calls format_source)
}
```

- [ ] **Step 2: Wire the module in `lib.rs` and point the caller at it**

Add a module declaration near the top of `lib.rs` (after the crate doc / `use` block):

```rust
mod fixes;
```

`lint_with_options` (still in `lib.rs`) calls `apply_fixes`. Add to the `lib.rs` `use` block:

```rust
use crate::fixes::apply_fixes;
```

(The crate-root `use kali_fmt::format_source;` is now unused in `lib.rs` — leave it; mid-plan unused-import warnings are acceptable and the whole block is removed at finalize.)

- [ ] **Step 3: Build**

Run: `cargo build -p kali_lint`
Expected: PASS (an unused-import warning for `kali_fmt::format_source` in `lib.rs` is acceptable).

- [ ] **Step 4: Test**

Run: `cargo test -p kali_lint`
Expected: PASS — 2 tests.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_lint/src/fixes.rs crates/kali_lint/src/lib.rs
git commit -m "refactor(kali_lint): extract fixes module [refactor]"
```

---

### Task 3: `style` module (4 independent token-scan checks)

**Files:**
- Create: `crates/kali_lint/src/style.rs`
- Modify: `crates/kali_lint/src/lib.rs`

**Interfaces:**
- Consumes: `crate::Analyzer` (its `pub(crate)` fields `tokens`, `diagnostics`, `fix_plan`), `kali_lexer::TokenType`, `kali_error::{_error_codes::w2, Diagnostic}`.
- Produces: `impl Analyzer` block with `pub(crate)` methods `check_explicit_any`, `check_no_console`, `check_debugger`, `check_eqeqeq` (called by `run()`).

- [ ] **Step 1: Create `style.rs` and move the 4 methods verbatim into an `impl Analyzer` block**

Cut `check_explicit_any`, `check_no_console`, `check_debugger`, `check_eqeqeq` (already `pub(crate)` from Task 1) out of the `lib.rs` `impl Analyzer` block. New file:

```rust
//! Token-level style rules: explicit-any, no-console, debugger, eqeqeq.

use kali_error::{_error_codes::w2, Diagnostic};
use kali_lexer::TokenType;

use crate::Analyzer;

impl Analyzer {
    pub(crate) fn check_explicit_any(&mut self) {
        // ... verbatim
    }

    pub(crate) fn check_no_console(&mut self) {
        // ... verbatim
    }

    pub(crate) fn check_debugger(&mut self) {
        // ... verbatim (uses self.fix_plan.debugger_tokens)
    }

    pub(crate) fn check_eqeqeq(&mut self) {
        // ... verbatim (uses self.fix_plan.eqeqeq_tokens)
    }
}
```

> These bodies index `self.tokens` via `.iter()`/`.windows()` and push to `self.diagnostics`; they reference `TokenType` variants and `w2::{EXPLICIT_ANY, NO_CONSOLE, DEBUGGER, EQEQEQ}`. They do not name the `Token` type directly. If `cargo build` reports a missing name, add the exact import it asks for.

- [ ] **Step 2: Wire the module in `lib.rs`**

Add:

```rust
mod style;
```

(No `pub use` — `style` exposes only `pub(crate)` methods. `run()` in `lib.rs` already calls `self.check_explicit_any()` etc.; they now resolve to the `pub(crate)` methods in `style`.)

- [ ] **Step 3: Build**

Run: `cargo build -p kali_lint`
Expected: PASS (crate-root `use` warnings acceptable).

- [ ] **Step 4: Test**

Run: `cargo test -p kali_lint`
Expected: PASS — 2 tests.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_lint/src/style.rs crates/kali_lint/src/lib.rs
git commit -m "refactor(kali_lint): extract style rule module [refactor]"
```

---

### Task 4: `control_flow` module (no-empty / no-unreachable)

**Files:**
- Create: `crates/kali_lint/src/control_flow.rs`
- Modify: `crates/kali_lint/src/lib.rs`

**Interfaces:**
- Consumes: `crate::Analyzer` (fields `statements`, `diagnostics`), `kali_ast::{BlockStatement, Statement}`, `kali_error::{_error_codes::w2, Diagnostic}`.
- Produces: `impl Analyzer` block with `pub(crate) fn check_no_empty_and_unreachable`; private free fns `check_statement_for_empty_and_unreachable`, `check_block_for_unreachable`, `is_terminating_statement`.

- [ ] **Step 1: Create `control_flow.rs` and move the method + 3 free fns verbatim**

Cut `check_no_empty_and_unreachable` (method, `pub(crate)` from Task 1) and the free fns `check_statement_for_empty_and_unreachable`, `check_block_for_unreachable`, `is_terminating_statement` out of `lib.rs`. New file:

```rust
//! Control-flow rules: empty blocks (no-empty) and unreachable code (no-unreachable).

use kali_ast::{BlockStatement, Statement};
use kali_error::{_error_codes::w2, Diagnostic};

use crate::Analyzer;

impl Analyzer {
    pub(crate) fn check_no_empty_and_unreachable(&mut self) {
        // ... verbatim (iterates self.statements, pushes to self.diagnostics
        //     via check_statement_for_empty_and_unreachable)
    }
}

fn check_statement_for_empty_and_unreachable(
    statement: &Statement,
    diagnostics: &mut Vec<Diagnostic>,
) {
    // ... verbatim (recurses; constructs Statement::BlockStatement(...), uses
    //     BlockStatement, w2::{NO_EMPTY, NO_UNREACHABLE}, calls check_block_for_unreachable)
}

fn check_block_for_unreachable(block: &BlockStatement, diagnostics: &mut Vec<Diagnostic>) {
    // ... verbatim
}

fn is_terminating_statement(statement: &Statement) -> bool {
    // ... verbatim
}
```

> The 3 free fns are private to `control_flow` (only called within this module). The method body calls `check_statement_for_empty_and_unreachable(statement, &mut self.diagnostics)`.

- [ ] **Step 2: Wire the module in `lib.rs`**

```rust
mod control_flow;
```

- [ ] **Step 3: Build**

Run: `cargo build -p kali_lint`
Expected: PASS.

- [ ] **Step 4: Test**

Run: `cargo test -p kali_lint`
Expected: PASS — 2 tests.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_lint/src/control_flow.rs crates/kali_lint/src/lib.rs
git commit -m "refactor(kali_lint): extract control_flow rule module [refactor]"
```

---

### Task 5: `scope` module (unused-vars / unused-imports / no-undef + declaration collection)

**Files:**
- Create: `crates/kali_lint/src/scope.rs`
- Modify: `crates/kali_lint/src/lib.rs`

**Interfaces:**
- Consumes: `crate::Analyzer` (fields `tokens`, `diagnostics`, `fix_plan`), `std::collections::{HashMap, HashSet}`, `kali_ast::{BlockStatement, Statement}` (+ fully-qualified `kali_ast::{ForInit, ForInLefthand, ForOfLefthand}` variants), `kali_lexer::{Token, TokenType}`, `kali_error::{_error_codes::w2, Diagnostic}`.
- Produces: `impl Analyzer` block with `pub(crate)` methods `check_no_unused_vars`, `check_no_unused_imports`, `check_no_undef`; `pub(crate) fn collect_statements_declarations` (called by `engine::Analyzer::collect_declared_names`); private free fns `collect_statement_declarations`, `collect_block_declarations`, `collect_import_ranges`, `builtin_globals`.

- [ ] **Step 1: Create `scope.rs` and move the 3 methods + 5 free fns verbatim**

Cut from `lib.rs`: methods `check_no_unused_vars`, `check_no_unused_imports`, `check_no_undef` (`pub(crate)` from Task 1); free fns `collect_statements_declarations` (`pub(crate)` from Task 1), `collect_statement_declarations`, `collect_block_declarations`, `collect_import_ranges`, `builtin_globals`. New file:

```rust
//! Scope rules: unused variables, unused imports, undefined references,
//! and the AST declaration-collection used to feed them.

use std::collections::{HashMap, HashSet};

use kali_ast::{BlockStatement, Statement};
use kali_error::{_error_codes::w2, Diagnostic};
use kali_lexer::{Token, TokenType};

use crate::Analyzer;

impl Analyzer {
    pub(crate) fn check_no_unused_vars(
        &mut self,
        declared: &HashMap<String, usize>,
        identifier_counts: &HashMap<String, usize>,
    ) {
        // ... verbatim
    }

    pub(crate) fn check_no_unused_imports(&mut self, identifier_counts: &HashMap<String, usize>) {
        // ... verbatim (calls collect_import_ranges, pushes to self.fix_plan.unused_import_ranges)
    }

    pub(crate) fn check_no_undef(&mut self, declared: &HashMap<String, usize>) {
        // ... verbatim (calls builtin_globals, indexes self.tokens)
    }
}

pub(crate) fn collect_statements_declarations(
    statements: &[Statement],
    counts: &mut HashMap<String, usize>,
) {
    // ... verbatim (calls collect_statement_declarations)
}

fn collect_statement_declarations(statement: &Statement, counts: &mut HashMap<String, usize>) {
    // ... verbatim (uses fully-qualified kali_ast::ForInit / ForInLefthand / ForOfLefthand)
}

fn collect_block_declarations(block: &BlockStatement, counts: &mut HashMap<String, usize>) {
    // ... verbatim
}

fn collect_import_ranges(tokens: &[Token]) -> Vec<(usize, usize)> {
    // ... verbatim
}

fn builtin_globals() -> HashSet<&'static str> {
    // ... verbatim
}
```

> `collect_statement_declarations` references `kali_ast::ForInit::…` etc. by fully-qualified path (no extra import). `collect_import_ranges` names `Token`; `check_no_unused_imports`/`check_no_undef` use `TokenType` and `HashSet`. Keep only `pub(crate)` on `collect_statements_declarations`; the other 4 free fns are private to `scope`.

- [ ] **Step 2: Wire the module + point `collect_declared_names` at it**

Add to `lib.rs`:

```rust
mod scope;
```

`Analyzer::collect_declared_names` (still in `lib.rs`) calls `collect_statements_declarations`. Add to the `lib.rs` `use` block:

```rust
use crate::scope::collect_statements_declarations;
```

- [ ] **Step 3: Build**

Run: `cargo build -p kali_lint`
Expected: PASS (crate-root `use` warnings acceptable).

- [ ] **Step 4: Test**

Run: `cargo test -p kali_lint`
Expected: PASS — 2 tests.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_lint/src/scope.rs crates/kali_lint/src/lib.rs
git commit -m "refactor(kali_lint): extract scope rule module [refactor]"
```

---

### Task 6: `variables` module (no-var / prefer-const)

**Files:**
- Create: `crates/kali_lint/src/variables.rs`
- Modify: `crates/kali_lint/src/lib.rs`

**Interfaces:**
- Consumes: `crate::{Analyzer, FixPlan}`, `kali_ast::{BlockStatement, Statement}` (+ fully-qualified `kali_ast::{ForInit, ForInLefthand, ForOfLefthand}` variants), `kali_lexer::{Token, TokenType}`, `kali_error::{_error_codes::w2, Diagnostic}`.
- Produces: `impl Analyzer` block with `pub(crate) fn check_no_var_and_prefer_const`; private free fns `walk_statement_for_var_rules`, `check_variable_declaration_kind`.

- [ ] **Step 1: Create `variables.rs` and move the method + 2 free fns verbatim**

Cut from `lib.rs`: method `check_no_var_and_prefer_const` (`pub(crate)` from Task 1); free fns `walk_statement_for_var_rules`, `check_variable_declaration_kind`. New file:

```rust
//! Declaration-keyword rules: no-var and prefer-const.

use kali_ast::{BlockStatement, Statement};
use kali_error::{_error_codes::w2, Diagnostic};
use kali_lexer::{Token, TokenType};

use crate::{Analyzer, FixPlan};

impl Analyzer {
    pub(crate) fn check_no_var_and_prefer_const(&mut self) {
        // ... verbatim (builds let_tokens from self.tokens, walks self.statements
        //     via walk_statement_for_var_rules, mutates self.diagnostics + self.fix_plan)
    }
}

fn walk_statement_for_var_rules(
    statement: &Statement,
    tokens: &[Token],
    let_tokens: &mut [(usize, TokenType)],
    declaration_index: &mut usize,
    diagnostics: &mut Vec<Diagnostic>,
    fix_plan: &mut FixPlan,
) {
    // ... verbatim (recurses; constructs BlockStatement { body: Vec::new() } and
    //     Statement::BlockStatement(...); uses fully-qualified kali_ast::ForInit / etc.;
    //     calls check_variable_declaration_kind; mutates fix_plan.var_tokens /
    //     fix_plan.let_to_const_tokens)
}

fn check_variable_declaration_kind(
    kind: &str,
    has_initializer: bool,
    let_tokens: &mut [(usize, TokenType)],
    declaration_index: &mut usize,
    diagnostics: &mut Vec<Diagnostic>,
    fix_plan: &mut FixPlan,
) {
    // ... verbatim
}
```

> `walk_statement_for_var_rules` names `BlockStatement` unqualified (the `BlockStatement { body: Vec::new() }` literals) → keep `use kali_ast::BlockStatement`. It also takes `fix_plan: &mut FixPlan` → needs `crate::FixPlan`. Both free fns are private to `variables`.

- [ ] **Step 2: Wire the module in `lib.rs`**

```rust
mod variables;
```

- [ ] **Step 3: Build**

Run: `cargo build -p kali_lint`
Expected: PASS. At this point every `check_*` method and all rule helpers have left `lib.rs`; expect unused-import warnings on the crate-root `use` block (cleared at finalize).

- [ ] **Step 4: Test**

Run: `cargo test -p kali_lint`
Expected: PASS — 2 tests.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_lint/src/variables.rs crates/kali_lint/src/lib.rs
git commit -m "refactor(kali_lint): extract variables rule module [refactor]"
```

---

### Task 7: Finalize — extract `engine`, thin the facade, co-locate tests

**Files:**
- Create: `crates/kali_lint/src/engine.rs`
- Create: `crates/kali_lint/src/engine_tests.rs`
- Delete: `crates/kali_lint/src/tests.rs`
- Modify: `crates/kali_lint/src/lib.rs`

**Interfaces:**
- Produces: a thin `lib.rs` (doc + `mod` decls + `pub use engine::*;` + `pub(crate) use engine::{Analyzer, FixPlan};`), and an `engine` module owning the public API + the shared `Analyzer`/`FixPlan` + the driver.

- [ ] **Step 1: Create `engine.rs` and move the driver + public API + shared state verbatim**

Cut out of `lib.rs` (everything that remains except the `mod`/`use` wiring): `lint`, `lint_with_options`, `LintResult`, `FixPlan`, `Analyzer` (struct + the `impl Analyzer` block holding `new`, `run`, `collect_declared_names`, `count_identifier_tokens`). New file:

```rust
//! Lint driver: public entry points, the shared `Analyzer`/`FixPlan` state,
//! and the `run()` orchestration that sequences the rule checks.

use std::collections::{HashMap, HashSet};

use kali_ast::Statement;
use kali_common::FileId;
use kali_error::Diagnostic;
use kali_lexer::{Lexer, Token, TokenType};
use kali_parser::Parser;

use crate::fixes::apply_fixes;
use crate::scope::collect_statements_declarations;

/// Lint the given source text.
pub fn lint(source: &str) -> Vec<Diagnostic> {
    // ... verbatim
}

/// Lint the given source text and optionally apply safe fixes.
pub fn lint_with_options(source: &str, fix: bool) -> LintResult {
    // ... verbatim (constructs Lexer, Parser, Analyzer; calls apply_fixes)
}

/// Lint result with optional fixed source.
#[derive(Debug, Clone)]
pub struct LintResult {
    pub diagnostics: Vec<Diagnostic>,
    pub fixed_source: Option<String>,
}

#[derive(Default)]
pub(crate) struct FixPlan {
    pub(crate) var_tokens: HashSet<usize>,
    pub(crate) let_to_const_tokens: HashSet<usize>,
    pub(crate) eqeqeq_tokens: HashMap<usize, &'static str>,
    pub(crate) debugger_tokens: HashSet<usize>,
    pub(crate) unused_import_ranges: Vec<(usize, usize)>,
}

pub(crate) struct Analyzer {
    pub(crate) tokens: Vec<Token>,
    pub(crate) statements: Vec<Statement>,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) fix_plan: FixPlan,
}

impl Analyzer {
    fn new(_source: &str, tokens: Vec<Token>, statements: Vec<Statement>) -> Self {
        // ... verbatim
    }

    fn run(&mut self) {
        // ... verbatim — calls collect_declared_names / count_identifier_tokens
        //     then the 9 rule methods (now resolved from style/variables/control_flow/scope)
    }

    fn collect_declared_names(&self) -> HashMap<String, usize> {
        // ... verbatim (calls collect_statements_declarations)
    }

    fn count_identifier_tokens(&self) -> HashMap<String, usize> {
        // ... verbatim (uses TokenType::Identifier)
    }
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod engine_tests;
```

> `new`/`run`/`collect_declared_names`/`count_identifier_tokens` stay private (same module as `run`). `Analyzer`/`FixPlan` stay `pub(crate)`. The `impl Analyzer` blocks in `style`/`variables`/`control_flow`/`scope` resolve `Analyzer` via their existing `use crate::Analyzer;` (kept valid by Step 3's re-export).

- [ ] **Step 2: Create `engine_tests.rs` with the 2 end-to-end tests**

Move both tests out of `tests.rs` verbatim. New file:

```rust
use crate::*;
use kali_error::_error_codes::w2;

#[test]
fn reports_basic_lint_issues() {
    let diagnostics = lint("var x = 1; let y = 2; debugger; if (x == y) { }");
    assert!(diagnostics
        .iter()
        .any(|diag| diag.code == Some(w2::NO_VAR as u32)));
    assert!(diagnostics
        .iter()
        .any(|diag| diag.code == Some(w2::PREFER_CONST as u32)));
    assert!(diagnostics
        .iter()
        .any(|diag| diag.code == Some(w2::DEBUGGER as u32)));
    assert!(diagnostics
        .iter()
        .any(|diag| diag.code == Some(w2::EQEQEQ as u32)));
}

#[test]
fn fix_mode_applies_basic_safe_rewrites() {
    let result = lint_with_options("var x = 1; debugger; if (x == 1) { }", true);
    let fixed = result.fixed_source.expect("fixed source");
    assert!(fixed.contains("let x = 1;"));
    assert!(!fixed.contains("debugger"));
    assert!(fixed.contains("==="));
}
```

> `use crate::*;` exposes `lint`, `lint_with_options`, `LintResult` (re-exported from `engine` by Step 3). The original `tests.rs` used `use super::*;`, which pulled in `w2` from the crate-root `use`; since the new file does not inherit that, add the explicit `use kali_error::_error_codes::w2;`.

- [ ] **Step 3: Rewrite `lib.rs` as the thin facade and delete `tests.rs`**

Replace the entire contents of `lib.rs` with:

```rust
//! Linter for Kali source files.

mod control_flow;
mod engine;
mod fixes;
mod scope;
mod style;
mod variables;

pub use engine::*;
pub(crate) use engine::{Analyzer, FixPlan};
```

This removes the old crate-root `use` block (now entirely unused), the old `#[cfg(test)] #[path = "tests.rs"] mod tests;` wiring, and every item now living in a module. `pub use engine::*;` restores `lint`/`lint_with_options`/`LintResult` at `kali_lint::…`; `pub(crate) use engine::{Analyzer, FixPlan};` keeps the rule modules' `crate::Analyzer` / `crate::FixPlan` imports resolving. Then delete `tests.rs`.

- [ ] **Step 4: Build with zero warnings**

Run: `cargo build -p kali_lint 2>&1 | grep -c warning`
Expected: `0`.

- [ ] **Step 5: Full test**

Run: `cargo test -p kali_lint`
Expected: PASS — 2 tests (now running via `engine::engine_tests`).

- [ ] **Step 6: Commit**

```bash
git add crates/kali_lint/src/engine.rs crates/kali_lint/src/engine_tests.rs crates/kali_lint/src/lib.rs
git rm crates/kali_lint/src/tests.rs
git commit -m "refactor(kali_lint): finalize facade, extract engine, co-locate tests, delete tests.rs [refactor]"
```

---

### Task 8: Whole-workspace verification + public-API proof

**Files:** none (verification + integration only).

- [ ] **Step 1: Consumer compiles unchanged**

Run: `cargo build -p kali_cli && cargo test -p kali_cli`
Expected: PASS with **no edits** to `kali_cli` — proves `lint_with_options` (the only name it imports) is preserved.

- [ ] **Step 2: Whole-workspace green**

Run: `cargo build --workspace && cargo test --workspace`
Expected: PASS across the workspace.

- [ ] **Step 3: Basename-multiset proof (series invariant)**

Confirm the public surface is exactly the 3 flat names, and the internals never leaked to `pub`:

```bash
# exactly the 3 public items, all in engine.rs:
git grep -nE '^\s*pub (fn|struct) ' crates/kali_lint/src/engine.rs
# expect: pub fn lint, pub fn lint_with_options, pub struct LintResult  (3 lines)

# no other module exposes any `pub` item:
git grep -nE '^\s*pub (fn|struct|enum|const) ' \
  crates/kali_lint/src/{style,variables,control_flow,scope,fixes}.rs
# expect: NO output

# Analyzer / FixPlan are pub(crate), never pub:
git grep -nE 'pub struct (Analyzer|FixPlan)' crates/kali_lint/src
# expect: NO output (only `pub(crate) struct …` exists)
```

- [ ] **Step 4: Integrate to local main (no push to origin)**

```bash
git checkout main
git merge --ff-only <feature-branch>
cargo build --workspace && cargo test --workspace   # re-verify on merged main
git branch -d <feature-branch>
```

Do **not** push to origin (matches the series default for crates 2–10, 12, 13).

- [ ] **Step 5: No commit needed** — verification + integration only.

---

## Self-Review

**Spec coverage:**
- `engine` driver + 5 concern modules (`style`/`variables`/`control_flow`/`scope`/`fixes`) → Tasks 2–7. ✓
- Receiver-widening (`Analyzer`/`FixPlan` + fields → `pub(crate)`) → Task 1. ✓
- 9 `check_*` methods → `pub(crate)`; `collect_statements_declarations` + `apply_fixes` → `pub(crate)`; all other helpers private → Task 1 + per-module steps. ✓
- 3-name public API preserved via `pub use engine::*;`; `crate::Analyzer`/`FixPlan` kept via `pub(crate) use` → Task 7 Step 3; consumer unchanged → Task 8 Steps 1, 3. ✓
- Facade logic-free (doc + mods + re-exports only) → Task 7 Step 3. ✓
- 2 end-to-end tests → `engine_tests.rs`, wired from `engine.rs`, `tests.rs` deleted → Task 7 Steps 2, 3, 6. ✓
- Per-task `cargo build` + `cargo test -p kali_lint`; finalize asserts 0 warnings → every task. ✓
- Local-main-only integration → Task 8 Step 4. ✓

**Placeholder scan:** No "TBD"/"add error handling"/"similar to". Bodies are "move verbatim by name" — this is a pure code-motion refactor whose canonical content is the existing source, referenced by exact item name; reproducing ~800 lines inline would be error-prone, so each task names the exact items to cut and the exact imports/visibility to apply. The 2 test bodies (the only genuinely re-typed code) are reproduced in full in Task 7 Step 2. ✓

**Type consistency:** `Analyzer`/`FixPlan` field visibilities set in Task 1 match every `self.<field>` access in the rule modules (Tasks 3–6). The 9 `check_*` signatures in Task 1 match the `impl Analyzer` blocks they land in. `collect_statements_declarations` (pub(crate), scope) matches `engine`'s `use crate::scope::collect_statements_declarations;`; `apply_fixes` (pub(crate), fixes) matches `engine`'s `use crate::fixes::apply_fixes;`. Module path `crate::Analyzer`/`crate::FixPlan` is stable across Tasks 2–6 (root struct) and Task 7+ (re-export). Test count: 2 (engine 2). ✓

**Known cross-module subtlety (called out in-task):** `engine` depends on two sibling modules — `scope::collect_statements_declarations` and `fixes::apply_fixes` — because the driver precomputes declarations and applies fixes. Both are `pub(crate)` (Task 1) and imported by `engine` at finalize (Task 7 Step 1). The `pub(crate) use engine::{Analyzer, FixPlan};` re-export (Task 7 Step 3) is what lets every rule module keep its `use crate::Analyzer;` import unchanged after the structs move into `engine`.
