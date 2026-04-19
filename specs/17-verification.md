# 17 — Formal Verification

## Overview

Kali uses Lean 4 to formally verify selected high-value invariants of the implementation over time. Verification is iterative — proofs are developed alongside the Rust implementation and updated as the spec evolves, and support claims stay limited to the currently published **proof-boundary manifest** rather than to the whole implementation.

Bootstrap-alignment note:
- the bootstrap brief's "formally verify implementation details with Lean while iterating on the Spec" requirement is normalized here as an iterative verification program with an honest published boundary from the start
- that means Phase 1 must be **proof-ready**, but it does **not** automatically imply blanket proof coverage or permission to market Kali as already **proof-backed** before the manifest names at least one concrete modeled subsystem and theorem set

## Scope

Focus verification on the highest-value areas where bugs have the most impact.

Important simplification rule: Lean proofs target a **core Kali calculus**, not the full surface language all at once. Early proof work should model the statically analyzable subset that excludes late-compatibility features such as `eval` / `Function()`, dynamic module loading, weak/finalization semantics, and browser/OS host details. Those outer features are handled by explicit phase gates in the implementation and only enter the proof story once their semantics stabilize.

## Proof-ready vs proof-backed

To keep the bootstrap's Lean requirement aligned with the rest of the phased spec, Kali distinguishes two verification states that were easy to blur together in earlier wording:

| Verification state | Minimum repository requirement | What docs/releases may claim |
|---|---|---|
| **proof-ready** | published `proofs/BOUNDARY.md`, honest proof-CI trigger policy, explicit no-overclaim discipline, and a truthful boundary description that may still be the shared empty placeholder manifest or a later provisional non-empty model | the repository is prepared for phased verification work, but does **not** claim shipped mechanized coverage yet |
| **proof-backed** | the manifest is non-empty, names at least one concrete modeled subsystem plus theorem/property inventory, and the named claims are mechanized rather than merely staged | release/support wording may cite formal verification, but only for the published boundary |

Practical rule:
- Phase 1 should be **proof-ready** from the start
- the Phase-1 contract is therefore repository/process hygiene first: published boundary, an honest proof-CI trigger policy, and explicit no-overclaim discipline
- **proof-backed** is not itself a blanket Phase-1 requirement; it becomes required only for release/support wording that wants to market formal verification as shipped evidence rather than as future-facing process readiness

Current repository status rule:
- `proofs/BOUNDARY.md` is the single source of truth for the repository's current verification state
- chapter summaries, release notes, and README copy should cite or quote that manifest rather than restating current proof coverage from memory or from this chapter's roadmap prose
- if the manifest is still the shared **placeholder proof-boundary manifest**, the honest repository claim remains **proof-ready** rather than **proof-backed**
- if the manifest is a mechanized non-empty proof boundary, the honest repository claim is **proof-backed for the published boundary** while remaining narrower than any later target it does not name
- repository summaries should reuse the canonical short summary from `proofs/BOUNDARY.md` verbatim: **Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target.**
- [19 — Feature Maturity](19-feature-maturity.md) makes the same guardrail explicit: proof-backed release/support claims while the published boundary is still empty are **Rejected by default**

Copy-paste wording shortcut:
- when a summary needs one sentence about current verification status, use the manifest's canonical short summary verbatim rather than paraphrasing it into a second near-duplicate status line

Verification maintenance packet:
- treat verification edits as one small packet rather than scattered wording cleanup: update `proofs/BOUNDARY.md` first for current-state claims, then sync this chapter for roadmap/discipline wording, then sync any affected summary/evidence owners (`README.md`, `specs/16-testing.md`, and `specs/19-feature-maturity.md` when a maturity row or release-claim boundary changes)
- practical shortcut: if an edit would change what the repository may honestly claim **today**, it is almost never a one-file change inside `specs/17-verification.md` alone

Claim-reading shortcut:
- to answer **what proof coverage is claimed today**, read `proofs/BOUNDARY.md` first
- to answer **what Phase 1 must guarantee**, read this chapter's **proof-ready vs proof-backed** split together with [19 — Feature Maturity](19-feature-maturity.md)
- to answer **what the first non-placeholder proof target should be**, read **First proof-backed milestone** below

### First proof-backed milestone

To keep the bootstrap's Lean requirement actionable instead of aspirational, the first non-placeholder proof boundary should be intentionally small and named up front.

Recommended first proof-backed milestone:
- one core typed/effectful calculus fragment with explicit syntax, typing, and small-step semantics;
- progress + preservation for that fragment;
- one conservative built-in effect-soundness theorem family over the sandbox-relevant capability subset;
- one declarative sandbox-policy decision/enforcement theorem family for the same modeled capability subset;
- one explicit covered-path inventory in `proofs/BOUNDARY.md` naming the Lean files plus the corresponding spec chapters / implementation areas those proofs are intended to constrain.

Consistency rule:
- `proofs/BOUNDARY.md` is the canonical published boundary, but the first non-placeholder scope should either mirror this milestone or explicitly point back here as the source of truth; the repository should not let the verification chapter and the published proof-boundary manifest drift into two different “first real proof target” stories.

Promotion rule:
- the repository becomes **proof-backed** only when that milestone (or another explicitly documented equivalent) is actually listed in `proofs/BOUNDARY.md` with named theorem/property claims
- until then, verification language in summaries should keep saying **proof-ready** rather than implying shipped mechanized coverage

### Covered-boundary edit discipline

To keep “verify while iterating on the spec” honest, Kali uses one explicit maintenance rule once the published boundary becomes non-empty:
- if a PR changes a spec chapter, Rust subsystem, or proof-facing invariant that is already named inside `proofs/BOUNDARY.md`'s **Covered implementation/spec paths**, the same PR should either update the matching Lean model/proofs or explicitly narrow the published boundary first
- widening the published boundary requires naming the new covered paths plus theorem/property inventory in `proofs/BOUNDARY.md`; widening must not be inferred from roadmap prose or from the presence of new Lean files alone
- shrinking the boundary is allowed as an honesty move during refactors, but release/support wording must immediately follow the narrower boundary rather than continuing to cite the old proof scope
- this rule applies equally to spec-first edits: a proof-backed claim is about the currently published model/spec correspondence, not just about passing implementation tests

Practical consequence:
- proof coverage may lag future roadmap intent, but it must not lag the published manifest for any area that is still claimed as covered

## Phase-aligned proof scope

| Phase | Verification focus |
|---|---|
| Phase 1 MVP | Reach and maintain the **proof-ready** state: published proof boundary, honest proof-CI trigger policy, and no proof-backed marketing beyond the manifest. Phase 1 does **not** require a non-empty proof boundary to ship, but any release/support wording that wants to present formal verification as shipped evidence must first become **proof-backed** with at least one concrete modeled theorem family over the core typed calculus or sandbox-policy core. |
| Phase 2 target | Built-in effect inference conservativity, ownership/escape/reference-counting model, and selected lowering-preservation lemmas |
| Phase 3 target | Specialization/layout-preservation lemmas for the proved fragment, plus stronger package/runtime-model correspondence where the host contract is already stable |
| Phase 4 compatibility | Late dynamic compatibility paths only after their semantics are frozen enough to model honestly; `eval`/dynamic loading remain outside the currently published proof boundary until then |

Rule:
- a phase can ship with partial proof coverage as long as support claims stay inside the documented proof boundary and the matching implementation/testing evidence still exists
- verification should deepen the same hard invariants the bootstrap cares about most: AOT-only execution, sandbox honesty, deterministic machine contracts, and memory safety without tracing GC

## Published Proof Boundary

Phase 1 should make the verification claim auditable through one published **proof-boundary manifest** at `proofs/BOUNDARY.md` (see [SPEC.md](../SPEC.md)) rather than through scattered prose.

That manifest should enumerate, at minimum:
- the modeled calculus/subsystem slice currently covered in Lean,
- the named theorems/properties currently claimed,
- trusted assumptions and explicitly unmodeled features,
- which implementation/spec subsystems are expected to remain aligned with the model,
- and the CI trigger rule for when the proof job must run.

Before the first proofs land, the manifest may truthfully stay in the shared **placeholder proof-boundary manifest** state from [SPEC.md](../SPEC.md). Once the Lean tree exists, the manifest may instead describe a **provisional non-empty proof boundary** that names concrete modeled subsystems and theorem/property inventory while still remaining proof-ready rather than proof-backed. In either case, the current repository state should always be read from `proofs/BOUNDARY.md`, not inferred from this chapter's examples or milestone plan.

Reading and claim rules:
- the manifest should state the current verification state explicitly using the same two-way split as this chapter: **proof-ready** vs **proof-backed**
- release notes, README summaries, maturity claims, and CI wiring should treat `proofs/BOUNDARY.md` as the single source of truth instead of paraphrasing current proof status in multiple places
- the **placeholder proof-boundary manifest** is acceptable during spec-first iteration and early implementation bootstrapping because it still preserves the **proof-ready** state
- it is **not** enough for a release to market Kali as already formally verified in Phase 1; any such release/support claim must first replace the placeholder state with at least one concrete modeled subsystem plus named theorem/property claims so the claim becomes genuinely **proof-backed**
- while the published proof boundary is still the placeholder manifest, proof CI is required only for changes under `proofs/`; once the manifest names covered implementation/spec subsystems, proof CI must also trigger for changes to those covered areas
- until concrete CI workflow files are actually present, that trigger rule is still the repository's normative proof-CI policy rather than evidence that hosted proof automation already exists
- broad phrases such as “formally verified” should be read as “verified for the currently published proof boundary in `proofs/BOUNDARY.md`”, not as blanket coverage of all language/runtime behavior

### Type System Soundness
Prove soundness for the **core typed fragment** first:
- **Progress**: well-typed core terms either are values, can step, or are blocked only at an explicitly modeled effect boundary / host boundary
- **Preservation**: evaluation preserves types in the modeled core semantics
- Model the core type language (primitives, unions, intersections, functions, objects, and the early capability-effect fragment as needed)
- Prove subtyping properties that are realistic for a structural system: reflexivity, transitivity, and coherence with a chosen type-equivalence relation
- Prove unification terminates for the HM-style inference fragment that Kali chooses to verify
- Prove principality only for the explicit HM-like fragment where principal types are expected to exist; do **not** overclaim principality for the entire TypeScript-compatible structural/subtyping surface
- Prove the supported constraint-solving fragment is decidable

### Effect System Correctness
- Prove effect inference is conservative for the built-in sandbox-relevant capability set (inferred effects ⊇ actual modeled effects)
- Prove the sandbox policy decision procedure and enforcement model are sound (in the model, if policy says "no FS", no filesystem effect step is admitted)
- If algebraic effect handlers are implemented in a later phase, prove their composition rules separately instead of mixing them into the initial capability-summary proof story

### Memory Safety
- Prove ownership analysis is sound for the modeled MIR/core-memory fragment (no use-after-free in the model)
- Prove reference counting maintains the required safety invariants for acyclic/shared values in the verified model
- Prove that releasing a live reference preserves the remaining well-formed live set in the current RC snapshot slice, and keep the local refcount-decrement update helper explicit — including the pure release helper's live-reference ownership/allocation and filtering corollaries, with the explicit `hasOwnership` / `allocated` / `liveAnnotated` predicate vocabulary called out in the RC snapshot model `KaliCore.Safety.releaseRefLiveRefsAreOwnedAndAllocated`, `releaseRefLiveRefsAreLiveAnnotated`, `releaseAndDecrementLiveRefsAreLiveAnnotated`, `releaseAndCollectLiveRefsAreLiveAnnotated` and, explicitly, the release-only helper theorem `KaliCore.Safety.releaseRefLiveRefsFiltered` alongside `KaliCore.Safety.releaseAndDecrementLiveRefsFiltered` and `KaliCore.Safety.releaseAndCollectLiveRefsFiltered`,  the live-to-released transition preservation theorem `KaliCore.Safety.releasePreservesWellFormed`, its release-recording, exact released-reference cons-shape via `KaliCore.Safety.releaseRefReleasedRefsCons`, `KaliCore.Safety.releaseAndDecrementReleasedRefsCons`, and `KaliCore.Safety.releaseAndCollectReleasedRefsCons`, target-cell positive-count preservation, last-ref zeroing, zero-count collection, zero-count removal from the decrement pass, positive-count preservation on the local collection helper, the release-and-decrement helper's original positive-count preservation theorem `KaliCore.Safety.releaseAndDecrementKeepsOriginalPositiveCountCells`, `KaliCore.Safety.releaseAndDecrementKeepsOtherPositiveCountCells`, the helper-level original-heap positive-count preservation lemma, the helper-level theorem `KaliCore.Safety.releaseAndCollectKeepsTargetCellWhenPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellPresentIffPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellAllocatedWhenPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellOrigin`, `KaliCore.Safety.releaseAndCollectTargetCellOriginAndPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellOriginOwnershipAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseAndCollectTargetCellOriginOwnershipAndPositiveCountAndLinearMemory`, `KaliCore.Safety.releaseAndCollectTargetCellOwnedAndAllocatedWhenPositiveCount` that the released target remains in the collected heap when its decremented count stays positive, the helper-level theorem `KaliCore.Safety.releaseRefHeapCharacterisation`, `KaliCore.Safety.releaseRefHeapCharacterisationAndLinearMemory`, `KaliCore.Safety.releaseRefHeapCellOrigin`, `KaliCore.Safety.releaseRefHeapCellOriginAndOwnership`, `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseRefHeapCellOriginOwnershipAndPositiveCountAndLinearMemory`, `KaliCore.Safety.releaseAndCollectHeapCellOrigin`, `KaliCore.Safety.releaseAndCollectHeapCellOriginAndOwnership`, `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCount`, plus its linear-memory companion `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` that the surviving collection-helper cells preserve their original name and ownership tag, the helper-level theorem `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndOwnership`, the release-and-decrement origin-and-positive-count theorem `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndPositiveCount`, the release-and-decrement origin/ownership/positivity theorem `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCount`, plus its linear-memory companion `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` that the decrement helper's surviving heap cells preserve their original name and ownership tag, the helper-level theorem `KaliCore.Safety.releaseAndDecrementHeapCharacterisation`, `KaliCore.Safety.releaseAndDecrementHeapCharacterisationAndLinearMemory`, the helper-level theorem `KaliCore.Safety.releaseAndCollectHeapCharacterisation`, `KaliCore.Safety.releaseAndCollectHeapCharacterisationAndLinearMemory`, the helper-level theorem `KaliCore.Safety.releaseAndCollectHeapCellOriginAndPositiveCount` that the surviving collection-helper cells are both traceable to the original heap and positive-count, the helper-level theorem `KaliCore.Safety.releaseAndCollectHeapIsPositiveCountFilter` that `releaseAndCollect` is exactly the positive-count filter of the decrement pass, the helper-level theorem `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount` that the local collection helper's final heap contains only positive-count cells, the helper-level theorem that original zero-count cells are dropped from the final heap, unrelated-heap preservation via `KaliCore.Safety.releaseAndDecrementKeepsOtherHeapEntries` and `KaliCore.Safety.releaseAndCollectKeepsOtherHeapEntries`, other-live-reference preservation via `KaliCore.Safety.releaseAndDecrementPreservesOtherLiveRefs` and `KaliCore.Safety.releaseAndCollectPreservesOtherLiveRefs`, the helper-level theorem that every surviving release-and-collect heap cell still comes from the original heap with only the released target decremented, helper-level ownership/allocation preservation corollaries on the decrement and collection paths, mechanized `KaliCore.Safety.noDanglingReference` theorem plus the helper-level no-dangling-reference corollaries `KaliCore.Safety.releaseRefNoDanglingReference`, `KaliCore.Safety.releaseAndDecrementNoDanglingReference`, and `KaliCore.Safety.releaseAndCollectNoDanglingReference`, ownership-envelope preservation on the release-only, decrement, and collection helpers, release-set preservation on the release-only, decrement, and collection helpers via `KaliCore.Safety.releaseRefPreservesReleasedRefs`, `KaliCore.Safety.releaseAndDecrementPreservesReleasedRefs`, and `KaliCore.Safety.releaseAndCollectPreservesReleasedRefs`, and live/released-disjointness bookkeeping, plus the local `releaseAndCollect` release-recording/disjointness theorems — while the fuller ownership/freeing story remains out of scope. The proof-summary anti-drift guard also tracks `KaliCore.Safety.liveRefsAreOwnedAndAllocated`, `KaliCore.Safety.releaseAndCollectLiveRefsAreOwnedAndAllocated`, `KaliCore.Safety.releaseAndDecrementPreservesOwnership`, `KaliCore.Safety.releaseAndCollectPreservesOwnership`, `releasedNotLive`, and `releasedNotLiveRef` so the mechanised inventory stays pinned to the live-reference and ownership slices. The companion linear-memory theorem `KaliCore.Safety.releaseAndCollectHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` is now spelled out directly in the summary so the provenance slice stays explicit rather than relying on shortened companion wording. The heap-characterisation slice now also names `KaliCore.Safety.releaseRefHeapCharacterisationAndLinearMemory`, `KaliCore.Safety.releaseAndDecrementHeapCharacterisationAndLinearMemory`, and `KaliCore.Safety.releaseAndCollectHeapCharacterisationAndLinearMemory` directly so the linear-memory pairing stays explicit there too. The same proof summary now also names `KaliCore.Safety.releaseAndCollectHeapCellOriginAndPositiveCountAndLinearMemory` explicitly.
- Treat cycle handling as a separate engineering/debugging concern unless and until the formal model includes it explicitly
- The current RC snapshot helper slice also pins `KaliCore.Safety.releaseAndCollectKeepsOtherPositiveCountCells`, `KaliCore.Safety.releaseAndCollectKeepsOriginalPositiveCountCells`, and the release-set preservation theorems `KaliCore.Safety.releaseRefPreservesReleasedRefs`, `KaliCore.Safety.releaseAndDecrementPreservesReleasedRefs`, `KaliCore.Safety.releaseAndCollectPreservesReleasedRefs`, which keeps the surviving non-target positivity story explicit and the release-set story explicit. It also names the remaining bookkeeping corollaries `KaliCore.Safety.releaseRecorded`, `KaliCore.Safety.releaseAndDecrementRecorded`, `KaliCore.Safety.releaseAndDecrementDecrementsTargetCell`, `KaliCore.Safety.releaseAndDecrementPreservesWellFormed`, `KaliCore.Safety.releaseAndDecrementLiveRefsAreOwnedAndAllocated`, `KaliCore.Safety.releaseAndDecrementReleasedNotLiveRef`, `KaliCore.Safety.releaseAndDecrementZeroesLastTargetCell`, `KaliCore.Safety.releaseAndCollectRecorded`, `KaliCore.Safety.releaseAndCollectKeepsPositiveCountCells`, `KaliCore.Safety.releaseAndCollectDropsOriginalZeroCountCells`, `KaliCore.Safety.releaseAndCollectPreservesWellFormed`, `KaliCore.Safety.releaseAndCollectReleasedNotLiveRef`, `KaliCore.Safety.releaseAndCollectRemovesZeroCountCells`, `KaliCore.Safety.releaseRefPreservesOwnership`, `KaliCore.Safety.releaseRefReleasedNotLiveRef`, `releasedNotLive`, and `releasedNotLiveRef`.
- Prove escape analysis is conservative (if analysis says "doesn't escape", it truly doesn't)

### Compilation Correctness (Selective)
Prove specific high-value lowering passes preserve the modeled semantics:
- `async/await` desugaring preserves execution order
- closure capture analysis captures at least the needed variables and does not omit live captures
- numeric specialization preserves semantics for the fragment whose preconditions are proved
- ownership/layout lowering preserves observable behavior for the verified subset

## Lean 4 Project Structure

Current-state clarification:
- follow the shared **current-repository-state vs target-contract reading** from [SPEC.md](../SPEC.md): the repository now contains a checked-in Lean project tree under `proofs/`, and the published boundary is proof-backed for the widened closed fragment — now including assignment and try/catch in addition to literals, variables, closed functions, application, sequencing, and conditionals — plus a small RC snapshot safety slice (including live-reference ownership/allocation projection, exact live-reference filtering via the release-only helper theorem `KaliCore.Safety.releaseRefLiveRefsFiltered` and the decrement/collection theorems `KaliCore.Safety.releaseAndDecrementLiveRefsFiltered` and `KaliCore.Safety.releaseAndCollectLiveRefsFiltered`, release-update preservation, explicit release-recording and exact released-reference cons-shape via `KaliCore.Safety.releaseRefReleasedRefsCons`, `KaliCore.Safety.releaseAndDecrementReleasedRefsCons`, and `KaliCore.Safety.releaseAndCollectReleasedRefsCons`, pure release-helper ownership/allocation and disjointness corollaries, ownership-envelope preservation on the release-only, decrement, and collection helpers, release-set preservation on the release-only, decrement, and collection helpers, target-cell decrement bookkeeping, heap-origin provenance for the release-and-decrement helper, the helper-level theorem `KaliCore.Safety.releaseAndDecrementHeapCharacterisation`, `KaliCore.Safety.releaseAndDecrementHeapCharacterisationAndLinearMemory`, the helper-level theorem `KaliCore.Safety.releaseAndCollectHeapCharacterisation`, `KaliCore.Safety.releaseAndCollectHeapCharacterisationAndLinearMemory`, the helper-level theorem `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndOwnership`, the release-and-decrement origin-and-positive-count theorem `KaliCore.Safety.releaseAndDecrementHeapCellOriginAndPositiveCount`, the release-and-decrement origin/ownership/positivity theorem `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCount`, plus its linear-memory companion `KaliCore.Safety.releaseAndDecrementHeapCellOriginOwnershipAndPositiveCountAndLinearMemory` that the decrement helper's surviving heap cells preserve their original name and ownership tag, last-ref zeroing, zero-count collection, zero-count removal from the decrement pass, zero-count removal from the collected heap, positive-count preservation on the local collection helper, the helper-level theorem `KaliCore.Safety.releaseAndDecrementKeepsTargetCellWhenPositiveCount`, `KaliCore.Safety.releaseAndDecrementTargetCellPositiveCountIff`, `KaliCore.Safety.releaseAndDecrementTargetCellOrigin` and the target-cell origin/positive-count theorem `KaliCore.Safety.releaseAndDecrementTargetCellOriginAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseAndDecrementTargetCellOriginAndPositiveCountAndLinearMemory`, `KaliCore.Safety.releaseAndDecrementTargetCellAllocatedWhenPositiveCount`, `KaliCore.Safety.releaseAndDecrementTargetCellOwnedAndAllocatedWhenPositiveCount`, and the target-cell positive-count iff bridge `KaliCore.Safety.releaseAndDecrementTargetCellPositiveCountIff` that the decremented target remains in the decrement-pass heap when its count stays positive, the helper-level theorem `KaliCore.Safety.releaseAndCollectKeepsTargetCellWhenPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellAllocatedWhenPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellOriginAndPositiveCount`, `KaliCore.Safety.releaseAndCollectTargetCellOriginOwnershipAndPositiveCount` plus its linear-memory companion `KaliCore.Safety.releaseAndCollectTargetCellOriginOwnershipAndPositiveCountAndLinearMemory`, `KaliCore.Safety.releaseAndCollectTargetCellOwnedAndAllocatedWhenPositiveCount` that the released target remains in the collected heap when its decremented count stays positive, the helper-level theorem `KaliCore.Safety.releaseAndCollectHeapCellsHavePositiveCount` that the local collection helper's final heap contains only positive-count cells, the helper-level theorem that positive-count cells from the original heap survive when they are not the released target and remain positive-count after collection, the helper-level theorem that original zero-count cells are dropped from the final heap, unrelated-heap preservation, other-live-reference preservation on the local `releaseAndCollect` helper, the helper-level theorem that `releaseAndCollect` is exactly the positive-count filter of the decrement pass, the helper-level theorem that every surviving release-and-collect heap cell still comes from the original heap with only the released target decremented, helper-level live-reference filtering theorems on the release-only, decrement, and collection helpers, helper-level ownership/allocation preservation corollaries on the decrement and collection paths, mechanized `KaliCore.Safety.noDanglingReference` theorem plus the helper-level no-dangling-reference corollaries `KaliCore.Safety.releaseRefNoDanglingReference`, `KaliCore.Safety.releaseAndDecrementNoDanglingReference`, and `KaliCore.Safety.releaseAndCollectNoDanglingReference`, disjointness on the decrement path, the local `releaseAndCollect` release-recording/disjointness theorems, and `KaliCore.Safety.releaseAndCollectDropsZeroCountCells`, and a refcount-decrement update helper) and a widened HIR lowering-correctness slice that now also includes `KaliIR.Value`, `KaliIR.LoweringCorrectness.lower_preserves_value`, and bare throw while still remaining narrower than the later Stage 4.2 target
- treat the layout below as the living proof-tree shape that the checked-in Lean model follows today, not as a claim that the boundary already covers the later ownership/memory-safety and lowering-correctness target

```text
proofs/
├── Kali/
│   ├── Syntax.lean          — AST and type syntax definitions
│   ├── Types/
│   │   ├── Core.lean        — Core type definitions
│   │   ├── Subtyping.lean   — Subtyping relation and proofs
│   │   ├── Unification.lean — Unification algorithm and termination proof
│   │   ├── Inference.lean   — Type inference and principality
│   │   └── Soundness.lean   — Progress + preservation theorems
│   ├── Effects/
│   │   ├── Core.lean        — Effect definitions
│   │   ├── Inference.lean   — Effect inference correctness
│   │   └── Handlers.lean    — Effect handler soundness (optional, later phase)
│   ├── Memory/
│   │   ├── Ownership.lean   — Ownership model
│   │   ├── Escape.lean      — Escape analysis correctness
│   │   └── RefCount.lean    — Reference counting invariants
│   └── Sandbox/
│       ├── Policy.lean      — Policy model
│       └── Enforcement.lean — Enforcement soundness
├── lakefile.lean
└── lean-toolchain
```

## Proof-Backed Support Boundary

Lean proofs are evidence for the behavior named by the published **proof-boundary manifest**, not a blanket support claim for all of Kali.

Canonical rule:
- a proof may justify stronger confidence for the core fragment currently named by that manifest
- it does **not** by itself promote a feature's maturity label or replace the command/profile-specific evidence tracks from [specs/16-testing.md](16-testing.md)
- public support wording should therefore require both: the proof claim staying inside the published **proof-boundary manifest** **and** the matching implementation/testing evidence for the command/profile being claimed
- when the implementation grows beyond the currently published proof boundary, the unsupported remainder must stay explicitly outside that manifest rather than being described as informally "covered enough"

This keeps the bootstrap's Lean-verification ambition aligned with the rest of the spec set: verification grows iteratively, but support claims remain evidence-backed and phase-correct.

## Methodology

### Modeling
- Define a simplified operational semantics for Kali's core language in Lean
- This is not the full ECMA-262 spec — focus on the subset relevant to type safety, built-in capability effects, ownership, and selected lowering passes
- Make the proof boundary explicit: late compatibility features remain outside `proofs/BOUNDARY.md` until their semantics are frozen
- The model is a **specification** that the Rust implementation must conform to

### Proof-Implementation Link
- Lean proofs verify properties of the **model**
- The Rust implementation is tested against the model via:
  - Property-based tests derived from Lean theorems
  - Test cases extracted from proof counterexample search
  - Manual review ensuring Rust code matches model structure

### Iterative Development
1. Write Lean model for a feature (e.g., union type narrowing)
2. Prove key properties
3. Implement in Rust, using the model as specification
4. Write property tests that check Rust against Lean model
5. When the spec changes, update Lean model first, re-verify, then update Rust

## CI Integration

Current proof-check command for the checked-in Lean tree:

```bash
# Build and check all Lean proofs for the current published proof boundary
cd proofs && lake build
```

Current-state rule:
- `cd proofs && lake build` is now the real proof-check command for the checked-in Lean project under `proofs/`; repositories that have not yet adopted the Lean tree may still treat this as illustrative rather than runnable
- the current repository obligation is the published proof-boundary manifest plus the trigger policy below

CI consistency rules:
- proof failure blocks merge for the subsystem set currently named by the published **proof-boundary manifest**; this is not a claim that all of Kali is already formalized
- the proof job should always trigger when a PR changes `proofs/`
- once the published **proof-boundary manifest** names covered Rust/spec subsystems, the proof job should also trigger when a PR changes one of those covered areas
- subsystems outside the current proof boundary may evolve without a mandatory proof job, but they must remain outside the documented proof claims until the model is extended

## Non-Goals

- Full ECMA-262 formalization (too large, diminishing returns)
- A proof that every TypeScript-compatible surface feature has principal types or a simple soundness theorem; the proof target is the explicitly modeled core fragment
- Verification of the WASM binary encoder (rely on `wasm-validate` + testing)
- Full verification of concrete host integrations (OS/filesystem/network behavior is tested, not mechanically proved end-to-end)
- Real-time proof checking during development; proof checking is a CI/policy concern rather than an every-edit requirement, and until a real Lean project exists in `proofs/` the repository only carries the published proof-boundary manifest plus its trigger policy

- Linear-memory payload preservation corollaries: `KaliCore.Safety.releaseRefPreservesLinearMemory`, `KaliCore.Safety.releaseAndDecrementPreservesLinearMemory`, and `KaliCore.Safety.releaseAndCollectPreservesLinearMemory`, plus the combined ownership/linear-memory corollaries `KaliCore.Safety.releaseRefPreservesOwnershipAndLinearMemory`, `KaliCore.Safety.releaseAndDecrementPreservesOwnershipAndLinearMemory`, and `KaliCore.Safety.releaseAndCollectPreservesOwnershipAndLinearMemory`, the combined wellformedness/linear-memory corollaries `KaliCore.Safety.releaseRefPreservesWellFormedAndLinearMemory`, `KaliCore.Safety.releaseAndDecrementPreservesWellFormedAndLinearMemory`, and `KaliCore.Safety.releaseAndCollectPreservesWellFormedAndLinearMemory`, plus the combined wellformedness/ownership/linear-memory corollaries `KaliCore.Safety.releaseRefPreservesWellFormedAndOwnershipAndLinearMemory`, `KaliCore.Safety.releaseAndDecrementPreservesWellFormedAndOwnershipAndLinearMemory`, and `KaliCore.Safety.releaseAndCollectPreservesWellFormedAndOwnershipAndLinearMemory`, plus the collection helper's heap-filter-and-linear-memory corollary `KaliCore.Safety.releaseAndCollectHeapIsPositiveCountFilterAndLinearMemory`.
