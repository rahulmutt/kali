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

- `kali run` and `kali test` now share an explicit browser-runtime rejection helper so both explicit and inherited `browser` API surfaces are gated consistently with the current later-compatibility row instead of accidentally executing against the single-threaded baseline; the helper now reports the browser API surface explicitly in the diagnostic text.
- The same rejection helper now also states that Kali does not yet define a standalone browser runtime contract, which keeps the user-facing error aligned with the Stage 5.2 contract wording instead of implying a fake in-process DOM story.
- Added CLI smoke coverage for both explicit and inherited browser API surfaces on `run` and `test`, including JSON-envelope regressions for the unsupported browser gate, so the phase-one browser runtime boundary stays honest until the standalone browser harness exists. The `run` browser-gate regression now also exercises the documented `--` guest-argument separator, and new sandbox-attached browser-gate regressions pin that `--sandbox` does not relax the `run` / `test` browser availability split.
- The runtime layer now mirrors that honesty check directly: direct `kali_runtime` callers that select the `browser` API surface are rejected with the same feature-unavailable diagnostic instead of falling through into the native standalone execution path.
- Browser-runtime unavailability now comes from one shared runtime helper, and the runtime exposes an explicit host-contract discriminator so the CLI and runtime stay aligned on the current wasmtime baseline versus a browser-requested contract boundary.
- The browser-runtime rejection diagnostic now also points users back at the Phase-1 browser-targeted command set (`kali check --api browser` and `kali build --bundle --api browser`) so the later-compatibility gate is actionable instead of only refusing the unsupported runtime shape.

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
