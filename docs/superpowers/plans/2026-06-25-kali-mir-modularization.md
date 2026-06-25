# kali_mir Modularization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the 1,569-line `crates/kali_mir/src/lib.rs` monolith into focused
modules behind a thin facade, with zero behavior change.

**Architecture:** Six flat data-type modules (`ownership`, `layout`, `binding`,
`function`, `node`, `program`), a flat structural-lowering module (`lower`), and
an `analysis/` subtree that impl-splits the ~900-line `OwnershipAnalyzer` engine
by concern (`mod`/`scope`/`walk`/`infer`/`resolve`). `lib.rs` becomes a ~30-line
facade re-exporting the public API. Tests relocate into sibling `*_tests.rs`
files sharing a `test_support.rs` helper.

**Tech Stack:** Rust (workspace crate), `cargo test -p kali_mir`, `cargo clippy`,
`cargo fmt`.

**Design doc:** `docs/superpowers/specs/2026-06-25-kali-mir-modularization-design.md`

## Global Constraints

Every task's requirements implicitly include this section.

- **Pure structural refactor — ZERO behavior change.** Items are only relocated
  and visibility widened. No logic is rewritten, no assertions changed.
- **Verbatim text-movement.** Move method/fn/type bodies **byte-for-byte**,
  including blank-line separators between items AND the original's exact
  qualification style. Do **NOT** convert inline `kali_hir::Foo` refs to
  imported short names or vice versa. Do **NOT** reorder items, fields, match
  arms, or methods. Prior crates caught dropped blank lines and silent
  requalification — watch for both.
- **Visibility widened minimally to `pub(crate)`, never bare `pub`.** Items
  whose sole caller is co-located stay private.
- **`cargo test -p kali_mir` is GREEN after every commit** (36 unit tests; no
  integration-test directory exists for this crate).
- **Public API preserved byte-for-byte** at flat paths — all 18 public types
  keep their public fields/methods:
  `kali_mir::{OwnershipClass, ThreadBoundaryDisposition, ThreadBoundaryBinding,
  ThreadBoundaryProfile, LayoutDescriptor, MirBindingKind, MirBinding,
  BorrowedLifetime, MirFunctionKind, MirFunction, MirNodeKind, MirNodeId,
  PlaceRef, PlaceValue, MirNode, MirBuilder, MirProgram, MirLowerer}`.
  Downstream consumers (`kali_codegen`, `kali_optimize`, `kali_lir`, `kali_cli`)
  must keep compiling.
- **Tests in sibling `*_tests.rs`** wired via `#[cfg(test)] #[path = "…"] mod`
  (AGENTS.md §5), **not** inline `#[cfg(test)] mod`. The shared parse/lower
  helper lives in `test_support.rs` wired as plain `#[cfg(test)] mod test_support;`.
- **Commit convention:** `refactor(kali_mir): <summary> [refactor]` (or
  `test`/`style`/`docs` prefix as fits), one commit per task.
- **`lib.rs` ends as a ~30-line facade:** crate docs + `mod` decls (alphabetical)
  + `pub use` re-exports + `cfg(test)` wiring. No fns/structs/enums/impls/macros.

### Proof obligation (the authoritative gate)

The durable check is the **basename multiset** of test names (module-path
prefixes change as tests relocate, so a raw full-name diff would falsely fail):

```
cargo test -p kali_mir -- --list 2>/dev/null | grep ': test$' \
  | sed -E 's/^.*:://; s/: test$//' | sort
```

Baseline (captured pre-flight): **36** sorted basenames, **no duplicates**.
Compare before vs after the test split with `diff` → must be **empty**. Use
`sort` **without** `-u`.

---

## File Structure

Pre-flight item map (line numbers in the **unmodified** `lib.rs`; they shift as
items are removed, so locate by **name** when extracting — grep current lines
first).

| target module | items (pre-flight lines in lib.rs) |
|---|---|
| `ownership.rs` | `OwnershipClass` (14, impl 92), `ThreadBoundaryDisposition` (23), `ThreadBoundaryBinding` (32), `ThreadBoundaryProfile` (40, impl 44) |
| `layout.rs` | `LayoutDescriptor` (129, impl 144) |
| `binding.rs` | `MirBindingKind` (180), `MirBinding` (189, impl 206), `BorrowedLifetime` (200) |
| `function.rs` | `MirFunctionKind` (257), `MirFunction` (265, impl 272) |
| `node.rs` | `MirNodeKind` (301), `MirNodeId` (315, impl 317), `PlaceRef` (325, impl 327), `PlaceValue` (335, impl 337), `MirNode` (345, impl 352), `MirBuilder` (374, impl 378) |
| `program.rs` | `MirProgram` (402, impl 408), free fns `validate_tree` (497), `function_scope_name` (534) |
| `lower.rs` | `MirLowerer` (543, impl 545), free fn `map_kind` (600) |
| `analysis/mod.rs` | `OwnershipAnalyzer` struct (897) + entry methods `new`/`analyze_program`/`function_flavor`; support `UseContext` (665), `BindingState` (672, impl 682), `ScopeState` (789, impl 799); free fns `default_ownership` (696), `parameter_escape_flags` (705), `function_binding_escapes` (714), `finalise_binding` (746) |
| `analysis/scope.rs` | `OwnershipAnalyzer` methods: `push_scope`, `pop_scope_and_record`, `current_scope_label`, `current_scope_index`, `current_scope_mut`, `precollect_scope_bindings`, `define_binding`, `collect_import_bindings` |
| `analysis/walk.rs` | `OwnershipAnalyzer` methods: `walk_scope_node`, `resolve_use`, `resolve_binding`, `is_heap_store_target` |
| `analysis/infer.rs` | `OwnershipAnalyzer` methods: `infer_layout`, `infer_binary_layout`, `infer_unary_layout`, `resolve_binding_layout`, `layout_field_name`, `object_property_order_key` |
| `analysis/resolve.rs` | `OwnershipAnalyzer` methods: `function_parameter_escape_flags`, `resolve_function_target`, `function_target_from_node`, `function_name_from_recent_functions`, `next_function_name` |

The crate's only external imports today are
`use std::collections::{BTreeMap, BTreeSet};` and
`use kali_hir::{FunctionFlavor, HirNode, HirNodeId, HirNodeKind, LoweringResult as HirLoweringResult};`.
Each module imports only the names it uses (precise per-module `use` lines given
per task). Crate-internal cross-references resolve via `use crate::{…}` (all
public types are re-exported at the crate root by the facade).

---

### Task 1: Baseline + widen visibility for extraction

No relocation yet. Confirm green, capture the proof baseline, and widen exactly
the items that will be accessed across module boundaries to `pub(crate)`.

**Files:**
- Modify: `crates/kali_mir/src/lib.rs`

**Interfaces:**
- Produces: `pub(crate)` visibility on the engine internals + the three private
  cross-module helper methods + `MirBuilder.nodes`, so later extraction tasks
  compile when the items move to sibling modules. No public API change.

- [ ] **Step 1: Capture the basename-multiset baseline**

Run:
```bash
cargo test -p kali_mir -- --list 2>/dev/null | grep ': test$' \
  | sed -E 's/^.*:://; s/: test$//' | sort > /tmp/kali_mir_baseline.txt
wc -l /tmp/kali_mir_baseline.txt        # expect 36
sort /tmp/kali_mir_baseline.txt | uniq -d   # expect empty (no dups)
```

- [ ] **Step 2: Confirm a green starting point**

Run: `cargo test -p kali_mir`
Expected: `test result: ok. 36 passed`.

- [ ] **Step 3: Widen the three private helper methods on public types to `pub(crate)`**

These are called from a module other than the one they will live in. Add the
`pub(crate)` prefix; change nothing else.
- `ThreadBoundaryProfile::push_binding` (lib.rs ~45) — `fn push_binding` → `pub(crate) fn push_binding` (called from `function.rs` + `program.rs`).
- `ThreadBoundaryProfile::finalize` (lib.rs ~53) — `fn finalize` → `pub(crate) fn finalize` (called from `function.rs` + `program.rs`).
- `LayoutDescriptor::scalar` (lib.rs ~145) — `fn scalar` → `pub(crate) fn scalar` (called from `analysis/infer.rs`).

Leave `MirNode::new`/`MirNode::with_text` **private** (only `MirBuilder` calls
them, co-located in `node.rs`). Leave `validate_tree`/`function_scope_name`
**private** (only `MirProgram` calls them, co-located in `program.rs`). Leave
`map_kind` **private** (only `MirLowerer` calls it, co-located in `lower.rs`).

- [ ] **Step 4: Widen `MirBuilder.nodes` field to `pub(crate)`**

`MirBuilder` (lib.rs ~374): `nodes: Vec<MirNode>,` → `pub(crate) nodes: Vec<MirNode>,`
(read by `MirLowerer::lower_hir_result` in `lower.rs`).

- [ ] **Step 5: Widen the `OwnershipAnalyzer` engine to `pub(crate)`**

The struct, all 5 fields, and all ~26 methods become `pub(crate)` (the struct is
named by `lower.rs`; the impl splits across `analysis/` siblings that call each
other's methods and read each other's fields):
- `struct OwnershipAnalyzer<'a>` (lib.rs ~897) → `pub(crate) struct OwnershipAnalyzer<'a>`.
- Its 5 fields (`nodes`, `function_flavors`, `functions`, `scope_stack`,
  `synthetic_function_counter`) → each prefixed `pub(crate)`.
- Every method in `impl<'a> OwnershipAnalyzer<'a>` (the ~26 fns listed in the
  File Structure table for `analysis/{mod,scope,walk,infer,resolve}.rs`) →
  prefixed `pub(crate)`. `new` and `analyze_program` are also called from
  `lower.rs`, so they too are `pub(crate)`.

- [ ] **Step 6: Widen the analysis support types + free fns to `pub(crate)`**

- `enum UseContext` (lib.rs ~665) → `pub(crate) enum UseContext`.
- `struct BindingState` (lib.rs ~672) → `pub(crate) struct BindingState`; all 7
  fields prefixed `pub(crate)`; its `impl` methods (`new`) prefixed `pub(crate)`.
- `struct ScopeState` (lib.rs ~789) → `pub(crate) struct ScopeState`; all 7
  fields prefixed `pub(crate)`; its `impl` methods (`new`, `define`,
  `get_binding_index`, `get_binding_mut`, `alias_function`, `capture_binding`,
  `finalize`) prefixed `pub(crate)`.
- Free fns `default_ownership` (~696), `parameter_escape_flags` (~705),
  `function_binding_escapes` (~714), `finalise_binding` (~746) → each prefixed
  `pub(crate)`.

- [ ] **Step 7: Verify still green (no relocation yet)**

Run: `cargo test -p kali_mir`
Expected: `36 passed`. (Clippy may warn about now-`pub(crate)` items not yet
read across a module boundary; that is expected and clears as modules are
extracted. The per-commit gate here is TEST-GREEN, not clippy-clean.)

- [ ] **Step 8: Commit**

```bash
git add crates/kali_mir/src/lib.rs
git commit -m "refactor(kali_mir): widen private items to pub(crate) for extraction [refactor]"
```

---

### Task 2: Extract `layout.rs`

**Files:**
- Create: `crates/kali_mir/src/layout.rs`
- Modify: `crates/kali_mir/src/lib.rs`

**Interfaces:**
- Produces: `crate::layout::LayoutDescriptor`, re-exported as `kali_mir::LayoutDescriptor`.

- [ ] **Step 1: Create `layout.rs`**

Create `crates/kali_mir/src/layout.rs`. Header:
```rust
//! MIR memory layout descriptors.
```
Then move, **byte-for-byte**, the `LayoutDescriptor` enum (with its derives/docs)
and its `impl LayoutDescriptor` block (`scalar` — now `pub(crate)` — and the
public `fingerprint`) from `lib.rs`. Add only the imports the moved code uses
(compiler-driven; `LayoutDescriptor` is self-contained — likely no imports
beyond what the bodies reference).

- [ ] **Step 2: Wire into `lib.rs`**

Remove the moved items. Add `mod layout;` (alphabetical among `mod` decls) and
`pub use layout::LayoutDescriptor;` (alphabetical among `pub use`s).

- [ ] **Step 3: Run tests**

Run: `cargo test -p kali_mir`
Expected: `36 passed`.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_mir/src/layout.rs crates/kali_mir/src/lib.rs
git commit -m "refactor(kali_mir): extract layout module [refactor]"
```

---

### Task 3: Extract `ownership.rs`

**Files:**
- Create: `crates/kali_mir/src/ownership.rs`
- Modify: `crates/kali_mir/src/lib.rs`

**Interfaces:**
- Consumes: `crate::MirBinding` (still in `lib.rs` at this point; resolves via
  the crate root).
- Produces: `kali_mir::{OwnershipClass, ThreadBoundaryDisposition, ThreadBoundaryBinding, ThreadBoundaryProfile}`.

- [ ] **Step 1: Create `ownership.rs`**

Header:
```rust
//! Ownership classes and thread-boundary types for MIR analysis.

use std::collections::BTreeMap;

use crate::MirBinding;
```
Move **byte-for-byte**, in their original order: `OwnershipClass` enum +
`impl OwnershipClass`; `ThreadBoundaryDisposition` enum; `ThreadBoundaryBinding`
struct; `ThreadBoundaryProfile` struct + `impl ThreadBoundaryProfile` (with the
now-`pub(crate)` `push_binding`/`finalize` and public `in_scope`). Adjust the
import list compiler-driven if the bodies reference more/fewer names — but do
not alter the bodies' qualification style.

- [ ] **Step 2: Wire into `lib.rs`**

Remove the moved items. Add `mod ownership;` and
`pub use ownership::{OwnershipClass, ThreadBoundaryBinding, ThreadBoundaryDisposition, ThreadBoundaryProfile};`
(keep `pub use` lists alphabetized).

- [ ] **Step 3: Run tests**

Run: `cargo test -p kali_mir`
Expected: `36 passed`.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_mir/src/ownership.rs crates/kali_mir/src/lib.rs
git commit -m "refactor(kali_mir): extract ownership module [refactor]"
```

---

### Task 4: Extract `binding.rs`

**Files:**
- Create: `crates/kali_mir/src/binding.rs`
- Modify: `crates/kali_mir/src/lib.rs`

**Interfaces:**
- Consumes: `crate::{LayoutDescriptor, OwnershipClass, ThreadBoundaryBinding, ThreadBoundaryDisposition}`.
- Produces: `kali_mir::{MirBindingKind, MirBinding, BorrowedLifetime}`.

- [ ] **Step 1: Create `binding.rs`**

Header:
```rust
//! MIR binding types and borrowed-lifetime summaries.

use crate::{LayoutDescriptor, OwnershipClass, ThreadBoundaryBinding, ThreadBoundaryDisposition};
```
Move **byte-for-byte**, in original order: `MirBindingKind` enum; `MirBinding`
struct; `BorrowedLifetime` struct; `impl MirBinding`. Adjust imports
compiler-driven (e.g. if `BorrowedLifetime` is referenced by `impl MirBinding`,
it is same-module). Do not alter body qualification.

- [ ] **Step 2: Wire into `lib.rs`**

Remove the moved items. Add `mod binding;` and
`pub use binding::{BorrowedLifetime, MirBinding, MirBindingKind};`.

- [ ] **Step 3: Run tests**

Run: `cargo test -p kali_mir`
Expected: `36 passed`.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_mir/src/binding.rs crates/kali_mir/src/lib.rs
git commit -m "refactor(kali_mir): extract binding module [refactor]"
```

---

### Task 5: Extract `function.rs`

**Files:**
- Create: `crates/kali_mir/src/function.rs`
- Modify: `crates/kali_mir/src/lib.rs`

**Interfaces:**
- Consumes: `kali_hir::FunctionFlavor`, `std::collections::BTreeSet`,
  `crate::{BorrowedLifetime, MirBinding, ThreadBoundaryProfile}`.
- Produces: `kali_mir::{MirFunctionKind, MirFunction}`.

- [ ] **Step 1: Create `function.rs`**

Header:
```rust
//! MIR function records and per-function summaries.

use std::collections::BTreeSet;

use kali_hir::FunctionFlavor;

use crate::{BorrowedLifetime, MirBinding, ThreadBoundaryProfile};
```
Move **byte-for-byte**, in original order: `MirFunctionKind` enum; `MirFunction`
struct; `impl MirFunction` (`binding`, `borrowed_lifetimes`,
`thread_boundary_profile` — the last calls the now-`pub(crate)`
`ThreadBoundaryProfile::push_binding`/`finalize`). Adjust imports compiler-driven.

- [ ] **Step 2: Wire into `lib.rs`**

Remove the moved items. Add `mod function;` and
`pub use function::{MirFunction, MirFunctionKind};`.

- [ ] **Step 3: Run tests**

Run: `cargo test -p kali_mir`
Expected: `36 passed`.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_mir/src/function.rs crates/kali_mir/src/lib.rs
git commit -m "refactor(kali_mir): extract function module [refactor]"
```

---

### Task 6: Extract `node.rs`

**Files:**
- Create: `crates/kali_mir/src/node.rs`
- Modify: `crates/kali_mir/src/lib.rs`

**Interfaces:**
- Consumes: `kali_hir::FunctionFlavor`.
- Produces: `kali_mir::{MirNodeKind, MirNodeId, PlaceRef, PlaceValue, MirNode, MirBuilder}`.
  `MirBuilder.nodes` is `pub(crate)` (read by `lower.rs`).

- [ ] **Step 1: Create `node.rs`**

Header:
```rust
//! MIR node kinds, ids, place references, and the arena builder.

use kali_hir::FunctionFlavor;
```
Move **byte-for-byte**, in original order: `MirNodeKind` enum; `MirNodeId`
struct + impl; `PlaceRef` struct + impl; `PlaceValue` struct + impl; `MirNode`
struct + impl (`MirNode::new`/`with_text` stay **private** — same module as
their only caller `MirBuilder`); `MirBuilder` struct (`pub(crate) nodes`) + impl
(`new`, `alloc`, `alloc_text`, `node_mut`). Adjust imports compiler-driven.

- [ ] **Step 2: Wire into `lib.rs`**

Remove the moved items. Add `mod node;` and
`pub use node::{MirBuilder, MirNode, MirNodeId, MirNodeKind, PlaceRef, PlaceValue};`.

- [ ] **Step 3: Run tests**

Run: `cargo test -p kali_mir`
Expected: `36 passed`.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_mir/src/node.rs crates/kali_mir/src/lib.rs
git commit -m "refactor(kali_mir): extract node module [refactor]"
```

---

### Task 7: Extract `program.rs`

**Files:**
- Create: `crates/kali_mir/src/program.rs`
- Modify: `crates/kali_mir/src/lib.rs`

**Interfaces:**
- Consumes: `std::collections::BTreeSet`, `crate::{BorrowedLifetime, MirFunction,
  MirFunctionKind, MirNode, MirNodeId, ThreadBoundaryProfile}` (adjust to the
  exact set the bodies use).
- Produces: `kali_mir::MirProgram`. Free fns `validate_tree` and
  `function_scope_name` move with it and stay **private** (sole caller is
  `MirProgram`, co-located here).

- [ ] **Step 1: Create `program.rs`**

Header:
```rust
//! Assembled MIR program and its query/summary API.
```
Then the imports the moved code uses (compiler-driven — start from the Consumes
list above; `validate_tree` is generic, check its trait bounds for needed
names). Move **byte-for-byte**, in original order: `MirProgram` struct +
`impl MirProgram` (all 9 methods incl. `validate` which calls `validate_tree`,
and the `*_scope`/`borrowed_lifetimes*`/`thread_boundary_profile*` methods which
call `function_scope_name`); then the private free fns `validate_tree` and
`function_scope_name`. Keep both free fns bare `fn` (private).

- [ ] **Step 2: Wire into `lib.rs`**

Remove the moved items. Add `mod program;` and `pub use program::MirProgram;`.

- [ ] **Step 3: Run tests**

Run: `cargo test -p kali_mir`
Expected: `36 passed`.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_mir/src/program.rs crates/kali_mir/src/lib.rs
git commit -m "refactor(kali_mir): extract program module [refactor]"
```

---

### Task 8: Extract `lower.rs`

**Files:**
- Create: `crates/kali_mir/src/lower.rs`
- Modify: `crates/kali_mir/src/lib.rs`

**Interfaces:**
- Consumes: `kali_hir::{FunctionFlavor, HirNode, HirNodeId, HirNodeKind,
  LoweringResult as HirLoweringResult}`, `crate::{MirBuilder, MirNodeId,
  MirNodeKind, MirProgram}`, and `crate::OwnershipAnalyzer` (still in `lib.rs`
  at this point — resolves via crate root since it is `pub(crate)`).
- Produces: `kali_mir::MirLowerer`. Free fn `map_kind` moves with it and stays
  **private** (sole caller `MirLowerer`).

- [ ] **Step 1: Create `lower.rs`**

Header:
```rust
//! Structural HIR→MIR lowering.

use kali_hir::{FunctionFlavor, HirNode, HirNodeId, HirNodeKind, LoweringResult as HirLoweringResult};

use crate::{MirBuilder, MirNodeId, MirNodeKind, MirProgram, OwnershipAnalyzer};
```
Move **byte-for-byte**, in original order: `MirLowerer` struct + `impl MirLowerer`
(`new`, `lower_hir`, `lower_hir_result`, `lower_hir_node`, `function_flavor`),
then the private free fn `map_kind`. Adjust imports compiler-driven (do not
change the bodies' qualification — note the ~60 inline `MirNodeKind::…`/
`HirNodeKind::…` arms in `map_kind` move verbatim).

- [ ] **Step 2: Wire into `lib.rs`**

Remove the moved items. Add `mod lower;` and `pub use lower::MirLowerer;`.

- [ ] **Step 3: Run tests**

Run: `cargo test -p kali_mir`
Expected: `36 passed`.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_mir/src/lower.rs crates/kali_mir/src/lib.rs
git commit -m "refactor(kali_mir): extract lower module [refactor]"
```

---

### Task 9: Create the `analysis/` subtree (`mod.rs` parent + `scope.rs`)

After this task, `lib.rs` still holds the `OwnershipAnalyzer` struct, the support
types (`UseContext`/`BindingState`/`ScopeState`), and the 4 analysis free fns;
only the scope-management methods move out. Sibling `impl` blocks reach the
struct/support via `use crate::{…}` (all `pub(crate)` from Task 1).

**Files:**
- Create: `crates/kali_mir/src/analysis/mod.rs`
- Create: `crates/kali_mir/src/analysis/scope.rs`
- Modify: `crates/kali_mir/src/lib.rs`

**Interfaces:**
- Consumes (in `scope.rs`): `crate::{OwnershipAnalyzer, ScopeState}` plus any
  HIR/crate types the moved methods reference.
- Produces: the `analysis` module path; `analysis/scope.rs` carries an
  `impl OwnershipAnalyzer` block.

- [ ] **Step 1: Create `analysis/mod.rs` as the subtree parent**

```rust
//! Ownership/escape analysis engine (split by concern).

mod scope;
```
(Sibling `mod` decls are added by later tasks as each sibling is created.)

- [ ] **Step 2: Create `analysis/scope.rs` with an `impl OwnershipAnalyzer` block**

Header:
```rust
//! Scope-stack management for the ownership analyzer.

use crate::{OwnershipAnalyzer, ScopeState};
```
(Adjust the `use crate::{…}` set + add any `kali_hir`/`std` imports the moved
bodies reference — compiler-driven.) Then an `impl<'a> OwnershipAnalyzer<'a>`
block whose generic header reproduces the original **exactly**. Inside, move
**byte-for-byte** these methods (each already `pub(crate)`
from Task 1): `push_scope`, `pop_scope_and_record`, `current_scope_label`,
`current_scope_index`, `current_scope_mut`, `precollect_scope_bindings`,
`define_binding`, `collect_import_bindings`.

> Note: the sibling's `impl` block header must reproduce the original generic
> header `impl<'a> OwnershipAnalyzer<'a>` verbatim so the moved method bodies
> (which reference `'a`) compile unchanged.

- [ ] **Step 3: Remove moved methods from `lib.rs`, add `mod analysis;`**

Delete the 8 moved methods from the `impl OwnershipAnalyzer` block in `lib.rs`.
Add `mod analysis;` to `lib.rs` (alphabetical among `mod` decls). No `pub use`
(the engine is not public API).

- [ ] **Step 4: Run tests**

Run: `cargo test -p kali_mir`
Expected: `36 passed`.

- [ ] **Step 5: Commit**

```bash
git add crates/kali_mir/src/analysis/ crates/kali_mir/src/lib.rs
git commit -m "refactor(kali_mir): create analysis subtree, extract scope pass [refactor]"
```

---

### Task 10: Extract `analysis/walk.rs`

**Files:**
- Create: `crates/kali_mir/src/analysis/walk.rs`
- Modify: `crates/kali_mir/src/analysis/mod.rs`, `crates/kali_mir/src/lib.rs`

**Interfaces:**
- Consumes: `crate::OwnershipAnalyzer` + the crate/HIR types the walk methods
  reference (e.g. `kali_hir::{HirNode, HirNodeId, HirNodeKind}`, `crate::UseContext`,
  `crate::MirBindingKind`, `crate::LayoutDescriptor` — set compiler-driven).

- [ ] **Step 1: Create `analysis/walk.rs`**

Header:
```rust
//! HIR scope-walking and use/binding resolution for the analyzer.
```
+ the precise `use` lines (compiler-driven). Then
`impl<'a> OwnershipAnalyzer<'a> { … }` (verbatim generic header) containing,
moved **byte-for-byte**: `walk_scope_node` (the ~250-line dispatcher),
`resolve_use`, `resolve_binding`, `is_heap_store_target`.

- [ ] **Step 2: Wire + remove from `lib.rs`**

Add `mod walk;` to `analysis/mod.rs` (alphabetical: `scope`, `walk`, then later
`infer`/`resolve` — keep alphabetical: `infer`, `resolve`, `scope`, `walk`).
Delete the 4 moved methods from `lib.rs`.

- [ ] **Step 3: Run tests**

Run: `cargo test -p kali_mir`
Expected: `36 passed`.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_mir/src/analysis/ crates/kali_mir/src/lib.rs
git commit -m "refactor(kali_mir): extract analysis walk pass [refactor]"
```

---

### Task 11: Extract `analysis/infer.rs`

**Files:**
- Create: `crates/kali_mir/src/analysis/infer.rs`
- Modify: `crates/kali_mir/src/analysis/mod.rs`, `crates/kali_mir/src/lib.rs`

**Interfaces:**
- Consumes: `crate::{OwnershipAnalyzer, LayoutDescriptor}` + types the infer
  methods reference (e.g. `kali_hir::{HirNode, HirNodeId, HirNodeKind}` — set
  compiler-driven). `LayoutDescriptor::scalar` is `pub(crate)` (Task 1).

- [ ] **Step 1: Create `analysis/infer.rs`**

Header:
```rust
//! Layout inference for analyzer bindings.
```
+ precise `use` lines (compiler-driven). Then `impl<'a> OwnershipAnalyzer<'a> { … }`
(verbatim generic header) containing, moved **byte-for-byte**: `infer_layout`,
`infer_binary_layout`, `infer_unary_layout`, `resolve_binding_layout`,
`layout_field_name`, `object_property_order_key`.

- [ ] **Step 2: Wire + remove from `lib.rs`**

Add `mod infer;` to `analysis/mod.rs` (keep alphabetical). Delete the 6 moved
methods from `lib.rs`.

- [ ] **Step 3: Run tests**

Run: `cargo test -p kali_mir`
Expected: `36 passed`.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_mir/src/analysis/ crates/kali_mir/src/lib.rs
git commit -m "refactor(kali_mir): extract analysis infer pass [refactor]"
```

---

### Task 12: Extract `analysis/resolve.rs`

**Files:**
- Create: `crates/kali_mir/src/analysis/resolve.rs`
- Modify: `crates/kali_mir/src/analysis/mod.rs`, `crates/kali_mir/src/lib.rs`

**Interfaces:**
- Consumes: `crate::OwnershipAnalyzer` + types the resolve methods reference
  (e.g. `kali_hir::HirNodeId`, `crate::MirFunction` — set compiler-driven).

- [ ] **Step 1: Create `analysis/resolve.rs`**

Header:
```rust
//! Function-target and escape-flag resolution for the analyzer.
```
+ precise `use` lines (compiler-driven). Then `impl<'a> OwnershipAnalyzer<'a> { … }`
(verbatim generic header) containing, moved **byte-for-byte**:
`function_parameter_escape_flags`, `resolve_function_target`,
`function_target_from_node`, `function_name_from_recent_functions`,
`next_function_name`.

- [ ] **Step 2: Wire + remove from `lib.rs`**

Add `mod resolve;` to `analysis/mod.rs` (final order: `infer`, `resolve`,
`scope`, `walk`). Delete the 5 moved methods from `lib.rs`. At this point the
`impl OwnershipAnalyzer` block remaining in `lib.rs` holds only the entry
methods `new`, `analyze_program`, `function_flavor`.

- [ ] **Step 3: Run tests**

Run: `cargo test -p kali_mir`
Expected: `36 passed`.

- [ ] **Step 4: Commit**

```bash
git add crates/kali_mir/src/analysis/ crates/kali_mir/src/lib.rs
git commit -m "refactor(kali_mir): extract analysis resolve pass [refactor]"
```

---

### Task 13: Move `OwnershipAnalyzer` core into `analysis/mod.rs`; reduce `lib.rs` to a facade

**Files:**
- Modify: `crates/kali_mir/src/analysis/mod.rs`, `crates/kali_mir/src/lib.rs`

**Interfaces:**
- Produces: `crate::analysis::OwnershipAnalyzer` (the struct's new home);
  `lower.rs`'s `use crate::OwnershipAnalyzer` still resolves because `lib.rs`
  re-exports it `pub(crate) use analysis::OwnershipAnalyzer;`.

- [ ] **Step 1: Move the struct, support types, free fns, and entry methods into `analysis/mod.rs`**

Above the `mod` decls in `analysis/mod.rs`, add the imports the moved code uses
(compiler-driven — expect `std::collections::{BTreeMap, BTreeSet}`,
`kali_hir::{FunctionFlavor, HirNode, HirNodeId}`, and `crate::{…}` for the MIR
types referenced). Move **byte-for-byte** from `lib.rs`:
- `pub(crate) struct OwnershipAnalyzer<'a>` + its `impl<'a> OwnershipAnalyzer<'a>`
  block now holding only `new`, `analyze_program`, `function_flavor`.
- `pub(crate) enum UseContext`.
- `pub(crate) struct BindingState` + its `impl`.
- `pub(crate) struct ScopeState` + its `impl`.
- Free fns `default_ownership`, `parameter_escape_flags`,
  `function_binding_escapes`, `finalise_binding`.

Keep the existing `mod infer; mod resolve; mod scope; mod walk;` decls.

- [ ] **Step 2: Reduce `lib.rs` to a pure facade**

`lib.rs` should now contain only: the crate-level `//!` docs, the `mod` decls
(alphabetical: `analysis`, `binding`, `function`, `layout`, `lower`, `node`,
`ownership`, `program`), the `pub use` re-exports of the 18 public types, the
`pub(crate) use analysis::OwnershipAnalyzer;` re-export (so `lower.rs`'s import
resolves unchanged), and the `cfg(test)` test wiring (still `mod tests;` until
Task 14). Example tail:

```rust
//! Mid-level IR (MIR) for the Kali compiler.
//!
//! MIR is a conservative structural lowering of HIR that preserves the source
//! shape while providing a stable bridge for later memory/ownership analysis.

mod analysis;
mod binding;
mod function;
mod layout;
mod lower;
mod node;
mod ownership;
mod program;

pub use binding::{BorrowedLifetime, MirBinding, MirBindingKind};
pub use function::{MirFunction, MirFunctionKind};
pub use layout::LayoutDescriptor;
pub use lower::MirLowerer;
pub use node::{MirBuilder, MirNode, MirNodeId, MirNodeKind, PlaceRef, PlaceValue};
pub use ownership::{
    OwnershipClass, ThreadBoundaryBinding, ThreadBoundaryDisposition, ThreadBoundaryProfile,
};
pub use program::MirProgram;

pub(crate) use analysis::OwnershipAnalyzer;

#[cfg(test)]
mod tests;
```
(Exact `pub use` grouping/wrapping will be normalized by `cargo fmt` in Task 15;
match rustfmt's output to avoid a churn diff there.)

- [ ] **Step 3: Run tests + clippy**

Run: `cargo test -p kali_mir && cargo clippy -p kali_mir --all-targets`
Expected: `36 passed`; clippy **clean** (all `pub(crate)` items are now consumed
across module boundaries). Verify `lib.rs` has no `fn`/`struct`/`enum`/`impl`/
`macro` items:
```bash
grep -nE '^\s*(pub(\(crate\))? )?(fn|struct|enum|impl|trait|macro_rules!) ' crates/kali_mir/src/lib.rs   # expect empty
```

- [ ] **Step 4: Commit**

```bash
git add crates/kali_mir/src/analysis/ crates/kali_mir/src/lib.rs
git commit -m "refactor(kali_mir): move analyzer core into analysis, reduce lib.rs to facade [refactor]"
```

---

### Task 14: Add shared `test_support` and relocate tests

The 36 tests in `tests.rs` move into sibling `*_tests.rs` files grouped by the
module they exercise, sharing a `test_support.rs` helper. **The basename-multiset
diff is the authoritative gate** — exact per-file placement is a target, not a
frozen contract; map each test by reading its body.

**Files:**
- Create: `crates/kali_mir/src/test_support.rs`
- Create: `crates/kali_mir/src/{ownership,layout,binding,function,node,program,lower}_tests.rs`
  and `crates/kali_mir/src/analysis/*_tests.rs` as needed (only those that receive tests)
- Delete: `crates/kali_mir/src/tests.rs`
- Modify: source modules (to wire their `*_tests.rs`), `crates/kali_mir/src/lib.rs`

- [ ] **Step 1: Create the shared test helper**

Inspect the top of `tests.rs` for the shared parse/lower helper(s) (the
`use` block + any `fn` that builds a `MirProgram`/`HirLoweringResult` from
source). Move them **byte-for-byte** into `test_support.rs`, making the helper
fn(s) `pub(crate)`. Header e.g.:
```rust
//! Shared helpers for kali_mir unit tests.
```

- [ ] **Step 2: Split `tests.rs` by cluster into sibling `*_tests.rs` files**

Read each `#[test]` body and move it (verbatim) into the `*_tests.rs` for the
module it exercises. Suggested grouping (confirm by reading bodies — adjust
freely as long as the basename diff stays empty):
- `ownership_tests.rs` — ownership-class & thread-boundary-profile tests
  (`test_ownership_classes_define_thread_boundary_rules`,
  `test_thread_boundary_profile_*`, `test_thread_boundary_profiles_*`,
  `test_representation_fingerprints_*`, `test_binding_thread_boundary_entry_*`).
- `layout_tests.rs` — `test_layout_fingerprints_*`, `test_object_layout_orders_*`.
- `program_tests.rs` — borrowed-lifetime/summary query tests
  (`test_borrowed_lifetime_reports_*`, `test_module_scope_summary_helpers_*`,
  `test_scope_filtered_mir_summaries_*`).
- `lower_tests.rs` — structural-lowering tests
  (`test_mir_lowering_preserves_*`, `test_call_expressions_lower_to_call_nodes`,
  `test_mir_validation_rejects_out_of_bounds_children`).
- `analysis/ownership_analysis_tests.rs` — escape/ownership-analysis tests
  (`test_aliased_function_expressions_*`, `test_array_element_values_escape_*`,
  `test_assignment_into_member_*`, `test_call_arguments_escape_*`,
  `test_captured_bindings_*`, `test_function_alias_chains_*`,
  `test_inline_*_function_calls_*`, `test_non_escaping_closure_*`,
  `test_object_literal_values_escape_*`, `test_returned_bindings_*`,
  `test_stack_local_bindings_*`).

Each `*_tests.rs` begins by importing what its tests use, e.g.
`use crate::*;` plus `use crate::test_support::*;` (match what the original
test bodies referenced; keep bodies verbatim).

- [ ] **Step 3: Wire each `*_tests.rs` into its source module**

At the bottom of each source module that received tests, add:
```rust
#[cfg(test)]
#[path = "ownership_tests.rs"]
mod ownership_tests;
```
(path/name per file; for `analysis/` siblings the `#[path]` is relative to
`analysis/`). In `lib.rs`, replace `#[cfg(test)] mod tests;` with
`#[cfg(test)] mod test_support;`. Delete `tests.rs`.

- [ ] **Step 4: Run tests + verify the basename proof**

Run: `cargo test -p kali_mir`
Expected: `36 passed`.
Then:
```bash
cargo test -p kali_mir -- --list 2>/dev/null | grep ': test$' \
  | sed -E 's/^.*:://; s/: test$//' | sort > /tmp/kali_mir_after.txt
diff /tmp/kali_mir_baseline.txt /tmp/kali_mir_after.txt   # expect EMPTY
```

- [ ] **Step 5: Commit**

```bash
git add crates/kali_mir/src/
git commit -m "test(kali_mir): relocate tests into sibling modules + shared test_support [refactor]"
```

---

### Task 15: Final verification + `cargo fmt`

**Files:**
- Modify: any `crates/kali_mir/src/*.rs` touched by `cargo fmt` (formatting only)

- [ ] **Step 1: Format**

Run: `cargo fmt -p kali_mir`
Then inspect the diff: `git diff` — it must be **formatting only** (no logic,
value, or item-order change). If `fmt` rewraps a `pub(crate)` signature or
re-groups imports, confirm each hunk is behavior-neutral.

- [ ] **Step 2: Full gate**

Run:
```bash
cargo test -p kali_mir                      # 36 passed
cargo clippy -p kali_mir --all-targets      # clean (no warnings)
cargo build                                 # workspace builds (downstream crates compile vs the facade)
```

- [ ] **Step 3: Re-verify the basename proof + facade**

```bash
cargo test -p kali_mir -- --list 2>/dev/null | grep ': test$' \
  | sed -E 's/^.*:://; s/: test$//' | sort | diff /tmp/kali_mir_baseline.txt -   # EMPTY
grep -nE '^\s*(pub(\(crate\))? )?(fn|struct|enum|impl|trait|macro_rules!) ' crates/kali_mir/src/lib.rs   # empty
wc -l crates/kali_mir/src/lib.rs            # ~30 lines
```

- [ ] **Step 4: Commit**

```bash
git add crates/kali_mir/src/
git commit -m "style(kali_mir): cargo fmt [refactor]"
```

---

## Self-Review notes

- **Spec coverage:** every module in the design's target layout has an
  extraction task (Tasks 2–13); widening (Task 1); test relocation +
  basename proof (Task 14); fmt/clippy/build gate (Task 15). Public-API
  preservation is enforced by the facade `pub use` block (Task 13 Step 2) and
  the workspace build (Task 15 Step 2).
- **Verbatim movement** is restated as a per-step instruction on every
  extraction task and pinned in Global Constraints.
- **`pub(crate)` widening set** (Task 1) is derived from the actual cross-module
  references: `MirBuilder.nodes`; `ThreadBoundaryProfile::{push_binding,finalize}`;
  `LayoutDescriptor::scalar`; the entire `OwnershipAnalyzer` engine + `UseContext`
  /`BindingState`/`ScopeState` + the 4 analysis free fns. Items with a co-located
  sole caller (`validate_tree`, `function_scope_name`, `map_kind`,
  `MirNode::{new,with_text}`) deliberately stay private.
- **Line numbers shift** as items are removed; tasks locate items by **name**
  (pre-flight line numbers are reference hints only). The controller should hand
  each implementer freshly-grepped current line ranges.
- **Import lists** per task are the expected set; implementers adjust them
  compiler-driven without altering the moved bodies' qualification style.
