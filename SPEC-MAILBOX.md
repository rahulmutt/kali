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
