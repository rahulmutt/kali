# Phase 14 — Optimization and Performance Promotion

## Goal

Turn optimization and PGO work into deterministic, evidence-backed performance claims without changing observable semantics.

## Owning specs

- `specs/07-specialization.md`
- `specs/08-wasm-codegen.md`
- `specs/16-testing.md`
- `specs/18-schemas.md`
- `specs/19-feature-maturity.md`

## Work packets

### 14.1 Optimization inventory upkeep

- Keep a concise current-evidence inventory of real `fast`, `release`, and `release-advanced` behavior.
- Update the inventory only when tests prove a mode's behavior.
- Preserve deterministic artifacts and schema-v1 output contracts.

### 14.2 Specialization depth

- Improve layout, representation, and call-shape specialization in claim-aligned slices.
- Treat `--max-specializations` as an exact upper bound.
- Preserve fallback paths and JavaScript-visible semantics.

### 14.3 PGO input hardening

- Keep `--profile` as deterministic build-only additive input.
- Reject malformed, unknown-field, version-mismatched, and nondeterministic profile data.
- Do not create a fourth build-mode vocabulary.

### 14.4 Benchmark promotion

- Use version-pinned workload fixtures with hash validation.
- Promote performance wording only with workload, build mode, baseline, repeatability, and artifact-determinism evidence.
- Keep package anecdotes separate from package-compatibility claims.

## Exit gate

- Optimizations preserve conformance, sandbox, schema, and proof-boundary tests.
- PGO output is deterministic.
- Public performance claims are workload-specific and evidence-backed.
