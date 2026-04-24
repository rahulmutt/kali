# Phase 7 — Runtime, Host, and Platform Expansion

## Goal

Add runtime and host breadth without weakening sandbox honesty or confusing deployment targets.

## Owning specs

- `specs/09-sandboxing.md`
- `specs/10-runtime.md`
- `specs/11-standard-apis.md`
- `specs/12-cli.md`
- `specs/18-schemas.md`
- `specs/19-feature-maturity.md`

## Work packets

### 7.1 Threaded runtime profile

- Implement `--wasm-threads` / `runtimeProfiles = ["wasm-threads"]` as an explicit opt-in.
- Support positive `maxThreads` only when the threaded profile is active and supported.
- Add tests for zero-capable budgets vs positive thread budgets.
- Preserve no tracing/background GC and AOT-only compilation.
- Progress: the runtime and `run`/`test` CLI paths now accept the threaded profile on supported execution contexts, and `check` / `build` / `effects` now also accept it on the supported non-browser analysis/build paths; positive `--max-threads` values are honored when that opt-in is present. Browser-targeted and registry-analysis rows still gate the profile separately, and deterministic guest-facing thread-spawn host import plumbing is now in place. Regression coverage now also pins the browser-targeted and registry-analysis rejection paths for `--wasm-threads`, while fuller lowering / multi-worker execution semantics remain follow-up work.

### 7.2 Standalone browser runtime contract

- Decide whether Kali will support `run --api browser` / `test --api browser` through a real browser host contract.
- If yes, specify runtime ownership, sandbox/effect limits, test harness behavior, and JSON outputs before implementation.
- Keep browser bundle/check support separate from standalone browser execution.

### 7.3 Late host APIs

- Add mutable env, subprocess, socket/listener, process identity/control, cwd/chdir, and similar APIs only with explicit policy/effect/resource contracts.
- Ensure host API visibility matches the selected `apiSurface`.

### 7.4 Late object/runtime APIs

- Triage `Proxy`, `WeakMap`, `WeakSet`, `FinalizationRegistry`, `SharedArrayBuffer`, `Atomics`, and broader `Intl`.
- Require conformance fixtures and memory-model review before promotion.

## Exit gate

- Every newly supported host/runtime capability has sandbox, effect, resource, and integration coverage.
- Browser runtime claims are backed by real browser execution evidence if opened.
- Unsupported contexts still fail with canonical diagnostics.
