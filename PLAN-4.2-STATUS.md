# Stage 4.2 Status Tracker

**Canonical summary:** **Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target.**

## Current status

The repository is already proof-backed for the published boundary. The remaining Stage 4.2 work is
not “make the boundary exist” but “widen the boundary without overclaiming”. Keep this tracker,
[`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md), [`plan/phase-4/02-formal-verification-depth.md`](./plan/phase-4/02-formal-verification-depth.md), and [`TODO.md`](./TODO.md) synchronized. The RC snapshot model itself is phrased through the explicit `hasOwnership` / `allocated` / `liveAnnotated` predicate vocabulary, and the schema-docs anti-drift guard now pins those names too, so the summary stays aligned with the model shape as well as the theorem inventory.

## Published RC snapshot theorem inventory

The current published boundary explicitly names the RC snapshot helper slice, including the pure release helper's plain origin theorem `releaseRefHeapCellOrigin` alongside the rest of the RC snapshot helper slice:

- `releaseRefNoDanglingReference`
- `releaseRefLiveRefsAreOwnedAndAllocated`
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
- `releaseRefHeapCellOriginOwnershipAndPositiveCount`
- `releaseRefPreservesOwnership`
- `releaseRefPreservesLinearMemory`
- `releaseRefPreservesOwnershipAndLinearMemory`
- `releaseAndCollectPreservesLinearMemory`
- `releaseAndCollectPreservesOwnershipAndLinearMemory`
- `releaseRefReleasedNotLiveRef`
- `releaseAndDecrementNoDanglingReference`
- `releaseAndDecrementKeepsTargetCellWhenPositiveCount`
- `releaseAndDecrementTargetCellOrigin`
- `releaseAndDecrementKeepsOriginalPositiveCountCells`
- `releaseAndDecrementKeepsOtherHeapEntries`
- `releaseAndDecrementPreservesOtherLiveRefs`
- `releaseAndDecrementHeapCellOrigin`
- `releaseAndDecrementHeapCellOriginAndOwnership`
- `releaseAndDecrementHeapCellOriginAndPositiveCount`
- `releaseAndDecrementHeapCellOriginOwnershipAndPositiveCount`
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
- `releaseAndCollectHeapCellOriginOwnershipAndPositiveCount`
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
