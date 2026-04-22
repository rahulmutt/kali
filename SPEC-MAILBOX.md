# SPEC mailbox

2026-04-22 resolution note
- clarified the build-only `--profile` PGO input in `specs/12-cli.md` and `specs/19-feature-maturity.md` so the public command surface now calls it out as an explicit opt-in rather than a hidden implementation detail

2026-04-22 follow-up note
- the new CLI build-profile input (`kali build --profile <file>`) should be reflected in the owning CLI and maturity docs so the PGO workflow stays aligned with the public command surface and the `--profile` flag does not remain a hidden implementation detail

2026-04-21 resolution note
- `specs/16-testing.md` already contains the canonical short summary string in the proof-claim discipline section, so no spec edit was required for that follow-up

2026-04-21 follow-up note
- widened the proof-summary inventory with `KaliCore.Safety.releaseAndCollectLiveRefsAreOwnedAndAllocatedAndLinearMemory`, and synchronized the spec summary chapters plus `README.md` / `proofs/BOUNDARY.md` so the live-reference/linear-memory companion stays pinned everywhere the anti-drift guard checks it

2026-04-21 follow-up note

While hardening the semver/browser bundle path, the workspace test suite revealed that `specs/16-testing.md` is missing the canonical proof-backed summary string that is already present in `README.md` and `proofs/BOUNDARY.md`.

Proposed fix:
- add the canonical short summary to the proof-claim discipline section in `specs/16-testing.md`
- keep the wording identical to the repository summary already used elsewhere:
  **"Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target."**

2026-04-21 follow-up note

While implementing the next test-evidence task, the CLI/runtime work surfaced a need to widen the test result contract so `kali test --coverage` can report a stable machine-readable coverage payload instead of remaining permanently gated.

Proposed fix:
- extend `specs/18-schemas.md` with the Phase-2 coverage result shape for `kali test --coverage`
- update `specs/12-cli.md` so `--coverage` is described as the stable test-coverage selector rather than a permanently phase-gated flag
- update `specs/16-testing.md` so the evidence lane explicitly calls for positive `kali test --coverage` coverage
- keep the new contract narrowly scoped to the documented `test` command path and its deterministic machine-readable output

2026-04-21 follow-up note

While hardening the function-coverage report path, the stable schema-v1 test result contract also needs to document that `kali test --coverage` normalizes per-file report paths relative to the effective project root when available and keeps the file list in deterministic order.

Proposed fix:
- update `specs/18-schemas.md` to state the normalized/sorted coverage-report path rule
- keep the implementation and tests aligned with that deterministic output contract

2026-04-21 follow-up note

While wiring runtime-profile metadata through the emitted build artifact contract, the schema notes should eventually mention that artifact metadata now carries a `runtimeProfiles` field so the threaded-profile axis remains explicit even when the current phase still normalizes to the empty set.

Proposed fix:
- update the artifact-metadata portion of `specs/18-schemas.md` if/when the build metadata contract is formalized there
- keep the emitted sidecar metadata and the stage notes aligned on the explicit `runtimeProfiles` axis

2026-04-21 resolution note
- the stable coverage-report path rule is already captured in `specs/18-schemas.md`, and the `runtimeProfiles` axis is already carried in the current schema vocabulary and Stage 5.1 progress note, so no further spec edit was required for these follow-ups

2026-04-21 resolution note
- the RC snapshot summary and the downstream proof-summary docs already name `KaliCore.Safety.releaseAndDecrementTargetCellPositiveCountIffAndLinearMemory` and `KaliCore.Safety.releaseAndCollectTargetCellPresentIffPositiveCountAndLinearMemory` explicitly, so the proof-boundary inventory remains synchronized without a spec edit

2026-04-21 follow-up note

While implementing Phase-3 subprocess support, the shared budget/availability prose still says positive `resources.maxSpawnedProcesses` values must be rejected until subprocess support exists. That wording needs to be updated once the direct invocation cap and runtime host-spawn path actually accept the Phase-3 budget.

Proposed fix:
- update the subprocess budget language in `specs/09-sandboxing.md`, `specs/12-cli.md`, and `specs/18-schemas.md` so the positive-cap rule reflects the implemented subprocess path instead of the pre-support rejection state
- keep the browser-targeted budget-compatibility wording intact so browser checks can still be described separately
- align the maturity wording in `specs/19-feature-maturity.md` only if the availability row itself needs to change

2026-04-21 resolution note
- updated the subprocess budget language in `specs/09-sandboxing.md`, `specs/12-cli.md`, and `specs/18-schemas.md` so the direct `--max-spawned-processes` / `resources.maxSpawnedProcesses` path now reflects the implemented subprocess budget handoff rather than the pre-support rejection wording

2026-04-21 follow-up note

While implementing the `kali run <file> [-- args...]` split, the CLI surface text in `specs/12-cli.md` still documents `kali run <file>` without the guest-argument separator, so the spec chapter should be updated to match the now-implemented command shape and the Node-path `process.argv` / default-surface argument routing.

Proposed fix:
- update `specs/12-cli.md` so the `kali run` command shape is documented as `kali run <file> [-- args...]`
- keep the availability wording unchanged; this is a shape/documentation update, not a maturity change
- preserve the existing node-gating language while making the guest-argument flow explicit

2026-04-21 resolution note
- `specs/12-cli.md` and `specs/19-feature-maturity.md` now both describe the `kali run <file> [-- args...]` command shape, so the CLI shape is synchronized across the command chapter and the maturity matrix

2026-04-22 resolution note
- documented the build-only `--profile` PGO input in `specs/12-cli.md`, added the matching later-compatibility row in `specs/19-feature-maturity.md`, and kept the README build summary aligned so the explicit opt-in PGO flag is no longer a hidden implementation detail
