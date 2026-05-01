# Active Roadmap

## Phase order

| Phase | Name | Can begin when | Promotion gate |
|---|---|---|---|
| 11 | Language semantics and conformance closure | now | supported vs gated semantics are backed by conformance and minimized regressions |
| 12 | Runtime, host, and capability expansion | after relevant Phase-11 semantics are stable | host/runtime capabilities have sandbox, effect, resource-budget, and JSON evidence |
| 13 | Ecosystem compatibility expansion | after related Phase-11/12 capabilities exist | package claims name support rungs and pass corpus evidence |
| 14 | Optimization and performance promotion | after optimization-sensitive semantics are stable | performance claims are benchmark-backed, deterministic, and mode-specific |
| 15 | Verification and machine-contract widening | can run in parallel | `proofs/BOUNDARY.md`, proof CI, and schemas match any widened claim |

## Parallelism

Safe parallel work:

- Phase 11 parser/checker/runtime fixtures and Phase 15 proof modeling can proceed together if proof claims do not cite unimplemented Rust behavior.
- Phase 12 host-surface design and Phase 13 package-corpus triage can proceed together, but package support cannot be promoted until the needed host surface exists.
- Phase 14 benchmark harness work can proceed before all optimizations are implemented, as long as no performance claim is made early.

Unsafe parallel work:

- Do not implement package compatibility by silently widening host APIs before Phase 12 contracts and sandbox effects exist.
- Do not add optimization passes that change observable semantics without Phase 11 conformance coverage.
- Do not widen proof-backed wording from implementation intuition; update the mechanized boundary first.

## Default packet size

Prefer small, claim-aligned packets:

1. one feature or compatibility slice;
2. one owning spec update if behavior changes;
3. one evidence lane update;
4. one maturity/proof-boundary update if public availability or proof claims change.
