# Kali — Active Implementation Plan

`PLAN.md` is the implementation playbook for [`SPEC.md`](./SPEC.md). It tracks future work from the current checked-in repository state only. Historical phase checklists and progress journals are intentionally not part of the active plan.

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

The repository is beyond the original MVP and several later surfaces. The live CLI exposes `doctor`, `init`, `install`, `fmt`, `lint`, `check`, `build`, `run`, `test`, `effects`, `package-effects`, and `package-audit`. Current implementation includes schema-v1 JSON envelopes, effect reporting, registry analysis, browser bundle and harness lanes, Node/Deno/browser API slices, embedding artifacts, coverage reporting, optimization/PGO evidence, package-corpus probes, and proof-backed claims limited to the published proof boundary.

Do not reopen completed Phase 1 through Phase 15 work as active tasks. Use the continuation phases below for remaining spec-owned gaps.

## Active roadmap

| Phase | Focus | Main outcome |
|---|---|---|
| [16](./plan/phase-16/README.md) | Semantic closure and conformance promotion | Remaining parser-only or partial language features are either faithfully implemented with evidence or kept behind explicit gates |
| [17](./plan/phase-17/README.md) | Host/runtime contract expansion | Threading, browser runtime, host APIs, and object-runtime APIs widen only with sandbox/resource/effect contracts |
| [18](./plan/phase-18/README.md) | Ecosystem compatibility by rung | Package support grows by exact package shape, command, API surface, and support rung |
| [19](./plan/phase-19/README.md) | Optimization and performance evidence | Optimization/PGO claims become deterministic, benchmark-backed, and mode-specific |
| [20](./plan/phase-20/README.md) | Verification and machine contracts | Proof boundary, schemas, diagnostics, and CLI JSON contracts widen without claim drift |

## Dependency order

```text
Phase 16 semantic closure
  ├── feeds Phase 18 package compatibility
  ├── feeds Phase 19 optimization correctness
  └── feeds Phase 20 formal models

Phase 17 host/runtime contracts
  ├── gates thread/resource-budget claims
  ├── gates standalone browser runtime support wording
  └── feeds Phase 18 executable/deployable package claims

Phase 18 ecosystem compatibility
  └── depends on the relevant Phase 16 semantics and Phase 17 host surface

Phase 19 optimization/performance
  └── depends on stable semantics and must preserve proof/schema boundaries

Phase 20 verification/contracts
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
