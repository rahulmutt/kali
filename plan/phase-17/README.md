# Phase 17 — Host/Runtime Contract Expansion

## Goal

Expand runtime and host capability only where Kali can mediate, test, and describe it honestly.

## Owning specs

- `specs/09-sandboxing.md`
- `specs/10-runtime.md`
- `specs/11-standard-apis.md`
- `specs/12-cli.md`
- `specs/18-schemas.md`
- `specs/19-feature-maturity.md`

## Work packets

### 17.1 Threaded runtime semantics

- Complete guest-facing threaded behavior beyond profile acceptance and helper plumbing.
- Define valid positive thread budgets by command, API surface, and runtime profile.
- Preserve AOT-only compilation, no tracing/background GC, deterministic JSON, and resource-limit honesty.
- Current progress: the positive `--max-threads` rejection path now carries both the resource-budget and threaded-profile config hints in text diagnostics, while the JSON error payload keeps the canonical `resources.maxThreads` message stable and now also includes structured CLI context for the explicit `--max-threads` request; runtime smoke now also exercises guest thread spawning through the host import path, including budget-exhaustion rejection for a second spawn under a one-thread limit, and browser-requested run/test harness smoke now also accepts positive `--max-threads` overrides when `--wasm-threads` is active in JS/TS/JSX/TSX input with JSON-output coverage, while browser-targeted `check` / `build --bundle` rejection coverage for the same threaded-profile gate now also mirrors JSX and TSX input on both explicit and inherited browser API-surface paths.
- Current progress: browser runtime summary parsing now treats unexpected top-level keys as shape-invalid and falls back to stdout when the configured summary file drifts from the contract; browser-requested JS run/test smoke now also covers that unexpected-top-level-key fallback path end-to-end, and the dedicated TS/JSX/TSX browser-harness fallback suites now mirror that same drift path in JSON test coverage.

### 17.2 Browser runtime contract

- Decide whether harness-assisted `run --api browser` / `test --api browser` graduates to stable standalone browser runtime support.
- Before any promotion, specify host ownership, summary JSON behavior, sandbox limitations, diagnostics, and failure modes.
- Keep browser-targeted `check` / `build --bundle`, harness execution, and post-deployment browser behavior separate.

### 17.3 Late host APIs and resources

- Add subprocess, socket/listener, worker/thread, env materialization, env mutation, cwd/process-control, and late Node/Deno module support only with policy/effect/resource contracts.
- Keep host visibility aligned with effective `apiSurface`, command family, runtime profile, and maturity gate.
- Preserve explicit gates for unavailable host members.

### 17.4 Late object/runtime APIs

- Triage `Proxy`, weak references, finalization, broader `Intl`, `SharedArrayBuffer`, `Atomics`, and wider object helpers against no-GC/no-JIT and optimization constraints.
- Promote only with conformance, sandbox/resource, and JSON-output evidence.
- Current progress: browser late-compat TSX smoke now also mirrors the full `Proxy.revocable` alias family, including the bracketed `globalThis["Proxy"].revocable` and `globalThis.Proxy["revocable"]` spellings, alongside the existing gated object-model aliases so the TSX late-object gate stays aligned with the JS/browser coverage.
## Exit gate

- New host/runtime support has integration, sandbox/effect, resource-budget, and JSON-output evidence.
- Support wording names exact command, API surface, profile, and artifact/runtime context.
- Unsupported host/object surfaces fail through canonical diagnostics.
