# 17 — Formal Verification

## Overview

Use Lean 4 to formally verify critical invariants of Kali's implementation. Verification is iterative — proofs are developed alongside the Rust implementation and updated as the spec evolves.

## Scope

Focus verification on the highest-value areas where bugs have the most impact.

Important simplification rule: Lean proofs target a **core Kali calculus**, not the full surface language all at once. Early proof work should model the statically analyzable subset that excludes late-compatibility features such as `eval`, dynamic module loading, weak/finalization semantics, and browser/OS host details. Those outer features are handled by explicit phase gates in the implementation and only enter the proof story once their semantics stabilize.

## Phase-aligned proof scope

To keep the bootstrap's Lean requirement aligned with the rest of the phased spec, proof work follows the same staged story rather than implying full-language verification from day one.

| Phase | Verification focus |
|---|---|
| Phase 1 MVP | Published proof boundary plus at least one concrete modeled theorem family over the core typed calculus / sandbox-policy core before any Phase-1 release that advertises formal verification; during earlier spec iteration the manifest may remain empty, but only if Kali avoids proof-backed support claims |
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

Before the first proofs land, the manifest may truthfully publish an **empty current proof boundary**. That is still preferable to omitting the file, because it prevents the rest of the spec from accidentally implying proof coverage that does not yet exist.

Follow the shared **proof activation split** from [SPEC.md](../SPEC.md):
- an empty manifest is acceptable during spec-first iteration and early implementation bootstrapping
- it is **not** enough for a release to market Kali as already formally verified in Phase 1
- any Phase-1 release note or support claim that leans on formal verification should first replace the empty boundary with at least one concrete modeled subsystem plus named theorem/property claims
- while the published proof boundary is empty, proof CI is required only for changes under `proofs/`
- once the manifest names covered implementation/spec subsystems, proof CI must also trigger for changes to those covered areas
- release notes and support wording should describe that activation state plainly instead of implying that a placeholder manifest already proves part of the implementation

Practical simplification:
- one manifest is the canonical verification boundary for release notes, maturity claims, and CI wiring
- chapters may summarize it, but they should not invent slightly different proof-scope claims of their own
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

```
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

Lean proofs are evidence for the **currently modeled subset**, not a blanket support claim for all of Kali.

Canonical rule:
- a proof may justify stronger confidence for the modeled core fragment
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

```bash
# Build and check all Lean proofs for the currently modeled subset
cd proofs && lake build

# In CI, run this job whenever the proof tree or a covered subsystem changes
```

CI consistency rules:
- proof failure blocks merge **for the currently modeled subset**; this is not a claim that all of Kali is already formalized
- the proof job should always trigger when a PR changes `proofs/`
- once the published **proof-boundary manifest** names covered Rust/spec subsystems, the proof job should also trigger when a PR changes one of those covered areas
- subsystems outside the current proof boundary may evolve without a mandatory proof job, but they must remain outside the documented proof claims until the model is extended

## Non-Goals

- Full ECMA-262 formalization (too large, diminishing returns)
- A proof that every TypeScript-compatible surface feature has principal types or a simple soundness theorem; the proof target is the explicitly modeled core fragment
- Verification of the WASM binary encoder (rely on `wasm-validate` + testing)
- Full verification of concrete host integrations (OS/filesystem/network behavior is tested, not mechanically proved end-to-end)
- Real-time proof checking during development (Lean builds run in CI)
