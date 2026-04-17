# Stage 2.1 Status Update

**Date:** 2026-04-12  
**Status:** ✅ Complete — MIR ownership analysis now covers stable heap-store shapes, array/member-store flows, inline function-expression call sites, and aliased function-expression call chains

## Summary

MIR ownership analysis now treats call arguments as escaping values for unknown or unresolved callees, and it now distinguishes inline function-expression call sites, direct aliases of function expressions, and alias chains that resolve back to the same lowered function target. The frontend also lowers object-literal properties into a dedicated composite `ObjectProperty` HIR node with literal keys. That gives the escape analyzer a stable heap-store shape to reason about, so values flowing into object literals are classified conservatively without mistaking property names for bindings. Array-element and member-assignment flows now extend that same conservative treatment, and closure bindings retain the concrete capture list in their layout descriptor so the environment shape stays explicit. The existing return/capture classification remains intact and the full workspace test suite still passes.

## Evidence

- `cargo test -p kali_mir --quiet` ✅
- `cargo test --workspace --quiet` ✅

## Notable Deliverables

- Call-argument escape tracking now marks bindings as escaping when they flow into unknown call sites
- Inline function-expression calls now reuse the callee's parameter escape summary so non-escaping parameters can stay stack-allocated
- Direct aliases of function expressions now resolve to the same lowered function target, even for anonymous function expressions
- Alias chains such as `const g = f; const h = g; h(...)` now preserve direct-callee escape precision
- Array-element and member-assignment escape tracking now classify heap-store flows conservatively
- Object-literal properties now lower through a dedicated `ObjectProperty` HIR node with literal keys
- Closure bindings now retain explicit capture lists in their closure layout descriptors
- Return/capture ownership classification remains unchanged
- Heap-store tracking is now unblocked by frontend/HIR normalization; see `PLAN-MAILBOX.md`

## Next Step

Begin the next Phase 2 workstreams with Stage 2.2 and Stage 2.3; Stage 2.1 is now complete.
