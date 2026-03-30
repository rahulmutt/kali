# 06 — Memory Management

## Principle

Kali has **no tracing garbage collector**. Allocation class and ownership strategy are chosen statically where possible through ownership analysis, lifetime inference, and escape analysis. The compiler may still insert deterministic runtime bookkeeping such as reference-count increments/decrements when sharing cannot be eliminated. This is inspired by Rust's ownership model, adapted for TypeScript/JavaScript semantics.

For compatibility-heavy APIs and dynamic features mentioned in this document, the canonical maturity/status matrix lives in [specs/19-feature-maturity.md](19-feature-maturity.md). This section focuses on memory strategy, not on redefining feature phase decisions.

For cross-spec consistency, the canonical representation-downgrade ladder also lives in [SPEC.md](../SPEC.md). This chapter provides the memory-specific consequences of those downgrades.

## Allocation Strategy Decision

For every value, the compiler determines:

1. **Where** it lives: stack or heap
2. **How** it's owned: unique, shared (reference-counted), or borrowed
3. **When** it's freed: scope exit, last use, or ref-count drop to zero

### Decision Flow

```
Value Created
  ├─ Primitive (number, boolean, null, undefined)
  │   → Prefer unboxed WASM locals/operand-stack representation while they stay local
  │   → Box/store only when crossing a boundary that requires heap/object representation
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
- **Escapes via global/module state**: stored in module-level variable → shared heap with deterministic reference counting

### Closure Analysis
Closures capture variables. For each capture:
- If the variable is only read and the closure doesn't outlive the variable → borrow
- If the variable is mutated or the closure escapes → lower to a shared heap cell with deterministic reference counting and interior mutability semantics
- If only one closure captures it and ownership can transfer → move
- Captured primitives may therefore stop being purely local/unboxed values once closure or aggregate boundaries require boxing/shared storage; the earlier “prefer unboxed locals” rule is only for non-escaping use sites

## Reference Counting

When shared ownership is needed, Kali uses compile-time-inserted deterministic reference counting:

### Compile-Time Ownership-Class Rule
To keep the bootstrap's “decide heap/stack/shared ownership at compile time” goal aligned with the rest of the spec:
- the compiler chooses the ownership class (`stack`, `owned heap`, `shared heap`, or `borrowed`) during analysis/lowering rather than leaving that choice to an opaque runtime policy
- inserted reference-count operations are a consequence of selecting the `shared heap` class, not a hidden fallback memory mode
- bounded deterministic cycle cleanup, when present, is only reclamation bookkeeping over values that were **already** lowered into shared-heap ownership; it does not retroactively change ownership classes or reintroduce tracing GC
- later dynamic-compatibility barriers such as `--compat eval` may force more values into the conservative `shared heap` class, but that is still an explicit compile/lowering decision visible to optimization and diagnostics rather than a background collector taking over

```rust
struct SharedHeader {
    ref_count: u32,
    // Reserved for internal cycle-reclamation bookkeeping when enabled
    aux_count: u32,
    // Type tag for runtime type checks (when needed)
    type_tag: u16,
    // Flags (e.g., frozen, sealed)
    flags: u16,
}
```

The header layout above is illustrative. The important contract is the ownership model, not the exact field names or bit layout.

### Shared-Ownership Optimizations
- **Elide inc/dec pairs**: When a borrow is provably temporary, skip ref counting
- **Move optimization**: Transfer ownership without inc+dec (just dec the source)
- **Static ref**: Objects known to live for program lifetime skip counting entirely
- **Batch dec**: When a scope exits with multiple shared values, batch the decrements
- **Inline ref count**: Small objects embed the count in their header (no indirection)

### Cycle Handling
JavaScript allows ordinary strong-reference cycles (e.g., `a.b = b; b.a = a`), so Kali needs an explicit strategy even without a tracing GC.

Canonical no-GC boundary:
- Kali does **not** introduce a general tracing/background collector as a hidden fallback for ordinary execution
- any cycle cleanup must remain **deterministic, bounded, and semantically invisible** bookkeeping over already-shared regions
- if Kali cannot justify such cleanup for a case/profile yet, the implementation may conservatively retain the cycle until region/sandbox teardown rather than silently weakening the no-tracing design

Early simplification:
1. **Prefer acyclic ownership when provable**: keep stack or unique-ownership layouts when escape analysis can prove they are sufficient
2. **Shared-heap fallback for cyclic graphs**: when objects must be shared, lower them to shared-heap ownership without changing their logical object layout unless other dynamic features force that too
3. **Bounded deterministic cycle cleanup**: shared regions may use trial-deletion/local reclamation techniques when ordinary ref counting cannot reclaim a cycle, but only as internal non-tracing housekeeping
4. **Sandbox/region teardown**: short-lived runtime instances may reclaim whole regions when the sandbox/program exits
5. **Debug leak reporting**: in debug mode, report unreclaimed shared cycles at shutdown with source locations where possible

Important separation rule:
- this is an **internal memory-management strategy** for ordinary object graphs
- it does **not** imply that JavaScript weak-reference APIs (`WeakMap`, `WeakSet`, `FinalizationRegistry`) are available early; those remain **Later compatibility** features as defined in [specs/19-feature-maturity.md](19-feature-maturity.md)
- it also does **not** imply movable GC, stop-the-world tracing, or user-visible finalization semantics in early phases

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
| Closures capturing variables | Shared heap cell with deterministic reference counting for escaping mutable captures |
| Circular references | Shared-heap fallback plus bounded deterministic non-tracing cycle cleanup, with region teardown as the conservative fallback |
| `arguments` object | Stack-allocated array when non-escaping; heap otherwise |
| Prototype chains | Static when class hierarchy is known; shared heap object for dynamic cases |
| WeakMap/WeakSet | Later-phase compatibility feature; unsupported in early phases until weak-reference semantics are specified without violating observable behavior |
| FinalizationRegistry | Later compatibility feature; unsupported in early phases until semantics fit the no-tracing-GC design |
| Global objects | Static lifetime, no counting |

### `eval` and Dynamic Features
`eval` and `Function()` are part of the later `--compat eval` path, not an early-phase execution feature.

Interpretation rule:
- **Phases 1-3**: Kali may parse and effect-track these forms, but normal execution still rejects them through the canonical availability path.
- **Phase 4 compatibility path**: once `--compat eval` exists and is enabled, the surrounding region becomes a conservative deoptimization/ownership barrier.

For that later compatibility path:
- all directly reachable locals in scope are conservatively heap-allocated/boxed
- ownership defaults to shared-heap representation with deterministic reference counting
- this is flagged as a performance warning by the compiler
- the sandbox system can still prohibit `eval` entirely
- full runtime `eval` semantics are only required in Phase 4 (see [specs/10-runtime.md](10-runtime.md))

## Memory Safety

Even without a GC, Kali guarantees memory safety:
- No use-after-free: lifetime analysis prevents dangling pointers
- No buffer overflows: bounds checks on array access (elided when provably safe)
- No uninitialized reads: all variables initialized before use (compiler-enforced)
- WASM's inherent sandboxing provides a safety net for linear memory access

Terminology note:
- `stack`, `owned heap`, `shared heap`, and `borrowed` are the canonical ownership categories across the memory, IR, optimization, and testing specs
- avoid using `Rc` as a separate semantic category in the spec set; it is an implementation technique for `shared heap`, not a user-facing model
