# Phase 3 — Specialization, Optimization & Ecosystem Breadth

**Implements:** the first major compatibility and performance broadening pass without weakening the core invariants

## Objective

Broaden Kali from a correct Phase-1/2 core into a faster and more compatible system by:
- deepening specialization and optimization,
- opening the documented Node path,
- widening package/runtime/browser breadth where the specs allow it,
- expanding host capabilities such as mutable env, subprocesses, and sockets.

## Why this order

Optimization and specialization come first because later Node, package, and host-expansion work should build on the compiler shape that Kali expects to keep. The phase is organized to avoid repeatedly widening compatibility over an unstable optimization core.

## Dependency shape

- First: [3.1 — Optimization & Specialization](./01-optimization-and-specialization.md)
- Then in parallel where safe: [3.2 — Node Compatibility](./02-node-compatibility.md) and [3.4 — Host Capability Expansion](./04-host-capability-expansion.md)
- Finally: [3.3 — Ecosystem Breadth](./03-ecosystem-breadth.md), which consumes the stronger optimization and host/package foundations

## Stages

| Stage | Focus | Primary spec owners |
|---|---|---|
| [3.1 — Optimization & Specialization](./01-optimization-and-specialization.md) | monomorphization, release-mode depth, measurable gains | `specs/07`, `specs/05`, `specs/08` |
| [3.2 — Node Compatibility](./02-node-compatibility.md) | `--api node` path and supported Node subset | `specs/11`, `specs/12`, `specs/19` |
| [3.3 — Ecosystem Breadth](./03-ecosystem-breadth.md) | broader package corpus, dynamic import breadth, bundle/package support | `specs/11`, `specs/14`, `specs/16` |
| [3.4 — Host Capability Expansion](./04-host-capability-expansion.md) | mutable env, subprocess, sockets/listeners, broader host APIs | `specs/09`, `specs/10`, `specs/11` |

## Coordination points

This phase must keep four things synchronized:
- release-mode vocabulary from [`specs/07-specialization.md`](../../specs/07-specialization.md),
- Node and Deno-oriented API claims from [`specs/11-standard-apis.md`](../../specs/11-standard-apis.md),
- package-support ladders from [`specs/14-packages.md`](../../specs/14-packages.md),
- evidence-backed promotions in [`specs/19-feature-maturity.md`](../../specs/19-feature-maturity.md).

## Exit gate

Phase 3 is complete only when:
- `--release` / `--release-advanced` show evidence-backed gains over `--fast`,
- the opened Node rows are supported by end-to-end evidence,
- host-capability expansions have matching sandbox/resource-limit tests,
- package-corpus breadth claims name the exact rung being opened,
- no widened compatibility surface overclaims beyond the maturity matrix.
