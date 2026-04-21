# Stage 4.2 — Formal Verification Depth

**Phase:** 4 — Advanced Compatibility & Deep Verification
**Spec refs:** [`specs/17-verification.md`](../../specs/17-verification.md), [`specs/16-testing.md`](../../specs/16-testing.md), [`proofs/BOUNDARY.md`](../../proofs/BOUNDARY.md), [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md)
**Depends on:** [2.4 — Lean Model Foundation](../phase-2/04-lean-model-foundation.md) (Lean workspace, core type-calculus model, type-soundness proof, and real CI proof jobs must exist before this stage deepens them); proof-*backed* claims require a non-empty, non-provisional published boundary in `proofs/BOUNDARY.md`, which this stage delivers

## Goal

Advance from the **provisional Lean model** established in Stage 2.4 to a full **proof-backed**
state: complete the memory-safety and lowering-correctness proofs, replace all `sorry`
placeholders in the type-soundness theorems, publish a non-provisional, non-empty proof boundary
in `proofs/BOUNDARY.md`, and enable **proof-backed** release/support claims.

## Workable Milestone

- Every `sorry` placeholder from Stage 2.4's type-soundness proofs is replaced by a complete
  mechanised proof.
- Memory-safety (no-dangling-reference) and HIR → LIR lowering-correctness proofs are
  complete for the bounded core calculus.
- `proofs/BOUNDARY.md` is updated from provisional to non-provisional, naming the concrete
  modelled subsystems with a full theorem inventory.
- CI proof jobs continue to run and block on failure; the boundary is now non-empty.
- Release notes and documentation may cite formal verification for the published boundary.

Current progress note:
- The published boundary is now proof-backed, with the current authoritative theorem inventory,
  covered-path list, and anti-drift expectations owned by [`proofs/BOUNDARY.md`](../../proofs/BOUNDARY.md).
- The current published slice includes widened RC snapshot safety theorems, live-reference
  ownership/allocation and filtering results, release/decrement/collection helper invariants,
  linear-memory preservation corollaries, and the current HIR lowering-correctness subset.
- The canonical short summary remains: **Kali is proof-backed for the published boundary; the
  current boundary is intentionally narrower than the later Stage 4.2 target.**

## Status

Stage 4.2 is complete.

Current verification scope and any further widening work are owned by the published proof boundary
and the normative verification docs, not by reopening this stage checklist. Read:
- [`proofs/BOUNDARY.md`](../../proofs/BOUNDARY.md) for the current proof-backed boundary,
- [`specs/17-verification.md`](../../specs/17-verification.md) for verification-program rules, and
- [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md) for claim and maturity boundaries.

This file remains the historical implementation playbook for moving Kali from the Stage-2.4
provisional Lean state to a proof-backed published boundary.
