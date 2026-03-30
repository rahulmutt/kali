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
| **proof-ready** | published `proofs/BOUNDARY.md`, honest proof-CI trigger policy, and explicit no-overclaim discipline | the repository is prepared for phased verification work, but does **not** claim shipped mechanized coverage yet |
| **proof-backed** | the manifest is non-empty and names at least one concrete modeled subsystem plus theorem/property inventory | release/support wording may cite formal verification, but only for the published boundary |

Practical rule:
- Phase 1 should be **proof-ready** from the start
- the Phase-1 contract is therefore repository/process hygiene first: published boundary, an honest proof-CI trigger policy, and explicit no-overclaim discipline
- **proof-backed** is not itself a blanket Phase-1 requirement; it becomes required only for release/support wording that wants to market formal verification as shipped evidence rather than as future-facing process readiness

Current repository status rule:
- `proofs/BOUNDARY.md` is the single source of truth for the repository's current verification state
- chapter summaries, release notes, and README copy should cite or quote that manifest rather than restating current proof coverage from memory or from this chapter's roadmap prose
- if the manifest is still the shared **placeholder proof-boundary manifest**, the honest repository claim remains **proof-ready** rather than **proof-backed**
- in that placeholder state, repository summaries should say the quiet part explicitly: Kali is proof-ready, but no mechanized proof coverage is currently claimed yet
- [19 — Feature Maturity](19-feature-maturity.md) makes the same guardrail explicit: proof-backed release/support claims while the published boundary is still empty are **Rejected by default**

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

Before the first proofs land, the manifest may truthfully stay in the shared **placeholder proof-boundary manifest** state from [SPEC.md](../SPEC.md). That is still preferable to omitting the file, because it prevents the rest of the spec from accidentally implying proof coverage that does not yet exist. The current repository state should always be read from `proofs/BOUNDARY.md`, not inferred from this chapter's examples or milestone plan.

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
- Treat cycle handling as a separate engineering/debugging concern unless and until the formal model includes it explicitly
- Prove escape analysis is conservative (if analysis says "doesn't escape", it truly doesn't)

### Compilation Correctness (Selective)
Prove specific high-value lowering passes preserve the modeled semantics:
- `async/await` desugaring preserves execution order
- closure capture analysis captures at least the needed variables and does not omit live captures
- numeric specialization preserves semantics for the fragment whose preconditions are proved
- ownership/layout lowering preserves observable behavior for the verified subset

## Lean 4 Project Structure

Current-state clarification:
- follow the shared **current-repository-state vs target-contract reading** from [SPEC.md](../SPEC.md): the repository does **not** yet contain this Lean project tree; today the only required verification artifact is the published `proofs/BOUNDARY.md` manifest
- treat the layout below as the **target proof-tree shape once mechanized proofs start landing**, not as a claim that those files already exist in the current repo state

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

Future-state command once the Lean proof tree exists:

```bash
# Build and check all Lean proofs for the current published proof boundary
cd proofs && lake build
```

Current-state rule:
- follow the shared **current-repository-state vs target-contract reading** from [SPEC.md](../SPEC.md): until `proofs/` contains an actual Lean project (`lakefile.lean`, `lean-toolchain`, and proof sources), this command is illustrative rather than currently runnable
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
- Real-time proof checking during development (Lean builds run in CI)
