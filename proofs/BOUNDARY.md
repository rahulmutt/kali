# Proof Boundary Manifest

Status: **placeholder proof-boundary manifest**.

This file is the canonical repository location for Kali's published **proof-boundary manifest**. Current proof-status summaries in `README.md`, `SPEC.md`, `specs/16-testing.md`, `specs/17-verification.md`, and `specs/19-feature-maturity.md` should point here instead of paraphrasing repository proof coverage from memory.

Current repository-state note:
- follow the shared **current-repository-state vs target-contract reading** from [SPEC.md](../SPEC.md): beyond this manifest itself, there is not yet a checked-in Lean proof source tree under `proofs/`
- the illustrative Lean project layout in [specs/17-verification.md](../specs/17-verification.md) is therefore a target layout for when proofs land, not a claim about the current repository contents
- while that remains true, this published manifest is the required verification artifact for the repository's current **proof-ready** baseline

Canonical verification state (following the shared **proof-ready vs proof-backed split** from [SPEC.md](../SPEC.md)):

| Item | Current state |
|---|---|
| proof-ready | **yes** — this manifest exists, truthfully declares the current claim boundary, and publishes the repository's current proof-CI trigger policy |
| proof-backed | **no** — the modeled boundary is still empty, so no release may market formal verification as a shipped Kali capability yet |
| repository claim | **no mechanized proof coverage is claimed yet** |
| canonical short summary | **Kali is proof-ready, not proof-backed; no mechanized proof coverage is claimed yet.** |

Release rule:
- this **placeholder proof-boundary manifest** is acceptable while the project is still iterating on the spec/implementation because it satisfies the Phase-1 **proof-ready** baseline without overclaiming proof coverage
- before any release markets formal verification as a shipped Kali capability, this manifest must move beyond the **placeholder proof-boundary manifest** state with at least one concrete modeled subsystem plus named theorem/property claims so the release becomes **proof-backed**
- until this manifest names a concrete modeled subsystem, [specs/17-verification.md](../specs/17-verification.md)'s **First proof-backed milestone** section is the planning source of truth for the first non-placeholder scope
- once this manifest becomes non-empty, it becomes the canonical published scope and should either mirror that milestone explicitly or point back to it, so the chapter-level plan and the manifest do not drift apart

Promotion checklist from proof-ready to proof-backed:
- name at least one concrete modeled subsystem rather than leaving the boundary empty
- list the theorem/property inventory explicitly (for example progress, preservation, conservative effect soundness, sandbox-policy soundness)
- name the covered implementation/spec paths that those proofs are intended to constrain
- update CI wiring so proof jobs trigger for those covered paths in addition to `proofs/`

Boundary-maintenance rule:
- once **Covered implementation/spec paths** becomes non-empty, a change to any covered path must land with one of these outcomes in the same PR: (a) matching Lean/model/proof updates, or (b) an explicit narrowing of the published boundary before the implementation/spec change lands
- widening the boundary also requires updating the named theorem/property inventory; new Lean files alone do not widen the claim surface
- release/support wording must always follow this file's current boundary immediately after such a change

Until Lean proofs land, this file should be kept explicit rather than omitted so Kali does not accidentally imply broader formal-verification coverage than it actually has.

## Modeled boundary
- No subsystem is yet claimed as mechanically proved in this repository.
- Recommended first non-placeholder scope once proofs start: reuse [specs/17-verification.md](../specs/17-verification.md)'s **First proof-backed milestone** section as the planning source of truth until this manifest becomes non-empty, rather than restating a second near-duplicate checklist here.

## Proof-CI trigger policy
- The current proof boundary is **empty**.
- Therefore proof CI is required only for changes under `proofs/`.
- If this manifest later names covered implementation/spec subsystems, proof CI must also trigger for changes to those covered areas.
- Until concrete CI workflow files exist, this section is the repository's normative proof-CI trigger policy rather than evidence that hosted proof automation is already configured.
- Until that happens, no release note, README text, or phase summary should imply mechanized coverage for any implementation subsystem.
- Operational rule: run proof CI when either condition becomes true:
  1. files under `proofs/` change, or
  2. a change touches a subsystem explicitly listed in **Covered implementation/spec paths** as inside the modeled boundary.
- Until a non-empty modeled boundary is published, only condition (1) is active.
- The absence of broader proof jobs must not be described as proof coverage.

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

