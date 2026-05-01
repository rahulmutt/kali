# Kali — Active Implementation Plan

`PLAN.md` is the implementation playbook for [`SPEC.md`](./SPEC.md). It tracks only future work from the current repository state; historical phase checklists are intentionally absent from `plan/`.

## Plan contract

After every implementation packet the repository must remain workable:

1. `cargo build --workspace` succeeds.
2. `cargo test --workspace` passes.
3. User-visible behavior remains aligned with [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md).
4. Hard invariants remain true: AOT-only guest-language compilation, pure Rust implementation, no tracing/background GC, sandbox-first honesty, and deterministic machine contracts.

Normative ownership remains unchanged:

- [`SPEC.md`](./SPEC.md) defines cross-spec rules and phase contracts.
- the owning chapter in [`specs/`](./specs) defines subsystem behavior.
- [`specs/19-feature-maturity.md`](./specs/19-feature-maturity.md) defines public availability and current-state notes.
- [`proofs/BOUNDARY.md`](./proofs/BOUNDARY.md) defines the current proof-backed boundary.
- this plan and `plan/` define sequencing only.

## Current baseline

The repository has advanced past the original Phase-1 through Phase-10 implementation records. The current checked-in surface includes the core CLI, public effect reporting, registry analysis, coverage reporting, public embedding artifacts, Node/browser/Deno compatibility slices, browser-harness execution, deterministic schema-v1 JSON contracts, PGO/profile validation, optimization evidence, and proof-backed claims for the published boundary.

Do not use old phase labels as active task lists. Use the active continuation phases below plus the owning specs.

## Active roadmap

The remaining work is organized as continuation phases. Phase numbers intentionally start at 11 to avoid confusing current goals with completed historical phases.

| Phase | Focus | Main outcome |
|---|---|---|
| [11](./plan/phase-11/README.md) | Language semantics and conformance closure | Implement or explicitly gate remaining high-value ECMA/TS semantics with conformance evidence |
| [12](./plan/phase-12/README.md) | Runtime, host, and capability expansion | Complete justified host/runtime APIs, threaded execution semantics, and browser contract hardening |
| [13](./plan/phase-13/README.md) | Ecosystem compatibility expansion | Grow package support by rung/context without broad npm or Node overclaims |
| [14](./plan/phase-14/README.md) | Optimization and performance promotion | Turn optimization/PGO work into benchmark-backed, deterministic performance claims |
| [15](./plan/phase-15/README.md) | Verification and machine-contract widening | Widen Lean and schema coverage while keeping proof and JSON claims exact |

## Dependency order

```text
Phase 11 semantic closure
  ├── feeds Phase 13 package compatibility
  ├── feeds Phase 14 optimization correctness
  └── feeds Phase 15 verification models

Phase 12 runtime/host expansion
  ├── gates late host APIs and positive resource-budget claims
  ├── gates browser-runtime support wording
  └── feeds Phase 13 package executable/deployable claims

Phase 13 ecosystem compatibility
  └── depends on the relevant Phase 11 semantics and Phase 12 host surface

Phase 14 optimization/performance
  └── depends on stable semantics and must preserve proof/schema boundaries

Phase 15 verification/contracts
  └── may run in parallel, but proof-backed wording changes only after `proofs/BOUNDARY.md` widens
```

## Completion packet for any active phase

Every phase packet must land with:

1. implementation changes;
2. tests/evidence appropriate to the claim;
3. docs/spec updates when public behavior changes;
4. maturity-matrix updates when availability changes;
5. proof-boundary updates if verification claims change;
6. `cargo test --workspace` evidence in the handoff.

## Reading rule

- Use this file for sequencing.
- Use `plan/phase-*/README.md` for active goals.
- Use `specs/19-feature-maturity.md` for whether a surface is publicly available.
- Use `proofs/BOUNDARY.md` for what is proof-backed today.
