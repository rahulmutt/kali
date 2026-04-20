# Stage 4.2 — Formal Verification Depth

**Phase:** 4 — Advanced Compatibility & Deep Verification
**Spec refs:** [`specs/17-verification.md`](../../specs/17-verification.md), [`specs/16-testing.md`](../../specs/16-testing.md), [`proofs/BOUNDARY.md`](../../proofs/BOUNDARY.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)
**Depends on:** [2.4 — Lean Model Foundation](../phase-2/04-lean-model-foundation.md) (Lean workspace, core type-calculus model, type-soundness proof, and real CI proof jobs must exist before this stage deepens them); proof-*backed* claims require a non-empty, non-provisional published boundary in `proofs/BOUNDARY.md`, which this stage delivers

## Goal

Advance from the **provisional Lean model** established in Stage 2.4 to a full **proof-backed**
state: complete the memory-safety and lowering-correctness proofs, replace all `sorry`
placeholders in the type-soundness theorems, publish a non-provisional, non-empty proof boundary
in `proofs/BOUNDARY.md`, and enable **proof-backed** release/support claims.

## Workable Milestone

- Every `sorry` placeholder from Stage 2.4's type-soundness proofs is replaced by a complete
  mechanised proof.
- Memory-safety (no-dangling-reference) and HIR → LIR lowering-correctness proofs are
  complete for the bounded core calculus.
- `proofs/BOUNDARY.md` is updated from provisional to non-provisional, naming the concrete
  modelled subsystems with a full theorem inventory.
- CI proof jobs continue to run and block on failure; the boundary is now non-empty.
- Release notes and documentation may cite formal verification for the published boundary.

Current progress note:
- the published boundary already includes the live-reference ownership/allocation projection theorem (`KaliCore.Safety.liveRefsAreOwnedAndAllocated`) alongside the release-only helper corollaries `KaliCore.Safety.releaseRefLiveRefsAreOwnedAndAllocated`, `releaseRefLiveRefsAreLiveAnnotated`, `releaseAndDecrementLiveRefsAreLiveAnnotated`, `releaseAndCollectLiveRefsAreLiveAnnotated`, `KaliCore.Safety.releaseRefLiveRefsFiltered`, and `KaliCore.Safety.releasePreservesWellFormed`, plus the exact live-reference filtering theorems for the decrement and collection helpers (`KaliCore.Safety.releaseAndDecrementLiveRefsFiltered` and `KaliCore.Safety.releaseAndCollectLiveRefsFiltered`), the base `KaliCore.Safety.noDanglingReference` theorem and the helper-level no-dangling-reference corollaries (`KaliCore.Safety.releaseRefNoDanglingReference`, `KaliCore.Safety.releaseAndDecrementNoDanglingReference`, and `KaliCore.Safety.releaseAndCollectNoDanglingReference`), `releaseRecorded`, `releasedNotLive`, and `releasedNotLiveRef`, and now also the pure release-helper corollaries — including the plain origin theorem `KaliCore.Safety.releaseRefHeapCellOrigin` — plus the closed-substitution helper `KaliCore.Soundness.subst_closed` and the literal-to-type helper `KaliCore.litTy`, and the full local RC helper slice: `KaliCore.Safety.releaseAndDecrementLiveRefsAreOwnedAndAllocated`, `KaliCore.Safety.releaseAndDecrementPreservesWellFormed`, `KaliCore.Safety.releaseAndDecrementPreservesOwnership`, `KaliCore.Safety.releaseAndDecrementTargetCellOrigin` and the target-cell origin/positive-count theorem `KaliCore.Safety.releaseAndDecrementTargetCellOriginAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseAndDecrementTargetCellOriginAndPositiveCountAndLinearMemory`, `KaliCore.Safety.releaseAndDecrementTargetCellAllocatedWhenPositiveCount`, `KaliCore.Safety.releaseAndDecrementTargetCellOwnedAndAllocatedWhenPositiveCount`, `KaliCore.Safety.releaseAndDecrementRecorded`, `KaliCore.Safety.releaseAndDecrementDecrementsTargetCell`, `KaliCore.Safety.releaseAndDecrementKeepsTargetCellWhenPositiveCount`, `KaliCore.Safety.releaseAndDecrementTargetCellPositiveCountIff`, `KaliCore.Safety.releaseAndDecrementHeapCellOrigin`, `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndOwnership`, `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndPositiveCount`, the release-and-decrement origin/ownership/positivity theorem `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCount`, plus its linear-memory companion `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCountAndLinearMemory`, `KaliCore.Safety.releaseAndDecrementKeepsOtherPositiveCountCells, releaseAndDecrementKeepsOriginalPositiveCountCells`, `KaliCore.Safety.releaseAndDecrementZeroesLastTargetCell`, `KaliCore.Safety.releaseAndDecrementKeepsOtherHeapEntries`, `KaliCore.Safety.releaseAndDecrementPreservesOtherLiveRefs`, `KaliCore.Safety.releaseAndDecrementReleasedNotLiveRef`, `KaliCore.Safety.releaseRefPreservesOwnership`, `KaliCore.Safety.releaseRefPreservesReleasedRefs`, `KaliCore.Safety.releaseRefReleasedRefsCons`, `KaliCore.Safety.releaseRefHeapCharacterisation`, `KaliCore.Safety.releaseRefHeapCharacterisationAndLinearMemory`, `KaliCore.Safety.releaseRefHeapCellOrigin`, `KaliCore.Safety.releaseRefHeapCellOriginAndOwnership`, `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCountAndLinearMemory`, `KaliCore.Safety.releaseAndDecrementReleasedRefsCons`, `KaliCore.Safety.releaseAndDecrementPreservesReleasedRefs`, `KaliCore.Safety.releaseAndCollectLiveRefsAreOwnedAndAllocated`, `KaliCore.Safety.releaseAndCollectPreservesWellFormed`, `KaliCore.Safety.releaseAndCollectPreservesOwnership`, `KaliCore.Safety.releaseAndCollectRecorded`, `KaliCore.Safety.releaseAndCollectDropsZeroCountCells`, `KaliCore.Safety.releaseAndCollectRemovesZeroCountCells`, `KaliCore.Safety.releaseAndCollectKeepsPositiveCountCells`, `KaliCore.Safety.releaseAndCollectKeepsTargetCellWhenPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellPresentIffPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellAllocatedWhenPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellOrigin`, `KaliCore.Safety.releaseAndCollectTargetCellOriginAndPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellOriginOwnershipAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseAndCollectTargetCellOriginOwnershipAndPositiveCountAndLinearMemory`, `KaliCore.Safety.releaseAndCollectTargetCellOwnedAndAllocatedWhenPositiveCount`, `KaliCore.Safety.releaseAndCollectKeepsOtherPositiveCountCells`, `KaliCore.Safety.releaseAndCollectKeepsOriginalPositiveCountCells`, `KaliCore.Safety.releaseAndCollectDropsOriginalZeroCountCells`, `KaliCore.Safety.releaseAndCollectHeapIsPositiveCountFilter`, `KaliCore.Safety.releaseAndCollectHeapCellOrigin`, `KaliCore.Safety.releaseAndCollectHeapCellOriginAndOwnership`, `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCount`, plus its linear-memory companion `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCountAndLinearMemory`, `KaliCore.Safety.releaseAndCollectHeapCellOriginAndPositiveCount`, `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount`, `KaliCore.Safety.releaseAndCollectKeepsOtherHeapEntries`, `KaliCore.Safety.releaseAndCollectPreservesOtherLiveRefs`, `KaliCore.Safety.releaseAndCollectReleasedNotLiveRef`, `KaliCore.Safety.releaseAndDecrementHeapCharacterisation`, `KaliCore.Safety.releaseAndDecrementHeapCharacterisationAndLinearMemory`, `KaliCore.Safety.releaseAndCollectHeapCharacterisation`, `KaliCore.Safety.releaseAndCollectHeapCharacterisationAndLinearMemory`, `KaliCore.Safety.releaseAndCollectReleasedRefsCons`, `KaliCore.Safety.releaseRefReleasedNotLiveRef`, `KaliCore.Safety.releaseAndCollectPreservesReleasedRefs`. That keeps the remaining Stage 4.2 memory work explicitly focused on the broader ownership / RC target rather than the earlier snapshot-ownership gap, and the proof-summary anti-drift guard now pins the exact live-reference filtering theorem names too. The canonical short summary is **Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target.** The proof-boundary inventory is also guarded by a schema-docs anti-drift test that compares the manifest's covered-path list to the actual proof source set. The linear-memory payload story now also includes the combined wellformedness/linear-memory corollaries `KaliCore.Safety.releaseRefPreservesWellFormedAndLinearMemory`, `KaliCore.Safety.releaseAndDecrementPreservesWellFormedAndLinearMemory`, and `KaliCore.Safety.releaseAndCollectPreservesWellFormedAndLinearMemory`, plus the combined wellformedness/ownership/linear-memory corollaries `KaliCore.Safety.releaseRefPreservesWellFormedAndOwnershipAndLinearMemory`, `KaliCore.Safety.releaseAndDecrementPreservesWellFormedAndOwnershipAndLinearMemory`, and `KaliCore.Safety.releaseAndCollectPreservesWellFormedAndOwnershipAndLinearMemory`. The proof slice also now has exact heap-characterisation theorems for the release-and-decrement and release-and-collect helpers, so the RC membership story is stated directly in addition to the existing origin/filter corollaries, including `KaliCore.Safety.releaseAndCollectHeapIsPositiveCountFilter`, and the HIR lowering story is explicit via `KaliIR.Value`, `KaliIR.LoweringCorrectness.lower_preserves_value`, `KaliIR.LoweringCorrectness.lower_preserves_step`, and `KaliIR.LoweringCorrectness.lower_preserves_steps`. The RC snapshot model itself is phrased through the explicit `hasOwnership` / `allocated` / `liveAnnotated` predicate vocabulary, and the schema-docs anti-drift guard now pins those names as well, and now also pins the live-reference ownership/allocation projection theorem, the ownership-preservation corollaries, the surviving-live-reference corollary on the collection path, the released-not-live theorems, and the decrement-path positive-count guard, so the tracker keeps the model shape visible alongside the theorem inventory. The companion linear-memory theorem `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` is now spelled out directly in the progress note so the provenance slice stays explicit rather than relying on shortened companion wording. The local collection helper now also carries the matching origin/positive-count + linear-memory companion `KaliCore.Safety.releaseAndCollectHeapCellOriginAndPositiveCountAndLinearMemory`, and the collection target-cell iff bridge `KaliCore.Safety.releaseAndCollectTargetCellPresentIffPositiveCount` plus the target-cell allocation corollary `KaliCore.Safety.releaseAndCollectTargetCellAllocatedWhenPositiveCount` are now called out explicitly too, while the final-heap positivity theorem `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount` stays named explicitly alongside them. The same note now also names `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount` explicitly, and the README/spec verification summaries mirror that wording, so the local collection helper's final-heap positivity story stays direct; the schema-docs anti-drift guard now pins that theorem name too. The release-only, decrement, and collection heap-characterisation theorems now also have explicit linear-memory companion forms `KaliCore.Safety.releaseRefHeapCharacterisationAndLinearMemory`, `KaliCore.Safety.releaseAndDecrementHeapCharacterisationAndLinearMemory`, and `KaliCore.Safety.releaseAndCollectHeapCharacterisationAndLinearMemory`.

## Current Repository State

Stage 4.2's implementation milestone has landed: the repository is already **proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target.**

The remaining work is widening work, not baseline enablement:

- broaden the ownership / RC proof story beyond the current snapshot helper slice,
- broaden lowering correctness beyond the current HIR subset,
- keep every proof-status summary and anti-drift test synchronized with `proofs/BOUNDARY.md`.

## Historical Stage Tasks

### 1. Complete the type-soundness proofs

Stage 2.4 established the Lean 4 workspace and the core type-calculus model. Stage 4.2 then
closed the remaining proof obligations and made the sorry-free state part of the published
proof-backed boundary.

### 3. Memory safety properties

Model the ownership / reference-counting memory model from Phase 2:

- Define the memory model (linear memory + RC heap).
- Prove that well-typed programs with correct ownership annotations never produce dangling
  references (no use-after-free).
- Track the current proof-backed slice as it widens: the repository now also proves that
  well-formed snapshots keep live references anchored in ownership and allocation, that
  releasing a live reference preserves the remaining well-formed live set, that
  released references remain outside the live-reference set, and that the local zero-count
  collection helper removes the freed decrement target and drops original zero-count cells from the
  final heap.
- Prove that the RC decrement path correctly frees all reachable objects (no leaks within the
  modelled subset).

### 4. HIR → LIR lowering correctness (within the modelled subset)

Prove that the HIR → LIR lowering preserves the semantics of the core calculus:

- For each HIR term in the modelled subset, the emitted LIR evaluates to the same value under
  the LIR operational semantics.

This is limited to the modelled subset (no `eval`, no dynamic dispatch beyond what the model
covers), and the current HIR slice now also covers bare throw.

### 5. Update `proofs/BOUNDARY.md`

Replace the provisional proof-boundary manifest with a concrete proof-backed one:

```markdown
# Proof Boundary

## Current status: proof-backed

## Modelled subsystems

### Core type system (KaliCore/Types.lean, KaliCore/Soundness.lean)
- Type soundness: progress + preservation for the core Kali calculus
- Excludes: eval, dynamic import, browser/OS host interactions

### Memory safety (KaliCore/Safety.lean)
- No dangling references in well-typed programs with correct ownership annotations
- No leaks within the modelled ownership subset
- Excludes: cross-FFI pointers, native addons

### HIR → LIR lowering (KaliIR/LoweringCorrectness.lean)
- Semantic preservation for the core calculus subset
- Excludes: eval, dynamic dispatch beyond the model

## What is NOT claimed
- Proof coverage of the full surface language
- Proof coverage of the WASM host runtime (wasmtime)
- Proof coverage of browser/OS host API interactions
- Proof coverage of eval / dynamic features
```

### 6. CI proof jobs

Update the CI pipeline to run Lean proof jobs on every commit touching `proofs/`:

```yaml
proof-check:
  if: paths changed under proofs/
  runs-on: ubuntu-latest
  steps:
    - uses: leanprover/lean4-action@v1
    - run: cd proofs && lake build
```

A failing proof job blocks merges, ensuring the published boundary stays honest.

### 7. Update release claims

Update `README.md`, `specs/19-feature-maturity.md`, and any affected summaries to replace the
proof-ready canonical summary with the proof-backed boundary statement, quoting the updated
`proofs/BOUNDARY.md` verbatim for the claimed subsystems.

### 8. Tests

- CI proof job: `lake build` in `proofs/` succeeds on every commit.
- Anti-drift test: assert that `proofs/BOUNDARY.md` content matches the actual set of `*.lean`
  files in the repository (CI fails if a proof file is deleted without updating the boundary).
- Regression: adding a new proof file without updating `proofs/BOUNDARY.md` triggers a CI
  warning (not a block; the update is required but may follow).

## Remaining Work

Even after the proof-backed milestone, the following work remains before the later, wider Stage-4.2 target can be claimed:

- widen the proof-backed boundary beyond the current published RC snapshot helper slice,
- widen the lowering-correctness model beyond the current HIR subset,
- avoid letting roadmap language imply broader proof coverage than `proofs/BOUNDARY.md` names.

## Out of Scope

- Proof coverage of the full Kali surface language (aspirational long-term goal).
- Proof coverage of wasmtime internals (wasmtime has its own verification program).
- Proof automation for dynamically-typed code paths (excluded from the modelled subset).

## Definition of Done

- [x] `proofs/BOUNDARY.md` is non-empty and names the modelled subsystems.
- [x] Lean proofs compile and pass: type soundness, memory safety, lowering correctness.
- [x] CI proof job runs and blocks on proof failures.
- [x] README and maturity matrix updated with proof-backed claims for the published boundary.
- [x] All Phase-1 through Phase-4.1 tests continue to pass.

## Linear-memory payload preservation

The current proof-backed RC snapshot model now carries an explicit linear-memory payload alongside the ownership / heap / live-reference state, and the release-only, decrement, and collection helpers preserve it via `KaliCore.Safety.releaseRefPreservesLinearMemory`, `KaliCore.Safety.releaseAndDecrementPreservesLinearMemory`, and `KaliCore.Safety.releaseAndCollectPreservesLinearMemory`. The same helpers now also package their ownership and linear-memory invariance together in the combined corollaries `KaliCore.Safety.releaseRefPreservesOwnershipAndLinearMemory`, `KaliCore.Safety.releaseAndDecrementPreservesOwnershipAndLinearMemory`, and `KaliCore.Safety.releaseAndCollectPreservesOwnershipAndLinearMemory`, while the combined wellformedness/ownership/linear-memory corollaries `KaliCore.Safety.releaseRefPreservesWellFormedAndOwnershipAndLinearMemory`, `KaliCore.Safety.releaseAndDecrementPreservesWellFormedAndOwnershipAndLinearMemory`, and `KaliCore.Safety.releaseAndCollectPreservesWellFormedAndOwnershipAndLinearMemory` now keep the explicit wellformedness, ownership, and payload story together, plus the combined wellformedness/ownership corollaries `KaliCore.Safety.releaseRefPreservesWellFormedAndOwnership`, `KaliCore.Safety.releaseAndDecrementPreservesWellFormedAndOwnership`, and `KaliCore.Safety.releaseAndCollectPreservesWellFormedAndOwnership`, while the local collection helper now also has the heap-filter-and-linear-memory corollary `KaliCore.Safety.releaseAndCollectHeapIsPositiveCountFilterAndLinearMemory`.
