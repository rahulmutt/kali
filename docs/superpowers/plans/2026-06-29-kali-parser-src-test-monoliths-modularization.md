# kali_parser src test-monolith modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split kali_parser's four multi-concern co-located `src/*_tests.rs` unit-test monoliths into thin facades + per-concern `#[path] mod` submodules, by pure verbatim code-motion.

**Architecture:** Each monolith `src/<name>_tests.rs` is declared from its product module via `#[path = "<name>_tests.rs"] mod <name>_tests;`. We turn each into a facade that keeps its original `use` line(s) (plus, for `declaration_tests`, its one non-test helper fn) and one `#[path = "<name>_tests/<group>.rs"] mod <group>;` per group, and we move each `#[test]` fn **verbatim** into the matching `src/<name>_tests/<group>.rs` file, each of which opens with `use super::*;`. No product code changes; the compiled test set is byte-for-byte identical.

**Tech Stack:** Rust 2021, cargo, kali workspace.

**Reusable tooling (optional, recommended):** `.superpowers/sdd/move_fns.py` in exact-name-set mode automates each split (`python3 move_fns.py <stem>_tests.rs "g1=name1,name2;g2=..."` run from `crates/kali_parser`); it drains the `#[test]` fns into submodules, **auto-retains any non-`#[test]` module-level fn in the facade**, and appends the `#[path] mod` decls. `.superpowers/sdd/verify.py` proves `{name: body}` from the original == from the submodules (+ facade-retained items). Manual code-motion that produces the exact files below is equally acceptable.

## Global Constraints

- **Pure verbatim code-motion, zero behavior change.** No new product code, no new tests, no renamed tests, no reformatting of moved bodies. Move each `#[test]` fn exactly as written.
- **Facades drain to 0 module-level `#[test]` fns.** The only things retained on a facade are its original `use` line(s), the new `#[path] mod` declarations, and — **only for `declaration_tests`** — its one non-test helper fn `assert_parse_class_method_modifiers_are_preserved`.
- **Submodule header:** every new `src/<name>_tests/<group>.rs` begins with exactly `use super::*;` and nothing else before the first moved fn. (The `class_method` submodule reaches the retained helper through this glob.)
- **Test count is the invariant.** kali_parser's lib test suite is **65 tests** before and after; per-file filters must report: declaration_tests 23, call_tests 8, mod_tests 10, module_tests 10. (The four name filters each uniquely select — no substring collision.)
- **No `pub`/`pub(crate)` widening, no `include_*!` pins** — verified 0 of each across all four files; no signature changes needed.
- **Product siblings unchanged:** `declaration.rs`, `expression/call.rs`, `expression/mod.rs`, `module.rs` keep their existing `#[cfg(test)] #[path = "F_tests.rs"] mod F_tests;` decls.
- **Build gate:** `cargo build -p kali_parser --tests` stays at **0 warnings** (baseline = 0).
- **`cargo fmt --check`** — accept known fmt nits per series convention (do not reformat moved bodies to satisfy it).
- **Commits:** one `refactor(kali_parser): split <file>_tests.rs into per-concern test submodules [refactor]` per task. Local-main ff-merge only; no origin push.

---

### Task 1: Split `declaration_tests.rs` (23 tests → 4 submodules)

**Files:**
- Create: `crates/kali_parser/src/declaration_tests/arrow.rs`
- Create: `crates/kali_parser/src/declaration_tests/generator.rs`
- Create: `crates/kali_parser/src/declaration_tests/class_method.rs`
- Create: `crates/kali_parser/src/declaration_tests/function.rs`
- Modify: `crates/kali_parser/src/declaration_tests.rs` (reduce to facade; **keep the `assert_parse_class_method_modifiers_are_preserved` helper**)
- Unchanged: `crates/kali_parser/src/declaration.rs` (`#[path = "declaration_tests.rs"] mod declaration_tests;` at lines 359-360 stays)

**Interfaces:**
- Consumes: nothing from other tasks.
- Produces: nothing other tasks depend on. (Each task is independent.)

- [ ] **Step 1: Confirm baseline count**

Run: `cargo test -p kali_parser declaration_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 23 passed; ...`

- [ ] **Step 2: Create `arrow.rs`**

Create `crates/kali_parser/src/declaration_tests/arrow.rs` starting with `use super::*;`, then move these fns **verbatim** from `declaration_tests.rs` (with their full bodies):
- `test_parse_parenthesized_arrow_function_expression`
- `test_parse_single_parameter_arrow_function_expression`
- `test_parse_async_arrow_function_expression`
- `test_parse_async_arrow_function_return_type_annotation_with_multiple_params`
- `test_parse_async_single_parameter_arrow_function_expression`
- `test_parse_async_arrow_function_return_type_annotation`
- `test_parse_arrow_function_return_type_annotation`

- [ ] **Step 3: Create `generator.rs`**

Create `crates/kali_parser/src/declaration_tests/generator.rs` starting with `use super::*;`, then move verbatim:
- `test_parse_generator_function_declaration`
- `test_parse_generator_delegating_yield_expression`
- `test_parse_generator_function_expression`
- `test_parse_async_generator_function_declaration`
- `test_parse_async_generator_function_expression`
- `test_parse_yield_expression_outside_generator_remains_identifier`

- [ ] **Step 4: Create `class_method.rs`**

Create `crates/kali_parser/src/declaration_tests/class_method.rs` starting with `use super::*;`, then move verbatim (these consume the retained facade helper via `use super::*;`):
- `test_parse_generator_class_method_preserves_generator_flag`
- `test_parse_generator_class_method_delegating_yield_expression`
- `test_parse_async_generator_class_method_preserves_generator_flags`
- `test_parse_class_expression_preserves_method_modifiers`
- `test_parse_default_export_class_expression_preserves_method_modifiers`
- `test_parse_default_export_class_declaration_preserves_method_modifiers`

- [ ] **Step 5: Create `function.rs`**

Create `crates/kali_parser/src/declaration_tests/function.rs` starting with `use super::*;`, then move verbatim:
- `test_parse_export_async_function_declaration`
- `test_parse_async_await_expression`
- `test_parse_function_declaration_stops_before_following_statement`
- `test_parse_async_function_expression`

- [ ] **Step 6: Reduce the facade**

Edit `crates/kali_parser/src/declaration_tests.rs`: delete all 23 `#[test]` fns (now moved), **keep** the three `use` lines and the `assert_parse_class_method_modifiers_are_preserved` helper fn in place, and append the four `#[path] mod` decls. The result must be exactly (helper body unchanged, shown abbreviated here — do NOT retype it, leave the existing fn untouched):

```rust
use crate::test_support::lex;
use crate::*;
use kali_ast::{Expression, Statement};

fn assert_parse_class_method_modifiers_are_preserved(
    source: &str,
    is_async: bool,
    generator: bool,
) {
    // ... existing body unchanged ...
}

#[path = "declaration_tests/arrow.rs"]
mod arrow;

#[path = "declaration_tests/generator.rs"]
mod generator;

#[path = "declaration_tests/class_method.rs"]
mod class_method;

#[path = "declaration_tests/function.rs"]
mod function;
```

- [ ] **Step 7: Verify count unchanged and tests pass**

Run: `cargo test -p kali_parser declaration_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 23 passed; 0 failed; ...`

- [ ] **Step 8: Verify whole-crate suite and build**

Run: `cargo test -p kali_parser --lib 2>&1 | grep 'test result'`
Expected: `test result: ok. 65 passed; ...`
Run: `cargo build -p kali_parser --tests 2>&1 | grep -c '^warning'`
Expected: `0`

- [ ] **Step 9: Commit**

```bash
git add crates/kali_parser/src/declaration_tests.rs crates/kali_parser/src/declaration_tests/
git commit -m "refactor(kali_parser): split declaration_tests.rs into per-concern test submodules [refactor]"
```

---

### Task 2: Split `expression/call_tests.rs` (8 tests → 3 submodules)

**Files:**
- Create: `crates/kali_parser/src/expression/call_tests/member.rs`
- Create: `crates/kali_parser/src/expression/call_tests/optional_chain.rs`
- Create: `crates/kali_parser/src/expression/call_tests/dynamic_import.rs`
- Modify: `crates/kali_parser/src/expression/call_tests.rs` (reduce to facade)
- Unchanged: `crates/kali_parser/src/expression/call.rs` (`#[path = "call_tests.rs"] mod call_tests;` at lines 219-220 stays)

**Interfaces:**
- Consumes: nothing. Produces: nothing other tasks depend on.

- [ ] **Step 1: Confirm baseline count**

Run: `cargo test -p kali_parser call_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 8 passed; ...`

- [ ] **Step 2: Create `member.rs`**

Create `crates/kali_parser/src/expression/call_tests/member.rs` starting with `use super::*;`, then move verbatim:
- `test_parse_bracketed_member_expression_chain`
- `test_parse_fully_bracketed_permission_escalation_member_expression_chain`
- `test_parse_mixed_bracket_dot_late_object_model_member_expression_chain`
- `test_parse_dot_delete_member_expression_after_keyword_property`
- `test_parse_dot_from_member_expression_after_keyword_property`

- [ ] **Step 3: Create `optional_chain.rs`**

Create `crates/kali_parser/src/expression/call_tests/optional_chain.rs` starting with `use super::*;`, then move verbatim:
- `test_parse_optional_chain_member_expression`
- `test_parse_optional_chain_index_expression`

- [ ] **Step 4: Create `dynamic_import.rs`**

Create `crates/kali_parser/src/expression/call_tests/dynamic_import.rs` starting with `use super::*;`, then move verbatim:
- `test_parse_dynamic_import_expression`

- [ ] **Step 5: Reduce the facade**

Replace the entire contents of `crates/kali_parser/src/expression/call_tests.rs` with exactly:

```rust
use crate::test_support::lex;
use crate::*;
use kali_ast::{Expression, Statement};

#[path = "call_tests/member.rs"]
mod member;

#[path = "call_tests/optional_chain.rs"]
mod optional_chain;

#[path = "call_tests/dynamic_import.rs"]
mod dynamic_import;
```

- [ ] **Step 6: Verify count unchanged and tests pass**

Run: `cargo test -p kali_parser call_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 8 passed; 0 failed; ...`

- [ ] **Step 7: Verify whole-crate suite and build**

Run: `cargo test -p kali_parser --lib 2>&1 | grep 'test result'`
Expected: `test result: ok. 65 passed; ...`
Run: `cargo build -p kali_parser --tests 2>&1 | grep -c '^warning'`
Expected: `0`

- [ ] **Step 8: Commit**

```bash
git add crates/kali_parser/src/expression/call_tests.rs crates/kali_parser/src/expression/call_tests/
git commit -m "refactor(kali_parser): split call_tests.rs into per-concern test submodules [refactor]"
```

---

### Task 3: Split `expression/mod_tests.rs` (10 tests → 3 submodules)

**Files:**
- Create: `crates/kali_parser/src/expression/mod_tests/unary.rs`
- Create: `crates/kali_parser/src/expression/mod_tests/binary.rs`
- Create: `crates/kali_parser/src/expression/mod_tests/type_ops.rs`
- Modify: `crates/kali_parser/src/expression/mod_tests.rs` (reduce to facade)
- Unchanged: `crates/kali_parser/src/expression/mod.rs` (`#[path = "mod_tests.rs"] mod mod_tests;` at lines 239-240 stays)

**Interfaces:**
- Consumes: nothing. Produces: nothing other tasks depend on.

- [ ] **Step 1: Confirm baseline count**

Run: `cargo test -p kali_parser mod_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 10 passed; ...`

- [ ] **Step 2: Create `unary.rs`**

Create `crates/kali_parser/src/expression/mod_tests/unary.rs` starting with `use super::*;`, then move verbatim:
- `test_parse_prefix_update_expression`
- `test_parse_void_unary_expression`
- `test_parse_bitwise_not_unary_expression`
- `test_parse_postfix_update_expression`

- [ ] **Step 3: Create `binary.rs`**

Create `crates/kali_parser/src/expression/mod_tests/binary.rs` starting with `use super::*;`, then move verbatim:
- `test_parse_nullish_coalescing_expression`
- `test_parse_exponentiation_expression`
- `test_parse_modulo_expression`
- `test_parse_compound_assignment_expression`

- [ ] **Step 4: Create `type_ops.rs`**

Create `crates/kali_parser/src/expression/mod_tests/type_ops.rs` starting with `use super::*;`, then move verbatim:
- `test_parse_type_assertion_expression`
- `test_parse_satisfies_expression`

- [ ] **Step 5: Reduce the facade**

Replace the entire contents of `crates/kali_parser/src/expression/mod_tests.rs` with exactly:

```rust
use crate::test_support::lex;
use crate::*;
use kali_ast::{AssignmentOperator, Expression, Statement, UpdateOperator};

#[path = "mod_tests/unary.rs"]
mod unary;

#[path = "mod_tests/binary.rs"]
mod binary;

#[path = "mod_tests/type_ops.rs"]
mod type_ops;
```

- [ ] **Step 6: Verify count unchanged and tests pass**

Run: `cargo test -p kali_parser mod_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 10 passed; 0 failed; ...`

- [ ] **Step 7: Verify whole-crate suite and build**

Run: `cargo test -p kali_parser --lib 2>&1 | grep 'test result'`
Expected: `test result: ok. 65 passed; ...`
Run: `cargo build -p kali_parser --tests 2>&1 | grep -c '^warning'`
Expected: `0`

- [ ] **Step 8: Commit**

```bash
git add crates/kali_parser/src/expression/mod_tests.rs crates/kali_parser/src/expression/mod_tests/
git commit -m "refactor(kali_parser): split mod_tests.rs into per-concern test submodules [refactor]"
```

---

### Task 4: Split `module_tests.rs` (10 tests → 2 submodules)

**Files:**
- Create: `crates/kali_parser/src/module_tests/import.rs`
- Create: `crates/kali_parser/src/module_tests/export.rs`
- Modify: `crates/kali_parser/src/module_tests.rs` (reduce to facade)
- Unchanged: `crates/kali_parser/src/module.rs` (`#[path = "module_tests.rs"] mod module_tests;` at lines 304-305 stays)

**Interfaces:**
- Consumes: nothing. Produces: nothing other tasks depend on.

- [ ] **Step 1: Confirm baseline count**

Run: `cargo test -p kali_parser module_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 10 passed; ...`

- [ ] **Step 2: Create `import.rs`**

Create `crates/kali_parser/src/module_tests/import.rs` starting with `use super::*;`, then move verbatim:
- `test_parse_side_effect_import_declaration`
- `test_parse_default_import_declaration`

- [ ] **Step 3: Create `export.rs`**

Create `crates/kali_parser/src/module_tests/export.rs` starting with `use super::*;`, then move verbatim:
- `test_parse_named_export_declaration`
- `test_parse_named_export_declaration_allows_default_aliases`
- `test_parse_export_all_declaration`
- `test_parse_default_export_function_declaration`
- `test_parse_default_export_generator_function_declaration`
- `test_parse_default_export_async_generator_function_declaration`
- `test_parse_default_export_anonymous_async_generator_function_declaration`
- `test_parse_default_export_anonymous_generator_function_declaration`

- [ ] **Step 4: Reduce the facade**

Replace the entire contents of `crates/kali_parser/src/module_tests.rs` with exactly:

```rust
use crate::test_support::lex;
use crate::*;
use kali_ast::{ExportSpecifier, ImportSpecifier, Statement};

#[path = "module_tests/import.rs"]
mod import;

#[path = "module_tests/export.rs"]
mod export;
```

- [ ] **Step 5: Verify count unchanged and tests pass**

Run: `cargo test -p kali_parser module_tests 2>/dev/null | grep 'test result'`
Expected: `test result: ok. 10 passed; 0 failed; ...`

- [ ] **Step 6: Verify whole-crate suite and build**

Run: `cargo test -p kali_parser --lib 2>&1 | grep 'test result'`
Expected: `test result: ok. 65 passed; ...`
Run: `cargo build -p kali_parser --tests 2>&1 | grep -c '^warning'`
Expected: `0`

- [ ] **Step 7: Commit**

```bash
git add crates/kali_parser/src/module_tests.rs crates/kali_parser/src/module_tests/
git commit -m "refactor(kali_parser): split module_tests.rs into per-concern test submodules [refactor]"
```

---

## Final verification (after all 4 tasks)

- [ ] **Whole-crate lib suite:** `cargo test -p kali_parser --lib 2>&1 | grep 'test result'` → `65 passed; 0 failed`.
- [ ] **Build gate:** `cargo build -p kali_parser --tests 2>&1 | grep -c '^warning'` → `0`.
- [ ] **Byte-identity proof:** for each split file, `python3 .superpowers/sdd/verify.py <orig_rs> "<submodule_glob>" [facade_glob_for_pins]` exits 0 (51/51 `#[test]` bodies byte-identical base→head). For `declaration_tests`, pass the facade glob so the retained helper is accounted for.
- [ ] **Dependent crate compiles unedited:** `cargo build -p kali_codegen` (a kali_parser consumer) builds clean.
- [ ] **Diff is motion-only:** `git diff --stat <base>..HEAD -- crates/kali_parser/` shows only the four `*_tests.rs` facades shrinking + new submodule files; no product-source (`declaration.rs`, `expression/call.rs`, `expression/mod.rs`, `module.rs`) line changes.
- [ ] **Fmt:** `cargo fmt -p kali_parser --check` — accept known nits per series convention; do not reformat moved bodies.
