# Phase 5 — Later Compatibility & Platform Expansion

**Implements:** the explicitly deferred spec surfaces that should not be pulled forward into earlier public promises

## Objective

Track and stage later-compatibility work without collapsing it into one vague “future features” bucket. This phase covers:
- the threaded runtime profile,
- standalone browser runtime/test support,
- programmable policy predicates and any algebraic-effect surface,
- late host/object-model compatibility corners,
- profile-guided optimization and broader language bindings.

## Planning rule

Phase 5 is a planning bucket, not an automatic product promise. A stage living in this phase does **not** mean the feature is publicly available; actual availability still comes only from [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md).

## Entry criteria

Phase 5 is for explicitly deferred breadth, so earlier phases should already be stable and evidence-backed before this phase begins opening new runtime/platform commitments.

## Dependency shape

- Foundation first: [5.1 — Threaded Runtime Profile](./01-threaded-runtime-profile.md)
- Runtime/host breadth next: [5.2](./02-standalone-browser-runtime-and-host-expansion.md) and [5.4](./04-late-host-and-object-compatibility.md)
- Policy/effects later: [5.3 — Programmable Policy & Algebraic Effects](./03-programmable-policy-and-algebraic-effects.md)
- Toolchain/bindings lane: [5.5 — PGO & Language Bindings](./05-pgo-and-language-bindings.md)

## Stages

| Stage | Focus | Primary spec owners |
|---|---|---|
| [5.1 — Threaded Runtime Profile](./01-threaded-runtime-profile.md) | `--wasm-threads`, `SharedArrayBuffer`, `Atomics`, thread budgets | `specs/09`, `specs/10`, `specs/11` |
| [5.2 — Standalone Browser Runtime & Host Expansion](./02-standalone-browser-runtime-and-host-expansion.md) | later `run/test --api browser` contract and backend work | `specs/10`, `specs/11`, `specs/12` |
| [5.3 — Programmable Policy & Algebraic Effects](./03-programmable-policy-and-algebraic-effects.md) | host-registered predicates and later effect mechanisms | `specs/09`, `specs/13` |
| [5.4 — Late Host & Object Compatibility](./04-late-host-and-object-compatibility.md) | weak refs, proxies, legacy/web-compat corners, late host APIs | `specs/10`, `specs/11` |
| [5.5 — PGO & Language Bindings](./05-pgo-and-language-bindings.md) | additive PGO workflow and non-Rust bindings over the public ABI | `specs/07`, `specs/13`, `specs/16` |

## Phase-level workable-state ladder

| After stage | The repository should be able to demonstrate |
|---|---|
| 5.1 | opt-in threaded runtime execution with documented budget enforcement |
| 5.2 | standalone browser runtime/test support without overclaiming browser mediation |
| 5.3 | programmable policy/effect extensions that preserve earlier declarative guarantees |
| 5.4 | late host/object-model breadth behind explicit gates and evidence |
| 5.5 | additive PGO and broader language bindings over stable public surfaces |

## Coordination points

- Thread-aware runtime work must not invent a second concurrency model.
- Browser runtime work must preserve the browser ambient-typing vs mediated-capability split.
- Programmable policy work must not weaken the declarative sandbox contract for earlier phases.
- PGO must remain additive to the stable build-mode vocabulary.
- Binding expansion must stay on the stable ABI/WIT surface rather than ad hoc wrappers.

## Exit gate

Phase 5 is complete only when each later-compatibility feature:
- has a stage-level implementation and evidence trail,
- updates its owning CLI/schema/error/maturity docs where required,
- preserves the hard invariants and deterministic machine contracts,
- is promoted in the maturity matrix one surface at a time rather than through a blanket claim.

Current repository note:
- all stage documents in this phase are now closed, so this README is historical implementation guidance rather than an open work queue
- any future widening of later-compatibility surfaces should be reflected in the owning spec chapters and maturity matrix, not by reopening these stage checklists
