# Active Plan Navigation

This directory contains only active continuation planning. Completed historical phase checklists were removed from the working plan to avoid treating already-implemented tasks as open work.

## Files

- [`00-current-state.md`](./00-current-state.md) — checked-in implementation baseline used by the active roadmap.
- [`01-roadmap.md`](./01-roadmap.md) — phase order, dependencies, and promotion gates.
- [`02-spec-gap-map.md`](./02-spec-gap-map.md) — remaining implementation goals mapped to owning specs.
- [`03-evidence-and-release-gates.md`](./03-evidence-and-release-gates.md) — evidence required before support claims widen.
- [`04-risk-register.md`](./04-risk-register.md) — active risks for future work.

## Active phases

- [`phase-6/`](./phase-6/README.md) — semantic conformance and frontend depth.
- [`phase-7/`](./phase-7/README.md) — runtime, host, and platform expansion.
- [`phase-8/`](./phase-8/README.md) — ecosystem breadth and package compatibility.
- [`phase-9/`](./phase-9/README.md) — optimization, PGO, and performance evidence.
- [`phase-10/`](./phase-10/README.md) — verification and contract hardening.

## Rules

- Do not reopen removed Phase-1 through Phase-5 checklists as active work.
- Do not infer availability from this plan; use [`../specs/19-feature-maturity.md`](../specs/19-feature-maturity.md).
- Do not infer proof-backed scope from this plan; use [`../proofs/BOUNDARY.md`](../proofs/BOUNDARY.md).
- Any public CLI/schema/diagnostic behavior change must update the owning specs before the work is considered complete.
