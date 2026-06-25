# Kali `kali_ast` Modularization — Design

**Date:** 2026-06-25
**Status:** Approved (design)
**Scope:** Apply the validated crate-modularization pattern to `kali_ast` (rollout iteration 10).
**Parent design:** [`2026-06-23-kali-crate-modularization-design.md`](./2026-06-23-kali-crate-modularization-design.md)

## Problem

`kali_ast/src/lib.rs` is a single 1,164-line file holding the entire
TypeScript/JavaScript AST definition — roughly 70 node types plus the arena
storage types. Tests live in a sibling `tests.rs` (5 tests). The crate is
foundational: most of the workspace depends on `kali_ast::*`, so the file is
read and navigated constantly, yet it has no internal structure.

This is the next crate in the parent design's documented rollout order
(`… parser, ast, error, sandbox, …`); 9 crates are already done (types, codegen,
runtime, optimize, common, parser, hir, mir, npm).

## Goal & Hard Constraints

Inherited verbatim from the parent design — **pure structural refactor, zero
behavior change.**

- The exact same set of tests exists and passes before and after.
- `lib.rs` becomes a thin **facade** (`mod` declarations + `pub use`
  re-exports). Every external path (`kali_ast::Expression`,
  `kali_ast::ASTBuilder`, …) keeps resolving. No public API churn.
- Unit tests live in **sibling `*_tests.rs` files wired via `#[path = "…"] mod`**
  (AGENTS.md convention), not inline `#[cfg(test)]` modules.

### Proof obligation

```
cargo test -p kali_ast -- --list   # snapshot test names → diff after refactor
cargo test -p kali_ast             # must stay green at every commit
```

The `--list` snapshot guards against silently dropping or duplicating any of the
5 tests while they are relocated.

## Source Decomposition Strategy

Unlike the impl-heavy crates done so far (one giant `impl` split by
responsibility), `kali_ast` is **pure data**: ~70 small
`#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]`
structs and enums, with the only meaningful `impl`s being the arena types
(`ASTBuilder`, `AST`) and the hand-written `NodeId` serde / `NodeKind` `PartialEq`.

The honest decomposition is therefore **flat sibling modules grouped by
syntactic category** — the same flat-type-module shape `kali_mir` adopted. The
parent design's "Approach B" (flat siblings) was rejected only for the
impl-heavy pilot (`kali_types`), where peer impl files had no grouping; for a
pure-data crate, grouping by AST category *is* the natural structure. No logic
is rewritten; types move verbatim.

### Target layout (`crates/kali_ast/src/`)

```
lib.rs          facade: crate docs + `mod` decls + `pub use <mod>::*` re-exports
node.rs         NodeId (+ serde impls, Display, ctors), Node (+ impl),
                NodeKind (+ PartialEq/Eq), ModuleItem type alias        (+ node_tests.rs)
builder.rs      ASTBuilder (+ impl, Default), AST (+ impl, Default,
                From<ASTBuilder>)                                       (+ builder_tests.rs)
statement.rs    Statement enum + statement structs: ExpressionStatement,
                Break/Continue/With/Return/Labeled/If/Switch (+ SwitchCase)/
                Throw/Try (+ CatchClause)/Debugger/Block/For (+ ForInit)/
                ForIn (+ ForInLefthand)/ForOf (+ ForOfLefthand)/While/DoWhile
declaration.rs  FunctionDeclaration, ClassDeclaration, ClassBody,
                MethodDefinition, VariableDeclaration, VariableDeclarator,
                TypeAliasDeclaration, InterfaceDeclaration, PropertySignature,
                EnumDeclaration, EnumMember                             (+ declaration_tests.rs)
module.rs       ImportDeclaration, ImportSpecifier, ImportNamedSpecifier,
                ImportName, ExportDeclaration, ExportNamedDeclaration,
                ExportSpecifier, ExportAllDeclaration,
                ExportDefaultDeclaration, ExportTypeDeclaration
expression.rs   Expression enum (+ AsRef) + operand structs (Binary/Unary/
                Call/Member/New/MetaProperty) + operators (Update/Assignment/
                Logical + their exprs) + Conditional/Sequence/Parenthesized/
                Yield/Await/OptionalChain (+ inner)/Chain/Spread/Rest/
                ImportExpression/DecoratedExpression + Template (+ Element)/
                TaggedTemplate + FunctionExpression (+ FunctionParam)/
                ArrowFunctionExpression/ClassExpression + TypeAssertion/
                SatisfiesExpression
literal.rs      LiteralValue, ArrayExpression, ObjectExpression,
                ObjectProperty, PropertyName, ObjectPropertyKind,
                ExpressionOrSpread
jsx.rs          JsxElement, JsxOpeningElement, JsxChild,
                JsxExpressionContainer, JsxFragment, JsxName,
                JsxAttributeItem, JsxAttribute, JsxAttributeValue,
                JsxSpreadAttribute, JsxSelfClosingElement, JsxClosingElement
```

Module groupings follow the syntactic categories already implicit in the file's
own comment banners (statements, declarations, expression types, "missing
types", JSX). The exact placement of any individual type is settled during
implementation; the structure above is the target shape, not a frozen
file-by-file contract. A single flat `expression.rs` (~450 lines) was chosen
over an `expression/` subtree to keep the pure-data shape simple and review
churn low; it remains well under the size of the monoliths being eliminated.

### Cross-module references

The categories reference each other densely:

- `statement.rs` → `Expression`, `VariableDeclaration`, `BlockStatement`
  (Block lives in `statement.rs`), and the import/export declarations it
  enumerates in `Statement`.
- `expression.rs` → `BlockStatement` (function bodies), `ClassBody` /
  `MethodDefinition` (class expressions).
- `declaration.rs` → `Expression`, `BlockStatement`.
- `module.rs` → `Expression`, `FunctionDeclaration`, `ClassDeclaration`,
  `ExportSpecifier`.
- `node.rs` → `Span` (from `kali_common`).

Each module resolves siblings via `use crate::{…}` at its head, matching the
named-import ethos of the already-modularized crates. `ClassBody` and
`MethodDefinition` live in `declaration.rs`; `expression.rs` imports them.

### Facade re-export

`lib.rs` re-exports each module with a glob:

```rust
pub use node::*;
pub use statement::*;
pub use declaration::*;
pub use module::*;
pub use expression::*;
pub use literal::*;
pub use jsx::*;
pub use builder::*;
```

Glob re-export (rather than enumerating ~70 names) guarantees every existing
`kali_ast::TypeName` path keeps resolving with zero public-API churn.

## Test Decomposition

Five tests move verbatim (already meaningfully named — no renaming, no
`test_support` builders needed) into the sibling `*_tests.rs` of the module each
exercises:

| Test | Destination |
|---|---|
| `test_node_id` | `node_tests.rs` |
| `test_ast_builder` | `builder_tests.rs` |
| `test_ast_conversion` | `builder_tests.rs` |
| `test_function_kind_metadata_survives_serde_roundtrip` | `declaration_tests.rs` |
| `test_ast_roundtrips_default_export_anonymous_generator_function_declaration` | `declaration_tests.rs` |

The two serde round-trip tests primarily exercise `FunctionDeclaration`,
`ClassDeclaration`/`ClassExpression`, and `ExportDefaultDeclaration`'s `#[serde]`
metadata, so they belong with `declaration.rs`. Each destination module is wired
with:

```rust
#[cfg(test)]
#[path = "node_tests.rs"]
mod node_tests;
```

The original `src/tests.rs` is deleted once empty. No `kali_test_support` or
per-crate `test_support/` module is introduced — the test surface is too small
to benefit.

## Execution & Verification Rhythm

Small, reviewable commits; `cargo test -p kali_ast` green after each:

1. Capture the `cargo test -p kali_ast -- --list` baseline (5 tests).
2. Extract source modules one category at a time behind the facade
   (`node` → `builder` → `statement` → `declaration` → `module` →
   `expression` → `literal` → `jsx`), keeping `lib.rs` re-exporting throughout.
3. Relocate the 5 tests into their matching sibling `*_tests.rs`; delete the
   emptied `tests.rs`.
4. Final check: diff against the `--list` baseline (same 5 names), run
   `cargo fmt` and `cargo clippy -p kali_ast`, confirm `cargo build --workspace`
   and `cargo test -p kali_ast` are green.

This crate fits the mechanical pattern cleanly and reuses the parent design
directly; no novel structure is required.
