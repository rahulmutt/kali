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
- Current progress: release-mode object-enumeration folding now has direct optimizer-level regression anchors for `Object.keys()`, `Object.entries()`, and `Object.values()` over literal object shapes, and release-advanced now also has matching regression anchors for the same literal-shape enumeration forms, so the current inventory keeps the three supported enumeration forms backed by tests in both optimization tiers rather than only by the runtime-smoke layer.
- Current progress: the same release and release-advanced object-enumeration folding path now also covers const-bound literal aliases, so `Object.keys()` / `Object.entries()` / `Object.values()` stay folded when the literal object is referenced through a `const` binding.

### 9.2 Specialization depth

- Improve layout/representation fingerprinting.
- Respect `--max-specializations` exactly as an upper bound.
- Ensure fallback paths preserve JavaScript semantics.
- Progress: zero-budget tagged-parameter MIR specialization now has a regression that keeps the original call target in place and prevents speculative `add_pair$spec$...` clones.
- Progress: release-advanced now also has a cap-exactness regression that shows duplicate root-call shapes can inline away without consuming the single specialization slot, while the remaining distinct call shape still produces exactly one MIR-specialized clone once the cap is reached.

### 9.3 PGO input

- Promote deterministic build-only `--profile` support only with strict schema validation.
- Keep PGO as an additive build input, not a fourth build mode.
- Add malformed/unknown-field rejection tests.
- Progress: CLI integration now rejects malformed PGO profile payloads with version, unknown-field, and top-level shape validation in both text and JSON build output modes.

### 9.4 Benchmark lane

- Add version-pinned benchmarks and fixtures.
- Require repeatability before any public performance wording changes.
- Keep package benchmark anecdotes separate from compatibility claims.
- Progress: the optimization benchmark smoke now uses checked-in, hash-validated fixture pairs (`math-benchmark-v1.ts` / `math-benchmark-v1.json` and `call-inlining-benchmark-v1.ts` / `call-inlining-benchmark-v1.json`) so the compile-time size/speed comparison lane is pinned to deterministic inputs instead of an ad-hoc inline source string.

## Exit gate

- Optimizations preserve all conformance and sandbox tests.
- PGO output is deterministic.
- Performance claims name workload, build mode, baseline, and evidence.
