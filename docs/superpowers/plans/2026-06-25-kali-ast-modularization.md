# kali_ast Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the 1,164-line `kali_ast/src/lib.rs` into eight flat, single-category modules behind a thin facade, with the 5 tests relocated into sibling `*_tests.rs` files — zero behavior change.

**Architecture:** `kali_ast` is pure data (~70 `#[derive(...)]` AST node types plus the arena types `ASTBuilder`/`AST`). Decompose by syntactic category into flat sibling modules (`node`, `builder`, `statement`, `declaration`, `module`, `expression`, `literal`, `jsx`). `lib.rs` becomes crate docs + `mod` declarations + `pub use <mod>::*` glob re-exports, so every existing `kali_ast::TypeName` path keeps resolving. Extraction order is dependency-independent: anything not yet extracted stays at the crate root, and anything extracted is re-exported back to the crate root, so the crate compiles after every task.

**Tech Stack:** Rust 2021, `serde`, `kali_common::Span`. Workspace crate.

**Spec:** `docs/superpowers/specs/2026-06-25-kali-ast-modularization-design.md`

## Global Constraints

- **Pure structural refactor — zero behavior change.** Move type/impl definitions verbatim; do not rewrite logic, rename types, change fields, or alter derives/attributes.
- **`cargo test -p kali_ast` must pass after every task** (every commit leaves the crate green and `cargo build --workspace` working).
- **Proof obligation:** the multiset of test **basenames** (final path segment of each `--list` entry) must stay exactly `{test_ast_builder, test_ast_conversion, test_ast_roundtrips_default_export_anonymous_generator_function_declaration, test_function_kind_metadata_survives_serde_roundtrip, test_node_id}` — 5 names. The module prefix changes (`tests::` → `node_tests::`/`builder_tests::`/`declaration_tests::`); that prefix change is the only intended `--list` difference.
- **Test wiring convention (AGENTS.md):** unit tests live in sibling `*_tests.rs` files wired via `#[cfg(test)] #[path = "..."] mod ...;` at the bottom of the owning module — never inline `#[cfg(test)] mod tests { ... }`.
- **No new dependencies; no `test_support` crate/module** — the test surface is too small to need one.
- Run `cargo fmt` and `cargo clippy -p kali_ast` clean at the end.

---

### Task 1: Baseline snapshot

**Files:**
- Create: `/tmp/claude-1000/-workspace/e8fa5ef1-c98f-412b-aeca-1ffc86df502b/scratchpad/kali_ast_list_before.txt`

**Interfaces:**
- Consumes: nothing.
- Produces: a sorted baseline file used by Task 10 to diff test basenames.

- [ ] **Step 1: Confirm the crate is green before touching anything**

Run: `cargo test -p kali_ast`
Expected: `test result: ok. 5 passed`

- [ ] **Step 2: Capture the sorted test-name baseline**

Run:
```bash
cargo test -p kali_ast -- --list 2>/dev/null | grep ': test$' \
  | sed 's/.*:://; s/: test$//' | sort \
  > /tmp/claude-1000/-workspace/e8fa5ef1-c98f-412b-aeca-1ffc86df502b/scratchpad/kali_ast_list_before.txt
cat /tmp/claude-1000/-workspace/e8fa5ef1-c98f-412b-aeca-1ffc86df502b/scratchpad/kali_ast_list_before.txt
```
Expected output (5 lines, basenames only):
```
test_ast_builder
test_ast_conversion
test_ast_roundtrips_default_export_anonymous_generator_function_declaration
test_function_kind_metadata_survives_serde_roundtrip
test_node_id
```

No commit (scratchpad file only).

---

### Task 2: Extract `node.rs` (+ relocate `test_node_id`)

**Files:**
- Create: `crates/kali_ast/src/node.rs`
- Create: `crates/kali_ast/src/node_tests.rs`
- Modify: `crates/kali_ast/src/lib.rs`
- Modify: `crates/kali_ast/src/tests.rs`

**Interfaces:**
- Consumes: `kali_common::Span`.
- Produces (re-exported at crate root): `NodeId`, `Node`, `NodeKind`, `ModuleItem`.

- [ ] **Step 1: Create `node.rs` and move the node-core items into it**

Move these items **verbatim** from `lib.rs` (original lines ~8–46, ~314–446, ~556–585) into the new `crates/kali_ast/src/node.rs`:
`NodeId` (struct + `impl serde::Serialize` + `impl serde::Deserialize` + `impl NodeId` + `impl Display`), `ModuleItem` type alias (`pub type ModuleItem = Node;`), `NodeKind` enum + `impl PartialEq for NodeKind` + `impl Eq for NodeKind`, and `Node` struct + `impl Node`.

Prepend this header to `node.rs`:
```rust
//! AST node identity and the legacy `NodeKind` tree node.

use kali_common::Span;

#[cfg(test)]
#[path = "node_tests.rs"]
mod node_tests;
```

- [ ] **Step 2: Create `node_tests.rs` with the relocated test**

Create `crates/kali_ast/src/node_tests.rs` (move `test_node_id` verbatim from `tests.rs`):
```rust
use crate::*;

#[test]
fn test_node_id() {
    let id = NodeId::new(42);
    assert_eq!(id.as_u32(), 42);
    assert_eq!(id.to_string(), "n42");
}
```

- [ ] **Step 3: Wire the module in `lib.rs` and remove the moved items**

In `lib.rs`: delete the moved definitions (NodeId/NodeKind/Node/ModuleItem and their impls), and add the module declaration + re-export near the top (after the crate doc comment):
```rust
mod node;

pub use node::*;
```
**Keep** the existing `use kali_common::Span;` at the crate root — `ASTBuilder::new_node` (still inline in `lib.rs` until Task 3) references `Span`. It is removed in Task 3 once `builder.rs` is extracted. Inline references to `NodeKind`/`Node` in `lib.rs` continue to resolve through the `pub use node::*;` re-export.

In `tests.rs`: delete the `test_node_id` function.

- [ ] **Step 4: Verify green**

Run: `cargo test -p kali_ast`
Expected: `test result: ok. 5 passed` (now `node_tests::test_node_id` + 4 under `tests::`).

- [ ] **Step 5: Commit**

```bash
git add crates/kali_ast/src/node.rs crates/kali_ast/src/node_tests.rs crates/kali_ast/src/lib.rs crates/kali_ast/src/tests.rs
git commit -m "refactor(kali_ast): extract node module [refactor]"
```

---

### Task 3: Extract `builder.rs` (+ relocate `test_ast_builder`, `test_ast_conversion`)

**Files:**
- Create: `crates/kali_ast/src/builder.rs`
- Create: `crates/kali_ast/src/builder_tests.rs`
- Modify: `crates/kali_ast/src/lib.rs`
- Modify: `crates/kali_ast/src/tests.rs`

**Interfaces:**
- Consumes (from crate root): `Node`, `NodeId`, `NodeKind`; `kali_common::Span`.
- Produces (re-exported at crate root): `ASTBuilder`, `AST`.

- [ ] **Step 1: Create `builder.rs` and move the arena types into it**

Move **verbatim** from `lib.rs` (original lines ~587–795): `ASTBuilder` struct + `impl ASTBuilder` + `impl Default for ASTBuilder`, `AST` struct + `impl AST` + `impl Default for AST`, and `impl std::convert::From<ASTBuilder> for AST`.

Prepend to `crates/kali_ast/src/builder.rs`:
```rust
//! Arena-style AST storage: the `ASTBuilder` and finalized `AST`.

use crate::{Node, NodeId, NodeKind};
use kali_common::Span;

#[cfg(test)]
#[path = "builder_tests.rs"]
mod builder_tests;
```

- [ ] **Step 2: Create `builder_tests.rs` with the relocated tests**

Create `crates/kali_ast/src/builder_tests.rs` (move both tests verbatim from `tests.rs`):
```rust
use crate::*;

#[test]
fn test_ast_builder() {
    let mut builder = ASTBuilder::new();
    let root_id = builder.new_node(NodeKind::Program, None);
    builder.set_root(root_id);

    let root = builder.get_node(root_id).unwrap();
    assert_eq!(root.kind, NodeKind::Program);

    assert!(builder.root().is_some());
}

#[test]
fn test_ast_conversion() {
    let mut builder = ASTBuilder::new();
    let root_id = builder.new_node(NodeKind::Program, None);
    builder.set_root(root_id);

    let ast: AST = builder.into();
    assert!(ast.root().is_some());
}
```

- [ ] **Step 3: Wire the module in `lib.rs` and remove the moved items**

In `lib.rs`: delete the moved `ASTBuilder`/`AST` definitions and impls; add `mod builder;` and `pub use builder::*;` alongside the other module declarations. **Remove the now-unused `use kali_common::Span;` from the crate root** (`builder.rs` carries its own copy; nothing inline in `lib.rs` uses `Span` anymore).

In `tests.rs`: delete `test_ast_builder` and `test_ast_conversion`.

- [ ] **Step 4: Verify green**

Run: `cargo test -p kali_ast`
Expected: `test result: ok. 5 passed`

- [ ] **Step 5: Commit**

```bash
git add crates/kali_ast/src/builder.rs crates/kali_ast/src/builder_tests.rs crates/kali_ast/src/lib.rs crates/kali_ast/src/tests.rs
git commit -m "refactor(kali_ast): extract builder module [refactor]"
```

---

### Task 4: Extract `statement.rs`

**Files:**
- Create: `crates/kali_ast/src/statement.rs`
- Modify: `crates/kali_ast/src/lib.rs`

**Interfaces:**
- Consumes (from crate root): `Expression`, `VariableDeclaration`, `FunctionDeclaration`, `ClassDeclaration`, `ImportDeclaration`, `ExportAllDeclaration`, `ExportNamedDeclaration`, `ExportDefaultDeclaration`, `EnumDeclaration`, `TypeAliasDeclaration`, `InterfaceDeclaration`.
- Produces (re-exported at crate root): `Statement`, `ExpressionStatement`, `BreakStatement`, `ContinueStatement`, `WithStatement`, `ReturnStatement`, `LabeledStatement`, `IfStatement`, `SwitchStatement`, `SwitchCase`, `ThrowStatement`, `TryStatement`, `CatchClause`, `DebuggerStatement`, `BlockStatement`, `ForStatement`, `ForInit`, `ForInStatement`, `ForInLefthand`, `ForOfStatement`, `ForOfLefthand`, `WhileStatement`, `DoWhileStatement`.

- [ ] **Step 1: Create `statement.rs` and move the statement items into it**

Move **verbatim** from `lib.rs` (original lines ~48–312): every statement struct/enum listed under "Produces" above, including the unified `Statement` enum (orig. lines ~282–312). Do **not** move `FunctionDeclaration`/`ClassDeclaration`/`VariableDeclaration`/`*Declaration` types — those go to `declaration.rs`/`module.rs` in later tasks and remain at the crate root for now.

Prepend to `crates/kali_ast/src/statement.rs`:
```rust
//! Statement node types and the unified `Statement` enum.

use crate::{
    ClassDeclaration, EnumDeclaration, ExportAllDeclaration, ExportDefaultDeclaration,
    ExportNamedDeclaration, Expression, FunctionDeclaration, ImportDeclaration,
    InterfaceDeclaration, TypeAliasDeclaration, VariableDeclaration,
};
```

- [ ] **Step 2: Wire the module in `lib.rs` and remove the moved items**

In `lib.rs`: delete the moved statement definitions; add `mod statement;` and `pub use statement::*;`.

- [ ] **Step 3: Verify green**

Run: `cargo test -p kali_ast`
Expected: `test result: ok. 5 passed`

- [ ] **Step 4: Commit**

```bash
git add crates/kali_ast/src/statement.rs crates/kali_ast/src/lib.rs
git commit -m "refactor(kali_ast): extract statement module [refactor]"
```

---

### Task 5: Extract `declaration.rs` (+ relocate the 2 serde round-trip tests)

**Files:**
- Create: `crates/kali_ast/src/declaration.rs`
- Create: `crates/kali_ast/src/declaration_tests.rs`
- Modify: `crates/kali_ast/src/lib.rs`
- Modify: `crates/kali_ast/src/tests.rs`

**Interfaces:**
- Consumes (from crate root): `BlockStatement`, `Expression`.
- Produces (re-exported at crate root): `FunctionDeclaration`, `ClassDeclaration`, `ClassBody`, `MethodDefinition`, `VariableDeclaration`, `VariableDeclarator`, `TypeAliasDeclaration`, `InterfaceDeclaration`, `PropertySignature`, `EnumDeclaration`, `EnumMember`.

- [ ] **Step 1: Create `declaration.rs` and move the declaration items into it**

Move **verbatim** from `lib.rs` (original lines ~199–280): `FunctionDeclaration`, `ClassDeclaration`, `ClassBody`, `MethodDefinition`, `VariableDeclaration`, `VariableDeclarator`, `TypeAliasDeclaration`, `InterfaceDeclaration`, `PropertySignature`, `EnumDeclaration`, `EnumMember`.

Prepend to `crates/kali_ast/src/declaration.rs`:
```rust
//! Declaration node types: functions, classes, variables, types, enums.

use crate::{BlockStatement, Expression};

#[cfg(test)]
#[path = "declaration_tests.rs"]
mod declaration_tests;
```

- [ ] **Step 2: Create `declaration_tests.rs` with the two relocated serde tests**

Create `crates/kali_ast/src/declaration_tests.rs` and move `test_function_kind_metadata_survives_serde_roundtrip` and `test_ast_roundtrips_default_export_anonymous_generator_function_declaration` **verbatim** from `tests.rs` (the full bodies as currently written), with this header instead of `use super::*;`:
```rust
use crate::*;
```
Keep both test functions byte-for-byte identical to their current bodies in `tests.rs` (they construct `FunctionDeclaration`/`FunctionExpression`/`ClassExpression`/`ClassDeclaration`/`Statement::ExportDefault(...)` values and assert serde round-trips). Do not edit assertions.

- [ ] **Step 3: Wire the module in `lib.rs`, remove moved items, delete emptied `tests.rs`**

In `lib.rs`: delete the moved declaration definitions; add `mod declaration;` and `pub use declaration::*;`. Remove the now-obsolete test wiring `#[cfg(test)] #[path = "tests.rs"] mod tests;`.

`tests.rs` is now empty of tests — delete the file:
```bash
git rm crates/kali_ast/src/tests.rs
```

- [ ] **Step 4: Verify green**

Run: `cargo test -p kali_ast`
Expected: `test result: ok. 5 passed` (now `declaration_tests::` ×2, `node_tests::` ×1, `builder_tests::` ×2; nothing under `tests::`).

- [ ] **Step 5: Commit**

```bash
git add crates/kali_ast/src/declaration.rs crates/kali_ast/src/declaration_tests.rs crates/kali_ast/src/lib.rs
git commit -m "refactor(kali_ast): extract declaration module, relocate remaining tests [refactor]"
```

---

### Task 6: Extract `module.rs` (import/export declarations)

**Files:**
- Create: `crates/kali_ast/src/module.rs`
- Modify: `crates/kali_ast/src/lib.rs`

**Interfaces:**
- Consumes (from crate root): `Expression`, `FunctionDeclaration`, `ClassDeclaration`.
- Produces (re-exported at crate root): `ImportDeclaration`, `ImportSpecifier`, `ImportNamedSpecifier`, `ImportName`, `ExportDeclaration`, `ExportNamedDeclaration`, `ExportSpecifier`, `ExportAllDeclaration`, `ExportDefaultDeclaration`, `ExportTypeDeclaration`.

- [ ] **Step 1: Create `module.rs` and move the import/export items into it**

Move **verbatim** from `lib.rs` (original lines ~593–676): `ImportDeclaration`, `ImportSpecifier`, `ImportNamedSpecifier`, `ImportName`, `ExportDeclaration`, `ExportNamedDeclaration`, `ExportSpecifier`, `ExportAllDeclaration`, `ExportDefaultDeclaration`, `ExportTypeDeclaration`.

Prepend to `crates/kali_ast/src/module.rs`:
```rust
//! ES module syntax: import and export declarations.

use crate::{ClassDeclaration, Expression, FunctionDeclaration};
```

- [ ] **Step 2: Wire the module in `lib.rs` and remove the moved items**

In `lib.rs`: delete the moved import/export definitions; add `mod module;` and `pub use module::*;`.

- [ ] **Step 3: Verify green**

Run: `cargo test -p kali_ast`
Expected: `test result: ok. 5 passed`

- [ ] **Step 4: Commit**

```bash
git add crates/kali_ast/src/module.rs crates/kali_ast/src/lib.rs
git commit -m "refactor(kali_ast): extract module (import/export) module [refactor]"
```

---

### Task 7: Extract `literal.rs`

**Files:**
- Create: `crates/kali_ast/src/literal.rs`
- Modify: `crates/kali_ast/src/lib.rs`

**Interfaces:**
- Consumes (from crate root): `Expression`, `SpreadElement`.
- Produces (re-exported at crate root): `LiteralValue`, `ArrayExpression`, `ObjectExpression`, `ObjectProperty`, `PropertyName`, `ObjectPropertyKind`, `ExpressionOrSpread`.

- [ ] **Step 1: Create `literal.rs` and move the literal/collection items into it**

Move **verbatim** from `lib.rs`: `LiteralValue` (orig. ~804–811), `ArrayExpression` (~813–817), `ObjectExpression` (~819–823), `ObjectProperty` (~825–831), `PropertyName` (~833–839), `ObjectPropertyKind` (~841–847), and `ExpressionOrSpread` (~1138–1143).

Prepend to `crates/kali_ast/src/literal.rs`:
```rust
//! Literal values and array/object literal expressions.

use crate::{Expression, SpreadElement};
```

- [ ] **Step 2: Wire the module in `lib.rs` and remove the moved items**

In `lib.rs`: delete the moved literal definitions; add `mod literal;` and `pub use literal::*;`.

- [ ] **Step 3: Verify green**

Run: `cargo test -p kali_ast`
Expected: `test result: ok. 5 passed`

- [ ] **Step 4: Commit**

```bash
git add crates/kali_ast/src/literal.rs crates/kali_ast/src/lib.rs
git commit -m "refactor(kali_ast): extract literal module [refactor]"
```

---

### Task 8: Extract `jsx.rs`

**Files:**
- Create: `crates/kali_ast/src/jsx.rs`
- Modify: `crates/kali_ast/src/lib.rs`

**Interfaces:**
- Consumes (from crate root): `Expression`.
- Produces (re-exported at crate root): `JsxElement`, `JsxOpeningElement`, `JsxChild`, `JsxExpressionContainer`, `JsxFragment`, `JsxName`, `JsxAttributeItem`, `JsxAttribute`, `JsxAttributeValue`, `JsxSpreadAttribute`, `JsxSelfClosingElement`, `JsxClosingElement`.

- [ ] **Step 1: Create `jsx.rs` and move all `Jsx*` items into it**

Move **verbatim** from `lib.rs` (original lines ~1049–1164): `JsxElement`, `JsxOpeningElement`, `JsxChild`, `JsxExpressionContainer`, `JsxFragment`, `JsxName`, `JsxAttributeItem`, `JsxAttribute`, `JsxAttributeValue`, `JsxSpreadAttribute`, `JsxSelfClosingElement`, `JsxClosingElement`.

Prepend to `crates/kali_ast/src/jsx.rs`:
```rust
//! JSX node types.

use crate::Expression;
```

- [ ] **Step 2: Wire the module in `lib.rs` and remove the moved items**

In `lib.rs`: delete the moved `Jsx*` definitions; add `mod jsx;` and `pub use jsx::*;`.

- [ ] **Step 3: Verify green**

Run: `cargo test -p kali_ast`
Expected: `test result: ok. 5 passed`

- [ ] **Step 4: Commit**

```bash
git add crates/kali_ast/src/jsx.rs crates/kali_ast/src/lib.rs
git commit -m "refactor(kali_ast): extract jsx module [refactor]"
```

---

### Task 9: Extract `expression.rs` and reduce `lib.rs` to a facade

**Files:**
- Create: `crates/kali_ast/src/expression.rs`
- Modify: `crates/kali_ast/src/lib.rs`

**Interfaces:**
- Consumes (from crate root): `LiteralValue`, `ArrayExpression`, `ObjectExpression`, `JsxElement`, `JsxFragment`, `ClassBody`, `BlockStatement`.
- Produces (re-exported at crate root): `Expression` (+ `impl AsRef`), `BinaryExpression`, `UnaryExpression`, `CallExpression`, `MemberExpression`, `NewExpression`, `MetaProperty`, `UpdateExpression`, `UpdateOperator`, `AssignmentExpression`, `AssignmentOperator`, `LogicalExpression`, `LogicalOperator`, `ConditionalExpression`, `SequenceExpression`, `ParenthesizedExpression`, `YieldExpression`, `AwaitExpression`, `OptionalChainExpression`, `OptionalChainInner`, `ChainExpression`, `SpreadElement`, `RestElement`, `ImportExpression`, `DecoratedExpression`, `TemplateLiteral`, `TemplateElement`, `TaggedTemplateExpression`, `FunctionExpression`, `FunctionParam`, `ArrowFunctionExpression`, `ClassExpression`, `TypeAssertion`, `SatisfiesExpression`.

- [ ] **Step 1: Create `expression.rs` and move all remaining expression items into it**

Move **verbatim** from `lib.rs` everything still left that is an expression type: the `Expression` enum + `impl AsRef<Expression> for Expression` (orig. ~449–525); `BinaryExpression`, `UnaryExpression`, `CallExpression`, `MemberExpression` (~527–554); and every remaining expression struct/enum in the "new types" region (~882–1133): `NewExpression`, `MetaProperty`, `TemplateLiteral`, `TemplateElement`, `TaggedTemplateExpression`, `UpdateExpression`, `UpdateOperator`, `AssignmentExpression`, `AssignmentOperator`, `LogicalExpression`, `LogicalOperator`, `ConditionalExpression`, `SequenceExpression`, `ParenthesizedExpression`, `YieldExpression`, `AwaitExpression`, `OptionalChainExpression`, `OptionalChainInner`, `ChainExpression`, `SpreadElement`, `RestElement`, `ImportExpression`, `DecoratedExpression`, `FunctionExpression`, `FunctionParam`, `ArrowFunctionExpression`, `ClassExpression`, `TypeAssertion`, `SatisfiesExpression`.

After this move, `lib.rs` should contain **no type definitions at all**.

Prepend to `crates/kali_ast/src/expression.rs`:
```rust
//! Expression node types and the unified `Expression` enum.

use crate::{
    ArrayExpression, BlockStatement, ClassBody, JsxElement, JsxFragment, LiteralValue,
    ObjectExpression,
};
```

- [ ] **Step 2: Reduce `lib.rs` to a pure facade**

`lib.rs` should now read exactly as the crate docs followed by module declarations and glob re-exports (order the `pub use` lines to match; no `use kali_common::Span;` at the root — it is unused there now):
```rust
//! AST definitions for TypeScript/JavaScript.
//!
//! This crate defines the Abstract Syntax Tree node types
//! and implements arena-based allocation for efficient AST construction.

mod builder;
mod declaration;
mod expression;
mod jsx;
mod literal;
mod module;
mod node;
mod statement;

pub use builder::*;
pub use declaration::*;
pub use expression::*;
pub use jsx::*;
pub use literal::*;
pub use module::*;
pub use node::*;
pub use statement::*;
```

- [ ] **Step 3: Verify green**

Run: `cargo test -p kali_ast`
Expected: `test result: ok. 5 passed`

- [ ] **Step 4: Commit**

```bash
git add crates/kali_ast/src/expression.rs crates/kali_ast/src/lib.rs
git commit -m "refactor(kali_ast): extract expression module, reduce lib.rs to facade [refactor]"
```

---

### Task 10: Final verification (proof diff, build, fmt, clippy)

**Files:**
- Modify: any kali_ast source file only if `cargo fmt` changes it.

**Interfaces:**
- Consumes: the baseline from Task 1.
- Produces: nothing (verification + formatting commit).

- [ ] **Step 1: Diff the test-basename multiset against the baseline**

Run:
```bash
cargo test -p kali_ast -- --list 2>/dev/null | grep ': test$' \
  | sed 's/.*:://; s/: test$//' | sort \
  > /tmp/claude-1000/-workspace/e8fa5ef1-c98f-412b-aeca-1ffc86df502b/scratchpad/kali_ast_list_after.txt
diff /tmp/claude-1000/-workspace/e8fa5ef1-c98f-412b-aeca-1ffc86df502b/scratchpad/kali_ast_list_before.txt \
     /tmp/claude-1000/-workspace/e8fa5ef1-c98f-412b-aeca-1ffc86df502b/scratchpad/kali_ast_list_after.txt
```
Expected: **no output** (identical 5 basenames — no test dropped or duplicated).

- [ ] **Step 2: Confirm the whole workspace still builds and the crate is green**

Run: `cargo build --workspace`
Expected: `Finished` with no errors.

Run: `cargo test -p kali_ast`
Expected: `test result: ok. 5 passed`

- [ ] **Step 3: Format and lint**

Run:
```bash
cargo fmt -p kali_ast
cargo clippy -p kali_ast --all-targets
```
Expected: `clippy` reports no warnings for `kali_ast`.

- [ ] **Step 4: Commit any formatting changes**

If `cargo fmt` changed files:
```bash
git add crates/kali_ast/src
git commit -m "style(kali_ast): cargo fmt [refactor]"
```
If nothing changed, skip the commit.

- [ ] **Step 5: Sanity-check the final layout**

Run: `ls crates/kali_ast/src && wc -l crates/kali_ast/src/lib.rs`
Expected: files `builder.rs builder_tests.rs declaration.rs declaration_tests.rs expression.rs jsx.rs lib.rs literal.rs module.rs node.rs node_tests.rs statement.rs`; `lib.rs` is ~20 lines (facade only). No `tests.rs`.
