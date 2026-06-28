# kali_lir Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Decompose `kali_lir`'s 215-line monolith `src/lib.rs` into a thin facade plus three per-concern sibling modules (`node.rs`, `program.rs`, `lower.rs`) with zero behavior change and a byte-identical public API.

**Architecture:** Pure code-motion mirroring the sibling `kali_mir`/`kali_hir` precedent. Each module is extracted incrementally — `node` first, then `program`, then `lower` — so the crate compiles and all 11 tests pass after every task. The facade re-exports each module's public items via `pub use`. One `pub(crate)` widening (`LirBuilder::nodes`) is required so `lower.rs` can read `builder.nodes` verbatim, exactly as `kali_mir` does.

**Tech Stack:** Rust workspace, `kali_lir` crate; depends on `kali_hir` (`FunctionFlavor`) and `kali_mir` (`MirNode`, `MirNodeId`, `MirNodeKind`, `MirProgram`).

## Global Constraints

- **Zero behavior change, byte-identical public API.** External consumers MUST compile unedited. Only `mod`/`pub use`/`use` wiring, `use` relocation, and the one `pub(crate) nodes` widening are allowed. Item bodies are moved **verbatim** — do not reformat, reorder, or "tidy" them.
- **Do NOT run `cargo fmt`.** Verbatim moves may leave some lines over 100 columns or stray blank lines; the repo's `cargo fmt --all --check` gate is already red on baseline, so these are not regressions. Running fmt would violate the verbatim mandate.
- **0-warning gate:** `cargo build -p kali_lir` must produce zero warnings after every task.
- **Test safety net:** the existing 11 tests in `src/tests.rs` ARE the regression suite. Do not add, edit, or relocate tests. `tests.rs` is unchanged across all tasks. After every task: `cargo test -p kali_lir` reports exactly 11 passed, 0 failed.
- **The `use super::*` test-gotcha:** `tests.rs` inherits `MirProgram` (from `kali_mir`) via `lib.rs`'s private `use kali_mir::{...}`. When that import migrates to `lower.rs` (Task 3), the facade must re-add `#[cfg(test)] use kali_mir::MirProgram;` or the tests lose access to `MirProgram`. See Task 3.
- **Integration is local-main ff-merge only — NEVER push to origin.** origin/main intentionally lags. See Task 4.
- Branch: `refactor/kali-lir-modularization`, off `main` at the baseline HEAD recorded in Task 0.
- Baseline (verified): `cargo test -p kali_lir` → 11 passed; `cargo build -p kali_lir` clean.

---

## File Structure

- `crates/kali_lir/src/node.rs` — **create.** LIR data model + arena builder: `LirNodeKind`, `LirNodeId`(+impl), `LirNode`(+impl), `LirBuilder`(+impl). Leaf module.
- `crates/kali_lir/src/program.rs` — **create.** Assembled program + structural validation: `LirProgram`(+impl `validate`), `validate_tree`. Leaf module.
- `crates/kali_lir/src/lower.rs` — **create.** MIR→LIR lowering engine: `LirLowerer`(+impl), `map_kind`. Leaf module.
- `crates/kali_lir/src/lib.rs` — **modify.** Becomes a ~18-line facade: module declarations + `pub use` re-exports + the `#[cfg(test)]` test-gotcha import + the `tests` module declaration.
- `crates/kali_lir/src/tests.rs` — **unchanged.** Do not touch.

Each new module file is a verbatim copy of a contiguous block from the baseline `lib.rs` (line numbers cited below from HEAD `dd055f63f`), plus a one-line `//!` header and the necessary `use` lines, exactly as `kali_mir`/`kali_hir` do for the same concerns.

---

### Task 0: Baseline & branch setup

**Files:**
- Read: `crates/kali_lir/src/lib.rs`, `crates/kali_lir/src/tests.rs`
- Write (scratch, git-ignored): `.superpowers/sdd/progress.md`

**Interfaces:**
- Consumes: the baseline source at `dd055f63f`.
- Produces: the `refactor/kali-lir-modularization` branch and the SDD ledger with the recorded baseline (HEAD, test count, test list). Every later task's verify step compares against this baseline.

- [ ] **Step 1: Confirm baseline is green**

Run:
```bash
git rev-parse --short HEAD
cargo build -p kali_lir 2>&1 | tail -3
cargo test -p kali_lir 2>&1 | tail -4
```
Expected: HEAD `dd055f63f`; build finishes with no warnings/errors; test result `ok. 11 passed; 0 failed`.

- [ ] **Step 2: Capture the baseline test list**

Run:
```bash
cargo test -p kali_lir -- --list 2>&1 | grep ': test' | sort > /tmp/kali_lir_baseline_tests.txt
wc -l /tmp/kali_lir_baseline_tests.txt
```
Expected: 11 lines. This file is the regression reference for Task 4's `--list` diff.

- [ ] **Step 3: Create the refactor branch**

Run:
```bash
git checkout -b refactor/kali-lir-modularization
```
Expected: `Switched to a new branch 'refactor/kali-lir-modularization'`.

- [ ] **Step 4: Initialize the SDD ledger**

Create `.superpowers/sdd/progress.md` (git-ignored scratch) with:
```markdown
# kali_lir modularization — SDD ledger

- Branch base: dd055f63f (main)
- Baseline: cargo test -p kali_lir → 11 passed; build clean, 0 warnings
- Test list captured: /tmp/kali_lir_baseline_tests.txt (11 tests)

## Tasks
- [ ] Task 1: extract node.rs
- [ ] Task 2: extract program.rs
- [ ] Task 3: extract lower.rs (+ pub(crate) widening + test-gotcha fix)
- [ ] Task 4: final verification + local-main ff-merge
```

- [ ] **Step 5: No commit yet**

Task 0 produces no source changes and the ledger is git-ignored — nothing to commit. Proceed to Task 1.

---

### Task 1: Extract `node.rs` (data model + arena builder)

**Files:**
- Create: `crates/kali_lir/src/node.rs`
- Modify: `crates/kali_lir/src/lib.rs`

**Interfaces:**
- Consumes: `kali_hir::FunctionFlavor` (for the `LirNode.function_flavor` field).
- Produces: `pub enum LirNodeKind`, `pub struct LirNodeId(pub u32)` (+`LirNodeId::new`), `pub struct LirNode` (+`LirNode::new`, `LirNode::with_text`), `pub struct LirBuilder` (+`LirBuilder::new`, `alloc`, `alloc_text`, `node_mut`, `into_nodes`). `LirBuilder::nodes` is widened from private to `pub(crate)` in this task — it is the **only** visibility change in the whole refactor, and it is required immediately because `LirLowerer::lower_program` (still in `lib.rs` this task) reads `builder.nodes` (lib.rs:132); once `LirBuilder` leaves `lib.rs`, that field must be crate-visible. This mirrors `kali_mir/src/node.rs:81` (`pub(crate) nodes: Vec<MirNode>`).

- [ ] **Step 1: Create `crates/kali_lir/src/node.rs` with verbatim content**

Create the file with exactly this content (bodies are verbatim from baseline `lib.rs:10–92`; only the `//!` header, the `use` line, and the `pub(crate)` prefix on `nodes` are additions):

```rust
//! LIR node kinds, ids, and the arena builder.

use kali_hir::FunctionFlavor;

/// LIR node kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LirNodeKind {
    Program,
    Block,
    Instruction,
    Value,
    Branch,
    Call,
    Literal,
    Unknown,
}

/// LIR node identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LirNodeId(pub u32);

impl LirNodeId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

/// LIR node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirNode {
    pub kind: LirNodeKind,
    pub text: Option<String>,
    pub children: Vec<LirNodeId>,
    pub function_flavor: Option<FunctionFlavor>,
}

impl LirNode {
    pub fn new(kind: LirNodeKind) -> Self {
        Self {
            kind,
            text: None,
            children: Vec::new(),
            function_flavor: None,
        }
    }

    pub fn with_text(kind: LirNodeKind, text: impl Into<String>) -> Self {
        Self {
            kind,
            text: Some(text.into()),
            children: Vec::new(),
            function_flavor: None,
        }
    }
}

/// LIR builder.
#[derive(Default)]
pub struct LirBuilder {
    pub(crate) nodes: Vec<LirNode>,
}

impl LirBuilder {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn alloc(&mut self, kind: LirNodeKind) -> LirNodeId {
        let id = LirNodeId(self.nodes.len() as u32);
        self.nodes.push(LirNode::new(kind));
        id
    }

    pub fn alloc_text(&mut self, kind: LirNodeKind, text: impl Into<String>) -> LirNodeId {
        let id = LirNodeId(self.nodes.len() as u32);
        self.nodes.push(LirNode::with_text(kind, text));
        id
    }

    pub fn node_mut(&mut self, id: LirNodeId) -> Option<&mut LirNode> {
        self.nodes.get_mut(id.0 as usize)
    }

    pub fn into_nodes(self) -> Vec<LirNode> {
        self.nodes
    }
}
```

- [ ] **Step 2: Rewrite `crates/kali_lir/src/lib.rs` to drop the moved items and re-export**

Replace the entire file with exactly this (the module doc, the two original `use` lines, and the `LirProgram`/`LirLowerer`/`map_kind`/`validate_tree` blocks remain verbatim and in place; the node items are gone; the new `mod node;` + `pub use node::{...}` lines are inserted after the `use` lines):

```rust
//! Low-level IR (LIR) for the Kali compiler.
//!
//! LIR is a linearized, codegen-oriented view of MIR. The current Phase-1
//! implementation keeps the lowering deterministic and structurally faithful so
//! later WASM emission can build on a stable node order.

pub use kali_hir::FunctionFlavor;
use kali_mir::{MirNode, MirNodeId, MirNodeKind, MirProgram};

mod node;
pub use node::{LirBuilder, LirNode, LirNodeKind, LirNodeId};

/// LIR lowering result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirProgram {
    pub root: LirNodeId,
    pub nodes: Vec<LirNode>,
}

impl LirProgram {
    /// Validate the structural consistency of the lowered LIR tree.
    pub fn validate(&self) -> Result<(), String> {
        validate_tree(
            "LIR",
            self.root,
            &self.nodes,
            |node| &node.children,
            |id| id.0 as usize,
        )
    }
}

/// LIR lowering.
#[derive(Default)]
pub struct LirLowerer;

impl LirLowerer {
    pub fn new() -> Self {
        Self
    }

    pub fn lower_mir(&self, _mir: MirNodeId) -> LirNodeId {
        LirNodeId::new(0)
    }

    pub fn lower_program(&self, mir: &MirProgram) -> LirProgram {
        let mut builder = LirBuilder::new();
        let root = self.lower_mir_node(&mut builder, &mir.nodes, mir.root);
        LirProgram {
            root,
            nodes: builder.nodes,
        }
    }

    fn lower_mir_node(
        &self,
        builder: &mut LirBuilder,
        nodes: &[MirNode],
        id: MirNodeId,
    ) -> LirNodeId {
        let node = &nodes[id.0 as usize];
        let kind = map_kind(&node.kind);
        let lir_id = match node.text.as_ref() {
            Some(text) => builder.alloc_text(kind, text.clone()),
            None => builder.alloc(kind),
        };
        if let Some(lir_node) = builder.node_mut(lir_id) {
            lir_node.function_flavor = node.function_flavor;
        }
        let mut children = Vec::with_capacity(node.children.len());
        for child in &node.children {
            children.push(self.lower_mir_node(builder, nodes, *child));
        }
        if let Some(lir_node) = builder.node_mut(lir_id) {
            lir_node.children = children;
        }
        lir_id
    }
}

fn map_kind(kind: &MirNodeKind) -> LirNodeKind {
    match kind {
        MirNodeKind::Program => LirNodeKind::Program,
        MirNodeKind::Block => LirNodeKind::Block,
        MirNodeKind::Function => LirNodeKind::Instruction,
        MirNodeKind::Decl => LirNodeKind::Instruction,
        MirNodeKind::Expr => LirNodeKind::Value,
        MirNodeKind::Call => LirNodeKind::Call,
        MirNodeKind::Literal => LirNodeKind::Literal,
        MirNodeKind::ControlFlow => LirNodeKind::Branch,
        MirNodeKind::Unknown => LirNodeKind::Unknown,
    }
}

fn validate_tree<Node, Id>(
    label: &str,
    root: Id,
    nodes: &[Node],
    children: impl Fn(&Node) -> &[Id],
    to_index: impl Fn(Id) -> usize,
) -> Result<(), String>
where
    Id: Copy,
{
    if nodes.is_empty() {
        return Err(format!("{label} tree contains no nodes"));
    }

    let root_index = to_index(root);
    if root_index >= nodes.len() {
        return Err(format!(
            "{label} root node id {root_index} is out of bounds for {} nodes",
            nodes.len()
        ));
    }

    for (index, node) in nodes.iter().enumerate() {
        for child in children(node) {
            let child_index = to_index(*child);
            if child_index >= nodes.len() {
                return Err(format!(
                    "{label} node {index} references child node id {child_index} outside the node table of {} nodes",
                    nodes.len()
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
```

- [ ] **Step 3: Verify build is clean (0 warnings)**

Run:
```bash
cargo build -p kali_lir 2>&1 | tail -5
```
Expected: no warnings, no errors. (If you see `field `nodes` of struct `LirBuilder` is private` at `lib.rs`'s `nodes: builder.nodes`, the `pub(crate)` prefix in `node.rs` is missing — re-check Step 1.)

- [ ] **Step 4: Verify all 11 tests still pass**

Run:
```bash
cargo test -p kali_lir 2>&1 | tail -4
```
Expected: `test result: ok. 11 passed; 0 failed; 0 ignored`.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_lir/src/node.rs crates/kali_lir/src/lib.rs
git commit -m "refactor(kali_lir): extract node module [refactor]"
```

- [ ] **Step 6: Update the SDD ledger**

In `.superpowers/sdd/progress.md`, tick `- [x] Task 1: extract node.rs`.

---

### Task 2: Extract `program.rs` (program + validation)

**Files:**
- Create: `crates/kali_lir/src/program.rs`
- Modify: `crates/kali_lir/src/lib.rs`

**Interfaces:**
- Consumes: `crate::{LirNode, LirNodeId}` (re-exported at the crate root by Task 1's `pub use node::{...}`) for the `LirProgram` fields.
- Produces: `pub struct LirProgram` (+`LirProgram::validate`). The private free fn `validate_tree` moves with it (only `LirProgram::validate` calls it).

- [ ] **Step 1: Create `crates/kali_lir/src/program.rs` with verbatim content**

Create the file with exactly this content (`LirProgram` + `validate` are verbatim from baseline `lib.rs:94–112`; `validate_tree` is verbatim from `lib.rs:176–211`; the `//!` header and `use crate::{...}` line are additions, mirroring `kali_mir/src/program.rs`'s `use crate::{...}` style):

```rust
//! Assembled LIR program and its structural validation.

use crate::{LirNode, LirNodeId};

/// LIR lowering result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirProgram {
    pub root: LirNodeId,
    pub nodes: Vec<LirNode>,
}

impl LirProgram {
    /// Validate the structural consistency of the lowered LIR tree.
    pub fn validate(&self) -> Result<(), String> {
        validate_tree(
            "LIR",
            self.root,
            &self.nodes,
            |node| &node.children,
            |id| id.0 as usize,
        )
    }
}

fn validate_tree<Node, Id>(
    label: &str,
    root: Id,
    nodes: &[Node],
    children: impl Fn(&Node) -> &[Id],
    to_index: impl Fn(Id) -> usize,
) -> Result<(), String>
where
    Id: Copy,
{
    if nodes.is_empty() {
        return Err(format!("{label} tree contains no nodes"));
    }

    let root_index = to_index(root);
    if root_index >= nodes.len() {
        return Err(format!(
            "{label} root node id {root_index} is out of bounds for {} nodes",
            nodes.len()
        ));
    }

    for (index, node) in nodes.iter().enumerate() {
        for child in children(node) {
            let child_index = to_index(*child);
            if child_index >= nodes.len() {
                return Err(format!(
                    "{label} node {index} references child node id {child_index} outside the node table of {} nodes",
                    nodes.len()
                ));
            }
        }
    }

    Ok(())
}
```

- [ ] **Step 2: Rewrite `crates/kali_lir/src/lib.rs` to drop `LirProgram`/`validate_tree` and re-export**

Replace the entire file with exactly this (the `LirProgram`/`impl LirProgram`/`validate_tree` block is removed; `mod program;` + `pub use program::LirProgram;` are added; everything else is unchanged from the end of Task 1):

```rust
//! Low-level IR (LIR) for the Kali compiler.
//!
//! LIR is a linearized, codegen-oriented view of MIR. The current Phase-1
//! implementation keeps the lowering deterministic and structurally faithful so
//! later WASM emission can build on a stable node order.

pub use kali_hir::FunctionFlavor;
use kali_mir::{MirNode, MirNodeId, MirNodeKind, MirProgram};

mod node;
mod program;
pub use node::{LirBuilder, LirNode, LirNodeKind, LirNodeId};
pub use program::LirProgram;

/// LIR lowering.
#[derive(Default)]
pub struct LirLowerer;

impl LirLowerer {
    pub fn new() -> Self {
        Self
    }

    pub fn lower_mir(&self, _mir: MirNodeId) -> LirNodeId {
        LirNodeId::new(0)
    }

    pub fn lower_program(&self, mir: &MirProgram) -> LirProgram {
        let mut builder = LirBuilder::new();
        let root = self.lower_mir_node(&mut builder, &mir.nodes, mir.root);
        LirProgram {
            root,
            nodes: builder.nodes,
        }
    }

    fn lower_mir_node(
        &self,
        builder: &mut LirBuilder,
        nodes: &[MirNode],
        id: MirNodeId,
    ) -> LirNodeId {
        let node = &nodes[id.0 as usize];
        let kind = map_kind(&node.kind);
        let lir_id = match node.text.as_ref() {
            Some(text) => builder.alloc_text(kind, text.clone()),
            None => builder.alloc(kind),
        };
        if let Some(lir_node) = builder.node_mut(lir_id) {
            lir_node.function_flavor = node.function_flavor;
        }
        let mut children = Vec::with_capacity(node.children.len());
        for child in &node.children {
            children.push(self.lower_mir_node(builder, nodes, *child));
        }
        if let Some(lir_node) = builder.node_mut(lir_id) {
            lir_node.children = children;
        }
        lir_id
    }
}

fn map_kind(kind: &MirNodeKind) -> LirNodeKind {
    match kind {
        MirNodeKind::Program => LirNodeKind::Program,
        MirNodeKind::Block => LirNodeKind::Block,
        MirNodeKind::Function => LirNodeKind::Instruction,
        MirNodeKind::Decl => LirNodeKind::Instruction,
        MirNodeKind::Expr => LirNodeKind::Value,
        MirNodeKind::Call => LirNodeKind::Call,
        MirNodeKind::Literal => LirNodeKind::Literal,
        MirNodeKind::ControlFlow => LirNodeKind::Branch,
        MirNodeKind::Unknown => LirNodeKind::Unknown,
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
```

- [ ] **Step 3: Verify build is clean (0 warnings)**

Run:
```bash
cargo build -p kali_lir 2>&1 | tail -5
```
Expected: no warnings, no errors.

- [ ] **Step 4: Verify all 11 tests still pass**

Run:
```bash
cargo test -p kali_lir 2>&1 | tail -4
```
Expected: `test result: ok. 11 passed; 0 failed; 0 ignored`.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_lir/src/program.rs crates/kali_lir/src/lib.rs
git commit -m "refactor(kali_lir): extract program module [refactor]"
```

- [ ] **Step 6: Update the SDD ledger**

In `.superpowers/sdd/progress.md`, tick `- [x] Task 2: extract program.rs`.

---

### Task 3: Extract `lower.rs` (lowering engine) + test-gotcha fix

**Files:**
- Create: `crates/kali_lir/src/lower.rs`
- Modify: `crates/kali_lir/src/lib.rs`

**Interfaces:**
- Consumes: `kali_mir::{MirNode, MirNodeId, MirNodeKind, MirProgram}` (the engine lowers from MIR) and `crate::{LirBuilder, LirNode, LirNodeKind, LirNodeId, LirProgram}` (re-exported at the crate root by Tasks 1–2).
- Produces: `pub struct LirLowerer` (+`new`, `lower_mir`, `lower_program`). The private `lower_mir_node` method and the private `map_kind` free fn move with it. After this task the facade is complete: `LirLowerer` is the last item to leave `lib.rs`.

**Test-gotcha note (load-bearing):** `tests.rs` uses `use super::*` and references `MirProgram` (in `parse_and_lower`'s return type) — currently inherited from `lib.rs`'s `use kali_mir::{..., MirProgram}`. Moving the lowerer (the sole production user of that import) to `lower.rs` makes the facade's `use kali_mir::{...}` line unused → it must be removed for the 0-warning gate → which severs the tests' access to `MirProgram`. The fix is the `#[cfg(test)] use kali_mir::MirProgram;` line in the facade below. Only `MirProgram` is re-imported — `tests.rs` does not reference `MirNode`, `MirNodeId`, or `MirNodeKind`.

- [ ] **Step 1: Create `crates/kali_lir/src/lower.rs` with verbatim content**

Create the file with exactly this content (`LirLowerer` + `lower_mir_node` are verbatim from baseline `lib.rs:114–160`; `map_kind` is verbatim from `lib.rs:162–174`; the `//!` header and the two `use` lines are additions, mirroring `kali_mir/src/lower.rs`):

```rust
//! Structural MIR→LIR lowering.

use kali_mir::{MirNode, MirNodeId, MirNodeKind, MirProgram};

use crate::{LirBuilder, LirNode, LirNodeKind, LirNodeId, LirProgram};

/// LIR lowering.
#[derive(Default)]
pub struct LirLowerer;

impl LirLowerer {
    pub fn new() -> Self {
        Self
    }

    pub fn lower_mir(&self, _mir: MirNodeId) -> LirNodeId {
        LirNodeId::new(0)
    }

    pub fn lower_program(&self, mir: &MirProgram) -> LirProgram {
        let mut builder = LirBuilder::new();
        let root = self.lower_mir_node(&mut builder, &mir.nodes, mir.root);
        LirProgram {
            root,
            nodes: builder.nodes,
        }
    }

    fn lower_mir_node(
        &self,
        builder: &mut LirBuilder,
        nodes: &[MirNode],
        id: MirNodeId,
    ) -> LirNodeId {
        let node = &nodes[id.0 as usize];
        let kind = map_kind(&node.kind);
        let lir_id = match node.text.as_ref() {
            Some(text) => builder.alloc_text(kind, text.clone()),
            None => builder.alloc(kind),
        };
        if let Some(lir_node) = builder.node_mut(lir_id) {
            lir_node.function_flavor = node.function_flavor;
        }
        let mut children = Vec::with_capacity(node.children.len());
        for child in &node.children {
            children.push(self.lower_mir_node(builder, nodes, *child));
        }
        if let Some(lir_node) = builder.node_mut(lir_id) {
            lir_node.children = children;
        }
        lir_id
    }
}

fn map_kind(kind: &MirNodeKind) -> LirNodeKind {
    match kind {
        MirNodeKind::Program => LirNodeKind::Program,
        MirNodeKind::Block => LirNodeKind::Block,
        MirNodeKind::Function => LirNodeKind::Instruction,
        MirNodeKind::Decl => LirNodeKind::Instruction,
        MirNodeKind::Expr => LirNodeKind::Value,
        MirNodeKind::Call => LirNodeKind::Call,
        MirNodeKind::Literal => LirNodeKind::Literal,
        MirNodeKind::ControlFlow => LirNodeKind::Branch,
        MirNodeKind::Unknown => LirNodeKind::Unknown,
    }
}
```

- [ ] **Step 2: Rewrite `crates/kali_lir/src/lib.rs` to its final facade form**

Replace the entire file with exactly this (the lowerer + `map_kind` are gone; the production `use kali_mir::{...}` line is gone — it now lives in `lower.rs`; `mod lower;` + `pub use lower::LirLowerer;` are added; the `#[cfg(test)] use kali_mir::MirProgram;` line is the test-gotcha fix):

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
use kali_mir::MirProgram;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
```

- [ ] **Step 3: Verify build is clean (0 warnings)**

Run:
```bash
cargo build -p kali_lir 2>&1 | tail -5
```
Expected: no warnings, no errors. (If you see `unused import: kali_mir::{...}` in `lib.rs`, the production `use kali_mir::{...}` line was not removed — re-check Step 2. If you see it in `lower.rs`, the line was duplicated there — remove the duplicate from `lower.rs`.)

- [ ] **Step 4: Verify all 11 tests still pass (this is the gotcha checkpoint)**

Run:
```bash
cargo test -p kali_lir 2>&1 | tail -4
```
Expected: `test result: ok. 11 passed; 0 failed; 0 ignored`. (If you see `error[E0433]/cannot find type MirProgram in tests.rs`, the `#[cfg(test)] use kali_mir::MirProgram;` line is missing from the facade — re-check Step 2.)

- [ ] **Step 5: Commit**

```bash
git add crates/kali_lir/src/lower.rs crates/kali_lir/src/lib.rs
git commit -m "refactor(kali_lir): extract lower module [refactor]"
```

- [ ] **Step 6: Update the SDD ledger**

In `.superpowers/sdd/progress.md`, tick `- [x] Task 3: extract lower.rs (+ pub(crate) widening + test-gotcha fix)`.

---

### Task 4: Final verification & local-main integration

**Files:**
- Read-only verification across the workspace.

**Interfaces:**
- Consumes: the completed branch from Task 3.
- Produces: the merged `main` (local only) with the modularized `kali_lir`, the deleted feature branch, and the verified `--list` diff against the Task 0 baseline.

- [ ] **Step 1: Confirm the full kali_lir suite is green and 0-warning**

Run:
```bash
cargo build -p kali_lir 2>&1 | tail -3
cargo test -p kali_lir 2>&1 | tail -4
```
Expected: build 0 warnings; `test result: ok. 11 passed; 0 failed`.

- [ ] **Step 2: Confirm the test list is identical to baseline**

Run:
```bash
cargo test -p kali_lir -- --list 2>&1 | grep ': test' | sort > /tmp/kali_lir_final_tests.txt
diff /tmp/kali_lir_baseline_tests.txt /tmp/kali_lir_final_tests.txt
```
Expected: empty diff (no output from `diff`). Same 11 tests, same names — confirming zero behavior/test change.

- [ ] **Step 3: Confirm external consumers compile unedited (byte-identical public API)**

Run:
```bash
cargo build -p kali_cli 2>&1 | tail -5
```
Expected: builds successfully with no `kali_lir`-related errors. (`kali_cli` transitively consumes `kali_lir`; an unedited build confirms the public API surface is unchanged.) Note: `kali_cli` has 2 PRE-EXISTING `build_bundles_*` integration-test failures (codegen/bundling, unrelated to this refactor) — those are not a concern; only the *build* must succeed here.

- [ ] **Step 4: ff-merge into local main (NEVER push to origin)**

Run:
```bash
git checkout main
git merge --ff-only refactor/kali-lir-modularization
```
Expected: `Fast-forward` to the branch tip. If git refuses with "not possible to fast-forward," STOP — do not create a merge commit; investigate (the branch base must have drifted).

- [ ] **Step 5: Re-verify on merged main**

Run:
```bash
git rev-parse --short HEAD
cargo build -p kali_lir 2>&1 | tail -3
cargo test -p kali_lir 2>&1 | tail -4
```
Expected: new HEAD at the branch tip; build 0 warnings; `11 passed; 0 failed`.

- [ ] **Step 6: Delete the feature branch**

Run:
```bash
git branch -d refactor/kali-lir-modularization
```
Expected: `Deleted branch refactor/kali-lir-modularization`.

- [ ] **Step 7: Finalize the SDD ledger**

In `.superpowers/sdd/progress.md`, tick `- [x] Task 4: final verification + local-main ff-merge` and record the merged main HEAD.

---
