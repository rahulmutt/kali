# Phase 4 — Dynamic Compatibility & Deep Verification

**Implements:** the hardest spec-promised dynamic features and the transition from proof-ready/foundational verification work into a proof-backed published boundary

## Objective

Complete the most difficult late-core compatibility work while keeping the compiler honest about what is actually supported:
- gated support for `eval`, `Function()`, and harder dynamic-loading paths
- public `package-audit` as a separate registry-analysis surface
- a non-empty published proof boundary with proof-backed release claims

## Entry criteria

Phase 4 should start only once the runtime, package, optimization, and proof-foundation work from earlier phases has stabilized enough that late dynamic features and proof-backed claims do not churn underneath active foundational redesign.

## Why this order

Dynamic compatibility depends on the runtime, packaging, host, and optimization machinery from the earlier phases. Verification depth depends on the Lean foundation from Phase 2 and on the canonical semantics established by the core pipeline. This phase therefore uses two tightly scoped streams instead of a broad compatibility grab-bag.

## Dependency shape

- [4.1 — Dynamic Compatibility](./01-dynamic-compatibility.md) depends on Phases 1–3 runtime/package/host groundwork
- [4.2 — Formal Verification Depth](./02-formal-verification-depth.md) depends on [2.4 — Lean Model Foundation](../phase-2/04-lean-model-foundation.md) and the current published proof discipline

## Stages

| Stage | Focus | Primary spec owners |
|---|---|---|
| [4.1 — Dynamic Compatibility](./01-dynamic-compatibility.md) | `--compat eval`, `Function()`, non-literal dynamic loading, `package-audit` | `specs/10`, `specs/12`, `specs/14`, `specs/18` |
| [4.2 — Formal Verification Depth](./02-formal-verification-depth.md) | non-empty proof boundary and proof-backed claims | `specs/16`, `specs/17`, `specs/19` |

## Phase-level workable-state ladder

| After stage | The repository should be able to demonstrate |
|---|---|
| 4.1 | gated dynamic compatibility features and public `package-audit` support within their documented limits |
| 4.2 | a non-empty published proof boundary with proof-backed claims constrained to that boundary |

## Coordination points

- Dynamic-feature gating must preserve the AOT-only invariant and never silently widen execution
- `package-audit` must stay distinct from the public effect-report surface
- Proof-backed claims must be limited to the exact content of `proofs/BOUNDARY.md`

## Exit gate

Phase 4 is complete only when:
- dynamic compatibility features are available only through their documented gates
- `package-audit` has its own stable schema/CLI contract
- `proofs/BOUNDARY.md` publishes a non-empty boundary
- README/spec/plan summaries all describe proof-backed status consistently
