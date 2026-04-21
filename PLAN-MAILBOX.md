# PLAN mailbox

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
