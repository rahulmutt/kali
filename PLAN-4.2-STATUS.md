# Stage 4.2 Status Update

**Date:** 2026-04-17  
**Status:** 🟢 Proof-backed boundary published for the widened closed fragment plus the RC snapshot safety slice and widened HIR lowering-correctness slice; the canonical repository summary is "Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target."

## Summary

`proofs/KaliCore/Soundness.lean` now compiles again and the Lean proof tree builds successfully. The proof model now covers literals, variables, closed functions, application, sequencing, conditionals, assignment, and try/catch, so the progress/preservation theorems stay honest and mechanically checked while still stopping short of bare throw plus the later Stage 4.2 ownership/memory-safety and lowering-correctness target described in `plan/phase-4/02-formal-verification-depth.md`. The ownership model now also has a mechanised RC snapshot safety story (`RcSnapshot`, `noDanglingReference`, and `releasedNotLive`), and the HIR lowering model now includes a small-step preservation bridge in `proofs/KaliIR/LoweringCorrectness.lean` on top of the structural equations for `lower_core`, `lower_let1`, `lower_seq`, `lower_if`, `lower_assign`, and `lower_tr`.

## Evidence

- `mise run lean-proofs` succeeds ✅
- `KaliCore.Soundness` now compiles without `sorry` placeholders ✅
- `KaliCore.Safety.noDanglingReference` is mechanised for the current RC snapshot model, and `releasedNotLive` covers the release-path split ✅
- `KaliIR.HIRModel` now records the structural lowering equations for the provisional HIR model ✅
- The current proof model now covers the core application/control-flow fragment plus assignment and try/catch, and a small RC snapshot safety slice, and remains narrower than the later Stage 4.2 ownership/memory-safety and lowering-correctness target ⚠️

## Notable Deliverables

- `KaliCore/Soundness.lean` now has a clean compile path for the widened closed fragment, including assignment and try/catch
- `KaliCore/Safety.lean` now proves the current no-dangling-reference statement for the RC snapshot model and its release-path liveness split
- `KaliIR/HIRModel.lean` now records the structural lowering equations for the provisional HIR model
- `KaliIR.LoweringCorrectness` now proves lowering preserves the modeled HIR step relation for the current subset, including assignment and try/catch
- `proofs/BOUNDARY.md` now publishes the proof-backed boundary for the widened closed fragment plus the RC snapshot safety slice and widened HIR lowering-correctness slice, and still matches the canonical repository summary verbatim
- The proof mailbox records the remaining widening work for the full Stage 4.2 story, especially ownership/memory safety and lowering correctness
- The published memory-safety slice now includes live-reference and release tracking, but it still stops short of the full ownership / RC target

## Current Limits

- The current Lean boundary now mechanizes application, sequencing, conditional, assignment, and try/catch soundness for the widened closed fragment
- The memory-safety theorem is still a bounded RC snapshot model, not the full ownership / RC safety proof target from Stage 4.2
- The lowering work is still intentionally narrower than full semantic preservation for the HIR → LIR model
- Context-shifting substitution and the remaining memory/lowering proofs remain future work for the later Stage 4.2 target
- `proofs/BOUNDARY.md` should continue tracking the currently mechanized theorem inventory as the boundary widens

## Next Step

Continue widening the proof model toward the remaining Stage 4.2 targets: fuller ownership/memory safety and the broader HIR → LIR semantic-preservation story, while keeping the published boundary honest about what is mechanized today.
