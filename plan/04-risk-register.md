# Active Risk Register

| Risk | Why it matters | Mitigation |
|---|---|---|
| Browser-targeted support is confused with standalone browser runtime | Overclaims `run/test --api browser` and sandbox enforcement after deployment | Keep Phase 7 browser-runtime work separate from existing browser bundle/check support; require real browser harness evidence |
| Node package successes become blanket npm claims | Package support depends on shape, host fit, command maturity, and rung | Phase 8 package reports must name exact rungs and contexts |
| Thread support weakens no-GC/no-JIT invariants | Threads interact with memory, host APIs, and resource budgets | Gate `--wasm-threads`; require explicit ownership/resource tests and schema updates |
| PGO becomes a hidden fourth build mode | Spec preserves `fast` / `release` / `release-advanced` vocabulary | Treat `--profile` as additive build input only; require deterministic schema |
| Optimization changes JavaScript-visible behavior | Aggressive specialization can break semantics | Phase 9 requires conformance regressions and deterministic artifact comparison |
| Proof claims drift beyond mechanized scope | The project already has proof-backed wording for a narrow boundary | Widen `proofs/BOUNDARY.md` only with theorem inventory and proof CI triggers |
| Package lifecycle hooks are misread as runtime support | `--allow-scripts` is install-time only | Keep install evidence separate from build/run/test package support |
| Native/binary package pressure erodes pure Rust/sandbox contracts | Unsupported install/runtime paths can introduce opaque behavior | Keep rejected-by-default posture unless specs deliberately introduce a mediated path |
