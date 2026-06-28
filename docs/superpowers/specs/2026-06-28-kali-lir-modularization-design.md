# kali_lir modularization — design (20th in series)

Date: 2026-06-28
Status: approved
Crate: `kali_lir` (20th crate in the kali workspace modularization series; kali_fmt was 19th)

## Goal & invariant

Pure code-motion. Decompose the single monolith `src/lib.rs` (215 lines) into a thin facade plus three per-concern sibling modules with **zero behavior change** and a **byte-identical public API**. External consumers MUST compile unedited.

Allowed changes only: `mod` declarations, `pub use` re-exports, and `use` relocation. Item bodies are moved **verbatim**. Do **not** run `cargo fmt` (verbatim moves may push some lines over 100 columns or leave stray blank lines; the repo's `cargo fmt --all --check` gate is already red on baseline, so these are not regressions).

**This crate needs exactly one visibility widening** — `LirBuilder::nodes` → `pub(crate)` (see "Visibility" below) — mirroring `kali_mir`'s `node.rs:81`, which widens the identical field for the identical reason.

## Baseline (branch base)

`cargo test -p kali_lir`: 11 tests pass; `cargo build -p kali_lir` clean. Record exact branch-base HEAD and test count in the SDD ledger before starting.

## Current shape

- `src/lib.rs` (215 lines): the LIR data model (`LirNodeKind`, `LirNodeId`+impl, `LirNode`+impl), the arena `LirBuilder`+impl, the assembled `LirProgram`+impl (`validate`), the `LirLowerer`+impl (the MIR→LIR lowering engine), and two free helpers — `map_kind` (MIR→LIR kind mapping, used only by the lowerer) and `validate_tree` (generic structural validator, used only by `LirProgram::validate`). All items are already `pub`; the helpers and `LirBuilder::nodes` / `LirLowerer::lower_mir_node` are private.
- `src/tests.rs`: co-located, declared in `lib.rs` via `#[path = "tests.rs"]`, uses `use super::*`. 11 tests exercise the public `LirLowerer::lower_program`, `LirProgram::validate`, and the public node types/`FunctionFlavor` re-export. Tests also reference `MirProgram` (from `kali_mir`) — inherited from `lib.rs`'s private `use kali_mir::{...}` via `use super::*` (used only in `parse_and_lower`'s return type).

## Approach

Mirror the established sibling-IR precedent exactly. `kali_mir` — the crate `LirLowerer` lowers *from* — is decomposed into `node.rs` (kinds/ids/node/builder), `program.rs` (program + `validate` + the `validate_tree` helper), and `lower.rs` (lowerer + `map_kind`), wired with `use crate::{...}` and re-exported from a thin facade. `kali_hir` uses the same shape. Applying the identical layout to `kali_lir` is the lowest-surprise choice: reviewers recognize it instantly, and the module the lowerer consumes has the same internal structure.

Three sibling modules, one verbatim move each:

- `node.rs` — the data model + arena builder.
- `program.rs` — the assembled program + structural validation (including the `validate_tree` helper, co-located exactly as in `kali_mir`).
- `lower.rs` — the lowering engine + `map_kind`.

The facade retains only the `pub use kali_hir::FunctionFlavor;` re-export and the per-module re-exports. No `pub(crate)` widening is required: every moved item is already `pub`, and every private helper stays private within the module that owns it.

## Target layout

### `node.rs` (~75 lines) — leaf module

Header: `//! LIR node kinds, ids, and the arena builder.`

```rust
use kali_hir::FunctionFlavor;

/// LIR node kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LirNodeKind { /* <verbatim lib.rs 12–21> */ }

/// LIR node identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LirNodeId(pub u32);

impl LirNodeId { /* <verbatim 27–31> */ }

/// LIR node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirNode { /* <verbatim 35–40> */ }

impl LirNode { /* <verbatim 42–60> */ }

/// LIR builder.
#[derive(Default)]
pub struct LirBuilder {
    pub(crate) nodes: Vec<LirNode>,   // <verbatim 65; widened `pub(crate)` — see "Visibility">
}

impl LirBuilder { /* <verbatim 68–92> */ }
```

`LirBuilder::nodes` is widened from private to `pub(crate)` (the single visibility change in this crate) so `lower.rs`'s `LirLowerer::lower_program` can read `builder.nodes` verbatim. The `use kali_hir::FunctionFlavor;` line migrates from `lib.rs` (`LirNode.function_flavor` field).

### `program.rs` (~45 lines) — leaf module

Header: `//! Assembled LIR program and its structural validation.`

```rust
use crate::{LirNode, LirNodeId};

/// LIR lowering result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirProgram { /* <verbatim 96–99> */ }

impl LirProgram {
    /// Validate the structural consistency of the lowered LIR tree.
    pub fn validate(&self) -> Result<(), String> { /* <verbatim 103–111> */ }
}

fn validate_tree<Node, Id>(/* <verbatim 176–211, stays private> */) { }
```

`validate_tree` stays private — only `LirProgram::validate` calls it. The `use crate::{LirNode, LirNodeId};` wiring mirrors `kali_mir`'s `program.rs` `use crate::{...}` style.

### `lower.rs` (~60 lines) — leaf module

Header: `//! Structural MIR→LIR lowering.`

```rust
use kali_mir::{MirNode, MirNodeId, MirNodeKind, MirProgram};

use crate::{LirBuilder, LirNode, LirNodeKind, LirNodeId, LirProgram};

/// LIR lowering.
#[derive(Default)]
pub struct LirLowerer;

impl LirLowerer { /* <verbatim 118–160; `lower_mir_node` stays private> */ }

fn map_kind(kind: &MirNodeKind) -> LirNodeKind { /* <verbatim 162–174, stays private> */ }
```

`map_kind` and `LirLowerer::lower_mir_node` stay private. The `use kali_mir::{...}` line migrates from `lib.rs` (the engine is the sole production user of those symbols).

### `lib.rs` facade (~20 lines)

```rust
//! Low-level IR (LIR) for the Kali compiler.
//!
//! LIR is a linearized, codegen-oriented view of MIR. The current Phase-1
//! implementation keeps the lowering deterministic and structurally faithful so
//! later WASM emission can build on a stable node order.

mod lower;
mod node;
mod program;

pub use kali_hir::FunctionFlavor;
pub use lower::LirLowerer;
pub use node::{LirBuilder, LirNode, LirNodeKind, LirNodeId};
pub use program::LirProgram;

#[cfg(test)]
use kali_mir::MirProgram; // test-gotcha fix — see "Test-gotcha fix" below

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
```

The facade keeps `pub use kali_hir::FunctionFlavor;` unchanged — it was already a public re-export in the original (line 7). `node.rs` adds its own private `use kali_hir::FunctionFlavor;` for the `LirNode.function_flavor` field (mirroring `kali_mir`'s `node.rs`). The `use kali_mir::{...}` line migrates entirely to `lower.rs` (the engine is its sole production user); nothing in the production facade references `kali_mir` anymore.

## Visibility

Exactly one `pub(crate)` widening is required: `LirBuilder::nodes` (private today) becomes `pub(crate) nodes: Vec<LirNode>;`. `LirLowerer::lower_program` (in `lower.rs`) reads `builder.nodes` verbatim (lib.rs:132), and once `LirBuilder` lives in `node.rs` and `LirLowerer` in `lower.rs`, that cross-module field access needs `pub(crate)`. This mirrors `kali_mir` exactly — `kali_mir/src/node.rs:81` declares `pub(crate) nodes: Vec<MirNode>` for the identical `builder.nodes` access in `kali_mir/src/lower.rs:30`. Every other moved item (`LirNodeKind`, `LirNodeId`, `LirNode`, `LirBuilder`, `LirProgram`, `LirLowerer`) is already `pub` and is re-exported via `pub use`, preserving the byte-identical public surface. All other private implementation details — `map_kind`, `validate_tree`, `LirLowerer::lower_mir_node` — remain private within their owning module. `pub(crate)` is invisible to external consumers, so the public API is unchanged.

## Test-gotcha fix (the recurring `use super::*` cutoff)

`tests.rs` uses `use super::*` and inherits `MirProgram` (from `kali_mir`) via `lib.rs`'s private `use kali_mir::{MirNode, MirNodeId, MirNodeKind, MirProgram};`. `MirProgram` appears only in `parse_and_lower`'s return type; the other three `kali_mir` symbols are not referenced by the tests.

Moving the lowerer — the sole production user of that `use kali_mir::{...}` line — into `lower.rs` leaves the facade with no production reference to any `kali_mir` symbol, so the import becomes unused and must be removed for the 0-warning gate. Removing it severs the tests' `use super::*` access to `MirProgram`.

Fix: re-add `#[cfg(test)] use kali_mir::MirProgram;` to the facade (import-only, no public-surface change). `tests.rs` itself is unchanged. Only `MirProgram` is re-imported — `MirNode`, `MirNodeId`, and `MirNodeKind` are not needed by the tests.

## Execution & verification rhythm

On a `refactor/kali-lir-modularization` branch off main; confirm baseline build+test green before starting. The crate is small enough for a single mechanical commit (the three verbatim moves + facade wiring + the one-line `#[cfg(test)]` re-import). After:

1. `cargo build -p kali_lir` — 0 warnings.
2. `cargo test -p kali_lir` — same 11 tests pass (diff the `--list` output against baseline).
3. `cargo build -p kali_cli` (and any consumer of `kali_lir`) — compiles unedited, confirming the byte-identical public API.

Integration is **local-main ff-merge only — NEVER push to origin** (origin/main intentionally lags). Re-verify build+test on merged main, then delete the branch.
