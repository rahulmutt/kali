# Stage 4.2 Status Update

**Date:** 2026-04-17  
**Status:** 🟡 Proof model compiled for the widened closed fragment

## Summary

`proofs/KaliCore/Soundness.lean` now compiles again and the Lean proof tree builds successfully. The proof model now covers literals, variables, closed functions, application, sequencing, and conditionals, so the progress/preservation theorems stay honest and mechanically checked while still stopping short of the full Stage 4.2 proof-backed boundary described in `plan/phase-4/02-formal-verification-depth.md`.

## Evidence

- `mise run lean-proofs` succeeds ✅
- `KaliCore.Soundness` now compiles without `sorry` placeholders ✅
- The current proof model now covers the core application/control-flow fragment and remains narrower than the full Stage 4.2 proof target ⚠️

## Notable Deliverables

- `KaliCore/Soundness.lean` now has a clean compile path for the widened closed fragment
- `proofs/BOUNDARY.md` remains aligned with the currently mechanized theorem inventory
- The proof mailbox records the remaining widening work for the full Stage 4.2 story

## Current Limits

- The current Lean boundary now mechanizes application, sequencing, and conditional soundness for the widened closed fragment
- Context-shifting substitution and the remaining memory/lowering proofs remain future work for the Stage 4.2 target
- `proofs/BOUNDARY.md` should continue tracking the currently mechanized theorem inventory as the boundary widens

## Next Step

Continue widening the proof model toward the remaining Stage 4.2 targets: ownership/memory safety and lowering correctness, while keeping the published boundary honest about what is mechanized today.
