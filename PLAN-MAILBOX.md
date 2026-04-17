## 2026-04-17 — Stage 4.2 zero-count collection follow-up

The RC snapshot proof slice now includes a local freeing step: `releaseAndCollect` filters zero-count cells after the decrement pass, and the proof boundary should now mention that collection helper alongside the existing release/decrement bookkeeping.

Suggested follow-up:
- update `PLAN-4.2-STATUS.md`, `TODO.md`, and the proof-boundary / verification summary docs so the progress tracker matches the new theorem inventory
- keep the story incremental: this is still a local zero-count collection slice, not the full Stage 4.2 ownership/freeing target

## 2026-04-17 — Stage 4.2 releaseAndCollect disjointness follow-up

I added the explicit `releaseAndCollectReleasedNotLiveRef` theorem to the RC snapshot slice, then synced `PLAN-4.2-STATUS.md` and `TODO.md` so the Stage 4.2 progress tracker reflects the new theorem inventory.

Suggested follow-up:
- keep the broader Stage 4.2 ownership/freeing work incremental; the local collection helper is still a slice, not the full ownership/freeing story
