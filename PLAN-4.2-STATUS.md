# Stage 4.2 Status Update

**Date:** 2026-04-17  
**Status:** 🟢 Proof-backed boundary published for the widened closed fragment; safety and lowering lemmas mechanized for the current published slice

## Summary

`proofs/KaliCore/Soundness.lean` now compiles again and the Lean proof tree builds successfully. The proof model now covers literals, variables, closed functions, application, sequencing, and conditionals, so the progress/preservation theorems stay honest and mechanically checked while still stopping short of the later Stage 4.2 ownership/memory-safety and lowering-correctness target described in `plan/phase-4/02-formal-verification-depth.md`. The ownership model now also has a mechanised `noDanglingReference` theorem, and the HIR lowering stub records the structural equations for `lower_core`, `lower_let1`, `lower_seq`, and `lower_if`.

## Evidence

- `mise run lean-proofs` succeeds ✅
- `KaliCore.Soundness` now compiles without `sorry` placeholders ✅
- `KaliCore.Safety.noDanglingReference` is mechanised for the current reference-free bounded syntax ✅
- `KaliIR.HIRModel` now records the structural lowering equations for the provisional HIR model ✅
- The current proof model now covers the core application/control-flow fragment and remains narrower than the later Stage 4.2 ownership/memory-safety and lowering-correctness target ⚠️

## Notable Deliverables

- `KaliCore/Soundness.lean` now has a clean compile path for the widened closed fragment
- `KaliCore/Safety.lean` now proves the current no-dangling-reference statement for the reference-free bounded syntax
- `KaliIR/HIRModel.lean` now records the structural lowering equations for the provisional HIR model
- `proofs/BOUNDARY.md` now publishes the proof-backed boundary for the widened closed fragment
- The proof mailbox records the remaining widening work for the full Stage 4.2 story

## Current Limits

- The current Lean boundary now mechanizes application, sequencing, and conditional soundness for the widened closed fragment
- The memory-safety theorem is still a reference-free model statement, not the full ownership / RC safety proof target from Stage 4.2
- The lowering work is still structural rather than full semantic preservation for the HIR → LIR model
- Context-shifting substitution and the remaining memory/lowering proofs remain future work for the later Stage 4.2 target
- `proofs/BOUNDARY.md` should continue tracking the currently mechanized theorem inventory as the boundary widens

## Next Step

Continue widening the proof model toward the remaining Stage 4.2 targets: ownership/memory safety and lowering correctness, while keeping the published boundary honest about what is mechanized today.
