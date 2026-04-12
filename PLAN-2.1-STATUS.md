# Stage 2.1 Status Update

**Date:** 2026-04-12  
**Status:** 🟡 In progress — MIR escape-analysis now sees stable heap-store shapes, broader array/member-store escape flows, inline function-expression call-site refinement, and aliased function-expression direct-call precision

## Summary

MIR ownership analysis now treats call arguments as escaping values for unknown or unresolved callees, and it now distinguishes both inline function-expression call sites and aliased function-expression call sites whose parameter bindings stay non-escaping. The frontend also lowers object-literal properties into a dedicated composite `ObjectProperty` HIR node with literal keys. That gives the escape analyzer a stable heap-store shape to reason about, so values flowing into object literals are classified conservatively without mistaking property names for bindings. Array-element and member-assignment flows now extend that same conservative treatment. The existing return/capture classification remains intact and the full workspace test suite still passes.

## Evidence

- `cargo test -p kali_mir --quiet` ✅
- `cargo test --workspace --quiet` ✅

## Notable Deliverables

- Call-argument escape tracking now marks bindings as escaping when they flow into unknown call sites
- Inline function-expression calls now reuse the callee's parameter escape summary so non-escaping parameters can stay stack-allocated
- Aliased function-expression calls now reuse the same parameter escape summary, so `const f = function (...) { ... }; f(...)` calls can stay precise
- Array-element and member-assignment escape tracking now classify heap-store flows conservatively
- Object-literal properties now lower through a dedicated `ObjectProperty` HIR node with literal keys
- Return/capture ownership classification remains unchanged
- Heap-store tracking is now unblocked by frontend/HIR normalization; see `PLAN-MAILBOX.md`

## Next Step

Continue broadening the remaining Stage 2.1 escape-analysis coverage, especially any remaining nested closure-shape edge cases or other non-local call targets, before marking the stage complete.
