# Phase 2 — Ownership, Effects & Public Embedding

**Implements:** the first post-MVP depth pass across IR, memory strategy, effect reporting, embedding, verification foundations, and coverage reporting

## Objective

Turn the Phase-1 MVP into a more principled and externally consumable platform by:
- making MIR and ownership analysis the canonical mid-pipeline representation,
- opening the stable public effect-report surface,
- promoting the early base-library artifact into a stable public embedding contract,
- establishing the Lean verification foundation beyond proof-ready hygiene,
- stabilizing coverage reporting for the documented `kali test --coverage` surface.

## Why this order

Stage 2.1 is the architectural hinge of the phase. Once MIR and ownership analysis become canonical, the compiler has the explicit memory and layout information needed for stronger effect modeling, more stable embedding metadata, and cleaner verification modeling. The remaining streams can then build on that more explicit representation instead of the earlier conservative pipeline.

## Dependency shape

- Main prerequisite: [2.1 — MIR & Ownership Analysis](./01-mir-and-ownership.md)
- Builds directly on 2.1: [2.2](./02-public-effect-reporting.md), [2.3](./03-public-embedding-surface.md), and the machine-readable/reporting portions of [2.5](./05-test-coverage-and-reporting.md)
- Verification stream: [2.4 — Lean Model Foundation](./04-lean-model-foundation.md) depends on the proof-ready baseline from Phase 1 and should coordinate with 2.1 on canonical semantics

## Stages

| Stage | Focus | Primary spec owners |
|---|---|---|
| [2.1 — MIR & Ownership Analysis](./01-mir-and-ownership.md) | canonical MIR, escape analysis, explicit ownership classes | `specs/05`, `specs/06` |
| [2.2 — Public Effect Reporting](./02-public-effect-reporting.md) | `effects` / `package-effects`, policy comparison, stable effect JSON | `specs/09`, `specs/12`, `specs/18` |
| [2.3 — Public Embedding Surface](./03-public-embedding-surface.md) | stable Rust embedding API plus public `--lib` / `--capi` / `--component` flows | `specs/13`, `specs/18` |
| [2.4 — Lean Model Foundation](./04-lean-model-foundation.md) | proof workspace, CI, and formal semantic core | `specs/17` |
| [2.5 — Test Coverage & Reporting](./05-test-coverage-and-reporting.md) | stable function-coverage contract and deterministic reporting | `specs/12`, `specs/16`, `specs/18` |

## Safe parallelism

After 2.1 lands, `2.2`, `2.3`, `2.4`, and most of `2.5` may proceed in parallel if they keep these shared surfaces aligned:
- effect names and output schemas,
- embedding artifact metadata,
- verification claims and `proofs/BOUNDARY.md`,
- coverage JSON envelopes and CLI flags.

## Exit gate

Phase 2 is complete only when:
- MIR is the canonical mid-level representation,
- ownership/escape analysis drives the documented memory strategy,
- public effect reporting is stable and schema-backed,
- public embedding outputs are versioned and deterministic,
- Lean proof jobs run in CI with a maintained boundary discipline,
- coverage reporting is deterministic for the contexts it claims to support.
