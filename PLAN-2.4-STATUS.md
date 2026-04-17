# Stage 2.4 Status Update

**Date:** 2026-04-12  
**Status:** ✅ Lean model foundation complete as the now proof-backed verification foundation

## Summary

Stage 2.4 now has a checked-in Lean 4 workspace under `proofs/` with the core Kali type-calculus model, provisional semantics, ownership stubs, and HIR lowering stubs. The proof boundary manifest names the Lean model files, CI has a real proof-check job path instead of the earlier stub, and the stage is complete while the repository is now proof-backed for the published boundary rather than merely proof-ready.

## Evidence

- `proofs/lakefile.lean` and `proofs/lean-toolchain` added ✅
- `proofs/KaliCore/*` and `proofs/KaliIR/*` model files added ✅
- `proofs/BOUNDARY.md` updated to a non-empty proof-backed manifest ✅
- CI proof-check workflow wired to `lake build` on `proofs/**` changes ✅
- Progress/preservation theorem statements are mechanized; the proof tree now compiles cleanly for the published boundary rather than relying on documented-sorry placeholders

## Notable Deliverables

- `KaliCore/Types.lean` defines the provisional core type and expression grammar
- `KaliCore/Semantics.lean` defines value, substitution, and the small-step relation for the bounded core fragment
- `KaliCore/Soundness.lean` states progress and preservation for the closed typed fragment
- `KaliCore/Safety.lean` records the ownership classes and the no-dangling-reference proposition
- `KaliIR/HIRModel.lean` adds a small provisional HIR model and lowering projection, and the lowering bridge now covers assignment and try/catch

## Current Limits

- The proof boundary is now proof-backed for the published boundary, while still remaining narrower than the later formal-verification depth stage.
- The main soundness theorems are mechanized for the published boundary, and only the later widening work remains for the broader memory-safety and lowering-correctness target.
- The memory-safety and lowering-correctness stories remain narrower slices than the later formal-verification depth stage.

## Next Step

Begin the Stage 4.2 proof-backed verification work: replace the remaining `sorry` placeholders, widen the published boundary only with named theorem/property inventory, and keep the proof-ready/proof-backed wording aligned with the manifest.
