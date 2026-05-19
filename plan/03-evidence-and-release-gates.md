# Evidence and Release Gates

A feature is not supportable merely because code exists. Promotion requires evidence matching the claim.

## Common gate for every packet

- `cargo build --workspace`
- `cargo test --workspace`
- command-specific integration coverage
- deterministic JSON/schema coverage for machine-readable output
- docs/spec/maturity updates when public behavior changes
- proof-boundary updates when verification wording changes

## Evidence by claim type

| Claim type | Required evidence |
|---|---|
| Language syntax/semantics | conformance fixtures, minimized regressions, parser/checker/runtime tests |
| Type-system behavior | checker baselines for TS and JS inference, negative diagnostics, stable snapshots |
| Runtime/host API | integration tests, sandbox enforcement tests, resource-budget tests |
| Browser runtime | real browser harness smoke tests, not only mocks or browser-targeted bundle checks |
| Package compatibility | package-corpus results by package, command, API surface, and support rung |
| Registry analysis | target-shape negatives, explicit package-version rejection, deterministic version selection, schema-v1 JSON output |
| Optimization | before/after correctness tests, deterministic artifacts, benchmark harness results |
| PGO | strict profile schema tests, deterministic profile consumption, no build-mode vocabulary drift |
| Verification | mechanized proofs, `proofs/BOUNDARY.md` theorem/property inventory, proof CI trigger policy |

## Promotion rule

Before changing support wording:

1. identify the exact availability context;
2. check `specs/19-feature-maturity.md`;
3. add evidence for that context;
4. update owning specs and schemas;
5. update README only after specs/maturity are aligned.

## Rejection discipline

Unsupported but documented surfaces should fail explicitly with the canonical diagnostic path from `specs/15-errors.md`, typically:

- `E5506` for real but unavailable command/profile/feature gates;
- `E5508` for invalid command shape, arity, or contradictory flags;
- `E6004` for missing/stale dependency materialization;
- `E5511` for unavailable statically known export surface on library-oriented builds.
- package-audit preview-shim regressions now also confirm the legacy `--preview` gate wins before malformed-target validation or registry lookup in JSON mode, including a valid single-target JSON-mode invocation so the registry never gets consulted on that path, keeping the command-shape boundary explicit.
