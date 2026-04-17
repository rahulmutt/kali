# Proof Boundary Manifest

Status: **proof-backed proof-boundary manifest**.

This file is the canonical repository location for Kali's published **proof-boundary manifest**. The repository now contains a checked-in Lean 4 proof tree under `proofs/`, and the published boundary is mechanized for the widened closed fragment — now including assignment and try/catch in addition to literals, variables, closed functions, application, sequencing, and conditionals — plus a small ownership / RC snapshot safety slice with live-reference ownership/allocation projection, release-update preservation, explicit release-recording, target-cell decrement bookkeeping, last-ref zeroing on the decrement path, zero-count collection on the decrement path, positive-count preservation on the local collection helper, the helper-level theorem that positive-count cells from the original heap survive when they are not the released target, unrelated-heap preservation, other-live-reference preservation on the local `releaseAndCollect` helper, helper-level ownership/allocation preservation corollaries on the decrement and collection paths, and live/released-disjointness theorems on the refcount-decrement path, including the local `releaseAndCollect` disjointness theorem, and a widened HIR lowering-correctness slice, including the current single-step and finite-trace lowering bridge, described below. The repository is therefore **proof-backed for the published boundary**, while remaining intentionally narrower than the later Stage 4.2 ownership/memory-safety and lowering-correctness target.

Current repository-state note:
- follow the shared **current-repository-state vs target-contract reading** from [SPEC.md](../SPEC.md): the Lean project tree now exists under `proofs/` and is built from `proofs/lakefile.lean`
- the proof sources are organized around `proofs/KaliCore.lean` and `proofs/KaliIR.lean`, which import the provisional model files listed below
- the current proof claims now cover the widened closed fragment (literals, variables, closed functions, application, sequencing, conditionals, assignment, and try/catch) and the proof file compiles without `sorry` placeholders
- the ownership slice now includes the live-reference ownership/allocation projection, live-to-released transition preservation, explicit release-recording, zero-count collection on the decrement path, positive-count preservation on the local collection helper, the helper-level theorem that positive-count cells from the original heap survive when they are not the released target, unrelated-heap preservation, other-live-reference preservation on the local `releaseAndCollect` helper, helper-level ownership/allocation preservation corollaries on the decrement and collection paths, and live/released-disjointness theorems on the refcount-decrement helper and the local `releaseAndCollect` helper, including the local collection helper's release-recording theorem, plus the no-dangling / release-liveness claims
- the lowering-correctness slice now includes both the single-step bridge and a finite HIR-trace preservation bridge for the current modeled subset

Canonical verification state (following the shared **proof-ready vs proof-backed split** from [SPEC.md](../SPEC.md)):

| Item | Current state |
|---|---|
| proof-ready | **yes** — this manifest exists, truthfully declares the current claim boundary, and publishes the repository's current proof-CI trigger policy |
| proof-backed | **yes** — the published boundary names mechanized claims for the widened closed fragment and the repository may cite proof coverage for that boundary |
| repository claim | **proof-backed for the published boundary** |
| canonical short summary | **Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target.** |

Release rule:
- this manifest is acceptable because it keeps the repository honest about the currently modeled slice while still publishing mechanized theorem/property claims for that slice
- before any release markets the later Stage 4.2 ownership/memory-safety or lowering-correctness target as shipped evidence, this manifest must widen to name those additional theorem/property claims explicitly
- the Lean tree is proof-backed for the published boundary, but it is still a modeling aid for the later widening work rather than evidence that the whole repository is already covered

## Modelled boundary

### Core type calculus (`proofs/KaliCore/Types.lean`, `proofs/KaliCore/Semantics.lean`, `proofs/KaliCore/Soundness.lean`)
- Type syntax: `Ty`, `LitVal`, and the provisional function/object/union/intersection forms
- Expression syntax: literals, variables, annotated functions, application, sequencing, conditionals, assignment, throw, and try/catch
- Runtime model: value predicate and small-step reduction for the bounded typed fragment; the current proof boundary models the closed literals / variables / closed-functions slice plus the application, sequencing, conditional, assignment, and try/catch subfragment that is now mechanised in `KaliCore.Soundness`
- Claimed theorem inventory: progress and preservation for the widened closed typed core fragment
- Current proof state: theorem statements are present and the core proof file now compiles, but the mechanised scope still stops short of bare throw plus the full Stage 4.2 memory/lowering target

### Ownership model (`proofs/KaliCore/Safety.lean`)
- Ownership classes: `stack`, `ownedHeap`, `sharedHeap`, `borrowed`
- Model shape: `RcCell` heap entries, `RcSnapshot` ownership/heap/live-reference state, released-reference tracking, and a local refcount-decrement update helper on the released cell
- Claimed property inventory: no dangling references for well-formed RC snapshots; live references remain owned and allocated; releasing a live reference preserves the remaining well-formed live set; releasing the last counted reference zeroes the target cell on the decrement path; the local zero-count collection helper records the release, preserves positive-count cells from the decrement pass, preserves positive-count cells from the original heap when they are not the released target, and removes the freed cell; released references are not live and stay disjoint from the live-reference set, including after the local `releaseAndCollect` helper runs
- Current proof state: the `noDanglingReference`, `liveRefsAreOwnedAndAllocated`, `releasePreservesWellFormed`, `releaseAndDecrementPreservesWellFormed`, `releaseAndDecrementLiveRefsAreOwnedAndAllocated`, `releaseAndDecrementRecorded`, `releaseAndDecrementDecrementsTargetCell`, `releaseAndDecrementZeroesLastTargetCell`, `releaseAndCollectRecorded`, `releaseAndCollectRemovesZeroCountCells`, `releaseAndCollectKeepsPositiveCountCells`, `releaseAndCollectKeepsOtherPositiveCountCells`, `releaseAndCollectPreservesWellFormed`, `releaseAndCollectLiveRefsAreOwnedAndAllocated`, `releaseAndCollectReleasedNotLiveRef`, `releaseAndDecrementKeepsOtherHeapEntries`, `releaseAndDecrementPreservesOtherLiveRefs`, `releaseAndDecrementReleasedNotLiveRef`, `releaseRecorded`, `releasedNotLive`, and `releasedNotLiveRef` theorems are mechanised for the current RC snapshot model, but the model remains narrower than the eventual Stage 4.2 ownership / RC target

### HIR lowering model (`proofs/KaliIR/HIRModel.lean`, `proofs/KaliIR/LoweringCorrectness.lean`)
- Provisional HIR syntax and a core lowering projection for future lowering-correctness work
- Claimed property inventory: structural lowering equations (`lower_core`, `lower_let1`, `lower_seq`, `lower_if`, `lower_assign`, `lower_tr`) plus a small-step lowering-preservation bridge and a finite-trace lowering-preservation bridge for the current HIR subset
- Current proof state: the structural equations are mechanised, `KaliIR.LoweringCorrectness.lower_preserves_step` now proves the current HIR step relation is preserved by lowering for the modeled subset, and `KaliIR.LoweringCorrectness.lower_preserves_steps` lifts that result to finite traces for the same subset, including assignment and try/catch; the subset still stops short of the full Stage 4.2 semantic-preservation target

## Claimed theorems/properties
- `KaliCore.Soundness.progress` — progress for the widened closed typed core fragment
- `KaliCore.Soundness.preservation` — preservation for the widened closed typed core fragment
- `KaliCore.Safety.noDanglingReference` — mechanised no-dangling-reference theorem for the current RC snapshot model
- `KaliCore.Safety.liveRefsAreOwnedAndAllocated` — mechanised theorem that well-formed snapshots keep live references anchored in ownership and allocation
- `KaliCore.Safety.releasePreservesWellFormed` — mechanised theorem that releasing a live reference preserves the remaining well-formed live set
- `KaliCore.Safety.releaseAndDecrementPreservesWellFormed` — mechanised theorem that the current release-and-decrement helper preserves the remaining well-formed live set
- `KaliCore.Safety.releaseAndDecrementLiveRefsAreOwnedAndAllocated` — mechanised theorem that the release-and-decrement helper keeps surviving live references anchored in ownership and allocation
- `KaliCore.Safety.releaseAndDecrementRecorded` — mechanised theorem that the release-and-decrement helper records the released reference in the released set
- `KaliCore.Safety.releaseAndDecrementDecrementsTargetCell` — mechanised theorem that the release-and-decrement helper decrements the targeted heap cell when it is present
- `KaliCore.Safety.releaseAndDecrementZeroesLastTargetCell` — mechanised theorem that the release-and-decrement helper zeroes the targeted heap cell when the released reference was the last live count
- `KaliCore.Safety.releaseAndDecrementKeepsOtherHeapEntries` — mechanised theorem that the release-and-decrement helper leaves unrelated heap entries untouched
- `KaliCore.Safety.releaseAndDecrementPreservesOtherLiveRefs` — mechanised theorem that non-target live references remain live after the release-and-decrement helper runs
- `KaliCore.Safety.releaseAndDecrementReleasedNotLiveRef` — mechanised theorem that released references stay disjoint from the live set after the release-and-decrement helper runs
- `KaliCore.Safety.releaseAndCollectRecorded` — mechanised theorem that the local release-and-collect helper records the released reference in the released set
- `KaliCore.Safety.releaseAndCollectKeepsPositiveCountCells` — mechanised theorem that the local release-and-collect helper preserves positive-count cells from the decrement pass
- `KaliCore.Safety.releaseAndCollectKeepsOtherPositiveCountCells` — mechanised theorem that positive-count cells from the original heap survive when they are not the released target
- `KaliCore.Safety.releaseAndCollectPreservesOtherLiveRefs` — mechanised theorem that other live references remain live after the local release-and-collect helper runs
- `KaliCore.Safety.releaseAndCollectLiveRefsAreOwnedAndAllocated` — mechanised theorem that the local release-and-collect helper keeps surviving live references anchored in ownership and allocation
- `KaliCore.Safety.releaseAndCollectReleasedNotLiveRef` — mechanised theorem that released references stay disjoint from the live set after the local release-and-collect helper runs
- `KaliCore.Safety.releaseRecorded` — mechanised theorem that a released reference is recorded in the released set after the release step
- `KaliCore.Safety.releasedNotLive` — mechanised theorem that released references are not live in the current RC snapshot model
- `KaliCore.Safety.releasedNotLiveRef` — mechanised theorem that well-formed snapshots keep released and live references disjoint
- `KaliIR.HIRModel.lower_core`, `lower_let1`, `lower_seq`, `lower_if`, `lower_assign`, `lower_tr` — structural lowering equations for the provisional HIR projection
- `KaliIR.LoweringCorrectness.lower_preserves_step` — lowering-preservation bridge for the current HIR step subset
- `KaliIR.LoweringCorrectness.lower_preserves_steps` — finite-trace lowering-preservation bridge for the same modeled HIR subset

## Trusted assumptions
- The proof tree is a proof-backed modeling aid for the published closed-fragment boundary.
- The current closed-fragment proof boundary is intentionally narrower than the eventual Stage 4.2 ownership/memory-safety and lowering-correctness target and must be widened before any claim about that later target.
- The ownership slice currently models a small RC snapshot with live-reference and release tracking, plus live/released disjointness, release-update preservation, unrelated-heap preservation, last-ref zeroing on the decrement path, a local zero-count collection helper, positive-count preservation on that helper, the new original-heap positive-count preservation lemma, other-live-reference preservation on that helper, and a local refcount-decrement update helper for well-formed snapshots; it is still narrower than the eventual full ownership / reference-counting story, even though `releaseAndCollect` now has its own explicit release-recording, positive-count preservation, other-live preservation, and disjointness theorems.
- The lowering-correctness bridge is intentionally limited to the current HIR subset, not the later full HIR → LIR semantic-preservation target; that subset now includes assignment and try/catch alongside the existing `let1` / sequencing / conditional bridge.
- The currently mechanised fragment now includes application, sequencing, and conditionals in addition to the original closed-literal/variable/closed-function slice.
- No mechanized proof coverage is claimed for Rust implementation code outside `proofs/`.

## Explicitly unmodeled features
- Full ECMAScript/TypeScript surface semantics
- Host integrations and OS behavior
- Dynamic compatibility features such as `eval` / `Function()`
- Browser/Node compatibility layers beyond the future modeled core
- Full lowering/codegen correctness end to end

## Covered implementation/spec paths
- `proofs/lakefile.lean`
- `proofs/KaliCore.lean`
- `proofs/KaliIR.lean`
- `proofs/KaliCore/Types.lean`
- `proofs/KaliCore/Semantics.lean`
- `proofs/KaliCore/Soundness.lean`
- `proofs/KaliCore/Safety.lean`
- `proofs/KaliIR/HIRModel.lean`
- `proofs/KaliIR/LoweringCorrectness.lean`

## Required implementation/spec alignment scope
- `proofs/lakefile.lean` must continue to declare the Lean roots that keep the provisional proof tree complete
- `proofs/KaliCore.lean` and `proofs/KaliIR.lean` serve as the root import surfaces for the provisional Lean model
- any change to the named proof files above should keep the boundary text and trigger policy in sync with the current modeled slice

## Proof-CI trigger policy
- Trigger proof CI when `proofs/**` changes.
- Because the published boundary is still provisional and only names Lean model files under `proofs/`, no broader Rust/spec trigger set is currently claimed.
- If the boundary later names covered implementation/spec subsystems outside `proofs/`, proof CI must also trigger for changes to those covered areas.

## Boundary-maintenance rule
- once the published boundary becomes non-provisional, a change to any covered path must land with matching proof updates or an explicit narrowing of the boundary first
- widening the boundary also requires updating the named theorem/property inventory; new Lean files alone do not widen the claim surface
- release/support wording must always follow this file's current boundary immediately after such a change
