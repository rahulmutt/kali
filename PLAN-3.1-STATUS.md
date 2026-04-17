# Stage 3.1 Status Update

**Date:** 2026-04-12  
**Status:** 🟡 Optimization scaffolding and specialization-cap plumbing landed for the Phase-3 pipeline; hot-path smoke coverage now also checks that optimized numeric paths stay unboxed

## Summary

Stage 3.1 now has the first real `kali_optimize` implementation wired into the build pipeline. `release` builds perform deterministic constant folding, branch elimination, and small-call inlining, while `release-advanced` adds algebraic-identity simplification plus dead top-level function pruning after inlining. The CLI build path now invokes the optimizer before WASM codegen, the build command accepts `--max-specializations` as a specialization-budget override, and the incremental cache key now incorporates that cap so different budgets do not collide. The specialization budget is now enforced per function owner, so separate hot paths keep independent caps while the code-size guard still blocks runaway fan-out. The workspace test suite remains green.

## Evidence

- `kali_optimize` now rewrites tree-shaped LIR in place ✅
- `release` folds literal expressions and constant branches before codegen ✅
- `release-advanced` adds algebraic simplifications such as `x + 0 -> x` ✅
- `release` now inlines small function bodies, and `release-advanced` prunes dead top-level functions after those inlines land ✅
- CLI runtime smoke tests now compare `fast`, `release`, and `release-advanced` instruction counts ✅
- CLI runtime smoke tests now also assert that a specialized numeric hot path emits no tag-check / untag boxing operators ✅
- `--max-specializations` now flows through the build pipeline and participates in deterministic cache keys ✅
- Specialization caps are scoped per function owner, and regression tests cover both shared-root and independent-function budgets ✅
- Repeated builds now populate and reuse `.kali-cache/incremental/` for unchanged modules ✅
- `cargo test --workspace` passes ✅

## Notable Deliverables

- `crates/kali_optimize/src/lib.rs` now contains a specialization plan, small-function inlining, and aggressive top-level dead-function pruning instead of the previous no-op placeholder
- The runtime smoke suite now guards the optimized hot path against tag-check / untag boxing regressions while still verifying numeric codegen shape
- `crates/kali_cli/src/build.rs` now calls the optimizer before lowering LIR to WASM
- `crates/kali_lir/src/lib.rs` exposes `into_nodes()` so the optimizer tests can construct deterministic programs from builders
- CLI smoke tests now assert that `release` removes literal add chains, `release` inlines simple call sites, and `release-advanced` prunes dead inlined helpers

## Current Limits

- Full generic/function/layout specialization is still pending; the current work only covers small-function call-site specialization and pruning.
- `release` / `release-advanced` still rely on the current LIR-level pass set rather than the later MIR-driven specialization model.
- The optimizer now respects a deterministic specialization budget for distinct optimization shapes, scoped per function owner, but the richer MIR-driven specialization planner described in the long-term stage plan is still ahead.

## Next Step

Continue Stage 3.1 by broadening from scalar call-site specialization to the full generic/layout specialization model and richer MIR-driven planning so the remaining phase-3 breadth targets can be closed.
