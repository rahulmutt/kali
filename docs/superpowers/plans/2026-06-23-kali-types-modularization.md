# kali_types Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Break `kali_types` (one ~7,900-line `lib.rs` + a 15,308-line `tests.rs`) into small, single-purpose modules with co-located sibling test files, shared/per-crate test helpers, and pragmatic macros — with zero behavior change.

**Architecture:** `lib.rs` becomes a thin facade (module declarations + `pub use` re-exports). The single ~7,000-line `impl TypeContext` is split across many files, each carrying its own `impl TypeContext { … }` block (legal within one crate). Tests move into sibling `*_tests.rs` files wired with `#[cfg(test)] #[path = "…"] mod`. A new `kali_test_support` dev-dependency crate holds cross-crate helpers; a per-crate `test_support` module holds kali_types-specific AST builders and macros.

**Tech Stack:** Rust 2021, Cargo workspace, `indexmap`, `serde_json`, `tempfile` (dev), existing crates `kali_ast`/`kali_common`/`kali_error`/`kali_lexer`/`kali_parser`/`kali_npm`.

## Global Constraints

- **Zero behavior change.** This is a pure structural refactor. The set of tests that exist and pass must be identical before and after (renames excepted, tracked explicitly).
- **Green at every commit.** `cargo test -p kali_types` must pass after every task. Never commit a red tree.
- **Public API preserved.** External paths such as `kali_types::TypeContext`, `kali_types::Scope`, `kali_types::ScopeType`, `kali_types::ResolutionResult`, `kali_types::ScopeRef`, `kali_types::TypeChecker` must keep resolving. The facade re-exports them with `pub use`.
- **Test convention.** Unit tests live in sibling `*tests.rs` files wired via `#[cfg(test)] #[path = "…"] mod …;` — never inline `#[cfg(test)] mod tests { … }` blocks. (AGENTS.md §5.)
- **No new runtime dependencies.** `kali_test_support` is a `dev-dependency` only.
- **Run from the worktree created in Task 1.** All paths below are relative to repo root.

---

### Task 1: Worktree, branch, and baseline test snapshot

**Files:**
- Create: `docs/superpowers/baselines/kali_types-tests-before.txt`

**Interfaces:**
- Produces: `kali_types-tests-before.txt` — the authoritative list of test names before refactor, used to diff against in Task 16.

- [ ] **Step 1: Create an isolated branch**

```bash
cd /workspace
git checkout -b refactor/kali-types-modularization
```

Expected: `Switched to a new branch 'refactor/kali-types-modularization'`

- [ ] **Step 2: Confirm the suite is green before any change**

```bash
cargo test -p kali_types 2>&1 | tail -5
```

Expected: `test result: ok.` with a nonzero count (≈372 unit tests) and no failures.

- [ ] **Step 3: Snapshot the exact set of test names**

```bash
mkdir -p docs/superpowers/baselines
cargo test -p kali_types -- --list 2>/dev/null | grep ': test$' | sed 's/: test$//' | sort > docs/superpowers/baselines/kali_types-tests-before.txt
wc -l docs/superpowers/baselines/kali_types-tests-before.txt
```

Expected: a line count matching the number of unit tests (≈372).

- [ ] **Step 4: Commit the baseline**

```bash
git add docs/superpowers/baselines/kali_types-tests-before.txt
git commit -m "chore: snapshot kali_types test baseline [refactor]"
```

---

### Task 2: Widen internal visibility to `pub(crate)` (the enabling step)

**Why first:** Once methods/fields live in sibling modules, Rust's privacy rules block cross-module access to *private* items. Promoting crate-internal items to `pub(crate)` up front turns every later extraction into pure text movement with no per-move visibility puzzles. This task changes only visibility keywords — no code moves, no behavior change.

**Files:**
- Modify: `crates/kali_types/src/lib.rs`

**Interfaces:**
- Produces: all `TypeContext` fields, all private `impl TypeContext` methods, `Scope` private fields, `StaticObjectIdentityValue` (+ its impl), and the top-level free helper fns become `pub(crate)`, so any sibling module can reference them.

- [ ] **Step 1: Promote `TypeContext` struct fields**

In `crates/kali_types/src/lib.rs`, the `pub struct TypeContext` block (starts line 401). Change each currently-private field to `pub(crate)`. The fields to change (currently no `pub`): `diagnostics`, `next_scope_id`, `next_binding_id`, `base_path`, `api_surface`, `runtime_profiles`, `sandbox_policy_attached`, `in_generator_function`, `has_generator_function`, `has_async_generator_function`, `has_generator_yield_delegation`. Leave the already-`pub` fields (`global_scope`, `scopes`, `scope_stack`, `type_env`) unchanged.

Result:
```rust
pub struct TypeContext {
    pub global_scope: Scope,
    pub scopes: IndexMap<NodeId, Scope>,
    pub scope_stack: Vec<NodeId>,
    pub type_env: IndexMap<NodeId, String>,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) next_scope_id: u32,
    pub(crate) next_binding_id: u32,
    pub(crate) base_path: Option<PathBuf>,
    pub(crate) api_surface: String,
    pub(crate) runtime_profiles: Vec<String>,
    pub(crate) sandbox_policy_attached: bool,
    pub(crate) in_generator_function: bool,
    pub(crate) has_generator_function: bool,
    pub(crate) has_async_generator_function: bool,
    pub(crate) has_generator_yield_delegation: bool,
}
```

- [ ] **Step 2: Promote the private `Scope` field**

In the `pub struct Scope` block (line 55), change `static_identity_values: IndexMap<String, StaticObjectIdentityValue>,` to `pub(crate) static_identity_values: IndexMap<String, StaticObjectIdentityValue>,`. Leave the already-`pub` fields unchanged. Also change the private `Scope` method `fn invalidate_static_binding` to `pub(crate) fn invalidate_static_binding`.

- [ ] **Step 3: Promote `StaticObjectIdentityValue` and its impl**

Change `enum StaticObjectIdentityValue {` (line 120) to `pub(crate) enum StaticObjectIdentityValue {`. Its `impl StaticObjectIdentityValue` methods that are private: promote each to `pub(crate)`.

- [ ] **Step 4: Promote private methods inside `impl TypeContext`**

Inside the `impl TypeContext` block (lines 419–7393), every method declared as `    fn ` (4-space indent, no `pub`) must become `    pub(crate) fn `. Methods already declared `    pub fn ` stay as-is. Apply mechanically:

```bash
cd /workspace
# Only lines 419..7393, only top-level impl methods (exactly 4-space indent).
awk 'NR>=419 && NR<=7393 && /^    fn /{sub(/^    fn /, "    pub(crate) fn ")} {print}' crates/kali_types/src/lib.rs > /tmp/lib_vis.rs && mv /tmp/lib_vis.rs crates/kali_types/src/lib.rs
```

Then verify nothing outside the impl was touched and the count is sane:
```bash
grep -c "    pub(crate) fn " crates/kali_types/src/lib.rs
```
Expected: ≈150 (the previously-private methods).

- [ ] **Step 5: Promote top-level free helper fns**

These top-level `fn` (not methods) are referenced by methods that will move to other modules; promote each to `pub(crate) fn`: `block_contains_yield_delegation` (line 163), `statement_contains_yield_delegation` (167), `expression_contains_yield_delegation` (311), `package_root_for_materialized_source` (7394), `reject_native_addon_package_source` (7418), `value_contains_native_addon_path` (7472), `native_addon_path` (7481), `is_ident_start` (7548), `is_ident_continue` (7552), `is_type_annotation_keyword` (7556), `is_property_name_context` (7589), `next_non_whitespace_char` (7616), `skip_quoted_annotation_segment` (7627), `parse_numeric_literal_value` (7644), `is_supported_static_ascii_char_code` (7651), `static_parse_float_ascii_integer` (7655), `static_parse_int_ascii` (7716), `builtin_globals` (7768), `node_builtin_globals` (7845), `node_builtin_specifiers` (7849), `is_node_builtin_specifier` (7870), `bind_builtin` (7875), `duplicate_binding` (7886).

(Line numbers are pre-edit references; locate each by name.)

- [ ] **Step 6: Build and test**

```bash
cargo build -p kali_types 2>&1 | tail -5
cargo test -p kali_types 2>&1 | tail -5
```

Expected: build succeeds (warnings about `pub(crate)` items that could be private are acceptable), `test result: ok.` with the same count as Task 1.

- [ ] **Step 7: Commit**

```bash
git add crates/kali_types/src/lib.rs
git commit -m "refactor(kali_types): widen internals to pub(crate) for module split [refactor]"
```

---

### Task 3: Create the `kali_test_support` crate (cross-crate helpers)

**Files:**
- Create: `crates/kali_test_support/Cargo.toml`
- Create: `crates/kali_test_support/src/lib.rs`
- Modify: `Cargo.toml` (workspace members + workspace.dependencies)
- Modify: `crates/kali_types/Cargo.toml` (dev-dependencies)

**Interfaces:**
- Produces:
  - `kali_test_support::fixtures::tempdir() -> tempfile::TempDir`
  - `kali_test_support::fixtures::write_file(dir: &std::path::Path, rel: &str, contents: &str) -> std::path::PathBuf`
  - `kali_test_support::fixtures::write_manifest(dir: &std::path::Path, json: &str) -> std::path::PathBuf` (writes `kali.json`)

- [ ] **Step 1: Create the crate manifest**

Create `crates/kali_test_support/Cargo.toml`:
```toml
[package]
name = "kali_test_support"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
tempfile = "3.10"
```

- [ ] **Step 2: Create the crate source with cross-crate fixture helpers**

Create `crates/kali_test_support/src/lib.rs`:
```rust
//! Shared test helpers reused across kali crates' test suites.
//!
//! Keep only genuinely cross-crate helpers here (filesystem fixtures,
//! manifest/process setup). Crate-specific builders belong in each crate's
//! own `test_support` module.

pub mod fixtures {
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    /// Create a throwaway temp directory for fixture files.
    pub fn tempdir() -> TempDir {
        tempfile::tempdir().expect("create tempdir")
    }

    /// Write `contents` to `dir/rel`, creating parent directories, and
    /// return the absolute path written.
    pub fn write_file(dir: &Path, rel: &str, contents: &str) -> PathBuf {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create fixture parent dirs");
        }
        std::fs::write(&path, contents).expect("write fixture file");
        path
    }

    /// Write a `kali.json` manifest into `dir` and return its path.
    pub fn write_manifest(dir: &Path, json: &str) -> PathBuf {
        write_file(dir, "kali.json", json)
    }
}
```

- [ ] **Step 3: Register the crate in the workspace**

In the root `Cargo.toml`, add `"crates/kali_test_support",` to `[workspace] members` (alphabetically near the other crates), and add to `[workspace.dependencies]`:
```toml
kali_test_support = { path = "crates/kali_test_support" }
```

- [ ] **Step 4: Add it as a dev-dependency of kali_types**

In `crates/kali_types/Cargo.toml`, under `[dev-dependencies]`, add:
```toml
kali_test_support = { workspace = true }
```

- [ ] **Step 5: Build the new crate**

```bash
cargo build -p kali_test_support 2>&1 | tail -5
cargo test -p kali_types 2>&1 | tail -5
```

Expected: both succeed; kali_types test count unchanged.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock crates/kali_test_support crates/kali_types/Cargo.toml
git commit -m "feat: add kali_test_support crate with shared fixture helpers [refactor]"
```

---

### Task 4: Add per-crate `test_support` module with AST builders and macros

**Files:**
- Create: `crates/kali_types/src/test_support.rs`
- Modify: `crates/kali_types/src/lib.rs` (declare the module under `cfg(test)`)

**Interfaces:**
- Produces (all `pub(crate)`, available to every `*_tests.rs`):
  - Builder fns migrated from the top of the current `tests.rs`: `sequence_expression(Vec<Expression>) -> Expression`, `optional_chain_global_this_math() -> Expression`, `optional_chain_global_this_math_pow() -> Expression`, and the other AST-construction helpers currently defined at the top of `tests.rs`.
  - Macro `assert_resolution!` — asserts a slice of statements resolves with the expected diagnostics (see Step 2 for exact form).
  - Macro `ident!`, `member!`, `call!` — small AST-builder DSL for the most repeated node shapes.

- [ ] **Step 1: Identify the existing local test helpers to migrate**

```bash
cd /workspace
awk '/^#\[test\]/{exit} {print}' crates/kali_types/src/tests.rs | grep -nE "^(pub )?fn " 
```
This lists every helper fn defined before the first `#[test]` (e.g. `sequence_expression`, `optional_chain_global_this_math`, `optional_chain_global_this_math_pow`, `assert_resolution`, `assert_object`, …). These are the helpers to move into `test_support.rs`.

- [ ] **Step 2: Create `test_support.rs`**

Create `crates/kali_types/src/test_support.rs`. Move the helper fns identified in Step 1 here verbatim, declaring each `pub(crate)`, and add the macros. Start the file with the imports those helpers need (copy the `use` lines from the top of `tests.rs` that they reference) plus `use super::*;`. Add the macros:

```rust
//! kali_types-specific test builders and macros (compiled under cfg(test)).
use super::*;
use kali_ast::*;

// --- builder functions (migrated from tests.rs) ---
// pub(crate) fn sequence_expression(...) -> Expression { ... }
// pub(crate) fn optional_chain_global_this_math() -> Expression { ... }
// ... (move the rest here, each made pub(crate))

/// Resolve `$stmts` in a fresh `TypeContext` and assert the produced
/// diagnostic count equals `$count`.
macro_rules! assert_resolution {
    ($stmts:expr, diagnostics: $count:expr $(,)?) => {{
        let mut ctx = $crate::TypeContext::new();
        let result = ctx.resolve_statements(&$stmts);
        assert_eq!(
            result.diagnostics.len(),
            $count,
            "unexpected diagnostics: {:?}",
            result.diagnostics
        );
        result
    }};
}
pub(crate) use assert_resolution;

/// `ident!("x")` → `Expression::Identifier("x".into())`.
macro_rules! ident {
    ($name:expr) => {
        kali_ast::Expression::Identifier($name.to_string())
    };
}
pub(crate) use ident;

/// `member!(obj, "prop")` → a `MemberExpression` wrapped in `Expression`.
macro_rules! member {
    ($obj:expr, $prop:expr) => {
        kali_ast::Expression::MemberExpression(Box::new(kali_ast::MemberExpression {
            object: $obj,
            property: $prop.to_string(),
        }))
    };
}
pub(crate) use member;

/// `call!(callee, [arg, ...])` → a `CallExpression` wrapped in `Expression`.
macro_rules! call {
    ($callee:expr, [ $($arg:expr),* $(,)? ]) => {
        kali_ast::Expression::CallExpression(Box::new(kali_ast::CallExpression {
            callee: $callee,
            arguments: vec![ $( kali_ast::ExpressionOrSpread::Expression($arg) ),* ],
        }))
    };
}
pub(crate) use call;
```

> Note: confirm `CallExpression`'s field names (`callee`, `arguments`) and the `ExpressionOrSpread` variant by reading `kali_ast` before finalizing the `call!` macro; adjust to match the actual struct. Likewise confirm `ResolutionResult`'s diagnostics accessor. If diagnostics are exposed as `result.diagnostics` use that; if via `ctx.diagnostics()` adjust the macro accordingly. Verify by reading `pub struct ResolutionResult` (line ≈113 of the pre-split lib.rs) before finalizing the macro body.

- [ ] **Step 3: Declare the module under cfg(test)**

In `crates/kali_types/src/lib.rs`, just above the `#[path = "tests.rs"] mod tests;` lines, add:
```rust
#[cfg(test)]
#[path = "test_support.rs"]
mod test_support;
```

- [ ] **Step 4: Remove the now-duplicated helpers from `tests.rs` and import from `test_support`**

At the top of `tests.rs`, delete the helper fns that were moved, and add:
```rust
use crate::test_support::*;
```
Leave all `#[test]` fns unchanged for now (they still call the same helper names, now resolved from `test_support`).

- [ ] **Step 5: Build and test**

```bash
cargo test -p kali_types 2>&1 | tail -5
```
Expected: `test result: ok.`, same count as Task 1.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_types/src/lib.rs crates/kali_types/src/test_support.rs crates/kali_types/src/tests.rs
git commit -m "refactor(kali_types): extract test_support builders + macros [refactor]"
```

---

## Source-extraction tasks (Tasks 5–13)

**Shared procedure for every source-extraction task below.** Each task creates one module file (or directory module), moves a named set of items out of `lib.rs` into it, adds the module declaration + re-exports to the facade, and proves green. The mechanical recipe is identical; only the file name and the item list change:

1. Create the new file with a `//!` doc line, the `use` header it needs (copy the relevant `use` lines from `lib.rs`; `use super::*;` is the simplest correct header for method-only modules and is acceptable), and — for modules that carry methods — an `impl TypeContext { … }` wrapper.
2. Cut the listed items (by name / line range) out of `lib.rs` and paste them inside the new file's `impl` block (methods) or at file top level (free fns / types).
3. In `lib.rs`, add `mod <name>;` (private — methods attach to `TypeContext` regardless of module path) or, for modules defining public types, `mod <name>; pub use <name>::{…};`.
4. `cargo build -p kali_types` then `cargo test -p kali_types`; both green.
5. Commit with message `refactor(kali_types): extract <module> module [refactor]`.

Because Task 2 made all cross-referenced items `pub(crate)`, methods may freely call methods that now live in other modules — Rust resolves `self.foo()` across impl blocks within the crate. **Do not change any method body.**

---

### Task 5: Extract `builtins.rs` and `package.rs`

**Files:**
- Create: `crates/kali_types/src/builtins.rs`
- Create: `crates/kali_types/src/package.rs`
- Modify: `crates/kali_types/src/lib.rs`

**Interfaces:**
- Produces: `builtins.rs` holds `StaticObjectIdentityValue` (+ impl) and the free fns `builtin_globals`, `node_builtin_globals`, `node_builtin_specifiers`, `is_node_builtin_specifier`, `bind_builtin`, `duplicate_binding`. `package.rs` holds `package_root_for_materialized_source`, `reject_native_addon_package_source`, `value_contains_native_addon_path`, `native_addon_path`.

- [ ] **Step 1:** Create `builtins.rs` with header `use super::*;` and move the items listed above into it (top-level, keeping their `pub(crate)` visibility). `StaticObjectIdentityValue` is referenced by `Scope` and resolver methods — keep it `pub(crate)`.
- [ ] **Step 2:** Create `package.rs` with header `use super::*;` (it needs `std::path::{Path, PathBuf}`, `serde_json`, `kali_error::diagnostic::Diagnostic` — `use super::*;` covers these via lib.rs re-imports; if any name is missing add an explicit `use`). Move the four package fns.
- [ ] **Step 3:** In `lib.rs` add `mod builtins;` and `mod package;`. If any moved item is referenced unqualified elsewhere in `lib.rs`, add `use builtins::*;` / `use package::*;` near the top.
- [ ] **Step 4:** `cargo build -p kali_types && cargo test -p kali_types 2>&1 | tail -5`. Expected: green, same count.
- [ ] **Step 5:** `git add -A && git commit -m "refactor(kali_types): extract builtins and package modules [refactor]"`

---

### Task 6: Extract `scope.rs`

**Files:**
- Create: `crates/kali_types/src/scope.rs`
- Modify: `crates/kali_types/src/lib.rs`

**Interfaces:**
- Produces (re-exported from facade): `ScopeType`, `Scope` (+ impl), `ScopeRef`. Move these out of `lib.rs`: `pub enum ScopeType` (line ≈42), `pub struct Scope` + `impl Scope` (≈55–112), `pub struct ScopeRef<'a>` (≈7492).

- [ ] **Step 1:** Create `scope.rs` with header:
```rust
//! Lexical scope model and resolver scope handle.
use super::*;
use indexmap::IndexMap;
use kali_ast::NodeId;
```
Move `ScopeType`, `Scope`, `impl Scope`, and `ScopeRef` here. `Scope::invalidate_static_binding` and `static_identity_values` are already `pub(crate)` (Task 2).
- [ ] **Step 2:** In `lib.rs`, add `mod scope; pub use scope::{Scope, ScopeRef, ScopeType};`. Remove the now-stale local definitions.
- [ ] **Step 3:** `cargo build -p kali_types && cargo test -p kali_types 2>&1 | tail -5`. Expected: green, same count. Confirm the public path still works:
```bash
cargo build -p kali_types 2>&1 | grep -i "error" || echo "no errors"
```
- [ ] **Step 4:** `git add -A && git commit -m "refactor(kali_types): extract scope module [refactor]"`

---

### Task 7: Extract `typecheck.rs`

**Files:**
- Create: `crates/kali_types/src/typecheck.rs`
- Modify: `crates/kali_types/src/lib.rs`

**Interfaces:**
- Produces (re-exported): `TypeChecker` (+ impl). Also moves the annotation-parsing free fns and the `check_*`/`typecheck` methods.

- [ ] **Step 1:** Create `typecheck.rs` with `use super::*;`. Move into it:
  - The free fns: `is_ident_start`, `is_ident_continue`, `is_type_annotation_keyword`, `is_property_name_context`, `next_non_whitespace_char`, `skip_quoted_annotation_segment`, `parse_numeric_literal_value`, `is_supported_static_ascii_char_code`, `static_parse_float_ascii_integer`, `static_parse_int_ascii`.
  - `pub struct TypeChecker` + `impl TypeChecker`.
  - Inside an `impl TypeContext { … }` block in this file: the methods `check_type_annotation`, `check_node`, `typecheck`, `resolve_type_annotation_text` (line ≈7245).
- [ ] **Step 2:** In `lib.rs`: `mod typecheck; pub use typecheck::TypeChecker;`.
- [ ] **Step 3:** Build + test green, same count.
- [ ] **Step 4:** `git commit -m "refactor(kali_types): extract typecheck module [refactor]"`

---

### Task 8: Create `context.rs` (struct home + construction/config/scope-management)

**Files:**
- Create: `crates/kali_types/src/context.rs`
- Modify: `crates/kali_types/src/lib.rs`

**Interfaces:**
- Produces (re-exported): `TypeContext`, `ResolutionResult`. Holds the struct definition, `impl Default`, and the lifecycle/config/scope-management methods.

- [ ] **Step 1:** Create `context.rs` with `use super::*;`. Move into it:
  - `pub struct TypeContext` (line ≈401) and `impl Default for TypeContext` (≈7485).
  - `pub struct ResolutionResult` (≈113).
  - An `impl TypeContext { … }` block containing: `new`, all `with_*` constructors, `api_surface`, `set_api_surface`, `set_runtime_profiles`, `set_sandbox_policy_attached`, `has_threaded_runtime_profile`, `push_scope`, `pop_scope`, `push_block_scope`, `push_function_scope`, `is_defined`, `define`, `diagnostics`, `drain_diagnostics`, `clear_diagnostics`, `resolve_name`, `next_binding_id`, `current_scope_id`, `scope_mut`, `bind_current_scope`, `bind_in_scope`, `variable_binding_scope`, `bind_function_params`, `bind_name_list`, `bind_type_params`.
- [ ] **Step 2:** In `lib.rs`: `mod context; pub use context::{ResolutionResult, TypeContext};`.
- [ ] **Step 3:** Build + test green, same count.
- [ ] **Step 4:** `git commit -m "refactor(kali_types): extract context module [refactor]"`

---

### Task 9: Extract `resolve/` (statement, expression, call, member, function, jsx)

**Files:**
- Create: `crates/kali_types/src/resolve/mod.rs`
- Create: `crates/kali_types/src/resolve/expression.rs`
- Create: `crates/kali_types/src/resolve/call.rs`
- Create: `crates/kali_types/src/resolve/member.rs`
- Create: `crates/kali_types/src/resolve/function.rs`
- Create: `crates/kali_types/src/resolve/jsx.rs`
- Modify: `crates/kali_types/src/lib.rs`

**Interfaces:**
- Produces: no new public types; all are `impl TypeContext` method clusters. Each submodule needs `use crate::*;` at top.

- [ ] **Step 1:** Create `resolve/mod.rs` with `use crate::*;` and submodule declarations:
```rust
//! Statement and expression resolution.
use crate::*;
mod expression;
mod call;
mod member;
mod function;
mod jsx;
```
Put the **statement-level** methods in an `impl TypeContext` here: `resolve_statements`, `resolve_statements_at_path`, `resolve_statements_in_file`, `resolve_statement_list`, `resolve_statement`, `resolve_block_statement`, `resolve_block_body`, `resolve_loop_body`, `resolve_switch_cases`, `resolve_variable_declaration`, `resolve_import_declaration`, `resolve_export_all`, `resolve_export_named`, `resolve_export_default`, `record_generator_function_lowering`, `emit_pending_generator_function_lowering_diagnostic`, plus the free fns `block_contains_yield_delegation`, `statement_contains_yield_delegation`, `expression_contains_yield_delegation` (move these three free fns into `resolve/mod.rs`).
- [ ] **Step 2:** `resolve/expression.rs` (`use crate::*;`, `impl TypeContext`): `resolve_expression`, `resolve_update_expression`, `resolve_update_binding_name`, `invalidate_static_binding`, `binding_is_mutable`, `resolve_identifier`, `resolve_optional_chain`, `resolve_template_literal`, `resolve_object_property`, `resolve_property_name`, `resolve_type_assertion`, `resolve_satisfies_expression`, `is_simple_for_of_binding_expression`, `is_simple_update_target_expression`, `resolve_import_expression`, `resolve_static_import_source`, `normalize_import_segment`, `resolve_relative_import_source`, `resolve_directory_index_candidate`, `resolve_import_source`.
- [ ] **Step 3:** `resolve/call.rs` (`use crate::*;`, `impl TypeContext`): `resolve_call_expression`, `call_member_access_name`, `unwrap_static_callable_expression`, `resolve_static_callable_name`, `contains_optional_chain`, `is_supported_static_callable_member_expression`, `is_supported_static_callable_member_name`.
- [ ] **Step 4:** `resolve/member.rs` (`use crate::*;`, `impl TypeContext`): `resolve_member_expression`, `member_access_name`, `is_runtime_args_slice_member`, `member_access_name_bracketed`, `member_access_name_single_quoted`, `member_access_single_quoted_root_name`, `member_access_bracketed_root_name`, `member_access_root_name`, `member_object_name`.
- [ ] **Step 5:** `resolve/function.rs` (`use crate::*;`, `impl TypeContext`): `resolve_function_expression`, `resolve_arrow_function`, `resolve_class_expression`, `resolve_class_body`.
- [ ] **Step 6:** `resolve/jsx.rs` (`use crate::*;`, `impl TypeContext`): `resolve_jsx_element`, `resolve_jsx_fragment`, `resolve_jsx_child`.
- [ ] **Step 7:** In `lib.rs`: `mod resolve;`. Build + test green, same count after each file is moved (commit may batch the whole `resolve/` directory).
- [ ] **Step 8:** `git commit -m "refactor(kali_types): extract resolve module tree [refactor]"`

---

### Task 10: Extract `static_analysis/array.rs` and `static_analysis/string.rs`

**Files:**
- Create: `crates/kali_types/src/static_analysis/mod.rs`
- Create: `crates/kali_types/src/static_analysis/array.rs`
- Create: `crates/kali_types/src/static_analysis/string.rs`
- Modify: `crates/kali_types/src/lib.rs`

**Interfaces:**
- Produces: `impl TypeContext` clusters; no new public types.

- [ ] **Step 1:** Create `static_analysis/mod.rs`:
```rust
//! Static-value analysis used during resolution.
use crate::*;
mod array;
mod string;
```
- [ ] **Step 2:** `static_analysis/array.rs` (`use crate::*;`, `impl TypeContext`): `is_static_array_iteration_target`, `is_static_literal_array_receiver`, `is_static_truthy_array_literal`, `is_static_non_empty_numeric_array_iteration_target`, `is_static_identity_array_filter_call`, `is_static_predicate_array_filter_call`, `is_static_identity_array_flat_map_call`, `is_static_array_from_call`, `is_static_identity_array_map_call`, `is_static_set_constructor_iteration_target`, `is_static_map_constructor_iteration_target`, `is_static_object_enumeration_iteration_target`, `is_static_array_iteration_element`, `unwrap_for_of_wrapper_expression`, `resolve_array_callback_member_call`, `resolve_array_slice_member_call`, `resolve_array_concat_member_call`, `resolve_array_at_member_call`, `resolve_array_join_member_call`, `resolve_array_to_string_member_call`, `resolve_array_search_member_call`, `resolve_array_is_array_call`, `resolve_static_array_is_array_argument`, `is_static_array_concat_receiver`, `is_identity_array_callback`, `is_identity_array_callback_expression`, `is_some_every_array_callback`, `is_some_every_array_callback_expression`, `is_some_every_array_callback_identity_operand`, `is_some_every_array_callback_operand`, `is_numeric_reducer_callback`, `is_numeric_reducer_callback_expression`, `is_static_numeric_literal_expr`, `is_identity_array_flat_map_callback`, `is_identity_array_flat_map_callback_expression`, `resolve_static_array_binding_name`.
- [ ] **Step 3:** `static_analysis/string.rs` (`use crate::*;`, `impl TypeContext`): `resolve_static_string_iterable_expression`, `resolve_static_string_expression`, `resolve_static_string_from_char_code_expression`, `resolve_static_string_normalize_expression`, `resolve_static_string_binding`, `resolve_static_string_from_source`, `resolve_string_search_member_call`, `resolve_string_slice_member_call`, `resolve_string_substring_member_call`, `resolve_string_repeat_member_call`, `resolve_static_string_concat_expression`, `resolve_string_concat_member_call`, `resolve_string_pad_member_call`, `resolve_string_at_member_call`, `resolve_string_char_at_member_call`, `resolve_string_char_code_at_member_call`, `resolve_string_code_point_at_member_call`, `resolve_string_trim_member_call`, `resolve_string_case_member_call`, `resolve_string_normalize_member_call`, `resolve_string_replace_member_call`, `resolve_string_split_member_call`, `resolve_string_from_char_code_call`, `is_string_from_char_code_callable_name`, `static_ascii_string_constructor_method`.
- [ ] **Step 4:** In `lib.rs`: `mod static_analysis;`. Build + test green, same count.
- [ ] **Step 5:** `git commit -m "refactor(kali_types): extract static_analysis array+string [refactor]"`

---

### Task 11: Extract `static_analysis/object.rs`, `math.rs`, `number.rs`, `promise.rs`

**Files:**
- Create: `crates/kali_types/src/static_analysis/object.rs`
- Create: `crates/kali_types/src/static_analysis/math.rs`
- Create: `crates/kali_types/src/static_analysis/number.rs`
- Create: `crates/kali_types/src/static_analysis/promise.rs`
- Modify: `crates/kali_types/src/static_analysis/mod.rs`

**Interfaces:**
- Produces: `impl TypeContext` clusters.

- [ ] **Step 1:** `static_analysis/object.rs` (`use crate::*;`, `impl TypeContext`): `resolve_static_object_identity_binding`, `resolve_static_object_identity_reference_name`, `resolve_static_object_identity_literal_value`, `resolve_static_object_binding_name`, `resolve_static_reference_binding_name`, `resolve_static_reference_root`, `resolve_static_object_keys_binding_name`, `resolve_static_object_model_target`, `resolve_static_object_keys_target`, `is_object_freeze_call`, `is_reflect_own_keys_call`, `resolve_static_object_from_entries_call`, `resolve_static_from_entries_entries`, `resolve_static_object_model_call`, `resolve_static_object_identity_call`, `resolve_static_object_model_call_target`, `resolve_frozen_late_object_model_call`, `resolve_frozen_late_object_model_name`.
- [ ] **Step 2:** `static_analysis/math.rs` (`use crate::*;`, `impl TypeContext`): `resolve_math_member_call`, `resolve_static_numeric_literal_value`, `resolve_math_round_like_static_literal_value`, `resolve_math_extrema_static_literal_value`, `resolve_math_inverse_hyperbolic_constant_value`, `resolve_math_hyperbolic_zero_constant_value`, `resolve_math_sqrt_static_literal_root`, `resolve_math_cbrt_static_literal_root`, `resolve_math_log2_static_literal_exponent`, `resolve_math_log10_static_literal_exponent`, `resolve_math_hypot_static_literal_root`, `resolve_perfect_square_i128`, `contains_non_integer_numeric_literal`, `contains_negative_numeric_literal`, `resolve_static_numeric_binding`.
- [ ] **Step 3:** `static_analysis/number.rs` (`use crate::*;`, `impl TypeContext`): `resolve_number_identity_call`, `resolve_global_number_predicate_call`, `resolve_number_parse_int_call`, `resolve_number_parse_float_call`.
- [ ] **Step 4:** `static_analysis/promise.rs` (`use crate::*;`, `impl TypeContext`): `resolve_promise_member_call`.
- [ ] **Step 5:** In `static_analysis/mod.rs` add `mod object; mod math; mod number; mod promise;`. Build + test green, same count.
- [ ] **Step 6:** `git commit -m "refactor(kali_types): extract static_analysis object/math/number/promise [refactor]"`

---

### Task 12: Extract `late_host.rs`

**Files:**
- Create: `crates/kali_types/src/late_host.rs`
- Modify: `crates/kali_types/src/lib.rs`

**Interfaces:**
- Produces: `impl TypeContext` cluster for late host/runtime/network/env/permission analysis.

- [ ] **Step 1:** Create `late_host.rs` (`use crate::*;`, `impl TypeContext`) with: `resolve_permission_query_call`, `resolve_process_kill_call`, `resolve_permissions_query_descriptor_name`, `resolve_threaded_runtime_member`, `resolve_late_host_control_member`, `resolve_late_subprocess_member`, `resolve_late_network_member`, `resolve_late_permission_escalation_member`, `resolve_deno_args_member`, `resolve_late_env_object_member`, `resolve_late_env_mutation_member`, `resolve_late_env_assignment_mutation`, `resolve_late_process_env_mutation_member`, `is_process_env_root_path`, `is_process_env_mutation_path`, `resolve_late_intl_member`, `resolve_late_object_model_member`.
- [ ] **Step 2:** In `lib.rs`: `mod late_host;`. Build + test green, same count.
- [ ] **Step 3:** `git commit -m "refactor(kali_types): extract late_host module [refactor]"`

---

### Task 13: Verify `lib.rs` is now a thin facade

**Files:**
- Modify: `crates/kali_types/src/lib.rs`

- [ ] **Step 1:** Confirm `lib.rs` now contains only: crate-level `//!` docs, top-level `use` imports still needed, the `mod`/`pub use` facade lines, and the `#[cfg(test)]` test-module wiring. No struct/impl/fn definitions should remain except possibly trivial shared imports.

```bash
cd /workspace
grep -nE "^(pub )?(struct|enum|impl|fn) " crates/kali_types/src/lib.rs || echo "facade clean — no definitions remain"
wc -l crates/kali_types/src/lib.rs
```
Expected: "facade clean" (or only intentional re-export helpers), and `lib.rs` is now a few dozen lines.

- [ ] **Step 2:** `cargo build -p kali_types && cargo test -p kali_types 2>&1 | tail -5`. Green, same count.
- [ ] **Step 3:** `git commit -am "refactor(kali_types): reduce lib.rs to module facade [refactor]"` (only if changes were made).

---

## Test co-location tasks (Tasks 14–15)

### Task 14: Split `tests.rs` into sibling `*_tests.rs` per module

**Files:**
- Create: `crates/kali_types/src/scope_tests.rs`, `context_tests.rs`, `typecheck_tests.rs`, `resolve/expression_tests.rs`, `resolve/call_tests.rs`, `resolve/member_tests.rs`, `resolve/function_tests.rs`, `resolve/jsx_tests.rs`, `static_analysis/array_tests.rs`, `static_analysis/string_tests.rs`, `static_analysis/object_tests.rs`, `static_analysis/math_tests.rs`, `static_analysis/number_tests.rs`, `static_analysis/promise_tests.rs`, `late_host_tests.rs`, `builtins_tests.rs` (only those that receive tests)
- Modify: each corresponding source module (add the `#[cfg(test)] #[path] mod` wiring)
- Delete (at end): `crates/kali_types/src/tests.rs` and its facade wiring

**Classification rule:** For each `#[test]` fn in `tests.rs`, read its body and assign it to the module whose methods/behavior it exercises (e.g. a test that drives `Math.*` static folding → `static_analysis/math_tests.rs`; a test about scope binding/shadowing → `scope_tests.rs`; a `Object.keys`/freeze test → `static_analysis/object_tests.rs`; member-access naming → `resolve/member_tests.rs`). When ambiguous, place it with the most specific method it calls.

**Renaming rule:** Rename `test_resolution_NNN` to a descriptive `snake_case` name reflecting what it asserts. Record every rename so Task 16's baseline diff is explainable.

- [ ] **Step 1:** Generate a worklist of every test and a first-pass bucket:
```bash
cd /workspace
grep -nE "^\s*fn (test_)?[a-z0-9_]+\(\)" crates/kali_types/src/tests.rs | wc -l
```
Expected: ≈372.

- [ ] **Step 2:** For each destination module `<m>`, create `<m>_tests.rs` with header:
```rust
use crate::*;
use crate::test_support::*;
```
(Add the specific `kali_ast`/`kali_common`/`kali_error` `use` lines each moved test needs — copy from the current `tests.rs` import block.)

- [ ] **Step 3:** Move the classified tests into their destination files, renaming per the renaming rule. Wire each into its source module by adding at the bottom of the source file (e.g. in `static_analysis/math.rs`):
```rust
#[cfg(test)]
#[path = "math_tests.rs"]
mod math_tests;
```
For directory modules the `#[path]` is relative to the source file's directory, so `static_analysis/math.rs` → `#[path = "math_tests.rs"]` resolves to `static_analysis/math_tests.rs`.

- [ ] **Step 4:** After each destination file is populated, run its tests:
```bash
cargo test -p kali_types 2>&1 | tail -5
```
Keep `tests.rs` shrinking; move in batches by destination module, committing per module:
```bash
git add -A && git commit -m "test(kali_types): co-locate <module> tests [refactor]"
```

- [ ] **Step 5:** When `tests.rs` is empty of `#[test]` fns, delete it and remove its wiring from `lib.rs`:
```bash
rm crates/kali_types/src/tests.rs
```
Remove the `#[path = "tests.rs"] mod tests;` lines from `lib.rs`.

- [ ] **Step 6:** `cargo test -p kali_types 2>&1 | tail -5`. Green.
- [ ] **Step 7:** `git add -A && git commit -m "test(kali_types): remove monolithic tests.rs [refactor]"`

---

### Task 15: Migrate moved tests onto builders/macros where it reduces boilerplate

**Files:**
- Modify: the new `*_tests.rs` files

- [ ] **Step 1:** In the co-located test files, replace repeated inline AST construction with the `test_support` builders and the `ident!`/`member!`/`call!`/`assert_resolution!` macros where they shorten the test without obscuring intent. Do **not** macro-ize tests where the explicit form is clearer.
- [ ] **Step 2:** `cargo test -p kali_types 2>&1 | tail -5`. Green, same count.
- [ ] **Step 3:** `git add -A && git commit -m "test(kali_types): adopt builders/macros in co-located tests [refactor]"`

---

### Task 16: Final verification, lint, and baseline diff

**Files:**
- Create: `docs/superpowers/baselines/kali_types-tests-after.txt`
- Create: `docs/superpowers/baselines/kali_types-tests-renames.md`

- [ ] **Step 1:** Regenerate the after-snapshot:
```bash
cd /workspace
cargo test -p kali_types -- --list 2>/dev/null | grep ': test$' | sed 's/: test$//' | sort > docs/superpowers/baselines/kali_types-tests-after.txt
```

- [ ] **Step 2:** Diff before vs after. Every difference must be an intentional rename:
```bash
diff docs/superpowers/baselines/kali_types-tests-before.txt docs/superpowers/baselines/kali_types-tests-after.txt
```
Record the full before→after rename mapping in `docs/superpowers/baselines/kali_types-tests-renames.md`. Confirm the **count is unchanged** (same number of `: test` lines) — no test was dropped:
```bash
wc -l docs/superpowers/baselines/kali_types-tests-*.txt
```

- [ ] **Step 3:** Format and lint:
```bash
cargo fmt -p kali_types
cargo clippy -p kali_types --all-targets 2>&1 | tail -15
```
Expected: no new clippy errors; warnings should be no worse than the pre-refactor baseline.

- [ ] **Step 4:** Full workspace sanity (kali_types is a dependency of many crates):
```bash
cargo build --workspace 2>&1 | tail -5
cargo test -p kali_types 2>&1 | tail -5
```
Expected: workspace builds; kali_types tests green.

- [ ] **Step 5:** Final per-file size check (confirm the monolith is gone):
```bash
find crates/kali_types/src -name '*.rs' | xargs wc -l | sort -rn | head -20
```
Expected: no single file near the old 7,896 / 15,308 line counts; modules are small and focused.

- [ ] **Step 6:** Commit and prepare for review:
```bash
git add -A
git commit -m "test(kali_types): record post-refactor baseline + renames [refactor]"
```

- [ ] **Step 7: STOP for pilot review.** Per the spec, pause here for sign-off before rolling the pattern out to other crates. Summarize: per-file line counts before/after, the rename mapping, and confirmation that the test count is unchanged and the suite is green.

---

## Self-Review Notes (for the implementer)

- If a moved method references a name that was previously visible only because everything lived in one module and you missed promoting it in Task 2, the fix is to add `pub(crate)` to that item — never to alter a method body.
- `use super::*;` vs `use crate::*;`: both work for method-only modules; `use crate::*;` is preferred in nested directory modules (`resolve/`, `static_analysis/`) to avoid surprises with `super` resolving to the directory's `mod.rs`.
- The `#[path]` attribute for a sibling test file is resolved **relative to the directory of the file containing the `mod` declaration**, so always use the bare filename (`#[path = "math_tests.rs"]`), not a path with directories.
- Do not change `Cargo.toml` dependency versions; only the additions in Task 3.
