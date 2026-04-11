# Stage 1.6 — HIR & LIR Lowering

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/05-ir.md`](../../specs/05-ir.md), [`specs/06-memory.md`](../../specs/06-memory.md), [`specs/01-architecture.md`](../../specs/01-architecture.md)  
**Depends on:** [1.5 — Type Checker](05-type-checker.md)  
**Status:** ✅ Complete - HIR/LIR lowering pipeline implemented

---

### Completed Features

- ✅ Deterministic AST/statement → HIR lowering for declarations, control flow, and representative expressions
- ✅ HIR → MIR lowering that preserves program shape and node ordering
- ✅ MIR → LIR lowering that preserves root shape for codegen handoff
- ✅ Node types for High-level IR, Mid-level IR, and Low-level IR

### Test Coverage

**Passing **(4)
- test_hir_builder
- test_lower_statements_to_hir
- test_mir_lowering_preserves_program_shape
- test_lir_lowering_preserves_root

**Missing HIR Coverage**:
- Type-checked AST → HIR lowering
- HIR → LIR lowering
- LIR pretty-printer
- Module linking

---

## Evidence

- ✅ `cargo build` succeeds
- ✅ `cargo test --workspace` passes
- ✅ 4 targeted lowering tests pass across the HIR/MIR/LIR crates

---

**Workable Milestone**: HIR/LIR lowering pipeline now exists for representative Phase-1 source shapes. The compiler can deterministically lower parsed statements to HIR, then MIR, then LIR, preserving the source-program tree shape needed for the later WASM codegen stage.
