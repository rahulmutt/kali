# 07 — Optimization & Specialization

Optimization and specialization passes are implemented in the `kali_optimize` crate, which operates on all IR levels. Analyses that feed into IR construction (e.g., escape analysis for MIR memory layout decisions) are co-located in the relevant IR crate (`kali_mir`) but invoked by `kali_optimize`.

## Generic Specialization

### Strategy
Aggressively specialize generic functions at each call site based on concrete types. This produces distinct WASM functions with optimized memory layouts for each type combination.

### What Gets Specialized
- **Generic functions**: `function map<T, U>(arr: T[], fn: (t: T) => U): U[]`
- **Generic classes**: Each instantiation gets its own vtable and memory layout
- **Polymorphic call sites**: If a function is called with 3 different type combinations, 3 specialized versions are emitted
- **Closures**: Specialized for their capture set's concrete types

### Specialization Process
1. Collect all call sites for each generic function
2. Group by unique type argument tuples
3. For each unique tuple, instantiate the function body with concrete types
4. Lower to the canonical layout-aware IR for the active phase: MIR in Phase 2+, or directly to LIR in Phase 1 while MIR is still being introduced
5. Apply type-specific optimizations (e.g., integer arithmetic for `number`)

### Memory Layout Specialization
```typescript
// Generic:
function first<T>(arr: T[]): T { return arr[0]; }

// Call sites:
first([1, 2, 3]);           // T = number → f64[] layout, direct f64 load
first(["a", "b"]);          // T = string → ptr[] layout, pointer load
first([{x: 1}, {x: 2}]);   // T = {x: number} → struct[] layout, 8-byte stride
```

Each specialization uses the most compact possible layout for its concrete type.

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
- **Optional external post-pass**: If users install `wasm-opt`, Kali may invoke it as a separate user-provided tool, but Kali's core optimization pipeline must remain fully implemented in Rust and must not depend on Binaryen

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
