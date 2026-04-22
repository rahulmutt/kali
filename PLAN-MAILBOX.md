2026-04-22 resolution note
- added embedding regression coverage for deterministic host-predicate registration order in the programmable-policy façade, so the Stage 5.3 plan now reflects the ordered evaluation path in the embedding layer as well as the sandbox registry
- updated the Stage 5.3 progress note to mention the new registration-order coverage alongside the existing host-predicate context and availability evidence

2026-04-22 resolution note
- added an explicit availability gate to the embedding-side host-predicate registration façade so disabled contexts now fail with the canonical `E5006` diagnostic instead of silently registering inert predicates
- updated the Stage 5.3 progress note so the programmable-policy plan records that explicit rejection path alongside the existing host-predicate façade

2026-04-22 resolution note
- added the deterministic binding-package manifest helper and the Python bundle loader for the higher-level language-binding workflow, then updated the Stage 5.5 progress note so the packaging/distribution shape is now explicit alongside the existing Python ctypes helper

2026-04-22 resolution note
- added browser-bundle smoke coverage for `crypto.subtle.digest` and `crypto.randomUUID`, then updated the Stage 5.4 progress note so the later Web Crypto breadth slice is now pinned at the deployable-through-host boundary as well as in the direct API and package-corpus paths

2026-04-22 resolution note

- tightened the Stage 5.4 Intl gating so `Intl.NumberFormat`-style member chains now hit the canonical `E5006` later-compatibility diagnostic directly, instead of relying only on the outer `Intl` identifier/member root

- updated the Stage 5.4 progress note to record the direct member-access coverage alongside the existing late host-control, object-model, and browser/WebCrypto breadth evidence

2026-04-22 resolution note
- added JSON-envelope regression coverage for unsupported `build --profile` version mismatches, so the Stage 5.5 PGO lane now pins the machine-readable failure path in addition to the existing text-path rejection coverage
- updated the Stage 5.5 progress note to mention the new JSON failure coverage alongside the existing repeated-build determinism and version-gated loader evidence

# PLAN mailbox

2026-04-22 resolution note
- emitted the stem-specific binding-package sidecar from `build --capi` and added CLI smoke coverage for the generated binding-package manifest companion, so the Stage 5.5 binding workflow now carries the package manifest in the actual build output instead of only in the helper library
- updated the Stage 5.5 progress note to call out the manifest sidecar and the new smoke coverage alongside the existing Python binding scaffold

2026-04-22 resolution note
- added deterministic packaging metadata for the maintained Python binding (`bindings/python/pyproject.toml` plus `README.md`) and a regression test that pins the distributable package surface, so the Stage 5.5 binding workflow now has an explicit package/distribution scaffold alongside the existing binding helper

2026-04-22 resolution note
- widened the Stage 5.4 browser package-corpus baseline with SHA-384 / SHA-512 `crypto.subtle.digest` coverage so the later Web Crypto breadth slice now mirrors the broader direct API tests in the package-evidence lane
- updated the Stage 5.4 progress note to record the broader digest coverage alongside the existing browser bundle smoke and direct API evidence

2026-04-22 resolution note
- clarified the Stage 5.5 PGO plan note so the build-only `--profile` input is now called out as an explicit opt-in in the stage's progress tracking, matching the CLI and maturity docs

2026-04-22 resolution note
- added late object-model gating for `Proxy`, `WeakMap`, `WeakSet`, and `FinalizationRegistry` in the resolver, plus `proxy-traps` dynamic-reason detection in the effect-analysis path, so Stage 5.4 now records the new later-compatibility evidence alongside the existing host-control gate
- updated the Stage 5.4 progress note and status to reflect that the stage is now in progress rather than purely planned

2026-04-22 resolution note
- broadened the browser support library with deterministic `crypto.subtle.digest` support for SHA-256/SHA-384/SHA-512, keeping the implementation pure Rust and leaving the broad Web Crypto maturity row unchanged because it already covers the later-compatibility slice
- updated the Stage 5.4 progress note to record the new digest helper alongside the existing late `Intl`/object-model gating evidence

2026-04-22 resolution note
- added CLI smoke coverage for `kali build --profile` so repeated profile-guided builds stay byte-stable across invocations and unsupported profile-data versions fail through the command path as well as the lower-level loader
- updated the Stage 5.5 progress note to record the new command-level profile-data evidence alongside the existing optimizer and binding groundwork

2026-04-22 resolution note
- added deterministic optimization-report helpers in `kali_optimize` so callers can distinguish attached profile data from hot-function inlining usage, and added regression coverage for the no-profile, cold-profile, and hot-profile cases
- wired `--profile` through the CLI build path with version-checked loading and a profile-fingerprint cache key, and updated the Stage 5.5 progress note so the PGO lane now records the explicit report/diagnostic surface alongside the existing profile-data and binding groundwork

2026-04-22 resolution note
- added deterministic host ABI metadata parsing and compatibility checks to the Python ctypes binding helper, so the Stage 5.5 language-binding lane now validates `cabi-metadata` version windows before exposing exports
- updated the Stage 5.5 progress note to record the metadata-gated binding path alongside the existing Python wrapper helper

2026-04-22 resolution note
- added a profile-guided inlining hook in `kali_optimize` so hot functions recorded in deterministic profile data now widen the inlining budget instead of leaving the PGO lane as collection-only plumbing
- updated the Stage 5.5 progress note to record that the later PGO lane now affects a concrete optimizer decision

2026-04-22 resolution note
- added the deterministic PGO profile-data format and optimizer plumbing in `kali_optimize`, so the Stage 5.5 plan now records the first collection/normalization groundwork for the later PGO lane
- updated the Stage 5.5 plan file to mark the stage as in progress and to call out the new normalization and JSON round-trip coverage

2026-04-22 resolution note
- added a direct runtime-layer browser API-surface gate in `kali_runtime` so `RuntimeCtx::execute` and `RuntimeCtx::execute_tests` now reject `browser` with the canonical feature-unavailable diagnostic before instantiating the native standalone path
- updated the Stage 5.2 progress note so the historical plan records the runtime-layer honesty check alongside the existing CLI browser gate coverage

2026-04-22 follow-up note
- added internal release-only and release-and-decrement live-reference ownership/allocation + linear-memory companion lemmas in `proofs/KaliCore/Safety.lean`, and noted that proof-tree factoring in the Stage 4.2 plan/status tracker while keeping the published boundary unchanged

2026-04-22 resolution note
- added explicit and inherited browser API-surface coverage for `kali effects`, so the public effect-report surface now exercises the browser analysis context that the maturity matrix already documents
- updated the Stage 2.2 progress note to record that browser-analysis coverage alongside the existing effect-report evidence

2026-04-22 resolution note
- tightened the Stage 5.2 browser-runtime rejection path so `test` now uses the shared browser-runtime helper instead of a generic one-off gate, and the diagnostic text names the browser API surface explicitly for both `run` and `test`
- added sandbox-attached browser-gate regressions for `run` and `test` in both text and JSON modes, then updated the Stage 5.2 progress note so the historical plan records the new honesty coverage

2026-04-22 resolution note
- centralized runtime-profile normalization in `kali_runtime` and reused the shared helper from the CLI runtime-profile resolver plus incremental cache-key derivation, so the threaded-profile canonicalization path now lives in one place across the runtime-adjacent callers
- updated the Stage 5.1 progress note to record the shared-helper rollout alongside the existing runtime-profile validation and preservation coverage

2026-04-22 resolution note
- added JSON-envelope regressions for `effects --wasm-threads` and inherited `runtimeProfiles = ["wasm-threads"]` so the Stage 5.1 threaded-profile gate now stays machine-readable on the effect-report surface as well as `check` / `run` / `test` / `package-effects`
- updated the Stage 5.1 progress note to mention the JSON `effects` gate coverage alongside the existing text and JSON smoke coverage on the other later-threaded surfaces

2026-04-21 resolution note
- added parser/lexer regression coverage for the semver probe's optional-chaining and multiline-template cases so `minVersion(... )?.version` and multi-line template bodies stay covered by the evidence suite
- updated the Stage 1.3 parser/AST follow-up notes and the Stage 1.14 evidence-hardening progress note to reflect that coverage

2026-04-21 resolution note
- added a `package-effects` smoke regression for inherited `runtimeProfiles = ["wasm-threads"]` so the Stage 5.1 threaded-profile handoff now has CLI-level package-analysis coverage in addition to the existing config/metadata/embedding/runtime checks
- updated the Stage 5.1 progress note to mention the new package-analysis evidence lane alongside the earlier runtime-profile gating coverage

2026-04-21 resolution note
- added canonical thread-spawn bookkeeping plumbing to the sandbox/runtime layer, including a dedicated `HostOperation::ThreadSpawn` policy check and runtime host-state thread counters, so the later threaded-profile budget path has a shared enforcement hook in place even while the profile remains gated
- recorded the plumbing in the Stage 5.1 progress note so the historical plan stays aligned with the implementation state

2026-04-21 resolution note
- added the CLI smoke regression `test_accepts_positive_spawned_process_budget_override` so the Phase-3 host-capability milestone now has symmetric acceptance evidence for `--max-spawned-processes` on both `run` and `test`
- updated the Stage 3.4 progress note to record the new acceptance coverage alongside the existing resource-limit handoff

2026-04-21 resolution note
- updated the Stage 4.2 progress note and schema-doc anti-drift guard so the explicit linear-memory companion theorems `KaliCore.Safety.releaseAndDecrementTargetCellPositiveCountIffAndLinearMemory` and `KaliCore.Safety.releaseAndCollectTargetCellPresentIffPositiveCountAndLinearMemory` stay pinned in the plan/test synchronization path
- kept the canonical proof-backed summary unchanged while tightening the theorem inventory regression to the target-cell split companions

2026-04-21 resolution note
- added a JSON-envelope regression for the later-threaded runtime-profile gate so `check --output json --wasm-threads` now asserts the canonical `E5006` diagnostic in machine-readable output as well as the text envelope
- recorded that coverage in the Stage 5.1 progress note so the historical plan matches the new evidence lane

2026-04-21 resolution note
- added a deterministic `Deno.Command` helper to `kali_api_deno` with captured stdout/stderr output and a regression test, then recorded the progress in the Stage 3.4 host-capability expansion note
- no spec edit was required because `Deno.Command` is already described as a Phase-3 target in the owning standard-APIs chapter

2026-04-21 resolution note
- updated the Stage 5.1 progress note so it explicitly names the `runtimeProfiles` metadata axis and the current empty-set normalization in emitted build metadata

2026-04-21 follow-up note

While hardening the threaded-runtime profile validation path, the CLI smoke suite still needs one end-to-end regression for duplicate `compilerOptions.runtimeProfiles` entries inherited from `kali.json` so the shared config loader is covered at the command level rather than only in the lower-level build/embed helpers.

Proposed fix:
- add a `kali build` smoke test that writes a manifest with duplicate `runtimeProfiles` entries and asserts the canonical `E5009` rejection path
- mention the new CLI-level duplicate-entry coverage in the Stage 5.1 progress note so the historical plan matches the regression suite

2026-04-21 resolution note
- added the CLI smoke regression for duplicate inherited `runtimeProfiles` entries and updated the Stage 5.1 progress note to mention the new command-level coverage

2026-04-21 follow-up note

While hardening the semver/browser bundle path and aligning the proof-summary docs, the schema-doc regression suite revealed that `plan/phase-4/02-formal-verification-depth.md` still lacks the canonical proof-backed summary text and theorem inventory snippets that are already used in `README.md`, `proofs/BOUNDARY.md`, and the spec chapters.

Proposed fix:
- add the canonical short summary to the Stage 4.2 plan file
- mirror the proof-summary theorem inventory snippets already pinned in the spec set so the plan prose stays in sync with the repository's proof-backed boundary wording

2026-04-21 resolution note
- added a `Current post-completion follow-up lanes` section to `PLAN.md` so the closed-stage follow-up pointers now land on real headings
- added `Remaining Work` sections to the Stage 3.1, Stage 3.3, and Stage 4.2 plan files so the TODO and plan references stay synchronized
- kept the canonical proof-backed summary string unchanged and explicitly pinned in the Stage 4.2 follow-up lane

2026-04-21 follow-up note
- widened the Stage 4.2 proof summary with `KaliCore.Safety.releaseAndCollectLiveRefsAreOwnedAndAllocatedAndLinearMemory`, and mirrored that theorem in the Stage 4.2 plan/status tracker so the collection-helper live-reference slice stays aligned with the explicit linear-memory payload

2026-04-21 follow-up note

While implementing the next test-evidence task, the phase-2 planning docs need to move `kali test --coverage` from the aspirational backlog into the executed milestone set and reflect the narrower function-level coverage contract that the code now provides.

Proposed fix:
- update `plan/phase-2/05-test-coverage-and-reporting.md` to record the delivered function-level coverage contract and keep the stage prose aligned with the shipped payload shape
- keep `PLAN.md` aligned with that stage status so the Phase-2 milestone map does not still describe coverage as only planned

2026-04-21 resolution note
- updated the Phase 2 milestone row and completion gate in `PLAN.md` to describe the shipped function-coverage contract explicitly
- tightened `plan/phase-2/05-test-coverage-and-reporting.md` so the historical stage prose now names the canonical function-level payload instead of implying a broader backlog

2026-04-21 follow-up note

While hardening the function-coverage report path, the completed Stage 2.5 coverage milestone should record the normalized/sorted per-file coverage output contract in its progress note so the historical implementation playbook matches the machine-readable result shape.

Proposed fix:
- update `plan/phase-2/05-test-coverage-and-reporting.md` progress notes to mention normalized coverage-report paths and deterministic ordering

2026-04-21 resolution note
- updated the Stage 5.1 progress note so it explicitly names the `runtimeProfiles` metadata axis and the current empty-set normalization in emitted build metadata

2026-04-21 follow-up note
- the shared runtime-profile validation path now also feeds the embedding compiler config and artifact metadata emission, so the Stage 5.1 progress note should mention that duplicate and unknown runtime-profile entries are rejected deterministically before the phase gate

2026-04-21 resolution note
- the shared runtime-profile validation path now also feeds the embedding compiler config and artifact metadata emission, so the Stage 5.1 progress note should mention that duplicate and unknown runtime-profile entries are rejected deterministically before the phase gate

2026-04-21 resolution note
- the Stage 2.5 progress note already records the deterministic multi-file ordering contract for `kali test --coverage`, including the reversed explicit file-input regression case and the normalized/sorted per-file coverage output contract, so no further plan churn was needed

2026-04-21 resolution note
- updated the Stage 2.5 progress note so it now calls out the reversed explicit file-input regression case, the deterministic coverage file ordering contract, and the new filter-aware coverage regression that keeps `kali test --coverage --filter ...` pinned to the selected test files only

2026-04-21 follow-up note

While preparing the threaded-runtime profile handoff, the resolver still reports `SharedArrayBuffer` and `Atomics` as plain undefined names instead of the canonical feature-gating diagnostic.

Proposed fix:
- teach the name resolver to emit the shared `E5006` availability diagnostic for those two later-compatibility globals so the first threaded primitives fail with the right maturity code rather than a generic undefined-name error
- add resolver-level coverage for the new diagnostics and record the threaded-profile progress in the Stage 5.1 plan note

2026-04-21 resolution note
- the threaded-runtime profile now has resolver-level coverage for `SharedArrayBuffer` and `Atomics`, which surface the canonical `E5006` later-compatibility diagnostic instead of falling back to a generic undefined-name error
- updated the Stage 5.1 progress note so the historical plan records the first language-visible threaded globals explicitly

2026-04-21 resolution note
- confirmed the Stage 5.1 progress note already covers the shared runtime-profile validation path, the embedding/config handoff, and the duplicate/unknown-entry rejection coverage, so the unresolved follow-up note is now closed without further plan churn

2026-04-21 follow-up note
- the threaded-runtime resolver still misses `globalThis.SharedArrayBuffer` / `globalThis.Atomics`, so the stage-5.1 progress note should be updated if we add member-expression gating for those language-visible primitives

2026-04-21 resolution note
- added member-expression gating for `globalThis.SharedArrayBuffer` / `globalThis.Atomics` and updated the Stage 5.1 progress note so the threaded-profile handoff records the broader language-visible primitive coverage

2026-04-21 resolution note
- updated the Stage 5.1 progress note to mention that runtime-profile validation now also preserves the selected runtime-profile vector into the runtime execution context / host-side runtime state

2026-04-21 follow-up note

While widening the Stage 4.2 proof inventory, the historical plan tracker should note the new linear-memory companion theorems that package the target-cell positive-count split for the release-and-decrement and release-and-collect paths.

Proposed fix:
- update `plan/phase-4/02-formal-verification-depth.md` and `PLAN-4.2-STATUS.md` to mention `KaliCore.Safety.releaseAndDecrementTargetCellPositiveCountIffAndLinearMemory` and `KaliCore.Safety.releaseAndCollectTargetCellPresentIffPositiveCountAndLinearMemory`
- keep the canonical short summary unchanged while expanding the theorem inventory wording to match the Lean proof tree

2026-04-21 resolution note
- confirmed the Stage 4.2 plan file and tracker already name the target-cell split companions explicitly, so the published proof inventory remains synchronized without changing the canonical short summary

2026-04-21 resolution note
- normalized set-like runtime-profile axes in the effect-report and runtime host contexts so noisy caller vectors are deduplicated and sorted before emission or host-state construction
- updated the Stage 5.1 progress note to record the canonicalized runtime-profile path alongside the existing CLI/config/metadata/runtime gating coverage

2026-04-21 resolution note
- updated the Stage 4.2 plan file, `PLAN-4.2-STATUS.md`, `proofs/BOUNDARY.md`, and `README.md` so the target-cell split companions are now named explicitly alongside the proof-backed boundary summary
- kept the canonical short summary unchanged

2026-04-21 follow-up note

While implementing Phase-3 subprocess support, the Stage 3.4 plan note still says the remaining resource-limit handoff is tracked here. Once the runtime host-spawn path and `--max-spawned-processes` invocation budget are actually wired through, the stage should be marked complete and the historical progress note should call out the accepted positive-cap behavior instead of the provisional rejection state.

Proposed fix:
- update `plan/phase-3/04-host-capability-expansion.md` to record the runtime `maxSpawnedProcesses` handoff, the accepted positive-cap budget path, and the corresponding stage completion status
- keep `PLAN.md` aligned if the Phase-3 milestone summary needs a one-line completion note

2026-04-21 resolution note
- updated the Stage 3.4 host-capability-expansion note to record the runtime `maxSpawnedProcesses` handoff, positive-cap acceptance, and stage completion status

2026-04-21 follow-up note
- while hardening the threaded-runtime handoff, the resolver-level `SharedArrayBuffer` / `Atomics` coverage still lacked an end-to-end CLI smoke regression for the language-visible `globalThis.SharedArrayBuffer` / `globalThis.Atomics` forms

Proposed fix:
- add `kali check` and JSON smoke coverage for `globalThis.SharedArrayBuffer` and `globalThis.Atomics` so the Stage 5.1 plan note can point at an end-to-end regression instead of only the lower-level resolver tests

2026-04-21 resolution note
- added end-to-end CLI smoke regressions for `globalThis.SharedArrayBuffer` and `globalThis.Atomics` in both text and JSON `kali check` output, so the Stage 5.1 threaded-global story now has user-visible smoke coverage in addition to the lower-level resolver tests
- updated the Stage 5.1 progress note to mention the new CLI smoke coverage alongside the existing resolver-level gating path

2026-04-21 resolution note
- the shared runtime-profile validation path now also rejects invalid entries from the public compile-source helper before cache lookup, so direct callers see the same deterministic duplicate/unknown rejection path as the CLI, metadata, embedding, runtime, and package-analysis entrypoints
- updated the Stage 5.1 progress note to record the helper-level validation path alongside the existing runtime-profile plumbing and gating coverage

2026-04-21 resolution note
- extended the runtime execution coverage so both `execute` and `execute_tests` preserve the normalized runtime-profile vector in `RuntimeOutcome`, and updated the Stage 5.1 progress note to call out the outcome-level preservation across both execution return paths

2026-04-21 follow-up note
- the Stage 5.1 thread-budget story still lacks a CLI smoke regression for sandbox-attached policies that set positive `resources.maxThreads`, so the plan note should record that policy-level rejection path once we add the command-level test

Proposed fix:
- add a `kali check --sandbox` smoke test that writes a policy with `resources.maxThreads: 1` and asserts the canonical `E5006` / `resources.maxThreads` rejection path
- mention the new policy-level CLI smoke coverage in the Stage 5.1 progress note so the historical plan reflects the command-level evidence

2026-04-21 resolution note
- added the `kali check --sandbox` smoke regression for positive `resources.maxThreads` policies and updated the Stage 5.1 progress note so the historical plan now records the command-level `E5006` / `resources.maxThreads` coverage alongside the existing runtime and direct-override tests

2026-04-21 resolution note
- added the `kali test --sandbox` smoke regression for positive `resources.maxThreads` policies so the Stage 5.1 evidence story now covers the policy-driven thread-budget rejection path on both execution commands
- updated the Stage 5.1 progress note to mention the `check` and `test` command-level `E5006` / `resources.maxThreads` coverage together

2026-04-21 follow-up note

While advancing the threaded-runtime-profile work, the browser/web compatibility crate needs a deterministic shared-memory baseline so later `SharedArrayBuffer` / `Atomics` plumbing can reuse one stable byte-buffer model instead of inventing a second ad hoc helper.

Proposed fix:
- add a small deterministic `SharedArrayBuffer` / `Atomics` baseline to `crates/kali_api_web`
- keep it internal to the runtime compatibility layer for now, with tests that prove clone-sharing and atomic byte updates are deterministic
- mention the new baseline in the Stage 5.1 progress note so the historical plan stays aligned with the implementation state

2026-04-21 resolution note
- added the deterministic `SharedArrayBuffer` / `Atomics` baseline to `kali_api_web` with clone-shared byte storage and byte-wise atomic helpers
- updated the Stage 5.1 progress note so the historical plan now records the shared-memory baseline alongside the existing threaded-profile plumbing and gating evidence

2026-04-21 resolution note
- added deterministic ordered post queues to the `kali_api_web` worker and broadcast-channel stubs so shared buffers stay first-class transport payloads instead of being flattened into JSON-only bookkeeping
- updated the Stage 5.1 progress note to record the new shared-buffer coordination baseline and the ordered transport queue

2026-04-21 follow-up note

While hardening the threaded runtime-topology model, the browser/runtime compatibility crate still needs an explicit regression for mixed live/terminated instances so the shutdown/leak report ordering stays deterministic when teardown does not happen in spawn order.

Proposed fix:
- add a `ThreadRuntimeTopology` regression that spawns multiple workers, terminates a middle instance first, and asserts the shutdown report keeps the remaining live snapshots in deterministic instance-id order
- mention the new ordering coverage in the Stage 5.1 progress note so the historical plan stays aligned with the evidence lane

2026-04-21 resolution note
- added the mixed live/terminated `ThreadRuntimeTopology` shutdown-order regression in `kali_api_web`
- updated the Stage 5.1 progress note to mention the deterministic shutdown ordering coverage alongside the existing threaded-topology baseline

2026-04-21 resolution note
- added a second `ThreadRuntimeTopology` shutdown-order regression in `kali_api_web` that terminates the first worker before shutdown, so the mixed live/terminated evidence now covers a stronger out-of-spawn-order teardown case
- updated the Stage 5.1 progress note to mention the terminated-first variant alongside the existing mixed live/terminated shutdown-order coverage

2026-04-21 resolution note
- added browser-targeted `check --api browser --wasm-threads` smoke coverage in both text and JSON forms so the canonical later-threaded gate stays visible on the browser analysis path as well as the default source-graph path
- updated the Stage 5.1 progress note to mention the new browser-targeted regression alongside the existing runtime-profile and package-analysis coverage

2026-04-21 follow-up note

While tightening the browser-runtime gate evidence, the CLI smoke suite still lacks JSON-output regressions for the existing `run` and `test` browser rejections. The text-path checks are already in place, but the machine-readable envelope should mirror them so the unsupported later-compatibility gate stays deterministic across both output formats.

Proposed fix:
- add `--output json` regressions for explicit and inherited browser API surfaces on `kali run` and `kali test`
- mention the new JSON coverage in the Stage 5.2 progress note so the historical plan records the machine-readable gate as well as the text gate

2026-04-21 resolution note
- added JSON-envelope regressions for explicit and inherited browser API surfaces on `kali run` and `kali test`, so the unsupported later-compatibility gate now has machine-readable coverage alongside the existing text-path smoke tests
- updated the Stage 5.2 progress note to mention the new JSON browser-gate coverage alongside the existing browser-runtime rejection helper and text smoke coverage

2026-04-21 resolution note
- added JSON-envelope regression coverage for the `run` browser gate when guest arguments are present after `--`, so the new command-shape split stays pinned even on the rejected later-compatibility path
- updated the Stage 5.2 progress note to mention the guest-argument separator coverage alongside the existing browser-runtime rejection helper and JSON smoke coverage

2026-04-21 follow-up note
- the threaded-runtime workspace still needs a regression that shared-buffer posts are ignored after a worker or broadcast channel has been terminated/closed, so the post-shutdown behavior stays symmetrical with the existing post-message ignore coverage

Proposed fix:
- add worker and broadcast-channel tests that post a `SharedArrayBuffer` after termination/close and assert the buffered shared payload list does not change
- mention the new post-close shared-buffer coverage in the Stage 5.1 progress note so the historical plan reflects the runtime shutdown semantics explicitly

2026-04-21 resolution note
- added worker and broadcast-channel regressions that ignore shared-buffer posts after termination/close, so the threaded-runtime shutdown semantics now match the existing post-message ignore coverage
- updated the Stage 5.1 progress note to record the post-close shared-buffer symmetry alongside the existing threaded-topology and shared-memory baseline coverage

2026-04-21 follow-up note

While hardening the Stage 5.1 thread-budget evidence, the sandbox-attached positive `resources.maxThreads` smoke coverage still only exists in text mode. The machine-readable envelope should mirror the same policy rejection so the later-threaded gate stays deterministic across CLI output formats.

Proposed fix:
- add `--output json` regressions for `kali check --sandbox` and `kali test --sandbox` when the attached policy sets `resources.maxThreads: 1`
- mention the new JSON coverage in the Stage 5.1 progress note so the historical plan reflects both output formats

2026-04-21 resolution note
- added JSON-envelope regressions for the sandbox-attached positive `resources.maxThreads` policy path on both `check` and `test`, so the Stage 5.1 thread-budget evidence now mirrors the existing text-path coverage in machine-readable output as well
- updated the Stage 5.1 progress note to mention the text/JSON symmetry alongside the existing resolver, runtime, and policy-level gating coverage

2026-04-22 resolution note
- canonicalized runtime-profile emission in `kali_runtime` so store construction and runtime outcomes now normalize the public `runtime_profiles` field even if a direct API caller mutates it after construction
- recorded the runtime-profile normalization follow-up in the Stage 5.1 progress note so the historical threaded-profile plan stays aligned with the executable contract

2026-04-22 resolution note
- closed the Stage 5.1 plan checklist by marking the threaded runtime-profile stage complete now that the canonical runtime-profile helper, shared validation path, and regression coverage are already in place
- left the canonical later-compatibility gates unchanged; this is a plan-status update only, not a maturity change

2026-04-22 resolution note
- added the Stage 5.3 progress note and the new embedding-side host-predicate API groundwork in `kali_sandbox` / `kali_embed`, so the programmable-policy plan now records the canonical predicate context, deterministic registration order, and declarative-first evaluation path

2026-04-22 resolution note
- extended the host-predicate coverage with deterministic `resources.maxThreads` / `activeThreads` context details, plus matching sandbox and embedding regressions, so the Stage 5.3 plan note now pins the host-specific detail payload in addition to the shared capability/subject context

2026-04-22 resolution note
- added late host-control gating for `Deno.pid` / `Deno.cwd` / `Deno.chdir` / `Deno.exit` and the corresponding `process.*` member accesses in the type resolver, so the stage-5.4 follow-up lane now has a canonical `E5006` availability diagnostic instead of a generic property lookup path
- updated the Stage 5.4 progress note to record the new availability gate coverage alongside the rest of the planned late host/object compatibility work

2026-04-22 resolution note
- documented the explicit build-only `--profile` PGO input in the CLI and maturity docs, and updated the Stage 5.5 progress note so the later PGO lane now records the flag as an explicit opt-in build input rather than a hidden implementation detail

2026-04-22 resolution note
- Stage 5.4 follow-up: align the late Web/Intl breadth story by gating `Intl` (direct `Intl`, `globalThis.Intl`, and member accesses such as `Intl.NumberFormat`) with the canonical `E5006` later-compatibility diagnostic, and add regression coverage alongside the existing Proxy/WeakMap/FinalizationRegistry checks.
- Added JSON-envelope regression coverage for the late Intl gate so the CLI smoke suite now pins the same `E5006` availability diagnostic in both text and machine-readable output.
- No spec wording change needed if the implementation stays a rejection/gating alignment; the existing later-compatibility rows in `specs/11-standard-apis.md` and `specs/19-feature-maturity.md` already own the availability claim.

2026-04-22 resolution note
- added CLI smoke coverage for direct and `globalThis`-qualified `WeakMap`, `WeakSet`, and `FinalizationRegistry` rejections in both text and JSON `check` output, so Stage 5.4 now pins the late object-model gate surface alongside the existing host-control, Proxy, Intl, and Web Crypto regressions
2026-04-22 follow-up note
- the Stage 5.4 browser package-corpus baseline should be widened to exercise `crypto.subtle.digest` so the later Web Crypto breadth slice has package-evidence coverage alongside the direct API tests
2026-04-22 resolution note
- widened the Stage 5.4 browser package-corpus baseline with `crypto.subtle.digest` coverage and added SHA-384/SHA-512 digest tests in `kali_api_web`, so the later Web Crypto breadth slice now has both direct API and package-evidence coverage

2026-04-22 resolution note
- synchronized the Stage 4.2 plan/status trackers so they now name `KaliCore.Safety.releaseRefLiveRefsAreOwnedAndAllocatedAndLinearMemory` and `KaliCore.Safety.releaseAndDecrementLiveRefsAreOwnedAndAllocatedAndLinearMemory` alongside the collection-helper companion, keeping the follow-up lane aligned with the proof tree

2026-04-22 resolution note
- added bigint-literal lexer/parser support so browser-bundle crypto smoke fixtures can return `0n` without the lexer splitting the suffix into a stray identifier
- recorded the new literal coverage in the Stage 1.3 parser/AST notes and the Stage 1.14 evidence-hardening progress note so the plan history matches the regression suite
2026-04-22 resolution note
- broadened the proxy dynamic-effect coverage so `Proxy.revocable(...)` and `globalThis.Proxy.revocable(...)` now share the existing `proxy-traps` reason alongside `new Proxy(...)`, and updated the Stage 5.3 progress note so the historical plan records the expanded proxy semantics slice
- marked Stage 5.3 as in progress so the plan status reflects that the proxy semantics slice now has delivered implementation evidence rather than remaining purely planned

2026-04-22 resolution note
- added the public embedding-side host-predicate registration façade in `kali_embed` (`OperationContext`, `PredicateDecision`, and `EmbeddingCtx::register_sandbox_predicate` / `check_operation_with_policy`), so the Stage 5.3 programmable-policy lane now has a concrete host API layered over the canonical sandbox predicate vocabulary
- updated the Stage 5.3 plan note to record the new embedding registration surface alongside the existing sandbox predicate context and declarative-first evaluation path

2026-04-22 resolution note
- widened the Stage 5.4 late host-control smoke coverage so the CLI regression story now includes the process-side member forms (`process.pid`, `process.cwd`, `process.chdir`, and `process.exit`) alongside the existing Deno-side gates
- updated the Stage 5.4 progress note to record that broader host-control coverage in the historical plan

2026-04-22 resolution note
- added an embedding regression that exercises the threaded-budget context detail path through `resources.maxThreads` / `activeThreads`, so host-registered predicates now have direct coverage for the later thread-budget vocabulary in addition to the existing console policy cases
- updated the Stage 5.3 progress note to mention the direct `ThreadSpawn` narrowing regression alongside the existing host-predicate context and availability coverage

2026-04-22 resolution note
- added a default-standalone semver package-bin regression that pins the existing `require(../package.json)` gate on the package-json path, leaving the remaining Node-path help/argument follow-up explicitly open
- updated the Stage 3.2 progress note so it now records the semver package-json rejection alongside the browser-bundle hardening without overclaiming the unfinished Node-path fixture

2026-04-22 resolution note
- renamed the Node-path semver smoke fixture so it now honestly describes the current bare `--api node` package-bin execution coverage instead of implying guest-argument or package-json support that the runtime path still does not provide
- updated the Stage 3.2 progress note to say the Node-path fixture is intentionally bare while the help/argument and package-json follow-up remains open

2026-04-22 resolution note
- added a dedicated Python `unittest` smoke harness under `bindings/python/tests/` and wired it into the Rust workspace test suite, so the maintained non-Rust binding now has first-class end-to-end smoke coverage instead of only ad hoc inline scripts
- updated the Stage 5.5 progress note to record the new Python smoke harness alongside the existing Python ctypes helper and packaging/distribution metadata
