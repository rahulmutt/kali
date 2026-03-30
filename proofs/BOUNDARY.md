# Proof Boundary Manifest

Status: placeholder for the initial published proof boundary. The repository currently claims **no mechanized proof coverage yet**, so this file documents the absence of proof claims explicitly.

Canonical verification state:
- **proof-ready**: yes — this published manifest exists and defines the current activation/claim boundary
- **proof-backed**: no — the modeled boundary is still empty, so no release may market formal verification as a shipped Kali capability yet

Release note:
- this placeholder is acceptable while the project is still iterating on the spec/implementation because it satisfies the Phase-1 **proof-ready** baseline without overclaiming proof coverage
- before any release markets formal verification as a shipped Kali capability, this manifest should be replaced with at least one concrete modeled subsystem plus named theorem/property claims so the release becomes **proof-backed**
- the first non-placeholder scope should mirror [specs/17-verification.md](../specs/17-verification.md)'s **First proof-backed milestone** section or point to an explicitly documented equivalent, so the chapter-level plan and this manifest do not drift apart

Promotion checklist from proof-ready to proof-backed:
- name at least one concrete modeled subsystem rather than leaving the boundary empty;
- list the theorem/property inventory explicitly (for example progress, preservation, conservative effect soundness, sandbox-policy soundness);
- name the covered implementation/spec paths that those proofs are intended to constrain;
- update CI wiring so proof jobs trigger for those covered paths in addition to `proofs/`.

This file is the canonical repository location for the **proof-boundary manifest** referenced by:
- `SPEC.md`
- `specs/17-verification.md`
- `specs/16-testing.md`
- `specs/19-feature-maturity.md`

Until Lean proofs land, this file should be kept explicit rather than omitted so Kali does not accidentally imply broader formal-verification coverage than it actually has.

## Modeled boundary
- No subsystem is yet claimed as mechanically proved in this repository.
- Recommended first non-placeholder scope once proofs start:
  - the core typed/effectful calculus fragment,
  - progress + preservation for that fragment,
  - conservative built-in effect-soundness theorems for the modeled capability subset,
  - and the declarative sandbox-policy decision/enforcement theorem family for that same subset.

## Current activation state
- The current proof boundary is **empty**.
- Therefore proof CI is required only for changes under `proofs/`.
- If this manifest later names covered implementation/spec subsystems, proof CI must also trigger for changes to those covered areas.
- Until concrete CI workflow files exist, this section is the repository's normative proof-CI trigger policy rather than evidence that hosted proof automation is already configured.
- Until that happens, no release note, README text, or phase summary should imply mechanized coverage for any implementation subsystem.

## Claimed theorems/properties
- None yet.

## Trusted assumptions
- All current implementation/spec behavior remains outside the mechanically proved set.
- Ordinary testing, review, and spec conformance remain the active evidence sources until proofs are added.

## Explicitly unmodeled features
- Full ECMAScript/TypeScript surface semantics
- Host integrations and OS behavior
- Dynamic compatibility features such as `eval` / `Function()`
- Browser/Node compatibility layers beyond the future modeled core
- Full lowering/codegen correctness end to end

## Covered implementation/spec paths
- None yet.
- Once the manifest becomes non-empty, list the exact Rust crate/spec chapter/path set whose behavior is being claimed against the Lean model so CI wiring and release wording can reference one canonical path inventory.

## Required implementation/spec alignment scope
- None yet beyond keeping this file honest.
- Once covered paths exist, this section should summarize the specific implementation/spec correspondence obligations for those paths rather than relying on implicit reviewer memory.

## CI trigger rule
Run proof CI when either condition becomes true:
1. files under `proofs/` change, or
2. a change touches a subsystem explicitly listed in **Covered implementation/spec paths** as inside the modeled boundary.

Until a non-empty modeled boundary is published, only condition (1) is active. The absence of broader proof jobs must not be described as proof coverage.
