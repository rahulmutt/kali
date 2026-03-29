# 07 — Specialization & Optimization

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
4. Lower to MIR with concrete memory layouts (no boxing)
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
- User-configurable via `--max-specializations N`
- In `--fast` mode (the default), skip specialization entirely (use boxed/tagged representation for generics)

## Optimization Passes

### Phase 1: HIR Optimizations (always applied)
- **Constant folding**: `1 + 2` → `3`, `"a" + "b"` → `"ab"`
- **Dead code elimination**: Unreachable branches, unused variables
- **Inlining** (small functions): Functions ≤ 20 HIR nodes inlined at call site
- **Constant propagation**: Track known values through assignments

### Phase 2: MIR Optimizations (--release)
- **Escape analysis refinement**: Promote heap → stack where possible
- **Rc elision**: Remove unnecessary reference count operations
- **Copy propagation**: Eliminate redundant copies/moves
- **Common subexpression elimination (CSE)**
- **Loop-invariant code motion (LICM)**
- **Devirtualization**: When the concrete type of a virtual call is known, call directly
- **String interning**: Deduplicate constant strings at compile time
- **Bounds check elimination**: Prove array accesses are in bounds, remove checks

### Phase 3: LIR/WASM Optimizations (--release-advanced)
- **Full specialization**: Specialize all generics (no cap)
- **Aggressive inlining**: Higher threshold, inline across module boundaries
- **Tail call optimization**: WASM tail-call proposal when available
- **WASM-specific peephole**: Combine instruction sequences, optimize local usage
- **Linear memory layout optimization**: Place frequently co-accessed data adjacently
- **LTO (Link-Time Optimization)**: Cross-module inlining and dead code elimination
- **wasm-opt integration**: Optionally pipe output through Binaryen's `wasm-opt` (external tool, not linked — must be installed separately; invoked as a subprocess)

## Dynamic Fallback

When the compiler cannot statically determine types or layouts:

```rust
enum ValueRepr {
    /// Known type, unboxed, native WASM value
    Unboxed { wasm_type: WasmValType },
    /// Known type, struct layout in linear memory
    Struct { layout: ObjectLayout },
    /// Unknown type, tagged union (NaN-boxing or tagged pointer)
    Tagged,
    /// Fully dynamic, hash map representation
    Dynamic,
}
```

### NaN-Boxing for Tagged Values
When a value's type is unknown at compile time, use NaN-boxing in a 64-bit float:
- Doubles: stored directly
- Integers (i31): tagged in NaN payload
- Pointers: tagged in NaN payload (32-bit WASM address space)
- Booleans, null, undefined: sentinel NaN values

### Transitions
The compiler inserts boxing/unboxing at boundaries between typed and untyped code:
- Calling an untyped function from typed code → box arguments
- Receiving results from untyped code → unbox (with runtime type check)
- These transitions are flagged in diagnostics so users know where performance degrades

## Profile-Guided Optimization (Future)

- Runtime profiling data from `--profile` runs
- Feed back into compiler for better inlining, specialization, and branch prediction hints
- Not in initial implementation
