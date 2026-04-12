# Stage 3.1 Status Update

**Date:** 2026-04-12  
**Status:** 🟡 Optimization scaffolding and specialization-cap plumbing landed for the Phase-3 pipeline

## Summary

Stage 3.1 now has the first real `kali_optimize` implementation wired into the build pipeline. `release` builds perform deterministic constant folding and branch elimination, while `release-advanced` adds a small algebraic-identity pass that can remove extra add/sub/mul overhead from hot paths. The CLI build path now invokes the optimizer before WASM codegen, the build command accepts `--max-specializations` as a specialization-budget override, and the incremental cache key now incorporates that cap so different budgets do not collide. The workspace test suite remains green.

## Evidence

- `kali_optimize` now rewrites tree-shaped LIR in place ✅
- `release` folds literal expressions and constant branches before codegen ✅
- `release-advanced` adds algebraic simplifications such as `x + 0 -> x` ✅
- CLI runtime smoke tests now compare `fast`, `release`, and `release-advanced` instruction counts ✅
- `--max-specializations` now flows through the build pipeline and participates in deterministic cache keys ✅
- Repeated builds now populate and reuse `.kali-cache/incremental/` for unchanged modules ✅
- `cargo test --workspace` passes ✅

## Notable Deliverables

- `crates/kali_optimize/src/lib.rs` now contains the first real optimizer implementation instead of the previous no-op placeholder
- `crates/kali_cli/src/build.rs` now calls the optimizer before lowering LIR to WASM
- `crates/kali_lir/src/lib.rs` exposes `into_nodes()` so the optimizer tests can construct deterministic programs from builders
- CLI smoke tests now assert that `release` removes literal add chains and `release-advanced` removes the `+ 0` identity case

## Current Limits

- Generic/function/layout specialization has not been implemented yet.
- `release` / `release-advanced` still rely on the current LIR-level pass set rather than the later MIR-driven specialization model.
- The optimizer now respects a deterministic specialization budget for distinct optimization shapes, but this is still a placeholder for the richer specialization planner described in the long-term stage plan.

## Next Step

Continue Stage 3.1 by adding actual specialization/planning data structures and broader release optimization coverage so the remaining DoD items can be closed.
