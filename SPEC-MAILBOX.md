# SPEC mailbox

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
