# Stage 4.2 Status Update

**Date:** 2026-04-17  
**Status:** 🟡 Proof model compiled, but still narrowed to a closed fragment

## Summary

`proofs/KaliCore/Soundness.lean` now compiles again and the Lean proof tree builds successfully, but the current proof model was narrowed to literals, variables, and closed functions so the progress/preservation theorems stay honest and mechanically checked. This is a useful proof-structure milestone, but it is not yet the full Stage 4.2 proof-backed boundary described in `plan/phase-4/02-formal-verification-depth.md`.

## Evidence

- `mise run lean-proofs` succeeds ✅
- `KaliCore.Soundness` now compiles without `sorry` placeholders ✅
- The current proof model is intentionally narrower than the full application/control-flow fragment ⚠️

## Notable Deliverables

- `KaliCore/Soundness.lean` now has a clean compile path for the closed fragment
- `KaliCore/Semantics.lean` was adjusted so substitution no longer descends into function bodies in this provisional fragment
- The proof mailbox records the remaining widening work for the full Stage 4.2 story

## Current Limits

- The current Lean boundary does **not** yet mechanize application, sequencing, or conditional soundness in the full Stage 4.2 sense
- Context-shifting substitution remains future work if we decide to widen the proof model back to the richer fragment
- `proofs/BOUNDARY.md` still needs to be kept aligned with the narrower fragment until the proof scope is widened again

## Next Step

Either widen the proof model back toward the full closed fragment described in the Stage 4.2 plan, or explicitly revise the plan/boundary wording to make the narrowed fragment the honest published target for the current iteration.
