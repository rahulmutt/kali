# Stage 1.6 — HIR & LIR Lowering

**Phase:** 1 — Core Compiler & Toolchain MVP  
**Spec refs:** [`specs/05-ir.md`](../../specs/05-ir.md), [`specs/06-memory.md`](../../specs/06-memory.md), [`specs/01-architecture.md`](../../specs/01-architecture.md)  
**Depends on:** [1.5 — Type Checker](05-type-checker.md)  
**Status:** 🟡 In Progress - HIR/LIR skeleton exists (1 test total pass)

---

### Completed Features

- ✅ HIR crate skeleton with builder pattern
- ✅ LIR crate skeleton with basic structure
- ✅ Node types for High-level IR and Low-level IR

### Test Coverage

**Passing **(1)
- test_hir_builder

**Missing HIR Coverage**:
- Type-checked AST → HIR lowering
- HIR → LIR lowering
- LIR pretty-printer
- Module linking

---

## Evidence

- ✅ `cargo build` succeeds
- ✅ `cargo test --workspace` passes
- 🟡 1 test passes (`cargo test -p kali_hir --lib`)

---

**Workable Milestone**: HIR/LIR scaffolding exists. Full lowering pipeline (`TypedAST → HIR → LIR`) requires implementation of:
- AST to HIR lowering
- HIR to LIR lowering  
- LIR pretty-printer
- Module linker

Foundation sufficient for next phase: WASM code generation (Stage 1.7).
