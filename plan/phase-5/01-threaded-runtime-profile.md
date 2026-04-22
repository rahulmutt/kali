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
- Effect-report context plumbing now carries `runtimeProfiles` and `compatFeatures` through the shared analysis context so later evidence can reuse the same axis-aligned shape. The shared analysis-context builder now normalizes both axes before report emission, keeping the CLI package-analysis path aligned with the same normalized report contract.
- The embedding config surface now retains runtime-profile requests and fails them explicitly in the current phase instead of silently dropping them.
- Build artifact metadata sidecars now carry an explicit `runtimeProfiles` axis, normalizing the current phase to the empty `[]` set while still threading the effective runtime-profile vector through executable, library, C-ABI, component, and browser-bundle metadata emission so the emitted artifact contract stays aligned with the same semantic knob even though the current phase still rejects threaded requests before artifact production.
- The runtime-profile validation path is now shared between CLI config loading, artifact metadata emission, the embedding compiler config, and the runtime execution context, so duplicate or unknown runtime-profile entries are rejected deterministically before the threaded-profile phase gate is applied and the selected runtime-profile vector is preserved all the way into the host-side runtime state; regression coverage now pins both the duplicate and unknown-entry cases across the CLI, metadata, embedding, runtime, and package-analysis surfaces, including CLI smoke regressions for duplicate `kali.json` runtime-profile entries on `build`, `run`, `test`, and `package-effects`, plus the explicit and inherited `run` / `test` `--wasm-threads` rejection paths, the `effects` inherited-runtime-profile rejection path, and the inherited `package-effects` runtime-profile rejection path. The runtime context also now carries the invocation-level thread-budget override through to host-state construction so the later threaded-profile path has one canonical effective-budget resolver instead of splitting policy and CLI limits, and both the effect-report and runtime host contexts now normalize their set-like runtime-profile axes before emission or store construction so machine-readable payloads stay sorted and deduplicated even when callers pass a noisier vector. The runtime execution outcome now also preserves the selected runtime-profile vector on both `execute` and `execute_tests`, so callers can observe the canonical normalized profile list that reached execution without having to reconstruct it from host state. Incremental compile-cache keys now include the normalized runtime-profile axis as well, so future threaded-profile builds will not alias cache entries across runtime-profile configurations. The public compile-source helper now validates runtime-profile inputs before cache lookup and compilation too, so direct callers observe the same deterministic duplicate/unknown rejection path as the CLI, metadata, embedding, runtime, and package-analysis entrypoints. JSON-envelope coverage now also asserts that the later-threaded gate reports the canonical `E5006` diagnostic in machine-readable `check --output json` output instead of only in the text envelope, and the execution-command JSON smoke suite now pins the same canonical `E5006` rejection for positive `--max-threads` overrides on both `run` and `test`. The `effects` command now mirrors that same later-threaded gate in JSON output too, so the reporting surface stays machine-readable on both the source-graph and package-analysis paths. Package-analysis smoke coverage now also exercises the inherited `runtimeProfiles = ["wasm-threads"]` case for `package-effects`, so the package-effects handoff is covered at the CLI level as well as in the lower-level config, metadata, embedding, and runtime tests. Browser-targeted `check` smoke coverage now pairs `--api browser` with `--wasm-threads` in both text and JSON output, so the canonical later-threaded gate stays visible on the browser analysis path as well as the default source-graph path.
- The runtime executor now also canonicalizes the public `runtime_profiles` field at store construction and result emission, so direct API callers who mutate the field after construction still get the same normalized runtime-profile contract as the CLI path.
- `kali_runtime` now exposes a canonical runtime-profile helper for direct callers, and the CLI runtime-profile resolver plus incremental cache-key derivation now reuse that shared helper so the deduplication / trimming logic lives in one place across the runtime-adjacent callers.
- The MIR ownership model now exposes canonical thread-boundary disposition helpers so later threaded-profile work can distinguish `shared heap` values that may cross runtime-instance boundaries from `stack`, `owned heap`, and `borrowed` values that must remain local, and the MIR regression suite now pins those ownership-to-boundary rules directly. The MIR layer now also exposes a deterministic thread-boundary profile summary over each function/program scope so the later worker/runtime plumbing can consume one canonical shareable-vs-local classification instead of reconstructing it ad hoc.
- The shared name-resolution layer now treats `SharedArrayBuffer` and `Atomics` as explicit later-compatibility globals and raises the canonical `E5006` threaded-profile diagnostic instead of a generic undefined-name error, including `globalThis.SharedArrayBuffer` / `globalThis.Atomics` member-access forms, so the first language-visible primitives for the later runtime profile are recognized consistently even though the threaded execution path is still gated off.
- The CLI now accepts the zero-cap thread-budget overrides documented for execution commands, with `--max-threads` / `--max-spawned-processes` normalizing through the shared `resources.*` vocabulary and rejecting positive values with the canonical `E5006` gate until their phase/profile contracts open.
- The sandbox/runtime layer now also has a dedicated thread-spawn bookkeeping hook (`HostOperation::ThreadSpawn`) plus runtime host-state thread counters, so later threaded-profile enforcement can reuse the same canonical budget path as spawned-process accounting instead of inventing a second thread-only limit vocabulary.
- CLI smoke coverage now exercises the resolver-level threaded-global gates end to end for `globalThis.SharedArrayBuffer` and `globalThis.Atomics`, so the Stage 5.1 regression story includes both the lower-level name-resolution checks and a user-visible `kali check` smoke fixture for the language-visible primitives. The same CLI smoke suite also now pins the sandbox-attached policy path that sets positive `resources.maxThreads`, so the command-level evidence covers the canonical `E5006` rejection for policy-driven thread budgets on `check` and `test` alike, now mirrored in both text and JSON envelope output.
- Browser-targeted build smoke coverage now pins the later-threaded gate too: `kali build --bundle --api browser` rejects inherited `runtimeProfiles = ["wasm-threads"]` with the canonical JSON `E5006` envelope instead of silently drifting into the browser bundle lane.
- The browser/runtime compatibility crate now includes a deterministic `SharedArrayBuffer` / `Atomics` shared-memory baseline with clone-shared byte storage and byte-wise atomic helpers, giving the later threaded-profile work one concrete internal memory model to reuse when the runtime gate opens. The atomics helper suite now also covers `exchange`, so the shared-memory baseline exercises the full byte-mutation path rather than just load/store/add/sub/compare-exchange. It also now routes worker and broadcast-channel posts through deterministic shared-buffer queues so cross-runtime-instance coordination preserves the shared backing store instead of flattening the transport back to JSON-only payloads. The same compatibility layer now carries a deterministic threaded runtime-topology model that assigns one runtime instance per worker/thread and produces a stable shutdown/leak report for any instances that were still live at shutdown, and the report now distinguishes instances that were already terminated before shutdown from those still live when teardown begins; the regression suite pins the mixed live/terminated shutdown ordering, including the terminated-first teardown variant, so the snapshot list stays deterministic when teardown does not happen in spawn order. Worker and broadcast-channel shared-buffer posts are now ignored after termination/close, matching the existing post-message shutdown behavior and keeping the shared-buffer path symmetrical.

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
