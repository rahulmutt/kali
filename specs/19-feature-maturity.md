# 19 — Feature Maturity

This document is the canonical matrix for features that are easy to describe inconsistently across architecture, runtime, package, and CLI specs.

If another spec needs to mention one of these features, it should link here for phase/status rather than restating a different maturity decision.

## Status Labels

- **Phase 1 MVP** — required for the first practically useful implementation
- **Phase 2 target** — planned once ownership/effects infrastructure lands
- **Phase 3 target** — planned once specialization/ecosystem work lands
- **Phase 4 compatibility** — supported only in the advanced compatibility phase
- **Later compatibility** — intentionally deferred until semantics and cost are justified
- **Opt-in only** — supported only behind an explicit flag or config
- **Rejected by default** — parser may accept the syntax, but compile/run should fail unless a later compatibility path is enabled

## Canonical Matrix

| Feature | Status | Rationale |
|---|---|---|
| Static ESM `import` / `export` | Phase 1 MVP | Core module system |
| CommonJS module lowering | Phase 1 MVP | Needed for broad npm package compatibility |
| `require("literal")` | Phase 1 MVP | Rewritten during compilation when statically resolvable |
| Dynamic `require()` | Rejected by default | Conflicts with the early single-linked-artifact model |
| Literal-string `import()` | Phase 3 target | Can be lowered to the already-linked graph without runtime WASM module linking |
| Non-literal `import(expr)` | Later compatibility | Requires a dynamic host-mediated path and conservative effect handling |
| `eval` | Phase 4 compatibility | Parsed and effect-tracked earlier, but full runtime support is deferred |
| `Function()` constructor | Phase 4 compatibility | Same status as `eval` |
| Built-in effect inference / `kali effects` | Phase 2 target | Required for sandbox-first analysis and policy checking |
| Explicit effect annotations / `pure` | Phase 2 target | Initially scoped to the built-in sandbox capability model |
| Algebraic effect declarations / handlers | Later compatibility | Experimental and must not block delivery of the core capability/effect system |
| `Proxy` | Later compatibility | High semantic cost and optimization barriers |
| `WeakMap` / `WeakSet` | Later compatibility | Deferred until weak-reference semantics fit the no-tracing-GC design |
| `FinalizationRegistry` | Later compatibility | Same reason as weak collections |
| `SharedArrayBuffer` / `Atomics` | Opt-in only | Requires WASM threads and a different runtime profile |
| npm lifecycle scripts | Opt-in only | Disabled by default for sandbox-first behavior |
| Native addons / `node-gyp` packages | Rejected by default | Violates the pure-Rust/no-native-addon constraints |
| DOM APIs in standalone runtime | Rejected by default | Kali does not embed a browser engine |

## Interpretation Rules

1. **Single-artifact rule**: Phase 1-3 builds target one linked WASM artifact for the resolved static graph.
2. **Parse vs support**: accepted syntax does not imply full runtime support; unsupported dynamic features should be diagnosed explicitly.
3. **Effect boundaries**: features marked as dynamic compatibility paths should be reflected in static effect analysis.
4. **No silent fallback**: if a feature cannot be implemented faithfully under the current phase constraints, Kali should reject or gate it rather than emulate it loosely.

## Features Most Likely to Appear in Diagnostics

The compiler should produce clear, stable diagnostics for these cases:
- dynamic `require()` in early phases
- non-literal `import(expr)` in early phases
- `eval` / `Function()` without explicit compatibility mode
- `Proxy` usage in unsupported runtime modes
- weak-reference APIs before their semantics are implemented

See also:
- [SPEC.md](../SPEC.md)
- [01 — Architecture](01-architecture.md)
- [10 — Runtime](10-runtime.md)
- [14 — Package Management](14-packages.md)
