# Stage 4.2 Status Tracker

**Canonical summary:** **Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target.**

## Current status

The repository is already proof-backed for the published boundary. The remaining Stage 4.2 work is
not “make the boundary exist” but “widen the boundary without overclaiming”. Keep this tracker,
[`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md), [`plan/phase-4/02-formal-verification-depth.md`](./plan/phase-4/02-formal-verification-depth.md), and [`TODO.md`](./TODO.md) synchronized. The RC snapshot model itself is phrased through the explicit `hasOwnership` / `allocated` / `liveAnnotated` predicate vocabulary, and the new live-annotated helper bundles `KaliCore.Safety.releaseRefLiveRefsAreOwnedAndAllocated`, `releaseRefLiveRefsAreLiveAnnotated`, `releaseAndDecrementLiveRefsAreLiveAnnotated`, and `releaseAndCollectLiveRefsAreLiveAnnotated` keep that vocabulary explicit across the release-only, decrement, and collection helpers; the schema-docs anti-drift guard now pins those names too, and now also pins the live-reference ownership/allocation projection theorem, the ownership-preservation corollaries, the surviving-live-reference corollary on the collection path, the released-not-live theorems, the base `KaliCore.Safety.noDanglingReference` theorem, and the decrement-path positive-count guard so the summary stays aligned with the model shape as well as the theorem inventory; the guard also names `KaliCore.Safety.liveRefsAreOwnedAndAllocated`, `KaliCore.Safety.releaseAndCollectLiveRefsAreOwnedAndAllocated`, `KaliCore.Safety.releaseAndDecrementPreservesOwnership`, `KaliCore.Safety.releaseAndCollectPreservesOwnership`, `releasedNotLive`, and `releasedNotLiveRef` explicitly. The top-level plan now mirrors that same explicit naming in the Stage 4.2 verification-depth follow-up lane, so the published-boundary theorem inventory stays aligned across the tracker and the plan. The collection-helper live-reference/linear-memory companion theorem `KaliCore.Safety.releaseAndCollectLiveRefsAreOwnedAndAllocatedAndLinearMemory` is also pinned in that lane so the live-reference slice keeps its linear-memory payload explicit. The companion linear-memory theorem `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` is now spelled out directly in the tracker so the provenance slice stays explicit rather than relying on shortened companion wording, and the decrement helper now has the matching companion theorem `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` so the provenance slice stays explicit on both helper paths. The release-only helper now also has the matching companion theorem `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCountAndLinearMemory`, so the pure release provenance slice stays explicit at the same granularity too. The local collection helper now also carries the matching origin/ownership/positive-count + linear-memory companion `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCountAndLinearMemory`, plus the matching origin/positive-count + linear-memory companion `KaliCore.Safety.releaseAndCollectHeapCellOriginAndPositiveCountAndLinearMemory`. The release-only, decrement, and collection heap-characterisation theorems now also have explicit linear-memory companion forms `KaliCore.Safety.releaseRefHeapCharacterisationAndLinearMemory`, `KaliCore.Safety.releaseAndDecrementHeapCharacterisationAndLinearMemory`, and `KaliCore.Safety.releaseAndCollectHeapCharacterisationAndLinearMemory`. The tracker now also names `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount` explicitly, and the README/spec verification summaries now mirror that wording so the final-heap positivity story stays direct rather than implied. The schema-docs anti-drift guard now pins that theorem name as well, so the summary and CI guard stay aligned. The current widening step now also adds the target-cell positive-count split companions `KaliCore.Safety.releaseAndDecrementTargetCellPositiveCountIffAndLinearMemory` and `KaliCore.Safety.releaseAndCollectTargetCellPresentIffPositiveCountAndLinearMemory` so the decrement and collection target-cell paths stay paired with the explicit linear-memory payload across the published-boundary summary. The published-boundary summary wording now calls those companion theorems out directly instead of leaving them implicit, so the target-cell split and its linear-memory payload remain aligned across the tracker, the proof boundary, and the plan notes.
 The published boundary also already includes the explicit linear-memory payload preservation corollaries, the combined wellformedness/linear-memory corollaries, and the target-cell iff bridges `KaliCore.Safety.releaseAndDecrementTargetCellPositiveCountIff` plus its linear-memory companion `KaliCore.Safety.releaseAndDecrementTargetCellPositiveCountIffAndLinearMemory`, and `KaliCore.Safety.releaseAndCollectTargetCellPresentIffPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseAndCollectTargetCellPresentIffPositiveCountAndLinearMemory`, with the collection helper's released-target survival/removal split kept explicit through `KaliCore.Safety.releaseAndCollectTargetCellPresentIffPositiveCount`, plus the target-cell origin/positive-count theorem `KaliCore.Safety.releaseAndDecrementTargetCellOriginAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseAndDecrementTargetCellOriginAndPositiveCountAndLinearMemory` — which the current progress wording now keeps explicit alongside the target-cell allocation corollaries `KaliCore.Safety.releaseAndDecrementTargetCellAllocatedWhenPositiveCount`, the collection target-cell origin/positive-count theorem `KaliCore.Safety.releaseAndCollectTargetCellOriginAndPositiveCount`, and `KaliCore.Safety.releaseAndCollectTargetCellAllocatedWhenPositiveCount` — so the remaining widening work sits beyond those payload, origin, and allocation bridges, while the local collection helper's heap-filter-and-linear-memory corollary `KaliCore.Safety.releaseAndCollectHeapIsPositiveCountFilterAndLinearMemory` keeps that payload explicitly paired with the heap-filter story; the widened soundness slice also explicitly names `KaliCore.Soundness.subst_closed` and the literal-to-type helper `KaliCore.litTy` so the proof inventory stays complete. The decrement-target origin/positive-count wording sync is now closed too: `KaliCore.Safety.releaseAndDecrementTargetCellOriginAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseAndDecrementTargetCellOriginAndPositiveCountAndLinearMemory` is explicitly named across the published boundary and summary docs.

## Published RC snapshot theorem inventory

The current published boundary explicitly names the RC snapshot helper slice, including the pure release helper's plain origin theorem `releaseRefHeapCellOrigin` alongside the rest of the RC snapshot helper slice:

- `releaseRefNoDanglingReference`
- `KaliCore.Safety.noDanglingReference`
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
- `releaseRefPreservesWellFormedAndOwnershipAndLinearMemory`
- `releaseAndDecrementPreservesWellFormedAndOwnershipAndLinearMemory`
- `releaseAndCollectPreservesWellFormedAndOwnershipAndLinearMemory`
- `releaseRefPreservesWellFormedAndOwnership`
- `releaseAndDecrementPreservesWellFormedAndOwnership`
- `releaseAndCollectPreservesWellFormedAndOwnership`
- `releaseRefReleasedNotLiveRef`
- `releaseAndDecrementNoDanglingReference`
- `releaseAndDecrementKeepsTargetCellWhenPositiveCount`
- `releaseAndDecrementTargetCellPositiveCountIff`
- `releaseAndDecrementTargetCellPositiveCountIffAndLinearMemory`
- `releaseAndDecrementTargetCellOrigin`
- `KaliCore.Safety.releaseAndDecrementTargetCellOriginAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseAndDecrementTargetCellOriginAndPositiveCountAndLinearMemory`
- `releaseAndCollectTargetCellOriginAndPositiveCount`, `releaseAndCollectTargetCellOriginOwnershipAndPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellOriginOwnershipAndPositiveCountAndLinearMemory`
- `releaseAndCollectTargetCellPresentIffPositiveCount`
- `releaseAndCollectTargetCellPresentIffPositiveCountAndLinearMemory`
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
- `releaseAndCollectTargetCellOriginAndPositiveCount`, `releaseAndCollectTargetCellOriginOwnershipAndPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellOriginOwnershipAndPositiveCountAndLinearMemory`
- `releaseAndCollectTargetCellOwnedAndAllocatedWhenPositiveCount`
- `releaseAndCollectPreservesLinearMemory`
- `releaseAndCollectPreservesOwnershipAndLinearMemory`
- `releaseAndCollectReleasedRefsCons`
- `releaseAndCollectPreservesReleasedRefs`
- `releaseAndDecrementHeapCharacterisation`
- `releaseAndCollectHeapCharacterisation`
- `releaseAndCollectHeapCellsHavePositiveCount`

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
