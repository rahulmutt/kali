# 17 — Formal Verification

## Overview

Use Lean 4 to formally verify critical invariants of Kali's implementation. Verification is iterative — proofs are developed alongside the Rust implementation and updated as the spec evolves.

## Scope

Focus verification on the highest-value areas where bugs have the most impact.

Important simplification rule: Lean proofs target a **core Kali calculus**, not the full surface language all at once. Early proof work should model the statically analyzable subset that excludes late-compatibility features such as `eval`, dynamic module loading, weak/finalization semantics, and browser/OS host details. Those outer features are handled by explicit phase gates in the implementation and only enter the proof story once their semantics stabilize.

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

## Methodology

### Modeling
- Define a simplified operational semantics for Kali's core language in Lean
- This is not the full ECMA-262 spec — focus on the subset relevant to type safety, built-in capability effects, ownership, and selected lowering passes
- Make the proof boundary explicit: late compatibility features remain outside the proof kernel until their semantics are frozen
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
# Build and check all Lean proofs
cd proofs && lake build

# Run in CI — proof failure blocks merge
```

Proofs are checked on every PR that modifies type system, effect system, or memory management code.

## Non-Goals

- Full ECMA-262 formalization (too large, diminishing returns)
- A proof that every TypeScript-compatible surface feature has principal types or a simple soundness theorem; the proof target is the explicitly modeled core fragment
- Verification of the WASM binary encoder (rely on `wasm-validate` + testing)
- Full verification of concrete host integrations (OS/filesystem/network behavior is tested, not mechanically proved end-to-end)
- Real-time proof checking during development (Lean builds run in CI)
