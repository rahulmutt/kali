# Stage 2.1 — MIR & Ownership Analysis

**Phase:** 2 — Ownership, Effects & Public Embedding  
**Spec refs:** [`specs/05-ir.md`](../../specs/05-ir.md), [`specs/06-memory.md`](../../specs/06-memory.md), [`specs/01-architecture.md`](../../specs/01-architecture.md)  
**Depends on:** Phase 1 complete (all stages 1.1–1.14)

## Goal

Implement `kali_mir` — the Mid-level IR — and the ownership/escape analysis passes that make MIR
the canonical lowering stage between HIR and LIR. Replace the Phase-1 conservative reference-
counting placeholder with deterministic compile-time ownership decisions: `stack`, `owned heap`,
`shared heap` (reference-counted), and `borrowed` lifetimes.

## Workable Milestone

- The full pipeline is `TypedAST → HIR → MIR → LIR → WASM`; HIR → LIR direct lowering is retired.
- Escape analysis correctly classifies most local values as stack-allocated; heap allocation is
  reserved for values that genuinely escape.
- Generated WASM is measurably leaner on representative benchmarks due to fewer heap allocations
  and fewer RC operations.
- All Phase-1 tests continue to pass.

## Implementation note

- 2026-04-12: `kali_mir` now exposes the canonical ownership classes (`Stack`, `OwnedHeap`,
  `SharedHeap`, `Borrowed`) plus deterministic layout descriptors and a scope-level ownership
  analyzer. Call-expression lowering now emits MIR `Call` nodes as part of the Phase-2 bridge.
  The codebase now also includes a MIR-backed LIR diff/benchmark harness that simulates the Phase-1
  conservative direct-lowering overhead on a stable legacy baseline so the stage can measure the
  smaller MIR-backed output against the legacy shape-preserving path. Layout-aware codegen now
  also recognizes transparent single-child wrappers and boolean-shaped branch conditions so it can
  skip the redundant truthiness cast when the shape is already known to be canonical 0/1.
  MIR closure bindings now retain the concrete capture list in their closure layout descriptors so
  the ownership model records not just who captured a value but also which values each closure
  environment needs to carry. The ownership analyzer now also distinguishes non-escaping closure
  captures from escaping ones, so borrowed lifetimes remain borrowed when a nested closure stays
  local instead of being forced into shared-heap ownership.

## Tasks

### 1. MIR node definitions (`kali_mir`)

MIR sits between HIR (desugared, still close to source) and LIR (WASM-ready, tagged values).
MIR's distinctive responsibility is **explicit memory layout and ownership annotation**:

- Every MIR value carries an `OwnershipClass`:
  - `Stack` — allocated in the shadow stack frame; no heap involvement.
  - `OwnedHeap` — one owner; freed when the owner goes out of scope.
  - `SharedHeap` — reference-counted; `rc_incref` / `rc_decref` emitted explicitly.
  - `Borrowed(lifetime)` — a non-owning reference; lifetime must not outlive its referent.
- Every MIR place (memory location) has an explicit layout descriptor derived from the type:
  - Scalar: `i32`, `i64`, `f64`.
  - Struct: fixed-offset field map.
  - Array: length-prefixed contiguous elements.
  - Closure: environment pointer + function pointer.
  - `TaggedVal`: uniform tagged representation (still used for polymorphic / unknown-type values).
- MIR functions use SSA-like form with explicit `PlaceRef` (reference to a place) and
  `PlaceValue` (loaded value from a place).

### 2. HIR → MIR lowering

Walk each `HirFunction` and produce a `MirFunction`:

- For each HIR local binding, run escape analysis (see below) to determine its `OwnershipClass`.
- Emit explicit `MirAlloc(layout)` for `OwnedHeap` and `SharedHeap` values.
- Emit `MirFree(ptr)` or `MirRcDecref(ptr)` at the end of the owning scope.
- Lower HIR calls that return owned values: insert RC operations at ownership transfer points.
- Lower HIR closures: capture analysis determines which captured variables go into the closure
  environment (heap-allocated) vs are passed by value.

### 3. Escape analysis

Implement a flow-sensitive, intra-procedural escape analysis:

- A value **escapes** if it is: returned from the function, stored into a heap object, captured
  by a closure, or passed to a function that stores it (callee-escapes analysis with a
  conservative approximation for indirect calls).
- Values that do not escape are classified `Stack`; those that escape to a single owner are
  `OwnedHeap`; those that are shared across multiple owners are `SharedHeap`.
- The analysis runs within the MIR phase before LIR lowering so that layout decisions are
  available to the LIR emitter.

### 4. Layout-aware LIR lowering

Update the `MIR → LIR` lowering (replacing the old `HIR → LIR` path):

- `Stack` values: emit shadow-stack frame slots; no `Alloc` call.
- `OwnedHeap` values: emit `Alloc` at construction, `Free` at scope end.
- `SharedHeap` values: emit `Alloc` + `RcIncref` at construction, `RcDecref` at scope end.
- `Borrowed` values: emit the pointer directly; no RC operations.
- Struct fields with known-scalar types are accessed via fixed-offset `Load` / `Store` without
  going through `TaggedVal` tagging/untagging. This eliminates the tag-check overhead for
  statically typed code.

### 5. Tests

- **Ownership classification tests**: given annotated HIR fixtures, assert each local binding
  receives the expected `OwnershipClass`.
- **LIR diff tests**: compare the LIR produced by the new MIR-backed pipeline against the
  Phase-1 direct-lowering output; assert that stack-allocated values no longer emit `Alloc` /
  `RcIncref` / `RcDecref`.
- **All Phase-1 integration tests must continue to pass.**
- **Benchmark**: on a representative compute-intensive fixture, measure WASM module size and
  instruction count; assert improvement over the Phase-1 baseline.

## Out of Scope

- Full inter-procedural escape analysis (Phase 3 depth).
- LLVM-style alias analysis (Phase 3 depth).
- Effect inference changes (Stage 2.2).
- Public embedding surface changes (Stage 2.3).

## Status

This stage is complete.

Treat this file as the historical implementation playbook for the milestone it delivered. For
current availability, constraints, and any later widening work, use the owning spec references at
the top of this file together with [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md).
