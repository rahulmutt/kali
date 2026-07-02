# Browser harness: JS-runtime preference + bundle-glue import parity

**Date:** 2026-07-01
**Status:** Part A implemented; Part B (test-only CDP smoke driver) implemented.

## Problem

Installing Chromium in the CI/dev container silently broke ~900 `kali_cli`
"browser" tests. Two independent regressions were involved:

1. **`int_to_string` LinkError (already fixed earlier this session).** Recent
   codegen commits (`2ada9fea3`, `235cc09e1`) started emitting three new
   `kali:rt` imports — `int_to_string`, `string_concat`, `float_to_fixed` — but
   the hand-mirrored JS import lists were not updated. This affected the two
   browser-runtime harness scripts in `crates/kali_runtime/src/browser/harness.rs`
   (fixed) **and** the two bundle-glue templates generated in
   `crates/kali_cli/src/bin/cmd_build.rs` (fixed in this change).

2. **Chromium preferred over a JS runtime.** `browser_harness_default_command_parts`
   ranked real browsers ahead of `node`/`bun`/`deno`. Once Chromium was present it
   became the default harness, routing tests into the incomplete real-browser path:
   `No usable sandbox!` in the container, and — once `--no-sandbox` let Chromium
   start — an indefinite hang, because the harness blocks on the browser process
   exiting (`harness.output()`) while a headless browser never exits on its own,
   and its `console.log` never reaches process stdout the way `node`'s does.

### Why node is the right default (not a downgrade)

Browser API support is validated by **compiling with `--api browser`** (exercising
the browser bundle/codegen path: glue, exports, WIT, worker/thread scripts, summary
schema) and **executing the emitted artifact through the browser-harness contract**
(`hostContract: browser-requested`, `runtimeBackend: browser-harness`,
`threadTopology`, console, args). That contract is modeled in JS and runs faithfully
under `node` — which is exactly why 137 of the ~233 browser test files already pin
`KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node`. The literal browser was only ever an
optional execution backend and was never the test oracle in this repo. A real
browser reproduces the harness stdout contract only under a DevTools driver.

## Part A — implemented

1. **JS-runtime-preferred default.** `browser_harness_default_command_parts` now
   selects `node`/`bun`/`deno` before any browser (browser only when no JS runtime
   exists). The ordering is factored into a pure
   `browser_harness_default_command_parts_from(is_available)` helper for
   deterministic unit testing. Explicit `KALI_BROWSER_BUNDLE_HARNESS_COMMAND`
   overrides still win and are respected verbatim.
2. **Bundle-glue import parity.** Both bundle templates (ESM and CJS) in
   `cmd_build.rs` gained `int_to_string`/`string_concat`/`float_to_fixed`, plus the
   `wasmHeap` binding and `allocGuestString`/`decodeStringHandleBytes` helpers,
   mirroring the `harness.rs` fix. String handles use the canonical ABI
   (`STRING_HANDLE_TAG | (offset << 32) | len`, bump-allocated via the exported
   mutable i32 `__heap` global; see `kali_runtime/src/host/memory.rs`).
3. **Reverted the interim `--no-sandbox` change.** With `node` as the default,
   Chromium is never auto-selected, so the auto-`--no-sandbox` logic was dead code.
   It is removed to keep the change focused; the flag requirement is captured in
   Part B where it is actually exercised.

Result: the browser suite goes green by routing to `node`, spawns **zero** browser
processes (eliminating the per-test Chromium explosion — ~64 concurrent processes
were observed), needs no new dependency, and stays consistent with how the majority
of browser tests already run.

## Part B — test-only CDP smoke driver (implemented)

Genuine in-browser execution (real V8/DOM/loader fidelity, and any bug a browser
would catch that the JS model cannot) requires driving Chromium over the **Chrome
DevTools Protocol**, because:

- a headless browser does not exit on its own → the harness's `Command::output()`
  blocks forever (only `window.close()`/`Browser.close`/a timeout ends it), and
- a page's `console.log` does not reach process stdout as clean lines (needed raw
  for `stdout.contains("3\n")` assertions and as pure JSON for summary-parsing
  tests) — CDP `Runtime.consoleAPICalled` events are the only faithful capture.

Design sketch for when it is wanted:

- A small, **explicitly gated** smoke suite (a handful of representative bundles),
  **not** the default path for the 96 default-resolving files.
- A synchronous CDP driver in `kali_runtime` (blocking, matching the crate's sync
  design): launch `chromium --headless --no-sandbox --remote-debugging-port=0`,
  read the chosen WS endpoint from stderr, open a localhost `ws://` connection
  (hand-rolled minimal client — no TLS needed — or the `tungstenite` crate),
  `Page.navigate`, collect `Runtime.consoleAPICalled`, detect completion via a
  sentinel console line or `Runtime.executionContextDestroyed`, then `Browser.close`.
- **One shared browser instance**, opening a new target/page per test (this is the
  only way to "reuse a single Chrome instead of many" — plain process launch is
  inherently one instance per invocation), plus a bounded launch timeout so it can
  never hang.

**Implemented as a gated, test-only driver.** Run the real-browser smoke test with:
`cargo test -p kali_cli --test browser_cdp_smoke -- --ignored`
(requires Chromium; launched with `--no-sandbox` for containers). The driver is
test infrastructure, not production code: it lives in
`crates/kali_cli/tests/cdp_driver/` with `tungstenite` as a `kali_cli`
dev-dependency, so nothing enters production builds.

Implementation notes: Chromium's `Runtime.addBinding` functions require exactly one
string argument, so harness pages call `globalThis.__kaliHarnessDone('')` — the
binding name is the shared `kali_runtime::BROWSER_HARNESS_DONE_BINDING` constant,
which the CDP driver re-uses. Both production follow-ups recorded here are closed
(2026-07-02): the bundle glue now exports a memoized `start()` helper that runs the
program's top-level statements (the wasm `_start` export) exactly once, so a bare
top-level program routes `console.log` through the `console_log` import in a
browser; and `kali_runtime::browser_bundle_harness_page` generates a browser-native
harness page (no `node:` imports — the glue's own `fetch` works over HTTP). The
smoke test still serves the bundle from an in-test localhost HTTP server (Chromium
blocks `fetch()` of `file://`), but its page now comes from the production
generator, and its fixture exercises both entry shapes: a bare top-level statement
via `start()` and an exported function via the per-export wrapper.

## Testing

- Unit: `browser_harness_default_command_prefers_js_runtime_over_browser` pins the
  `node > bun > deno > browser > node-fallback` selection order deterministically.
- Integration: previously-failing `build_emits_*`/bundle and `run_supports_*`
  binaries pass under the node default with zero browser processes.
