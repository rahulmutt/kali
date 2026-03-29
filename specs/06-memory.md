# 06 — Memory Management

## Principle

Kali has **no tracing garbage collector**. Allocation class and ownership strategy are chosen statically where possible through ownership analysis, lifetime inference, and escape analysis. The compiler may still insert deterministic runtime bookkeeping such as reference-count increments/decrements when sharing cannot be eliminated. This is inspired by Rust's ownership model, adapted for TypeScript/JavaScript semantics.

For compatibility-heavy APIs and dynamic features mentioned in this document, the canonical maturity/status matrix lives in [specs/19-feature-maturity.md](19-feature-maturity.md). This section focuses on memory strategy, not on redefining feature phase decisions.

## Allocation Strategy Decision

For every value, the compiler determines:

1. **Where** it lives: stack or heap
2. **How** it's owned: unique, shared (reference-counted), or borrowed
3. **When** it's freed: scope exit, last use, or ref-count drop to zero

### Decision Flow

```
Value Created
  ├─ Primitive (number, boolean, null, undefined)
  │   → Always stack (WASM locals/operand stack)
  │
  ├─ Small fixed-size struct (known layout, no escaping)
  │   → Stack allocation (alloca-like in linear memory stack)
  │
  ├─ Does not escape its creating scope?
  │   → Stack allocation (even for objects/arrays)
  │
  ├─ Escapes but single owner (returned, moved into container)?
  │   → Heap allocation, unique ownership, freed by owner
  │
  └─ Escapes and shared (stored in multiple places, closures)?
      → Heap allocation, reference-counted shared ownership
```

## Escape Analysis

Determines whether a value's lifetime exceeds its creating scope:

- **Does not escape**: used only within the function, not stored in long-lived structures
- **Escapes via return**: moved to caller, single owner transfer
- **Escapes via closure capture**: captured by a closure that outlives the scope → shared
- **Escapes via container**: stored in an object/array that escapes → follows container's ownership
- **Escapes via global/module state**: stored in module-level variable → heap + Rc

### Closure Analysis
Closures capture variables. For each capture:
- If the variable is only read and the closure doesn't outlive the variable → borrow
- If the variable is mutated or the closure escapes → lower to a shared heap cell with deterministic reference counting and interior mutability semantics
- If only one closure captures it and ownership can transfer → move

## Reference Counting

When shared ownership is needed, Kali uses compile-time-inserted reference counting:

```rust
struct RcHeader {
    ref_count: u32,
    // Optional weak count for cycle detection
    weak_count: u32,
    // Type tag for runtime type checks (when needed)
    type_tag: u16,
    // Flags (e.g., frozen, sealed)
    flags: u16,
}
```

### Rc Optimizations
- **Elide inc/dec pairs**: When a borrow is provably temporary, skip ref counting
- **Move optimization**: Transfer ownership without inc+dec (just dec the source)
- **Static ref**: Objects known to live for program lifetime skip counting entirely
- **Batch dec**: When a scope exits with multiple Rc values, batch the decrements
- **Inline ref count**: Small objects embed the count in their header (no indirection)

### Cycle Detection
JavaScript allows reference cycles (e.g., `a.b = b; b.a = a`). Strategies (all deterministic, none are tracing GC):
1. **Static detection**: If the type system can prove a cycle is possible, use weak references automatically for back-edges
2. **Trial deletion**: On `rc_dec` reaching a suspect threshold, perform local trial deletion (Bacon & Rajan algorithm) — this is a targeted, deterministic cycle reclamation, not a tracing GC
3. **Scope-limited**: For short-lived computations (typical sandbox use), use region-based allocation — all memory freed in bulk when the scope/sandbox exits
4. **Leak detection**: In debug mode, report potential cycles at program exit with source locations

## Stack Allocation in Linear Memory

WASM linear memory has a software stack for:
- Local aggregates (objects, arrays) that don't escape
- Temporary buffers (string building, array operations)
- Function call frames for captured state

```
┌──────────────────────┐ high address
│  Stack (grows ↓)     │
│                      │
│  ─── free space ───  │
│                      │
│  Heap (grows ↑)      │
├──────────────────────┤
│  Static data         │
│  (strings, globals)  │
└──────────────────────┘ low address
```

## Heap Allocator

A custom allocator in the WASM runtime (no malloc/libc):
- **Size classes**: Small allocations (≤256 bytes) use size-class freelists
- **Large allocations**: Bump allocator with free-list for reuse
- **Alignment**: All allocations aligned to 8 bytes minimum
- Implemented in Rust, compiled to WASM as part of `kali_runtime`

## JavaScript Semantics Compatibility

### Challenges
JavaScript's semantics assume GC. Key cases:

| JS Pattern | Kali Strategy |
|---|---|
| Closures capturing variables | Shared heap cell with deterministic ref counting for escaping mutable captures |
| Circular references | Weak refs, explicit back-edge lowering, or bounded cycle-reclamation strategies |
| `arguments` object | Stack-allocated array when non-escaping; heap otherwise |
| Prototype chains | Static when class hierarchy is known; shared heap object for dynamic cases |
| WeakMap/WeakSet | Later-phase compatibility feature; unsupported in early phases until weak-reference semantics are specified without violating observable behavior |
| FinalizationRegistry | Later compatibility feature; unsupported in early phases until semantics fit the no-tracing-GC design |
| Global objects | Static lifetime, no counting |

### `eval` and Dynamic Features
When `eval` or `Function()` is used:
- All local variables in scope are conservatively heap-allocated
- Ownership defaults to shared heap representation with deterministic reference counting
- This is flagged as a performance warning by the compiler
- The sandbox system can prohibit `eval` entirely
- Full runtime `eval` semantics are only required in Phase 4 (see [specs/10-runtime.md](10-runtime.md))

## Memory Safety

Even without a GC, Kali guarantees memory safety:
- No use-after-free: lifetime analysis prevents dangling pointers
- No buffer overflows: bounds checks on array access (elided when provably safe)
- No uninitialized reads: all variables initialized before use (compiler-enforced)
- WASM's inherent sandboxing provides a safety net for linear memory access
