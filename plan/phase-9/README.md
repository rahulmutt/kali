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

### 9.2 Specialization depth

- Improve layout/representation fingerprinting.
- Respect `--max-specializations` exactly as an upper bound.
- Ensure fallback paths preserve JavaScript semantics.

### 9.3 PGO input

- Promote deterministic build-only `--profile` support only with strict schema validation.
- Keep PGO as an additive build input, not a fourth build mode.
- Add malformed/unknown-field rejection tests.
- Progress: CLI integration now rejects malformed PGO profile payloads with version and unknown-field validation in both text and JSON build output modes.

### 9.4 Benchmark lane

- Add version-pinned benchmarks and fixtures.
- Require repeatability before any public performance wording changes.
- Keep package benchmark anecdotes separate from compatibility claims.

## Exit gate

- Optimizations preserve all conformance and sandbox tests.
- PGO output is deterministic.
- Performance claims name workload, build mode, baseline, and evidence.
