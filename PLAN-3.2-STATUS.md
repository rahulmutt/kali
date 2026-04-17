# Stage 3.2 Status Update

**Date:** 2026-04-17  
**Status:** 🟡 Node compatibility is now threaded through the CLI/runtime command surface, package host-fit validation, and a widening runtime-linker projection layer for common Node built-ins; the helper surface now also exposes the remaining process/util/assert helpers plus Buffer base64/hex round-tripping, lexical path-relative resolution, and deterministic URL parse/resolve helpers, and the runtime linker now covers path.relative plus the existing path, crypto, os, child-process, process stdout/stderr, util formatting/assertion, and URL projection alongside the fs/stream/http/process wiring, while the package corpus now includes Node-assuming coverage, but broader Node execution coverage is still incomplete

## Summary

Stage 3.2 is still not complete, but the Node-targeted command path is now wired through check/build/run/test, the type/name resolver accepts `node:` imports in the Node context, and the package-management side still respects the Node-targeted project surface. The npm package host-fit check reads the project's `compilerOptions.apiSurface` and treats `node` as an allow-list context for Node-only builtins, while the default standalone context continues to reject those packages with the canonical `E6005` diagnostic. The `kali_api_node` helper layer now includes process/path/crypto/events/buffer/util primitives plus promise-style filesystem, stream, HTTP, child-process, os, and URL scaffolding, and the runtime linker can execute the Node-specific `process.argv` / `process.env` reads, process stdout/stderr writes, path normalization, path joining/resolution, path-relative computation, URL parse/resolve, crypto hashing/HMAC, child-process spawning, util formatting, assert-equality checks, and os platform/arch/eol/cpus probes when the effective API surface is `node`. Buffer helpers now also support base64/hex round-tripping so the binary-data slice looks more like a real Node buffer workflow. The package corpus now includes representative Node-assuming packages as well, so the corpus split covers the documented Node context instead of only the browser/default standalone lanes. Workspace tests are still green.

## Evidence

- `kali_cli` now threads `--api node` through check/build/run/test command resolution and compile-time analysis ✅
- `kali_types` accepts `node:` imports and node-only globals in the Node analysis context, while still rejecting them in the default standalone context ✅
- `kali_npm` now recognizes the Node-targeted project surface when validating package host fit ✅
- Default standalone package installs still reject Node-only builtins with `E6005` ✅
- `kali_api_node` now carries the pure-Rust helper layer for process/path/crypto/events/buffer/util plus fs/url/child_process/os scaffolding, plus promise-style filesystem, stream, HTTP, and child-process helpers for the Node runtime projection; Buffer now supports base64/hex round-tripping, NodePath now includes relative resolution, NodeUrl now exposes parse/resolve helpers, and the runtime linker now exposes `path.relative`, `util.format`, `assert.equal`, and `buffer` conversions alongside `url.parse` / `url.resolve` ✅
- `kali_api_node` now exposes `NodeUtil` / `NodeAssert` namespace accessors and `NodeProcess` argv helpers so the Node helper surface can be reused without reaching into raw fields ✅
- The runtime linker now registers Node-specific host imports under `kali:node` when the effective API surface is `node`, covering `fs/promises`, stream concatenation, HTTP GET execution paths, `process.argv` / `process.env`, path normalization/join/resolve helpers, crypto hashing/HMAC/UUID helpers, child-process spawning, util formatting, assert equality, and os platform/arch/eol/cpus probes ✅
- Added Node-context coverage so the command path, host-fit validator, runtime-linker projection, and the expanded Node helper path now have positive paths in tests as well as the negative default path ✅
- Added runtime smoke coverage for the `url.parse` / `url.resolve` host-import projection and the new util/buffer/assert host-import slice so the broader Node helper surface is exercised end-to-end ✅
- Added Node-assuming package corpus coverage for `axios`, `express`, and `chalk` stubs in the Node context ✅
- `cargo test --workspace` passes ✅

## Current Limits

- Node built-in execution coverage is still partial; the current wiring now reaches the runtime linker for `fs/promises`, stream, HTTP, `process.argv` / `process.env`, process stdout/stderr writes, path, crypto, child-process, util formatting, assert equality, and os probes, but the broader Node helper surface remains to be filled out.
- The package host-fit change prepares the install-time side of the Phase-3 story and now shares the same Node-targeted analysis context as the command path, and the corpus now exercises that path with Node-assuming packages too.
- The remaining follow-on work is to broaden the runtime projection coverage to the rest of the documented Node subset that still lacks host-import wiring, especially the event-emitter and any later compatibility breadth beyond the child-process projection.

## Next Step

Broaden the Node runtime helper surface beyond `fs/promises`, stream, HTTP, `process.argv` / `process.env`, process stdout/stderr writes, path, crypto, child-process, util, buffer, assert, and os to cover the remaining documented Phase-3 subset while preserving the current runtime-linker projection path.
