# Stage 3.2 Status Update

**Date:** 2026-04-12  
**Status:** 🟡 Node compatibility is now threaded through the CLI/runtime command surface and package host-fit validation, but the built-in projection story is still partial

## Summary

Stage 3.2 is still not complete, but the Node-targeted command path is now wired through check/build/run/test, the type/name resolver accepts `node:` imports in the Node context, and the package-management side still respects the Node-targeted project surface. The npm package host-fit check reads the project's `compilerOptions.apiSurface` and treats `node` as an allow-list context for Node-only builtins, while the default standalone context continues to reject those packages with the canonical `E6005` diagnostic. The broad `kali_api_node` helper layer and its unit tests remain in place, and workspace tests are still green.

## Evidence

- `kali_cli` now threads `--api node` through check/build/run/test command resolution and compile-time analysis ✅
- `kali_types` accepts `node:` imports and node-only globals in the Node analysis context, while still rejecting them in the default standalone context ✅
- `kali_npm` now recognizes the Node-targeted project surface when validating package host fit ✅
- Default standalone package installs still reject Node-only builtins with `E6005` ✅
- `kali_api_node` still carries the pure-Rust helper layer for process/path/crypto/events/buffer/util plus fs/url/os scaffolding ✅
- Added Node-context coverage so the command path and host-fit validator now have positive paths in tests as well as the negative default path ✅
- `cargo test --workspace` passes ✅

## Current Limits

- Node built-in execution coverage is still partial; the current wiring covers command-context acceptance and static analysis, but not the full runtime projection for the entire Node helper surface.
- The package host-fit change prepares the install-time side of the Phase-3 story and now shares the same Node-targeted analysis context as the command path.
- The broader Node built-in host-import surface is still a follow-on task.

## Next Step

Broaden the Node helper host imports and runtime projection so common Node built-ins execute, not just analyze, under the now-wired command context.
