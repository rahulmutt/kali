# Stage 3.2 Status Update

**Date:** 2026-04-17  
**Status:** 🟡 Node compatibility is now threaded through the CLI/runtime command surface, package host-fit validation, and a partial runtime-linker projection layer for common Node built-ins, but the full Node helper surface is still incomplete

## Summary

Stage 3.2 is still not complete, but the Node-targeted command path is now wired through check/build/run/test, the type/name resolver accepts `node:` imports in the Node context, and the package-management side still respects the Node-targeted project surface. The npm package host-fit check reads the project's `compilerOptions.apiSurface` and treats `node` as an allow-list context for Node-only builtins, while the default standalone context continues to reject those packages with the canonical `E6005` diagnostic. The `kali_api_node` helper layer now includes promise-style filesystem, stream, and HTTP helpers, and the runtime linker can execute those Node-specific host imports when the effective API surface is `node`. Workspace tests are still green.

## Evidence

- `kali_cli` now threads `--api node` through check/build/run/test command resolution and compile-time analysis ✅
- `kali_types` accepts `node:` imports and node-only globals in the Node analysis context, while still rejecting them in the default standalone context ✅
- `kali_npm` now recognizes the Node-targeted project surface when validating package host fit ✅
- Default standalone package installs still reject Node-only builtins with `E6005` ✅
- `kali_api_node` now carries the pure-Rust helper layer for process/path/crypto/events/buffer/util plus fs/url/os scaffolding, plus promise-style filesystem, stream, and HTTP helpers for the Node runtime projection ✅
- The runtime linker now registers Node-specific host imports under `kali:node` when the effective API surface is `node`, covering `fs/promises`, stream concatenation, and HTTP GET execution paths ✅
- Added Node-context coverage so the command path, host-fit validator, and runtime-linker projection now have positive paths in tests as well as the negative default path ✅
- `cargo test --workspace` passes ✅

## Current Limits

- Node built-in execution coverage is still partial; the current wiring now reaches the runtime linker for `fs/promises`, stream, and HTTP, but the broader Node helper surface remains to be filled out.
- The package host-fit change prepares the install-time side of the Phase-3 story and now shares the same Node-targeted analysis context as the command path.
- The remaining follow-on work is to broaden the lower-level `Buffer`, `assert`, `util`, and `process` helpers, then extend the projection coverage to the rest of the documented Node subset.

## Next Step

Broaden the Node runtime helper surface beyond `fs/promises`, stream, and HTTP to cover the remaining documented Phase-3 subset (`Buffer`, `assert`, `util`, and `process`) while preserving the current runtime-linker projection path.
