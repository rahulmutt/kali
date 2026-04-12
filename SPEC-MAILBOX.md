# SPEC Mailbox

## 2026-04-12 — Lean proof boundary wording

The repository now has a checked-in provisional Lean proof tree under `proofs/` and `proofs/BOUNDARY.md` is no longer a pure placeholder manifest. The current wording in `specs/17-verification.md` and the proof-related rows in `specs/19-feature-maturity.md` still talk about the shared **placeholder proof-boundary manifest** as the only pre-proof state.

Suggested follow-up:
- revise the verification chapter and maturity row wording to distinguish a **provisional non-empty proof boundary** from the original empty placeholder state
- keep the canonical short summary in `proofs/BOUNDARY.md` unchanged until the proofs become genuinely proof-backed
- ensure any future proof-backed wording still points at the published boundary rather than the staging history
