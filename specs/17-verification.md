# 17 — Formal Verification

Kali uses Lean 4 to verify selected high-value invariants over time. Verification is iterative and bounded: public claims are limited to the currently published boundary in [`proofs/BOUNDARY.md`](../proofs/BOUNDARY.md).

Planning ownership:
- this chapter defines the **verification program**, proof-boundary discipline, and claim rules
- [`PLAN.md`](../PLAN.md) and [`plan/`](../plan) own milestone sequencing, proof-work ordering, and implementation tasks
- [`proofs/BOUNDARY.md`](../proofs/BOUNDARY.md) owns the repository's **current** proof-backed scope

## Core verification rule

Lean proofs justify claims only for the subsystem slice named by the published proof boundary. They do **not** automatically:
- widen feature maturity rows,
- replace implementation or conformance testing,
- imply blanket coverage of the full JS/TS surface or all host behavior.

## Proof-ready vs proof-backed

Kali distinguishes two verification states:

| State | Minimum requirement | Allowed public claim |
|---|---|---|
| **proof-ready** | published `proofs/BOUNDARY.md`, honest proof-CI trigger policy, and no-overclaim discipline | the repository is prepared for phased verification work |
| **proof-backed** | non-empty published boundary naming concrete modeled subsystems plus mechanized theorem/property inventory | formal verification may be cited, but only for the published boundary |

Rules:
- Phase 1 requires the **proof-ready** baseline.
- Proof-backed release/support wording requires a non-empty published boundary.
- The current verification state is always read from [`proofs/BOUNDARY.md`](../proofs/BOUNDARY.md), not inferred from roadmap prose.

## Scope discipline

Verification targets a **core Kali calculus** and selected high-value implementation invariants first. Early proof work should not overclaim the full language surface.

Late-compatibility features such as dynamic code execution, dynamic module loading, weak/finalization semantics, and concrete browser/OS host behavior remain outside the proof boundary until their semantics are stable enough to model honestly.

## Published proof boundary

`proofs/BOUNDARY.md` is the canonical published statement of:
- the modeled calculus/subsystem slice currently covered,
- the named theorems/properties currently claimed,
- trusted assumptions and explicitly unmodeled features,
- covered implementation/spec paths, and
- the proof-CI trigger rule.

Release notes, README summaries, and maturity claims must treat that manifest as the single source of truth for current proof status.
- current proof-backed boundary snapshot: **Verification**: reuse the canonical repository summary from [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md): **Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target.** The published boundary currently includes the widened closed fragment plus a small RC snapshot safety slice with live-reference ownership/allocation projection via the explicit `hasOwnership` / `allocated` / `liveAnnotated` predicate vocabulary, exact live-reference filtering via the release-only helper theorem `KaliCore.Safety.releaseRefLiveRefsFiltered` and the decrement/collection theorems `KaliCore.Safety.releaseAndDecrementLiveRefsFiltered` and `KaliCore.Safety.releaseAndCollectLiveRefsFiltered`, the release-only helper's live-reference ownership/allocation corollary `KaliCore.Safety.releaseRefLiveRefsAreOwnedAndAllocated`, `releaseRefLiveRefsAreLiveAnnotated`, `releaseAndDecrementLiveRefsAreLiveAnnotated`, `releaseAndCollectLiveRefsAreLiveAnnotated`, the release-only helper's live-reference filtering corollary `KaliCore.Safety.releaseRefLiveRefsFiltered`, and the live-to-released transition preservation `KaliCore.Safety.releasePreservesWellFormed`, explicit release-recording and exact released-reference cons-shape via `KaliCore.Safety.releaseRefReleasedRefsCons`, `KaliCore.Safety.releaseRefHeapCharacterisation`, `KaliCore.Safety.releaseRefHeapCellOrigin`, `KaliCore.Safety.releaseRefHeapCellOriginAndOwnership`, `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCountAndLinearMemory`, `KaliCore.Safety.releaseAndDecrementReleasedRefsCons`, and `KaliCore.Safety.releaseAndCollectReleasedRefsCons` on the release-only, decrement, and collection helpers, pure release-helper ownership/allocation and disjointness corollaries, ownership-envelope preservation on the release-only, decrement, and collection helpers, release-set preservation on the release-only, decrement, and collection helpers via `KaliCore.Safety.releaseRefPreservesReleasedRefs`, `KaliCore.Safety.releaseAndDecrementPreservesReleasedRefs`, and `KaliCore.Safety.releaseAndCollectPreservesReleasedRefs`, target-cell decrement bookkeeping, heap-origin provenance for the release-and-decrement helper, the release-and-decrement target-cell origin theorem `KaliCore.Safety.releaseAndDecrementTargetCellOrigin` and the target-cell origin/positive-count theorem `KaliCore.Safety.releaseAndDecrementTargetCellOriginAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseAndDecrementTargetCellOriginAndPositiveCountAndLinearMemory`, the release-and-decrement provenance-and-ownership theorem `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndOwnership`, the release-and-decrement origin-and-positive-count theorem `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndPositiveCount`, the release-and-decrement origin/ownership/positivity theorem `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCount`, plus its linear-memory companion `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCountAndLinearMemory`, the heap-characterisation theorems `KaliCore.Safety.releaseAndDecrementHeapCharacterisation` and `KaliCore.Safety.releaseAndCollectHeapCharacterisation`, the release-and-decrement positive-count preservation theorem `KaliCore.Safety.releaseAndDecrementKeepsOtherPositiveCountCells`, `KaliCore.Safety.releaseAndDecrementKeepsOriginalPositiveCountCells`, the release-and-decrement target-cell positive-count preservation theorem `KaliCore.Safety.releaseAndDecrementKeepsTargetCellWhenPositiveCount`, the release-and-decrement target-cell positive-count iff theorem `KaliCore.Safety.releaseAndDecrementTargetCellPositiveCountIff` plus its linear-memory companion `KaliCore.Safety.releaseAndDecrementTargetCellPositiveCountIffAndLinearMemory`, the release-and-decrement target-allocation corollary `KaliCore.Safety.releaseAndDecrementTargetCellOrigin`, `KaliCore.Safety.releaseAndDecrementTargetCellAllocatedWhenPositiveCount`, `KaliCore.Safety.releaseAndDecrementTargetCellOwnedAndAllocatedWhenPositiveCount`, the release-and-collect target-cell iff theorem `KaliCore.Safety.releaseAndCollectTargetCellPresentIffPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseAndCollectTargetCellPresentIffPositiveCountAndLinearMemory`, the release-and-collect target-allocation corollary `KaliCore.Safety.releaseAndCollectTargetCellAllocatedWhenPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellOrigin`, `KaliCore.Safety.releaseAndCollectTargetCellOriginAndPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellOriginOwnershipAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseAndCollectTargetCellOriginOwnershipAndPositiveCountAndLinearMemory`, `KaliCore.Safety.releaseAndCollectTargetCellOwnedAndAllocatedWhenPositiveCount`, and the bundled `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCount` helper theorem, plus its linear-memory companion `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCountAndLinearMemory`, last-ref zeroing, zero-count collection, zero-count removal from the decrement pass via `releaseAndCollectDropsZeroCountCells`, zero-count removal from the collected heap via `releaseAndCollectRemovesZeroCountCells`, positive-count preservation on the local collection helper, the helper-level theorem `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount` that the local collection helper's final heap contains only positive-count cells, the helper-level theorem `KaliCore.Safety.releaseAndCollectKeepsOtherPositiveCountCells`, `KaliCore.Safety.releaseAndCollectKeepsOriginalPositiveCountCells` that positive-count cells from the original heap survive when they are not the released target and remain positive-count after collection, the helper-level theorem `KaliCore.Safety.releaseAndCollectKeepsTargetCellWhenPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellAllocatedWhenPositiveCount` that the released target remains in the collected heap when its decremented count stays positive, the helper-level theorem `KaliCore.Safety.releaseAndCollectHeapCellOriginAndOwnership` that the surviving collection-helper cells preserve their original name and ownership tag, `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCountAndLinearMemory`, the helper-level theorem `KaliCore.Safety.releaseAndCollectHeapCellOriginAndPositiveCount` that the surviving collection-helper cells are both traceable to the original heap and positive-count, the helper-level theorem `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount` that the local collection helper's final heap contains only positive-count cells, the helper-level theorem `KaliCore.Safety.releaseAndCollectHeapCellOrigin` that every surviving release-and-collect heap cell still comes from the original heap with only the released target decremented, unrelated-heap preservation via `KaliCore.Safety.releaseAndDecrementKeepsOtherHeapEntries` and `KaliCore.Safety.releaseAndCollectKeepsOtherHeapEntries`, the helper-level theorem that original zero-count cells are dropped from the final heap, other-live-reference preservation via `KaliCore.Safety.releaseAndDecrementPreservesOtherLiveRefs` and `KaliCore.Safety.releaseAndCollectPreservesOtherLiveRefs`, the helper-level ownership/allocation preservation corollaries on the decrement and collection paths, the mechanized `KaliCore.Safety.noDanglingReference` theorem plus the helper-level no-dangling-reference corollaries `KaliCore.Safety.releaseRefNoDanglingReference`, `KaliCore.Safety.releaseAndDecrementNoDanglingReference`, and `KaliCore.Safety.releaseAndCollectNoDanglingReference`, and a refcount-decrement update helper, plus the remaining bookkeeping corollaries `KaliCore.Safety.releaseRecorded`, `KaliCore.Safety.releaseAndDecrementRecorded`, `KaliCore.Safety.releaseAndDecrementDecrementsTargetCell`, `KaliCore.Safety.releaseAndDecrementPreservesWellFormed`, `KaliCore.Safety.releaseAndDecrementLiveRefsAreOwnedAndAllocated`, `KaliCore.Safety.releaseAndDecrementReleasedNotLiveRef`, `KaliCore.Safety.releaseAndDecrementZeroesLastTargetCell`, `KaliCore.Safety.releaseAndCollectRecorded`, `KaliCore.Safety.releaseAndCollectKeepsPositiveCountCells`, `KaliCore.Safety.releaseAndCollectDropsOriginalZeroCountCells`, `KaliCore.Safety.releaseAndCollectPreservesWellFormed`, `KaliCore.Safety.releaseAndCollectReleasedNotLiveRef`, `KaliCore.Safety.releaseAndCollectRemovesZeroCountCells`, `KaliCore.Safety.releaseRefPreservesOwnership`, `KaliCore.Safety.releaseRefReleasedNotLiveRef`, `releasedNotLive`, and `releasedNotLiveRef`, plus a widened HIR lowering-correctness slice that now also includes `KaliIR.Value`, `KaliIR.LoweringCorrectness.lower_preserves_value`, and bare throw. The RC snapshot provenance wording also now spells out `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` explicitly alongside `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCount`, so the companion theorem is named directly where the summary needs it.
- lowering theorem names pinned here for the proof-backstop summary: `KaliIR.Value`, `KaliIR.LoweringCorrectness.lower_preserves_value`, `KaliIR.LoweringCorrectness.lower_preserves_step`, `KaliIR.LoweringCorrectness.lower_preserves_steps`
- additional proof-summary theorem names pinned here for the proof-backstop summary: `KaliCore.Soundness.subst_closed`, `KaliCore.litTy`
- additional proof-summary theorem names pinned here for the proof-backstop summary: `releaseAndCollectLiveRefsAreOwnedAndAllocated`, `liveRefsAreOwnedAndAllocated`, `releaseAndDecrementPreservesOwnership`, `releaseRefHeapCharacterisationAndLinearMemory`, `releaseRefPreservesLinearMemory`, `releaseRefPreservesOwnershipAndLinearMemory`, `releaseAndCollectPreservesLinearMemory`, `releaseAndCollectHeapIsPositiveCountFilterAndLinearMemory`, `releaseAndCollectPreservesOwnershipAndLinearMemory`, `releaseRefPreservesWellFormedAndLinearMemory`, `releaseAndDecrementPreservesWellFormedAndLinearMemory`, `releaseAndCollectPreservesWellFormedAndLinearMemory`, `releaseRefPreservesWellFormedAndOwnershipAndLinearMemory`, `releaseAndDecrementPreservesWellFormedAndOwnershipAndLinearMemory`, `releaseAndCollectPreservesWellFormedAndOwnershipAndLinearMemory`, `releaseRefPreservesWellFormedAndOwnership`, `releaseAndDecrementPreservesWellFormedAndOwnership`, `releaseAndCollectPreservesWellFormedAndOwnership`, `releaseAndDecrementPreservesLinearMemory`, `releaseAndDecrementPreservesOwnershipAndLinearMemory`, `releaseAndCollectPreservesOwnership`, `releaseAndCollectHeapCellOriginAndPositiveCountAndLinearMemory`, `releaseAndCollectHeapIsPositiveCountFilter`, `releaseAndCollectPreservesLinearMemory`, `releaseAndCollectPreservesOwnershipAndLinearMemory`, `releaseAndCollectLiveRefsAreOwnedAndAllocated`, `releaseAndDecrementHeapCharacterisationAndLinearMemory`, `releaseAndCollectHeapCharacterisationAndLinearMemory`

## Covered-boundary edit discipline

Once the published boundary is non-empty:
- if a change touches a subsystem or invariant named inside the boundary, the same change must either update the matching Lean model/proofs or narrow the published boundary first
- widening the boundary requires explicitly naming the new covered paths and theorem inventory in `proofs/BOUNDARY.md`
- shrinking the boundary is allowed as an honesty move, but all public wording must immediately follow the narrower boundary

## Verification focus areas

### Type-system soundness
For the modeled core fragment, verification should prioritize:
- progress
- preservation
- realistic structural-typing lemmas needed by the proved fragment
- termination/decidability results only for the explicit inference fragment being verified

Do not overclaim principality or full-language soundness for the whole TypeScript-compatible surface.

### Effect-system correctness
For the modeled capability subset, verification should prioritize:
- conservative effect inference
- sound sandbox-policy decision/enforcement behavior

### Memory safety
For the modeled ownership/reference-counting fragment, verification should prioritize:
- no-dangling-reference style safety invariants
- soundness of ownership/escape analysis assumptions used by the model
- refcount-helper invariants for the published RC snapshot slice named in `proofs/BOUNDARY.md`

The exact current theorem inventory belongs in the published boundary, not duplicated here.

### Selective lowering correctness
Where modeled, prove high-value lowering/desugaring steps preserve the intended semantics of the verified fragment.

## Proof-backed support boundary

A proof claim is one evidence lane among several. Public support wording should require both:
1. the proof claim staying inside the published proof boundary, and
2. the matching implementation/testing evidence from [16 — Testing](./16-testing.md) for the command/profile being claimed.

This prevents proof prose from outpacing runtime, package, sandbox, or CLI evidence.

## Methodology

### Modeling
- Define a simplified operational semantics for Kali's core language in Lean.
- Model only the fragment needed for the current proof claim.
- Keep unmodeled features explicit in the proof boundary.

### Proof ↔ implementation link
Lean proves properties of the model. The implementation is kept aligned through:
- tests derived from the model,
- regression cases informed by proof work, and
- review of the implementation/spec correspondence for covered paths.

### Iterative workflow
1. define or revise the model
2. prove the target properties
3. align the implementation with the model
4. add or update evidence showing the covered behavior matches the claim

## CI discipline

Proof CI follows the published boundary:
- proof jobs always trigger for changes under `proofs/`
- once the boundary names covered implementation/spec paths, proof jobs also trigger for changes to those covered areas
- areas outside the current boundary may evolve without widening proof claims

Hosted CI layout and milestone sequencing belong to the implementation plan rather than this chapter.

## Non-goals

This chapter does **not** claim:
- full ECMA-262 formalization,
- full-host end-to-end verification,
- blanket proof coverage for all TypeScript-compatible surface features,
- verification of every backend/tooling component merely because a core calculus fragment is proved.

## Practical implementation note

Concrete proof milestones, Lean-project staging, and deeper verification expansion belong to the plan set, primarily:
- [`PLAN.md`](../PLAN.md)
- [`plan/phase-2/04-lean-model-foundation.md`](../plan/phase-2/04-lean-model-foundation.md)
- [`plan/phase-4/02-formal-verification-depth.md`](../plan/phase-4/02-formal-verification-depth.md)
- the current proof-summary inventory also pins `KaliCore.Safety.releaseAndCollectLiveRefsAreOwnedAndAllocatedAndLinearMemory`
