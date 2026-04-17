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
- the published boundary already includes the live-reference ownership/allocation projection theorem (`KaliCore.Safety.liveRefsAreOwnedAndAllocated`) alongside `noDanglingReference`, `releasePreservesWellFormed`, `releaseRecorded`, `releasedNotLive`, and `releasedNotLiveRef`, and now also the pure release-helper corollaries plus the full local RC helper slice: `KaliCore.Safety.releaseAndDecrementLiveRefsAreOwnedAndAllocated`, `KaliCore.Safety.releaseAndDecrementPreservesOwnership`, `KaliCore.Safety.releaseAndDecrementRecorded`, `KaliCore.Safety.releaseAndDecrementDecrementsTargetCell`, `KaliCore.Safety.releaseAndDecrementHeapCellOrigin`, `KaliCore.Safety.releaseAndDecrementZeroesLastTargetCell`, `KaliCore.Safety.releaseAndDecrementKeepsOtherHeapEntries`, `KaliCore.Safety.releaseAndDecrementPreservesOtherLiveRefs`, `KaliCore.Safety.releaseAndDecrementReleasedNotLiveRef`, `KaliCore.Safety.releaseRefPreservesOwnership`, `KaliCore.Safety.releaseRefPreservesReleasedRefs`, `KaliCore.Safety.releaseAndCollectLiveRefsAreOwnedAndAllocated`, `KaliCore.Safety.releaseAndCollectPreservesOwnership`, `KaliCore.Safety.releaseAndCollectRecorded`, `KaliCore.Safety.releaseAndCollectDropsZeroCountCells`, `KaliCore.Safety.releaseAndCollectRemovesZeroCountCells`, `KaliCore.Safety.releaseAndCollectKeepsPositiveCountCells`, `KaliCore.Safety.releaseAndCollectKeepsOtherPositiveCountCells`, `KaliCore.Safety.releaseAndCollectDropsOriginalZeroCountCells`, `KaliCore.Safety.releaseAndCollectHeapIsPositiveCountFilter`, `KaliCore.Safety.releaseAndCollectHeapCellOrigin`, `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount`, `KaliCore.Safety.releaseAndCollectPreservesOtherLiveRefs`, `KaliCore.Safety.releaseAndCollectReleasedNotLiveRef`, `KaliCore.Safety.releaseRefReleasedNotLiveRef`, `KaliCore.Safety.releaseAndDecrementPreservesReleasedRefs`, and `KaliCore.Safety.releaseAndCollectPreservesReleasedRefs`. That keeps the remaining Stage 4.2 memory work explicitly focused on the broader ownership / RC target rather than the earlier snapshot-ownership gap. The proof-boundary inventory is also guarded by a schema-docs anti-drift test that compares the manifest's covered-path list to the actual proof source set.

## Tasks

### 1. Complete the type-soundness proofs

Stage 2.4 establishes the Lean 4 workspace, core type-calculus model, and initial progress +
preservation proofs, but may leave `sorry` placeholders in complex proof branches. This task
replaces every `sorry` in `KaliCore/Soundness.lean` with a complete mechanised proof:

- Close all remaining cases in the **progress** theorem.
- Close all remaining cases in the **preservation** theorem.
- Ensure no `sorry` remains in the type-soundness files; the sorry-free gate that was a CI
  warning in Stage 2.4 becomes a CI block in this stage.

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

## Out of Scope

- Proof coverage of the full Kali surface language (aspirational long-term goal).
- Proof coverage of wasmtime internals (wasmtime has its own verification program).
- Proof automation for dynamically-typed code paths (excluded from the modelled subset).

## Definition of Done

- [ ] `proofs/BOUNDARY.md` is non-empty and names the modelled subsystems.
- [ ] Lean proofs compile and pass: type soundness, memory safety, lowering correctness.
- [ ] CI proof job runs and blocks on proof failures.
- [ ] README and maturity matrix updated with proof-backed claims for the published boundary.
- [ ] All Phase-1 through Phase-4.1 tests continue to pass.
