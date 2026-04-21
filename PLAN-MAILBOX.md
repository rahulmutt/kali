# PLAN mailbox

2026-04-21 resolution note
- updated the Stage 5.1 progress note so it explicitly mentions the `runtimeProfiles` axis and the current empty-set normalization in emitted build metadata

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

2026-04-21 follow-up note

While wiring the threaded-profile axis into artifact metadata, the Stage 5.1 progress note should mention that emitted build metadata now carries `runtimeProfiles` explicitly even though the current phase still normalizes to the empty set.

Proposed fix:
- update `plan/phase-5/01-threaded-runtime-profile.md` progress notes to mention the explicit `runtimeProfiles` metadata axis and the current empty-set normalization

2026-04-21 resolution note
- updated the Stage 5.1 progress note so it explicitly names the `runtimeProfiles` metadata axis and the current empty-set normalization in emitted build metadata
