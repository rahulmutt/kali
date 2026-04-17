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
