## 2026-04-19 — Stage 4.2 decrement target origin/positive-count widening

I found a small proof-summary widening worth tracking in the current published RC snapshot boundary: add a decrement-path target theorem that states the released target cell remains traceable to the original heap with a positive count when it survives `releaseAndDecrement`.

Planned update:
- add `KaliCore.Safety.releaseAndDecrementTargetCellOriginAndPositiveCount` to the proof-backed RC snapshot inventory, then sync `proofs/BOUNDARY.md` and the summary docs that enumerate the published theorem set
- keep the claim narrow: this is a helper-level RC widening, not the full Stage 4.2 ownership/freeing target
## 2026-04-19 — Stage 4.2 release-only linear-memory companion widening

Completed: the published proof boundary and summary docs now name `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` explicitly alongside the other helper companions, so the release-only provenance slice is already aligned and no SPEC.md changes were required for this follow-up.

## 2026-04-19 — Stage 4.2 ownership provenance wording sync

I found a remaining proof-summary drift point in the RC snapshot ownership/provenance slice: the summary docs should name `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCount` and its linear-memory companion explicitly and consistently instead of using the old concatenated wording in a few places.

Planned update:
- sync `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, `proofs/BOUNDARY.md`, `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, and `TODO.md` so the theorem names are spelled out cleanly
- keep the claim narrow: this is a wording / anti-drift sync for the published boundary, not a boundary widening

## 2026-04-19 — Stage 4.2 wellformedness / linear-memory corollary widening

I widened the RC snapshot proof slice with combined wellformedness/linear-memory corollaries for the release-only, decrement, and collection helpers: `KaliCore.Safety.releaseRefPreservesWellFormedAndLinearMemory`, `KaliCore.Safety.releaseAndDecrementPreservesWellFormedAndLinearMemory`, and `KaliCore.Safety.releaseAndCollectPreservesWellFormedAndLinearMemory`.

Planned update:
- sync the proof-boundary manifest and the verification summaries so the new combined corollaries are named explicitly alongside the current RC helper inventory
- keep the claim narrow: this is still a helper-level RC proof sync, not the broader Stage 4.2 ownership/freeing target

## 2026-04-19 — Stage 4.2 decrement target positive-count iff bridge

I widened the RC snapshot proof slice with a small helper theorem that states the decrement target's positive-count status after `releaseAndDecrement` is equivalent to the original count being greater than one: `KaliCore.Safety.releaseAndDecrementTargetCellPositiveCountIff`.

Planned update:
- sync the proof-boundary manifest and the verification summaries so the new decrement-path iff bridge is named explicitly alongside the current RC helper inventory
- keep the claim narrow: this is still a helper-level RC proof sync, not the broader Stage 4.2 ownership/freeing target

## 2026-04-18 — Stage 4.2 pure release-origin helper widening

I found a small gap in the published RC snapshot wording: the pure release helper had a heap-characterisation theorem and an origin/ownership theorem, but the proof boundary did not name the plain origin theorem `KaliCore.Safety.releaseRefHeapCellOrigin` explicitly.

Planned update:
- add `KaliCore.Safety.releaseRefHeapCellOrigin` to the RC snapshot proof slice, then sync `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, and `proofs/BOUNDARY.md` so the pure release-helper provenance story is explicit at the same granularity as the other helper families
- keep the claim narrow: this is a proof-summary / helper-theorem sync for the published boundary, not a boundary widening beyond the current RC snapshot model

## 2026-04-18 — Stage 4.2 target allocation follow-up

I plan to widen the current RC snapshot proof slice with explicit target-allocation corollaries for the refcount update helpers (`KaliCore.Safety.releaseAndDecrementTargetCellAllocatedWhenPositiveCount` and `KaliCore.Safety.releaseAndCollectTargetCellAllocatedWhenPositiveCount`), so the published boundary can name the allocation bridge alongside the existing positive-count and provenance lemmas.

Suggested follow-up:
- update the proof-boundary manifest and verification summaries so the new allocation corollaries are named explicitly alongside the current RC helper inventory
- keep the claim narrow: this is an allocation-bridge widening on top of the existing RC slice, not the full Stage 4.2 ownership/freeing target

## 2026-04-18 — Stage 4.2 heap-characterisation wording sync

I found that the verification summary docs still describe the RC snapshot heap-characterisation story a bit too generically, even though the published boundary already names `KaliCore.Safety.releaseAndDecrementHeapCharacterisation` and `KaliCore.Safety.releaseAndCollectHeapCharacterisation` explicitly.

Suggested follow-up:
- update `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` so the heap-characterisation theorems are named explicitly alongside the current RC snapshot inventory
- keep the claim narrow: this is a wording sync for the published boundary, not a boundary widening

## 2026-04-17 — Stage 4.2 helper-level no-dangling corollaries

I widened the proof-backed RC snapshot slice with helper-level no-dangling-reference corollaries for the release-only, decrement, and collection helpers (`KaliCore.Safety.releaseRefNoDanglingReference`, `KaliCore.Safety.releaseAndDecrementNoDanglingReference`, and `KaliCore.Safety.releaseAndCollectNoDanglingReference`).

Suggested follow-up:
- sync the proof-boundary manifest and the verification summaries so the no-dangling helper corollaries are named explicitly alongside the existing RC snapshot theorem inventory
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a helper-level safety corollary slice, not the full RC target

## 2026-04-17 — Stage 4.2 decrement positive-count follow-up

I widened the current RC snapshot proof slice with `KaliCore.Safety.releaseAndDecrementKeepsOtherPositiveCountCells`, then synced the proof-boundary manifest plus the verification summaries in `README.md`, `specs/16-testing.md`, and `specs/19-feature-maturity.md` so the published boundary and release-claim wording now name the decrement-path positive-count preservation theorem explicitly.

Suggested follow-up:
- keep the Stage 4.2 RC snapshot inventory aligned if the decrement helper widens again
- keep the published boundary intentionally narrower than the full ownership/freeing target

## 2026-04-17 — Stage 4.2 origin/positivity helper sync

I widened the proof-backed RC snapshot slice with `KaliCore.Safety.releaseAndCollectHeapCellOriginAndPositiveCount`, which bundles the surviving-cell origin and positive-count facts for the local collection helper on top of the existing origin and positivity lemmas.

Suggested follow-up:
- keep the published boundary, verification summaries, and test inventory aligned if the local collection helper widens again
- treat this as a helper-level proof-maintenance pass, not a boundary-wide ownership/freeing widening

## 2026-04-17 — Stage 4.2 proof-summary string sync

I updated `specs/19-feature-maturity.md` so it now includes the canonical short proof-backed summary string verbatim alongside the RC snapshot theorem names, which keeps the maturity matrix aligned with `proofs/BOUNDARY.md`, `README.md`, and the proof-boundary tests.

Suggested follow-up:
- keep the summary string verbatim whenever the published proof boundary or its theorem inventory changes
- treat this as a wording sync only; the mechanized boundary itself did not widen

## 2026-04-17 — Stage 4.2 testing-summary wording sync

I tightened `specs/16-testing.md` so the repository-state note and proof-backed-claims guidance now name the RC snapshot helper slice more explicitly, including `KaliCore.Safety.releaseAndCollectKeepsTargetCellWhenPositiveCount`, `KaliCore.Safety.releaseAndCollectHeapIsPositiveCountFilter`, `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount`, and the pure release-helper ownership/disjointness corollaries.

Suggested follow-up:
- keep the wording narrow; this is a summary-doc precision pass, not a boundary widening

## 2026-04-17 — Stage 4.2 testing-summary sync follow-up

I synced `specs/16-testing.md` so the repository-state note and proof-backed claims guidance now explicitly mention the current RC snapshot helper slice's heap-characterisation, target-cell retention, and final-heap positive-count theorems alongside the other release-helper claims.

Suggested follow-up:
- keep the Stage 4.2 claim narrow; this is still a summary-doc wording pass, not a boundary widening

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

## 2026-04-17 — Stage 4.2 RC helper theorem inventory sync

If the proof slice widens with a bundled `releaseAndCollect` heap-cell theorem that combines provenance, ownership/name preservation, and positive-count facts, the proof-boundary summary owners in `proofs/BOUNDARY.md`, `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` should stay aligned with the expanded theorem inventory.

Suggested follow-up:
- keep the proof claims narrow and verbatim to the published boundary
- update the maturity and verification summaries only if the new theorem is reflected in the proof boundary and actual Lean source set

## 2026-04-17 — Stage 4.2 release-and-decrement ownership sync

I widened the proof-backed RC snapshot slice with `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndOwnership`, which makes the decrement helper's surviving heap provenance and ownership tag explicit alongside the existing target-cell and collection-helper bundle theorems.

Suggested follow-up:
- sync `proofs/BOUNDARY.md`, `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` so the published proof-boundary inventory names the new decrement-helper provenance/ownership theorem
- keep the broader Stage 4.2 ownership/freeing target narrower than this helper-level slice

## 2026-04-17 — Verification summary no-dangling naming sync

I plan to make the proof-summary docs name the helper-level no-dangling-reference corollaries explicitly (`KaliCore.Safety.releaseRefNoDanglingReference`, `KaliCore.Safety.releaseAndDecrementNoDanglingReference`, and `KaliCore.Safety.releaseAndCollectNoDanglingReference`) so the README and spec summaries match the current proof-boundary inventory more tightly.

Suggested follow-up:
- update `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` so the proof-summary wording names the no-dangling corollaries explicitly alongside the rest of the RC snapshot slice
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a summary-doc naming sync, not a boundary widening

## 2026-04-17 — Stage 4.2 exact releasedRefs bookkeeping follow-up

I plan to widen the current RC snapshot proof slice with exact `releasedRefs` cons-shape theorems for the release-only, decrement, and collection helpers, so the proof-boundary manifest can name the release bookkeeping shape explicitly alongside the existing live-reference and no-dangling corollaries.

Suggested follow-up:
- update `proofs/BOUNDARY.md`, `README.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, and the Stage 4.2 status tracker so the new releasedRefs bookkeeping theorems are named explicitly
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a helper-level bookkeeping slice, not the full RC target

## 2026-04-18 — Stage 4.2 exact releasedRefs wording sync

I found one remaining verification-summary drift point: the published boundary already names the exact released-reference cons-shape theorems, but the root summary docs still describe that slice a bit too generically.

Suggested follow-up:
- update `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` so the release-only, decrement, and collection helpers are named explicitly via `KaliCore.Safety.releaseRefReleasedRefsCons`, `KaliCore.Safety.releaseAndDecrementReleasedRefsCons`, and `KaliCore.Safety.releaseAndCollectReleasedRefsCons`
- keep the claim narrow: this is a wording sync for the published boundary, not a boundary widening

## 2026-04-18 — Stage 4.2 release-and-decrement origin/ownership/positivity follow-up

I plan to widen the current RC snapshot proof slice with a bundled theorem for surviving `releaseAndDecrement` heap cells that packages the original-heap provenance, name/ownership preservation, and positive-count fact together, so the proof-boundary summary can name the combined helper fact explicitly if that slice lands.

Suggested follow-up:
- update `proofs/BOUNDARY.md`, `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` if the new theorem lands
- keep the broader Stage 4.2 ownership/freeing target incremental; this would still be a helper-level conjunction theorem, not the full RC target

## 2026-04-18 — Stage 4.2 decrement origin/positive-count follow-up

I widened the current RC snapshot proof slice with `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndPositiveCount`, which packages the decrement helper's surviving-cell provenance and positive-count fact in one helper theorem alongside the existing RC snapshot inventory.

Suggested follow-up:
- sync `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, and `proofs/BOUNDARY.md` so the published boundary names the new decrement origin/positive-count helper explicitly
- keep the broader Stage 4.2 ownership/freeing target incremental; this is still a helper-level provenance/positivity theorem, not the full RC target

## 2026-04-18 — Stage 4.2 heap-filter anti-drift widening

I found one remaining proof-summary drift point in the verification wording: the published boundary already names `KaliCore.Safety.releaseAndCollectHeapIsPositiveCountFilter` explicitly, but the high-level release-claim summaries still describe that helper only indirectly.

Suggested follow-up:
- update the README and verification/maturity summaries so the heap/filter theorem is named explicitly alongside the current RC snapshot inventory
- keep the claim narrow: this is a wording / anti-drift sync for the published boundary, not a boundary widening

## 2026-04-18 — Stage 4.2 pure release helper origin/ownership follow-up

I added the direct pure release-helper theorem `KaliCore.Safety.releaseRefHeapCellOriginAndOwnership`, which packages origin and ownership preservation for `releaseRef` so the published boundary can name the release-only heap provenance story more explicitly alongside `releaseRefHeapCharacterisation`.

Suggested follow-up:
- update `proofs/BOUNDARY.md`, `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` so the new theorem is named explicitly
- keep the claim narrow: this is still a helper-level provenance theorem on the published boundary, not a broader ownership/freeing target

## 2026-04-18 — Stage 4.2 unrelated-heap / other-live wording sync

I found one remaining proof-summary drift point: the published boundary already mechanizes the unrelated-heap and other-live helper theorems, but the top-level summary docs still described those slices only generically.

Suggested follow-up:
- update `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` so the summary prose names `KaliCore.Safety.releaseAndDecrementKeepsOtherHeapEntries`, `KaliCore.Safety.releaseAndCollectKeepsOtherHeapEntries`, `KaliCore.Safety.releaseAndDecrementPreservesOtherLiveRefs`, and `KaliCore.Safety.releaseAndCollectPreservesOtherLiveRefs` explicitly
- keep the claim narrow: this is a wording / anti-drift sync for the published boundary, not a boundary widening

## 2026-04-18 — Stage 4.2 decrement target-origin follow-up

I widened the proof model with a target-origin theorem for the release-and-decrement helper, so the published boundary can now name `KaliCore.Safety.releaseAndDecrementTargetCellOrigin` explicitly alongside the existing target-cell bookkeeping and heap-characterisation theorems.

Suggested follow-up:
- update the verification summaries and maturity docs so the new theorem is named explicitly
- keep the claim narrow: this is still a helper-level RC slice, not the fuller ownership/freeing target


## 2026-04-19 — Stage 4.2 RC predicate vocabulary sync

I made the proof-boundary and verification summaries explicitly name the RC snapshot predicate vocabulary (`hasOwnership`, `allocated`, and `liveAnnotated`) so the model-shape wording stays as concrete as the theorem inventory.

Suggested follow-up:
- keep this as a wording-only sync; it does not widen the published proof boundary

## 2026-04-19 — Stage 4.2 decrement target positive-count iff bridge

I synced the proof-summary docs so they now name `KaliCore.Safety.releaseAndDecrementTargetCellPositiveCountIff` explicitly alongside the current RC snapshot inventory.

Planned update:
- keep `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` aligned with the published boundary wording whenever the RC snapshot slice widens again
- keep the claim narrow: this is a proof-summary / anti-drift sync for the published boundary, not a boundary widening

## 2026-04-19 — Stage 4.2 ownership provenance wording sync

I synchronized the RC snapshot provenance wording so `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` is spelled out directly alongside `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCount` in the summary docs and Stage 4.2 progress trackers.

Planned update:
- keep the companion theorem named directly in `README.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, `PLAN-4.2-STATUS.md`, and `plan/phase-4/02-formal-verification-depth.md` whenever the RC snapshot wording changes again
- keep the claim narrow: this is a wording / anti-drift sync for the published boundary, not a boundary widening

## 2026-04-19 — Stage 4.2 decrement linear-memory companion widening

I found a small remaining gap in the RC snapshot proof slice: the decrement helper already has the origin/ownership/positive-count theorem, and the summary docs now name the matching linear-memory companion explicitly as well.

Planned update:
- add `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` to the proof-backed RC slice, then sync the proof boundary, verification summaries, and anti-drift test so the new companion theorem is named explicitly alongside the current decrement helper inventory
- keep the claim narrow: this is still a helper-level RC proof sync, not the broader Stage 4.2 ownership/freeing target

Completed:
- the proof-backed boundary summary, stage plan, and anti-drift guard now also name `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` explicitly.

## 2026-04-19 — Stage 4.2 release-and-collect origin/positive-count linear-memory companion widening

I found a small RC-slice widening that fits the current proof-backed boundary pattern: the local `releaseAndCollect` helper can name the combined origin / positive-count + linear-memory companion explicitly, mirroring the existing release-only and decrement helper companion style.

Planned update:
- add `KaliCore.Safety.releaseAndCollectHeapCellOriginAndPositiveCountAndLinearMemory` to `proofs/KaliCore/Safety.lean`, then sync `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`, `proofs/BOUNDARY.md`, `PLAN-4.2-STATUS.md`, `plan/phase-4/02-formal-verification-depth.md`, `TODO.md`, and `crates/kali_cli/tests/schema_docs.rs` so the new companion theorem is named explicitly in the published boundary and drift guard
- keep the claim narrow: this is a helper-level proof-summary widening for the published boundary, not the broader Stage 4.2 ownership/freeing target

Completed:
- the proof-backed boundary summary, stage plan, and anti-drift guard now also name `KaliCore.Safety.releaseAndCollectHeapCellOriginAndPositiveCountAndLinearMemory` explicitly.

## 2026-04-19 — Stage 4.2 final-heap positive-count wording sync

I found one remaining proof-summary drift point in the RC snapshot slice: the local collection helper's final-heap positivity theorem `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount` is mechanized already, but some summary prose still describes it generically.

Planned update:
- sync the verification summary docs (`README.md`, `specs/17-verification.md`, `specs/19-feature-maturity.md`) so the theorem is named explicitly wherever the proof-backed boundary inventory is repeated
- keep the claim narrow: this is a wording / anti-drift sync for the published boundary, not a boundary widening

## 2026-04-19 — Stage 4.2 no-dangling-reference summary sync

I found a small proof-summary drift gap: the published boundary already names `KaliCore.Safety.noDanglingReference`, but the summary docs tracked by the proof-summary anti-drift guard do not mention it explicitly yet.

Planned update:
- sync the verification-facing summaries in `README.md`, `specs/16-testing.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` so the base no-dangling theorem is named alongside the helper-level corollaries
- keep the claim narrow: this is a wording / anti-drift sync for the published boundary, not a boundary widening
