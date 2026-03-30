# Proof Boundary Manifest

Status: placeholder for the initial published proof boundary.

This file is the canonical repository location for the **proof-boundary manifest** referenced by:
- `SPEC.md`
- `specs/17-verification.md`
- `specs/16-testing.md`
- `specs/19-feature-maturity.md`

Until Lean proofs land, this file should be kept explicit rather than omitted so Kali does not accidentally imply broader formal-verification coverage than it actually has.

## Modeled boundary
- No subsystem is yet claimed as mechanically proved in this repository.
- Planned initial focus: the core typed/effectful calculus plus the declarative sandbox-policy decision procedure.

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
2. a change touches a subsystem later listed here as inside the modeled boundary.

Until then, the absence of proof jobs must not be described as proof coverage.
