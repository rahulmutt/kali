# Active Roadmap

## Phase order

| Phase | Name | Can begin when | Promotion gate |
|---|---|---|---|
| 21 | Semantic completeness and conformance | now | supported vs gated semantics are backed by conformance, minimized regressions, and canonical diagnostics; current continuation work now also includes the Node `process.kill(0)` liveness-probe slice, including wrapped `process` and wrapped `process.kill` call-target forms plus the mixed frozen `Object.freeze(globalThis["process"].kill)(0)` alias, its receiver-freeze `+0` siblings, and the newer parenthesized receiver-freeze spellings around `globalThis.process["kill"]` / `globalThis["process"]["kill"]`, alongside the existing host-control gating work, and the browser late-compat JS fixture now reuses the shared zero-probe helper inventory too |
| 22 | Host/runtime capability contracts | after relevant Phase-21 semantics are stable | host/runtime capabilities have sandbox, effect, resource-budget, and JSON evidence |
| 23 | Ecosystem compatibility by rung | after related Phase-21/22 capabilities exist | package claims name package shape, support rung, command, API surface, and evidence |
| 24 | Optimization and performance evidence | after optimization-sensitive semantics are stable | performance claims are benchmark-backed, deterministic, and mode-specific |
| 25 | Verification and machine contracts | can run in parallel | `proofs/BOUNDARY.md`, proof CI, schemas, diagnostics, and CLI contracts match any widened claim |

## Parallelism

Safe parallel work:

- Phase 21 parser/checker/runtime fixtures and Phase 25 proof modeling may proceed together if proof claims do not cite unimplemented Rust behavior.
- Phase 22 host-contract design and Phase 23 package-corpus triage may proceed together, but package support cannot be promoted until the needed host surface exists.
- Phase 24 benchmark harness work may proceed before all optimizations are implemented, as long as no public performance claim is made early.

Unsafe parallel work:

- Do not implement package compatibility by silently widening host APIs before Phase 22 contracts and sandbox effects exist.
- Do not add optimization passes that change observable semantics without Phase 21 conformance coverage.
- Do not widen proof-backed wording from implementation intuition; mechanize the boundary and update `proofs/BOUNDARY.md` first.

## Default packet size

Prefer small, claim-aligned packets:

1. one feature or compatibility slice;
2. one owning spec update if behavior changes;
3. one evidence lane update;
4. one maturity/proof-boundary update if public availability or proof claims change.
