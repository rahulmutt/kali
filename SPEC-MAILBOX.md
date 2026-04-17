## 2026-04-17 — Stage 4.2 zero-count collection follow-up

I widened the current RC snapshot proof slice with a local freeing step: `releaseAndCollect` now filters zero-count cells after the decrement pass, and the new theorem inventory should mention that zero-count collection explicitly alongside the existing release/decrement bookkeeping.

Suggested follow-up:
- sync `proofs/BOUNDARY.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, and `README.md` so the proof-backed boundary names the zero-count collection slice
- keep the broader Stage 4.2 ownership/freeing target narrower than this local collection helper

## 2026-04-17 — Stage 4.2 releaseAndCollect disjointness follow-up

I added the explicit `releaseAndCollectReleasedNotLiveRef` theorem to the RC snapshot slice, then synced the proof-boundary / verification summaries and the Stage 4.2 progress tracker so the local collection helper is now named in the published boundary and supporting docs.

Suggested follow-up:
- keep widening the Stage 4.2 RC story incrementally, especially any additional release/collection helper invariants that can be mechanized without overclaiming the full ownership/freeing target
