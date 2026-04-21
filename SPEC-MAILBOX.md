# SPEC mailbox

2026-04-21 follow-up note

While hardening the semver/browser bundle path, the workspace test suite revealed that `specs/16-testing.md` is missing the canonical proof-backed summary string that is already present in `README.md` and `proofs/BOUNDARY.md`.

Proposed fix:
- add the canonical short summary to the proof-claim discipline section in `specs/16-testing.md`
- keep the wording identical to the repository summary already used elsewhere:
  **"Kali is proof-backed for the published boundary; the current boundary is intentionally narrower than the later Stage 4.2 target."**
