# 07 — Optimization & Specialization

Current repository-state clarification:
- the crate names in this chapter describe the intended ownership/decomposition from [01 — Architecture](./01-architecture.md); the repository may already contain similarly named crates, but optimization-surface availability and maturity still come from the owning chapters plus [19 — Feature Maturity](./19-feature-maturity.md)

Optimization and specialization passes are implemented in the `kali_optimize` crate, which operates on all IR levels. Analyses that feed into IR construction (e.g., escape analysis for MIR memory layout decisions) are co-located in the relevant IR crate (`kali_mir`) but invoked by `kali_optimize`.

## Generic Specialization

### Strategy
Aggressively specialize generic functions at each call site, but key those specializations by the canonical **layout/representation fingerprint** plus any remaining semantic distinctions that still affect correctness. This produces distinct WASM functions when code shape or observable behavior materially differs, without cloning code merely because two source-level types have different names.

Simplification rule:
- specialization keys are **layout-first**, not source-type-name-first
- if two instantiations lower to the same parameter/result fingerprints and require the same guards/runtime operations, they should share one specialization
- if two instantiations differ only in source-level typing detail that disappears before MIR/LIR/codegen, they should not force separate emitted WASM functions

### What Gets Specialized
- **Generic functions**: `function map<T, U>(arr: T[], fn: (t: T) => U): U[]`
- **Generic classes**: each materially distinct layout/dispatch shape gets its own vtable and layout instance
- **Polymorphic call sites**: if a function is called with 3 materially different fingerprint combinations, 3 specialized versions are emitted
- **Closures**: specialized for materially different capture-layout fingerprints rather than for cosmetic source-type differences alone

### Specialization Process
1. Collect all call sites for each generic function
2. Lower each candidate call into a provisional specialization key built from parameter/result **layout/representation fingerprints** plus any remaining semantic distinctions that still affect correctness
3. Group by that canonical key rather than by raw type-argument spelling alone
4. For each unique key, instantiate the function body with the corresponding concrete lowering assumptions
5. Lower to the canonical layout-aware IR for the active phase: MIR from the Phase 2 target onward, or directly to LIR in Phase 1 while MIR is still being introduced
6. Apply type-/layout-specific optimizations (for example integer fast paths for eligible `number` flows)

Examples of distinctions that may still keep separate specializations even when layouts look similar:
- runtime checks needed to preserve JavaScript-visible numeric behavior
- different drop/refcount paths caused by ownership/escape differences that survive lowering
- calling-convention differences at ABI boundaries

Examples of distinctions that should normally **not** force separate specializations by themselves:
- alias names
- generic parameter names
- source-level types that collapse to the same tagged fallback representation

### Memory Layout Specialization
```typescript
// Generic:
function first<T>(arr: T[]): T { return arr[0]; }

// Call sites:
first([1, 2, 3]);           // T = number → f64[] layout, direct f64 load
first(["a", "b"]);          // T = string → ptr[] layout, pointer load
first([{x: 1}, {x: 2}]);   // T = {x: number} → struct[] layout, 8-byte stride
```

Each specialization uses the most compact possible layout for its concrete lowering.

Important simplification:
- if two source-level instantiations produce the same array/object layout fingerprint and the same surrounding runtime obligations, they should reuse one specialization instead of emitting duplicate code
- this is how Kali stays faithful to the bootstrap's “specialize memory layouts aggressively” goal without equating that goal with “clone on every nominal type difference”

### Specialization Limits
- Cap specializations per function (default: 16) to prevent code size explosion
- Beyond the cap, fall back to a "generic" version using boxed/tagged representation
- User-configurable via `--max-specializations N` / `compilerOptions.maxSpecializations`
- `--max-specializations` is an **upper bound**, not a promise that every build mode spends that budget
- In `--fast` mode (the default), skip most user-authored generic specialization entirely (effectively treating the user-authored generic budget as `0` unless a later documented heuristic says otherwise)
- In `--release` and `--release-advanced`, the configured cap becomes the main user-visible ceiling for the generic/layout-driven specialization pipeline
- `--release-advanced` may use more of that configured budget more aggressively than `--release`, but it must still respect the explicit user/configured cap rather than silently removing it

## Optimization Passes

Build-mode clarification:
- `fast`, `release`, and `release-advanced` are user-visible from Phase 1.
- What changes across phases is **how much optimizer/compiler machinery exists behind those stable mode names**, not whether the flags themselves exist.
- Therefore early `--release` / `--release-advanced` builds may start with a modest subset of the long-term optimizations described below and grow stronger as MIR, specialization, and later LIR passes mature.
- The phase-labeled sections below describe when major optimization families become available; they do **not** mean the corresponding build-mode flag first appears in that phase.

### Phase 1 floor: HIR Optimizations (always applied)
- **Constant folding**: `1 + 2` → `3`, `"a" + "b"` → `"ab"`
- **Dead code elimination**: Unreachable branches, unused variables
- **Inlining** (small functions): Functions ≤ 20 HIR nodes inlined at call site
- **Constant propagation**: Track known values through assignments

### Phase 2 additions: MIR Optimizations (primarily strengthen `--release` and above)
- **Escape analysis refinement**: Promote heap → stack where possible
- **Shared-refcount elision**: Remove unnecessary reference count operations
- **Copy propagation**: Eliminate redundant copies/moves
- **Common subexpression elimination (CSE)**
- **Loop-invariant code motion (LICM)**
- **Devirtualization**: When the concrete type of a virtual call is known, call directly
- **String interning**: Deduplicate constant strings at compile time
- **Bounds check elimination**: Prove array accesses are in bounds, remove checks

### Phase 3 additions: LIR/WASM Optimizations (primarily strengthen `--release-advanced`)
- **Expanded specialization budget**: Specialize far more generic instantiations than `--release`, potentially approaching whole-program monomorphization for hot code, while still retaining an emergency fallback for pathological code-size growth
- **Aggressive inlining**: Higher threshold, inline across module boundaries
- **Tail call optimization**: WASM tail-call proposal when available
- **WASM-specific peephole**: Combine instruction sequences, optimize local usage
- **Linear memory layout optimization**: Place frequently co-accessed data adjacently
- **LTO (Link-Time Optimization)**: Cross-module inlining and dead code elimination
- **Optional external post-pass**: If users install `wasm-opt`, Kali may invoke it as a separate user-provided tool, but this remains an additive `release-advanced` helper only. Follow the shared **Pure-Rust implementation contract** from [SPEC.md](../SPEC.md): Kali's core optimization pipeline, tests, and feature claims must remain valid without Binaryen or any other external post-pass tool.

## Performance-claim discipline

The bootstrap brief's benchmark and "on par with Rust" aspirations are normalized here as an optimization-evidence program, not as a blanket early-phase performance promise.

Rules:
- performance claims must name the workload class, build mode, and comparison baseline instead of saying only that Kali is "fast"
- benchmark wins are evidence for optimization maturity, not a substitute for semantic-correctness evidence
- the canonical benchmark lane belongs to the testing/evidence program: adapted Computer Language Benchmarks Game workloads and representative real-world regressions should be version-pinned and rerunnable
- public performance wording should stay phase-correct: Phase 1 may claim stable build-mode vocabulary and deterministic benchmarking hooks, while stronger throughput/latency claims belong to later optimization phases once evidence exists
- package-oriented benchmark anecdotes must not be used to widen host/API compatibility claims

## Dynamic Fallback

When specialization is not possible (type unknown, exceeds specialization cap), the compiler falls back to tagged/dynamic representations. See [05 — Intermediate Representations](05-ir.md#value-representation) for the `ValueRepr` enum and NaN-boxing scheme.

Clarification:
- disabling generic specialization in `--fast` does **not** mean the whole compiler becomes dynamically typed
- monomorphic code, concrete object layouts, and straightforward scalar optimizations should still use the best representation already justified by the checker and IR pipeline
- the fallback applies specifically at specialization boundaries where extra compile-time work or code-size growth would otherwise be required

In `--fast` mode, user-authored generics normally use the `Tagged` representation (no additional specialization regardless of the configured cap). In `--release`, generics are specialized up to the configured cap, with remaining call sites using `Tagged`. In `--release-advanced`, the compiler may spend that configured cap more aggressively and use stronger profitability heuristics, but it must still respect the explicit user/configured cap and retain an emergency fallback for pathological code-size growth.

## Profile-Guided Optimization (Future)

- Runtime profiling data from `--profile` runs
- Feed back into compiler for better inlining, specialization, and branch prediction hints
- Not in initial implementation
