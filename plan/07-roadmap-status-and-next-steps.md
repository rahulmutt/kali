# Roadmap Status and Next Steps

This guide complements [`../PLAN.md`](../PLAN.md) by answering a different question:

> **Given the current repository state, which parts of the plan are historical milestones, which remain active follow-up lanes, and where should new implementation work start?**

Use this file when the phase/stage roadmap is clear but the repository has already moved past the original bootstrap sequence.

## Core rule

- [`../PLAN.md`](../PLAN.md) and the phase/stage files still own the implementation order.
- [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md) still owns public availability.
- [`../proofs/BOUNDARY.md`](../proofs/BOUNDARY.md) still owns the current proof-backed boundary.
- This file only summarizes **planning status** and the current **follow-up lanes** so contributors do not have to infer them from scattered historical notes.

## Current status snapshot

| Area | Planning status | Read first |
|---|---|---|
| Phase 1 core compiler/toolchain roadmap | Historical sequence, but still the canonical explanation of how the MVP was built | [`phase-1/README.md`](./phase-1/README.md) |
| Phase 2 ownership/effects/embedding roadmap | Historical sequence with useful architectural rationale | [`phase-2/README.md`](./phase-2/README.md) |
| Phase 3 optimization and compatibility roadmap | Historical sequence plus active hardening/breadth follow-up notes | [`phase-3/README.md`](./phase-3/README.md) |
| Phase 4 dynamic compatibility and verification roadmap | Historical sequence plus the canonical proof-depth follow-up pointer | [`phase-4/README.md`](./phase-4/README.md) |
| Phase 5 later-compatibility expansion | Planning bucket for explicitly deferred breadth; do not read it as a shipped promise | [`phase-5/README.md`](./phase-5/README.md) |
| Current repository growth order | Historical for the main workspace rollout | [`06-current-workspace-rollout.md`](./06-current-workspace-rollout.md) |

## How to read historical stage files

Many stage documents are now closed and contain repository-state notes such as “historical implementation playbook” or “stage complete.”

Read those files in this order:

1. use the stage file for the original dependency order, milestone shape, and definition of done,
2. use the owning spec chapter for the current normative contract,
3. use [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md) for current availability,
4. use this file to see whether the stage still has an explicit follow-up lane.

Do **not** reopen a closed stage just because the repo keeps evolving. If new work changes the public contract, update the owning spec and maturity docs first, then either:
- extend the documented follow-up lane, or
- add a new plan document when the work is large enough to deserve its own stage-level sequencing.

## Canonical active follow-up lanes

These are the plan-level follow-up lanes that remain worth consulting when implementing from the current repository state.

### 1. Specialization-depth follow-up

Primary source:
- [`phase-3/01-optimization-and-specialization.md`](./phase-3/01-optimization-and-specialization.md) → `Remaining Work`

Use this lane for:
- deeper optimization passes,
- more evidence-backed release-mode improvements,
- work that refines specialization limits or performance proof points without changing the core release-mode vocabulary.

### 2. Ecosystem/package/browser breadth follow-up

Primary source:
- [`phase-3/03-ecosystem-breadth.md`](./phase-3/03-ecosystem-breadth.md) → `Remaining Work`

Use this lane for:
- broader package-corpus support,
- browser-targeted breadth hardening,
- compatibility improvements that must still name their exact support rung.

### 3. Verification-depth follow-up

Primary sources:
- [`phase-4/02-formal-verification-depth.md`](./phase-4/02-formal-verification-depth.md) → `Remaining Work`
- [`../proofs/BOUNDARY.md`](../proofs/BOUNDARY.md)

Use this lane for:
- widening the proof-backed boundary,
- strengthening proof/implementation links,
- updating proof summaries after theorem inventory changes.

Guardrail:
- the theorem inventory itself must continue to live in [`../proofs/BOUNDARY.md`](../proofs/BOUNDARY.md), not in this file.

## Recommended starting points by task type

| If you are working on... | Start here | Then read |
|---|---|---|
| a new compiler frontend or IR change | [`../PLAN.md`](../PLAN.md) + relevant phase README | owning spec chapter + stage file |
| runtime, sandbox, or host-surface work | [`02-workstreams-and-handoffs.md`](./02-workstreams-and-handoffs.md) | relevant phase README + owning specs |
| package or ecosystem compatibility | [`phase-3/03-ecosystem-breadth.md`](./phase-3/03-ecosystem-breadth.md) | [`../specs/14-packages.md`](../specs/14-packages.md), [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md) |
| proof or verification work | [`phase-4/02-formal-verification-depth.md`](./phase-4/02-formal-verification-depth.md) | [`../proofs/BOUNDARY.md`](../proofs/BOUNDARY.md), [`../specs/17-verification.md`](../specs/17-verification.md) |
| current workspace layout/growth questions | [`01-repository-layout.md`](./01-repository-layout.md) | [`06-current-workspace-rollout.md`](./06-current-workspace-rollout.md) |
| whether something is actually shipped | [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md) | owning spec chapter |

## When to create a new planning document

Create a new plan doc instead of only editing historical notes when all of the following are true:

1. the work is larger than a small hardening pass,
2. it has its own dependency order or completion gate,
3. it touches more than one subsystem or evidence lane,
4. future contributors would benefit from a dedicated “what next” checklist.

Otherwise, prefer updating the relevant phase README, stage follow-up section, or this status guide.

## Maintenance rule

Keep this file compact and status-oriented.

- Do not duplicate spec contracts here.
- Do not duplicate the proof-boundary inventory here.
- Do not let this file drift into a second top-level roadmap.
- When a follow-up lane closes or a new one appears, update this guide and the owning phase/stage doc together.
