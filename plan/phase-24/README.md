# Phase 24 — Optimization and Performance Evidence

## Goal

Turn optimization and PGO work into deterministic, evidence-backed performance claims without changing observable semantics.

## Owning specs

- `specs/07-specialization.md`
- `specs/08-wasm-codegen.md`
- `specs/16-testing.md`
- `specs/18-schemas.md`
- `specs/19-feature-maturity.md`

## Work packets

### 24.1 Optimization inventory upkeep

- Keep a concise current-evidence inventory of real `fast`, `release`, and `release-advanced` behavior.
- The benchmark fixture set now also tracks `math-pow-builtin-js` alongside `math-pow-builtin` so `Math.pow` evidence stays visible in both TS and JS workload forms, `math-abs-sign-builtin-js` now mirrors the `Math.abs` / `Math.sign` slice into the JS workload form too, the new `math-trunc-builtin-js` / `math-ceil-builtin-js` fixtures now mirror the `Math.trunc` / `Math.ceil` slices into JS as well, the new `math-floor-builtin` / `math-floor-builtin-js` pair now keeps the `Math.floor` workload visible in both source classes, the new `math-round-builtin` / `math-round-builtin-js` pair now keeps the `Math.round` workload visible in both source classes, the `math-imul` slice now also has a JS workload form (`math-imul-builtin-js`) so the integer-math inventory stays paired across source classes, and the generic `folded-arithmetic` benchmark now also has a JS workload form (`folded-arithmetic-js`) so the baseline arithmetic inventory stays paired across source classes.
- Update the inventory only when tests prove a mode's behavior.
- Preserve deterministic artifacts and schema-v1 output contracts.

### 24.2 Specialization depth

- Improve layout, representation, call-shape, object-shape, builtin-folding, and cross-module specialization in claim-aligned slices.
- Treat `--max-specializations` as an exact upper bound.
- Preserve fallback paths and JavaScript-visible semantics.
- Keep optimization tests coupled to conformance and sandbox/effect expectations where observable behavior could change.

### 24.3 PGO input hardening

- Keep `--profile` as deterministic build-only additive input.
- Reject malformed, unknown-field, version-mismatched, and nondeterministic profile data.
- Do not create a fourth build-mode vocabulary.
- Current progress: empty and whitespace-only `--profile` files now fail with the canonical `E5509` parse path in both human and JSON `build` output, alongside the existing malformed/version/unknown-field coverage.

### 24.4 Benchmark promotion

- Use version-pinned workload fixtures with hash validation.
- Promote performance wording only with workload, build mode, baseline, repeatability, and artifact-determinism evidence.
- Keep package anecdotes separate from package-compatibility claims.

## Exit gate

- Optimizations preserve conformance, sandbox, schema, and proof-boundary tests.
- PGO output is deterministic.
- Public performance claims are workload-specific and evidence-backed.
