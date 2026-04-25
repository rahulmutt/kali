# Phase 9 — Optimization, PGO, and Performance Evidence

## Goal

Improve generated code and performance claims while keeping outputs deterministic and semantics correct.

## Owning specs

- `specs/07-specialization.md`
- `specs/08-wasm-codegen.md`
- `specs/16-testing.md`
- `specs/18-schemas.md`
- `specs/19-feature-maturity.md`

## Work packets

### 9.1 Optimization inventory

- Document which `fast`, `release`, and `release-advanced` optimizations are currently real.
- Add regression tests for existing passes and artifact determinism.
- Progress: sandboxed `build --lib` repeated-build determinism now has coverage alongside the existing artifact-stability checks.
- Progress: the current-mode inventory now has a checked-in snapshot in [`optimization-inventory.md`](./optimization-inventory.md), and the fast-mode minimality path now has a direct optimizer-level regression anchor so the inventory is backed by both docs and tests.

### 9.2 Specialization depth

- Improve layout/representation fingerprinting.
- Respect `--max-specializations` exactly as an upper bound.
- Ensure fallback paths preserve JavaScript semantics.
- Progress: zero-budget tagged-parameter MIR specialization now has a regression that keeps the original call target in place and prevents speculative `add_pair$spec$...` clones.

### 9.3 PGO input

- Promote deterministic build-only `--profile` support only with strict schema validation.
- Keep PGO as an additive build input, not a fourth build mode.
- Add malformed/unknown-field rejection tests.
- Progress: CLI integration now rejects malformed PGO profile payloads with version and unknown-field validation in both text and JSON build output modes.

### 9.4 Benchmark lane

- Add version-pinned benchmarks and fixtures.
- Require repeatability before any public performance wording changes.
- Keep package benchmark anecdotes separate from compatibility claims.
- Progress: the optimization benchmark smoke now uses a checked-in, hash-validated fixture pair (`math-benchmark-v1.ts` / `math-benchmark-v1.json`) so the compile-time size/speed comparison lane is pinned to deterministic inputs instead of an ad-hoc inline source string.

## Exit gate

- Optimizations preserve all conformance and sandbox tests.
- PGO output is deterministic.
- Performance claims name workload, build mode, baseline, and evidence.
