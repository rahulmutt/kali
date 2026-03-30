# Proof Boundary Manifest

Status: placeholder for the initial published proof boundary. The repository currently claims **no mechanized proof coverage yet**, so this file documents the absence of proof claims rather than satisfying the future Phase-1 proof milestone by itself.

Release note:
- this placeholder is acceptable while the project is still iterating on the spec/implementation without making proof-backed support claims
- before any release markets formal verification as a shipped Kali capability, this manifest should be replaced with at least one concrete modeled subsystem plus named theorem/property claims

This file is the canonical repository location for the **proof-boundary manifest** referenced by:
- `SPEC.md`
- `specs/17-verification.md`
- `specs/16-testing.md`
- `specs/19-feature-maturity.md`

Until Lean proofs land, this file should be kept explicit rather than omitted so Kali does not accidentally imply broader formal-verification coverage than it actually has.

## Modeled boundary
- No subsystem is yet claimed as mechanically proved in this repository.
- Planned initial focus once proofs start: the core typed/effectful calculus plus the declarative sandbox-policy decision procedure.

## Current activation state
- The current proof boundary is **empty**.
- Therefore proof CI is required only for changes under `proofs/`.
- If this manifest later names covered implementation/spec subsystems, proof CI must also trigger for changes to those covered areas.
- Until that happens, no release note, README text, or phase summary should imply mechanized coverage for any implementation subsystem.

## Claimed theorems/properties
- None yet.

## Trusted assumptions
- All current implementation/spec behavior remains outside the mechanically proved set.
- Ordinary testing, review, and spec conformance remain the active evidence sources until proofs are added.

## Explicitly unmodeled features
- Full ECMAScript/TypeScript surface semantics
- Host integrations and OS behavior
- Dynamic compatibility features such as `eval`
- Browser/Node compatibility layers beyond the future modeled core
- Full lowering/codegen correctness end to end

## Required implementation/spec alignment scope
- None yet beyond keeping this file honest.

## CI trigger rule
Run proof CI when either condition becomes true:
1. files under `proofs/` change, or
2. a change touches a subsystem explicitly listed here as inside the modeled boundary.

Until a non-empty modeled boundary is published, only condition (1) is active. The absence of broader proof jobs must not be described as proof coverage.
