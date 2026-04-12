# Stage 2.4 Status Update

**Date:** 2026-04-12  
**Status:** 🟡 Lean model foundation landed as a provisional proof-ready step

## Summary

Stage 2.4 now has a checked-in Lean 4 workspace under `proofs/` with the core Kali type-calculus model, provisional semantics, ownership stubs, and HIR lowering stubs. The proof boundary manifest has been updated to name the Lean model files, and CI now has a real proof-check job path instead of the earlier stub.

## Evidence

- `proofs/lakefile.lean` and `proofs/lean-toolchain` added ✅
- `proofs/KaliCore/*` and `proofs/KaliIR/*` model files added ✅
- `proofs/BOUNDARY.md` updated to a provisional non-empty manifest ✅
- CI proof-check workflow wired to `lake build` on `proofs/**` changes ✅
- Progress/preservation theorem statements are present; the main proofs remain documented-sorry placeholders pending later mechanization

## Notable Deliverables

- `KaliCore/Types.lean` defines the provisional core type and expression grammar
- `KaliCore/Semantics.lean` defines value, substitution, and the small-step relation for the bounded core fragment
- `KaliCore/Soundness.lean` states progress and preservation for the closed typed fragment
- `KaliCore/Safety.lean` records the ownership classes and the no-dangling-reference proposition
- `KaliIR/HIRModel.lean` adds a small provisional HIR model and lowering projection

## Current Limits

- The proof boundary is still provisional and proof-ready, not proof-backed.
- The main soundness theorems are documented with `sorry` placeholders and will need to be closed or replaced before any proof-backed release claim.
- The memory-safety and lowering-correctness stories remain stubs for the later formal-verification depth stage.

## Next Step

Continue tightening the Lean proof obligations and keep the boundary wording aligned with the provisional model until the later proof-backed stage.
