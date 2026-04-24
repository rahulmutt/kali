# Active Roadmap

## Phase order

| Phase | Name | Can begin when | Promotion gate |
|---|---|---|---|
| 6 | Semantic conformance and frontend depth | now | conformance dashboard and regression lanes identify supported vs gated semantics clearly |
| 7 | Runtime, host, and platform expansion | after relevant Phase-6 semantics are stable | new host/runtime capabilities have sandbox/effect/resource tests |
| 8 | Ecosystem breadth and package compatibility | after related Phase-6/7 capabilities exist | package claims name support rungs and pass corpus evidence |
| 9 | Optimization, PGO, and performance evidence | after optimization-sensitive semantics are stable | performance claims are benchmark-backed and deterministic |
| 10 | Verification and contract hardening | can run in parallel | `proofs/BOUNDARY.md` and proof CI match any widened claim |

## Parallelism

Safe parallel work:

- Phase 6 parser/checker fixtures and Phase 10 proof modeling can proceed together if proof claims do not cite unimplemented Rust behavior.
- Phase 7 host-surface design and Phase 8 package-corpus triage can proceed together, but package support cannot be promoted until the needed host surface exists.
- Phase 9 benchmark harness work can proceed before all optimizations are implemented, as long as no performance claim is made early.

Unsafe parallel work:

- Do not implement package compatibility by silently widening host APIs before Phase 7 contracts and sandbox effects exist.
- Do not add optimization passes that change observable semantics without Phase 6 conformance coverage.
- Do not widen proof-backed wording from implementation intuition; update the mechanized boundary first.

## Default packet size

Prefer small, claim-aligned packets:

1. one feature or compatibility slice;
2. one owning spec update if behavior changes;
3. one evidence lane update;
4. one maturity/proof-boundary update if public availability or proof claims change.
