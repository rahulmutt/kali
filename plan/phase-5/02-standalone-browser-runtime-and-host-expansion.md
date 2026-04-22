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

- The browser-runtime contract metadata now exposes a reusable supported-commands note helper, and the shared browser-runtime rejection diagnostics now consume that helper so the command-family wording stays centralized with the later contract descriptor instead of being hand-written in each call site.
- The browser-runtime contract now also exposes a centralized summary note, and the CLI/runtime browser-gate diagnostics reuse it so the later-compatibility wording stays aligned across the direct runtime rejection path and the user-facing smoke tests.
- `kali run` and `kali test` now share an explicit browser-runtime rejection helper so both explicit and inherited `browser` API surfaces are gated consistently with the current later-compatibility row instead of accidentally executing against the single-threaded baseline; the helper now reports the browser API surface explicitly in the diagnostic text.
- The same rejection helper now also states that Kali does not yet define a standalone browser runtime contract, which keeps the user-facing error aligned with the Stage 5.2 contract wording instead of implying a fake in-process DOM story.
- Added CLI smoke coverage for both explicit and inherited browser API surfaces on `run` and `test`, including JSON-envelope regressions for the unsupported browser gate, so the phase-one browser runtime boundary stays honest until the standalone browser harness exists. The `run` browser-gate regression now also exercises the documented `--` guest-argument separator, and new sandbox-attached browser-gate regressions pin that `--sandbox` does not relax the `run` / `test` browser availability split.
- The runtime layer now mirrors that honesty check directly: direct `kali_runtime` callers that select the `browser` API surface are rejected with the same feature-unavailable diagnostic instead of falling through into the native standalone execution path.
- Browser-runtime unavailability now comes from one shared runtime helper, and the runtime exposes an explicit host-contract discriminator so the CLI and runtime stay aligned on the current wasmtime baseline versus a browser-requested contract boundary.
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
- The CLI JSON payloads for successful `run` and `test` invocations now carry the canonical `hostContract` label alongside the exit/runtime counters, so machine-readable consumers can read the active execution contract without scraping the diagnostics path.
- The browser-bundle smoke harness now prefers the installed `bun` runner when available, and it also honors an explicit `KALI_BROWSER_BUNDLE_HARNESS_COMMAND` override so external runners can swap in a real browser wrapper without changing the fixture contract; the emitted-bundle smoke lane still preserves the same deterministic fetch/wasm loading behavior by default. The override now accepts argv-style command strings, preserves empty quoted arguments, rejects unterminated quotes before falling back to the default harness command, and now also treats an empty executable token as malformed, so browser wrappers that need launcher flags can be exercised without a shell-specific wrapper script.
- The browser-bundle harness override parser now fails closed on malformed command specs instead of quietly reusing the default `bun` / `node` harness, and the browser smoke tests pin that rejection path for both empty executable tokens and unterminated quoted invocations.
- The browser-bundle harness command-spec splitter now lives in `kali_runtime`, so the browser smoke helpers and future browser-runtime plumbing share one argv-style parsing rule instead of duplicating the shell-like tokenizer locally.

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
