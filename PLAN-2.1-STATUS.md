# Stage 2.1 Status Update

**Date:** 2026-04-12  
**Status:** 🟡 In progress — MIR escape-analysis coverage tightened for call arguments

## Summary

MIR ownership analysis now treats call arguments as escaping values, so conservative intra-procedural analysis can classify locals that flow into unknown callees more accurately. The existing return/capture classification remains intact and the full workspace test suite still passes.

## Evidence

- `cargo test -p kali_mir --quiet` ✅
- `cargo test --workspace --quiet` ✅

## Notable Deliverables

- Call-argument escape tracking now marks bindings as escaping when they flow into unknown call sites
- Return/capture ownership classification remains unchanged
- The stage still has a frontend/HIR normalization follow-up for precise heap-store tracking; see `PLAN-MAILBOX.md`

## Next Step

Normalize the HIR shape for heap-store / object-literal cases, then revisit the remaining Stage 2.1 escape-analysis coverage.
