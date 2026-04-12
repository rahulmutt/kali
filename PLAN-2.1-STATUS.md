# Stage 2.1 Status Update

**Date:** 2026-04-12  
**Status:** 🟡 In progress — HIR object-literal normalization landed; MIR escape-analysis now sees stable heap-store shapes

## Summary

MIR ownership analysis now treats call arguments as escaping values, and the frontend now lowers object-literal properties into a dedicated composite `ObjectProperty` HIR node with literal keys. That gives the escape analyzer a stable heap-store shape to reason about, so values flowing into object literals are classified conservatively without mistaking property names for bindings. The existing return/capture classification remains intact and the full workspace test suite still passes.

## Evidence

- `cargo test -p kali_mir --quiet` ✅
- `cargo test --workspace --quiet` ✅

## Notable Deliverables

- Call-argument escape tracking now marks bindings as escaping when they flow into unknown call sites
- Object-literal properties now lower through a dedicated `ObjectProperty` HIR node with literal keys
- Return/capture ownership classification remains unchanged
- Heap-store tracking is now unblocked by frontend/HIR normalization; see `PLAN-MAILBOX.md`

## Next Step

Continue broadening the remaining Stage 2.1 escape-analysis coverage beyond call/return/object-store flows.
