# Stage 3.1 Status Update

**Date:** 2026-04-18  
**Status:** 🟡 Optimization scaffolding and specialization-cap plumbing now includes a representative benchmark suite; MIR-aware call-site specialization now clones larger layout-stable callees, reoptimizes the specialized bodies after substitution, and now also reuses shared closure- and struct-valued MIR bindings across identical call-site layouts while still keeping optimized numeric paths unboxed and const-bound object/array layouts folded through member access in the optimizer

## Summary

Stage 3.1 now has the first real `kali_optimize` implementation wired into the build pipeline. `release` builds perform deterministic constant folding, branch elimination, and small-call inlining, while `release-advanced` adds algebraic-identity simplification plus dead top-level function pruning after inlining. The CLI build path now invokes the optimizer before WASM codegen, the build command accepts `--max-specializations` as a specialization-budget override, and the incremental cache key now incorporates that cap so different budgets do not collide. The specialization budget is now enforced per function owner, so separate hot paths keep independent caps while the code-size guard still blocks runaway fan-out. A new layout-aware prepass now tracks const-bound object literals and array literals, folding property reads and constant-index element reads to their statically known values before codegen, and the new MIR-driven specialization pass now clones larger functions when their MIR parameter layouts are stable enough to justify partial substitution, then re-runs optimization on the specialized clone so literal-heavy hot paths can fold further. The MIR-aware layout signature path now also recognizes shared closure- and struct-valued bindings, so identical higher-order call-site layouts can reuse one specialized clone instead of fanning out per binding name. A representative benchmark suite now captures compile time, WASM size, and instruction-count regressions across `fast`, `release`, and `release-advanced`, and the workspace test suite remains green.

## Evidence

- `kali_optimize` now rewrites tree-shaped LIR in place ✅
- `release` folds literal expressions and constant branches before codegen ✅
- `release-advanced` adds algebraic simplifications such as `x + 0 -> x` ✅
- `release` now inlines small function bodies, and `release-advanced` prunes dead top-level functions after those inlines land ✅
- The optimizer now folds const-bound object property reads to the corresponding literal field values before codegen ✅
- The optimizer now also folds const-bound array element reads when the index is statically known or bound to a constant numeric value ✅
- MIR-aware call-site specialization now clones larger layout-stable callees, reoptimizes the specialized clone, and preserves the existing call target shape for codegen ✅
- MIR-aware layout signatures now also collapse shared closure- and struct-valued bindings to a single specialization when the call-site layout is otherwise identical ✅
- CLI runtime smoke tests now compare `fast`, `release`, and `release-advanced` instruction counts ✅
- CLI runtime smoke tests now also assert that a specialized numeric hot path emits no tag-check / untag boxing operators ✅
- A representative optimization benchmark now records compile time, WASM size, instruction count, and add-op deltas across the three build modes ✅
- `--max-specializations` now flows through the build pipeline and participates in deterministic cache keys ✅
- Specialization caps are scoped per function owner, and regression tests cover both shared-root and independent-function budgets ✅
- Repeated builds now populate and reuse `.kali-cache/incremental/` for unchanged modules ✅
- `cargo test --workspace` passes ✅

## Notable Deliverables

- `crates/kali_optimize/src/lib.rs` now contains a specialization plan, small-function inlining, MIR-aware call-site specialization, and aggressive top-level dead-function pruning instead of the previous no-op placeholder
- The runtime smoke suite now guards the optimized hot path against tag-check / untag boxing regressions while still verifying numeric codegen shape
- `crates/kali_cli/src/build.rs` now calls the optimizer before lowering LIR to WASM
- `crates/kali_lir/src/lib.rs` exposes `into_nodes()` so the optimizer tests can construct deterministic programs from builders
- CLI smoke tests now assert that `release` removes literal add chains, `release` inlines simple call sites, and `release-advanced` prunes dead inlined helpers
- MIR-aware specialization now has shared closure-layout and struct-layout regression tests so identical higher-order call-site layouts reuse one specialized clone

## Current Limits

- The richer generic-instantiation planner is still ahead; the current work now covers small-function call-site specialization plus MIR-annotated layout-stable clones, shared closure- and struct-layout signatures, const-bound object and array layout folding, and pruning, but not the broader cross-module instantiation model.
- `release` / `release-advanced` still rely on the current LIR-level pass set rather than a full MIR-to-specialized-WASM pipeline.
- The optimizer now respects a deterministic specialization budget for distinct optimization shapes, scoped per function owner, and the MIR-aware clone path still stays inside that cap while the broader planner work remains ahead.

## Next Step

Continue Stage 3.1 by broadening MIR-aware specialization from the current layout-stable clone path toward the fuller generic-instantiation planner and cross-module specialization model, especially around richer closure/struct layout sharing, so the remaining phase-3 breadth targets can be closed.
