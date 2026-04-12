# Stage 1.8 Status Update

**Date:** 2026-04-11  
**Status:** 🚧 Runtime execution wired for simple modules

## Summary

Stage 1.8 now has a working wasmtime-backed execution path for the current compiler output. The CLI can compile a source file to WASM, instantiate it through the runtime crate, and report pass/fail results for simple smoke-test inputs.

The runtime linker now also exposes the basic console host imports (`console_log`, `console_error`, `console_warn`) expected by the early host-surface work, and `kali test` now supports the Phase-1 `--filter` narrowing step while rejecting `--coverage` with the documented phase-gating diagnostic.

## Evidence

- `cargo test -p kali_runtime --lib` ✅
- `cargo test -p kali_cli --lib` ✅
- `cargo test -p kali_cli --test runtime_smoke` ✅
- `cargo test --workspace` ✅

## Notable Deliverables

- `kali_runtime` now instantiates emitted WASM modules with wasmtime
- `kali run` executes a compiled module instead of printing a stub message
- `kali test` supports explicit file sets and project-tree discovery for the current source-file patterns, plus the Phase-1 `--filter` narrowing step
- Declaration-only entrypoints are rejected for `run` and `test` with the canonical invalid-entrypoint diagnostic (`E5007`)
- `kali test --coverage` is rejected with the documented phase-gating diagnostic (`E5006`) until the report contract exists
- Smoke tests cover successful execution, invalid declaration-only inputs, the `ok N` test-report path, filter narrowing, and coverage gating

## Current Limits

- The runtime still exercises the compiler's simple WASM output rather than a full guest JS host surface
- Host APIs such as `console`, `fetch`, timers, and Deno-style filesystem calls remain pending
- The test runner still treats a module's successful execution as a pass when no guest-side registrations are present; the real `Kali.test(...)` registration protocol still needs a proper guest-side implementation

## Next Step

Continue Stage 1.8 by filling in the guest host surface and the real test-registration protocol.
