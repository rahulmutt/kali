
## 2026-04-17 — Stage 4.2 original zero-count follow-up

I widened the current RC snapshot proof slice with `KaliCore.Safety.releaseAndCollectDropsOriginalZeroCountCells`, which makes the local release-and-collect helper's original zero-count filtering behavior explicit.

Suggested follow-up:
- sync `proofs/BOUNDARY.md`, `README.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, and `PLAN-4.2-STATUS.md` so the published boundary inventory names the new original-zero-count helper theorem
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a helper-level no-leak slice, not the full ownership/freeing story

## 2026-04-17 — Stage 4.2 heap-characterisation sync

I added `KaliCore.Safety.releaseAndCollectHeapIsPositiveCountFilter`, which characterises the local collection helper's heap as exactly the positive-count filter of the decrement pass.

Suggested follow-up:
- keep the broader ownership/freeing widening incremental; this remains a helper-level local collection fact, not the full Stage 4.2 ownership/freeing target

## 2026-04-17 — Stage 4.2 zero-count-removal sync

I synced the Stage 4.2 progress summary so the current RC snapshot proof slice now names `releaseAndCollectDropsZeroCountCells` explicitly alongside the other local collection-helper theorems.

Suggested follow-up:
- keep the later ownership/freeing widening incremental; this is still a local helper-level slice, not the full Stage 4.2 ownership/freeing target

## 2026-04-17 — Stage 4.2 zero-count freeing follow-up

I widened the current RC snapshot proof slice with `KaliCore.Safety.releaseAndCollectDropsZeroCountCells`, which makes the local collection helper's zero-count removal behavior explicit in the theorem inventory.

Suggested follow-up:
- update `PLAN-4.2-STATUS.md`, `TODO.md`, and the proof-boundary / verification summary docs so the progress tracker names the new zero-count-removal lemma
- keep the story incremental: this is still a local freeing slice, not the full Stage 4.2 ownership/freeing target

## 2026-04-17 — Stage 4.2 zero-count collection follow-up

The RC snapshot proof slice now includes a local freeing step: `releaseAndCollect` filters zero-count cells after the decrement pass, and the proof boundary should now mention that collection helper alongside the existing release/decrement bookkeeping.

Suggested follow-up:
- update `PLAN-4.2-STATUS.md`, `TODO.md`, and the proof-boundary / verification summary docs so the progress tracker matches the new theorem inventory
- keep the story incremental: this is still a local zero-count collection slice, not the full Stage 4.2 ownership/freeing target

## 2026-04-17 — Stage 4.2 releaseAndCollect disjointness follow-up

I added the explicit `releaseAndCollectReleasedNotLiveRef` theorem to the RC snapshot slice, then synced `PLAN-4.2-STATUS.md` and `TODO.md` so the Stage 4.2 progress tracker reflects the new theorem inventory.

Suggested follow-up:
- keep the broader Stage 4.2 ownership/freeing work incremental; the local collection helper is still a slice, not the full ownership/freeing story

## 2026-04-17 — Stage 4.2 releaseAndCollect recording follow-up

I added `releaseAndCollectRecorded` to the RC snapshot proof slice, so the Stage 4.2 progress tracker now includes explicit release-recording for the local collection helper alongside the existing zero-count collection and disjointness results.

Suggested follow-up:
- keep widening the memory-safety slice incrementally; the current helper-level claim is still narrower than the full ownership/freeing target

## 2026-04-17 — Stage 4.2 RC freeing follow-up

I plan to widen the Stage 4.2 memory-safety slice with a helper-level lemma showing that `releaseAndCollect` preserves positive-count cells from the decrement pass, so the current local collection story has an explicit "only zero-count cells are dropped" theorem alongside the existing target-cell removal result.

Suggested follow-up:
- update `PLAN-4.2-STATUS.md` and `TODO.md` so the progress tracker names the new positive-count preservation lemma
- if the boundary wording changes, sync the proof-boundary / verification summary docs in the same pass

## 2026-04-17 — Stage 4.2 releaseAndCollect positive-count follow-up

The proof-backed RC snapshot slice now includes `KaliCore.Safety.releaseAndCollectKeepsOtherPositiveCountCells`, so the progress tracker should name the positive-count-preservation/no-leak helper explicitly alongside the existing zero-count collection and disjointness theorems.

Suggested follow-up:
- update the Stage 4.2 status / progress docs and TODO tracker to mention the new helper theorem
- keep the story incremental; this is still a local collection-helper slice, not the full ownership/freeing target

## 2026-04-17 — Stage 4.2 releaseAndCollect other-live preservation follow-up

I plan to widen the Stage 4.2 RC snapshot slice with a helper-level theorem that `releaseAndCollect` preserves other live references, so the progress tracker can explicitly name that local collection-helper invariant alongside the existing zero-count collection and disjointness results.

Suggested follow-up:
- update `PLAN-4.2-STATUS.md` and `TODO.md` so the tracker names the new local-helper preservation theorem
- if the boundary wording changes, sync the proof-boundary / verification summary docs in the same pass

## 2026-04-17 — Stage 4.2 live-reference ownership/allocation follow-up

I added helper corollaries that both the decrement path and the local collection helper preserve the ownership/allocation story for surviving live references (`releaseAndDecrementLiveRefsAreOwnedAndAllocated` and `releaseAndCollectLiveRefsAreOwnedAndAllocated`).

Suggested follow-up:
- update `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, and `TODO.md` so the progress tracker explicitly names the helper-level ownership/allocation preservation corollaries
- keep the broader Stage 4.2 ownership/freeing target incremental; these are helper corollaries on top of the current proof-backed slice

## 2026-04-17 — Stage 4.2 helper corollary sync follow-up

I synced the Stage 4.2 progress tracker and TODO notes so the current proof-backed slice now names the helper-level ownership/allocation corollaries (`releaseAndDecrementLiveRefsAreOwnedAndAllocated` and `releaseAndCollectLiveRefsAreOwnedAndAllocated`) alongside the existing zero-count collection and disjointness bookkeeping.

Suggested follow-up:
- keep the remaining Stage 4.2 memory-safety widening incremental; this is still a helper-level slice, not the full ownership/freeing target

## 2026-04-17 — Stage 4.2 positive-count final-heap follow-up

I widened the current RC snapshot proof slice with `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount`, so the proof-backed boundary now names the local collection helper's positive-count-only final heap property alongside the existing heap-characterisation theorem.

Suggested follow-up:
- keep the broader Stage 4.2 ownership/freeing story incremental; this remains a helper-level slice, not the full RC target

## 2026-04-17 — Stage 4.2 pure release-helper follow-up

I widened the current RC snapshot proof slice with pure release-helper corollaries (`releaseRefLiveRefsAreOwnedAndAllocated` and `releaseRefReleasedNotLiveRef`) so the proof-backed boundary now covers the release-only helper in addition to the decrement and local collection paths.

Suggested follow-up:
- sync `PLAN-4.2-STATUS.md`, `TODO.md`, and the proof-boundary / verification summary docs so the progress tracker names the new pure release-helper corollaries
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a helper-level slice, not the full ownership/freeing story

## 2026-04-17 — Stage 4.2 original zero-count follow-up

I synced the Stage 4.2 progress note in `plan/phase-4/02-formal-verification-depth.md` so it now explicitly names `KaliCore.Safety.releaseAndCollectDropsOriginalZeroCountCells` alongside the existing local collection-helper theorems.

Suggested follow-up:
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a helper-level no-leak slice, not the full ownership/freeing story

## 2026-04-17 — Stage 4.2 ownership-envelope preservation follow-up

I widened the Stage 4.2 RC snapshot proof slice with explicit ownership-envelope preservation theorems for the release-only, decrement, and collection helpers (`KaliCore.Safety.releaseRefPreservesOwnership`, `KaliCore.Safety.releaseAndDecrementPreservesOwnership`, and `KaliCore.Safety.releaseAndCollectPreservesOwnership`).

Suggested follow-up:
- update `PLAN-4.2-STATUS.md`, `TODO.md`, and the proof-boundary / verification summary docs so the progress tracker names the new ownership-envelope preservation lemmas
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a helper-level ownership-map slice, not the full ownership/freeing story
