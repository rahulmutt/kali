# Active Plan Navigation

This directory contains only active continuation planning. Completed implementation notes and historical phase checklists are intentionally omitted so implemented work is not mistaken for open scope.

## Files

- [`00-current-state.md`](./00-current-state.md) — concise checked-in baseline for planning.
- [`01-roadmap.md`](./01-roadmap.md) — phase order, dependencies, and packet sizing.
- [`02-spec-gap-map.md`](./02-spec-gap-map.md) — remaining implementation goals mapped to owning specs.
- [`03-evidence-and-release-gates.md`](./03-evidence-and-release-gates.md) — evidence required before support claims widen.
- [`04-risk-register.md`](./04-risk-register.md) — active risks for future work.

## Active phases

- [`phase-21/`](./phase-21/README.md) — semantic completeness and conformance.
- [`phase-22/`](./phase-22/README.md) — host/runtime capability contracts.
- [`phase-23/`](./phase-23/README.md) — ecosystem compatibility by rung.
- [`phase-24/`](./phase-24/README.md) — optimization and performance evidence.
- [`phase-25/`](./phase-25/README.md) — verification and machine contracts.

## Rules

- Do not reopen removed historical phase checklists as active work.
- Do not infer availability from this plan; use [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md).
- Do not infer proof-backed scope from this plan; use [`../proofs/BOUNDARY.md`](../proofs/BOUNDARY.md).
- Any public CLI/schema/diagnostic behavior change must update the owning specs before the work is considered complete.
