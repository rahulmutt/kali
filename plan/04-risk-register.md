# Active Risk Register

| Risk | Why it matters | Mitigation |
|---|---|---|
| Browser-targeted support is confused with standalone browser runtime | Overclaims `run/test --api browser` and sandbox enforcement after deployment | Keep Phase 22 browser-runtime work separate from browser bundle/check support; require explicit host contract and browser-harness evidence |
| Node package successes become blanket npm claims | Package support depends on shape, host fit, command maturity, and rung | Phase 23 package reports must name exact rungs and contexts |
| Thread support weakens no-GC/no-JIT invariants | Threads interact with memory, host APIs, and resource budgets | Gate threaded semantics by profile and target; require ownership/resource tests and schema updates |
| PGO becomes a hidden fourth build mode | Spec preserves `fast` / `release` / `release-advanced` vocabulary | Treat `--profile` as additive build input only; require deterministic schema |
| Optimization changes JavaScript-visible behavior | Aggressive specialization can break semantics | Phase 24 requires conformance regressions and deterministic artifact comparison |
| Proof claims drift beyond mechanized scope | The project is proof-backed only for the published boundary | Widen `proofs/BOUNDARY.md` only with theorem inventory and proof CI triggers |
| Package lifecycle hooks are misread as runtime support | `--allow-scripts` is install-time only | Keep install evidence separate from build/run/test package support |
| Native/binary package pressure erodes pure Rust/sandbox contracts | Unsupported install/runtime paths can introduce opaque behavior | Keep rejected-by-default posture unless specs deliberately introduce a mediated path |
| Plan files become implementation journals | Long progress logs obscure remaining work | Keep active plan docs concise and move exact support/evidence truth to specs, tests, and proof boundary |
