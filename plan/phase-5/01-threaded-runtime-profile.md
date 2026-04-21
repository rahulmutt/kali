# Stage 5.1 — Threaded Runtime Profile

**Phase:** 5 — Later Compatibility & Platform Expansion  
**Spec refs:** [`specs/10-runtime.md`](../../specs/10-runtime.md), [`specs/09-sandboxing.md`](../../specs/09-sandboxing.md), [`specs/06-memory.md`](../../specs/06-memory.md), [`specs/12-cli.md`](../../specs/12-cli.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)  
**Depends on:** [1.8 — Runtime & Execution](../phase-1/08-runtime-execution.md), [2.1 — MIR & Ownership Analysis](../phase-2/01-mir-and-ownership.md), [4.1 — Dynamic Compatibility](../phase-4/01-dynamic-compatibility.md) for the fully stabilized runtime/profile gating path

## Goal

Introduce the later opt-in threaded runtime profile without weakening Kali's single-linked-payload,
AOT-only, and no-tracing-GC invariants. This stage owns the runtime-profile plumbing behind
`--wasm-threads` / `runtimeProfiles = ["wasm-threads"]`, plus the first supported
`SharedArrayBuffer` / `Atomics` surface and thread-budget enforcement.

## Workable Milestone

- `kali run --wasm-threads main.ts` and equivalent inherited-config forms work on supported
  targets and fail explicitly on unsupported ones.
- `SharedArrayBuffer` and `Atomics` behave correctly inside the documented threaded profile.
- Thread creation and worker/runtime-instance startup obey `resources.maxThreads` and the
  zero-cap tightening rule.
- CLI, config, embedding, diagnostics, and effect/sandbox handling all agree on one canonical
  threaded-profile contract.

## Progress

- CLI and config parsing now recognize the documented `--wasm-threads` / `compilerOptions.runtimeProfiles` axis, normalize `wasm-threads`, and reject it explicitly with the canonical `E5006` gate until the profile actually opens.
- Effect-report context plumbing now carries `runtimeProfiles` and `compatFeatures` through the shared analysis context so later evidence can reuse the same axis-aligned shape.
- The embedding config surface now retains runtime-profile requests and fails them explicitly in the current phase instead of silently dropping them.
- Build artifact metadata sidecars now carry an explicit `runtimeProfiles` axis, normalizing the current phase to the empty `[]` set while still threading the effective runtime-profile vector through executable, library, C-ABI, component, and browser-bundle metadata emission so the emitted artifact contract stays aligned with the same semantic knob even though the current phase still rejects threaded requests before artifact production.
- The runtime-profile validation path is now shared between CLI config loading, artifact metadata emission, and the embedding compiler config, so duplicate or unknown runtime-profile entries are rejected deterministically before the threaded-profile phase gate is applied; regression coverage now pins both the duplicate and unknown-entry cases across the CLI, metadata, and embedding surfaces, including CLI smoke regressions for duplicate `kali.json` runtime-profile entries plus the explicit and inherited `run` / `test` `--wasm-threads` rejection paths.
- The CLI now accepts the zero-cap thread-budget overrides documented for execution commands, with `--max-threads` / `--max-spawned-processes` normalizing through the shared `resources.*` vocabulary and rejecting positive values with the canonical `E5006` gate until their phase/profile contracts open.

## Tasks

### 1. Runtime-profile plumbing

Implement one canonical runtime-profile path across:

- CLI `--wasm-threads`
- `kali.json#compilerOptions.runtimeProfiles`
- embedding config builders / enums
- emitted analysis/artifact metadata
- canonical `E5006` gating before the profile is available or on unsupported targets

The runtime must not silently degrade a threaded request back to the single-threaded baseline.

### 2. Shared memory model

Add the runtime and memory-layer machinery needed for shared memory:

- shared linear-memory / shared-buffer strategy for the threaded profile
- per-thread stack and allocator state
- safe interaction between shared-heap ownership and deterministic reference counting
- explicit rules for values that may cross thread boundaries versus values that must remain local

This stage should reuse the existing ownership-class vocabulary (`stack`, `owned heap`,
`shared heap`, `borrowed`) instead of inventing a separate thread-only memory model.

### 3. `SharedArrayBuffer` / `Atomics`

Open the first language-visible threaded primitives:

- `SharedArrayBuffer`
- `Atomics` operations backed by WASM atomics
- structured-clone or worker-message rules for cross-runtime-instance coordination

Implementation must preserve the spec's opt-in-only stance: these features are not part of the
default runtime profile.

### 4. Thread-budget and sandbox enforcement

Make the sandbox/runtime contract thread-aware:

- enforce `resources.maxThreads`
- preserve the `0` means explicit deny/tightening rule
- reject positive thread budgets before the threaded profile is actually active
- ensure policy validation, runtime enforcement, and diagnostics all use the same vocabulary

### 5. Worker/runtime-instance execution model

Define the first supported threaded execution topology:

- one Kali runtime instance per worker/thread
- documented startup / teardown / message-passing semantics
- deterministic shutdown and leak reporting behavior for threaded runs

This stage should keep browser-worker semantics and standalone browser runtime support out of scope
unless they are explicitly added in a later stage.

### 6. Tests

- positive threaded-profile integration tests on supported engines/targets
- negative tests for `--wasm-threads` on unsupported targets
- `SharedArrayBuffer` / `Atomics` correctness fixtures
- policy/CLI/resource-limit tests for `resources.maxThreads`
- determinism and race-regression coverage for runtime startup/shutdown paths

## Out of Scope

- standalone browser worker/runtime parity
- weak references / finalization APIs
- alternative execution engines beyond the canonical backend contract
- broader later-compatibility object-model work tracked in Stage 5.4

## Status

Planned.
