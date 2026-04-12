# Stage 1.8 Status Update

**Date:** 2026-04-11  
**Status:** 🚧 Runtime execution wired for simple modules

## Summary

Stage 1.8 now has a working wasmtime-backed execution path for the current compiler output. The CLI can compile a source file to WASM, instantiate it through the runtime crate, and report pass/fail results for simple smoke-test inputs.

## Evidence

- `cargo test -p kali_runtime --lib` ✅
- `cargo test -p kali_cli --lib` ✅
- `cargo test -p kali_cli --test runtime_smoke` ✅
- `cargo test --workspace` ✅

## Notable Deliverables

- `kali_runtime` now instantiates emitted WASM modules with wasmtime
- `kali run` executes a compiled module instead of printing a stub message
- `kali test` supports explicit file sets and project-tree discovery for the current source-file patterns
- Declaration-only entrypoints are rejected for `run` and `test` with the canonical invalid-entrypoint diagnostic (`E5007`)
- Smoke tests cover successful execution, invalid declaration-only inputs, and the `ok N` test-report path

## Current Limits

- The runtime still exercises the compiler's simple WASM output rather than a full guest JS host surface
- Host APIs such as `console`, `fetch`, timers, and Deno-style filesystem calls remain pending
- The test runner currently treats successful execution as a pass; the `Kali.test(...)` registration protocol still needs a proper guest-side implementation

## Next Step

Continue Stage 1.8 by filling in the guest host surface and the real test-registration protocol.
