# 17 — Formal Verification

## Overview

Use Lean 4 to formally verify critical invariants of Kali's implementation. Verification is iterative — proofs are developed alongside the Rust implementation and updated as the spec evolves.

## Scope

Focus verification on the highest-value areas where bugs have the most impact:

### Type System Soundness
Prove that Kali's type system is sound:
- **Progress**: Well-typed programs don't get stuck (can always take a step or are values)
- **Preservation**: Evaluation preserves types (if `e : T` and `e → e'`, then `e' : T`)
- Model the core type language (primitives, unions, intersections, functions, objects)
- Prove subtyping is transitive and antisymmetric
- Prove unification algorithm terminates and finds principal types
- Prove constraint solving is decidable for the supported constraint language

### Effect System Correctness
- Prove effect inference is conservative (inferred effects ⊇ actual effects)
- Prove the sandbox policy decision procedure and enforcement model are sound (in the model, if policy says "no FS", no filesystem effect step is admitted)
- If algebraic effect handlers are implemented in a later phase, prove their composition rules separately

### Memory Safety
- Prove ownership analysis is sound (no use-after-free in the model)
- Prove reference counting maintains invariant: refcount = 0 ⟹ object is unreachable
- Prove escape analysis is conservative (if analysis says "doesn't escape", it truly doesn't)

### Compilation Correctness (Selective)
- Prove specific lowering passes preserve semantics:
  - `async/await` desugaring preserves execution order
  - Closure capture analysis captures exactly the needed variables
  - Numeric type specialization (`f64` → `i32`) preserves arithmetic results for integer inputs

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
- This is not the full ECMA-262 spec — focus on the subset relevant to type safety and effects
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
- Verification of the WASM binary encoder (rely on `wasm-validate` + testing)
- Full verification of concrete host integrations (OS/filesystem/network behavior is tested, not mechanically proved end-to-end)
- Real-time proof checking during development (Lean builds run in CI)
