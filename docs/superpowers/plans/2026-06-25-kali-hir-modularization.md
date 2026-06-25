# kali_hir Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split `kali_hir`'s monolithic `lib.rs` (HIR node types + one ~900-line `impl HirLowerer`) and `tests.rs` (20 tests) into small, single-responsibility modules behind a thin facade, with zero behavior change.

**Architecture:** Impl-split. The HIR node types split by concern (`node`/`builder`/`result`/`helpers`); the `HirLowerer` struct stays in one file and its lowering methods split by responsibility into a `lowering/` subtree, each file carrying its own `impl HirLowerer { … }`. `lib.rs` becomes a facade (`mod` decls + `pub use`). Tests co-locate into sibling `*_tests.rs` files wired via `#[cfg(test)] #[path] mod`, sharing a `cfg(test)` `parse()` helper.

**Tech Stack:** Rust (edition 2021), Cargo workspace. Deps: `kali_ast`, `kali_common`, `kali_error`, `kali_types`. Dev-deps: `kali_parser`, `kali_lexer` (already configured — no Cargo.toml change).

## Global Constraints

- **Pure structural refactor — zero behavior change.** No logic rewritten; only items relocated and visibility widened.
- **Verbatim text-movement.** Method/fn/type bodies move byte-for-byte, including blank-line separators and the original's exact qualification style. Do **not** convert inline `kali_ast::Foo` refs to imported short names, or vice versa.
- **Exact same tests before and after.** The basename multiset of `cargo test -p kali_hir -- --list` must be identical (no test added, dropped, renamed, or duplicated). Canonical proof command:
  ```
  cargo test -p kali_hir -- --list 2>/dev/null | grep ': test$' | sed -E 's/^.*:://; s/: test$//' | sort
  ```
  Compare before vs after with `diff` → empty. Use `sort` **without** `-u` (preserve any duplicate basename; hold the count constant).
- `cargo test -p kali_hir` must be green after **every** commit (20 tests; no integration-test directory exists for this crate).
- `lib.rs` ends as a thin facade: module declarations + `pub use` re-exports + crate docs + `cfg(test)` test wiring. Public paths preserved unchanged: `kali_hir::{HirLowerer, LoweringResult, FunctionFlavor, HirBuilder, HirNode, HirNodeId, HirNodeKind}` and their public methods/fields.
- Unit tests live in sibling `*_tests.rs` files wired via `#[cfg(test)] #[path = "…"] mod`, per AGENTS.md — **not** inline `#[cfg(test)]` modules.
- Final commit only after `cargo fmt -p kali_hir`, `cargo clippy -p kali_hir --all-targets` clean, `cargo build` (workspace) clean, and the basename-multiset diff empty.
- Commit convention: `refactor(kali_hir): <summary> [refactor]` (or `test`/`style`/`docs`).

---

## File Structure

Source layout when complete (`crates/kali_hir/src/`):

```
lib.rs           facade: crate docs + mod decls + pub use + cfg(test) test wiring
node.rs          HirNodeKind, HirNode (+impl), HirNodeId (+impl)
builder.rs       HirBuilder (+impl, +impl Default)
result.rs        FunctionFlavor (+impl), LoweringResult (+impl), validate_tree (private free fn)
helpers.rs       free fns: lower_literal_value, logical_op_text, update_op_text, assignment_op_text, object_property_kind_text
lowering/
  mod.rs         HirLowerer struct + push_child! macro + impl Default;
                 entry API (lower_statements, lower_program_from_ast, lower_node, diagnostics, clear_diagnostics);
                 shared helpers (push_child, record_function_flavor, next_synthetic_function_name)
  statement.rs   lower_statement + lower_block, lower_class_body, lower_method_definition, lower_variable_declarator
  expression.rs  lower_expression + lower_template_literal, lower_update_expression, lower_assignment_expression, lower_optional_chain
  function.rs    lower_function_expression, lower_arrow_function_expression, lower_class_expression
  object.rs      lower_object_property, lower_property_name
  module.rs      lower_import_specifier, lower_export_specifier, lower_export_default
```

Test layout when complete:

```
test_support.rs              cfg(test) shared `parse()` helper
builder_tests.rs
result_tests.rs
lowering/
  statement_tests.rs
  expression_tests.rs
  function_tests.rs
  object_tests.rs
```

**Item → module map** (current `lib.rs` line numbers, for relocation):

| Module | Items (current lines) |
|---|---|
| `node.rs` | `HirNodeKind` enum 37–97; `HirNode` struct 101–110 + `impl HirNode` 122–140; `HirNodeId` struct 114 + `impl HirNodeId` 116–120 |
| `builder.rs` | `HirBuilder` struct 143–146; `impl HirBuilder` 148–178; `impl Default for HirBuilder` 180–184 |
| `result.rs` | `FunctionFlavor` enum 188–193 + `impl FunctionFlavor` 195–204; `LoweringResult` struct 207–216 + `impl LoweringResult` 218–237; `validate_tree` free fn 1209–1244 |
| `helpers.rs` | `lower_literal_value` 1159–1167, `logical_op_text` 1169–1175, `update_op_text` 1177–1184, `assignment_op_text` 1186–1199, `object_property_kind_text` 1201–1207 |
| `lowering/mod.rs` | `HirLowerer` struct 240–245; `push_child!` macro 247–252; `impl Default for HirLowerer` 1153–1157; from `impl HirLowerer`: `new` 255, `diagnostics` 264, `clear_diagnostics` 268, `lower_statements` 273, `lower_program_from_ast` 301, `lower_node` 310, `next_synthetic_function_name` 999, `record_function_flavor` 1142, `push_child` 1146 |
| `lowering/statement.rs` | `lower_statement` 318–674, `lower_class_body` 676–682, `lower_method_definition` 684–708, `lower_block` 710–716, `lower_variable_declarator` 718–732 |
| `lowering/expression.rs` | `lower_expression` 734–926, `lower_template_literal` 928–942, `lower_optional_chain` 1015–1026, `lower_update_expression` 1028–1036, `lower_assignment_expression` 1038–1047 |
| `lowering/function.rs` | `lower_function_expression` 944–973, `lower_arrow_function_expression` 975–997, `lower_class_expression` 1005–1013 |
| `lowering/object.rs` | `lower_object_property` 1049–1058, `lower_property_name` 1060–1083 |
| `lowering/module.rs` | `lower_import_specifier` 1085–1107, `lower_export_specifier` 1109–1120, `lower_export_default` 1122–1140 |

Exact placement of a borderline method (e.g. `lower_block` in `statement.rs` vs `lowering/mod.rs`) may shift during implementation as long as it compiles and the suite stays green; the table is the target, not a frozen contract.

**Import idiom for the lowering pass modules.** During extraction the `HirLowerer` struct and the `push_child!` macro stay at the crate root (in `lib.rs`) until the final facade task. Every `lowering/*.rs` pass module therefore references them via the crate root: `use crate::HirLowerer;` and `use crate::push_child;`. In the final task the struct + macro move into `lowering/mod.rs`, and `lib.rs`'s facade re-exports both back to the crate root (`pub use lowering::HirLowerer;` + `pub(crate) use lowering::push_child;`) so the pass modules' imports never change. Cross-type names resolve via `use crate::node::{HirNodeId, HirNodeKind};`, `use crate::result::FunctionFlavor;`, and `use crate::helpers::{…};`. Beyond these explicit `crate::` imports, let the compiler errors drive the exact `kali_ast` `use` list per module, copying names (and their qualification style) from the originals.

---

### Task 1: Baseline + widen visibility for extraction

**Files:**
- Create: a baseline file under the session scratchpad (not the repo).
- Modify: `crates/kali_hir/src/lib.rs` (visibility + one `pub(crate) use` line only).

**Interfaces:**
- Consumes: nothing.
- Produces: a `pub(crate)` surface on `HirLowerer` fields + private lowering methods, the two `HirBuilder` fields, the 5 free helper fns, and a path-importable `push_child` macro — so sibling-module `impl` blocks and the unrelocated `tests.rs` keep compiling.

- [ ] **Step 1: Capture the basename-multiset baseline**

```bash
cargo test -p kali_hir -- --list 2>/dev/null | grep ': test$' \
  | sed -E 's/^.*:://; s/: test$//' | sort > "$SCRATCH/kali_hir_baseline.txt"
wc -l "$SCRATCH/kali_hir_baseline.txt"
```
(`$SCRATCH` = the session scratchpad dir.) Expected: 20 lines. Keep this file for the final diff. Also note whether any basename appears twice (none expected) so the final check holds the same count.

- [ ] **Step 2: Confirm a green starting point**

Run: `cargo test -p kali_hir`
Expected: PASS, `test result: ok. 20 passed`.

- [ ] **Step 3: Widen `HirLowerer` and `HirBuilder` field visibility to `pub(crate)`**

In `crates/kali_hir/src/lib.rs`:

```rust
pub struct HirBuilder {
    pub(crate) nodes: Vec<HirNode>,
    pub(crate) next_id: HirNodeId,
}
```
```rust
pub struct HirLowerer {
    pub(crate) builder: HirBuilder,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) function_flavors: Vec<(HirNodeId, FunctionFlavor)>,
    pub(crate) synthetic_function_counter: usize,
}
```
(`nodes` is read by `lower_statements`; `next_id` is read by `test_hir_builder` — both cross a module boundary after extraction.)

- [ ] **Step 4: Widen the private `impl HirLowerer` methods to `pub(crate)`**

For every method in `impl HirLowerer` **except** the already-`pub` `new`, `diagnostics`, `clear_diagnostics`, `lower_statements`, `lower_program_from_ast`, `lower_node`, prefix the existing `fn` with `pub(crate) ` (leave bodies untouched). These are: `lower_statement`, `lower_class_body`, `lower_method_definition`, `lower_block`, `lower_variable_declarator`, `lower_expression`, `lower_template_literal`, `lower_function_expression`, `lower_arrow_function_expression`, `next_synthetic_function_name`, `lower_class_expression`, `lower_optional_chain`, `lower_update_expression`, `lower_assignment_expression`, `lower_object_property`, `lower_property_name`, `lower_import_specifier`, `lower_export_specifier`, `lower_export_default`, `record_function_flavor`, `push_child`.

- [ ] **Step 5: Widen the free helper fns to `pub(crate)`**

Prefix `pub(crate) ` on `lower_literal_value`, `logical_op_text`, `update_op_text`, `assignment_op_text`, `object_property_kind_text`. **Leave `validate_tree` private** — its only caller, `LoweringResult::validate`, will be co-located with it in `result.rs`.

- [ ] **Step 6: Make the `push_child!` macro path-importable**

Immediately after the `macro_rules! push_child { … }` block in `lib.rs`, add:

```rust
pub(crate) use push_child;
```
This makes `crate::push_child` a valid import path for sibling modules (textual-scope use within `lib.rs` itself keeps working).

- [ ] **Step 7: Verify still green (no relocation yet)**

Run: `cargo test -p kali_hir`
Expected: PASS, 20 passed. (No `#![allow]` needed yet; widened-but-unused warnings do not occur because every widened item still has an in-crate caller.)

- [ ] **Step 8: Commit**

```bash
git add crates/kali_hir/src/lib.rs
git commit -m "refactor(kali_hir): widen private items to pub(crate) for extraction [refactor]"
```

---

### Task 2: Extract `node.rs`

**Files:**
- Create: `crates/kali_hir/src/node.rs`
- Modify: `crates/kali_hir/src/lib.rs`

**Interfaces:**
- Consumes: `kali_common::Span`.
- Produces: `pub struct HirNodeKind`/`HirNode`/`HirNodeId` (+ their impls), re-exported from the crate root via the facade.

- [ ] **Step 1: Create `node.rs` with the node types**

Move `HirNodeKind` (37–97), `HirNode` struct (101–110) + `impl HirNode` (122–140), and `HirNodeId` struct (114) + `impl HirNodeId` (116–120) verbatim. Header:

```rust
//! HIR node representation: kinds, nodes, and node identifiers.

use kali_common::Span;
```

- [ ] **Step 2: Wire the module into `lib.rs`**

Delete the moved items from `lib.rs`. Add `mod node;` with the other decls and re-export:

```rust
pub use node::{HirNode, HirNodeId, HirNodeKind};
```
Keep `lib.rs`'s own `use kali_common::Span;` only if still referenced there (the remaining `HirLowerer` code does not use `Span` directly — remove the import if it becomes unused).

- [ ] **Step 3: Run tests**

Run: `cargo test -p kali_hir`
Expected: PASS, 20 passed.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_hir/src/node.rs crates/kali_hir/src/lib.rs
git commit -m "refactor(kali_hir): extract node module [refactor]"
```

---

### Task 3: Extract `builder.rs`

**Files:**
- Create: `crates/kali_hir/src/builder.rs`
- Modify: `crates/kali_hir/src/lib.rs`

**Interfaces:**
- Consumes: `crate::node::{HirNode, HirNodeId, HirNodeKind}`, `kali_common::Span`.
- Produces: `pub struct HirBuilder` (with `pub(crate)` fields + its `pub` methods), re-exported from the crate root.

- [ ] **Step 1: Create `builder.rs`**

Move `HirBuilder` struct (143–146), `impl HirBuilder` (148–178), and `impl Default for HirBuilder` (180–184) verbatim. Header:

```rust
//! Arena builder that allocates HIR nodes by id.

use crate::node::{HirNode, HirNodeId, HirNodeKind};
use kali_common::Span;
```

- [ ] **Step 2: Wire into `lib.rs`**

Delete the moved items. Add `mod builder;` and `pub use builder::HirBuilder;`. The `HirLowerer` struct field `builder: HirBuilder` and `HirBuilder::new()` resolve via the re-export.

- [ ] **Step 3: Run tests**

Run: `cargo test -p kali_hir`
Expected: PASS, 20 passed. (`test_hir_builder` still reads `builder.next_id` — now `pub(crate)` — and `lower_statements` reads `builder.nodes`.)

- [ ] **Step 4: Commit**

```bash
git add crates/kali_hir/src/builder.rs crates/kali_hir/src/lib.rs
git commit -m "refactor(kali_hir): extract builder module [refactor]"
```

---

### Task 4: Extract `result.rs`

**Files:**
- Create: `crates/kali_hir/src/result.rs`
- Modify: `crates/kali_hir/src/lib.rs`

**Interfaces:**
- Consumes: `crate::node::{HirNode, HirNodeId}`, `kali_error::diagnostic::Diagnostic`.
- Produces: `pub enum FunctionFlavor` (+impl), `pub struct LoweringResult` (+impl), re-exported from the crate root; `validate_tree` stays a private free fn co-located here.

- [ ] **Step 1: Create `result.rs`**

Move `FunctionFlavor` (188–193) + `impl FunctionFlavor` (195–204), `LoweringResult` (207–216) + `impl LoweringResult` (218–237), and the **private** `validate_tree` free fn (1209–1244) verbatim. Header:

```rust
//! Lowering output: function-flavor metadata, the lowering result, and tree validation.

use crate::node::{HirNode, HirNodeId};
use kali_error::diagnostic::Diagnostic;
```
`LoweringResult::validate` calls `validate_tree(...)` (same module — no widening, no path change).

- [ ] **Step 2: Wire into `lib.rs`**

Delete the moved items (including `validate_tree`). Add `mod result;` and `pub use result::{FunctionFlavor, LoweringResult};`. The remaining `HirLowerer` code (`record_function_flavor`, `FunctionFlavor::from_flags`, `LoweringResult { … }` construction in `lower_statements`) resolves via the re-export.

- [ ] **Step 3: Run tests**

Run: `cargo test -p kali_hir`
Expected: PASS, 20 passed.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_hir/src/result.rs crates/kali_hir/src/lib.rs
git commit -m "refactor(kali_hir): extract result module [refactor]"
```

---

### Task 5: Extract `helpers.rs`

**Files:**
- Create: `crates/kali_hir/src/helpers.rs`
- Modify: `crates/kali_hir/src/lib.rs`

**Interfaces:**
- Consumes: `kali_ast` operator/literal types.
- Produces: `pub(crate)` free fns `lower_literal_value`, `logical_op_text`, `update_op_text`, `assignment_op_text`, `object_property_kind_text`, called by the (still-in-`lib.rs`) lowering methods and later by `lowering/expression.rs` + `lowering/object.rs`.

- [ ] **Step 1: Create `helpers.rs`**

Move the 5 free fns (1159–1207) verbatim. **Preserve each fn's exact qualification style:** `logical_op_text` and `update_op_text` use inline `kali_ast::LogicalOperator` / `kali_ast::UpdateOperator`; `assignment_op_text` and `object_property_kind_text` use imported short names `AssignmentOperator` / `ObjectPropertyKind`. Header:

```rust
//! Pure text/value formatting helpers shared by the lowering passes.

use kali_ast::{AssignmentOperator, LiteralValue, ObjectPropertyKind};
```
(Do not import `LogicalOperator`/`UpdateOperator` — they stay inline-qualified, matching the originals.)

- [ ] **Step 2: Wire into `lib.rs`**

Delete the moved fns. Add `mod helpers;` (no `pub use` — internal fns). The lowering methods still in `lib.rs` (`lower_expression`, `lower_update_expression`, `lower_assignment_expression`, `lower_object_property`) call these by bare name, so add at the top of `lib.rs`:

```rust
use crate::helpers::{
    assignment_op_text, logical_op_text, lower_literal_value, object_property_kind_text,
    update_op_text,
};
```
This import migrates into `lowering/expression.rs` / `lowering/object.rs` as those methods move (Tasks 7 and 9); remove it from `lib.rs` once no caller remains there.

- [ ] **Step 3: Run tests**

Run: `cargo test -p kali_hir`
Expected: PASS, 20 passed.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_hir/src/helpers.rs crates/kali_hir/src/lib.rs
git commit -m "refactor(kali_hir): extract helpers module [refactor]"
```

---

### Task 6: Extract `lowering/statement.rs` (create the `lowering/` subtree)

**Files:**
- Create: `crates/kali_hir/src/lowering/mod.rs`, `crates/kali_hir/src/lowering/statement.rs`
- Modify: `crates/kali_hir/src/lib.rs`

**Interfaces:**
- Consumes: `crate::HirLowerer`, `crate::push_child`, `crate::node::{HirNodeId, HirNodeKind}`, `crate::result::FunctionFlavor`, `kali_ast` statement types.
- Produces: `impl HirLowerer` statement-lowering methods (still `pub(crate)`), reachable from the entry API and the other lowering modules unchanged. Establishes `lowering/mod.rs` as the subtree parent.

- [ ] **Step 1: Create `lowering/mod.rs` as the subtree parent**

For now it only declares the statement submodule (more `mod` lines are added in Tasks 7–9; the struct + macro move here in Task 10):

```rust
//! AST → HIR lowering passes (one `impl HirLowerer` per responsibility).

mod statement;
```

- [ ] **Step 2: Create `lowering/statement.rs` with an `impl HirLowerer` block**

Move `lower_statement` (318–674), `lower_class_body` (676–682), `lower_method_definition` (684–708), `lower_block` (710–716), `lower_variable_declarator` (718–732) verbatim. Header:

```rust
//! Statement lowering: the `lower_statement` dispatcher + block/class/declarator helpers.

use crate::node::{HirNodeId, HirNodeKind};
use crate::result::FunctionFlavor;
use crate::{push_child, HirLowerer};
use kali_ast::{ /* statement types used by the moved bodies — compiler-driven */ };

impl HirLowerer {
    // … moved methods verbatim …
}
```
The `push_child!` invocations inside the moved bodies resolve via `use crate::push_child;`.

- [ ] **Step 3: Remove moved methods from `lib.rs`, add `mod lowering;`**

Delete the 5 methods from `lib.rs`'s `impl HirLowerer`. Add `mod lowering;` with the other `mod` decls. Trim any `kali_ast` import in `lib.rs` now used only by the moved bodies (the compiler flags unused names).

- [ ] **Step 4: Run tests**

Run: `cargo test -p kali_hir`
Expected: PASS, 20 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_hir/src/lowering crates/kali_hir/src/lib.rs
git commit -m "refactor(kali_hir): extract lowering statement pass [refactor]"
```

---

### Task 7: Extract `lowering/expression.rs`

**Files:**
- Create: `crates/kali_hir/src/lowering/expression.rs`
- Modify: `crates/kali_hir/src/lowering/mod.rs`, `crates/kali_hir/src/lib.rs`

**Interfaces:**
- Consumes: `crate::HirLowerer`, `crate::push_child`, `crate::node::{HirNodeId, HirNodeKind}`, `crate::helpers::{lower_literal_value, logical_op_text, update_op_text, assignment_op_text}`, `kali_ast` expression types.
- Produces: `impl HirLowerer` expression-lowering methods.

- [ ] **Step 1: Create `lowering/expression.rs`**

Move `lower_expression` (734–926), `lower_template_literal` (928–942), `lower_optional_chain` (1015–1026), `lower_update_expression` (1028–1036), `lower_assignment_expression` (1038–1047) verbatim. Header:

```rust
//! Expression lowering: the `lower_expression` dispatcher + template/update/assignment/optional-chain helpers.

use crate::helpers::{assignment_op_text, logical_op_text, lower_literal_value, update_op_text};
use crate::node::{HirNodeId, HirNodeKind};
use crate::{push_child, HirLowerer};
use kali_ast::{ /* expression types used by the moved bodies — compiler-driven */ };

impl HirLowerer {
    // … moved methods verbatim …
}
```
`lower_expression` calls `self.lower_function_expression` / `self.lower_arrow_function_expression` / `self.lower_class_expression` / `self.lower_object_property` (still in `lib.rs` until Tasks 8–9) — these resolve as `pub(crate)` methods on the shared type.

- [ ] **Step 2: Add `mod expression;` to `lowering/mod.rs`; remove moved methods from `lib.rs`**

Add `mod expression;` (keep modules alphabetical). Delete the 5 methods from `lib.rs`. Move the `crate::helpers::{…}` import contributions for these four fns out of `lib.rs` (keep `object_property_kind_text` in `lib.rs` until `lower_object_property` leaves in Task 9). Trim newly-unused `kali_ast` names from `lib.rs`.

- [ ] **Step 3: Run tests**

Run: `cargo test -p kali_hir`
Expected: PASS, 20 passed.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_hir/src/lowering crates/kali_hir/src/lib.rs
git commit -m "refactor(kali_hir): extract lowering expression pass [refactor]"
```

---

### Task 8: Extract `lowering/function.rs`

**Files:**
- Create: `crates/kali_hir/src/lowering/function.rs`
- Modify: `crates/kali_hir/src/lowering/mod.rs`, `crates/kali_hir/src/lib.rs`

**Interfaces:**
- Consumes: `crate::HirLowerer`, `crate::push_child`, `crate::node::{HirNodeId, HirNodeKind}`, `crate::result::FunctionFlavor`, `kali_ast::{FunctionExpression, ArrowFunctionExpression, ClassExpression, ReturnStatement, Statement}`.
- Produces: `impl HirLowerer` function/class-expression lowering methods.

- [ ] **Step 1: Create `lowering/function.rs`**

Move `lower_function_expression` (944–973), `lower_arrow_function_expression` (975–997), `lower_class_expression` (1005–1013) verbatim, using the same scaffold (`impl HirLowerer`, `use crate::{push_child, HirLowerer};`, `use crate::result::FunctionFlavor;`, `use crate::node::{HirNodeId, HirNodeKind};`, compiler-driven `kali_ast` list). Note: `lower_class_expression` calls `self.lower_class_body` (in `statement.rs`) — resolves via the shared type.

- [ ] **Step 2: Add `mod function;` to `lowering/mod.rs`; remove from `lib.rs`**

- [ ] **Step 3: Run tests**

Run: `cargo test -p kali_hir`
Expected: PASS, 20 passed.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_hir/src/lowering crates/kali_hir/src/lib.rs
git commit -m "refactor(kali_hir): extract lowering function pass [refactor]"
```

---

### Task 9: Extract `lowering/object.rs` and `lowering/module.rs`

**Files:**
- Create: `crates/kali_hir/src/lowering/object.rs`, `crates/kali_hir/src/lowering/module.rs`
- Modify: `crates/kali_hir/src/lowering/mod.rs`, `crates/kali_hir/src/lib.rs`

**Interfaces:**
- Consumes: `crate::HirLowerer`, `crate::push_child`, `crate::node::{HirNodeId, HirNodeKind}`, `crate::helpers::object_property_kind_text` (object), `kali_ast` object + import/export types.
- Produces: `impl HirLowerer` object-property and import/export lowering methods. After this task the only `impl HirLowerer` methods left in `lib.rs` are the entry/core/shared set destined for `lowering/mod.rs`.

- [ ] **Step 1: Create `lowering/object.rs`**

Move `lower_object_property` (1049–1058), `lower_property_name` (1060–1083) verbatim. Header:

```rust
//! Object-literal lowering: property and property-name handling.

use crate::helpers::object_property_kind_text;
use crate::node::{HirNodeId, HirNodeKind};
use crate::{push_child, HirLowerer};
use kali_ast::{ObjectProperty, PropertyName};

impl HirLowerer { /* … */ }
```

- [ ] **Step 2: Create `lowering/module.rs`**

Move `lower_import_specifier` (1085–1107), `lower_export_specifier` (1109–1120), `lower_export_default` (1122–1140) verbatim, with `use crate::{push_child, HirLowerer};`, `use crate::node::{HirNodeId, HirNodeKind};`, and the compiler-driven `kali_ast` list (includes `ImportSpecifier`, `ExportSpecifier`, `ExportDefaultDeclaration`, `Statement`, `FunctionDeclaration`, `ClassDeclaration`). `lower_export_default` calls `self.lower_statement` / `self.lower_expression` — resolve via the shared type.

- [ ] **Step 3: Add `mod module; mod object;` to `lowering/mod.rs`; remove from `lib.rs`**

Delete the 5 methods from `lib.rs`. Remove the now-unused `use crate::helpers::object_property_kind_text;` from `lib.rs` (the last helper consumer has left). Trim newly-unused `kali_ast` names from `lib.rs`.

- [ ] **Step 4: Run tests**

Run: `cargo test -p kali_hir`
Expected: PASS, 20 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_hir/src/lowering crates/kali_hir/src/lib.rs
git commit -m "refactor(kali_hir): extract lowering object and module passes [refactor]"
```

---

### Task 10: Move `HirLowerer` core into `lowering/mod.rs`; reduce `lib.rs` to a facade

**Files:**
- Modify: `crates/kali_hir/src/lowering/mod.rs`, `crates/kali_hir/src/lib.rs`

**Interfaces:**
- Consumes: every `lowering/*.rs` pass module's `impl HirLowerer` extensions; `crate::builder::HirBuilder`, `crate::node::{HirNodeId, HirNodeKind}`, `crate::result::{FunctionFlavor, LoweringResult}`.
- Produces: `pub struct HirLowerer`, its constructor + entry API + shared helpers, and the `push_child!` macro — all re-exported from the crate root so the pass modules' `crate::HirLowerer` / `crate::push_child` imports keep resolving.

- [ ] **Step 1: Move the struct, macro, and core methods into `lowering/mod.rs`**

Into `lowering/mod.rs` (above the `mod` declarations), move from `lib.rs`: the `HirLowerer` struct (240–245), the `macro_rules! push_child { … }` block (247–252) **immediately followed by `pub(crate) use push_child;`**, `impl Default for HirLowerer` (1153–1157), and the remaining `impl HirLowerer` methods (`new`, `diagnostics`, `clear_diagnostics`, `lower_statements`, `lower_program_from_ast`, `lower_node`, `next_synthetic_function_name`, `record_function_flavor`, `push_child`). Add its imports:

```rust
use crate::builder::HirBuilder;
use crate::node::{HirNodeId, HirNodeKind};
use crate::result::{FunctionFlavor, LoweringResult};
use kali_ast::{Statement, AST, NodeId};
use kali_error::diagnostic::Diagnostic;
```
Keep the existing `mod statement; mod expression; mod function; mod object; mod module;` declarations.

- [ ] **Step 2: Reduce `lib.rs` to a pure facade**

`lib.rs` should now contain only crate docs, module declarations, re-exports, and the `cfg(test)` test wiring:

```rust
//! High-level intermediate representation (HIR) for the Kali compiler.
//!
//! This crate provides a deterministic AST-to-HIR lowering layer used by the
//! later MIR/LIR stages. The implementation is intentionally conservative and
//! source-shaped so the phase-1 pipeline can round-trip representative programs
//! without inventing extra semantics.

mod builder;
mod helpers;
mod lowering;
mod node;
mod result;

pub use builder::HirBuilder;
pub use lowering::HirLowerer;
pub use node::{HirNode, HirNodeId, HirNodeKind};
pub use result::{FunctionFlavor, LoweringResult};

pub(crate) use lowering::push_child;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
```
`pub use lowering::HirLowerer;` and `pub(crate) use lowering::push_child;` re-export the struct and macro to the crate root, so the pass modules' `use crate::HirLowerer;` / `use crate::push_child;` need no change. Remove all now-unused top-level `use` imports (they live in the modules that need them).

- [ ] **Step 3: Run tests + clippy**

Run: `cargo test -p kali_hir && cargo clippy -p kali_hir --all-targets`
Expected: PASS, 20 passed; clippy clean (no new warnings).

- [ ] **Step 4: Commit**

```bash
git add crates/kali_hir/src/lowering crates/kali_hir/src/lib.rs
git commit -m "refactor(kali_hir): move HirLowerer core into lowering, reduce lib.rs to facade [refactor]"
```

---

### Task 11: Add shared `test_support` and relocate tests

**Files:**
- Create: `crates/kali_hir/src/test_support.rs`, `builder_tests.rs`, `result_tests.rs`, `lowering/statement_tests.rs`, `lowering/expression_tests.rs`, `lowering/function_tests.rs`, `lowering/object_tests.rs`
- Modify: `crates/kali_hir/src/builder.rs`, `result.rs`, `lowering/{statement,expression,function,object}.rs` (`#[cfg(test)] #[path] mod` wiring); `lib.rs` (drop the `tests.rs` wiring, add `test_support` wiring); delete `crates/kali_hir/src/tests.rs`

**Interfaces:**
- Consumes: the public `HirLowerer`/`HirBuilder`/types API and `kali_parser::Parser` + `kali_lexer::Lexer`.
- Produces: `pub(crate) fn parse(source: &str) -> Vec<kali_ast::Statement>` for all test modules.

- [ ] **Step 1: Create the shared test helper**

```rust
// crates/kali_hir/src/test_support.rs
//! Shared test helpers for the HIR test modules.
use kali_ast::Statement;
use kali_common::FileId;
use kali_lexer::Lexer;
use kali_parser::Parser;

pub(crate) fn parse(source: &str) -> Vec<Statement> {
    let lexer = Lexer::new(FileId::new(0), source.to_string());
    let tokens = lexer.lex_all().tokens;
    let mut parser = Parser::new(FileId::new(0), tokens);
    parser.parse(None).statements
}
```
Wire it in `lib.rs` under cfg(test), replacing the old `mod tests;` wiring:

```rust
#[cfg(test)]
mod test_support;
```

- [ ] **Step 2: Split `tests.rs` by cluster into sibling `*_tests.rs` files**

Read each of the 20 tests and move it (verbatim, **same name**) into the `*_tests.rs` file matching the source module it exercises. Target mapping (settle borderline cases by reading the body):

| Test (tests.rs line) | Destination |
|---|---|
| `test_hir_builder` (15) | `builder_tests.rs` |
| `test_hir_validation_rejects_out_of_bounds_children` (620) | `result_tests.rs` |
| `test_lower_statements_to_hir` (23), `test_lower_program_from_ast_matches_statement_lowering_for_empty_ast_shell` (46), `test_lower_statements_records_export_all_nodes` (466) | `lowering/statement_tests.rs` |
| the eight `…records_function_flavor_metadata…` tests (60, 95, 154, 177, 200, 225, 250, 297, 353, 409 — incl. class-method/class-expression/default-export forms) | `lowering/function_tests.rs` |
| `test_object_literal_lowers_to_stable_property_shape` (483), `test_numeric_object_property_names_lower_as_string_literals` (512), `…_from_parsed_source_as_string_literals` (535), `…_negative_zero_as_string_literal_zero` (561) | `lowering/object_tests.rs` |
| `test_update_expression_lowers_prefix_and_postfix_forms` (587) | `lowering/expression_tests.rs` |

Each `*_tests.rs` starts with:

```rust
use crate::test_support::parse;
use crate::*;            // HirBuilder, HirLowerer, HirNodeKind, LoweringResult, FunctionFlavor, …
use kali_ast::{ /* types referenced by the tests in this file, e.g. UpdateExpression, UpdateOperator, AST */ };
```
Replace the old in-file `fn parse` with `use crate::test_support::parse;`. Do **not** rename any test. Preserve the `kali_common::FileId` reference where a test constructs ids — import or qualify it to match the moved body verbatim.

- [ ] **Step 3: Wire each `*_tests.rs` into its source module**

At the bottom of each source module add the wiring, e.g. in `builder.rs`:

```rust
#[cfg(test)]
#[path = "builder_tests.rs"]
mod builder_tests;
```
For the `lowering/*` submodules the `#[path]` is relative to that submodule file (e.g. `lowering/object.rs` → `#[path = "object_tests.rs"]`). Delete the old `#[cfg(test)] #[path = "tests.rs"] mod tests;` from `lib.rs` and the file `src/tests.rs`.

- [ ] **Step 4: Run tests**

Run: `cargo test -p kali_hir`
Expected: PASS, 20 passed.

- [ ] **Step 5: Diff the basename-multiset baseline**

```bash
cargo test -p kali_hir -- --list 2>/dev/null | grep ': test$' \
  | sed -E 's/^.*:://; s/: test$//' | sort > "$SCRATCH/kali_hir_after.txt"
diff "$SCRATCH/kali_hir_baseline.txt" "$SCRATCH/kali_hir_after.txt"
```
Expected: **empty diff** (no test added, dropped, renamed, or duplicated). If non-empty, a test was misplaced/duplicated — fix before committing.

- [ ] **Step 6: Commit**

```bash
git add crates/kali_hir/src
git commit -m "test(kali_hir): relocate tests into sibling modules + shared test_support [refactor]"
```

---

### Task 12: Final verification

**Files:** none (verification only).

- [ ] **Step 1: Format**

Run: `cargo fmt -p kali_hir` then `git diff --stat`
If fmt changed files, review the diff (must be formatting-only — no logic/value change) and commit: `git commit -am "style(kali_hir): cargo fmt [refactor]"`.

- [ ] **Step 2: Clippy**

Run: `cargo clippy -p kali_hir --all-targets`
Expected: clean (no new warnings vs. baseline; no new `#[allow]`).

- [ ] **Step 3: Full suite + workspace build**

Run: `cargo test -p kali_hir && cargo build`
Expected: 20 passed; workspace builds (confirms no downstream crate broke on the facade).

- [ ] **Step 4: Confirm the basename-multiset proof is empty**

Re-run the Step-5 diff from Task 11. Expected: empty.

- [ ] **Step 5: Confirm `lib.rs` is a thin facade**

Run: `wc -l crates/kali_hir/src/lib.rs` and `grep -nE '^\s*(fn|impl|struct|enum|macro_rules!)' crates/kali_hir/src/lib.rs`
Expected: a small file (~25 lines) of crate docs + `mod` decls + `pub use` + cfg(test) wiring; the grep returns nothing (no types/impls/fns/macros remain in `lib.rs`).

---

## Self-Review

- **Spec coverage:** type-cluster split into `node`/`builder`/`result`/`helpers` (Tasks 2–5); `HirLowerer` impl-split into the `lowering/` subtree (Tasks 6–10); facade (Task 10); the two crate-specific wrinkles — `push_child!` macro relocation via `pub(crate) use` path-import (set up Task 1 Step 6, re-homed Task 10) and `HirBuilder.nodes`/`next_id` + `HirLowerer` field widening (Task 1 Steps 3–5); incidental `#[allow(unused_imports)]` removal (the big `kali_ast` import never reaches the facade — split across the per-module precise imports); test co-location with no renames + shared `parse` (Task 11); basename-multiset proof (Task 1 capture, Task 11/12 diff); fmt/clippy/green/workspace-build finish (Task 12). All spec sections mapped.
- **Placeholder scan:** No "TBD"/"handle edge cases"/"similar to". Relocation steps name exact items + line numbers; helper/test-support code and the final facade are shown in full. The only deferred detail is the per-module `kali_ast` `use` list, explicitly delegated to compiler-driven resolution (matching the kali_parser precedent) with qualification-style preservation called out.
- **Type consistency:** `HirLowerer`, `HirBuilder`, `LoweringResult`, `FunctionFlavor`, `HirNode`/`HirNodeId`/`HirNodeKind`, `push_child` (macro), `parse(&str) -> Vec<Statement>` used consistently across tasks; facade `pub use` names match the struct/enum names; pass-module imports (`crate::HirLowerer`, `crate::push_child`) match the re-export set established in Task 10.
