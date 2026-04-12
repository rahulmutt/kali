# Stage 3.1 Status Update

**Date:** 2026-04-12  
**Status:** 🟡 Optimization scaffolding landed for the Phase-3 pipeline

## Summary

Stage 3.1 now has the first real `kali_optimize` implementation wired into the build pipeline. `release` builds perform deterministic constant folding and branch elimination, while `release-advanced` adds a small algebraic-identity pass that can remove extra add/sub/mul overhead from hot paths. The CLI build path now invokes the optimizer before WASM codegen, and the workspace test suite remains green.

## Evidence

- `kali_optimize` now rewrites tree-shaped LIR in place ✅
- `release` folds literal expressions and constant branches before codegen ✅
- `release-advanced` adds algebraic simplifications such as `x + 0 -> x` ✅
- CLI runtime smoke tests now compare `fast`, `release`, and `release-advanced` instruction counts ✅
- `cargo test --workspace` passes ✅

## Notable Deliverables

- `crates/kali_optimize/src/lib.rs` now contains the first real optimizer implementation instead of the previous no-op placeholder
- `crates/kali_cli/src/build.rs` now calls the optimizer before lowering LIR to WASM
- `crates/kali_lir/src/lib.rs` exposes `into_nodes()` so the optimizer tests can construct deterministic programs from builders
- CLI smoke tests now assert that `release` removes literal add chains and `release-advanced` removes the `+ 0` identity case

## Current Limits

- Generic/function/layout specialization has not been implemented yet.
- Incremental compilation still remains a later Stage 3 follow-up.
- The optimizer is still tree-local and does not yet model the full MIR/LIR pass pipeline described in the long-term stage plan.

## Next Step

Continue Stage 3.1 by adding actual specialization/planning data structures and broader incremental/release optimization coverage so the remaining DoD items can be closed.
