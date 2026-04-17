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
