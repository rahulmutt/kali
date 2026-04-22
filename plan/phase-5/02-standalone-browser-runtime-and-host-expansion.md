# Stage 5.2 — Standalone Browser Runtime & Host Expansion

**Phase:** 5 — Later Compatibility & Platform Expansion  
**Spec refs:** [`specs/11-standard-apis.md`](../../specs/11-standard-apis.md), [`specs/10-runtime.md`](../../specs/10-runtime.md), [`specs/12-cli.md`](../../specs/12-cli.md), [`specs/09-sandboxing.md`](../../specs/09-sandboxing.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** [1.11 — Build Artifacts](../phase-1/11-build-artifacts.md), [3.3 — Ecosystem Breadth](../phase-3/03-ecosystem-breadth.md), and preferably [5.1 — Threaded Runtime Profile](01-threaded-runtime-profile.md) before any worker-aware browser runtime claims

## Goal

Move browser support from the Phase-1 browser-targeted analysis/build lane to a genuine later
standalone browser runtime/test contract, while also making the host-adapter/runtime-backend story
explicit enough to support broader host deployment needs without forking Kali's public semantics.

## Workable Milestone

- `kali run --api browser <file>` and `kali test --api browser [files...]` have a documented,
  evidence-backed runtime contract.
- Browser execution uses a real browser host path rather than pretending the native standalone
  runtime implements the DOM.
- The browser host adapter preserves the documented mediated-capability and sandbox honesty rules.
- Runtime backend expansion, if implemented, remains behind one stable public contract and does not
  fork CLI, schema, or diagnostic behavior.

## Progress

- Added a reusable browser-harness execution helper in `kali_runtime` that launches the configured command, appends the script entrypoint plus args, and returns deterministic stdout/stderr/exit-status capture for future browser runtime wiring and the existing smoke harnesses.
- `kali run --api browser` and `kali test --api browser` now flow through that configured browser harness command when it is present, so the later browser-runtime path is reachable from the user-facing CLI instead of only from lower-level runtime tests.
- The browser-harness execution outcome now also carries the requested host-contract label (`browser-requested`) so the future browser-runtime contract has one explicit machine-readable selector to reuse alongside the captured command, output, and summary fields.
- Browser harness command selection now fails closed on an explicit empty override value instead of silently falling back to the default host command, keeping malformed `KALI_BROWSER_BUNDLE_HARNESS_COMMAND` inputs honest for the later browser-runtime path.
- The shared browser-runtime execution outcome now also preserves the guest-reported argument list from the harness summary alongside the registered-test list, giving the later browser contract one more deterministic summary field to reuse instead of only parsing stdout text.
- Added a self-contained browser-runtime harness script generator in `kali_runtime` that embeds WASM bytes, bridges console output, and can emit a simple registered-test summary so future real-browser execution has one deterministic script shape to reuse.
- Factored the shared browser-bundle smoke harness prelude into a reusable `kali_runtime` script generator so future browser runtime wiring and the current browser-bundle smoke tests can build on one deterministic fetch/wasm bootstrap instead of duplicating the prelude inline.
- The browser-runtime contract metadata now exposes a structured descriptor helper in `kali_runtime`, and the shared browser-runtime rejection diagnostics now consume that descriptor so the command-family wording, host label, host description, and future command scope stay centralized instead of being hand-written in each call site.
- The browser-runtime contract metadata now exposes a reusable supported-commands note helper, and the shared browser-runtime rejection diagnostics now consume that helper so the command-family wording stays centralized with the later contract descriptor instead of being hand-written in each call site.
- The browser-runtime contract now also exposes a centralized summary note, and the CLI/runtime browser-gate diagnostics reuse it so the later-compatibility wording stays aligned across the direct runtime rejection path and the user-facing smoke tests.
- The browser harness command-spec parser now lives in `kali_runtime`, giving the browser smoke lane and future browser-runtime harness one shared argv-style command splitter plus a deterministic bun/node default selector instead of duplicating the override logic in tests.
- `kali run` and `kali test` now share an explicit browser-runtime rejection helper so both explicit and inherited `browser` API surfaces are gated consistently with the current later-compatibility row instead of accidentally executing against the single-threaded baseline; the helper now reports the browser API surface explicitly in the diagnostic text.
- The same rejection helper now also states that Kali does not yet define a standalone browser runtime contract, which keeps the user-facing error aligned with the Stage 5.2 contract wording instead of implying a fake in-process DOM story.
- Added CLI smoke coverage for both explicit and inherited browser API surfaces on `run` and `test`, including JSON-envelope regressions for the unsupported browser gate, so the phase-one browser runtime boundary stays honest until the standalone browser harness exists. The `run` browser-gate regression now also exercises the documented `--` guest-argument separator, and new sandbox-attached browser-gate regressions pin that `--sandbox` does not relax the `run` / `test` browser availability split.
- The runtime layer now mirrors that honesty check directly: direct `kali_runtime` callers that select the `browser` API surface are rejected with the same feature-unavailable diagnostic instead of falling through into the native standalone execution path.
- Browser-runtime unavailability now comes from one shared runtime helper, and the runtime exposes an explicit host-contract discriminator so the CLI and runtime stay aligned on the current wasmtime baseline versus a browser-requested contract boundary.
- The browser-harness execution helper now preserves the fully resolved argv vector alongside stdout/stderr/exit status, so the future browser-runtime harness has one deterministic command record to reuse when the real browser host opens.
- The browser-runtime rejection diagnostic now also points users back at the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) so the later-compatibility gate is actionable instead of only refusing the unsupported runtime shape.
- Added a canonical runtime-host label helper in `kali_runtime` so browser-runtime diagnostics and future browser-harness logging can reuse one stable contract label (`browser-requested`) instead of inventing a second browser-only wording path.
- The CLI browser-runtime smoke suite now pins that canonical `browser-requested` host label in both text and JSON output for `run` and `test`, keeping the user-facing runtime diagnostics aligned with the new shared label helper.
- The browser-runtime rejection diagnostic now also carries the selected host contract as a structured note, so machine-readable consumers can recover the same `browser-requested` label without scraping the free-form message.
- The browser-runtime rejection path now also emits structured `DiagnosticContext` payloads for explicit `--api browser` and inherited browser-config cases, so JSON consumers can tell whether the request came from CLI or manifest selection without scraping prose.
- Direct `kali_runtime` callers that select the browser API surface now carry a minimal structured diagnostic context too (`origin = default`, `effectiveValue = browser`), so the runtime-layer gate is machine-readable even when the CLI is not the entrypoint.
- The runtime execution path now threads the selected host contract through successful `RuntimeOutcome` values as well, so later browser backend work has one explicit execution-contract field to reuse without guessing from the API-surface string.
- The runtime crate now also carries an explicit `RuntimeBackend` abstraction with the canonical `wasmtime` label on `RuntimeCtx`, `KaliHostState`, and `RuntimeOutcome`, keeping the current baseline backend name stable without pretending the standalone browser host is available yet.
- The browser-runtime rejection diagnostic now also names the current `wasmtime` backend explicitly, so the later browser contract stays tied to one visible baseline backend instead of only naming the requested browser host.
- Direct runtime browser-gate diagnostics now reuse a shared browser-request context helper, so the `run` / `test` rejection path carries the same requested/effective browser context shape as the CLI entrypoint instead of dropping the requested value on the floor.
- The browser-runtime rejection diagnostic now also carries an explicit supported-command note derived from the shared contract descriptor, and the CLI `run` / `test` browser gate now reuses the same shared browser-request context helper so the later-compatibility contract stays centralized across CLI and runtime callers.
- The same browser-runtime rejection path now also carries a stable host-description note (`browser runtime host description: real browser host`) so the future standalone browser contract stays explicit in both text and machine-readable diagnostics instead of only naming the host contract label.
- The browser-runtime rejection helper now also carries a contract-scope note that names `run` / `test` as the only future browser-runtime commands and spells out that entrypoints, stdout/stderr capture, and exit-status mapping belong to the future browser harness, keeping the contract wording centralized across runtime and CLI diagnostics.
- Added a browser-runtime contract descriptor in `kali_runtime` that records the intended future `run` / `test` command family, the canonical `browser-requested` host label, and the current browser-targeted command-set hint so the stage has a single shared contract definition to build on.
- The descriptor now also carries the host-description note itself, so the runtime diagnostics can reuse one structured browser-contract payload instead of mixing descriptor fields with a separate host-description helper.
- The CLI JSON payloads for successful `run` and `test` invocations now carry the canonical `hostContract` label alongside the exit/runtime counters, so machine-readable consumers can read the active execution contract without scraping the diagnostics path.
- The browser-bundle smoke harness now prefers the installed `bun` runner when available, and it also honors an explicit `KALI_BROWSER_BUNDLE_HARNESS_COMMAND` override so external runners can swap in a real browser wrapper without changing the fixture contract; the emitted-bundle smoke lane still preserves the same deterministic fetch/wasm loading behavior by default. The override now accepts argv-style command strings, preserves empty quoted arguments, rejects unterminated quotes before falling back to the default harness command, and now also treats an empty executable token as malformed, so browser wrappers that need launcher flags can be exercised without a shell-specific wrapper script.
- The browser-bundle harness override parser now fails closed on malformed command specs instead of quietly reusing the default `bun` / `node` harness, and the browser smoke tests pin that rejection path for both empty executable tokens and unterminated quoted invocations.
- The browser-bundle harness command-spec splitter now lives in `kali_runtime`, so the browser smoke helpers and future browser-runtime plumbing share one argv-style parsing rule instead of duplicating the shell-like tokenizer locally.
- The browser-harness command helper now also exposes a checked result form for malformed override diagnostics, so future browser-runtime code can surface bad `KALI_BROWSER_BUNDLE_HARNESS_COMMAND` values deterministically instead of relying on a panic-only path.
- Browser-harness launch failures now preserve the fully resolved command vector in the error payload and surface it in the diagnostic text, so later browser-host wiring can recover the exact launch plan that failed instead of only seeing the executable name and script path.
- Build artifact metadata sidecars now carry explicit `hostContract` / `runtimeBackend` provenance fields alongside the existing `apiSurface` / `runtimeProfiles` axes, keeping browser-targeted bundle outputs explicit about the current Kali-hosted Wasmtime producer without implying that standalone browser execution is already available.
- The shared browser-harness plumbing now also exposes a structured launch-plan helper, so future browser runtime wiring can inspect the resolved executable, harness args, script path, forwarded args, and current working directory before launching instead of rebuilding that command plan ad hoc.
- The runtime crate now also includes a reusable browser-execution helper that materializes the self-contained harness script into a temp file, launches the configured browser command, and parses the harness summary of registered tests; that keeps the future standalone browser path anchored to one deterministic library entrypoint instead of ad hoc test-only scaffolding.
- The browser-runtime harness now emits an explicit empty test-summary payload even when no guest tests register, so later `test` plumbing has one deterministic summary envelope to parse instead of relying on a missing-summary special case.
- The browser-bundle runtime harness now also knows how to import the emitted browser-targeted bundle glue through `loadWithImports`, letting the later browser runtime path observe a custom import bridge and a deterministic registered-test summary over the linked-artifact layout instead of only the raw embedded-WASM helper.
- The browser runtime and browser-bundle harnesses now execute exported `__kali_callback_<id>` test callbacks, report `testsFailed`, and fail nonzero on callback traps so the future browser runtime test path has a realistic callback-execution summary instead of only a registration log.
- `RuntimeCtx::execute` now also honors an explicitly configured browser harness command for browser API-surface requests, so the browser-requested runtime path can be exercised through the shared browser harness helper when an explicit host command is supplied while still preserving the default browser gate when no harness is configured.

## Tasks

### 1. Standalone browser runtime contract

Define the supported meaning of browser execution:

- which commands are supported (`run`, `test`)
- what the runtime host actually is (real browser harness, remote-controlled browser, or another
  explicitly documented browser host)
- how entrypoints, test discovery, stdout/stderr capture, and exit status map onto CLI behavior
- how this differs from the earlier browser-targeted build lane

The implementation must not blur browser ambient typing with a fake in-process DOM emulation story.

### 2. Browser runner / test harness

Implement the execution harness for later browser commands:

- launch and control a real browser host
- load the emitted/browser-targeted artifact set
- bridge console output, test results, and failures back into the canonical CLI envelopes
- preserve deterministic artifact and report paths as far as the browser host allows

### 3. Browser sandbox and capability honesty

Keep the browser contract honest:

- document which capability/resource claims remain static-only versus runtime-enforced
- preserve the browser ambient-typing vs mediated-capability split
- avoid claiming Kali-controlled post-deployment sandboxing where the browser host actually owns
  enforcement
- add any browser-runtime-specific diagnostics needed for unsupported capability requests

### 4. Host-adapter and backend expansion

If the runtime grows beyond the early native backend baseline, do it through one explicit
abstraction layer:

- keep `wasmtime` as the canonical baseline contract
- add backend selection only if sandbox/resource/diagnostic behavior can remain stable
- avoid backend-specific CLI flags or machine-output drift

This task covers runtime-host/backend expansion, not a second language execution model.

### 5. Browser-runtime package evidence

Extend package and API coverage for the standalone browser runtime path:

- distinguish browser **executable** support from the earlier browser **checkable** /
  **deployable-through-host** claims
- add curated browser-runtime package fixtures
- keep Node/Deno-only globals unavailable unless a later spec explicitly reopens them

### 6. Tests

- real-browser `run --api browser` smoke tests
- real-browser `test --api browser` suites with deterministic result capture
- negative tests that keep unsupported browser-runtime shapes gated until their rows open
- browser-runtime package-corpus fixtures at the exact claimed support rung
- backend-contract regression tests if alternative backends are introduced

## Out of Scope

- broader Node-surface widening
- project-local executable sandbox policy code
- weak-reference/proxy compatibility work tracked in Stage 5.4

## Status

Planned.
