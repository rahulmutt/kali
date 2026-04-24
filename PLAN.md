# Kali — Active Implementation Plan

`PLAN.md` is the implementation playbook for [`SPEC.md`](./SPEC.md). It now tracks only future work from the current repository state. Earlier phase plans were removed from `plan/` because their stage checklists are completed historical records in this snapshot.

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

The repository has moved beyond the original Phase-1 through Phase-4 implementation graph. The current checked-in state includes:

- core CLI commands: `init`, `install`, `fmt`, `lint`, `check`, `build`, `run`, and `test`;
- stable JSON envelopes and schema-v1 result payloads;
- sandbox policy validation/enforcement plus public effect reporting;
- `kali effects`, `kali package-effects`, and `kali package-audit`;
- `kali test --coverage`;
- public embedding flows: `build --lib`, `build --capi`, and `build --component`;
- documented Node-capable `check` / `build` / `run` / `test` subsets and Node-capable effect-analysis lanes;
- the documented `--compat eval` path;
- proof-backed claims for the published boundary in `proofs/BOUNDARY.md`.

Do not use old phase labels as active task lists. Use the active roadmap below plus the owning specs.

## Active roadmap

The remaining work is organized as continuation phases. Phase numbers intentionally start at 6 to avoid confusing these active goals with the completed historical Phase-1 through Phase-5 stage documents that were removed.

| Phase | Focus | Main outcome |
|---|---|---|
| [6](./plan/phase-6/README.md) | Semantic conformance and frontend depth | Wider ECMA/TS/JS semantics, better module/dynamic-import coverage, and a conformance evidence dashboard |
| [7](./plan/phase-7/README.md) | Runtime, host, and platform expansion | Threaded runtime profile, standalone browser run/test contract, and late host/object APIs where justified |
| [8](./plan/phase-8/README.md) | Ecosystem breadth and package compatibility | Broader npm/JSR package rungs, Node/browser package-corpus expansion, and compatibility triage automation |
| [9](./plan/phase-9/README.md) | Optimization, PGO, and performance evidence | Deterministic PGO input, stronger release-advanced pipeline, and benchmark-backed performance claims |
| [10](./plan/phase-10/README.md) | Verification and contract hardening | Wider Lean proof boundary, implementation/spec traceability, and proof-trigger expansion when claims widen |

## Dependency order

```text
Phase 6 semantic depth
  ├── feeds Phase 8 package compatibility
  ├── feeds Phase 9 optimization correctness
  └── feeds Phase 10 verification models

Phase 7 runtime/host expansion
  ├── is required before browser runtime/package executable claims
  ├── is required before positive thread-budget support
  └── gates late host/object APIs

Phase 8 ecosystem breadth
  └── depends on Phase 6 semantics and the relevant Phase 7 host surface

Phase 9 optimization/PGO
  └── depends on stable semantics and must preserve Phase 10 proof/contract boundaries

Phase 10 verification
  └── may run in parallel, but proof-backed wording changes only after `proofs/BOUNDARY.md` widens
```

## Active phase summaries

### Phase 6 — Semantic conformance and frontend depth

Goal: make Kali less demo-shaped and more conformance-shaped without widening public claims faster than evidence.

Primary goals:
- expand parser/checker/runtime coverage for the current published ECMA-262 target;
- deepen TypeScript/JavaScript checker behavior under the bounded inference contract;
- implement or explicitly gate remaining module semantics such as literal-string `import()`;
- add conformance dashboards and minimized regression fixtures.

### Phase 7 — Runtime, host, and platform expansion

Goal: add runtime/profile breadth only where Kali can mediate and test it honestly.

Primary goals:
- implement the opt-in `wasm-threads` runtime profile and positive thread budgets;
- define and implement a real standalone browser `run` / `test` contract, if still justified;
- add late process/working-directory/object-model APIs only after their sandbox/effect contracts are explicit;
- preserve the browser-targeted vs standalone-browser split.

### Phase 8 — Ecosystem breadth and package compatibility

Goal: turn package support from selected smoke tests into measured support rungs.

Primary goals:
- expand package-corpus coverage by host/API context and support rung;
- improve Node/browser package resolution and compatibility where specs already permit it;
- keep native/binary/bootstrap-heavy packages rejected by default unless the spec changes;
- automate package triage output without conflating installability, checkability, buildability, executability, and deployability.

### Phase 9 — Optimization, PGO, and performance evidence

Goal: make performance claims evidence-backed and deterministic.

Primary goals:
- finish deterministic build-only PGO input support (`--profile`) if not fully promoted;
- deepen release/release-advanced MIR/LIR/WASM optimization passes;
- add version-pinned benchmark lanes;
- ensure optimization does not bypass sandbox, diagnostics, or proof boundaries.

### Phase 10 — Verification and contract hardening

Goal: widen formal and machine-contract confidence while keeping proof claims precise.

Primary goals:
- widen the Lean model toward ownership/effects/lowering subsets named in `proofs/BOUNDARY.md`;
- add proof-trigger coverage only when boundary claims name affected implementation/spec paths;
- strengthen schema/docs conformance checks;
- keep public wording limited to the published boundary.

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
