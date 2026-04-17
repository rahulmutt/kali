## 2026-04-17 — Stage 4.2 heap-positive-heap follow-up

I found one remaining summary-doc drift point after the proof boundary widened: the published boundary already names `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount`, but the high-level verification summaries still need to mention that final-heap positive-count theorem explicitly alongside the other RC snapshot slice claims.

Suggested follow-up:
- sync `README.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` so the published verification summaries name `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount` alongside the other `releaseAndCollect` helper theorems
- keep the Stage 4.2 claim narrow; this is still a helper-level no-negative-count theorem, not the full ownership/freeing story

## 2026-04-17 — Stage 4.2 target-cell retention follow-up

I widened the current RC snapshot proof slice with `KaliCore.Safety.releaseAndCollectKeepsTargetCellWhenPositiveCount`, which states that the released target cell survives the local `releaseAndCollect` helper when its decremented count is still positive.

Suggested follow-up:
- sync `README.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` so the proof-backed boundary inventory names the new target-cell retention theorem
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a helper-level retention theorem, not the full ownership/freeing story


## 2026-04-17 — Stage 4.2 original zero-count follow-up

I widened the current RC snapshot proof slice with `KaliCore.Safety.releaseAndCollectDropsOriginalZeroCountCells`, which makes the local release-and-collect helper's original zero-count filtering behavior explicit.

Suggested follow-up:
- sync `proofs/BOUNDARY.md`, `README.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, and `PLAN-4.2-STATUS.md` so the published boundary inventory names the new original-zero-count helper theorem
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a helper-level no-leak slice, not the full ownership/freeing story

## 2026-04-17 — Stage 4.2 heap-characterisation sync

I widened the current RC snapshot proof slice with `KaliCore.Safety.releaseAndCollectHeapIsPositiveCountFilter`, which makes the local collection helper's heap/positive-count characterisation explicit in the theorem inventory.

Suggested follow-up:
- sync `proofs/BOUNDARY.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, and `README.md` so the proof-backed boundary names the new heap-characterisation lemma alongside the existing collection helper bookkeeping
- keep the broader Stage 4.2 ownership/freeing target narrower than this helper-level slice

## 2026-04-17 — Stage 4.2 zero-count-removal sync

I synced the proof-backed summary prose in `README.md` and `specs/19-feature-maturity.md` so the published boundary inventory now names `releaseAndCollectDropsZeroCountCells` explicitly.

Suggested follow-up:
- keep the broader Stage 4.2 ownership/freeing target narrower than this local helper-level slice

## 2026-04-17 — Stage 4.2 zero-count freeing follow-up

I widened the current RC snapshot proof slice with `KaliCore.Safety.releaseAndCollectDropsZeroCountCells`, which explicitly states that zero-count cells from the decrement pass are removed by the local collection helper.

Suggested follow-up:
- sync `proofs/BOUNDARY.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, and `README.md` so the proof-backed boundary names the new zero-count-removal lemma alongside the existing collection helper bookkeeping
- keep the broader Stage 4.2 ownership/freeing target narrower than this local helper-level slice

## 2026-04-17 — Stage 4.2 zero-count collection follow-up

I widened the current RC snapshot proof slice with a local freeing step: `releaseAndCollect` now filters zero-count cells after the decrement pass, and the new theorem inventory should mention that zero-count collection explicitly alongside the existing release/decrement bookkeeping.

Suggested follow-up:
- sync `proofs/BOUNDARY.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, and `README.md` so the proof-backed boundary names the zero-count collection slice
- keep the broader Stage 4.2 ownership/freeing target narrower than this local collection helper

## 2026-04-17 — Stage 4.2 releaseAndCollect disjointness follow-up

I added the explicit `releaseAndCollectReleasedNotLiveRef` theorem to the RC snapshot slice, then synced the proof-boundary / verification summaries and the Stage 4.2 progress tracker so the local collection helper is now named in the published boundary and supporting docs.

Suggested follow-up:
- keep widening the Stage 4.2 RC story incrementally, especially any additional release/collection helper invariants that can be mechanized without overclaiming the full ownership/freeing target

## 2026-04-17 — Stage 4.2 releaseAndCollect recording follow-up

I widened the current RC snapshot proof slice with `KaliCore.Safety.releaseAndCollectRecorded`, so the local collection helper now records the released reference in addition to filtering zero-count cells and preserving the remaining live set.

Suggested follow-up:
- keep the Stage 4.2 memory-safety story incremental; the local collection helper is still a slice, not the full ownership/freeing target
- if the boundary widens again, sync `proofs/BOUNDARY.md`, `README.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` together so the claim inventory stays aligned

## 2026-04-17 — Stage 4.2 RC freeing follow-up

I plan to widen the current proof-backed memory-safety slice with a slightly more general RC freeing lemma: `releaseAndCollect` will explicitly preserve positive-count cells from the decrement pass, complementing the existing target-cell zero-count removal theorem.

Suggested follow-up:
- sync `proofs/BOUNDARY.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, and `README.md` so the published boundary mentions the positive-count preservation / local no-leak slice
- keep the claim narrow: this is still the local collection helper story, not the full Stage 4.2 ownership/freeing target

## 2026-04-17 — Stage 4.2 releaseAndCollect positive-count follow-up

I widened the current RC snapshot proof slice with `KaliCore.Safety.releaseAndCollectKeepsOtherPositiveCountCells`, which states that positive-count cells from the original heap remain in the collected heap when they are not the released target.

Suggested follow-up:
- sync `proofs/BOUNDARY.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, and `README.md` so the published boundary inventory names the new no-leak helper theorem
- keep the broader Stage 4.2 ownership/freeing target narrower than this helper-level slice

## 2026-04-17 — Stage 4.2 releaseAndCollect other-live preservation follow-up

I widened the current RC snapshot proof slice with a helper-level lemma that `releaseAndCollect` preserves any other live reference's ownership/allocation story, so the local collection helper now explicitly covers the remaining live set as well as the release/decrement bookkeeping.

Suggested follow-up:
- sync the proof boundary / verification summaries and the Stage 4.2 progress tracker so the new local-helper theorem is named explicitly
- keep the claim narrow: this is still the local collection-helper slice, not the full ownership/freeing target

## 2026-04-17 — Stage 4.2 live-reference ownership/allocation follow-up

I added helper corollaries that both the decrement path and the local collection helper preserve the ownership/allocation story for surviving live references (`releaseAndDecrementLiveRefsAreOwnedAndAllocated` and `releaseAndCollectLiveRefsAreOwnedAndAllocated`).

Suggested follow-up:
- update `proofs/BOUNDARY.md`, `README.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, and the Stage 4.2 status tracker so the current published boundary inventory names the helper-level ownership/allocation preservation corollaries
- keep the broader Stage 4.2 ownership/freeing target incremental; these are helper corollaries on top of the current proof-backed slice

## 2026-04-17 — Stage 4.2 positive-count final-heap follow-up

I added `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount`, which makes the local collection helper's final positive-count-only heap property explicit on top of the existing heap-characterisation theorem.

Suggested follow-up:
- keep the Stage 4.2 RC widening incremental; this is still a helper-level local collection fact, not the broader ownership/freeing target

## 2026-04-17 — Verification summary sync for pure release-helper corollaries

I widened the current proof-backed RC snapshot slice with pure release-helper corollaries (`releaseRefLiveRefsAreOwnedAndAllocated` and `releaseRefReleasedNotLiveRef`) on top of the existing release/decrement/collection helper invariants.

Suggested follow-up:
- sync `proofs/BOUNDARY.md`, `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` so the proof-backed boundary inventory names the new release-helper theorems
- keep the broader Stage 4.2 ownership/freeing target narrower than this helper-level slice

## 2026-04-17 — Stage 4.2 ownership-envelope preservation follow-up

I widened the RC snapshot proof slice with explicit ownership-envelope preservation theorems for the release-only, decrement, and collection helpers (`KaliCore.Safety.releaseRefPreservesOwnership`, `KaliCore.Safety.releaseAndDecrementPreservesOwnership`, and `KaliCore.Safety.releaseAndCollectPreservesOwnership`).

Suggested follow-up:
- sync `proofs/BOUNDARY.md`, `README.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` so the proof-backed boundary inventory names the new ownership-envelope preservation lemmas
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a helper-level ownership-map slice, not the full ownership/freeing story

## 2026-04-17 — Stage 4.2 heap-origin provenance sync

I synced the proof-backed verification summaries after widening the RC snapshot proof slice with `KaliCore.Safety.releaseAndCollectHeapCellOrigin`, which makes the local `releaseAndCollect` helper's surviving-cell provenance explicit.

Suggested follow-up:
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a helper-level provenance theorem, not the full RC target

## 2026-04-17 — Stage 4.2 release-and-decrement heap-origin verification sync

I added `KaliCore.Safety.releaseAndDecrementHeapCellOrigin` to the proof-backed RC snapshot slice, making the decrement helper's surviving heap provenance explicit alongside the existing decrement/collect helper invariants.

Suggested follow-up:
- update `proofs/BOUNDARY.md`, `README.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` so the published proof boundary inventory names the new release-and-decrement heap-origin theorem
- keep the broader Stage 4.2 ownership/freeing target narrower than this helper-level provenance slice

## 2026-04-17 — Stage 4.2 release-set monotonicity follow-up

I widened the current RC snapshot proof slice with release-set monotonicity corollaries (`releaseRefPreservesReleasedRefs`, `releaseAndDecrementPreservesReleasedRefs`, and `releaseAndCollectPreservesReleasedRefs`) so the published boundary can explicitly name the already-released-set preservation story alongside the existing release bookkeeping.

Suggested follow-up:
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a helper-level monotonicity slice, not the full RC target

## 2026-04-17 — Stage 4.2 live-reference filtering follow-up

I widened the proof-backed RC snapshot slice with exact live-reference filtering theorems for the release-only, decrement, and collection helpers (`KaliCore.Safety.releaseRefLiveRefsFiltered`, `KaliCore.Safety.releaseAndDecrementLiveRefsFiltered`, and `KaliCore.Safety.releaseAndCollectLiveRefsFiltered`).

Suggested follow-up:
- keep the published proof boundary / verification summary docs aligned with the theorem inventory if the live-reference model widens again
- keep the broader Stage 4.2 ownership/freeing target incremental; these are helper-level shape theorems, not the full RC story

## 2026-04-17 — Stage 4.2 target-cell retention sync

I synced the proof-backed boundary and verification summaries after adding `KaliCore.Safety.releaseAndCollectKeepsTargetCellWhenPositiveCount`, so the published inventory now names the local collection helper's target-cell retention theorem alongside the rest of the RC snapshot slice.

Completed follow-up:
- updated `proofs/BOUNDARY.md`, `README.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, and `TODO.md` so the theorem inventory and status trackers stay aligned
- kept the claim narrow: this is still a helper-level retention theorem, not the full Stage 4.2 ownership/freeing story

## 2026-04-17 — Stage 4.2 target-cell retention wording sync

I synced the proof-boundary verification prose so `KaliCore.Safety.releaseAndCollectKeepsTargetCellWhenPositiveCount` is named explicitly in the repository summaries and phase-maturity wording.

Completed follow-up:
- updated `README.md`, `proofs/BOUNDARY.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` so the published verification summary explicitly names the target-cell retention theorem
- kept the Stage 4.2 claim narrow; this is still the local release-and-collect helper story, not the full ownership/freeing target

## 2026-04-17 — Stage 4.2 heap-positive testing-summary sync

I noticed `specs/16-testing.md` still lagged the latest proof-boundary inventory wording in its verification-posture summary, even though the main verification/spec docs already name `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount` and `KaliCore.Safety.releaseAndCollectKeepsTargetCellWhenPositiveCount` explicitly.

Suggested follow-up:
- sync `specs/16-testing.md` so the current repository-state note and proof-backed-claims guidance name the latest RC snapshot theorem inventory explicitly
- keep the claim narrow; this is still a summary-doc wording pass, not a boundary widening
