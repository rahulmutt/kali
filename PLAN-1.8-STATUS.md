# Stage 1.8 Status Update

**Date:** 2026-04-12  
**Status:** ✅ Runtime execution complete — wasmtime-backed execution, the default Deno host-surface subset, timer/microtask scheduling, Web-baseline time/random primitives, guest-side support-library helpers, and edge-case fixture coverage are all in place

## Summary

Stage 1.8 now has a working wasmtime-backed execution path for the current compiler output. The CLI can compile a source file to WASM, instantiate it through the runtime crate, and report pass/fail results for repo-backed smoke fixtures and direct file inputs.

The runtime linker now also exposes the basic console host imports (`console_log`, `console_error`, `console_warn`) expected by the early host-surface work, plus a first Deno-oriented host-surface subset for filesystem read/write, environment lookup, arguments, fetch, and timer/microtask scheduling. The Web-baseline host primitives now include `performance.now()` and `crypto.getRandomValues()` so the host-side baseline has concrete time/random coverage. The guest-side support-library helpers now cover URL parsing/resolution, text encoding/decoding, structured cloning, abort signaling, and event primitives. `kali test` now honors guest-registered `Kali.test(...)` callbacks, supports the Phase-1 `--filter` narrowing step, discovers checked-in test fixtures from the project tree, and still rejects `--coverage` with the documented phase-gating diagnostic.

## Evidence

- `cargo test -p kali_runtime --lib` ✅
- `cargo test -p kali_cli --lib` ✅
- `cargo test -p kali_cli --test runtime_smoke` ✅
- `cargo test --workspace` ✅

## Notable Deliverables

- `kali_runtime` now instantiates emitted WASM modules with wasmtime
- `kali run` executes a compiled module instead of printing a stub message
- `kali test` supports explicit file sets and project-tree discovery for the current source-file patterns, plus the Phase-1 `--filter` narrowing step
- Guest-registered `Kali.test(...)` callbacks are collected and executed by the runtime test runner
- Declaration-only entrypoints are rejected for `run` and `test` with the canonical invalid-entrypoint diagnostic (`E5007`)
- `kali test --coverage` is rejected with the documented phase-gating diagnostic (`E5006`) until the report contract exists
- The runtime now drains queued microtasks before timers and can clear scheduled timers inside the host event loop
- Smoke tests cover a repo-backed successful run fixture, guest test registration, invalid declaration-only inputs, checked-in test discovery, the `ok N` test-report path, filter narrowing, timer/microtask ordering, timer clearing, and coverage gating

## Current Limits

- The runtime still exercises the compiler's simple WASM output rather than a full guest JS host surface
- The guest-side Web baseline support-library helpers now live in `kali_api_web`; remaining Stage 1.8 work is runtime-boundary polish and any final evidence hardening rather than missing primitives
- Runtime edge-case coverage now includes timer/interval clearing, mocked fetch failure, entrypoint trap diagnostics, and the time/random host primitives.
- The spelled-out default API surface (`--api deno`) is now locked in for both `run` and `test`, matching the implicit default path.
- Stage 1.8 has no remaining implementation follow-up; the stage is now closed.

## Next Step

Runtime execution is closed; proceed with the remaining plan-level handoff work and Phase 2 planning.
