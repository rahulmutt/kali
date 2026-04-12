# Stage 3.2 Status Update

**Date:** 2026-04-12  
**Status:** 🟡 Node compatibility scaffolding is still partial, but package host-fit validation now respects the Node-targeted project surface

## Summary

Stage 3.2 is still not complete, but the package-management side has moved one step closer to the planned Node compatibility surface. The npm package host-fit check now reads the project's `compilerOptions.apiSurface` and treats `node` as an allow-list context for Node-only builtins, while the default standalone context continues to reject those packages with the canonical `E6005` diagnostic. The broad `kali_api_node` helper layer and its unit tests remain in place, and workspace tests are still green.

## Evidence

- `kali_npm` now recognizes the Node-targeted project surface when validating package host fit ✅
- Default standalone package installs still reject Node-only builtins with `E6005` ✅
- `kali_api_node` still carries the pure-Rust helper layer for process/path/crypto/events/buffer/util plus fs/url/os scaffolding ✅
- Added Node-context coverage so the host-fit validator now has a positive path in tests as well as the negative default path ✅
- `cargo test --workspace` passes ✅

## Current Limits

- `--api node` is still gated in the CLI, runtime wiring is still pending, and Node compatibility is not yet publicly available.
- The package host-fit change only prepares the install-time side of the Phase-3 story; it does not yet make Node-aware runtime execution or `check/build/test` command paths available.
- The broader Node built-in execution surface is still a follow-on task.

## Next Step

Wire the Node API surface through the remaining CLI/runtime paths so `--api node` becomes a real command context instead of only a latent package-validation capability.
