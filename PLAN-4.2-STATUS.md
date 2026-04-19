# Stage 4.2 Status Tracker

**Canonical summary:** **Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target.**

## Current status

The repository is already proof-backed for the published boundary. The remaining Stage 4.2 work is
not “make the boundary exist” but “widen the boundary without overclaiming”. Keep this tracker,
[`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md), [`plan/phase-4/02-formal-verification-depth.md`](./plan/phase-4/02-formal-verification-depth.md), and [`TODO.md`](./TODO.md) synchronized. The RC snapshot model itself is phrased through the explicit `hasOwnership` / `allocated` / `liveAnnotated` predicate vocabulary, and the new live-annotated helper bundles `KaliCore.Safety.releaseRefLiveRefsAreOwnedAndAllocated`, `releaseRefLiveRefsAreLiveAnnotated`, `releaseAndDecrementLiveRefsAreLiveAnnotated`, and `releaseAndCollectLiveRefsAreLiveAnnotated` keep that vocabulary explicit across the release-only, decrement, and collection helpers; the schema-docs anti-drift guard now pins those names too, and now also pins the live-reference ownership/allocation projection theorem, the ownership-preservation corollaries, the surviving-live-reference corollary on the collection path, the released-not-live theorems, and the decrement-path positive-count guard so the summary stays aligned with the model shape as well as the theorem inventory; the guard also names `KaliCore.Safety.liveRefsAreOwnedAndAllocated`, `KaliCore.Safety.releaseAndCollectLiveRefsAreOwnedAndAllocated`, `KaliCore.Safety.releaseAndDecrementPreservesOwnership`, `KaliCore.Safety.releaseAndCollectPreservesOwnership`, `releasedNotLive`, and `releasedNotLiveRef` explicitly. The companion linear-memory theorem `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` is now spelled out directly in the tracker so the provenance slice stays explicit rather than relying on shortened companion wording, and the decrement helper now has the matching companion theorem `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` so the provenance slice stays explicit on both helper paths. The published boundary also already includes the explicit linear-memory payload preservation corollaries, the combined wellformedness/linear-memory corollaries, and the target-cell iff bridges `KaliCore.Safety.releaseAndDecrementTargetCellPositiveCountIff` and `KaliCore.Safety.releaseAndCollectTargetCellPresentIffPositiveCount`, plus the target-cell allocation corollaries `KaliCore.Safety.releaseAndDecrementTargetCellAllocatedWhenPositiveCount` and `KaliCore.Safety.releaseAndCollectTargetCellAllocatedWhenPositiveCount`, so the remaining widening work sits beyond those payload and allocation bridges, while the local collection helper's heap-filter-and-linear-memory corollary `KaliCore.Safety.releaseAndCollectHeapIsPositiveCountFilterAndLinearMemory` keeps that payload explicitly paired with the heap-filter story; the widened soundness slice also explicitly names `KaliCore.Soundness.subst_closed` so the proof inventory stays complete.

## Published RC snapshot theorem inventory

The current published boundary explicitly names the RC snapshot helper slice, including the pure release helper's plain origin theorem `releaseRefHeapCellOrigin` alongside the rest of the RC snapshot helper slice:

- `releaseRefNoDanglingReference`
- `releaseRefLiveRefsAreOwnedAndAllocated`, `releaseRefLiveRefsAreLiveAnnotated`, `releaseAndDecrementLiveRefsAreLiveAnnotated`, `releaseAndCollectLiveRefsAreLiveAnnotated`
- `releaseRefLiveRefsFiltered`
- `releaseAndDecrementLiveRefsFiltered`
- `releaseAndCollectLiveRefsFiltered`
- `releasePreservesWellFormed`
- `releaseRecorded`
- `releaseAndDecrementRecorded`
- `releaseAndDecrementDecrementsTargetCell`
- `releaseAndDecrementPreservesWellFormed`
- `releaseAndDecrementLiveRefsAreOwnedAndAllocated`
- `releaseAndDecrementReleasedNotLiveRef`
- `releaseAndDecrementZeroesLastTargetCell`
- `releaseRefReleasedRefsCons`
- `releaseRefPreservesReleasedRefs`
- `releaseRefHeapCharacterisation`
- `releaseRefHeapCellOrigin`
- `releaseRefHeapCellOriginAndOwnership`
- `releaseRefHeapCellOriginOwnershipAndPositiveCount` plus its linear-memory companion `releaseRefHeapCellOriginOwnershipAndPositiveCountAndLinearMemory`
- `releaseRefPreservesOwnership`
- `releaseRefPreservesLinearMemory`
- `releaseRefPreservesOwnershipAndLinearMemory`
- `releaseAndCollectPreservesLinearMemory`
- `releaseAndCollectHeapIsPositiveCountFilterAndLinearMemory`
- `releaseAndCollectPreservesOwnershipAndLinearMemory`
- `releaseRefPreservesWellFormedAndLinearMemory`
- `releaseAndDecrementPreservesWellFormedAndLinearMemory`
- `releaseAndCollectPreservesWellFormedAndLinearMemory`
- `releaseRefReleasedNotLiveRef`
- `releaseAndDecrementNoDanglingReference`
- `releaseAndDecrementKeepsTargetCellWhenPositiveCount`
- `releaseAndDecrementTargetCellPositiveCountIff`
- `releaseAndDecrementTargetCellOrigin`
- `releaseAndDecrementKeepsOtherPositiveCountCells`
- `releaseAndDecrementKeepsOriginalPositiveCountCells`
- `releaseAndDecrementKeepsOtherHeapEntries`
- `releaseAndDecrementPreservesOtherLiveRefs`
- `releaseAndDecrementHeapCellOrigin`
- `releaseAndDecrementHeapCellOriginAndOwnership`
- `releaseAndDecrementHeapCellOriginAndPositiveCount`
- `releaseAndDecrementHeapCellOriginOwnershipAndPositiveCount`
- `releaseAndDecrementHeapCellOriginOwnershipAndPositiveCountAndLinearMemory`
- `releaseAndDecrementTargetCellAllocatedWhenPositiveCount`
- `releaseAndDecrementTargetCellOwnedAndAllocatedWhenPositiveCount`
- `releaseAndDecrementPreservesLinearMemory`
- `releaseAndDecrementPreservesOwnershipAndLinearMemory`
- `releaseAndDecrementReleasedRefsCons`
- `releaseAndDecrementPreservesReleasedRefs`
- `releaseAndCollectNoDanglingReference`
- `releaseAndCollectRecorded`
- `releaseAndCollectDropsZeroCountCells`
- `releaseAndCollectRemovesZeroCountCells`
- `releaseAndCollectKeepsPositiveCountCells`
- `releaseAndCollectDropsOriginalZeroCountCells`
- `releaseAndCollectKeepsOtherPositiveCountCells`
- `releaseAndCollectKeepsOriginalPositiveCountCells`
- `releaseAndCollectKeepsOtherHeapEntries`
- `releaseAndCollectPreservesOtherLiveRefs`
- `releaseAndCollectKeepsTargetCellWhenPositiveCount`
- `releaseAndCollectTargetCellPresentIffPositiveCount`
- `releaseAndCollectPreservesWellFormed`
- `releaseAndCollectReleasedNotLiveRef`
- `releaseAndCollectHeapCellOrigin`
- `releaseAndCollectHeapCellOriginAndOwnership`
- `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCount`, plus its linear-memory companion `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCountAndLinearMemory`
- `releaseAndCollectHeapCellOriginAndPositiveCount`
- `releaseAndCollectHeapIsPositiveCountFilter`
- `releaseAndCollectHeapCellsHavePositiveCount`
- `releaseAndCollectTargetCellAllocatedWhenPositiveCount`
- `releaseAndCollectTargetCellOrigin`
- `releaseAndCollectTargetCellOriginOwnershipAndPositiveCount`
- `releaseAndCollectTargetCellOwnedAndAllocatedWhenPositiveCount`
- `releaseAndCollectPreservesLinearMemory`
- `releaseAndCollectPreservesOwnershipAndLinearMemory`
- `releaseAndCollectReleasedRefsCons`
- `releaseAndCollectPreservesReleasedRefs`
- `releaseAndDecrementHeapCharacterisation`
- `releaseAndCollectHeapCharacterisation`

## Published lowering slice

The current published lowering slice explicitly names:

- `KaliIR.Value`
- `KaliIR.LoweringCorrectness.lower_preserves_value`
- `KaliIR.LoweringCorrectness.lower_preserves_step`
- `KaliIR.LoweringCorrectness.lower_preserves_steps`

## Remaining widening work

The remaining work is to widen the proof-backed boundary beyond the current published slice while
keeping the summary honest:

- broaden the ownership / RC story beyond the current snapshot helper model,
- broaden lowering correctness beyond the current HIR subset,
- keep every new theorem/property mirrored in `proofs/BOUNDARY.md`, this tracker, and the stage plan before claiming wider proof-backed coverage.
