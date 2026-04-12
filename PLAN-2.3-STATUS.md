# Stage 2.3 Status Update

**Date:** 2026-04-12  
**Status:** ✅ Public embedding surface complete — `kali build --lib` now emits a WIT sidecar, `kali build --capi` emits the C header plus C ABI metadata, `kali build --component` emits a valid component artifact, and `kali_embed` exposes the stable in-tree embedding API.

## Summary

Stage 2.3 now has the public embedding surface wired end-to-end. The CLI emits deterministic library-side WIT sidecars, the C ABI projection writes the generated header and compatibility metadata, and the component projection emits a validator-passing component wrapper. The in-process `kali_embed` API also exposes stable `compile_file` / `compile_lib` entry points with deterministic artifact metadata.

## Evidence

- `cargo test -p kali_capi --quiet` ✅
- `cargo test -p kali_embed --quiet` ✅
- `cargo test -p kali_cli --test runtime_smoke --quiet` ✅
- `cargo test --workspace --quiet` ✅

## Notable Deliverables

- `kali build --lib <file>` emits `*.lib.wasm`, `*.lib.wit`, and `*.lib.meta.json`
- `kali build --capi <file>` emits `*.capi.wasm`, `*.wit`, `*.h`, and `*.capi.meta.json`
- `kali build --component <file>` emits `*.component.wasm`, `*.wit`, and `*.component.meta.json`
- `kali_capi` now owns deterministic C-header generation plus the C ABI metadata payload
- `kali_embed` now exposes stable library/executable compile entry points and the WIT sidecar string

## Current Limits

- Public embedding support is in-tree and tested, but the broader Phase 2 program still has remaining work in ownership analysis and later verification stages.
- Cross-language component composition beyond the deterministic WIT sidecar remains later compatibility.

## Next Step

Continue with Stage 2.4 — Lean Model Foundation, and keep the Phase 2 completion gate open until all four stage tracks are done.
