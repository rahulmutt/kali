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
- Progress: the current-mode inventory now has a checked-in snapshot in [`optimization-inventory.md`](./optimization-inventory.md), and the fast-mode minimality path now has direct optimizer-level regression anchors for literal binary expressions and object-enumeration calls over literal shapes so the inventory is backed by both docs and tests.
- Current progress: the optimization inventory snapshot is now also pinned by the schema-doc drift net in `crates/kali_cli/tests/schema_docs.rs`, so the checked-in mode/evidence summary stays deterministic alongside the existing optimizer and benchmark coverage.
- Current progress: release-mode object-enumeration folding now has direct optimizer-level regression anchors for `Object.keys()`, `Object.entries()`, and `Object.values()` over literal object shapes, and release-advanced now also has matching regression anchors for the same literal-shape enumeration forms, so the current inventory keeps the three supported enumeration forms backed by tests in both optimization tiers rather than only by the runtime-smoke layer.
- Current progress: the same release and release-advanced object-enumeration folding path now also covers const-bound literal aliases, so `Object.keys()` / `Object.entries()` / `Object.values()` stay folded when the literal object is referenced through a `const` binding, and the alias-chain regression now also survives a second `const` alias hop before enumeration so the folding path stays honest through a deeper rebinding chain.
- Current progress: release-advanced now also has a nullish literal-argument specialization regression, keeping the higher optimization tier aligned with the literal-argument specialization family already exercised in release mode.
- Current progress: fast mode now also keeps nullish literal-argument callsites unspecialized, so the nullish specialization family remains gated to the release tiers while the fast-mode minimality baseline stays explicit.

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
- Progress: the optimization benchmark smoke now uses checked-in, hash-validated fixture workloads (`math-benchmark-v1.ts` / `math-benchmark-v1.json`, `math-trunc-benchmark-v1.ts` / `math-trunc-benchmark-v1.json`, `call-inlining-benchmark-v1.ts` / `call-inlining-benchmark-v1.json`, `closure-inlining-benchmark-v1.ts` / `closure-inlining-benchmark-v1.json`, `object-enumeration-benchmark-v1.ts` / `object-enumeration-benchmark-v1.json`, `object-enumeration-alias-chain-benchmark-v1.ts` / `object-enumeration-alias-chain-benchmark-v1.json`, `identity-chain-benchmark-v1.ts` / `identity-chain-benchmark-v1.json`, `nested-wrapper-pruning-benchmark-v1.ts` / `nested-wrapper-pruning-benchmark-v1.json`, `algebraic-simplification-benchmark-v1.ts` / `algebraic-simplification-benchmark-v1.json`, `nullish-specialization-repeat-benchmark-v1.ts` / `nullish-specialization-repeat-benchmark-v1.json`, `specialization-reuse-benchmark-v1.ts` / `specialization-reuse-benchmark-v1.json`, `const-array-element-access-benchmark-v1.ts` / `const-array-element-access-benchmark-v1.json`, `const-object-property-access-benchmark-v1.ts` / `const-object-property-access-benchmark-v1.json`, and `nullish-benchmark-v1.ts` / `nullish-benchmark-v1.json`) so the compile-time size/speed comparison lane is pinned to deterministic inputs instead of an ad-hoc inline source string, the object-enumeration alias-chain workload keeps the rebinding path visible in the benchmark lane, the new const-array element access workload keeps the array-specialization path visible in the benchmark lane, the new const-object property access workload keeps the property-specialization path visible in the benchmark lane, the new Math.trunc workload keeps the built-in math lowering path visible in the benchmark lane, and the runtime-smoke benchmark loop now exercises the specialization-reuse workload too.

## Exit gate

- Optimizations preserve all conformance and sandbox tests.
- PGO output is deterministic.
- Performance claims name workload, build mode, baseline, and evidence.
