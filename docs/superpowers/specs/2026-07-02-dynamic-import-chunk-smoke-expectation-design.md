# Fix browser-bundle dynamic-import chunk smoke tests (stale `0n` expectation)

**Date:** 2026-07-02
**Status:** Approved

## Problem

Thirteen `runtime_smoke` tests fail on main with `unexpected chunk result 7`
from the node harness:

- 7 in `crates/kali_cli/tests/runtime_smoke/build.rs`
  (`*_emits_browser_bundle_chunks_for_template_literal_dynamic_imports*`)
- 6 in `crates/kali_cli/tests/runtime_smoke/misc.rs`
  (`browser_bundle_js_exposes_runtime_dynamic_import_loader*`,
  `browser_bundle_normalizes_runtime_dynamic_import_specifiers*`)

All 13 are the call sites of the shared helper
`assert_browser_bundle_dynamic_import_loader` (`crates/kali_cli/tests/runtime_smoke.rs`),
whose generated harness body asserts the dynamically imported chunk's
`lazyValue()` returns `0n`, and whose stdout assertion requires a `'0'`.

Every one of those call sites writes the chunk fixture
`export function lazyValue() { return 7; }`. The `0n` expectation was pinned in
phase 11 (`3c718dcf7`), when chunk function bodies compiled to stubs returning
the i64 default `0`. Verified: at `3c718dcf7` the compiled chunk really returned
`0n`; on current main it returns `7n` — the compiler now compiles chunk function
bodies for real, so the pinned stub value is stale. This is a test-expectation
bug exposed by a codegen improvement, not a runtime regression. It predates the
2026-07-02 template-literal series (probed `7n` at `2fa683743` and `280b76451`).

Returning a BigInt (`7n`, not `7`) is the established browser-bundle ABI: chunk
glue exposes raw wasm exports, and i64 results surface as BigInt. Other lanes
(`runtime_smoke/run.rs`, `runtime_smoke/test.rs`,
`browser_template_literal_dynamic_import_harness.rs`) use `return 0n` fixtures
against their own harness scripts — self-consistent, passing, out of scope.

## Change

Test-only; no product code. In `assert_browser_bundle_dynamic_import_loader`
(`crates/kali_cli/tests/runtime_smoke.rs`):

1. Harness body: `if (value !== 0n)` → `if (value !== 7n)`.
2. Stdout assertion: `stdout.contains('0')` → `stdout.contains('7')`.
3. Add a short comment on the helper noting the expected `7n` mirrors the
   callers' `return 7` chunk fixture.

This makes the lane strictly stronger: `7n` proves the chunk body actually
executed with its source semantics, whereas `0n` was indistinguishable from a
dead stub. A regression back to stub codegen now fails these tests.

## Verification

- `cargo test -p kali_cli --test runtime_smoke -- dynamic_import` — 45 tests,
  currently 13 failing, must go fully green.
- `cargo test -p kali_cli --test runtime_smoke -- template` — second check over
  the originally observed failure set.

A background `git bisect` (good `3c718dcf7` → bad `280b76451`) identifies the
codegen commit that turned chunk stubs into real bodies; it is cited in the
commit message as provenance but the fix does not depend on it.

## Risks / error handling

None beyond the test suite itself. The change touches one shared helper used
only by the 13 affected tests.
