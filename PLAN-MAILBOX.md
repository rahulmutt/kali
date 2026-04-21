# PLAN mailbox

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
- extended the runtime execution coverage so both `execute` and `execute_tests` preserve the normalized runtime-profile vector in `RuntimeOutcome`, and updated the Stage 5.1 progress note to call out the outcome-level preservation across both execution return paths
