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
- **Rejected by default** — parser may accept the syntax, but compile/run should fail unless the documented compatibility switch is enabled in a phase that actually implements the feature

## Canonical Matrix

| Feature | Status | Rationale |
|---|---|---|
| Static ESM `import` / `export` | Phase 1 MVP | Core module system |
| CommonJS module lowering | Phase 1 MVP | Needed for broad npm package compatibility |
| `require("literal")` | Phase 1 MVP | Rewritten during compilation when statically resolvable |
| Dynamic `require()` | Rejected by default | Conflicts with the early single-linked-artifact model |
| Literal-string `import()` | Phase 3 target | Can be lowered to the already-linked graph without runtime WASM module linking |
| Non-literal `import(expr)` | Later compatibility | Requires a dynamic host-mediated path and conservative effect handling |
| `eval` | Phase 4 compatibility | Parsed and effect-tracked earlier, but full runtime support is deferred; compatibility path is `--compat eval` when implemented |
| `Function()` constructor | Phase 4 compatibility | Same status as `eval` and uses the same compatibility switch |
| Built-in effect inference / `kali effects` | Phase 2 target | Required for sandbox-first analysis and policy checking |
| Explicit effect annotations / `pure` | Phase 2 target | Initially scoped to the built-in sandbox capability model |
| Algebraic effect declarations / handlers | Later compatibility | Experimental and must not block delivery of the core capability/effect system |
| Sandbox validator functions | Later compatibility | Initial policies stay declarative; host-registered pure validators may be added later for embedding scenarios |
| `Proxy` | Later compatibility | High semantic cost and optimization barriers |
| `WeakMap` / `WeakSet` | Later compatibility | Deferred until weak-reference semantics fit the no-tracing-GC design |
| `FinalizationRegistry` | Later compatibility | Same reason as weak collections |
| `SharedArrayBuffer` / `Atomics` | Opt-in only | Requires WASM threads and a different runtime profile |
| `--wasm-threads` | Opt-in only | Enables the threaded runtime profile; must fail explicitly on unsupported targets/engines |
| `--api browser` for `check` / `build --bundle` | Phase 1 MVP | Browser-targeted analysis/build without claiming DOM support in the standalone runtime |
| `run --api browser` | Rejected by default | Early standalone runtime does not emulate a browser host |
| npm lifecycle scripts | Opt-in only | Disabled by default for sandbox-first behavior |
| Native addons / `node-gyp` packages | Rejected by default | Violates the pure-Rust/no-native-addon constraints |
| npm packages that require unsupported Node core modules | Phase 3 target | Depends on broader `--api node` compatibility work |
| DOM APIs in standalone runtime | Rejected by default | Kali does not embed a browser engine |

## Interpretation Rules

1. **Single-artifact rule**: Phase 1-3 builds target one linked WASM artifact for the resolved static graph.
2. **Parse vs support**: accepted syntax does not imply full runtime support; unsupported dynamic features should be diagnosed explicitly.
3. **Effect boundaries**: features marked as dynamic compatibility paths should be reflected in static effect analysis.
4. **No silent fallback**: if a feature cannot be implemented faithfully under the current phase constraints, Kali should reject or gate it rather than emulate it loosely.
5. **Canonical gating diagnostic**: use the shared feature-maturity diagnostic contract (`E5006`) so CLI, checker, runtime, and package tooling report phase/profile gating consistently.

## Canonical Command/Profile Matrix

This table exists to stop drift between CLI examples, runtime behavior, package tooling, and error reporting.

| Command / profile | Early-phase status | Canonical handling |
|---|---|---|
| `kali run main.ts` | Phase 1 MVP | Compile and execute with default `--api deno` profile |
| `kali run --api deno main.ts` | Phase 1 MVP | Supported standalone runtime path |
| `kali run --api node main.ts` | Phase 3 target | Reject with `E5006` until the documented Node subset lands |
| `kali run --api browser main.ts` | Rejected by default | Reject with `E5006`; browser is a check/build profile first |
| `kali check --api browser main.ts` | Phase 1 MVP | Supported browser-targeted analysis/profile |
| `kali build --bundle --api browser main.ts` | Phase 1 MVP | Supported browser artifact path (`.wasm` + JS glue) |
| `kali effects main.ts` | Phase 2 target | Before then: unavailable or explicitly experimental, never a partial bespoke report |
| `kali package-effects lodash` | Phase 2 target | Depends on effect-report pipeline; reject/mark experimental before then |
| `--compat eval` | Phase 4 compatibility | Before runtime support exists, reject with `E5006` rather than parsing and silently ignoring the flag |
| `--wasm-threads` | Opt-in only | Supported only with the threaded runtime profile; reject explicitly when unavailable |

## Features Most Likely to Appear in Diagnostics

The compiler should produce clear, stable diagnostics for these cases, using the canonical `E5006` shape from [specs/15-errors.md](15-errors.md) unless a stricter subsystem-specific error is more informative:
- dynamic `require()` in early phases
- non-literal `import(expr)` in early phases
- `eval` / `Function()` without `--compat eval`
- sandbox validator functions before the documented embedding-only compatibility path exists
- `Proxy` usage in unsupported runtime modes
- weak-reference APIs before their semantics are implemented
- `--api node` or browser-only assumptions outside the documented profile
- `--wasm-threads` on targets/profiles that do not support the threaded runtime

## Canonical Early-Phase Handling

To reduce drift between CLI, runtime, package, and error-reporting specs, unsupported or gated features should follow this table unless a later spec explicitly tightens the behavior.

| Feature | Parse support | Early-phase semantic handling |
|---|---|---|
| dynamic `require()` | Yes | Reject by default with a feature-maturity diagnostic |
| non-literal `import(expr)` | Yes | Reject by default; mark as a dynamic effect boundary when analyzed |
| literal-string `import()` | Yes | Parse early; enable only once lowered to the already-linked graph |
| `eval` / `Function()` | Yes | Report `Eval` effect; reject by default unless `--compat eval` is enabled and the runtime phase supports it |
| `pure` / explicit effect annotations | Yes | Parse early; checker enables and validates in Phase 2+ |
| `Proxy` | Yes | Type-check where possible, but reject unsupported runtime lowering paths |
| `WeakMap` / `WeakSet` / `FinalizationRegistry` | Yes | Reject or gate until faithful semantics are implemented |

This table intentionally separates syntax acceptance from semantic support so the parser and AST can stay broad without forcing premature runtime commitments.

See also:
- [SPEC.md](../SPEC.md)
- [01 — Architecture](01-architecture.md)
- [10 — Runtime](10-runtime.md)
- [14 — Package Management](14-packages.md)
