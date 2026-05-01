# Active Plan Navigation

This directory contains only active continuation planning. Completed historical phase checklists were removed so implemented work is not treated as open.

## Files

- [`00-current-state.md`](./00-current-state.md) — checked-in implementation baseline used by the active roadmap.
- [`01-roadmap.md`](./01-roadmap.md) — continuation phase order, dependencies, and promotion gates.
- [`02-spec-gap-map.md`](./02-spec-gap-map.md) — remaining implementation goals mapped to owning specs.
- [`03-evidence-and-release-gates.md`](./03-evidence-and-release-gates.md) — evidence required before support claims widen.
- [`04-risk-register.md`](./04-risk-register.md) — active risks for future work.

## Active phases

- [`phase-11/`](./phase-11/README.md) — language semantics and conformance closure.
- [`phase-12/`](./phase-12/README.md) — runtime, host, and capability expansion.
- [`phase-13/`](./phase-13/README.md) — ecosystem compatibility expansion.
- [`phase-14/`](./phase-14/README.md) — optimization and performance promotion.
- [`phase-15/`](./phase-15/README.md) — verification and machine-contract widening.

## Rules

- Do not reopen removed Phase-1 through Phase-10 checklists as active work.
- Do not infer availability from this plan; use [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md).
- Do not infer proof-backed scope from this plan; use [`../proofs/BOUNDARY.md`](../proofs/BOUNDARY.md).
- Any public CLI/schema/diagnostic behavior change must update the owning specs before the work is considered complete.
